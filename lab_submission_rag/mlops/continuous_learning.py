"""
Continuous Learning Pipeline for TracSeq 2.0 MLOps

Automated model retraining, data drift detection, and model improvement.
"""

import asyncio
import hashlib
import pickle
import uuid
from dataclasses import asdict, dataclass, field
from datetime import datetime, timedelta
from enum import Enum
from pathlib import Path
from typing import Any, Callable, Dict, List, Optional, Union

import aiofiles
import numpy as np
import pandas as pd
import structlog
from sqlalchemy import JSON, Boolean, Column, DateTime, Float, Integer, String, Text, create_engine
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker

logger = structlog.get_logger(__name__)


class TrainingStatus(Enum):
    SCHEDULED = "scheduled"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


class TriggerType(Enum):
    SCHEDULED = "scheduled"
    PERFORMANCE_DEGRADATION = "performance_degradation"
    DATA_DRIFT = "data_drift"
    MANUAL = "manual"
    NEW_DATA_THRESHOLD = "new_data_threshold"


@dataclass
class DataDriftConfig:
    """Configuration for data drift detection"""

    enabled: bool = True
    detection_method: str = "statistical"  # "statistical", "wasserstein", "kl_divergence"
    threshold: float = 0.05
    window_size: int = 1000
    reference_window_size: int = 5000
    features_to_monitor: List[str] = field(default_factory=list)


@dataclass
class PerformanceDegradationConfig:
    """Configuration for performance degradation detection"""

    enabled: bool = True
    metrics_to_monitor: List[str] = field(default_factory=lambda: ["accuracy", "f1_score"])
    degradation_threshold: float = 0.05  # 5% drop
    evaluation_window: int = 100
    minimum_samples: int = 50


@dataclass
class RetrainingConfig:
    """Configuration for automated retraining"""

    model_type: str
    training_pipeline: str  # Reference to training pipeline

    # Scheduling
    schedule_cron: Optional[str] = None  # e.g., "0 2 * * 1" for weekly at 2 AM
    min_retrain_interval_hours: int = 24

    # Trigger conditions
    data_drift_config: DataDriftConfig = field(default_factory=DataDriftConfig)
    performance_config: PerformanceDegradationConfig = field(
        default_factory=PerformanceDegradationConfig
    )
    new_data_threshold: int = 1000  # Retrain when this many new samples

    # Training parameters
    validation_split: float = 0.2
    test_split: float = 0.1
    hyperparameter_search: bool = True
    cross_validation_folds: int = 5

    # Quality gates
    minimum_accuracy: float = 0.85
    minimum_improvement: float = 0.01  # 1% improvement required

    # Resource limits
    max_training_time_hours: int = 4
    max_memory_gb: int = 8

    # Metadata
    created_at: datetime = field(default_factory=datetime.utcnow)
    created_by: str = ""
    enabled: bool = True


@dataclass
class TrainingRun:
    """Represents a single training run"""

    run_id: str
    config_id: str
    trigger_type: TriggerType
    trigger_reason: str

    # Data information
    dataset_hash: str
    training_samples: int
    validation_samples: int
    test_samples: int

    # Training details
    hyperparameters: Dict[str, Any]
    training_duration_seconds: float

    # Results
    metrics: Dict[str, float]
    model_path: Optional[str] = None

    # Status
    status: TrainingStatus = TrainingStatus.SCHEDULED
    started_at: Optional[datetime] = None
    completed_at: Optional[datetime] = None
    error_message: Optional[str] = None

    # Comparison with previous model
    baseline_metrics: Dict[str, float] = field(default_factory=dict)
    improvement: Dict[str, float] = field(default_factory=dict)

    created_at: datetime = field(default_factory=datetime.utcnow)


@dataclass
class DataDriftReport:
    """Data drift detection report"""

    report_id: str
    config_id: str

    # Detection details
    detection_method: str
    reference_period: tuple[datetime, datetime]
    comparison_period: tuple[datetime, datetime]

    # Results
    drift_detected: bool
    drift_score: float
    threshold: float
    affected_features: List[str]

    # Statistical details
    statistical_tests: Dict[str, Dict[str, float]]  # feature -> {statistic, p_value}
    feature_distributions: Dict[str, Dict[str, Any]]

    created_at: datetime = field(default_factory=datetime.utcnow)


Base = declarative_base()


class RetrainingConfigRecord(Base):
    """Database model for retraining configurations"""

    __tablename__ = "retraining_configs"

    config_id = Column(String, primary_key=True)
    model_type = Column(String, nullable=False)
    training_pipeline = Column(String, nullable=False)

    # Scheduling
    schedule_cron = Column(String)
    min_retrain_interval_hours = Column(Integer)

    # Trigger configs as JSON
    data_drift_config = Column(JSON)
    performance_config = Column(JSON)
    new_data_threshold = Column(Integer)

    # Training parameters
    validation_split = Column(Float)
    test_split = Column(Float)
    hyperparameter_search = Column(Boolean)
    cross_validation_folds = Column(Integer)

    # Quality gates
    minimum_accuracy = Column(Float)
    minimum_improvement = Column(Float)

    # Resource limits
    max_training_time_hours = Column(Integer)
    max_memory_gb = Column(Integer)

    # Metadata
    created_at = Column(DateTime, default=datetime.utcnow)
    created_by = Column(String)
    enabled = Column(Boolean, default=True)

    # Tracking
    last_training_run = Column(String)
    last_retrain_at = Column(DateTime)


class TrainingRunRecord(Base):
    """Database model for training runs"""

    __tablename__ = "training_runs"

    run_id = Column(String, primary_key=True)
    config_id = Column(String, nullable=False)
    trigger_type = Column(String, nullable=False)
    trigger_reason = Column(Text)

    # Data information
    dataset_hash = Column(String)
    training_samples = Column(Integer)
    validation_samples = Column(Integer)
    test_samples = Column(Integer)

    # Training details
    hyperparameters = Column(JSON)
    training_duration_seconds = Column(Float)

    # Results
    metrics = Column(JSON)
    model_path = Column(String)

    # Status
    status = Column(String, default=TrainingStatus.SCHEDULED.value)
    started_at = Column(DateTime)
    completed_at = Column(DateTime)
    error_message = Column(Text)

    # Comparison
    baseline_metrics = Column(JSON)
    improvement = Column(JSON)

    created_at = Column(DateTime, default=datetime.utcnow)


class DataDriftRecord(Base):
    """Database model for data drift reports"""

    __tablename__ = "data_drift_reports"

    report_id = Column(String, primary_key=True)
    config_id = Column(String, nullable=False)

    # Detection details
    detection_method = Column(String)
    reference_period_start = Column(DateTime)
    reference_period_end = Column(DateTime)
    comparison_period_start = Column(DateTime)
    comparison_period_end = Column(DateTime)

    # Results
    drift_detected = Column(Boolean)
    drift_score = Column(Float)
    threshold = Column(Float)
    affected_features = Column(JSON)

    # Statistical details
    statistical_tests = Column(JSON)
    feature_distributions = Column(JSON)

    created_at = Column(DateTime, default=datetime.utcnow)


class ContinuousLearningPipeline:
    """
    Automated continuous learning pipeline for model retraining and improvement.

    Features:
    - Data drift detection
    - Performance degradation monitoring
    - Automated retraining triggers
    - Quality gates and validation
    - Resource management
    - A/B testing integration
    """

    def __init__(
        self, database_url: str, data_dir: Union[str, Path], model_registry, experiment_tracker
    ):
        self.data_dir = Path(data_dir)
        self.data_dir.mkdir(parents=True, exist_ok=True)

        # Dependencies
        self.model_registry = model_registry
        self.experiment_tracker = experiment_tracker

        # Database setup
        self.engine = create_engine(database_url)
        Base.metadata.create_all(self.engine)
        self.SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=self.engine)

        # Training pipelines registry
        self.training_pipelines: Dict[str, Callable] = {}

        # Background tasks
        self._monitoring_tasks: List[asyncio.Task] = []

    def register_training_pipeline(self, name: str, pipeline_func: Callable):
        """Register a training pipeline function."""
        self.training_pipelines[name] = pipeline_func
        logger.info("Training pipeline registered", name=name)

    async def create_retraining_config(self, config: RetrainingConfig) -> str:
        """Create a new retraining configuration."""
        if not config.config_id:
            config.config_id = f"config_{uuid.uuid4().hex[:8]}"

        # Validate configuration
        await self._validate_retraining_config(config)

        # Store in database
        with self.SessionLocal() as session:
            record = RetrainingConfigRecord(
                config_id=config.config_id,
                model_type=config.model_type,
                training_pipeline=config.training_pipeline,
                schedule_cron=config.schedule_cron,
                min_retrain_interval_hours=config.min_retrain_interval_hours,
                data_drift_config=asdict(config.data_drift_config),
                performance_config=asdict(config.performance_config),
                new_data_threshold=config.new_data_threshold,
                validation_split=config.validation_split,
                test_split=config.test_split,
                hyperparameter_search=config.hyperparameter_search,
                cross_validation_folds=config.cross_validation_folds,
                minimum_accuracy=config.minimum_accuracy,
                minimum_improvement=config.minimum_improvement,
                max_training_time_hours=config.max_training_time_hours,
                max_memory_gb=config.max_memory_gb,
                created_at=config.created_at,
                created_by=config.created_by,
                enabled=config.enabled,
            )
            session.add(record)
            session.commit()

        logger.info(
            "Retraining configuration created",
            config_id=config.config_id,
            model_type=config.model_type,
        )

        return config.config_id

    async def start_monitoring(self):
        """Start continuous monitoring for all enabled configurations."""
        configs = await self.list_configs(enabled_only=True)

        for config in configs:
            # Start monitoring tasks
            if config.data_drift_config.enabled:
                task = asyncio.create_task(self._monitor_data_drift(config.config_id))
                self._monitoring_tasks.append(task)

            if config.performance_config.enabled:
                task = asyncio.create_task(self._monitor_performance(config.config_id))
                self._monitoring_tasks.append(task)

            # Start scheduled training if configured
            if config.schedule_cron:
                task = asyncio.create_task(self._schedule_training(config.config_id))
                self._monitoring_tasks.append(task)

        logger.info(
            "Continuous learning monitoring started", active_tasks=len(self._monitoring_tasks)
        )

    async def stop_monitoring(self):
        """Stop all monitoring tasks."""
        for task in self._monitoring_tasks:
            task.cancel()

        await asyncio.gather(*self._monitoring_tasks, return_exceptions=True)
        self._monitoring_tasks.clear()

        logger.info("Continuous learning monitoring stopped")

    async def trigger_retraining(
        self, config_id: str, trigger_type: TriggerType, trigger_reason: str, force: bool = False
    ) -> Optional[str]:
        """Trigger a retraining run."""
        config = await self.get_config(config_id)
        if not config:
            logger.error("Configuration not found", config_id=config_id)
            return None

        if not config.enabled and not force:
            logger.warning("Configuration disabled", config_id=config_id)
            return None

        # Check minimum interval
        if not force and not await self._check_minimum_interval(
            config_id, config.min_retrain_interval_hours
        ):
            logger.info(
                "Skipping retraining due to minimum interval",
                config_id=config_id,
                min_interval_hours=config.min_retrain_interval_hours,
            )
            return None

        # Create training run
        run_id = f"run_{uuid.uuid4().hex[:8]}"

        # Prepare data
        data_hash, train_samples, val_samples, test_samples = await self._prepare_training_data(
            config
        )

        # Get baseline metrics
        baseline_metrics = await self._get_baseline_metrics(config.model_type)

        training_run = TrainingRun(
            run_id=run_id,
            config_id=config_id,
            trigger_type=trigger_type,
            trigger_reason=trigger_reason,
            dataset_hash=data_hash,
            training_samples=train_samples,
            validation_samples=val_samples,
            test_samples=test_samples,
            hyperparameters={},  # Will be filled during training
            training_duration_seconds=0.0,
            metrics={},
            baseline_metrics=baseline_metrics,
        )

        # Store in database
        await self._store_training_run(training_run)

        # Start training asynchronously
        task = asyncio.create_task(self._execute_training_run(training_run, config))

        logger.info(
            "Retraining triggered",
            run_id=run_id,
            config_id=config_id,
            trigger_type=trigger_type.value,
            trigger_reason=trigger_reason,
        )

        return run_id

    async def get_training_run(self, run_id: str) -> Optional[TrainingRun]:
        """Get training run details."""
        with self.SessionLocal() as session:
            record = (
                session.query(TrainingRunRecord).filter(TrainingRunRecord.run_id == run_id).first()
            )

            if not record:
                return None

            return TrainingRun(
                run_id=record.run_id,
                config_id=record.config_id,
                trigger_type=TriggerType(record.trigger_type),
                trigger_reason=record.trigger_reason,
                dataset_hash=record.dataset_hash,
                training_samples=record.training_samples,
                validation_samples=record.validation_samples,
                test_samples=record.test_samples,
                hyperparameters=record.hyperparameters or {},
                training_duration_seconds=record.training_duration_seconds or 0.0,
                metrics=record.metrics or {},
                model_path=record.model_path,
                status=TrainingStatus(record.status),
                started_at=record.started_at,
                completed_at=record.completed_at,
                error_message=record.error_message,
                baseline_metrics=record.baseline_metrics or {},
                improvement=record.improvement or {},
                created_at=record.created_at,
            )

    async def get_config(self, config_id: str) -> Optional[RetrainingConfig]:
        """Get retraining configuration."""
        with self.SessionLocal() as session:
            record = (
                session.query(RetrainingConfigRecord)
                .filter(RetrainingConfigRecord.config_id == config_id)
                .first()
            )

            if not record:
                return None

            # Reconstruct dataclass
            data_drift_config = (
                DataDriftConfig(**record.data_drift_config)
                if record.data_drift_config
                else DataDriftConfig()
            )
            performance_config = (
                PerformanceDegradationConfig(**record.performance_config)
                if record.performance_config
                else PerformanceDegradationConfig()
            )

            return RetrainingConfig(
                config_id=record.config_id,
                model_type=record.model_type,
                training_pipeline=record.training_pipeline,
                schedule_cron=record.schedule_cron,
                min_retrain_interval_hours=record.min_retrain_interval_hours,
                data_drift_config=data_drift_config,
                performance_config=performance_config,
                new_data_threshold=record.new_data_threshold,
                validation_split=record.validation_split,
                test_split=record.test_split,
                hyperparameter_search=record.hyperparameter_search,
                cross_validation_folds=record.cross_validation_folds,
                minimum_accuracy=record.minimum_accuracy,
                minimum_improvement=record.minimum_improvement,
                max_training_time_hours=record.max_training_time_hours,
                max_memory_gb=record.max_memory_gb,
                created_at=record.created_at,
                created_by=record.created_by,
                enabled=record.enabled,
            )

    async def list_configs(self, enabled_only: bool = False) -> List[RetrainingConfig]:
        """List all retraining configurations."""
        with self.SessionLocal() as session:
            query = session.query(RetrainingConfigRecord)

            if enabled_only:
                query = query.filter(RetrainingConfigRecord.enabled == True)

            records = query.all()

            configs = []
            for record in records:
                config = await self.get_config(record.config_id)
                if config:
                    configs.append(config)

            return configs

    async def list_training_runs(
        self,
        config_id: Optional[str] = None,
        status: Optional[TrainingStatus] = None,
        limit: int = 50,
    ) -> List[TrainingRun]:
        """List training runs."""
        with self.SessionLocal() as session:
            query = session.query(TrainingRunRecord)

            if config_id:
                query = query.filter(TrainingRunRecord.config_id == config_id)
            if status:
                query = query.filter(TrainingRunRecord.status == status.value)

            query = query.order_by(TrainingRunRecord.created_at.desc()).limit(limit)
            records = query.all()

            runs = []
            for record in records:
                run = await self.get_training_run(record.run_id)
                if run:
                    runs.append(run)

            return runs

    async def detect_data_drift(self, config_id: str) -> Optional[DataDriftReport]:
        """Manually trigger data drift detection."""
        config = await self.get_config(config_id)
        if not config or not config.data_drift_config.enabled:
            return None

        return await self._detect_data_drift(config)

    async def _monitor_data_drift(self, config_id: str):
        """Continuous data drift monitoring."""
        while True:
            try:
                config = await self.get_config(config_id)
                if not config or not config.enabled or not config.data_drift_config.enabled:
                    break

                # Detect drift
                drift_report = await self._detect_data_drift(config)

                if drift_report and drift_report.drift_detected:
                    logger.warning(
                        "Data drift detected",
                        config_id=config_id,
                        drift_score=drift_report.drift_score,
                        threshold=drift_report.threshold,
                    )

                    # Trigger retraining
                    await self.trigger_retraining(
                        config_id,
                        TriggerType.DATA_DRIFT,
                        f"Data drift detected: score={drift_report.drift_score:.4f}, "
                        f"threshold={drift_report.threshold:.4f}",
                    )

                # Wait before next check
                await asyncio.sleep(3600)  # Check every hour

            except Exception as e:
                logger.error("Error in data drift monitoring", config_id=config_id, error=str(e))
                await asyncio.sleep(300)  # Wait 5 minutes before retry

    async def _monitor_performance(self, config_id: str):
        """Continuous performance monitoring."""
        while True:
            try:
                config = await self.get_config(config_id)
                if not config or not config.enabled or not config.performance_config.enabled:
                    break

                # Check performance degradation
                degradation_detected = await self._check_performance_degradation(config)

                if degradation_detected:
                    logger.warning("Performance degradation detected", config_id=config_id)

                    # Trigger retraining
                    await self.trigger_retraining(
                        config_id,
                        TriggerType.PERFORMANCE_DEGRADATION,
                        "Performance degradation detected",
                    )

                # Wait before next check
                await asyncio.sleep(1800)  # Check every 30 minutes

            except Exception as e:
                logger.error("Error in performance monitoring", config_id=config_id, error=str(e))
                await asyncio.sleep(300)  # Wait 5 minutes before retry

    async def _schedule_training(self, config_id: str):
        """Scheduled training based on cron expression."""
        # This is a simplified implementation
        # In production, use a proper cron scheduler like APScheduler

        config = await self.get_config(config_id)
        if not config or not config.schedule_cron:
            return

        # For now, implement simple daily scheduling
        while True:
            try:
                if not config.enabled:
                    break

                # Wait until next scheduled time
                # This is simplified - implement proper cron parsing
                await asyncio.sleep(86400)  # Daily

                # Trigger scheduled retraining
                await self.trigger_retraining(
                    config_id,
                    TriggerType.SCHEDULED,
                    f"Scheduled retraining: {config.schedule_cron}",
                )

            except Exception as e:
                logger.error("Error in scheduled training", config_id=config_id, error=str(e))
                await asyncio.sleep(3600)  # Wait 1 hour before retry

    async def _execute_training_run(self, training_run: TrainingRun, config: RetrainingConfig):
        """Execute a training run."""
        try:
            # Update status
            training_run.status = TrainingStatus.RUNNING
            training_run.started_at = datetime.utcnow()
            await self._update_training_run(training_run)

            # Get training pipeline
            if config.training_pipeline not in self.training_pipelines:
                raise ValueError(f"Training pipeline '{config.training_pipeline}' not found")

            training_func = self.training_pipelines[config.training_pipeline]

            # Prepare training data
            train_data, val_data, test_data = await self._load_training_data(
                training_run.dataset_hash, config
            )

            # Start experiment tracking
            exp_config = {
                "name": f"Continuous Learning - {config.model_type}",
                "description": f"Automated retraining triggered by {training_run.trigger_type.value}",
                "model_type": config.model_type,
                "trigger_type": training_run.trigger_type.value,
                "trigger_reason": training_run.trigger_reason,
            }

            experiment_id = await self.experiment_tracker.start_experiment(exp_config)

            # Execute training
            start_time = datetime.utcnow()

            model, hyperparameters, metrics = await training_func(
                train_data=train_data,
                val_data=val_data,
                test_data=test_data,
                config=config,
                experiment_tracker=self.experiment_tracker,
            )

            end_time = datetime.utcnow()
            training_duration = (end_time - start_time).total_seconds()

            # Validate results
            if not await self._validate_training_results(metrics, config):
                raise ValueError("Training results did not meet quality gates")

            # Calculate improvement
            improvement = {}
            for metric, value in metrics.items():
                if metric in training_run.baseline_metrics:
                    baseline = training_run.baseline_metrics[metric]
                    improvement[metric] = (value - baseline) / baseline if baseline > 0 else 0

            # Check minimum improvement
            primary_metric = "accuracy"  # Configure based on model type
            if primary_metric in improvement:
                if improvement[primary_metric] < config.minimum_improvement:
                    logger.warning(
                        "Insufficient improvement, model will not be promoted",
                        run_id=training_run.run_id,
                        improvement=improvement[primary_metric],
                        required=config.minimum_improvement,
                    )

            # Save model
            model_path = await self._save_trained_model(model, training_run.run_id)

            # Update training run
            training_run.status = TrainingStatus.COMPLETED
            training_run.completed_at = end_time
            training_run.hyperparameters = hyperparameters
            training_run.training_duration_seconds = training_duration
            training_run.metrics = metrics
            training_run.model_path = model_path
            training_run.improvement = improvement

            await self._update_training_run(training_run)

            # Register model in registry
            from .model_registry import ModelMetadata, ModelStatus

            metadata = ModelMetadata(
                model_id=f"{config.model_type}_continuous_learning",
                version=f"cl_{training_run.run_id}",
                name=f"Continuous Learning Model - {config.model_type}",
                description=f"Model retrained via continuous learning pipeline. "
                f"Triggered by: {training_run.trigger_reason}",
                model_type=config.model_type,
                framework="sklearn",  # Configurable
                accuracy=metrics.get("accuracy", 0.0),
                precision=metrics.get("precision", 0.0),
                recall=metrics.get("recall", 0.0),
                f1_score=metrics.get("f1_score", 0.0),
                confidence_score=metrics.get("confidence_score", 0.0),
                training_data_hash=training_run.dataset_hash,
                training_duration_seconds=training_duration,
                hyperparameters=hyperparameters,
                feature_columns=[],  # Fill from training data
                status=ModelStatus.VALIDATING,
                created_at=training_run.completed_at,
                updated_at=training_run.completed_at,
                created_by="continuous_learning_pipeline",
            )

            model_registry_id = await self.model_registry.register_model(
                model, metadata, hyperparameters
            )

            # Complete experiment
            await self.experiment_tracker.complete_experiment(
                experiment_id,
                final_metrics=metrics,
                notes=f"Continuous learning run {training_run.run_id} completed successfully",
            )

            logger.info(
                "Training run completed successfully",
                run_id=training_run.run_id,
                model_registry_id=model_registry_id,
                metrics=metrics,
                improvement=improvement,
            )

        except Exception as e:
            # Update training run with error
            training_run.status = TrainingStatus.FAILED
            training_run.completed_at = datetime.utcnow()
            training_run.error_message = str(e)

            await self._update_training_run(training_run)

            logger.error("Training run failed", run_id=training_run.run_id, error=str(e))

    async def _detect_data_drift(self, config: RetrainingConfig) -> Optional[DataDriftReport]:
        """Detect data drift using statistical tests."""
        drift_config = config.data_drift_config

        # Get reference and comparison data
        # This is a placeholder - implement actual data loading
        reference_data = await self._get_reference_data(
            config.model_type, drift_config.reference_window_size
        )
        comparison_data = await self._get_recent_data(config.model_type, drift_config.window_size)

        if reference_data is None or comparison_data is None:
            return None

        # Detect drift using statistical tests
        drift_detected = False
        drift_score = 0.0
        affected_features = []
        statistical_tests = {}

        features_to_check = drift_config.features_to_monitor or reference_data.columns.tolist()

        for feature in features_to_check:
            if feature in reference_data.columns and feature in comparison_data.columns:
                # Kolmogorov-Smirnov test
                from scipy.stats import ks_2samp

                statistic, p_value = ks_2samp(
                    reference_data[feature].dropna(), comparison_data[feature].dropna()
                )

                statistical_tests[feature] = {"statistic": statistic, "p_value": p_value}

                if p_value < drift_config.threshold:
                    drift_detected = True
                    affected_features.append(feature)
                    drift_score = max(drift_score, statistic)

        # Create drift report
        report_id = f"drift_{uuid.uuid4().hex[:8]}"

        report = DataDriftReport(
            report_id=report_id,
            config_id=config.config_id,
            detection_method=drift_config.detection_method,
            reference_period=(
                datetime.utcnow() - timedelta(days=30),
                datetime.utcnow() - timedelta(days=7),
            ),
            comparison_period=(datetime.utcnow() - timedelta(days=7), datetime.utcnow()),
            drift_detected=drift_detected,
            drift_score=drift_score,
            threshold=drift_config.threshold,
            affected_features=affected_features,
            statistical_tests=statistical_tests,
            feature_distributions={},  # Could add distribution summaries
        )

        # Store report
        await self._store_drift_report(report)

        return report

    async def _check_performance_degradation(self, config: RetrainingConfig) -> bool:
        """Check for performance degradation."""
        perf_config = config.performance_config

        # Get recent model performance
        # This is a placeholder - implement actual performance data loading
        recent_metrics = await self._get_recent_performance_metrics(
            config.model_type, perf_config.evaluation_window
        )

        if not recent_metrics or len(recent_metrics) < perf_config.minimum_samples:
            return False

        # Get baseline performance
        baseline_metrics = await self._get_baseline_metrics(config.model_type)

        # Check for degradation
        for metric in perf_config.metrics_to_monitor:
            if metric in baseline_metrics and metric in recent_metrics:
                current_avg = np.mean(recent_metrics[metric])
                baseline = baseline_metrics[metric]

                degradation = (baseline - current_avg) / baseline if baseline > 0 else 0

                if degradation > perf_config.degradation_threshold:
                    logger.warning(
                        "Performance degradation detected",
                        metric=metric,
                        current=current_avg,
                        baseline=baseline,
                        degradation=degradation,
                        threshold=perf_config.degradation_threshold,
                    )
                    return True

        return False

    async def _validate_retraining_config(self, config: RetrainingConfig):
        """Validate retraining configuration."""
        if config.training_pipeline not in self.training_pipelines:
            raise ValueError(f"Training pipeline '{config.training_pipeline}' not registered")

        if config.validation_split + config.test_split >= 1.0:
            raise ValueError("Validation and test splits must sum to less than 1.0")

    async def _check_minimum_interval(self, config_id: str, min_hours: int) -> bool:
        """Check if minimum retraining interval has passed."""
        with self.SessionLocal() as session:
            record = (
                session.query(RetrainingConfigRecord)
                .filter(RetrainingConfigRecord.config_id == config_id)
                .first()
            )

            if not record or not record.last_retrain_at:
                return True

            time_since_last = datetime.utcnow() - record.last_retrain_at
            return time_since_last.total_seconds() >= min_hours * 3600

    async def _prepare_training_data(self, config: RetrainingConfig) -> tuple[str, int, int, int]:
        """Prepare training data and return hash and sample counts."""
        # This is a placeholder - implement actual data preparation
        # For now, return dummy values

        total_samples = 10000
        train_samples = int(total_samples * (1 - config.validation_split - config.test_split))
        val_samples = int(total_samples * config.validation_split)
        test_samples = int(total_samples * config.test_split)

        # Create hash of data
        data_hash = hashlib.md5(
            f"{config.model_type}_{datetime.utcnow().date()}".encode()
        ).hexdigest()

        return data_hash, train_samples, val_samples, test_samples

    async def _get_baseline_metrics(self, model_type: str) -> Dict[str, float]:
        """Get baseline metrics for comparison."""
        # Get current production model metrics
        prod_model_info = await self.model_registry.get_production_model(model_type)

        if prod_model_info:
            _, metadata = prod_model_info
            return {
                "accuracy": metadata.accuracy,
                "precision": metadata.precision,
                "recall": metadata.recall,
                "f1_score": metadata.f1_score,
            }

        # Default baseline if no production model
        return {"accuracy": 0.85, "precision": 0.85, "recall": 0.85, "f1_score": 0.85}

    async def _store_training_run(self, run: TrainingRun):
        """Store training run in database."""
        with self.SessionLocal() as session:
            record = TrainingRunRecord(
                run_id=run.run_id,
                config_id=run.config_id,
                trigger_type=run.trigger_type.value,
                trigger_reason=run.trigger_reason,
                dataset_hash=run.dataset_hash,
                training_samples=run.training_samples,
                validation_samples=run.validation_samples,
                test_samples=run.test_samples,
                hyperparameters=run.hyperparameters,
                training_duration_seconds=run.training_duration_seconds,
                metrics=run.metrics,
                model_path=run.model_path,
                status=run.status.value,
                started_at=run.started_at,
                completed_at=run.completed_at,
                error_message=run.error_message,
                baseline_metrics=run.baseline_metrics,
                improvement=run.improvement,
                created_at=run.created_at,
            )
            session.add(record)
            session.commit()

    async def _update_training_run(self, run: TrainingRun):
        """Update training run in database."""
        with self.SessionLocal() as session:
            record = (
                session.query(TrainingRunRecord)
                .filter(TrainingRunRecord.run_id == run.run_id)
                .first()
            )

            if record:
                record.status = run.status.value
                record.started_at = run.started_at
                record.completed_at = run.completed_at
                record.hyperparameters = run.hyperparameters
                record.training_duration_seconds = run.training_duration_seconds
                record.metrics = run.metrics
                record.model_path = run.model_path
                record.error_message = run.error_message
                record.improvement = run.improvement
                session.commit()

    async def _store_drift_report(self, report: DataDriftReport):
        """Store drift report in database."""
        with self.SessionLocal() as session:
            record = DataDriftRecord(
                report_id=report.report_id,
                config_id=report.config_id,
                detection_method=report.detection_method,
                reference_period_start=report.reference_period[0],
                reference_period_end=report.reference_period[1],
                comparison_period_start=report.comparison_period[0],
                comparison_period_end=report.comparison_period[1],
                drift_detected=report.drift_detected,
                drift_score=report.drift_score,
                threshold=report.threshold,
                affected_features=report.affected_features,
                statistical_tests=report.statistical_tests,
                feature_distributions=report.feature_distributions,
                created_at=report.created_at,
            )
            session.add(record)
            session.commit()

    async def _validate_training_results(
        self, metrics: Dict[str, float], config: RetrainingConfig
    ) -> bool:
        """Validate training results against quality gates."""
        accuracy = metrics.get("accuracy", 0.0)
        return accuracy >= config.minimum_accuracy

    async def _save_trained_model(self, model: Any, run_id: str) -> str:
        """Save trained model to disk."""
        model_path = self.data_dir / "models" / f"{run_id}_model.pkl"
        model_path.parent.mkdir(parents=True, exist_ok=True)

        async with aiofiles.open(model_path, "wb") as f:
            model_bytes = pickle.dumps(model)
            await f.write(model_bytes)

        return str(model_path)

    # Placeholder methods for data access - implement based on your data infrastructure
    async def _get_reference_data(
        self, model_type: str, window_size: int
    ) -> Optional[pd.DataFrame]:
        """Get reference data for drift detection."""
        # Implement based on your data storage
        return None

    async def _get_recent_data(self, model_type: str, window_size: int) -> Optional[pd.DataFrame]:
        """Get recent data for drift detection."""
        # Implement based on your data storage
        return None

    async def _get_recent_performance_metrics(
        self, model_type: str, window_size: int
    ) -> Optional[Dict[str, List[float]]]:
        """Get recent performance metrics."""
        # Implement based on your monitoring system
        return None

    async def _load_training_data(
        self, data_hash: str, config: RetrainingConfig
    ) -> tuple[Any, Any, Any]:
        """Load prepared training data."""
        # Implement based on your data storage
        # Return train_data, val_data, test_data
        return None, None, None
