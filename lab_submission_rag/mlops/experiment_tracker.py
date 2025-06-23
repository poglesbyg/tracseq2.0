"""
Experiment Tracker for TracSeq 2.0 MLOps Pipeline

Tracks experiments, hyperparameters, metrics, and artifacts for reproducible ML workflows.
"""

import json
import uuid
from dataclasses import asdict, dataclass, field
from datetime import datetime
from enum import Enum
from pathlib import Path
from typing import Any, Dict, List, Optional, Union

import aiofiles
import matplotlib.pyplot as plt
import pandas as pd
import structlog
from sqlalchemy import JSON, Column, DateTime, Float, Integer, String, Text, create_engine
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker

logger = structlog.get_logger(__name__)


class ExperimentStatus(Enum):
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


@dataclass
class ExperimentConfig:
    """Experiment configuration and metadata"""

    experiment_id: str
    name: str
    description: str
    tags: List[str] = field(default_factory=list)

    # Model configuration
    model_type: str = ""
    framework: str = ""
    algorithm: str = ""

    # Data configuration
    dataset_name: str = ""
    dataset_version: str = ""
    train_split: float = 0.8
    validation_split: float = 0.1
    test_split: float = 0.1

    # Training configuration
    hyperparameters: Dict[str, Any] = field(default_factory=dict)
    training_config: Dict[str, Any] = field(default_factory=dict)

    # Environment
    python_version: str = ""
    requirements: List[str] = field(default_factory=list)
    gpu_enabled: bool = False

    # Tracking
    created_at: datetime = field(default_factory=datetime.utcnow)
    created_by: str = ""
    parent_experiment_id: Optional[str] = None


@dataclass
class ExperimentMetrics:
    """Real-time experiment metrics"""

    experiment_id: str
    step: int
    epoch: Optional[int] = None

    # Training metrics
    train_loss: Optional[float] = None
    train_accuracy: Optional[float] = None
    train_precision: Optional[float] = None
    train_recall: Optional[float] = None
    train_f1: Optional[float] = None

    # Validation metrics
    val_loss: Optional[float] = None
    val_accuracy: Optional[float] = None
    val_precision: Optional[float] = None
    val_recall: Optional[float] = None
    val_f1: Optional[float] = None

    # Custom metrics
    custom_metrics: Dict[str, float] = field(default_factory=dict)

    # System metrics
    memory_usage_mb: Optional[float] = None
    gpu_usage_percent: Optional[float] = None
    training_time_seconds: Optional[float] = None

    timestamp: datetime = field(default_factory=datetime.utcnow)


@dataclass
class ExperimentArtifact:
    """Experiment artifacts (models, plots, data files)"""

    artifact_id: str
    experiment_id: str
    name: str
    artifact_type: str  # "model", "plot", "data", "config", "log"
    file_path: str
    file_size_bytes: int
    description: str = ""
    metadata: Dict[str, Any] = field(default_factory=dict)
    created_at: datetime = field(default_factory=datetime.utcnow)


Base = declarative_base()


class ExperimentRecord(Base):
    """Database model for experiments"""

    __tablename__ = "experiments"

    experiment_id = Column(String, primary_key=True)
    name = Column(String, nullable=False)
    description = Column(Text)
    tags = Column(JSON)

    # Configuration
    model_type = Column(String)
    framework = Column(String)
    algorithm = Column(String)

    # Data config
    dataset_name = Column(String)
    dataset_version = Column(String)
    train_split = Column(Float)
    validation_split = Column(Float)
    test_split = Column(Float)

    # Training config
    hyperparameters = Column(JSON)
    training_config = Column(JSON)

    # Environment
    python_version = Column(String)
    requirements = Column(JSON)
    gpu_enabled = Column(String)  # Store as string for compatibility

    # Status and timing
    status = Column(String, default=ExperimentStatus.RUNNING.value)
    created_at = Column(DateTime, default=datetime.utcnow)
    started_at = Column(DateTime)
    completed_at = Column(DateTime)
    created_by = Column(String)
    parent_experiment_id = Column(String)

    # Final results
    final_metrics = Column(JSON)
    notes = Column(Text)


class MetricsRecord(Base):
    """Database model for experiment metrics"""

    __tablename__ = "experiment_metrics"

    id = Column(Integer, primary_key=True, autoincrement=True)
    experiment_id = Column(String, nullable=False)
    step = Column(Integer, nullable=False)
    epoch = Column(Integer)

    # Training metrics
    train_loss = Column(Float)
    train_accuracy = Column(Float)
    train_precision = Column(Float)
    train_recall = Column(Float)
    train_f1 = Column(Float)

    # Validation metrics
    val_loss = Column(Float)
    val_accuracy = Column(Float)
    val_precision = Column(Float)
    val_recall = Column(Float)
    val_f1 = Column(Float)

    # Custom metrics as JSON
    custom_metrics = Column(JSON)

    # System metrics
    memory_usage_mb = Column(Float)
    gpu_usage_percent = Column(Float)
    training_time_seconds = Column(Float)

    timestamp = Column(DateTime, default=datetime.utcnow)


class ArtifactRecord(Base):
    """Database model for experiment artifacts"""

    __tablename__ = "experiment_artifacts"

    artifact_id = Column(String, primary_key=True)
    experiment_id = Column(String, nullable=False)
    name = Column(String, nullable=False)
    artifact_type = Column(String, nullable=False)
    file_path = Column(String, nullable=False)
    file_size_bytes = Column(Integer)
    description = Column(Text)
    metadata = Column(JSON)
    created_at = Column(DateTime, default=datetime.utcnow)


class ExperimentTracker:
    """
    Comprehensive experiment tracking for ML workflows.

    Features:
    - Experiment configuration and metadata tracking
    - Real-time metrics logging
    - Artifact management (models, plots, data)
    - Experiment comparison and analysis
    - Hyperparameter optimization integration
    """

    def __init__(self, tracking_dir: Union[str, Path], database_url: str):
        self.tracking_dir = Path(tracking_dir)
        self.tracking_dir.mkdir(parents=True, exist_ok=True)

        # Database setup
        self.engine = create_engine(database_url)
        Base.metadata.create_all(self.engine)
        self.SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=self.engine)

        # Create directories
        self.experiments_dir = self.tracking_dir / "experiments"
        self.artifacts_dir = self.tracking_dir / "artifacts"
        self.plots_dir = self.tracking_dir / "plots"

        for dir_path in [self.experiments_dir, self.artifacts_dir, self.plots_dir]:
            dir_path.mkdir(parents=True, exist_ok=True)

        self.current_experiment: Optional[str] = None

    async def start_experiment(self, config: ExperimentConfig) -> str:
        """Start a new experiment."""
        if not config.experiment_id:
            config.experiment_id = f"exp_{uuid.uuid4().hex[:8]}"

        # Create experiment directory
        exp_dir = self.experiments_dir / config.experiment_id
        exp_dir.mkdir(parents=True, exist_ok=True)

        # Save configuration
        config_path = exp_dir / "config.json"
        async with aiofiles.open(config_path, "w") as f:
            await f.write(json.dumps(asdict(config), default=str, indent=2))

        # Store in database
        with self.SessionLocal() as session:
            record = ExperimentRecord(
                experiment_id=config.experiment_id,
                name=config.name,
                description=config.description,
                tags=config.tags,
                model_type=config.model_type,
                framework=config.framework,
                algorithm=config.algorithm,
                dataset_name=config.dataset_name,
                dataset_version=config.dataset_version,
                train_split=config.train_split,
                validation_split=config.validation_split,
                test_split=config.test_split,
                hyperparameters=config.hyperparameters,
                training_config=config.training_config,
                python_version=config.python_version,
                requirements=config.requirements,
                gpu_enabled=str(config.gpu_enabled),
                created_at=config.created_at,
                started_at=datetime.utcnow(),
                created_by=config.created_by,
                parent_experiment_id=config.parent_experiment_id,
            )
            session.add(record)
            session.commit()

        self.current_experiment = config.experiment_id

        logger.info(
            "Experiment started",
            experiment_id=config.experiment_id,
            name=config.name,
            model_type=config.model_type,
        )

        return config.experiment_id

    async def log_metrics(self, metrics: ExperimentMetrics):
        """Log metrics for the current experiment."""
        if not metrics.experiment_id and self.current_experiment:
            metrics.experiment_id = self.current_experiment

        if not metrics.experiment_id:
            raise ValueError("No experiment ID provided and no current experiment")

        # Store in database
        with self.SessionLocal() as session:
            record = MetricsRecord(
                experiment_id=metrics.experiment_id,
                step=metrics.step,
                epoch=metrics.epoch,
                train_loss=metrics.train_loss,
                train_accuracy=metrics.train_accuracy,
                train_precision=metrics.train_precision,
                train_recall=metrics.train_recall,
                train_f1=metrics.train_f1,
                val_loss=metrics.val_loss,
                val_accuracy=metrics.val_accuracy,
                val_precision=metrics.val_precision,
                val_recall=metrics.val_recall,
                val_f1=metrics.val_f1,
                custom_metrics=metrics.custom_metrics,
                memory_usage_mb=metrics.memory_usage_mb,
                gpu_usage_percent=metrics.gpu_usage_percent,
                training_time_seconds=metrics.training_time_seconds,
                timestamp=metrics.timestamp,
            )
            session.add(record)
            session.commit()

    async def log_artifact(
        self,
        name: str,
        artifact_type: str,
        content: Union[bytes, str, Any],
        experiment_id: Optional[str] = None,
        description: str = "",
        metadata: Optional[Dict[str, Any]] = None,
    ) -> str:
        """Log an artifact (file, model, plot, etc.)."""
        if not experiment_id:
            experiment_id = self.current_experiment

        if not experiment_id:
            raise ValueError("No experiment ID provided and no current experiment")

        artifact_id = f"artifact_{uuid.uuid4().hex[:8]}"

        # Determine file extension and path
        ext_map = {
            "model": ".pkl",
            "plot": ".png",
            "data": ".csv",
            "config": ".json",
            "log": ".txt",
        }

        ext = ext_map.get(artifact_type, ".bin")
        file_path = self.artifacts_dir / experiment_id / f"{name}_{artifact_id}{ext}"
        file_path.parent.mkdir(parents=True, exist_ok=True)

        # Save content
        file_size = 0
        if isinstance(content, bytes):
            async with aiofiles.open(file_path, "wb") as f:
                await f.write(content)
                file_size = len(content)
        elif isinstance(content, str):
            async with aiofiles.open(file_path, "w") as f:
                await f.write(content)
                file_size = len(content.encode())
        else:
            # Assume it's a serializable object
            import pickle

            content_bytes = pickle.dumps(content)
            async with aiofiles.open(file_path, "wb") as f:
                await f.write(content_bytes)
                file_size = len(content_bytes)

        # Create artifact record
        artifact = ExperimentArtifact(
            artifact_id=artifact_id,
            experiment_id=experiment_id,
            name=name,
            artifact_type=artifact_type,
            file_path=str(file_path),
            file_size_bytes=file_size,
            description=description,
            metadata=metadata or {},
        )

        # Store in database
        with self.SessionLocal() as session:
            record = ArtifactRecord(
                artifact_id=artifact.artifact_id,
                experiment_id=artifact.experiment_id,
                name=artifact.name,
                artifact_type=artifact.artifact_type,
                file_path=artifact.file_path,
                file_size_bytes=artifact.file_size_bytes,
                description=artifact.description,
                metadata=artifact.metadata,
                created_at=artifact.created_at,
            )
            session.add(record)
            session.commit()

        logger.info(
            "Artifact logged",
            experiment_id=experiment_id,
            artifact_id=artifact_id,
            name=name,
            type=artifact_type,
        )

        return artifact_id

    async def complete_experiment(
        self,
        experiment_id: Optional[str] = None,
        final_metrics: Optional[Dict[str, float]] = None,
        notes: Optional[str] = None,
        status: ExperimentStatus = ExperimentStatus.COMPLETED,
    ):
        """Mark an experiment as completed."""
        if not experiment_id:
            experiment_id = self.current_experiment

        if not experiment_id:
            raise ValueError("No experiment ID provided and no current experiment")

        with self.SessionLocal() as session:
            record = (
                session.query(ExperimentRecord)
                .filter(ExperimentRecord.experiment_id == experiment_id)
                .first()
            )

            if record:
                record.status = status.value
                record.completed_at = datetime.utcnow()
                record.final_metrics = final_metrics
                record.notes = notes
                session.commit()

        if experiment_id == self.current_experiment:
            self.current_experiment = None

        logger.info("Experiment completed", experiment_id=experiment_id, status=status.value)

    async def get_experiment(self, experiment_id: str) -> Optional[ExperimentConfig]:
        """Get experiment configuration."""
        with self.SessionLocal() as session:
            record = (
                session.query(ExperimentRecord)
                .filter(ExperimentRecord.experiment_id == experiment_id)
                .first()
            )

            if not record:
                return None

            return ExperimentConfig(
                experiment_id=record.experiment_id,
                name=record.name,
                description=record.description,
                tags=record.tags or [],
                model_type=record.model_type or "",
                framework=record.framework or "",
                algorithm=record.algorithm or "",
                dataset_name=record.dataset_name or "",
                dataset_version=record.dataset_version or "",
                train_split=record.train_split or 0.8,
                validation_split=record.validation_split or 0.1,
                test_split=record.test_split or 0.1,
                hyperparameters=record.hyperparameters or {},
                training_config=record.training_config or {},
                python_version=record.python_version or "",
                requirements=record.requirements or [],
                gpu_enabled=record.gpu_enabled == "True",
                created_at=record.created_at,
                created_by=record.created_by or "",
                parent_experiment_id=record.parent_experiment_id,
            )

    async def get_experiment_metrics(self, experiment_id: str) -> pd.DataFrame:
        """Get all metrics for an experiment as a DataFrame."""
        with self.SessionLocal() as session:
            records = (
                session.query(MetricsRecord)
                .filter(MetricsRecord.experiment_id == experiment_id)
                .order_by(MetricsRecord.step)
                .all()
            )

            if not records:
                return pd.DataFrame()

            # Convert to DataFrame
            data = []
            for record in records:
                row = {
                    "step": record.step,
                    "epoch": record.epoch,
                    "train_loss": record.train_loss,
                    "train_accuracy": record.train_accuracy,
                    "train_precision": record.train_precision,
                    "train_recall": record.train_recall,
                    "train_f1": record.train_f1,
                    "val_loss": record.val_loss,
                    "val_accuracy": record.val_accuracy,
                    "val_precision": record.val_precision,
                    "val_recall": record.val_recall,
                    "val_f1": record.val_f1,
                    "memory_usage_mb": record.memory_usage_mb,
                    "gpu_usage_percent": record.gpu_usage_percent,
                    "training_time_seconds": record.training_time_seconds,
                    "timestamp": record.timestamp,
                }

                # Add custom metrics
                if record.custom_metrics:
                    row.update(record.custom_metrics)

                data.append(row)

            return pd.DataFrame(data)

    async def list_experiments(
        self,
        limit: int = 50,
        status: Optional[ExperimentStatus] = None,
        model_type: Optional[str] = None,
        tags: Optional[List[str]] = None,
    ) -> List[Dict[str, Any]]:
        """List experiments with optional filtering."""
        with self.SessionLocal() as session:
            query = session.query(ExperimentRecord)

            if status:
                query = query.filter(ExperimentRecord.status == status.value)
            if model_type:
                query = query.filter(ExperimentRecord.model_type == model_type)

            query = query.order_by(ExperimentRecord.created_at.desc()).limit(limit)
            records = query.all()

            experiments = []
            for record in records:
                # Filter by tags if specified
                if tags and record.tags:
                    if not any(tag in record.tags for tag in tags):
                        continue

                experiments.append(
                    {
                        "experiment_id": record.experiment_id,
                        "name": record.name,
                        "description": record.description,
                        "model_type": record.model_type,
                        "status": record.status,
                        "created_at": record.created_at,
                        "completed_at": record.completed_at,
                        "created_by": record.created_by,
                        "tags": record.tags,
                        "final_metrics": record.final_metrics,
                    }
                )

            return experiments

    async def compare_experiments(
        self, experiment_ids: List[str], metrics: List[str] = None
    ) -> pd.DataFrame:
        """Compare multiple experiments."""
        if not metrics:
            metrics = ["train_accuracy", "val_accuracy", "train_loss", "val_loss"]

        comparison_data = []

        for exp_id in experiment_ids:
            config = await self.get_experiment(exp_id)
            metrics_df = await self.get_experiment_metrics(exp_id)

            if config and not metrics_df.empty:
                # Get final metrics (last step)
                final_metrics = metrics_df.iloc[-1]

                row = {
                    "experiment_id": exp_id,
                    "name": config.name,
                    "model_type": config.model_type,
                    "algorithm": config.algorithm,
                }

                # Add requested metrics
                for metric in metrics:
                    row[metric] = final_metrics.get(metric, None)

                # Add hyperparameters
                for key, value in config.hyperparameters.items():
                    row[f"hp_{key}"] = value

                comparison_data.append(row)

        return pd.DataFrame(comparison_data)

    async def plot_experiment_metrics(
        self, experiment_id: str, metrics: List[str] = None, save_path: Optional[str] = None
    ) -> str:
        """Generate plots for experiment metrics."""
        if not metrics:
            metrics = ["train_loss", "val_loss", "train_accuracy", "val_accuracy"]

        metrics_df = await self.get_experiment_metrics(experiment_id)
        if metrics_df.empty:
            raise ValueError(f"No metrics found for experiment {experiment_id}")

        # Create subplots
        fig, axes = plt.subplots(2, 2, figsize=(15, 10))
        fig.suptitle(f"Experiment {experiment_id} Metrics", fontsize=16)

        available_metrics = [
            m for m in metrics if m in metrics_df.columns and metrics_df[m].notna().any()
        ]

        for i, metric in enumerate(available_metrics[:4]):
            row, col = i // 2, i % 2
            ax = axes[row, col]

            ax.plot(metrics_df["step"], metrics_df[metric], marker="o", linewidth=2)
            ax.set_title(f'{metric.replace("_", " ").title()}')
            ax.set_xlabel("Step")
            ax.set_ylabel(metric)
            ax.grid(True, alpha=0.3)

        # Hide unused subplots
        for i in range(len(available_metrics), 4):
            row, col = i // 2, i % 2
            axes[row, col].set_visible(False)

        plt.tight_layout()

        # Save plot
        if not save_path:
            save_path = self.plots_dir / f"{experiment_id}_metrics.png"

        plt.savefig(save_path, dpi=300, bbox_inches="tight")
        plt.close()

        # Log as artifact
        with open(save_path, "rb") as f:
            plot_content = f.read()

        await self.log_artifact(
            name="metrics_plot",
            artifact_type="plot",
            content=plot_content,
            experiment_id=experiment_id,
            description="Training and validation metrics visualization",
        )

        return str(save_path)
