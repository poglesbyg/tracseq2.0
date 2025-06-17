"""
Model Registry for TracSeq 2.0 MLOps Pipeline

Handles model versioning, metadata tracking, and lifecycle management.
"""

import json
import hashlib
import pickle
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Any, Union
from dataclasses import dataclass, asdict
from enum import Enum
import asyncio
import aiofiles
import structlog
from sqlalchemy import create_engine, Column, String, DateTime, JSON, Float, Integer
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker

logger = structlog.get_logger(__name__)

class ModelStatus(Enum):
    TRAINING = "training"
    VALIDATING = "validating"
    STAGING = "staging"
    PRODUCTION = "production"
    ARCHIVED = "archived"
    FAILED = "failed"

@dataclass
class ModelMetadata:
    """Comprehensive model metadata tracking"""
    model_id: str
    version: str
    name: str
    description: str
    model_type: str  # e.g., "rag_extractor", "confidence_scorer", "quality_classifier"
    framework: str   # e.g., "transformers", "scikit-learn", "pytorch"
    
    # Performance metrics
    accuracy: float
    precision: float
    recall: float
    f1_score: float
    confidence_score: float
    
    # Training information
    training_data_hash: str
    training_duration_seconds: float
    hyperparameters: Dict[str, Any]
    feature_columns: List[str]
    
    # Deployment information
    status: ModelStatus
    created_at: datetime
    updated_at: datetime
    created_by: str
    
    # Model artifacts
    model_path: str
    config_path: str
    requirements_path: Optional[str] = None
    
    # Performance tracking
    prediction_count: int = 0
    average_inference_time_ms: float = 0.0
    error_rate: float = 0.0
    
    # A/B testing
    traffic_percentage: float = 0.0
    ab_test_id: Optional[str] = None

Base = declarative_base()

class ModelRecord(Base):
    """SQLAlchemy model for persistent storage"""
    __tablename__ = "model_registry"
    
    model_id = Column(String, primary_key=True)
    version = Column(String, primary_key=True)
    name = Column(String, nullable=False)
    description = Column(String)
    model_type = Column(String, nullable=False)
    framework = Column(String, nullable=False)
    
    # Performance metrics as JSON
    metrics = Column(JSON)
    
    # Training info
    training_data_hash = Column(String)
    training_duration_seconds = Column(Float)
    hyperparameters = Column(JSON)
    feature_columns = Column(JSON)
    
    # Status and timestamps
    status = Column(String, nullable=False)
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)
    created_by = Column(String, nullable=False)
    
    # Paths
    model_path = Column(String, nullable=False)
    config_path = Column(String, nullable=False)
    requirements_path = Column(String)
    
    # Performance tracking
    prediction_count = Column(Integer, default=0)
    average_inference_time_ms = Column(Float, default=0.0)
    error_rate = Column(Float, default=0.0)
    
    # A/B testing
    traffic_percentage = Column(Float, default=0.0)
    ab_test_id = Column(String)

class ModelRegistry:
    """
    Advanced model registry with versioning, metadata tracking, and lifecycle management.
    """
    
    def __init__(self, registry_path: Union[str, Path], database_url: str):
        self.registry_path = Path(registry_path)
        self.registry_path.mkdir(parents=True, exist_ok=True)
        
        # Database setup
        self.engine = create_engine(database_url)
        Base.metadata.create_all(self.engine)
        self.SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=self.engine)
        
        # Create directories
        self.models_dir = self.registry_path / "models"
        self.configs_dir = self.registry_path / "configs"
        self.artifacts_dir = self.registry_path / "artifacts"
        
        for dir_path in [self.models_dir, self.configs_dir, self.artifacts_dir]:
            dir_path.mkdir(parents=True, exist_ok=True)
    
    async def register_model(
        self,
        model: Any,
        metadata: ModelMetadata,
        config: Dict[str, Any],
        requirements: Optional[List[str]] = None
    ) -> str:
        """
        Register a new model with full metadata and artifacts.
        
        Args:
            model: The trained model object
            metadata: Comprehensive model metadata
            config: Model configuration dictionary
            requirements: Optional list of Python package requirements
            
        Returns:
            str: Model registration ID
        """
        try:
            # Generate unique paths
            model_filename = f"{metadata.model_id}_v{metadata.version}.pkl"
            config_filename = f"{metadata.model_id}_v{metadata.version}_config.json"
            requirements_filename = f"{metadata.model_id}_v{metadata.version}_requirements.txt"
            
            model_path = self.models_dir / model_filename
            config_path = self.configs_dir / config_filename
            requirements_path = self.artifacts_dir / requirements_filename if requirements else None
            
            # Save model artifacts
            await self._save_model_artifacts(
                model, model_path, config, config_path, requirements, requirements_path
            )
            
            # Update metadata with paths
            metadata.model_path = str(model_path)
            metadata.config_path = str(config_path)
            metadata.requirements_path = str(requirements_path) if requirements_path else None
            
            # Store in database
            await self._store_model_metadata(metadata)
            
            logger.info(
                "Model registered successfully",
                model_id=metadata.model_id,
                version=metadata.version,
                model_type=metadata.model_type
            )
            
            return f"{metadata.model_id}:{metadata.version}"
            
        except Exception as e:
            logger.error(
                "Failed to register model",
                model_id=metadata.model_id,
                error=str(e)
            )
            raise
    
    async def get_model(self, model_id: str, version: Optional[str] = None) -> tuple[Any, ModelMetadata]:
        """
        Retrieve a model and its metadata.
        
        Args:
            model_id: Model identifier
            version: Specific version (latest if None)
            
        Returns:
            tuple: (model_object, metadata)
        """
        metadata = await self.get_model_metadata(model_id, version)
        if not metadata:
            raise ValueError(f"Model {model_id}:{version} not found")
        
        # Load model from disk
        async with aiofiles.open(metadata.model_path, 'rb') as f:
            model_bytes = await f.read()
            model = pickle.loads(model_bytes)
        
        return model, metadata
    
    async def get_model_metadata(self, model_id: str, version: Optional[str] = None) -> Optional[ModelMetadata]:
        """Get model metadata from registry."""
        with self.SessionLocal() as session:
            query = session.query(ModelRecord).filter(ModelRecord.model_id == model_id)
            
            if version:
                query = query.filter(ModelRecord.version == version)
            else:
                # Get latest version
                query = query.order_by(ModelRecord.created_at.desc())
            
            record = query.first()
            if not record:
                return None
            
            return self._record_to_metadata(record)
    
    async def list_models(
        self,
        model_type: Optional[str] = None,
        status: Optional[ModelStatus] = None,
        limit: int = 50
    ) -> List[ModelMetadata]:
        """List models with optional filtering."""
        with self.SessionLocal() as session:
            query = session.query(ModelRecord)
            
            if model_type:
                query = query.filter(ModelRecord.model_type == model_type)
            if status:
                query = query.filter(ModelRecord.status == status.value)
            
            query = query.order_by(ModelRecord.created_at.desc()).limit(limit)
            records = query.all()
            
            return [self._record_to_metadata(record) for record in records]
    
    async def update_model_status(self, model_id: str, version: str, status: ModelStatus) -> bool:
        """Update model status."""
        with self.SessionLocal() as session:
            record = session.query(ModelRecord).filter(
                ModelRecord.model_id == model_id,
                ModelRecord.version == version
            ).first()
            
            if not record:
                return False
            
            record.status = status.value
            record.updated_at = datetime.utcnow()
            session.commit()
            
            logger.info(
                "Model status updated",
                model_id=model_id,
                version=version,
                status=status.value
            )
            return True
    
    async def update_performance_metrics(
        self,
        model_id: str,
        version: str,
        prediction_count: int,
        inference_time_ms: float,
        error_rate: float
    ) -> bool:
        """Update real-time performance metrics."""
        with self.SessionLocal() as session:
            record = session.query(ModelRecord).filter(
                ModelRecord.model_id == model_id,
                ModelRecord.version == version
            ).first()
            
            if not record:
                return False
            
            # Update metrics with exponential moving average
            alpha = 0.1  # Smoothing factor
            record.prediction_count += prediction_count
            record.average_inference_time_ms = (
                alpha * inference_time_ms + (1 - alpha) * record.average_inference_time_ms
            )
            record.error_rate = alpha * error_rate + (1 - alpha) * record.error_rate
            record.updated_at = datetime.utcnow()
            
            session.commit()
            return True
    
    async def promote_model(self, model_id: str, version: str, to_status: ModelStatus) -> bool:
        """Promote model through deployment pipeline."""
        current_metadata = await self.get_model_metadata(model_id, version)
        if not current_metadata:
            return False
        
        # Validation rules for promotion
        promotion_rules = {
            ModelStatus.STAGING: lambda m: m.status == ModelStatus.VALIDATING and m.accuracy > 0.90,
            ModelStatus.PRODUCTION: lambda m: m.status == ModelStatus.STAGING and m.error_rate < 0.05,
        }
        
        if to_status in promotion_rules:
            if not promotion_rules[to_status](current_metadata):
                logger.warning(
                    "Model promotion blocked due to validation rules",
                    model_id=model_id,
                    version=version,
                    current_status=current_metadata.status,
                    target_status=to_status
                )
                return False
        
        # Demote current production model if promoting to production
        if to_status == ModelStatus.PRODUCTION:
            await self._demote_current_production_model(model_id)
        
        return await self.update_model_status(model_id, version, to_status)
    
    async def delete_model(self, model_id: str, version: str) -> bool:
        """Delete model and its artifacts."""
        metadata = await self.get_model_metadata(model_id, version)
        if not metadata:
            return False
        
        # Don't delete production models
        if metadata.status == ModelStatus.PRODUCTION:
            logger.warning(
                "Cannot delete production model",
                model_id=model_id,
                version=version
            )
            return False
        
        # Delete files
        for path_str in [metadata.model_path, metadata.config_path, metadata.requirements_path]:
            if path_str:
                path = Path(path_str)
                if path.exists():
                    path.unlink()
        
        # Delete from database
        with self.SessionLocal() as session:
            record = session.query(ModelRecord).filter(
                ModelRecord.model_id == model_id,
                ModelRecord.version == version
            ).first()
            
            if record:
                session.delete(record)
                session.commit()
        
        logger.info("Model deleted", model_id=model_id, version=version)
        return True
    
    async def get_production_model(self, model_type: str) -> Optional[tuple[Any, ModelMetadata]]:
        """Get current production model of specified type."""
        with self.SessionLocal() as session:
            record = session.query(ModelRecord).filter(
                ModelRecord.model_type == model_type,
                ModelRecord.status == ModelStatus.PRODUCTION.value
            ).first()
            
            if not record:
                return None
            
            metadata = self._record_to_metadata(record)
            
            # Load model
            async with aiofiles.open(metadata.model_path, 'rb') as f:
                model_bytes = await f.read()
                model = pickle.loads(model_bytes)
            
            return model, metadata
    
    async def _save_model_artifacts(
        self,
        model: Any,
        model_path: Path,
        config: Dict[str, Any],
        config_path: Path,
        requirements: Optional[List[str]],
        requirements_path: Optional[Path]
    ):
        """Save model artifacts to disk."""
        # Save model
        async with aiofiles.open(model_path, 'wb') as f:
            model_bytes = pickle.dumps(model)
            await f.write(model_bytes)
        
        # Save config
        async with aiofiles.open(config_path, 'w') as f:
            await f.write(json.dumps(config, indent=2))
        
        # Save requirements
        if requirements and requirements_path:
            async with aiofiles.open(requirements_path, 'w') as f:
                await f.write('\n'.join(requirements))
    
    async def _store_model_metadata(self, metadata: ModelMetadata):
        """Store model metadata in database."""
        with self.SessionLocal() as session:
            record = ModelRecord(
                model_id=metadata.model_id,
                version=metadata.version,
                name=metadata.name,
                description=metadata.description,
                model_type=metadata.model_type,
                framework=metadata.framework,
                metrics={
                    "accuracy": metadata.accuracy,
                    "precision": metadata.precision,
                    "recall": metadata.recall,
                    "f1_score": metadata.f1_score,
                    "confidence_score": metadata.confidence_score
                },
                training_data_hash=metadata.training_data_hash,
                training_duration_seconds=metadata.training_duration_seconds,
                hyperparameters=metadata.hyperparameters,
                feature_columns=metadata.feature_columns,
                status=metadata.status.value,
                created_at=metadata.created_at,
                updated_at=metadata.updated_at,
                created_by=metadata.created_by,
                model_path=metadata.model_path,
                config_path=metadata.config_path,
                requirements_path=metadata.requirements_path,
                prediction_count=metadata.prediction_count,
                average_inference_time_ms=metadata.average_inference_time_ms,
                error_rate=metadata.error_rate,
                traffic_percentage=metadata.traffic_percentage,
                ab_test_id=metadata.ab_test_id
            )
            
            session.add(record)
            session.commit()
    
    def _record_to_metadata(self, record: ModelRecord) -> ModelMetadata:
        """Convert database record to metadata object."""
        return ModelMetadata(
            model_id=record.model_id,
            version=record.version,
            name=record.name,
            description=record.description,
            model_type=record.model_type,
            framework=record.framework,
            accuracy=record.metrics.get("accuracy", 0.0),
            precision=record.metrics.get("precision", 0.0),
            recall=record.metrics.get("recall", 0.0),
            f1_score=record.metrics.get("f1_score", 0.0),
            confidence_score=record.metrics.get("confidence_score", 0.0),
            training_data_hash=record.training_data_hash,
            training_duration_seconds=record.training_duration_seconds,
            hyperparameters=record.hyperparameters,
            feature_columns=record.feature_columns,
            status=ModelStatus(record.status),
            created_at=record.created_at,
            updated_at=record.updated_at,
            created_by=record.created_by,
            model_path=record.model_path,
            config_path=record.config_path,
            requirements_path=record.requirements_path,
            prediction_count=record.prediction_count,
            average_inference_time_ms=record.average_inference_time_ms,
            error_rate=record.error_rate,
            traffic_percentage=record.traffic_percentage,
            ab_test_id=record.ab_test_id
        )
    
    async def _demote_current_production_model(self, model_id: str):
        """Demote current production model to staging."""
        with self.SessionLocal() as session:
            current_prod = session.query(ModelRecord).filter(
                ModelRecord.model_id == model_id,
                ModelRecord.status == ModelStatus.PRODUCTION.value
            ).first()
            
            if current_prod:
                current_prod.status = ModelStatus.STAGING.value
                current_prod.updated_at = datetime.utcnow()
                session.commit()
                
                logger.info(
                    "Previous production model demoted to staging",
                    model_id=current_prod.model_id,
                    version=current_prod.version
                ) 
