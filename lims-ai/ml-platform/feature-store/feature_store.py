#!/usr/bin/env python3
"""
TracSeq 2.0 - Feature Store Service
Centralized feature management for ML pipelines
"""

import os
import json
import logging
from typing import Dict, Any, List, Optional
from datetime import datetime, timedelta
from enum import Enum

import numpy as np
import pandas as pd
from fastapi import FastAPI, HTTPException, Query
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field
import redis
from sqlalchemy import create_engine, text

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Initialize FastAPI app
app = FastAPI(
    title="TracSeq Feature Store API",
    description="Centralized feature management for laboratory ML pipelines",
    version="1.0.0"
)

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Initialize connections
try:
    # Extract Redis host from REDIS_URL or use REDIS_HOST
    redis_url = os.getenv('REDIS_URL', '')
    if redis_url and redis_url.startswith('redis://'):
        # Parse redis://host:port format
        redis_parts = redis_url.replace('redis://', '').split(':')
        redis_host = redis_parts[0]
        redis_port = int(redis_parts[1]) if len(redis_parts) > 1 else 6379
    else:
        redis_host = os.getenv('REDIS_HOST', 'localhost')
        redis_port = int(os.getenv('REDIS_PORT', '6379'))
    redis_client = redis.Redis(host=redis_host, port=redis_port, decode_responses=True)
    redis_client.ping()
    logger.info(f"Connected to Redis at {redis_host}:{redis_port}")
except Exception as e:
    logger.error(f"Failed to connect to Redis: {e}")
    redis_client = None

# Database connection
try:
    database_url = os.getenv('DATABASE_URL', 'postgresql://ml_user:ml_pass@localhost:5432/ml_platform')
    engine = create_engine(database_url)
    logger.info("Connected to PostgreSQL database")
except Exception as e:
    logger.error(f"Failed to connect to database: {e}")
    engine = None

# Enums
class FeatureType(str, Enum):
    numeric = "numeric"
    categorical = "categorical"
    boolean = "boolean"
    text = "text"
    timestamp = "timestamp"

class FeatureSource(str, Enum):
    batch = "batch"
    streaming = "streaming"
    computed = "computed"
    external = "external"

# Pydantic models
class FeatureDefinition(BaseModel):
    name: str = Field(..., description="Feature name")
    entity_type: str = Field(..., description="Entity type (e.g., sample, storage_unit)")
    type: FeatureType = Field(..., description="Feature data type")
    source: FeatureSource = Field(..., description="Feature source type")
    description: Optional[str] = Field(None, description="Feature description")
    tags: Optional[Dict[str, str]] = Field(default={}, description="Feature tags")
    created_at: Optional[datetime] = Field(default_factory=datetime.utcnow)

class FeatureValue(BaseModel):
    entity_id: str = Field(..., description="Entity identifier")
    feature_name: str = Field(..., description="Feature name")
    value: Any = Field(..., description="Feature value")
    timestamp: Optional[datetime] = Field(default_factory=datetime.utcnow)

class FeatureQuery(BaseModel):
    entity_ids: List[str] = Field(..., description="List of entity identifiers")
    feature_names: List[str] = Field(..., description="List of feature names to retrieve")
    as_of_time: Optional[datetime] = Field(None, description="Point-in-time for feature values")

class FeatureVector(BaseModel):
    entity_id: str
    features: Dict[str, Any]
    timestamp: datetime

# In-memory feature registry (in production, this would be in the database)
feature_registry = {
    "sample_age_hours": {
        "entity_type": "sample",
        "type": "numeric",
        "source": "computed",
        "description": "Age of sample in hours since collection",
        "tags": {"category": "temporal", "unit": "hours"},
        "created_at": datetime.utcnow()
    },
    "storage_temperature": {
        "entity_type": "sample",
        "type": "numeric", 
        "source": "streaming",
        "description": "Current storage temperature in Celsius",
        "tags": {"category": "environmental", "unit": "celsius"},
        "created_at": datetime.utcnow()
    },
    "sample_volume_ml": {
        "entity_type": "sample",
        "type": "numeric",
        "source": "batch",
        "description": "Sample volume in milliliters",
        "tags": {"category": "physical", "unit": "ml"},
        "created_at": datetime.utcnow()
    },
    "sample_type": {
        "entity_type": "sample",
        "type": "categorical",
        "source": "batch",
        "description": "Type of biological sample",
        "tags": {"category": "metadata"},
        "created_at": datetime.utcnow()
    },
    "quality_risk_score": {
        "entity_type": "sample",
        "type": "numeric",
        "source": "computed",
        "description": "Computed quality risk score (0-1)",
        "tags": {"category": "quality", "range": "0-1"},
        "created_at": datetime.utcnow()
    },
    "storage_capacity_used": {
        "entity_type": "storage_unit",
        "type": "numeric",
        "source": "streaming",
        "description": "Percentage of storage capacity used",
        "tags": {"category": "capacity", "unit": "percentage"},
        "created_at": datetime.utcnow()
    },
    "days_since_last_maintenance": {
        "entity_type": "storage_unit",
        "type": "numeric",
        "source": "computed",
        "description": "Days since last maintenance",
        "tags": {"category": "maintenance", "unit": "days"},
        "created_at": datetime.utcnow()
    }
}

def generate_mock_feature_value(feature_name: str, entity_id: str) -> Any:
    """Generate mock feature values for testing"""
    import random
    
    if feature_name == "sample_age_hours":
        return random.uniform(0, 720)  # 0-30 days
    elif feature_name == "storage_temperature":
        return random.choice([-80, -20, 4, 23, 37])  # Common lab temperatures
    elif feature_name == "sample_volume_ml":
        return random.uniform(0.1, 10.0)
    elif feature_name == "sample_type":
        return random.choice(["blood", "saliva", "tissue", "urine", "serum"])
    elif feature_name == "quality_risk_score":
        return random.uniform(0, 1)
    elif feature_name == "storage_capacity_used":
        return random.uniform(0, 100)
    elif feature_name == "days_since_last_maintenance":
        return random.randint(0, 365)
    else:
        return random.uniform(0, 100)

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "service": "feature-store",
        "version": "1.0.0",
        "timestamp": datetime.utcnow(),
        "redis_connected": redis_client is not None and redis_client.ping(),
        "database_connected": engine is not None
    }

@app.get("/features", response_model=List[FeatureDefinition])
async def list_features(entity_type: Optional[str] = Query(None, description="Filter by entity type")):
    """List all registered features"""
    features = []
    
    for name, info in feature_registry.items():
        if entity_type is None or info["entity_type"] == entity_type:
            features.append(FeatureDefinition(
                name=name,
                entity_type=info["entity_type"],
                type=info["type"],
                source=info["source"],
                description=info["description"],
                tags=info["tags"],
                created_at=info["created_at"]
            ))
    
    return features

@app.post("/register-feature", response_model=Dict[str, str])
async def register_feature(feature: FeatureDefinition):
    """Register a new feature in the feature store"""
    if feature.name in feature_registry:
        raise HTTPException(status_code=409, detail=f"Feature {feature.name} already exists")
    
    feature_registry[feature.name] = {
        "entity_type": feature.entity_type,
        "type": feature.type,
        "source": feature.source,
        "description": feature.description,
        "tags": feature.tags,
        "created_at": datetime.utcnow()
    }
    
    logger.info(f"Registered new feature: {feature.name}")
    return {"message": f"Feature {feature.name} registered successfully", "status": "success"}

@app.get("/features/{feature_name}", response_model=FeatureDefinition)
async def get_feature_definition(feature_name: str):
    """Get feature definition by name"""
    if feature_name not in feature_registry:
        raise HTTPException(status_code=404, detail=f"Feature {feature_name} not found")
    
    info = feature_registry[feature_name]
    return FeatureDefinition(
        name=feature_name,
        entity_type=info["entity_type"],
        type=info["type"],
        source=info["source"],
        description=info["description"],
        tags=info["tags"],
        created_at=info["created_at"]
    )

@app.post("/features/values", response_model=Dict[str, str])
async def ingest_feature_values(values: List[FeatureValue]):
    """Ingest feature values into the feature store"""
    try:
        for value in values:
            # Validate feature exists
            if value.feature_name not in feature_registry:
                raise HTTPException(
                    status_code=400, 
                    detail=f"Feature {value.feature_name} not registered"
                )
            
            # Cache in Redis (if available)
            if redis_client:
                cache_key = f"feature:{value.feature_name}:{value.entity_id}"
                cache_value = {
                    "value": value.value,
                    "timestamp": value.timestamp.isoformat()
                }
                redis_client.setex(cache_key, 3600, json.dumps(cache_value, default=str))  # 1 hour TTL
        
        logger.info(f"Ingested {len(values)} feature values")
        return {"message": f"Successfully ingested {len(values)} feature values", "status": "success"}
        
    except Exception as e:
        logger.error(f"Feature ingestion error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/features/query", response_model=List[FeatureVector])
async def query_features(query: FeatureQuery):
    """Query feature values for given entities and features"""
    try:
        results = []
        
        for entity_id in query.entity_ids:
            features = {}
            
            for feature_name in query.feature_names:
                # Validate feature exists
                if feature_name not in feature_registry:
                    raise HTTPException(
                        status_code=400,
                        detail=f"Feature {feature_name} not registered"
                    )
                
                # Try to get from cache first
                feature_value = None
                if redis_client:
                    cache_key = f"feature:{feature_name}:{entity_id}"
                    cached_value = redis_client.get(cache_key)
                    if cached_value:
                        cached_data = json.loads(cached_value)
                        feature_value = cached_data["value"]
                
                # If not in cache, generate mock value (in production, query from database)
                if feature_value is None:
                    feature_value = generate_mock_feature_value(feature_name, entity_id)
                
                features[feature_name] = feature_value
            
            results.append(FeatureVector(
                entity_id=entity_id,
                features=features,
                timestamp=query.as_of_time or datetime.utcnow()
            ))
        
        return results
        
    except Exception as e:
        logger.error(f"Feature query error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/features/{feature_name}/values/{entity_id}")
async def get_feature_value(feature_name: str, entity_id: str):
    """Get a specific feature value for an entity"""
    if feature_name not in feature_registry:
        raise HTTPException(status_code=404, detail=f"Feature {feature_name} not found")
    
    # Try cache first
    if redis_client:
        cache_key = f"feature:{feature_name}:{entity_id}"
        cached_value = redis_client.get(cache_key)
        if cached_value:
            cached_data = json.loads(cached_value)
            return {
                "entity_id": entity_id,
                "feature_name": feature_name,
                "value": cached_data["value"],
                "timestamp": cached_data["timestamp"],
                "source": "cache"
            }
    
    # Generate mock value (in production, query from database)
    value = generate_mock_feature_value(feature_name, entity_id)
    timestamp = datetime.utcnow()
    
    return {
        "entity_id": entity_id,
        "feature_name": feature_name,
        "value": value,
        "timestamp": timestamp,
        "source": "computed"
    }

@app.get("/entity-types")
async def list_entity_types():
    """List all entity types in the feature store"""
    entity_types = set()
    for feature_info in feature_registry.values():
        entity_types.add(feature_info["entity_type"])
    
    return {"entity_types": list(entity_types)}

@app.get("/features/stats")
async def get_feature_stats():
    """Get statistics about the feature store"""
    stats = {
        "total_features": len(feature_registry),
        "features_by_type": {},
        "features_by_source": {},
        "features_by_entity_type": {}
    }
    
    for feature_info in feature_registry.values():
        # Count by type
        feature_type = feature_info["type"]
        stats["features_by_type"][feature_type] = stats["features_by_type"].get(feature_type, 0) + 1
        
        # Count by source
        source = feature_info["source"]
        stats["features_by_source"][source] = stats["features_by_source"].get(source, 0) + 1
        
        # Count by entity type
        entity_type = feature_info["entity_type"]
        stats["features_by_entity_type"][entity_type] = stats["features_by_entity_type"].get(entity_type, 0) + 1
    
    return stats

if __name__ == "__main__":
    import uvicorn
    
    port = int(os.getenv('PORT', '8095'))
    host = os.getenv('HOST', '0.0.0.0')
    
    logger.info(f"Starting TracSeq Feature Store API on {host}:{port}")
    uvicorn.run(app, host=host, port=port)