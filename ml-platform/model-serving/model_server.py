# TracSeq 2.0 - ML Model Serving Infrastructure
# Real-time prediction service with versioning and A/B testing

import asyncio
import json
import logging
from abc import ABC, abstractmethod
from dataclasses import dataclass
from datetime import datetime
from enum import Enum
from typing import Any, Dict, List, Optional, Union
from uuid import UUID, uuid4

import numpy as np
import pandas as pd
from fastapi import FastAPI, HTTPException, BackgroundTasks
from fastapi.responses import JSONResponse
from pydantic import BaseModel, Field
import redis
from sqlalchemy import create_engine, Column, String, Float, Integer, DateTime, JSON
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker
import mlflow
from prometheus_client import Counter, Histogram, Gauge
import joblib
import pickle

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Database Models
Base = declarative_base()

class ModelStatus(str, Enum):
    STAGING = "staging"
    PRODUCTION = "production"
    ARCHIVED = "archived"
    FAILED = "failed"

class ModelType(str, Enum):
    SAMPLE_QUALITY = "sample_quality"
    STORAGE_OPTIMIZATION = "storage_optimization"
    SEQUENCING_SUCCESS = "sequencing_success"
    CONTAMINATION_DETECTION = "contamination_detection"
    WORKFLOW_DURATION = "workflow_duration"

# Request/Response Models
class PredictionRequest(BaseModel):
    model_type: ModelType
    features: Dict[str, Any]
    request_id: Optional[str] = Field(default_factory=lambda: str(uuid4()))
    experiment_id: Optional[str] = None
    metadata: Optional[Dict[str, Any]] = {}

class PredictionResponse(BaseModel):
    request_id: str
    model_type: ModelType
    model_version: str
    prediction: Union[float, int, str, Dict[str, Any]]
    confidence: Optional[float] = None
    feature_importance: Optional[Dict[str, float]] = None
    prediction_time_ms: float
    metadata: Dict[str, Any] = {}

class BatchPredictionRequest(BaseModel):
    model_type: ModelType
    features_batch: List[Dict[str, Any]]
    request_id: Optional[str] = Field(default_factory=lambda: str(uuid4()))

# Database Models
class ModelRegistry(Base):
    __tablename__ = "model_registry"
    
    id = Column(String, primary_key=True)
    model_type = Column(String, nullable=False)
    version = Column(String, nullable=False)
    status = Column(String, nullable=False)
    metrics = Column(JSON)
    model_path = Column(String, nullable=False)
    feature_schema = Column(JSON)
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)
    metadata = Column(JSON)

class PredictionLog(Base):
    __tablename__ = "prediction_logs"
    
    id = Column(String, primary_key=True)
    request_id = Column(String, nullable=False)
    model_id = Column(String, nullable=False)
    model_type = Column(String, nullable=False)
    model_version = Column(String, nullable=False)
    features = Column(JSON)
    prediction = Column(JSON)
    confidence = Column(Float)
    prediction_time_ms = Column(Float)
    created_at = Column(DateTime, default=datetime.utcnow)

# Model Interface
class BaseModel(ABC):
    """Abstract base class for all ML models"""
    
    def __init__(self, model_id: str, version: str):
        self.model_id = model_id
        self.version = version
        self.model = None
        self.feature_names = []
        self.model_type = None
        
    @abstractmethod
    async def load(self, model_path: str):
        """Load model from storage"""
        pass
    
    @abstractmethod
    async def predict(self, features: Dict[str, Any]) -> Dict[str, Any]:
        """Make prediction"""
        pass
    
    @abstractmethod
    async def validate_features(self, features: Dict[str, Any]) -> bool:
        """Validate input features"""
        pass
    
    async def explain(self, features: Dict[str, Any]) -> Dict[str, float]:
        """Get feature importance for prediction"""
        return {}

# Sample Quality Model
class SampleQualityModel(BaseModel):
    """Predicts sample quality score based on collection and storage parameters"""
    
    def __init__(self, model_id: str, version: str):
        super().__init__(model_id, version)
        self.model_type = ModelType.SAMPLE_QUALITY
        self.feature_names = [
            "collection_time_hours",
            "temperature_celsius",
            "volume_ml",
            "sample_type",
            "storage_duration_hours",
            "processing_delay_hours",
            "contamination_risk_score"
        ]
    
    async def load(self, model_path: str):
        """Load sample quality model"""
        try:
            self.model = joblib.load(model_path)
            logger.info(f"Loaded sample quality model from {model_path}")
        except Exception as e:
            logger.error(f"Failed to load model: {e}")
            raise
    
    async def predict(self, features: Dict[str, Any]) -> Dict[str, Any]:
        """Predict sample quality score"""
        # Validate features
        if not await self.validate_features(features):
            raise ValueError("Invalid features for sample quality model")
        
        # Prepare features
        feature_vector = self._prepare_features(features)
        
        # Make prediction
        quality_score = float(self.model.predict(feature_vector)[0])
        confidence = self._calculate_confidence(feature_vector)
        
        # Determine quality category
        if quality_score >= 0.8:
            quality_category = "excellent"
        elif quality_score >= 0.6:
            quality_category = "good"
        elif quality_score >= 0.4:
            quality_category = "fair"
        else:
            quality_category = "poor"
        
        return {
            "quality_score": quality_score,
            "quality_category": quality_category,
            "confidence": confidence,
            "recommendations": self._generate_recommendations(quality_score, features)
        }
    
    async def validate_features(self, features: Dict[str, Any]) -> bool:
        """Validate required features are present"""
        required_features = [
            "collection_time_hours",
            "temperature_celsius",
            "volume_ml",
            "sample_type"
        ]
        return all(f in features for f in required_features)
    
    def _prepare_features(self, features: Dict[str, Any]) -> np.ndarray:
        """Convert features to model input format"""
        # One-hot encode sample type
        sample_type_encoded = [0, 0, 0, 0]  # blood, tissue, dna, rna
        sample_types = ["blood", "tissue", "dna", "rna"]
        if features["sample_type"] in sample_types:
            sample_type_encoded[sample_types.index(features["sample_type"])] = 1
        
        feature_vector = [
            features.get("collection_time_hours", 0),
            features.get("temperature_celsius", -80),
            features.get("volume_ml", 5),
            *sample_type_encoded,
            features.get("storage_duration_hours", 0),
            features.get("processing_delay_hours", 0),
            features.get("contamination_risk_score", 0)
        ]
        
        return np.array([feature_vector])
    
    def _calculate_confidence(self, feature_vector: np.ndarray) -> float:
        """Calculate prediction confidence"""
        if hasattr(self.model, 'predict_proba'):
            probas = self.model.predict_proba(feature_vector)[0]
            return float(max(probas))
        return 0.85  # Default confidence
    
    def _generate_recommendations(self, quality_score: float, features: Dict[str, Any]) -> List[str]:
        """Generate recommendations based on quality score"""
        recommendations = []
        
        if quality_score < 0.6:
            if features.get("temperature_celsius", -80) > -70:
                recommendations.append("Store sample at -80°C for optimal quality")
            if features.get("processing_delay_hours", 0) > 24:
                recommendations.append("Process samples within 24 hours of collection")
            if features.get("volume_ml", 0) < 3:
                recommendations.append("Collect at least 3ml for reliable analysis")
        
        return recommendations

# Model Server
class ModelServer:
    """ML Model serving infrastructure with A/B testing and versioning"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.models: Dict[str, Dict[str, BaseModel]] = {}
        self.redis_client = None
        self.db_session = None
        self.metrics = self._init_metrics()
        
    def _init_metrics(self):
        """Initialize Prometheus metrics"""
        return {
            "prediction_count": Counter(
                "ml_predictions_total",
                "Total number of predictions",
                ["model_type", "model_version", "status"]
            ),
            "prediction_latency": Histogram(
                "ml_prediction_duration_seconds",
                "Prediction latency",
                ["model_type", "model_version"]
            ),
            "model_accuracy": Gauge(
                "ml_model_accuracy",
                "Model accuracy score",
                ["model_type", "model_version"]
            ),
            "active_models": Gauge(
                "ml_active_models",
                "Number of active models",
                ["model_type"]
            )
        }
    
    async def initialize(self):
        """Initialize model server"""
        # Connect to Redis for caching
        self.redis_client = redis.Redis(
            host=self.config["redis_host"],
            port=self.config["redis_port"],
            decode_responses=True
        )
        
        # Initialize database
        engine = create_engine(self.config["database_url"])
        Base.metadata.create_all(engine)
        Session = sessionmaker(bind=engine)
        self.db_session = Session()
        
        # Load active models
        await self.load_active_models()
    
    async def load_active_models(self):
        """Load all active models from registry"""
        active_models = self.db_session.query(ModelRegistry).filter(
            ModelRegistry.status.in_([ModelStatus.STAGING, ModelStatus.PRODUCTION])
        ).all()
        
        for model_record in active_models:
            await self.load_model(
                model_record.model_type,
                model_record.version,
                model_record.model_path
            )
    
    async def load_model(self, model_type: str, version: str, model_path: str):
        """Load a specific model version"""
        try:
            # Create model instance based on type
            if model_type == ModelType.SAMPLE_QUALITY:
                model = SampleQualityModel(str(uuid4()), version)
            else:
                raise ValueError(f"Unknown model type: {model_type}")
            
            # Load model weights
            await model.load(model_path)
            
            # Store in model registry
            if model_type not in self.models:
                self.models[model_type] = {}
            self.models[model_type][version] = model
            
            logger.info(f"Loaded model {model_type} version {version}")
            self.metrics["active_models"].labels(model_type=model_type).inc()
            
        except Exception as e:
            logger.error(f"Failed to load model {model_type} v{version}: {e}")
            raise
    
    async def predict(self, request: PredictionRequest) -> PredictionResponse:
        """Make prediction with model selection and A/B testing"""
        start_time = datetime.utcnow()
        
        try:
            # Select model version (A/B testing logic)
            model = await self._select_model(request)
            
            # Check cache
            cache_key = self._get_cache_key(request)
            cached_result = self.redis_client.get(cache_key)
            if cached_result and not request.experiment_id:
                return PredictionResponse(**json.loads(cached_result))
            
            # Make prediction
            prediction_result = await model.predict(request.features)
            
            # Calculate prediction time
            prediction_time_ms = (datetime.utcnow() - start_time).total_seconds() * 1000
            
            # Create response
            response = PredictionResponse(
                request_id=request.request_id,
                model_type=request.model_type,
                model_version=model.version,
                prediction=prediction_result.get("quality_score", prediction_result),
                confidence=prediction_result.get("confidence"),
                prediction_time_ms=prediction_time_ms,
                metadata={
                    **request.metadata,
                    "quality_category": prediction_result.get("quality_category"),
                    "recommendations": prediction_result.get("recommendations", [])
                }
            )
            
            # Cache result
            self.redis_client.setex(
                cache_key,
                300,  # 5 minutes TTL
                response.json()
            )
            
            # Log prediction
            await self._log_prediction(request, response, model)
            
            # Update metrics
            self.metrics["prediction_count"].labels(
                model_type=request.model_type,
                model_version=model.version,
                status="success"
            ).inc()
            
            self.metrics["prediction_latency"].labels(
                model_type=request.model_type,
                model_version=model.version
            ).observe(prediction_time_ms / 1000)
            
            return response
            
        except Exception as e:
            logger.error(f"Prediction failed: {e}")
            self.metrics["prediction_count"].labels(
                model_type=request.model_type,
                model_version="unknown",
                status="failure"
            ).inc()
            raise HTTPException(status_code=500, detail=str(e))
    
    async def _select_model(self, request: PredictionRequest) -> BaseModel:
        """Select model version based on A/B testing configuration"""
        model_type = request.model_type.value
        
        if model_type not in self.models:
            raise ValueError(f"No models available for type: {model_type}")
        
        available_versions = self.models[model_type]
        
        # If experiment ID provided, use consistent hashing for A/B test
        if request.experiment_id:
            # Simple hash-based selection for A/B testing
            hash_value = hash(request.experiment_id)
            version_index = hash_value % len(available_versions)
            selected_version = list(available_versions.keys())[version_index]
        else:
            # Default to production version
            production_versions = [
                v for v, m in available_versions.items()
                if self._is_production_version(model_type, v)
            ]
            selected_version = production_versions[0] if production_versions else list(available_versions.keys())[0]
        
        return available_versions[selected_version]
    
    def _is_production_version(self, model_type: str, version: str) -> bool:
        """Check if a model version is in production"""
        model_record = self.db_session.query(ModelRegistry).filter(
            ModelRegistry.model_type == model_type,
            ModelRegistry.version == version
        ).first()
        
        return model_record and model_record.status == ModelStatus.PRODUCTION
    
    def _get_cache_key(self, request: PredictionRequest) -> str:
        """Generate cache key for prediction request"""
        feature_hash = hash(json.dumps(request.features, sort_keys=True))
        return f"prediction:{request.model_type}:{feature_hash}"
    
    async def _log_prediction(self, request: PredictionRequest, response: PredictionResponse, model: BaseModel):
        """Log prediction for monitoring and analysis"""
        log_entry = PredictionLog(
            id=str(uuid4()),
            request_id=request.request_id,
            model_id=model.model_id,
            model_type=request.model_type.value,
            model_version=model.version,
            features=request.features,
            prediction={"value": response.prediction, "metadata": response.metadata},
            confidence=response.confidence,
            prediction_time_ms=response.prediction_time_ms
        )
        
        self.db_session.add(log_entry)
        self.db_session.commit()
    
    async def batch_predict(self, request: BatchPredictionRequest) -> List[PredictionResponse]:
        """Make batch predictions"""
        predictions = []
        
        for features in request.features_batch:
            pred_request = PredictionRequest(
                model_type=request.model_type,
                features=features,
                request_id=request.request_id
            )
            prediction = await self.predict(pred_request)
            predictions.append(prediction)
        
        return predictions
    
    async def get_model_info(self, model_type: ModelType) -> Dict[str, Any]:
        """Get information about available models"""
        if model_type.value not in self.models:
            return {"error": f"No models available for type: {model_type}"}
        
        models_info = []
        for version, model in self.models[model_type.value].items():
            model_record = self.db_session.query(ModelRegistry).filter(
                ModelRegistry.model_type == model_type.value,
                ModelRegistry.version == version
            ).first()
            
            models_info.append({
                "version": version,
                "status": model_record.status if model_record else "unknown",
                "metrics": model_record.metrics if model_record else {},
                "created_at": model_record.created_at.isoformat() if model_record else None
            })
        
        return {
            "model_type": model_type.value,
            "available_versions": models_info,
            "active_version_count": len(models_info)
        }

# FastAPI Application
app = FastAPI(title="TracSeq ML Model Server", version="1.0.0")

# Global model server instance
model_server = None

@app.on_event("startup")
async def startup_event():
    """Initialize model server on startup"""
    global model_server
    
    config = {
        "redis_host": "localhost",
        "redis_port": 6379,
        "database_url": "postgresql://ml_user:ml_pass@localhost:5436/ml_platform",
        "model_storage_path": "/models"
    }
    
    model_server = ModelServer(config)
    await model_server.initialize()

@app.post("/predict", response_model=PredictionResponse)
async def predict(request: PredictionRequest):
    """Make a single prediction"""
    return await model_server.predict(request)

@app.post("/batch_predict", response_model=List[PredictionResponse])
async def batch_predict(request: BatchPredictionRequest):
    """Make batch predictions"""
    return await model_server.batch_predict(request)

@app.get("/models/{model_type}")
async def get_model_info(model_type: ModelType):
    """Get information about available models"""
    return await model_server.get_model_info(model_type)

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {"status": "healthy", "timestamp": datetime.utcnow().isoformat()}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8094)