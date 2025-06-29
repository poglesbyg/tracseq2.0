# TracSeq 2.0 - Feature Store System
# Centralized feature management for ML pipelines

import asyncio
import json
import logging
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from enum import Enum
from typing import Any, Dict, List, Optional, Tuple, Union
from uuid import UUID, uuid4

import numpy as np
import pandas as pd
from fastapi import FastAPI, HTTPException, Query
from pydantic import BaseModel, Field, validator
import redis
from sqlalchemy import create_engine, Column, String, Float, Integer, DateTime, JSON, Boolean, Index
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker
from sqlalchemy.sql import text
import pyarrow as pa
import pyarrow.parquet as pq
from scipy import stats
import hashlib

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Database Models
Base = declarative_base()

class FeatureType(str, Enum):
    NUMERIC = "numeric"
    CATEGORICAL = "categorical"
    BINARY = "binary"
    TIMESTAMP = "timestamp"
    EMBEDDING = "embedding"
    JSON = "json"

class FeatureSource(str, Enum):
    BATCH = "batch"
    STREAMING = "streaming"
    REQUEST = "request"
    COMPUTED = "computed"

# Feature Definition Models
class FeatureDefinition(Base):
    __tablename__ = "feature_definitions"
    __table_args__ = (
        Index('idx_feature_entity', 'feature_name', 'entity_type'),
    )
    
    id = Column(String, primary_key=True)
    feature_name = Column(String, nullable=False)
    entity_type = Column(String, nullable=False)  # sample, patient, sequencing_run, etc.
    feature_type = Column(String, nullable=False)
    description = Column(String)
    source = Column(String, nullable=False)
    computation_logic = Column(JSON)  # For computed features
    dependencies = Column(JSON)  # Other features this depends on
    version = Column(Integer, default=1)
    is_active = Column(Boolean, default=True)
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)
    metadata = Column(JSON)

class FeatureValue(Base):
    __tablename__ = "feature_values"
    __table_args__ = (
        Index('idx_entity_feature_timestamp', 'entity_id', 'feature_id', 'timestamp'),
    )
    
    id = Column(String, primary_key=True)
    feature_id = Column(String, nullable=False)
    entity_id = Column(String, nullable=False)
    value = Column(JSON, nullable=False)
    timestamp = Column(DateTime, nullable=False)
    created_at = Column(DateTime, default=datetime.utcnow)

class FeatureSet(Base):
    __tablename__ = "feature_sets"
    
    id = Column(String, primary_key=True)
    name = Column(String, nullable=False, unique=True)
    description = Column(String)
    entity_type = Column(String, nullable=False)
    features = Column(JSON, nullable=False)  # List of feature names
    version = Column(Integer, default=1)
    is_active = Column(Boolean, default=True)
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)

# Request/Response Models
class FeatureRequest(BaseModel):
    entity_type: str
    entity_id: str
    features: List[str]
    point_in_time: Optional[datetime] = None

class FeatureResponse(BaseModel):
    entity_id: str
    features: Dict[str, Any]
    timestamp: datetime
    metadata: Dict[str, Any] = {}

class FeatureSetRequest(BaseModel):
    entity_type: str
    entity_id: str
    feature_set_name: str
    point_in_time: Optional[datetime] = None

class BatchFeatureRequest(BaseModel):
    entity_type: str
    entity_ids: List[str]
    features: List[str]
    point_in_time: Optional[datetime] = None

# Feature Engineering
class FeatureEngineer:
    """Handles feature computation and transformation"""
    
    def __init__(self):
        self.transformers = {}
        self._register_default_transformers()
    
    def _register_default_transformers(self):
        """Register default feature transformers"""
        # Laboratory-specific transformers
        self.transformers["sample_age_hours"] = self._compute_sample_age
        self.transformers["temperature_deviation"] = self._compute_temperature_deviation
        self.transformers["quality_risk_score"] = self._compute_quality_risk_score
        self.transformers["storage_utilization"] = self._compute_storage_utilization
        self.transformers["processing_delay_category"] = self._compute_processing_delay_category
    
    async def compute_feature(self, feature_def: FeatureDefinition, entity_id: str, context: Dict[str, Any]) -> Any:
        """Compute a feature value based on its definition"""
        if feature_def.source == FeatureSource.COMPUTED:
            transformer = self.transformers.get(feature_def.feature_name)
            if transformer:
                return await transformer(entity_id, context, feature_def.computation_logic)
            else:
                raise ValueError(f"No transformer found for feature: {feature_def.feature_name}")
        return None
    
    async def _compute_sample_age(self, entity_id: str, context: Dict[str, Any], logic: Dict[str, Any]) -> float:
        """Compute sample age in hours"""
        collection_time = context.get("collection_time")
        if collection_time:
            age = (datetime.utcnow() - collection_time).total_seconds() / 3600
            return round(age, 2)
        return 0.0
    
    async def _compute_temperature_deviation(self, entity_id: str, context: Dict[str, Any], logic: Dict[str, Any]) -> float:
        """Compute deviation from target temperature"""
        current_temp = context.get("current_temperature", -80)
        target_temp = context.get("target_temperature", -80)
        return abs(current_temp - target_temp)
    
    async def _compute_quality_risk_score(self, entity_id: str, context: Dict[str, Any], logic: Dict[str, Any]) -> float:
        """Compute quality risk score based on multiple factors"""
        # Factors affecting quality
        age_hours = context.get("sample_age_hours", 0)
        temp_deviation = context.get("temperature_deviation", 0)
        volume_ml = context.get("volume_ml", 5)
        processing_delay = context.get("processing_delay_hours", 0)
        
        # Risk calculation
        age_risk = min(age_hours / 168, 1.0)  # Max risk at 1 week
        temp_risk = min(temp_deviation / 10, 1.0)  # Max risk at 10°C deviation
        volume_risk = max(0, 1 - volume_ml / 5)  # Risk if less than 5ml
        delay_risk = min(processing_delay / 48, 1.0)  # Max risk at 48 hours
        
        # Weighted average
        weights = logic.get("weights", {
            "age": 0.3,
            "temperature": 0.3,
            "volume": 0.2,
            "delay": 0.2
        })
        
        risk_score = (
            weights["age"] * age_risk +
            weights["temperature"] * temp_risk +
            weights["volume"] * volume_risk +
            weights["delay"] * delay_risk
        )
        
        return round(risk_score, 3)
    
    async def _compute_storage_utilization(self, entity_id: str, context: Dict[str, Any], logic: Dict[str, Any]) -> float:
        """Compute storage location utilization"""
        used_capacity = context.get("used_capacity", 0)
        total_capacity = context.get("total_capacity", 100)
        if total_capacity > 0:
            return round(used_capacity / total_capacity, 3)
        return 0.0
    
    async def _compute_processing_delay_category(self, entity_id: str, context: Dict[str, Any], logic: Dict[str, Any]) -> str:
        """Categorize processing delay"""
        delay_hours = context.get("processing_delay_hours", 0)
        
        if delay_hours < 6:
            return "immediate"
        elif delay_hours < 24:
            return "same_day"
        elif delay_hours < 48:
            return "next_day"
        else:
            return "delayed"

# Feature Store Implementation
class FeatureStore:
    """Centralized feature store for ML pipelines"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.redis_client = None
        self.db_session = None
        self.feature_engineer = FeatureEngineer()
        self.cache_ttl = config.get("cache_ttl", 3600)  # 1 hour default
    
    async def initialize(self):
        """Initialize feature store connections"""
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
        
        logger.info("Feature store initialized")
    
    async def register_feature(self, feature_def: Dict[str, Any]) -> str:
        """Register a new feature definition"""
        feature = FeatureDefinition(
            id=str(uuid4()),
            feature_name=feature_def["name"],
            entity_type=feature_def["entity_type"],
            feature_type=feature_def["type"],
            description=feature_def.get("description"),
            source=feature_def["source"],
            computation_logic=feature_def.get("computation_logic"),
            dependencies=feature_def.get("dependencies", []),
            metadata=feature_def.get("metadata", {})
        )
        
        self.db_session.add(feature)
        self.db_session.commit()
        
        logger.info(f"Registered feature: {feature.feature_name}")
        return feature.id
    
    async def get_features(self, request: FeatureRequest) -> FeatureResponse:
        """Get feature values for an entity"""
        feature_values = {}
        
        for feature_name in request.features:
            # Check cache first
            cache_key = self._get_cache_key(request.entity_type, request.entity_id, feature_name)
            cached_value = self.redis_client.get(cache_key)
            
            if cached_value:
                feature_values[feature_name] = json.loads(cached_value)
            else:
                # Get feature definition
                feature_def = self.db_session.query(FeatureDefinition).filter(
                    FeatureDefinition.feature_name == feature_name,
                    FeatureDefinition.entity_type == request.entity_type,
                    FeatureDefinition.is_active == True
                ).first()
                
                if not feature_def:
                    logger.warning(f"Feature not found: {feature_name}")
                    continue
                
                # Get or compute feature value
                value = await self._get_or_compute_feature(
                    feature_def,
                    request.entity_id,
                    request.point_in_time
                )
                
                if value is not None:
                    feature_values[feature_name] = value
                    # Cache the value
                    self.redis_client.setex(
                        cache_key,
                        self.cache_ttl,
                        json.dumps(value)
                    )
        
        return FeatureResponse(
            entity_id=request.entity_id,
            features=feature_values,
            timestamp=request.point_in_time or datetime.utcnow(),
            metadata={
                "entity_type": request.entity_type,
                "feature_count": len(feature_values)
            }
        )
    
    async def _get_or_compute_feature(self, feature_def: FeatureDefinition, entity_id: str, point_in_time: Optional[datetime]) -> Any:
        """Get feature value from storage or compute if needed"""
        if feature_def.source == FeatureSource.COMPUTED:
            # Get context for computation
            context = await self._get_computation_context(feature_def, entity_id, point_in_time)
            value = await self.feature_engineer.compute_feature(feature_def, entity_id, context)
            
            # Store computed value
            await self._store_feature_value(feature_def.id, entity_id, value, point_in_time or datetime.utcnow())
            return value
        else:
            # Get from storage
            return await self._get_stored_feature(feature_def.id, entity_id, point_in_time)
    
    async def _get_computation_context(self, feature_def: FeatureDefinition, entity_id: str, point_in_time: Optional[datetime]) -> Dict[str, Any]:
        """Get context needed for feature computation"""
        context = {}
        
        # Get dependent features
        if feature_def.dependencies:
            dep_request = FeatureRequest(
                entity_type=feature_def.entity_type,
                entity_id=entity_id,
                features=feature_def.dependencies,
                point_in_time=point_in_time
            )
            dep_response = await self.get_features(dep_request)
            context.update(dep_response.features)
        
        # Add entity metadata
        # This would fetch from actual entity storage
        context["entity_id"] = entity_id
        context["entity_type"] = feature_def.entity_type
        
        return context
    
    async def _get_stored_feature(self, feature_id: str, entity_id: str, point_in_time: Optional[datetime]) -> Any:
        """Get feature value from storage"""
        query = self.db_session.query(FeatureValue).filter(
            FeatureValue.feature_id == feature_id,
            FeatureValue.entity_id == entity_id
        )
        
        if point_in_time:
            query = query.filter(FeatureValue.timestamp <= point_in_time)
        
        feature_value = query.order_by(FeatureValue.timestamp.desc()).first()
        
        if feature_value:
            return feature_value.value
        return None
    
    async def _store_feature_value(self, feature_id: str, entity_id: str, value: Any, timestamp: datetime):
        """Store a feature value"""
        feature_value = FeatureValue(
            id=str(uuid4()),
            feature_id=feature_id,
            entity_id=entity_id,
            value=value if isinstance(value, dict) else {"value": value},
            timestamp=timestamp
        )
        
        self.db_session.add(feature_value)
        self.db_session.commit()
    
    def _get_cache_key(self, entity_type: str, entity_id: str, feature_name: str) -> str:
        """Generate cache key for feature"""
        return f"feature:{entity_type}:{entity_id}:{feature_name}"
    
    async def get_feature_set(self, request: FeatureSetRequest) -> FeatureResponse:
        """Get all features in a feature set"""
        feature_set = self.db_session.query(FeatureSet).filter(
            FeatureSet.name == request.feature_set_name,
            FeatureSet.entity_type == request.entity_type,
            FeatureSet.is_active == True
        ).first()
        
        if not feature_set:
            raise ValueError(f"Feature set not found: {request.feature_set_name}")
        
        # Get all features in the set
        feature_request = FeatureRequest(
            entity_type=request.entity_type,
            entity_id=request.entity_id,
            features=feature_set.features,
            point_in_time=request.point_in_time
        )
        
        return await self.get_features(feature_request)
    
    async def batch_get_features(self, request: BatchFeatureRequest) -> List[FeatureResponse]:
        """Get features for multiple entities"""
        responses = []
        
        # Use asyncio for parallel processing
        tasks = []
        for entity_id in request.entity_ids:
            feature_request = FeatureRequest(
                entity_type=request.entity_type,
                entity_id=entity_id,
                features=request.features,
                point_in_time=request.point_in_time
            )
            tasks.append(self.get_features(feature_request))
        
        responses = await asyncio.gather(*tasks)
        return responses
    
    async def compute_feature_statistics(self, feature_name: str, entity_type: str, time_window: timedelta) -> Dict[str, Any]:
        """Compute statistics for a feature"""
        cutoff_time = datetime.utcnow() - time_window
        
        # Get recent feature values
        feature_def = self.db_session.query(FeatureDefinition).filter(
            FeatureDefinition.feature_name == feature_name,
            FeatureDefinition.entity_type == entity_type
        ).first()
        
        if not feature_def:
            raise ValueError(f"Feature not found: {feature_name}")
        
        values = self.db_session.query(FeatureValue.value).filter(
            FeatureValue.feature_id == feature_def.id,
            FeatureValue.timestamp >= cutoff_time
        ).all()
        
        if not values:
            return {"error": "No data available for the specified time window"}
        
        # Extract numeric values
        numeric_values = []
        for v in values:
            if isinstance(v[0], dict) and "value" in v[0]:
                val = v[0]["value"]
            else:
                val = v[0]
            
            if isinstance(val, (int, float)):
                numeric_values.append(val)
        
        if not numeric_values:
            return {"error": "No numeric values found"}
        
        # Compute statistics
        return {
            "feature_name": feature_name,
            "entity_type": entity_type,
            "count": len(numeric_values),
            "mean": np.mean(numeric_values),
            "std": np.std(numeric_values),
            "min": np.min(numeric_values),
            "max": np.max(numeric_values),
            "percentiles": {
                "25": np.percentile(numeric_values, 25),
                "50": np.percentile(numeric_values, 50),
                "75": np.percentile(numeric_values, 75)
            }
        }

# Laboratory-specific feature definitions
def create_laboratory_features():
    """Create feature definitions for laboratory domain"""
    return [
        {
            "name": "sample_age_hours",
            "entity_type": "sample",
            "type": FeatureType.NUMERIC,
            "source": FeatureSource.COMPUTED,
            "description": "Age of sample in hours since collection",
            "computation_logic": {},
            "dependencies": ["collection_time"]
        },
        {
            "name": "temperature_deviation",
            "entity_type": "sample",
            "type": FeatureType.NUMERIC,
            "source": FeatureSource.COMPUTED,
            "description": "Deviation from target storage temperature",
            "computation_logic": {},
            "dependencies": ["current_temperature", "target_temperature"]
        },
        {
            "name": "quality_risk_score",
            "entity_type": "sample",
            "type": FeatureType.NUMERIC,
            "source": FeatureSource.COMPUTED,
            "description": "Overall quality risk score (0-1)",
            "computation_logic": {
                "weights": {
                    "age": 0.3,
                    "temperature": 0.3,
                    "volume": 0.2,
                    "delay": 0.2
                }
            },
            "dependencies": ["sample_age_hours", "temperature_deviation", "volume_ml", "processing_delay_hours"]
        },
        {
            "name": "storage_utilization",
            "entity_type": "storage_location",
            "type": FeatureType.NUMERIC,
            "source": FeatureSource.COMPUTED,
            "description": "Storage location utilization percentage",
            "computation_logic": {},
            "dependencies": ["used_capacity", "total_capacity"]
        },
        {
            "name": "processing_delay_category",
            "entity_type": "sample",
            "type": FeatureType.CATEGORICAL,
            "source": FeatureSource.COMPUTED,
            "description": "Category of processing delay",
            "computation_logic": {},
            "dependencies": ["processing_delay_hours"]
        }
    ]

# FastAPI Application
app = FastAPI(title="TracSeq Feature Store", version="1.0.0")

# Global feature store instance
feature_store = None

@app.on_event("startup")
async def startup_event():
    """Initialize feature store on startup"""
    global feature_store
    
    config = {
        "redis_host": "localhost",
        "redis_port": 6379,
        "database_url": "postgresql://ml_user:ml_pass@localhost:5436/ml_platform",
        "cache_ttl": 3600
    }
    
    feature_store = FeatureStore(config)
    await feature_store.initialize()
    
    # Register laboratory features
    for feature_def in create_laboratory_features():
        await feature_store.register_feature(feature_def)

@app.post("/features", response_model=FeatureResponse)
async def get_features(request: FeatureRequest):
    """Get feature values for an entity"""
    return await feature_store.get_features(request)

@app.post("/feature-set", response_model=FeatureResponse)
async def get_feature_set(request: FeatureSetRequest):
    """Get all features in a feature set"""
    return await feature_store.get_feature_set(request)

@app.post("/batch-features", response_model=List[FeatureResponse])
async def batch_get_features(request: BatchFeatureRequest):
    """Get features for multiple entities"""
    return await feature_store.batch_get_features(request)

@app.get("/feature-stats/{feature_name}")
async def get_feature_statistics(
    feature_name: str,
    entity_type: str,
    days: int = Query(7, description="Time window in days")
):
    """Get statistics for a feature"""
    time_window = timedelta(days=days)
    return await feature_store.compute_feature_statistics(feature_name, entity_type, time_window)

@app.post("/register-feature")
async def register_feature(feature_def: Dict[str, Any]):
    """Register a new feature definition"""
    feature_id = await feature_store.register_feature(feature_def)
    return {"feature_id": feature_id, "status": "registered"}

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {"status": "healthy", "timestamp": datetime.utcnow().isoformat()}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8095)