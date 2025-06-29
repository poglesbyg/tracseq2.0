# TracSeq 2.0 - MLOps Pipeline
# Complete ML lifecycle management for laboratory predictions

import asyncio
import json
import logging
import os
import shutil
from datetime import datetime, timedelta
from enum import Enum
from typing import Any, Dict, List, Optional, Tuple, Union
from uuid import UUID, uuid4

import mlflow
import mlflow.sklearn
import mlflow.pyfunc
from mlflow.tracking import MlflowClient
from mlflow.models import infer_signature
import numpy as np
import pandas as pd
import joblib
from fastapi import FastAPI, HTTPException, UploadFile, File, BackgroundTasks
from pydantic import BaseModel, Field
import docker
from sqlalchemy import create_engine, Column, String, Float, Integer, DateTime, JSON, Boolean
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker
import boto3
from prometheus_client import Counter, Histogram, Gauge
import yaml

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Database Models
Base = declarative_base()

class ModelStage(str, Enum):
    DEVELOPMENT = "development"
    STAGING = "staging"
    PRODUCTION = "production"
    ARCHIVED = "archived"

class DeploymentTarget(str, Enum):
    KUBERNETES = "kubernetes"
    DOCKER = "docker"
    SAGEMAKER = "sagemaker"
    EDGE = "edge"

class DeploymentStatus(str, Enum):
    PENDING = "pending"
    IN_PROGRESS = "in_progress"
    DEPLOYED = "deployed"
    FAILED = "failed"
    ROLLBACK = "rollback"

# Database Models
class MLModel(Base):
    __tablename__ = "ml_models"
    
    id = Column(String, primary_key=True)
    name = Column(String, nullable=False)
    version = Column(String, nullable=False)
    model_type = Column(String, nullable=False)
    framework = Column(String)
    stage = Column(String, default=ModelStage.DEVELOPMENT)
    mlflow_run_id = Column(String)
    model_uri = Column(String)
    metrics = Column(JSON)
    parameters = Column(JSON)
    tags = Column(JSON)
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)
    created_by = Column(String)

class ModelDeployment(Base):
    __tablename__ = "model_deployments"
    
    id = Column(String, primary_key=True)
    model_id = Column(String, nullable=False)
    deployment_name = Column(String, nullable=False)
    target = Column(String, nullable=False)
    endpoint_url = Column(String)
    configuration = Column(JSON)
    status = Column(String, default=DeploymentStatus.PENDING)
    health_check_url = Column(String)
    created_at = Column(DateTime, default=datetime.utcnow)
    deployed_at = Column(DateTime)
    retired_at = Column(DateTime)

class ExperimentRun(Base):
    __tablename__ = "experiment_runs"
    
    id = Column(String, primary_key=True)
    experiment_name = Column(String, nullable=False)
    mlflow_experiment_id = Column(String)
    mlflow_run_id = Column(String)
    status = Column(String)
    parameters = Column(JSON)
    metrics = Column(JSON)
    artifacts = Column(JSON)
    tags = Column(JSON)
    started_at = Column(DateTime)
    completed_at = Column(DateTime)
    created_by = Column(String)

# Request/Response Models
class ExperimentRequest(BaseModel):
    name: str
    description: Optional[str] = None
    parameters: Dict[str, Any] = {}
    tags: Dict[str, str] = {}
    user_id: str

class ModelRegistrationRequest(BaseModel):
    run_id: str
    model_name: str
    model_type: str
    tags: Dict[str, str] = {}
    description: Optional[str] = None

class ModelDeploymentRequest(BaseModel):
    model_id: str
    deployment_name: str
    target: DeploymentTarget
    configuration: Dict[str, Any] = {}
    auto_scale: bool = True
    min_replicas: int = Field(default=1, ge=1, le=10)
    max_replicas: int = Field(default=3, ge=1, le=20)

class ModelPromotionRequest(BaseModel):
    model_id: str
    target_stage: ModelStage
    approval_notes: Optional[str] = None
    approved_by: str

class ModelMetricsResponse(BaseModel):
    model_id: str
    model_name: str
    version: str
    stage: str
    metrics: Dict[str, float]
    created_at: datetime
    comparison_metrics: Optional[Dict[str, float]] = None

# MLOps Pipeline Manager
class MLOpsPipeline:
    """Complete MLOps pipeline for model lifecycle management"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.db_session = None
        self.mlflow_client = None
        self.docker_client = None
        self.metrics = self._init_metrics()
        
    def _init_metrics(self):
        """Initialize Prometheus metrics"""
        return {
            "experiments_total": Counter(
                "mlops_experiments_total",
                "Total number of experiments",
                ["status"]
            ),
            "model_deployments": Counter(
                "mlops_model_deployments_total",
                "Total number of model deployments",
                ["target", "status"]
            ),
            "deployment_duration": Histogram(
                "mlops_deployment_duration_seconds",
                "Deployment duration",
                ["target"]
            ),
            "active_deployments": Gauge(
                "mlops_active_deployments",
                "Number of active deployments",
                ["stage"]
            )
        }
    
    async def initialize(self):
        """Initialize MLOps pipeline"""
        # Initialize MLflow
        mlflow.set_tracking_uri(self.config["mlflow_tracking_uri"])
        self.mlflow_client = MlflowClient()
        
        # Initialize database
        engine = create_engine(self.config["database_url"])
        Base.metadata.create_all(engine)
        Session = sessionmaker(bind=engine)
        self.db_session = Session()
        
        # Initialize Docker client
        self.docker_client = docker.from_env()
        
        logger.info("MLOps pipeline initialized")
    
    async def create_experiment(self, request: ExperimentRequest) -> str:
        """Create a new ML experiment"""
        try:
            # Create MLflow experiment
            experiment_id = mlflow.create_experiment(
                request.name,
                tags={**request.tags, "created_by": request.user_id}
            )
            
            # Create database record
            experiment_run = ExperimentRun(
                id=str(uuid4()),
                experiment_name=request.name,
                mlflow_experiment_id=experiment_id,
                status="created",
                parameters=request.parameters,
                tags=request.tags,
                created_by=request.user_id,
                started_at=datetime.utcnow()
            )
            
            self.db_session.add(experiment_run)
            self.db_session.commit()
            
            self.metrics["experiments_total"].labels(status="created").inc()
            
            return experiment_run.id
            
        except Exception as e:
            logger.error(f"Failed to create experiment: {e}")
            self.metrics["experiments_total"].labels(status="failed").inc()
            raise
    
    async def register_model(self, request: ModelRegistrationRequest) -> str:
        """Register a trained model"""
        try:
            # Get run info from MLflow
            run = self.mlflow_client.get_run(request.run_id)
            
            # Register model in MLflow
            model_version = mlflow.register_model(
                f"runs:/{request.run_id}/model",
                request.model_name
            )
            
            # Create database record
            model = MLModel(
                id=str(uuid4()),
                name=request.model_name,
                version=str(model_version.version),
                model_type=request.model_type,
                framework=run.data.tags.get("framework", "unknown"),
                stage=ModelStage.DEVELOPMENT,
                mlflow_run_id=request.run_id,
                model_uri=f"models:/{request.model_name}/{model_version.version}",
                metrics=dict(run.data.metrics),
                parameters=dict(run.data.params),
                tags={**dict(run.data.tags), **request.tags},
                created_by=run.data.tags.get("mlflow.user", "system")
            )
            
            self.db_session.add(model)
            self.db_session.commit()
            
            logger.info(f"Registered model {model.name} version {model.version}")
            return model.id
            
        except Exception as e:
            logger.error(f"Failed to register model: {e}")
            raise
    
    async def deploy_model(self, request: ModelDeploymentRequest) -> str:
        """Deploy a model to production"""
        deployment_id = str(uuid4())
        start_time = datetime.utcnow()
        
        try:
            # Get model info
            model = self.db_session.query(MLModel).get(request.model_id)
            if not model:
                raise ValueError(f"Model {request.model_id} not found")
            
            # Create deployment record
            deployment = ModelDeployment(
                id=deployment_id,
                model_id=request.model_id,
                deployment_name=request.deployment_name,
                target=request.target.value,
                configuration=request.configuration,
                status=DeploymentStatus.IN_PROGRESS
            )
            
            self.db_session.add(deployment)
            self.db_session.commit()
            
            # Deploy based on target
            if request.target == DeploymentTarget.DOCKER:
                endpoint_url = await self._deploy_to_docker(model, deployment, request)
            elif request.target == DeploymentTarget.KUBERNETES:
                endpoint_url = await self._deploy_to_kubernetes(model, deployment, request)
            elif request.target == DeploymentTarget.SAGEMAKER:
                endpoint_url = await self._deploy_to_sagemaker(model, deployment, request)
            else:
                raise ValueError(f"Unsupported deployment target: {request.target}")
            
            # Update deployment status
            deployment.endpoint_url = endpoint_url
            deployment.health_check_url = f"{endpoint_url}/health"
            deployment.status = DeploymentStatus.DEPLOYED
            deployment.deployed_at = datetime.utcnow()
            self.db_session.commit()
            
            # Update metrics
            duration = (datetime.utcnow() - start_time).total_seconds()
            self.metrics["deployment_duration"].labels(target=request.target.value).observe(duration)
            self.metrics["model_deployments"].labels(
                target=request.target.value,
                status="success"
            ).inc()
            self.metrics["active_deployments"].labels(stage=model.stage).inc()
            
            logger.info(f"Successfully deployed model {model.name} to {request.target}")
            return deployment_id
            
        except Exception as e:
            logger.error(f"Deployment failed: {e}")
            
            # Update deployment status
            deployment = self.db_session.query(ModelDeployment).get(deployment_id)
            deployment.status = DeploymentStatus.FAILED
            self.db_session.commit()
            
            self.metrics["model_deployments"].labels(
                target=request.target.value,
                status="failed"
            ).inc()
            
            raise
    
    async def _deploy_to_docker(self, model: MLModel, deployment: ModelDeployment, request: ModelDeploymentRequest) -> str:
        """Deploy model as Docker container"""
        try:
            # Load model
            model_uri = model.model_uri
            loaded_model = mlflow.pyfunc.load_model(model_uri)
            
            # Create serving script
            serving_script = self._create_serving_script(model)
            
            # Build Docker image
            image_tag = f"tracseq-model-{model.name}:{model.version}"
            
            dockerfile_content = f"""
FROM python:3.9-slim

WORKDIR /app

# Install dependencies
RUN pip install mlflow scikit-learn xgboost lightgbm fastapi uvicorn

# Copy model and serving script
COPY model /app/model
COPY serve.py /app/serve.py

# Expose port
EXPOSE 8000

# Run serving script
CMD ["uvicorn", "serve:app", "--host", "0.0.0.0", "--port", "8000"]
"""
            
            # Create temporary directory
            temp_dir = f"/tmp/model-deploy-{deployment.id}"
            os.makedirs(temp_dir, exist_ok=True)
            
            # Save model
            mlflow.pyfunc.save_model(
                path=os.path.join(temp_dir, "model"),
                python_model=loaded_model
            )
            
            # Save serving script
            with open(os.path.join(temp_dir, "serve.py"), "w") as f:
                f.write(serving_script)
            
            # Save Dockerfile
            with open(os.path.join(temp_dir, "Dockerfile"), "w") as f:
                f.write(dockerfile_content)
            
            # Build image
            image, logs = self.docker_client.images.build(
                path=temp_dir,
                tag=image_tag,
                rm=True
            )
            
            # Run container
            container = self.docker_client.containers.run(
                image_tag,
                name=f"model-{deployment.deployment_name}",
                detach=True,
                ports={'8000/tcp': None},
                environment={
                    "MODEL_NAME": model.name,
                    "MODEL_VERSION": model.version
                },
                restart_policy={"Name": "unless-stopped"}
            )
            
            # Get container port
            container.reload()
            port = container.ports['8000/tcp'][0]['HostPort']
            
            # Clean up
            shutil.rmtree(temp_dir)
            
            return f"http://localhost:{port}"
            
        except Exception as e:
            logger.error(f"Docker deployment failed: {e}")
            raise
    
    async def _deploy_to_kubernetes(self, model: MLModel, deployment: ModelDeployment, request: ModelDeploymentRequest) -> str:
        """Deploy model to Kubernetes"""
        # This would use kubectl or Kubernetes Python client
        # For now, return a placeholder
        return f"http://{request.deployment_name}.models.svc.cluster.local"
    
    async def _deploy_to_sagemaker(self, model: MLModel, deployment: ModelDeployment, request: ModelDeploymentRequest) -> str:
        """Deploy model to AWS SageMaker"""
        # This would use boto3 to deploy to SageMaker
        # For now, return a placeholder
        return f"https://{request.deployment_name}.sagemaker.aws.com"
    
    def _create_serving_script(self, model: MLModel) -> str:
        """Create FastAPI serving script for model"""
        return f"""
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import mlflow.pyfunc
import numpy as np
import pandas as pd
from typing import Dict, Any, List
import os

app = FastAPI(title="{model.name} Model Server", version="{model.version}")

# Load model
model = mlflow.pyfunc.load_model("/app/model")

class PredictionRequest(BaseModel):
    features: Dict[str, Any]

class PredictionResponse(BaseModel):
    prediction: Any
    model_name: str = "{model.name}"
    model_version: str = "{model.version}"

@app.post("/predict", response_model=PredictionResponse)
async def predict(request: PredictionRequest):
    try:
        # Convert features to DataFrame
        df = pd.DataFrame([request.features])
        
        # Make prediction
        prediction = model.predict(df)
        
        # Convert numpy types to Python types
        if isinstance(prediction, np.ndarray):
            prediction = prediction.tolist()
            if len(prediction) == 1:
                prediction = prediction[0]
        
        return PredictionResponse(prediction=prediction)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/health")
async def health():
    return {{"status": "healthy", "model": "{model.name}", "version": "{model.version}"}}

@app.get("/metrics")
async def metrics():
    return {{"requests_total": 0, "prediction_errors": 0}}
"""
    
    async def promote_model(self, request: ModelPromotionRequest) -> bool:
        """Promote model to different stage"""
        try:
            # Get model
            model = self.db_session.query(MLModel).get(request.model_id)
            if not model:
                raise ValueError(f"Model {request.model_id} not found")
            
            # Validate promotion rules
            if request.target_stage == ModelStage.PRODUCTION:
                # Check if model has been in staging
                if model.stage != ModelStage.STAGING:
                    raise ValueError("Model must be in staging before promotion to production")
                
                # Check model performance metrics
                if not self._validate_model_metrics(model):
                    raise ValueError("Model does not meet performance criteria for production")
            
            # Update model stage in MLflow
            self.mlflow_client.transition_model_version_stage(
                name=model.name,
                version=model.version,
                stage=request.target_stage.value.capitalize(),
                archive_existing_versions=True
            )
            
            # Update database
            old_stage = model.stage
            model.stage = request.target_stage.value
            model.updated_at = datetime.utcnow()
            
            # Add promotion history
            if not model.tags:
                model.tags = {}
            model.tags[f"promoted_to_{request.target_stage.value}"] = datetime.utcnow().isoformat()
            model.tags[f"promoted_by_{request.target_stage.value}"] = request.approved_by
            
            self.db_session.commit()
            
            logger.info(f"Promoted model {model.name} from {old_stage} to {request.target_stage}")
            return True
            
        except Exception as e:
            logger.error(f"Model promotion failed: {e}")
            raise
    
    def _validate_model_metrics(self, model: MLModel) -> bool:
        """Validate model meets performance criteria"""
        if not model.metrics:
            return False
        
        # Check minimum performance thresholds
        thresholds = self.config.get("production_thresholds", {
            "accuracy": 0.85,
            "f1_score": 0.80,
            "precision": 0.80,
            "recall": 0.80
        })
        
        for metric, threshold in thresholds.items():
            if metric in model.metrics and model.metrics[metric] < threshold:
                logger.warning(f"Model {model.name} {metric}={model.metrics[metric]} below threshold {threshold}")
                return False
        
        return True
    
    async def get_model_metrics(self, model_id: str) -> ModelMetricsResponse:
        """Get model performance metrics"""
        model = self.db_session.query(MLModel).get(model_id)
        if not model:
            raise ValueError(f"Model {model_id} not found")
        
        # Get comparison metrics from production model if exists
        comparison_metrics = None
        if model.stage != ModelStage.PRODUCTION:
            prod_model = self.db_session.query(MLModel).filter(
                MLModel.name == model.name,
                MLModel.stage == ModelStage.PRODUCTION
            ).first()
            
            if prod_model:
                comparison_metrics = prod_model.metrics
        
        return ModelMetricsResponse(
            model_id=model.id,
            model_name=model.name,
            version=model.version,
            stage=model.stage,
            metrics=model.metrics or {},
            created_at=model.created_at,
            comparison_metrics=comparison_metrics
        )
    
    async def rollback_deployment(self, deployment_id: str) -> bool:
        """Rollback a failed deployment"""
        try:
            deployment = self.db_session.query(ModelDeployment).get(deployment_id)
            if not deployment:
                raise ValueError(f"Deployment {deployment_id} not found")
            
            # Stop current deployment
            if deployment.target == DeploymentTarget.DOCKER:
                container_name = f"model-{deployment.deployment_name}"
                try:
                    container = self.docker_client.containers.get(container_name)
                    container.stop()
                    container.remove()
                except docker.errors.NotFound:
                    pass
            
            # Update status
            deployment.status = DeploymentStatus.ROLLBACK
            deployment.retired_at = datetime.utcnow()
            self.db_session.commit()
            
            self.metrics["active_deployments"].labels(stage="production").dec()
            
            logger.info(f"Rolled back deployment {deployment_id}")
            return True
            
        except Exception as e:
            logger.error(f"Rollback failed: {e}")
            raise
    
    async def create_ab_test(self, model_a_id: str, model_b_id: str, traffic_split: float = 0.5) -> str:
        """Create A/B test between two models"""
        # Implementation for A/B testing
        # This would set up routing rules to split traffic between models
        ab_test_id = str(uuid4())
        logger.info(f"Created A/B test {ab_test_id} between models {model_a_id} and {model_b_id}")
        return ab_test_id

# Model Monitoring
class ModelMonitor:
    """Monitor deployed models for drift and performance degradation"""
    
    def __init__(self, mlops_pipeline: MLOpsPipeline):
        self.pipeline = mlops_pipeline
        self.monitoring_tasks = {}
    
    async def start_monitoring(self, deployment_id: str, interval_minutes: int = 5):
        """Start monitoring a deployed model"""
        if deployment_id in self.monitoring_tasks:
            return
        
        task = asyncio.create_task(self._monitor_deployment(deployment_id, interval_minutes))
        self.monitoring_tasks[deployment_id] = task
    
    async def _monitor_deployment(self, deployment_id: str, interval_minutes: int):
        """Monitor deployment health and performance"""
        while True:
            try:
                deployment = self.pipeline.db_session.query(ModelDeployment).get(deployment_id)
                if not deployment or deployment.status != DeploymentStatus.DEPLOYED:
                    break
                
                # Check health
                # This would make HTTP request to health endpoint
                is_healthy = True  # Placeholder
                
                if not is_healthy:
                    logger.warning(f"Deployment {deployment_id} is unhealthy")
                    # Could trigger auto-recovery or alerts
                
                await asyncio.sleep(interval_minutes * 60)
                
            except Exception as e:
                logger.error(f"Monitoring error for deployment {deployment_id}: {e}")
                await asyncio.sleep(60)

# FastAPI Application
app = FastAPI(title="TracSeq MLOps Service", version="1.0.0")

# Global instances
mlops_pipeline = None
model_monitor = None

@app.on_event("startup")
async def startup_event():
    """Initialize MLOps pipeline on startup"""
    global mlops_pipeline, model_monitor
    
    config = {
        "mlflow_tracking_uri": "postgresql://ml_user:ml_pass@localhost:5436/mlflow",
        "database_url": "postgresql://ml_user:ml_pass@localhost:5436/ml_platform",
        "production_thresholds": {
            "accuracy": 0.85,
            "f1_score": 0.80
        }
    }
    
    mlops_pipeline = MLOpsPipeline(config)
    await mlops_pipeline.initialize()
    
    model_monitor = ModelMonitor(mlops_pipeline)

@app.post("/experiments")
async def create_experiment(request: ExperimentRequest):
    """Create a new ML experiment"""
    experiment_id = await mlops_pipeline.create_experiment(request)
    return {"experiment_id": experiment_id, "status": "created"}

@app.post("/models/register")
async def register_model(request: ModelRegistrationRequest):
    """Register a trained model"""
    model_id = await mlops_pipeline.register_model(request)
    return {"model_id": model_id, "status": "registered"}

@app.post("/models/deploy")
async def deploy_model(request: ModelDeploymentRequest, background_tasks: BackgroundTasks):
    """Deploy a model to production"""
    deployment_id = await mlops_pipeline.deploy_model(request)
    
    # Start monitoring in background
    background_tasks.add_task(model_monitor.start_monitoring, deployment_id)
    
    return {"deployment_id": deployment_id, "status": "deploying"}

@app.post("/models/promote")
async def promote_model(request: ModelPromotionRequest):
    """Promote model to different stage"""
    success = await mlops_pipeline.promote_model(request)
    return {"success": success, "new_stage": request.target_stage}

@app.get("/models/{model_id}/metrics")
async def get_model_metrics(model_id: str):
    """Get model performance metrics"""
    return await mlops_pipeline.get_model_metrics(model_id)

@app.post("/deployments/{deployment_id}/rollback")
async def rollback_deployment(deployment_id: str):
    """Rollback a deployment"""
    success = await mlops_pipeline.rollback_deployment(deployment_id)
    return {"success": success, "status": "rolled_back"}

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {"status": "healthy", "timestamp": datetime.utcnow().isoformat()}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8097)