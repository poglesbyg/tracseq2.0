"""
Data Pipeline for TracSeq 2.0 MLOps

Automated data preprocessing, feature engineering, and data quality validation.
"""

import asyncio
import hashlib
import uuid
from collections.abc import Callable
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from pathlib import Path
from typing import Any

import aiofiles
import numpy as np
import pandas as pd
import structlog
from sklearn.preprocessing import LabelEncoder, StandardScaler
from sqlalchemy import (
    JSON,
    Boolean,
    Column,
    DateTime,
    Float,
    Integer,
    String,
    Text,
    create_engine,
)
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker

logger = structlog.get_logger(__name__)


class PipelineStatus(Enum):
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"


class DataQualityIssue(Enum):
    MISSING_VALUES = "missing_values"
    OUTLIERS = "outliers"
    DUPLICATES = "duplicates"
    SCHEMA_MISMATCH = "schema_mismatch"
    DATA_DRIFT = "data_drift"


@dataclass
class DataSchema:
    """Data schema definition"""

    schema_id: str
    name: str
    version: str

    # Column definitions
    columns: dict[str, dict[str, Any]]  # column_name -> {type, nullable, constraints}

    # Validation rules
    validation_rules: list[dict[str, Any]] = field(default_factory=list)

    # Metadata
    description: str = ""
    created_at: datetime = field(default_factory=datetime.utcnow)
    created_by: str = ""


@dataclass
class DataQualityReport:
    """Data quality assessment report"""

    report_id: str
    dataset_hash: str
    schema_id: str

    # Quality metrics
    total_rows: int
    total_columns: int
    missing_value_percentage: float
    duplicate_rows: int

    # Quality issues
    issues: list[dict[str, Any]] = field(default_factory=list)

    # Column-level statistics
    column_stats: dict[str, dict[str, Any]] = field(default_factory=dict)

    # Overall quality score (0-1)
    quality_score: float = 1.0

    created_at: datetime = field(default_factory=datetime.utcnow)


@dataclass
class FeatureEngineeringConfig:
    """Configuration for feature engineering"""

    config_id: str
    name: str

    # Feature transformations
    numeric_features: list[str] = field(default_factory=list)
    categorical_features: list[str] = field(default_factory=list)
    text_features: list[str] = field(default_factory=list)

    # Scaling and encoding
    scaling_method: str = "standard"  # "standard", "minmax", "robust"
    encoding_method: str = "label"  # "label", "onehot", "target"

    # Feature selection
    feature_selection_enabled: bool = True
    max_features: int | None = None
    correlation_threshold: float = 0.95

    # Text processing
    text_vectorization: str = "tfidf"  # "tfidf", "word2vec", "bert"
    max_vocab_size: int = 10000

    # Custom transformations
    custom_transformations: list[dict[str, Any]] = field(default_factory=list)

    created_at: datetime = field(default_factory=datetime.utcnow)
    created_by: str = ""


@dataclass
class PipelineRun:
    """Data pipeline execution record"""

    run_id: str
    pipeline_id: str
    trigger_type: str  # "manual", "scheduled", "triggered"

    # Input data
    input_data_path: str
    input_data_hash: str
    input_rows: int

    # Configuration
    schema_id: str
    feature_config_id: str

    # Output data
    output_data_path: str | None = None
    output_data_hash: str | None = None
    output_rows: int = 0

    # Quality assessment
    quality_report_id: str | None = None
    data_quality_score: float = 0.0

    # Processing details
    processing_steps: list[str] = field(default_factory=list)
    feature_count: int = 0

    # Status and timing
    status: PipelineStatus = PipelineStatus.PENDING
    started_at: datetime | None = None
    completed_at: datetime | None = None
    processing_time_seconds: float = 0.0

    # Logs and errors
    processing_logs: list[str] = field(default_factory=list)
    error_message: str | None = None

    created_at: datetime = field(default_factory=datetime.utcnow)


Base = declarative_base()


class DataSchemaRecord(Base):
    """Database model for data schemas"""

    __tablename__ = "data_schemas"

    schema_id = Column(String, primary_key=True)
    name = Column(String, nullable=False)
    version = Column(String, nullable=False)

    # Schema definition
    columns = Column(JSON, nullable=False)
    validation_rules = Column(JSON)

    # Metadata
    description = Column(Text)
    created_at = Column(DateTime, default=datetime.utcnow)
    created_by = Column(String)


class DataQualityReportRecord(Base):
    """Database model for data quality reports"""

    __tablename__ = "data_quality_reports"

    report_id = Column(String, primary_key=True)
    dataset_hash = Column(String, nullable=False)
    schema_id = Column(String, nullable=False)

    # Quality metrics
    total_rows = Column(Integer)
    total_columns = Column(Integer)
    missing_value_percentage = Column(Float)
    duplicate_rows = Column(Integer)

    # Quality issues and stats
    issues = Column(JSON)
    column_stats = Column(JSON)
    quality_score = Column(Float)

    created_at = Column(DateTime, default=datetime.utcnow)


class FeatureEngineeringConfigRecord(Base):
    """Database model for feature engineering configurations"""

    __tablename__ = "feature_engineering_configs"

    config_id = Column(String, primary_key=True)
    name = Column(String, nullable=False)

    # Feature definitions
    numeric_features = Column(JSON)
    categorical_features = Column(JSON)
    text_features = Column(JSON)

    # Processing settings
    scaling_method = Column(String, default="standard")
    encoding_method = Column(String, default="label")

    # Feature selection
    feature_selection_enabled = Column(Boolean, default=True)
    max_features = Column(Integer)
    correlation_threshold = Column(Float, default=0.95)

    # Text processing
    text_vectorization = Column(String, default="tfidf")
    max_vocab_size = Column(Integer, default=10000)

    # Custom transformations
    custom_transformations = Column(JSON)

    created_at = Column(DateTime, default=datetime.utcnow)
    created_by = Column(String)


class PipelineRunRecord(Base):
    """Database model for pipeline runs"""

    __tablename__ = "pipeline_runs"

    run_id = Column(String, primary_key=True)
    pipeline_id = Column(String, nullable=False)
    trigger_type = Column(String, nullable=False)

    # Input data
    input_data_path = Column(String, nullable=False)
    input_data_hash = Column(String, nullable=False)
    input_rows = Column(Integer)

    # Configuration
    schema_id = Column(String, nullable=False)
    feature_config_id = Column(String, nullable=False)

    # Output data
    output_data_path = Column(String)
    output_data_hash = Column(String)
    output_rows = Column(Integer, default=0)

    # Quality and processing
    quality_report_id = Column(String)
    data_quality_score = Column(Float, default=0.0)
    processing_steps = Column(JSON)
    feature_count = Column(Integer, default=0)

    # Status and timing
    status = Column(String, default=PipelineStatus.PENDING.value)
    started_at = Column(DateTime)
    completed_at = Column(DateTime)
    processing_time_seconds = Column(Float, default=0.0)

    # Logs and errors
    processing_logs = Column(JSON)
    error_message = Column(Text)

    created_at = Column(DateTime, default=datetime.utcnow)


class DataPipeline:
    """
    Comprehensive data pipeline for ML workflows.

    Features:
    - Data validation and quality assessment
    - Automated feature engineering
    - Data preprocessing and transformation
    - Schema management and validation
    - Data lineage tracking
    - Quality monitoring and alerting
    """

    def __init__(self, database_url: str, data_dir: str | Path):
        self.data_dir = Path(data_dir)
        self.data_dir.mkdir(parents=True, exist_ok=True)

        # Database setup
        self.engine = create_engine(database_url)
        Base.metadata.create_all(self.engine)
        self.SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=self.engine)

        # Processing directories
        self.raw_data_dir = self.data_dir / "raw"
        self.processed_data_dir = self.data_dir / "processed"
        self.features_dir = self.data_dir / "features"

        for dir_path in [self.raw_data_dir, self.processed_data_dir, self.features_dir]:
            dir_path.mkdir(parents=True, exist_ok=True)

        # Custom transformation functions
        self.transformation_functions: dict[str, Callable] = {}

    def register_transformation(self, name: str, func: Callable):
        """Register a custom transformation function."""
        self.transformation_functions[name] = func
        logger.info("Custom transformation registered", name=name)

    async def create_schema(self, schema: DataSchema) -> str:
        """Create a new data schema."""
        if not schema.schema_id:
            schema.schema_id = f"schema_{uuid.uuid4().hex[:8]}"

        # Store in database
        with self.SessionLocal() as session:
            record = DataSchemaRecord(
                schema_id=schema.schema_id,
                name=schema.name,
                version=schema.version,
                columns=schema.columns,
                validation_rules=schema.validation_rules,
                description=schema.description,
                created_at=schema.created_at,
                created_by=schema.created_by,
            )
            session.add(record)
            session.commit()

        logger.info(
            "Data schema created",
            schema_id=schema.schema_id,
            name=schema.name,
            version=schema.version,
        )

        return schema.schema_id

    async def create_feature_config(self, config: FeatureEngineeringConfig) -> str:
        """Create feature engineering configuration."""
        if not config.config_id:
            config.config_id = f"feature_config_{uuid.uuid4().hex[:8]}"

        # Store in database
        with self.SessionLocal() as session:
            record = FeatureEngineeringConfigRecord(
                config_id=config.config_id,
                name=config.name,
                numeric_features=config.numeric_features,
                categorical_features=config.categorical_features,
                text_features=config.text_features,
                scaling_method=config.scaling_method,
                encoding_method=config.encoding_method,
                feature_selection_enabled=config.feature_selection_enabled,
                max_features=config.max_features,
                correlation_threshold=config.correlation_threshold,
                text_vectorization=config.text_vectorization,
                max_vocab_size=config.max_vocab_size,
                custom_transformations=config.custom_transformations,
                created_at=config.created_at,
                created_by=config.created_by,
            )
            session.add(record)
            session.commit()

        logger.info(
            "Feature engineering configuration created",
            config_id=config.config_id,
            name=config.name,
        )

        return config.config_id

    async def run_pipeline(
        self,
        pipeline_id: str,
        input_data_path: str,
        schema_id: str,
        feature_config_id: str,
        trigger_type: str = "manual",
    ) -> str:
        """Execute the data pipeline."""
        run_id = f"run_{uuid.uuid4().hex[:8]}"

        # Calculate input data hash
        input_data_hash = await self._calculate_file_hash(input_data_path)

        # Get input data size
        df = pd.read_csv(input_data_path)
        input_rows = len(df)

        # Create pipeline run record
        pipeline_run = PipelineRun(
            run_id=run_id,
            pipeline_id=pipeline_id,
            trigger_type=trigger_type,
            input_data_path=input_data_path,
            input_data_hash=input_data_hash,
            input_rows=input_rows,
            schema_id=schema_id,
            feature_config_id=feature_config_id,
        )

        await self._store_pipeline_run(pipeline_run)

        # Execute pipeline asynchronously
        task = asyncio.create_task(self._execute_pipeline(pipeline_run))

        logger.info(
            "Data pipeline started", run_id=run_id, pipeline_id=pipeline_id, input_rows=input_rows
        )

        return run_id

    async def get_pipeline_run(self, run_id: str) -> PipelineRun | None:
        """Get pipeline run by ID."""
        with self.SessionLocal() as session:
            record = (
                session.query(PipelineRunRecord).filter(PipelineRunRecord.run_id == run_id).first()
            )

            if not record:
                return None

            return PipelineRun(
                run_id=record.run_id,
                pipeline_id=record.pipeline_id,
                trigger_type=record.trigger_type,
                input_data_path=record.input_data_path,
                input_data_hash=record.input_data_hash,
                input_rows=record.input_rows,
                schema_id=record.schema_id,
                feature_config_id=record.feature_config_id,
                output_data_path=record.output_data_path,
                output_data_hash=record.output_data_hash,
                output_rows=record.output_rows,
                quality_report_id=record.quality_report_id,
                data_quality_score=record.data_quality_score,
                processing_steps=record.processing_steps or [],
                feature_count=record.feature_count,
                status=PipelineStatus(record.status),
                started_at=record.started_at,
                completed_at=record.completed_at,
                processing_time_seconds=record.processing_time_seconds,
                processing_logs=record.processing_logs or [],
                error_message=record.error_message,
                created_at=record.created_at,
            )

    async def assess_data_quality(self, data_path: str, schema_id: str) -> DataQualityReport:
        """Assess data quality against schema."""
        report_id = f"quality_{uuid.uuid4().hex[:8]}"

        # Load data
        df = pd.read_csv(data_path)
        data_hash = await self._calculate_file_hash(data_path)

        # Get schema
        schema = await self.get_schema(schema_id)
        if not schema:
            raise ValueError(f"Schema {schema_id} not found")

        # Initialize report
        report = DataQualityReport(
            report_id=report_id,
            dataset_hash=data_hash,
            schema_id=schema_id,
            total_rows=len(df),
            total_columns=len(df.columns),
        )

        # Check for missing values
        missing_percentage = (df.isnull().sum().sum() / (len(df) * len(df.columns))) * 100
        report.missing_value_percentage = missing_percentage

        if missing_percentage > 10:  # More than 10% missing values
            report.issues.append(
                {
                    "type": DataQualityIssue.MISSING_VALUES.value,
                    "severity": "high" if missing_percentage > 25 else "medium",
                    "description": f"High percentage of missing values: {missing_percentage:.2f}%",
                    "affected_columns": df.columns[df.isnull().any()].tolist(),
                }
            )

        # Check for duplicates
        duplicate_rows = df.duplicated().sum()
        report.duplicate_rows = duplicate_rows

        if duplicate_rows > 0:
            report.issues.append(
                {
                    "type": DataQualityIssue.DUPLICATES.value,
                    "severity": "medium",
                    "description": f"Found {duplicate_rows} duplicate rows",
                    "count": duplicate_rows,
                }
            )

        # Schema validation
        for column_name, column_def in schema.columns.items():
            if column_name not in df.columns:
                report.issues.append(
                    {
                        "type": DataQualityIssue.SCHEMA_MISMATCH.value,
                        "severity": "high",
                        "description": f"Missing required column: {column_name}",
                        "column": column_name,
                    }
                )
                continue

            # Type validation
            expected_type = column_def.get("type", "object")
            actual_type = str(df[column_name].dtype)

            # Column-level statistics
            if df[column_name].dtype in ["int64", "float64"]:
                report.column_stats[column_name] = {
                    "type": actual_type,
                    "missing_count": df[column_name].isnull().sum(),
                    "mean": df[column_name].mean(),
                    "std": df[column_name].std(),
                    "min": df[column_name].min(),
                    "max": df[column_name].max(),
                    "unique_values": df[column_name].nunique(),
                }
            else:
                report.column_stats[column_name] = {
                    "type": actual_type,
                    "missing_count": df[column_name].isnull().sum(),
                    "unique_values": df[column_name].nunique(),
                    "most_frequent": (
                        df[column_name].mode().iloc[0] if not df[column_name].empty else None
                    ),
                }

        # Calculate overall quality score
        quality_score = 1.0
        quality_score -= min(0.5, missing_percentage / 100)  # Penalize missing values
        quality_score -= min(0.3, duplicate_rows / len(df))  # Penalize duplicates
        quality_score -= len([i for i in report.issues if i["severity"] == "high"]) * 0.2
        quality_score -= len([i for i in report.issues if i["severity"] == "medium"]) * 0.1

        report.quality_score = max(0.0, quality_score)

        # Store report
        await self._store_quality_report(report)

        logger.info(
            "Data quality assessment completed",
            report_id=report_id,
            quality_score=report.quality_score,
            issues_count=len(report.issues),
        )

        return report

    async def get_schema(self, schema_id: str) -> DataSchema | None:
        """Get data schema by ID."""
        with self.SessionLocal() as session:
            record = (
                session.query(DataSchemaRecord)
                .filter(DataSchemaRecord.schema_id == schema_id)
                .first()
            )

            if not record:
                return None

            return DataSchema(
                schema_id=record.schema_id,
                name=record.name,
                version=record.version,
                columns=record.columns,
                validation_rules=record.validation_rules or [],
                description=record.description or "",
                created_at=record.created_at,
                created_by=record.created_by or "",
            )

    async def get_feature_config(self, config_id: str) -> FeatureEngineeringConfig | None:
        """Get feature engineering configuration by ID."""
        with self.SessionLocal() as session:
            record = (
                session.query(FeatureEngineeringConfigRecord)
                .filter(FeatureEngineeringConfigRecord.config_id == config_id)
                .first()
            )

            if not record:
                return None

            return FeatureEngineeringConfig(
                config_id=record.config_id,
                name=record.name,
                numeric_features=record.numeric_features or [],
                categorical_features=record.categorical_features or [],
                text_features=record.text_features or [],
                scaling_method=record.scaling_method,
                encoding_method=record.encoding_method,
                feature_selection_enabled=record.feature_selection_enabled,
                max_features=record.max_features,
                correlation_threshold=record.correlation_threshold,
                text_vectorization=record.text_vectorization,
                max_vocab_size=record.max_vocab_size,
                custom_transformations=record.custom_transformations or [],
                created_at=record.created_at,
                created_by=record.created_by or "",
            )

    async def _execute_pipeline(self, pipeline_run: PipelineRun):
        """Execute the complete data pipeline."""
        try:
            start_time = datetime.utcnow()

            # Update status
            pipeline_run.status = PipelineStatus.RUNNING
            pipeline_run.started_at = start_time
            await self._update_pipeline_run(pipeline_run)

            # Step 1: Data Quality Assessment
            pipeline_run.processing_logs.append("Starting data quality assessment...")
            pipeline_run.processing_steps.append("data_quality_assessment")

            quality_report = await self.assess_data_quality(
                pipeline_run.input_data_path, pipeline_run.schema_id
            )

            pipeline_run.quality_report_id = quality_report.report_id
            pipeline_run.data_quality_score = quality_report.quality_score

            # Check if quality is acceptable
            if quality_report.quality_score < 0.5:
                raise Exception(f"Data quality too low: {quality_report.quality_score:.2f}")

            # Step 2: Data Preprocessing
            pipeline_run.processing_logs.append("Starting data preprocessing...")
            pipeline_run.processing_steps.append("data_preprocessing")

            df = pd.read_csv(pipeline_run.input_data_path)
            df = await self._preprocess_data(df, quality_report)

            # Step 3: Feature Engineering
            pipeline_run.processing_logs.append("Starting feature engineering...")
            pipeline_run.processing_steps.append("feature_engineering")

            feature_config = await self.get_feature_config(pipeline_run.feature_config_id)
            if not feature_config:
                raise Exception(f"Feature config {pipeline_run.feature_config_id} not found")

            df_features = await self._engineer_features(df, feature_config)
            pipeline_run.feature_count = len(df_features.columns)

            # Step 4: Save processed data
            output_filename = f"processed_{pipeline_run.run_id}.csv"
            output_path = self.processed_data_dir / output_filename

            df_features.to_csv(output_path, index=False)

            pipeline_run.output_data_path = str(output_path)
            pipeline_run.output_data_hash = await self._calculate_file_hash(str(output_path))
            pipeline_run.output_rows = len(df_features)

            # Complete pipeline
            end_time = datetime.utcnow()
            pipeline_run.status = PipelineStatus.COMPLETED
            pipeline_run.completed_at = end_time
            pipeline_run.processing_time_seconds = (end_time - start_time).total_seconds()
            pipeline_run.processing_logs.append("Pipeline completed successfully")

            await self._update_pipeline_run(pipeline_run)

            logger.info(
                "Data pipeline completed",
                run_id=pipeline_run.run_id,
                input_rows=pipeline_run.input_rows,
                output_rows=pipeline_run.output_rows,
                feature_count=pipeline_run.feature_count,
                processing_time=pipeline_run.processing_time_seconds,
            )

        except Exception as e:
            # Handle pipeline failure
            pipeline_run.status = PipelineStatus.FAILED
            pipeline_run.completed_at = datetime.utcnow()
            pipeline_run.error_message = str(e)
            pipeline_run.processing_logs.append(f"Pipeline failed: {str(e)}")

            await self._update_pipeline_run(pipeline_run)

            logger.error("Data pipeline failed", run_id=pipeline_run.run_id, error=str(e))

    async def _preprocess_data(
        self, df: pd.DataFrame, quality_report: DataQualityReport
    ) -> pd.DataFrame:
        """Preprocess data based on quality assessment."""
        processed_df = df.copy()

        # Handle missing values
        if quality_report.missing_value_percentage > 0:
            for column in processed_df.columns:
                if processed_df[column].isnull().any():
                    if processed_df[column].dtype in ["int64", "float64"]:
                        # Fill numeric columns with median
                        processed_df[column].fillna(processed_df[column].median(), inplace=True)
                    else:
                        # Fill categorical columns with mode
                        mode_value = processed_df[column].mode()
                        if not mode_value.empty:
                            processed_df[column].fillna(mode_value.iloc[0], inplace=True)

        # Remove duplicates
        if quality_report.duplicate_rows > 0:
            processed_df = processed_df.drop_duplicates()

        # Handle outliers using IQR method
        for column in processed_df.select_dtypes(include=[np.number]).columns:
            Q1 = processed_df[column].quantile(0.25)
            Q3 = processed_df[column].quantile(0.75)
            IQR = Q3 - Q1
            lower_bound = Q1 - 1.5 * IQR
            upper_bound = Q3 + 1.5 * IQR

            # Cap outliers
            processed_df[column] = processed_df[column].clip(lower=lower_bound, upper=upper_bound)

        return processed_df

    async def _engineer_features(
        self, df: pd.DataFrame, config: FeatureEngineeringConfig
    ) -> pd.DataFrame:
        """Apply feature engineering transformations."""
        features_df = df.copy()

        # Numeric feature scaling
        if config.numeric_features:
            numeric_columns = [col for col in config.numeric_features if col in features_df.columns]
            if numeric_columns:
                if config.scaling_method == "standard":
                    scaler = StandardScaler()
                elif config.scaling_method == "minmax":
                    from sklearn.preprocessing import MinMaxScaler

                    scaler = MinMaxScaler()
                elif config.scaling_method == "robust":
                    from sklearn.preprocessing import RobustScaler

                    scaler = RobustScaler()
                else:
                    scaler = StandardScaler()

                features_df[numeric_columns] = scaler.fit_transform(features_df[numeric_columns])

        # Categorical feature encoding
        if config.categorical_features:
            categorical_columns = [
                col for col in config.categorical_features if col in features_df.columns
            ]
            for col in categorical_columns:
                if config.encoding_method == "label":
                    le = LabelEncoder()
                    features_df[col] = le.fit_transform(features_df[col].astype(str))
                elif config.encoding_method == "onehot":
                    # One-hot encoding
                    dummies = pd.get_dummies(features_df[col], prefix=col)
                    features_df = pd.concat([features_df, dummies], axis=1)
                    features_df = features_df.drop(col, axis=1)

        # Text feature processing
        if config.text_features:
            from sklearn.feature_extraction.text import TfidfVectorizer

            for col in config.text_features:
                if col in features_df.columns:
                    if config.text_vectorization == "tfidf":
                        vectorizer = TfidfVectorizer(
                            max_features=config.max_vocab_size, stop_words="english"
                        )

                        tfidf_matrix = vectorizer.fit_transform(features_df[col].astype(str))
                        tfidf_df = pd.DataFrame(
                            tfidf_matrix.toarray(),
                            columns=[f"{col}_tfidf_{i}" for i in range(tfidf_matrix.shape[1])],
                        )

                        features_df = pd.concat([features_df, tfidf_df], axis=1)
                        features_df = features_df.drop(col, axis=1)

        # Feature selection
        if config.feature_selection_enabled:
            features_df = await self._select_features(features_df, config)

        # Apply custom transformations
        for transformation in config.custom_transformations:
            transform_name = transformation.get("name")
            if transform_name in self.transformation_functions:
                features_df = self.transformation_functions[transform_name](
                    features_df, transformation
                )

        return features_df

    async def _select_features(
        self, df: pd.DataFrame, config: FeatureEngineeringConfig
    ) -> pd.DataFrame:
        """Select features based on correlation and importance."""
        # Remove highly correlated features
        if config.correlation_threshold < 1.0:
            # Calculate correlation matrix for numeric features only
            numeric_df = df.select_dtypes(include=[np.number])
            if not numeric_df.empty:
                corr_matrix = numeric_df.corr().abs()

                # Find highly correlated feature pairs
                upper_triangle = corr_matrix.where(
                    np.triu(np.ones(corr_matrix.shape), k=1).astype(bool)
                )

                # Select features to drop
                to_drop = [
                    column
                    for column in upper_triangle.columns
                    if any(upper_triangle[column] > config.correlation_threshold)
                ]

                df = df.drop(columns=to_drop)

        # Limit number of features if specified
        if config.max_features and len(df.columns) > config.max_features:
            # Simple feature selection - keep first N features
            # In production, use more sophisticated methods like SelectKBest
            df = df.iloc[:, : config.max_features]

        return df

    async def _calculate_file_hash(self, file_path: str) -> str:
        """Calculate hash of a file."""
        hash_md5 = hashlib.md5()
        async with aiofiles.open(file_path, "rb") as f:
            async for chunk in f:
                hash_md5.update(chunk)
        return hash_md5.hexdigest()

    async def _store_pipeline_run(self, run: PipelineRun):
        """Store pipeline run in database."""
        with self.SessionLocal() as session:
            record = PipelineRunRecord(
                run_id=run.run_id,
                pipeline_id=run.pipeline_id,
                trigger_type=run.trigger_type,
                input_data_path=run.input_data_path,
                input_data_hash=run.input_data_hash,
                input_rows=run.input_rows,
                schema_id=run.schema_id,
                feature_config_id=run.feature_config_id,
                output_data_path=run.output_data_path,
                output_data_hash=run.output_data_hash,
                output_rows=run.output_rows,
                quality_report_id=run.quality_report_id,
                data_quality_score=run.data_quality_score,
                processing_steps=run.processing_steps,
                feature_count=run.feature_count,
                status=run.status.value,
                started_at=run.started_at,
                completed_at=run.completed_at,
                processing_time_seconds=run.processing_time_seconds,
                processing_logs=run.processing_logs,
                error_message=run.error_message,
                created_at=run.created_at,
            )
            session.add(record)
            session.commit()

    async def _update_pipeline_run(self, run: PipelineRun):
        """Update pipeline run in database."""
        with self.SessionLocal() as session:
            record = (
                session.query(PipelineRunRecord)
                .filter(PipelineRunRecord.run_id == run.run_id)
                .first()
            )

            if record:
                record.status = run.status.value
                record.started_at = run.started_at
                record.completed_at = run.completed_at
                record.output_data_path = run.output_data_path
                record.output_data_hash = run.output_data_hash
                record.output_rows = run.output_rows
                record.quality_report_id = run.quality_report_id
                record.data_quality_score = run.data_quality_score
                record.processing_steps = run.processing_steps
                record.feature_count = run.feature_count
                record.processing_time_seconds = run.processing_time_seconds
                record.processing_logs = run.processing_logs
                record.error_message = run.error_message
                session.commit()

    async def _store_quality_report(self, report: DataQualityReport):
        """Store data quality report in database."""
        with self.SessionLocal() as session:
            record = DataQualityReportRecord(
                report_id=report.report_id,
                dataset_hash=report.dataset_hash,
                schema_id=report.schema_id,
                total_rows=report.total_rows,
                total_columns=report.total_columns,
                missing_value_percentage=report.missing_value_percentage,
                duplicate_rows=report.duplicate_rows,
                issues=report.issues,
                column_stats=report.column_stats,
                quality_score=report.quality_score,
                created_at=report.created_at,
            )
            session.add(record)
            session.commit()
