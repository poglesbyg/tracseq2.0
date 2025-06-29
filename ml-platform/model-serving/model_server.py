#!/usr/bin/env python3
"""
TracSeq 2.0 - Model Serving Service
Real-time model inference API for laboratory predictions
"""

import os
import json
import logging
from typing import Dict, Any, List, Optional
from datetime import datetime

import numpy as np
import pandas as pd
from fastapi import FastAPI, HTTPException, BackgroundTasks
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field
import redis
import joblib
from prometheus_client import Counter, Histogram, generate_latest
from fastapi.responses import Response

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Initialize FastAPI app
app = FastAPI(
    title="TracSeq Model Serving API",
    description="Real-time model inference for laboratory predictions",
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

# Initialize Redis connection
try:
    redis_host = os.getenv('REDIS_HOST', 'localhost')
    redis_port = int(os.getenv('REDIS_PORT', '6379'))
    redis_client = redis.Redis(host=redis_host, port=redis_port, decode_responses=True)
    redis_client.ping()
    logger.info(f"Connected to Redis at {redis_host}:{redis_port}")
except Exception as e:
    logger.error(f"Failed to connect to Redis: {e}")
    redis_client = None

# Prometheus metrics
REQUEST_COUNT = Counter('model_predictions_total', 'Total model predictions', ['model_name', 'status'])
REQUEST_DURATION = Histogram('model_prediction_duration_seconds', 'Model prediction duration')

# Pydantic models
class PredictionRequest(BaseModel):
    model_name: str = Field(..., description="Name of the model to use")
    features: Dict[str, Any] = Field(..., description="Input features for prediction")
    metadata: Optional[Dict[str, Any]] = Field(default={}, description="Additional metadata")

class PredictionResponse(BaseModel):
    prediction: Any = Field(..., description="Model prediction result")
    confidence: Optional[float] = Field(None, description="Prediction confidence score")
    model_version: str = Field(..., description="Version of the model used")
    timestamp: datetime = Field(default_factory=datetime.utcnow)
    processing_time_ms: float = Field(..., description="Processing time in milliseconds")

class ModelInfo(BaseModel):
    name: str
    version: str
    description: str
    input_features: List[str]
    output_type: str
    last_updated: datetime

# In-memory model registry (in production, this would be in a database)
models = {
    "sample_quality_classifier": {
        "version": "1.0.0",
        "description": "Predicts sample quality based on storage conditions",
        "input_features": ["temperature", "humidity", "storage_duration", "sample_type"],
        "output_type": "classification",
        "model": None,  # Would load actual model here
        "last_updated": datetime.utcnow()
    },
    "temperature_predictor": {
        "version": "1.0.0", 
        "description": "Predicts optimal storage temperature for samples",
        "input_features": ["sample_type", "volume", "concentration"],
        "output_type": "regression",
        "model": None,
        "last_updated": datetime.utcnow()
    },
    "sequencing_time_estimator": {
        "version": "1.0.0",
        "description": "Estimates sequencing completion time",
        "input_features": ["sample_count", "read_length", "coverage_depth"],
        "output_type": "regression", 
        "model": None,
        "last_updated": datetime.utcnow()
    }
}

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "service": "model-serving",
        "version": "1.0.0",
        "timestamp": datetime.utcnow(),
        "redis_connected": redis_client is not None and redis_client.ping()
    }

@app.get("/models", response_model=List[ModelInfo])
async def list_models():
    """List all available models"""
    return [
        ModelInfo(
            name=name,
            version=info["version"],
            description=info["description"],
            input_features=info["input_features"],
            output_type=info["output_type"],
            last_updated=info["last_updated"]
        )
        for name, info in models.items()
    ]

@app.get("/models/{model_name}", response_model=ModelInfo)
async def get_model_info(model_name: str):
    """Get information about a specific model"""
    if model_name not in models:
        raise HTTPException(status_code=404, detail=f"Model {model_name} not found")
    
    info = models[model_name]
    return ModelInfo(
        name=model_name,
        version=info["version"],
        description=info["description"],
        input_features=info["input_features"],
        output_type=info["output_type"],
        last_updated=info["last_updated"]
    )

def mock_predict(model_name: str, features: Dict[str, Any]) -> Dict[str, Any]:
    """Mock prediction function (replace with actual model inference)"""
    import random
    
    if model_name == "sample_quality_classifier":
        # Mock quality classification
        quality_scores = ["excellent", "good", "fair", "poor"]
        prediction = random.choice(quality_scores)
        confidence = random.uniform(0.7, 0.95)
        
    elif model_name == "temperature_predictor":
        # Mock temperature prediction
        prediction = random.uniform(-80, 4)  # Temperature range for lab storage
        confidence = random.uniform(0.8, 0.98)
        
    elif model_name == "sequencing_time_estimator":
        # Mock time estimation (hours)
        prediction = random.uniform(2, 48)
        confidence = random.uniform(0.75, 0.92)
        
    else:
        prediction = "unknown"
        confidence = 0.0
    
    return {"prediction": prediction, "confidence": confidence}

@app.post("/predict", response_model=PredictionResponse)
async def predict(request: PredictionRequest):
    """Make a prediction using the specified model"""
    start_time = datetime.utcnow()
    
    try:
        # Validate model exists
        if request.model_name not in models:
            REQUEST_COUNT.labels(model_name=request.model_name, status="error").inc()
            raise HTTPException(status_code=404, detail=f"Model {request.model_name} not found")
        
        model_info = models[request.model_name]
        
        # Validate input features
        required_features = set(model_info["input_features"])
        provided_features = set(request.features.keys())
        missing_features = required_features - provided_features
        
        if missing_features:
            REQUEST_COUNT.labels(model_name=request.model_name, status="error").inc()
            raise HTTPException(
                status_code=400, 
                detail=f"Missing required features: {list(missing_features)}"
            )
        
        # Check cache first (if Redis is available)
        cache_key = f"prediction:{request.model_name}:{hash(json.dumps(request.features, sort_keys=True))}"
        cached_result = None
        
        if redis_client:
            try:
                cached_result = redis_client.get(cache_key)
                if cached_result:
                    logger.info(f"Cache hit for {cache_key}")
                    cached_data = json.loads(cached_result)
                    REQUEST_COUNT.labels(model_name=request.model_name, status="cache_hit").inc()
                    return PredictionResponse(**cached_data)
            except Exception as e:
                logger.warning(f"Cache read error: {e}")
        
        # Make prediction
        result = mock_predict(request.model_name, request.features)
        
        # Calculate processing time
        processing_time = (datetime.utcnow() - start_time).total_seconds() * 1000
        
        response = PredictionResponse(
            prediction=result["prediction"],
            confidence=result["confidence"],
            model_version=model_info["version"],
            processing_time_ms=processing_time
        )
        
        # Cache result (if Redis is available)
        if redis_client:
            try:
                cache_data = response.dict()
                redis_client.setex(cache_key, 300, json.dumps(cache_data, default=str))  # 5 min TTL
            except Exception as e:
                logger.warning(f"Cache write error: {e}")
        
        REQUEST_COUNT.labels(model_name=request.model_name, status="success").inc()
        return response
        
    except HTTPException:
        raise
    except Exception as e:
        REQUEST_COUNT.labels(model_name=request.model_name, status="error").inc()
        logger.error(f"Prediction error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/models/{model_name}/reload")
async def reload_model(model_name: str, background_tasks: BackgroundTasks):
    """Reload a model (useful for model updates)"""
    if model_name not in models:
        raise HTTPException(status_code=404, detail=f"Model {model_name} not found")
    
    def reload_task():
        # In production, this would reload the model from storage
        models[model_name]["last_updated"] = datetime.utcnow()
        logger.info(f"Model {model_name} reloaded")
    
    background_tasks.add_task(reload_task)
    return {"message": f"Model {model_name} reload initiated", "status": "pending"}

@app.get("/metrics")
async def get_metrics():
    """Prometheus metrics endpoint"""
    return Response(generate_latest(), media_type="text/plain")

@app.post("/batch-predict")
async def batch_predict(requests: List[PredictionRequest]):
    """Batch prediction endpoint for multiple requests"""
    results = []
    
    for req in requests:
        try:
            result = await predict(req)
            results.append({"status": "success", "result": result})
        except Exception as e:
            results.append({"status": "error", "error": str(e)})
    
    return {"batch_results": results, "total_requests": len(requests)}

if __name__ == "__main__":
    import uvicorn
    
    port = int(os.getenv('PORT', '8094'))
    host = os.getenv('HOST', '0.0.0.0')
    
    logger.info(f"Starting TracSeq Model Serving API on {host}:{port}")
    uvicorn.run(app, host=host, port=port)