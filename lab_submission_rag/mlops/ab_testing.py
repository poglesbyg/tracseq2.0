"""
A/B Testing Framework for TracSeq 2.0 MLOps Pipeline

Enables controlled model rollouts, statistical significance testing, and performance monitoring.
"""

import hashlib
import uuid
from dataclasses import asdict, dataclass, field
from datetime import datetime, timedelta
from enum import Enum
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple, Union

import numpy as np
import structlog
from scipy import stats
from sqlalchemy import JSON, Boolean, Column, DateTime, Float, Integer, String, Text, create_engine
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker

logger = structlog.get_logger(__name__)


class TestStatus(Enum):
    PLANNING = "planning"
    RUNNING = "running"
    PAUSED = "paused"
    COMPLETED = "completed"
    CANCELLED = "cancelled"


class TestType(Enum):
    CHAMPION_CHALLENGER = "champion_challenger"
    MULTIVARIATE = "multivariate"
    GRADUAL_ROLLOUT = "gradual_rollout"
    CANARY = "canary"


@dataclass
class ABTestConfig:
    """A/B test configuration"""

    test_id: str
    name: str
    description: str
    test_type: TestType

    # Models being tested
    control_model_id: str
    control_model_version: str
    treatment_models: List[Dict[str, str]]  # [{"model_id": "...", "version": "..."}]

    # Traffic allocation
    traffic_allocation: Dict[str, float]  # {"control": 0.5, "treatment_1": 0.3, "treatment_2": 0.2}

    # Test parameters
    hypothesis: str
    primary_metric: str
    secondary_metrics: List[str] = field(default_factory=list)
    minimum_detectable_effect: float = 0.05  # 5% improvement
    confidence_level: float = 0.95
    statistical_power: float = 0.8

    # Duration and sample size
    planned_duration_days: int = 14
    minimum_sample_size: int = 1000
    maximum_sample_size: Optional[int] = None

    # Filtering criteria
    user_filters: Dict[str, Any] = field(default_factory=dict)
    feature_flags: List[str] = field(default_factory=list)

    # Safety controls
    guardrail_metrics: List[str] = field(default_factory=list)
    error_rate_threshold: float = 0.1
    latency_threshold_ms: float = 1000

    # Metadata
    created_at: datetime = field(default_factory=datetime.utcnow)
    created_by: str = ""
    start_date: Optional[datetime] = None
    end_date: Optional[datetime] = None
    status: TestStatus = TestStatus.PLANNING


@dataclass
class ABTestResult:
    """A/B test results and statistical analysis"""

    test_id: str
    variant_id: str

    # Sample statistics
    sample_size: int
    conversion_rate: Optional[float] = None
    mean_value: Optional[float] = None
    std_deviation: Optional[float] = None

    # Performance metrics
    accuracy: Optional[float] = None
    precision: Optional[float] = None
    recall: Optional[float] = None
    f1_score: Optional[float] = None

    # Custom metrics
    custom_metrics: Dict[str, float] = field(default_factory=dict)

    # System performance
    average_latency_ms: float = 0.0
    error_rate: float = 0.0
    throughput_rps: float = 0.0

    # Statistical significance
    p_value: Optional[float] = None
    confidence_interval: Optional[Tuple[float, float]] = None
    effect_size: Optional[float] = None
    is_statistically_significant: bool = False

    # Timestamps
    calculated_at: datetime = field(default_factory=datetime.utcnow)


@dataclass
class ABTestInteraction:
    """Individual user interaction in A/B test"""

    interaction_id: str
    test_id: str
    variant_id: str
    user_id: str

    # Request details
    request_data: Dict[str, Any]
    response_data: Dict[str, Any]

    # Performance metrics
    latency_ms: float
    success: bool
    error_message: Optional[str] = None

    # Business metrics
    conversion: Optional[bool] = None
    value: Optional[float] = None

    # Metadata
    timestamp: datetime = field(default_factory=datetime.utcnow)
    user_agent: Optional[str] = None
    ip_address: Optional[str] = None


Base = declarative_base()


class ABTestRecord(Base):
    """Database model for A/B tests"""

    __tablename__ = "ab_tests"

    test_id = Column(String, primary_key=True)
    name = Column(String, nullable=False)
    description = Column(Text)
    test_type = Column(String, nullable=False)

    # Models
    control_model_id = Column(String, nullable=False)
    control_model_version = Column(String, nullable=False)
    treatment_models = Column(JSON)

    # Configuration
    traffic_allocation = Column(JSON)
    hypothesis = Column(Text)
    primary_metric = Column(String)
    secondary_metrics = Column(JSON)
    minimum_detectable_effect = Column(Float)
    confidence_level = Column(Float)
    statistical_power = Column(Float)

    # Duration
    planned_duration_days = Column(Integer)
    minimum_sample_size = Column(Integer)
    maximum_sample_size = Column(Integer)

    # Filters
    user_filters = Column(JSON)
    feature_flags = Column(JSON)

    # Safety
    guardrail_metrics = Column(JSON)
    error_rate_threshold = Column(Float)
    latency_threshold_ms = Column(Float)

    # Status and timing
    status = Column(String, default=TestStatus.PLANNING.value)
    created_at = Column(DateTime, default=datetime.utcnow)
    created_by = Column(String)
    start_date = Column(DateTime)
    end_date = Column(DateTime)

    # Results
    final_results = Column(JSON)
    winner_variant = Column(String)
    statistical_significance = Column(Boolean)


class ABTestInteractionRecord(Base):
    """Database model for test interactions"""

    __tablename__ = "ab_test_interactions"

    interaction_id = Column(String, primary_key=True)
    test_id = Column(String, nullable=False)
    variant_id = Column(String, nullable=False)
    user_id = Column(String, nullable=False)

    # Request/response
    request_data = Column(JSON)
    response_data = Column(JSON)

    # Performance
    latency_ms = Column(Float)
    success = Column(Boolean)
    error_message = Column(Text)

    # Business metrics
    conversion = Column(Boolean)
    value = Column(Float)

    # Metadata
    timestamp = Column(DateTime, default=datetime.utcnow)
    user_agent = Column(String)
    ip_address = Column(String)


class ABTestManager:
    """
    Comprehensive A/B testing framework for ML models.

    Features:
    - Multi-variant testing (A/B/C/D/...)
    - Statistical significance testing
    - Gradual traffic ramp-up
    - Real-time monitoring and guardrails
    - Automated test conclusion
    """

    def __init__(self, database_url: str, results_dir: Union[str, Path]):
        self.results_dir = Path(results_dir)
        self.results_dir.mkdir(parents=True, exist_ok=True)

        # Database setup
        self.engine = create_engine(database_url)
        Base.metadata.create_all(self.engine)
        self.SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=self.engine)

        self.active_tests: Dict[str, ABTestConfig] = {}
        self._load_active_tests()

    async def create_test(self, config: ABTestConfig) -> str:
        """Create a new A/B test."""
        if not config.test_id:
            config.test_id = f"test_{uuid.uuid4().hex[:8]}"

        # Validate configuration
        await self._validate_test_config(config)

        # Calculate required sample size
        if config.minimum_sample_size == 1000:  # Default value
            config.minimum_sample_size = self._calculate_sample_size(
                config.minimum_detectable_effect, config.confidence_level, config.statistical_power
            )

        # Store in database
        with self.SessionLocal() as session:
            record = ABTestRecord(
                test_id=config.test_id,
                name=config.name,
                description=config.description,
                test_type=config.test_type.value,
                control_model_id=config.control_model_id,
                control_model_version=config.control_model_version,
                treatment_models=config.treatment_models,
                traffic_allocation=config.traffic_allocation,
                hypothesis=config.hypothesis,
                primary_metric=config.primary_metric,
                secondary_metrics=config.secondary_metrics,
                minimum_detectable_effect=config.minimum_detectable_effect,
                confidence_level=config.confidence_level,
                statistical_power=config.statistical_power,
                planned_duration_days=config.planned_duration_days,
                minimum_sample_size=config.minimum_sample_size,
                maximum_sample_size=config.maximum_sample_size,
                user_filters=config.user_filters,
                feature_flags=config.feature_flags,
                guardrail_metrics=config.guardrail_metrics,
                error_rate_threshold=config.error_rate_threshold,
                latency_threshold_ms=config.latency_threshold_ms,
                status=config.status.value,
                created_at=config.created_at,
                created_by=config.created_by,
                start_date=config.start_date,
                end_date=config.end_date,
            )
            session.add(record)
            session.commit()

        logger.info(
            "A/B test created",
            test_id=config.test_id,
            name=config.name,
            test_type=config.test_type.value,
        )

        return config.test_id

    async def start_test(self, test_id: str) -> bool:
        """Start an A/B test."""
        config = await self.get_test_config(test_id)
        if not config:
            logger.error("Test not found", test_id=test_id)
            return False

        if config.status != TestStatus.PLANNING:
            logger.error("Test not in planning state", test_id=test_id, status=config.status)
            return False

        # Update status and start date
        config.status = TestStatus.RUNNING
        config.start_date = datetime.utcnow()
        config.end_date = config.start_date + timedelta(days=config.planned_duration_days)

        await self._update_test_status(test_id, config.status, config.start_date, config.end_date)

        # Load into memory for fast access
        self.active_tests[test_id] = config

        logger.info(
            "A/B test started",
            test_id=test_id,
            start_date=config.start_date,
            end_date=config.end_date,
        )

        return True

    async def assign_variant(self, test_id: str, user_id: str) -> Optional[str]:
        """Assign a user to a test variant."""
        if test_id not in self.active_tests:
            return None

        config = self.active_tests[test_id]

        # Apply user filters
        if not await self._user_matches_filters(user_id, config.user_filters):
            return None

        # Deterministic assignment based on user ID and test ID
        assignment_hash = hashlib.md5(f"{test_id}:{user_id}".encode()).hexdigest()
        assignment_value = int(assignment_hash[:8], 16) / (16**8)  # Convert to 0-1 range

        # Determine variant based on traffic allocation
        cumulative_probability = 0.0
        for variant, probability in config.traffic_allocation.items():
            cumulative_probability += probability
            if assignment_value <= cumulative_probability:
                return variant

        return "control"  # Fallback

    async def log_interaction(self, interaction: ABTestInteraction):
        """Log a user interaction in an A/B test."""
        # Store in database
        with self.SessionLocal() as session:
            record = ABTestInteractionRecord(
                interaction_id=interaction.interaction_id,
                test_id=interaction.test_id,
                variant_id=interaction.variant_id,
                user_id=interaction.user_id,
                request_data=interaction.request_data,
                response_data=interaction.response_data,
                latency_ms=interaction.latency_ms,
                success=interaction.success,
                error_message=interaction.error_message,
                conversion=interaction.conversion,
                value=interaction.value,
                timestamp=interaction.timestamp,
                user_agent=interaction.user_agent,
                ip_address=interaction.ip_address,
            )
            session.add(record)
            session.commit()

        # Check guardrails
        await self._check_guardrails(interaction.test_id)

    async def calculate_results(self, test_id: str) -> Dict[str, ABTestResult]:
        """Calculate current test results with statistical analysis."""
        config = await self.get_test_config(test_id)
        if not config:
            raise ValueError(f"Test {test_id} not found")

        # Get all interactions
        with self.SessionLocal() as session:
            interactions = (
                session.query(ABTestInteractionRecord)
                .filter(ABTestInteractionRecord.test_id == test_id)
                .all()
            )

        # Group by variant
        variant_data = {}
        for interaction in interactions:
            variant = interaction.variant_id
            if variant not in variant_data:
                variant_data[variant] = []
            variant_data[variant].append(interaction)

        # Calculate results for each variant
        results = {}
        control_result = None

        for variant_id, variant_interactions in variant_data.items():
            result = self._calculate_variant_results(variant_id, variant_interactions, config)
            results[variant_id] = result

            if variant_id == "control":
                control_result = result

        # Calculate statistical significance vs control
        if control_result:
            for variant_id, result in results.items():
                if variant_id != "control":
                    self._calculate_statistical_significance(
                        control_result, result, config.primary_metric
                    )

        # Store results
        await self._store_test_results(test_id, results)

        return results

    async def check_for_early_stopping(self, test_id: str) -> bool:
        """Check if test should be stopped early based on statistical significance."""
        results = await self.calculate_results(test_id)
        config = await self.get_test_config(test_id)

        if not config or len(results) < 2:
            return False

        # Check minimum sample size reached
        total_samples = sum(result.sample_size for result in results.values())
        if total_samples < config.minimum_sample_size:
            return False

        # Check statistical significance
        for variant_id, result in results.items():
            if variant_id != "control" and result.is_statistically_significant:
                logger.info(
                    "Early stopping criteria met",
                    test_id=test_id,
                    variant_id=variant_id,
                    p_value=result.p_value,
                )
                return True

        return False

    async def stop_test(self, test_id: str, reason: str = "Manual stop") -> bool:
        """Stop an A/B test."""
        config = await self.get_test_config(test_id)
        if not config:
            return False

        # Calculate final results
        final_results = await self.calculate_results(test_id)

        # Determine winner
        winner_variant = self._determine_winner(final_results, config.primary_metric)

        # Update database
        with self.SessionLocal() as session:
            record = session.query(ABTestRecord).filter(ABTestRecord.test_id == test_id).first()

            if record:
                record.status = TestStatus.COMPLETED.value
                record.end_date = datetime.utcnow()
                record.final_results = {
                    variant_id: asdict(result) for variant_id, result in final_results.items()
                }
                record.winner_variant = winner_variant
                record.statistical_significance = any(
                    result.is_statistically_significant for result in final_results.values()
                )
                session.commit()

        # Remove from active tests
        if test_id in self.active_tests:
            del self.active_tests[test_id]

        logger.info(
            "A/B test completed", test_id=test_id, winner_variant=winner_variant, reason=reason
        )

        return True

    async def get_test_config(self, test_id: str) -> Optional[ABTestConfig]:
        """Get test configuration."""
        with self.SessionLocal() as session:
            record = session.query(ABTestRecord).filter(ABTestRecord.test_id == test_id).first()

            if not record:
                return None

            return ABTestConfig(
                test_id=record.test_id,
                name=record.name,
                description=record.description,
                test_type=TestType(record.test_type),
                control_model_id=record.control_model_id,
                control_model_version=record.control_model_version,
                treatment_models=record.treatment_models,
                traffic_allocation=record.traffic_allocation,
                hypothesis=record.hypothesis,
                primary_metric=record.primary_metric,
                secondary_metrics=record.secondary_metrics or [],
                minimum_detectable_effect=record.minimum_detectable_effect,
                confidence_level=record.confidence_level,
                statistical_power=record.statistical_power,
                planned_duration_days=record.planned_duration_days,
                minimum_sample_size=record.minimum_sample_size,
                maximum_sample_size=record.maximum_sample_size,
                user_filters=record.user_filters or {},
                feature_flags=record.feature_flags or [],
                guardrail_metrics=record.guardrail_metrics or [],
                error_rate_threshold=record.error_rate_threshold,
                latency_threshold_ms=record.latency_threshold_ms,
                created_at=record.created_at,
                created_by=record.created_by,
                start_date=record.start_date,
                end_date=record.end_date,
                status=TestStatus(record.status),
            )

    async def list_tests(
        self, status: Optional[TestStatus] = None, limit: int = 50
    ) -> List[Dict[str, Any]]:
        """List A/B tests with optional filtering."""
        with self.SessionLocal() as session:
            query = session.query(ABTestRecord)

            if status:
                query = query.filter(ABTestRecord.status == status.value)

            query = query.order_by(ABTestRecord.created_at.desc()).limit(limit)
            records = query.all()

            tests = []
            for record in records:
                tests.append(
                    {
                        "test_id": record.test_id,
                        "name": record.name,
                        "test_type": record.test_type,
                        "status": record.status,
                        "created_at": record.created_at,
                        "start_date": record.start_date,
                        "end_date": record.end_date,
                        "winner_variant": record.winner_variant,
                        "statistical_significance": record.statistical_significance,
                    }
                )

            return tests

    def _calculate_sample_size(
        self, effect_size: float, confidence_level: float, power: float
    ) -> int:
        """Calculate required sample size for statistical power."""
        # Simplified calculation - in practice, use more sophisticated methods
        z_alpha = stats.norm.ppf(1 - (1 - confidence_level) / 2)
        z_beta = stats.norm.ppf(power)

        # Assuming binomial distribution with baseline conversion rate of 0.1
        p1 = 0.1
        p2 = p1 * (1 + effect_size)

        pooled_p = (p1 + p2) / 2

        sample_size = (
            (
                z_alpha * np.sqrt(2 * pooled_p * (1 - pooled_p))
                + z_beta * np.sqrt(p1 * (1 - p1) + p2 * (1 - p2))
            )
            ** 2
        ) / (p2 - p1) ** 2

        return max(1000, int(sample_size))

    def _calculate_variant_results(
        self, variant_id: str, interactions: List[ABTestInteractionRecord], config: ABTestConfig
    ) -> ABTestResult:
        """Calculate results for a single variant."""
        if not interactions:
            return ABTestResult(test_id=config.test_id, variant_id=variant_id, sample_size=0)

        # Basic statistics
        sample_size = len(interactions)
        successful_interactions = [i for i in interactions if i.success]
        conversions = [i for i in interactions if i.conversion]

        # Performance metrics
        accuracy = len(successful_interactions) / sample_size if sample_size > 0 else 0.0
        conversion_rate = len(conversions) / sample_size if sample_size > 0 else 0.0
        avg_latency = np.mean([i.latency_ms for i in interactions])
        error_rate = 1 - accuracy

        # Custom metrics from response data
        custom_metrics = {}
        if successful_interactions:
            # Extract metrics from response data
            for interaction in successful_interactions:
                if interaction.response_data:
                    for key, value in interaction.response_data.items():
                        if isinstance(value, (int, float)) and key.endswith("_score"):
                            if key not in custom_metrics:
                                custom_metrics[key] = []
                            custom_metrics[key].append(value)

        # Average custom metrics
        for key, values in custom_metrics.items():
            custom_metrics[key] = np.mean(values)

        return ABTestResult(
            test_id=config.test_id,
            variant_id=variant_id,
            sample_size=sample_size,
            conversion_rate=conversion_rate,
            accuracy=accuracy,
            custom_metrics=custom_metrics,
            average_latency_ms=avg_latency,
            error_rate=error_rate,
            calculated_at=datetime.utcnow(),
        )

    def _calculate_statistical_significance(
        self, control: ABTestResult, treatment: ABTestResult, metric: str
    ):
        """Calculate statistical significance using two-proportion z-test."""
        if control.sample_size == 0 or treatment.sample_size == 0:
            return

        # Get metric values
        control_value = getattr(control, metric, None) or control.custom_metrics.get(metric, 0)
        treatment_value = getattr(treatment, metric, None) or treatment.custom_metrics.get(
            metric, 0
        )

        if control_value is None or treatment_value is None:
            return

        # Two-proportion z-test for conversion rates
        if metric in ["conversion_rate", "accuracy"]:
            x1 = int(control_value * control.sample_size)
            n1 = control.sample_size
            x2 = int(treatment_value * treatment.sample_size)
            n2 = treatment.sample_size

            # Pooled proportion
            p_pool = (x1 + x2) / (n1 + n2)
            se = np.sqrt(p_pool * (1 - p_pool) * (1 / n1 + 1 / n2))

            if se > 0:
                z_score = (treatment_value - control_value) / se
                p_value = 2 * (1 - stats.norm.cdf(abs(z_score)))

                treatment.p_value = p_value
                treatment.effect_size = (treatment_value - control_value) / control_value
                treatment.is_statistically_significant = p_value < 0.05

                # Confidence interval
                margin_error = 1.96 * se  # 95% CI
                treatment.confidence_interval = (
                    treatment_value - margin_error,
                    treatment_value + margin_error,
                )

    def _determine_winner(self, results: Dict[str, ABTestResult], metric: str) -> Optional[str]:
        """Determine the winning variant based on the primary metric."""
        if len(results) < 2:
            return None

        best_variant = None
        best_value = float("-inf")

        for variant_id, result in results.items():
            value = getattr(result, metric, None) or result.custom_metrics.get(metric, 0)
            if value and value > best_value:
                best_value = value
                best_variant = variant_id

        return best_variant

    async def _validate_test_config(self, config: ABTestConfig):
        """Validate A/B test configuration."""
        # Check traffic allocation sums to 1.0
        total_traffic = sum(config.traffic_allocation.values())
        if abs(total_traffic - 1.0) > 0.01:
            raise ValueError(f"Traffic allocation must sum to 1.0, got {total_traffic}")

        # Check control variant is included
        if "control" not in config.traffic_allocation:
            raise ValueError("Control variant must be included in traffic allocation")

    async def _update_test_status(
        self,
        test_id: str,
        status: TestStatus,
        start_date: Optional[datetime] = None,
        end_date: Optional[datetime] = None,
    ):
        """Update test status in database."""
        with self.SessionLocal() as session:
            record = session.query(ABTestRecord).filter(ABTestRecord.test_id == test_id).first()

            if record:
                record.status = status.value
                if start_date:
                    record.start_date = start_date
                if end_date:
                    record.end_date = end_date
                session.commit()

    async def _store_test_results(self, test_id: str, results: Dict[str, ABTestResult]):
        """Store test results in database."""
        results_data = {variant_id: asdict(result) for variant_id, result in results.items()}

        with self.SessionLocal() as session:
            record = session.query(ABTestRecord).filter(ABTestRecord.test_id == test_id).first()

            if record:
                record.final_results = results_data
                session.commit()

    async def _check_guardrails(self, test_id: str):
        """Check guardrail metrics and pause test if necessary."""
        config = await self.get_test_config(test_id)
        if not config or not config.guardrail_metrics:
            return

        results = await self.calculate_results(test_id)

        for variant_id, result in results.items():
            # Check error rate
            if result.error_rate > config.error_rate_threshold:
                logger.warning(
                    "Guardrail violation: High error rate",
                    test_id=test_id,
                    variant_id=variant_id,
                    error_rate=result.error_rate,
                    threshold=config.error_rate_threshold,
                )
                await self._pause_test(test_id, f"High error rate in {variant_id}")
                return

            # Check latency
            if result.average_latency_ms > config.latency_threshold_ms:
                logger.warning(
                    "Guardrail violation: High latency",
                    test_id=test_id,
                    variant_id=variant_id,
                    latency=result.average_latency_ms,
                    threshold=config.latency_threshold_ms,
                )
                await self._pause_test(test_id, f"High latency in {variant_id}")
                return

    async def _pause_test(self, test_id: str, reason: str):
        """Pause a test due to guardrail violations."""
        await self._update_test_status(test_id, TestStatus.PAUSED)

        if test_id in self.active_tests:
            self.active_tests[test_id].status = TestStatus.PAUSED

        logger.error("A/B test paused due to guardrail violation", test_id=test_id, reason=reason)

    async def _user_matches_filters(self, user_id: str, filters: Dict[str, Any]) -> bool:
        """Check if user matches test filters."""
        # Implement user filtering logic based on your user data
        # For now, return True (no filtering)
        return True

    def _load_active_tests(self):
        """Load active tests into memory on startup."""
        with self.SessionLocal() as session:
            active_records = (
                session.query(ABTestRecord)
                .filter(ABTestRecord.status == TestStatus.RUNNING.value)
                .all()
            )

            for record in active_records:
                config = ABTestConfig(
                    test_id=record.test_id,
                    name=record.name,
                    description=record.description,
                    test_type=TestType(record.test_type),
                    control_model_id=record.control_model_id,
                    control_model_version=record.control_model_version,
                    treatment_models=record.treatment_models,
                    traffic_allocation=record.traffic_allocation,
                    hypothesis=record.hypothesis,
                    primary_metric=record.primary_metric,
                    secondary_metrics=record.secondary_metrics or [],
                    minimum_detectable_effect=record.minimum_detectable_effect,
                    confidence_level=record.confidence_level,
                    statistical_power=record.statistical_power,
                    planned_duration_days=record.planned_duration_days,
                    minimum_sample_size=record.minimum_sample_size,
                    maximum_sample_size=record.maximum_sample_size,
                    user_filters=record.user_filters or {},
                    feature_flags=record.feature_flags or [],
                    guardrail_metrics=record.guardrail_metrics or [],
                    error_rate_threshold=record.error_rate_threshold,
                    latency_threshold_ms=record.latency_threshold_ms,
                    created_at=record.created_at,
                    created_by=record.created_by,
                    start_date=record.start_date,
                    end_date=record.end_date,
                    status=TestStatus(record.status),
                )
                self.active_tests[record.test_id] = config
