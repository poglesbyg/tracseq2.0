# TracSeq 2.0 MLOps Pipeline Setup Guide

## Overview

This guide provides comprehensive instructions for setting up and using the TracSeq 2.0 MLOps pipeline - a complete machine learning operations infrastructure designed specifically for laboratory information management systems.

## 🎯 What is the TracSeq 2.0 MLOps Pipeline?

The TracSeq 2.0 MLOps pipeline is an enterprise-grade machine learning operations platform that provides:

- **Model Registry & Versioning**: Centralized model management with automated versioning
- **Experiment Tracking**: Comprehensive tracking of ML experiments with metrics and artifacts
- **A/B Testing Framework**: Statistical testing for model comparison and gradual rollouts
- **Continuous Learning**: Automated retraining pipelines with data drift detection
- **Model Monitoring**: Real-time performance monitoring with alerting
- **Automated Deployment**: Multi-strategy deployment with rollback capabilities
- **Data Pipeline**: Automated data preprocessing and feature engineering

## 🏗️ Architecture Overview

```mermaid
graph TB
    A[Data Sources] --> B[Data Pipeline]
    B --> C[Feature Store]
    C --> D[Model Training]
    D --> E[Experiment Tracker]
    D --> F[Model Registry]
    F --> G[A/B Testing]
    F --> H[Deployment Manager]
    H --> I[Production Models]
    I --> J[Model Monitor]
    J --> K[Continuous Learning]
    K --> D
    
    subgraph "MLOps Components"
        E
        F
        G
        H
        J
        K
    end
```

## 📋 Prerequisites

### System Requirements
- Python 3.9 or higher
- Docker and Docker Compose
- PostgreSQL (for production) or SQLite (for development)
- Redis (for background tasks)
- 8GB+ RAM recommended
- 50GB+ disk space for model storage

### Required Knowledge
- Python programming
- Basic ML/Data Science concepts
- Docker fundamentals
- Basic database operations

## 🚀 Quick Start

### 1. Installation

```bash
# Clone the repository (if not already done)
cd lab_submission_rag

# Install MLOps dependencies
pip install -r mlops_requirements.txt

# Or install specific components
pip install pandas numpy scikit-learn sqlalchemy aiofiles structlog
```

### 2. Basic Configuration

Create a configuration file:

```python
from mlops.config import ConfigManager, Environment

# Load default development configuration
config_manager = ConfigManager()
config = config_manager.load_config(Environment.DEVELOPMENT)

print(f"Database URL: {config.database.url}")
print(f"Model Registry Path: {config.model_registry.registry_path}")
```

### 3. Initialize Components

```python
import asyncio
from mlops import (
    ModelRegistry, ExperimentTracker, ABTestManager,
    ContinuousLearningPipeline, ModelMonitor, 
    ModelDeploymentManager, DataPipeline
)

async def initialize_mlops():
    # Initialize all components
    model_registry = ModelRegistry(
        registry_path="./mlops_data/models",
        database_url="sqlite:///mlops.db"
    )
    
    experiment_tracker = ExperimentTracker(
        tracking_dir="./mlops_data/experiments", 
        database_url="sqlite:///mlops.db"
    )
    
    # Start monitoring
    model_monitor = ModelMonitor(
        database_url="sqlite:///mlops.db",
        dashboard_dir="./mlops_data/dashboards"
    )
    await model_monitor.start_monitoring()
    
    print("✅ MLOps pipeline initialized!")

# Run initialization
asyncio.run(initialize_mlops())
```

### 4. Run the Demo

```bash
cd lab_submission_rag
python mlops_example.py
```

This will run a complete demonstration of all MLOps components.

## 📚 Detailed Component Guides

### Model Registry

The Model Registry provides centralized model management with versioning, metadata tracking, and lifecycle management.

#### Key Features:
- **Version Management**: Automatic versioning with semantic versioning support
- **Metadata Tracking**: Comprehensive model metadata including performance metrics
- **Lifecycle Management**: Support for staging, production, and archival states
- **Rollback Support**: Easy rollback to previous model versions

#### Usage Example:

```python
from mlops.model_registry import ModelRegistry, ModelMetadata, ModelStatus
from sklearn.ensemble import RandomForestClassifier
import asyncio

async def register_model_example():
    registry = ModelRegistry(
        registry_path="./models",
        database_url="sqlite:///mlops.db"
    )
    
    # Train a model
    model = RandomForestClassifier(n_estimators=100)
    # ... training code ...
    
    # Create metadata
    metadata = ModelMetadata(
        model_id="lab_classifier",
        version="1.0.0",
        name="Lab Sample Classifier",
        description="Classifies lab samples for approval",
        model_type="classification",
        framework="sklearn",
        accuracy=0.95,
        precision=0.93,
        recall=0.97,
        f1_score=0.95,
        created_by="data_scientist"
    )
    
    # Register model
    model_id = await registry.register_model(
        model=model,
        metadata=metadata,
        config={"n_estimators": 100}
    )
    
    print(f"Model registered: {model_id}")
    
    # Promote to production
    await registry.promote_model("lab_classifier", "1.0.0", ModelStatus.PRODUCTION)

asyncio.run(register_model_example())
```

### Experiment Tracking

Track ML experiments with comprehensive metrics, hyperparameters, and artifacts.

#### Key Features:
- **Experiment Management**: Organize experiments with metadata and tags
- **Metrics Logging**: Real-time metrics tracking with visualization
- **Artifact Storage**: Store models, plots, and other experiment artifacts
- **Comparison Tools**: Compare experiments across different runs

#### Usage Example:

```python
from mlops.experiment_tracker import ExperimentTracker, ExperimentConfig, ExperimentMetrics
import asyncio

async def experiment_tracking_example():
    tracker = ExperimentTracker(
        tracking_dir="./experiments",
        database_url="sqlite:///mlops.db"
    )
    
    # Start experiment
    config = ExperimentConfig(
        experiment_id="",
        name="Hyperparameter Tuning",
        description="Testing different n_estimators values",
        model_type="random_forest",
        hyperparameters={"n_estimators": 100, "max_depth": 10}
    )
    
    experiment_id = await tracker.start_experiment(config)
    
    # Log metrics during training
    for epoch in range(10):
        metrics = ExperimentMetrics(
            experiment_id=experiment_id,
            step=epoch,
            train_accuracy=0.8 + epoch * 0.02,
            val_accuracy=0.75 + epoch * 0.015
        )
        await tracker.log_metrics(metrics)
    
    # Log artifacts
    await tracker.log_artifact(
        name="model",
        artifact_type="model", 
        content=model,
        experiment_id=experiment_id
    )
    
    # Complete experiment
    await tracker.complete_experiment(experiment_id)

asyncio.run(experiment_tracking_example())
```

### A/B Testing Framework

Conduct statistically rigorous A/B tests for model comparison.

#### Key Features:
- **Multiple Test Types**: Champion/Challenger, Multivariate, Canary deployments
- **Statistical Analysis**: Automatic significance testing with confidence intervals
- **Traffic Management**: Flexible traffic allocation between variants
- **Guardrails**: Automatic stopping for safety violations

#### Usage Example:

```python
from mlops.ab_testing import ABTestManager, ABTestConfig, TestType
import asyncio

async def ab_testing_example():
    ab_manager = ABTestManager(
        database_url="sqlite:///mlops.db",
        results_dir="./ab_tests"
    )
    
    # Create A/B test
    config = ABTestConfig(
        test_id="",
        name="Model v1.1 vs v1.0",
        description="Testing improved model performance",
        test_type=TestType.CHAMPION_CHALLENGER,
        control_model_id="lab_classifier",
        control_model_version="1.0.0",
        treatment_models=[{"model_id": "lab_classifier", "version": "1.1.0"}],
        traffic_allocation={"control": 0.5, "treatment_1": 0.5},
        hypothesis="New model improves accuracy by 2%",
        primary_metric="accuracy"
    )
    
    test_id = await ab_manager.create_test(config)
    await ab_manager.start_test(test_id)
    
    # Simulate interactions
    for i in range(100):
        user_id = f"user_{i}"
        variant = await ab_manager.assign_variant(test_id, user_id)
        
        # Log interaction
        interaction = ABTestInteraction(
            interaction_id=f"int_{i}",
            test_id=test_id,
            variant_id=variant,
            user_id=user_id,
            request_data={"sample_type": "blood"},
            response_data={"prediction": "approved", "confidence": 0.95},
            latency_ms=150,
            success=True
        )
        await ab_manager.log_interaction(interaction)
    
    # Calculate results
    results = await ab_manager.calculate_results(test_id)
    for variant, result in results.items():
        print(f"{variant}: {result.accuracy:.3f} accuracy")

asyncio.run(ab_testing_example())
```

### Continuous Learning

Automate model retraining with data drift detection and performance monitoring.

#### Key Features:
- **Data Drift Detection**: Statistical tests for distribution changes
- **Performance Monitoring**: Automatic retraining on degradation
- **Scheduled Retraining**: Cron-based scheduled retraining
- **Quality Gates**: Validation before model promotion

#### Usage Example:

```python
from mlops.continuous_learning import ContinuousLearningPipeline, RetrainingConfig
import asyncio

async def continuous_learning_example():
    cl_pipeline = ContinuousLearningPipeline(
        database_url="sqlite:///mlops.db",
        data_dir="./continuous_learning",
        model_registry=model_registry,
        experiment_tracker=experiment_tracker
    )
    
    # Define training pipeline
    async def training_pipeline(train_data, val_data, test_data, config, tracker):
        # Your training logic here
        model = RandomForestClassifier()
        # ... training ...
        return model, {"n_estimators": 100}, {"accuracy": 0.92}
    
    cl_pipeline.register_training_pipeline("lab_model_pipeline", training_pipeline)
    
    # Create retraining configuration
    config = RetrainingConfig(
        config_id="",
        model_type="lab_classifier",
        training_pipeline="lab_model_pipeline",
        schedule_cron="0 2 * * 1",  # Weekly at 2 AM
        new_data_threshold=1000,
        minimum_accuracy=0.85
    )
    
    config_id = await cl_pipeline.create_retraining_config(config)
    
    # Start monitoring
    await cl_pipeline.start_monitoring()

asyncio.run(continuous_learning_example())
```

### Model Monitoring

Real-time monitoring of model performance with alerting.

#### Key Features:
- **Performance Metrics**: Accuracy, latency, throughput monitoring
- **System Metrics**: CPU, memory, and resource usage tracking
- **Alerting**: Threshold-based alerts with multiple channels
- **Dashboards**: Automated dashboard generation

#### Usage Example:

```python
from mlops.monitoring import ModelMonitor, MetricThreshold, MetricType
import asyncio

async def monitoring_example():
    monitor = ModelMonitor(
        database_url="sqlite:///mlops.db",
        dashboard_dir="./dashboards"
    )
    
    await monitor.start_monitoring()
    
    # Create threshold
    threshold = MetricThreshold(
        threshold_id="",
        model_id="lab_classifier",
        metric_name="prediction_latency",
        metric_type=MetricType.LATENCY,
        warning_threshold=500.0,
        critical_threshold=1000.0,
        operator="gt"
    )
    
    threshold_id = await monitor.create_threshold(threshold)
    
    # Record metrics
    await monitor.record_prediction_metrics(
        model_id="lab_classifier",
        model_version="1.0.0",
        prediction_latency_ms=250.0,
        prediction_success=True,
        accuracy=0.94
    )
    
    # Generate dashboard
    dashboard_path = await monitor.generate_dashboard("lab_classifier")
    print(f"Dashboard: {dashboard_path}")

asyncio.run(monitoring_example())
```

### Deployment Manager

Automated model deployment with multiple strategies and rollback support.

#### Key Features:
- **Deployment Strategies**: Blue-green, rolling, canary deployments
- **Health Checks**: Automated health validation
- **Rollback**: Automatic rollback on failures
- **Traffic Management**: Gradual traffic migration

#### Usage Example:

```python
from mlops.deployment_manager import ModelDeploymentManager, DeploymentConfig, DeploymentStrategy
import asyncio

async def deployment_example():
    deployment_manager = ModelDeploymentManager(
        database_url="sqlite:///mlops.db",
        container_registry_url="localhost:5000"
    )
    
    await deployment_manager.start_monitoring()
    
    # Deploy model
    config = DeploymentConfig(
        deployment_id="",
        model_id="lab_classifier", 
        model_version="1.0.0",
        environment=DeploymentEnvironment.STAGING,
        strategy=DeploymentStrategy.BLUE_GREEN,
        replicas=2,
        auto_rollback_enabled=True
    )
    
    deployment_id = await deployment_manager.deploy_model(config)
    print(f"Deployment started: {deployment_id}")

asyncio.run(deployment_example())
```

### Data Pipeline

Automated data preprocessing and feature engineering.

#### Key Features:
- **Schema Validation**: Automatic data validation against schemas
- **Quality Assessment**: Data quality scoring and issue detection
- **Feature Engineering**: Automated feature transformation
- **Pipeline Orchestration**: End-to-end data processing workflows

#### Usage Example:

```python
from mlops.data_pipeline import DataPipeline, DataSchema, FeatureEngineeringConfig
import asyncio

async def data_pipeline_example():
    pipeline = DataPipeline(
        database_url="sqlite:///mlops.db",
        data_dir="./data_pipeline"
    )
    
    # Define schema
    schema = DataSchema(
        schema_id="",
        name="Lab Data Schema",
        version="1.0",
        columns={
            "sample_id": {"type": "string", "nullable": False},
            "concentration": {"type": "float", "nullable": True},
            "approved": {"type": "boolean", "nullable": False}
        }
    )
    
    schema_id = await pipeline.create_schema(schema)
    
    # Define feature engineering
    feature_config = FeatureEngineeringConfig(
        config_id="",
        name="Lab Feature Engineering",
        numeric_features=["concentration"],
        categorical_features=["sample_type"],
        scaling_method="standard"
    )
    
    feature_config_id = await pipeline.create_feature_config(feature_config)
    
    # Run pipeline
    run_id = await pipeline.run_pipeline(
        pipeline_id="lab_pipeline",
        input_data_path="./data.csv",
        schema_id=schema_id,
        feature_config_id=feature_config_id
    )

asyncio.run(data_pipeline_example())
```

## 🔧 Configuration

### Environment Variables

```bash
# Database Configuration
export MLOPS_DATABASE_URL="postgresql://user:pass@localhost:5432/mlops"
export MLOPS_REDIS_URL="redis://localhost:6379"

# Storage Configuration  
export MLOPS_REGISTRY_PATH="/data/models"
export MLOPS_TRACKING_DIR="/data/experiments"
export MLOPS_DATA_DIR="/data/pipeline"

# Deployment Configuration
export MLOPS_CONTAINER_REGISTRY="registry.company.com"
export MLOPS_KUBERNETES_CONFIG="/path/to/kubeconfig"

# Security Configuration
export MLOPS_JWT_SECRET="your-secret-key"
export MLOPS_API_KEY="your-api-key"

# Monitoring Configuration
export MLOPS_HEALTH_CHECK_INTERVAL="30"
export MLOPS_ERROR_THRESHOLD="0.05"
```

### Configuration File

Create `mlops_config.json`:

```json
{
  "production": {
    "database": {
      "url": "postgresql://user:pass@prod-db:5432/mlops",
      "max_connections": 50,
      "pool_size": 20
    },
    "model_registry": {
      "registry_path": "/data/models",
      "backup_enabled": true,
      "backup_interval_hours": 6
    },
    "monitoring": {
      "health_check_interval_seconds": 15,
      "alert_cooldown_minutes": 5
    },
    "deployment": {
      "container_registry_url": "registry.company.com",
      "default_replicas": 3
    }
  }
}
```

## 📊 Monitoring and Observability

### Metrics Dashboard

Access the monitoring dashboard at:
- Development: `http://localhost:8080/dashboard`
- Production: Configure with your monitoring solution

### Key Metrics to Monitor

1. **Model Performance**
   - Accuracy, Precision, Recall, F1-Score
   - Prediction confidence scores
   - Error rates

2. **System Performance**
   - Prediction latency
   - Throughput (requests per second)
   - Memory and CPU usage

3. **Data Quality**
   - Data drift scores
   - Missing value percentages
   - Schema validation failures

4. **Operational Metrics**
   - Deployment success rates
   - Model training completion times
   - A/B test statistical significance

### Alerting

Configure alerts for:

```python
# High latency alert
latency_alert = MetricThreshold(
    model_id="lab_classifier",
    metric_name="prediction_latency",
    warning_threshold=500.0,
    critical_threshold=1000.0,
    alert_channels=["email", "slack"]
)

# Low accuracy alert
accuracy_alert = MetricThreshold(
    model_id="lab_classifier", 
    metric_name="accuracy",
    warning_threshold=0.85,
    critical_threshold=0.80,
    operator="lt",
    alert_channels=["email", "pagerduty"]
)
```

## 🚀 Production Deployment

### Docker Deployment

1. **Build MLOps Images**

```bash
# Build base MLOps image
docker build -t tracseq-mlops:latest .

# Build component-specific images
docker build -t tracseq-model-server:latest -f Dockerfile.model-server .
docker build -t tracseq-training:latest -f Dockerfile.training .
```

2. **Docker Compose**

```yaml
version: '3.8'
services:
  mlops-api:
    image: tracseq-mlops:latest
    ports:
      - "8080:8080"
    environment:
      - MLOPS_DATABASE_URL=postgresql://user:pass@db:5432/mlops
      - MLOPS_REDIS_URL=redis://redis:6379
    depends_on:
      - db
      - redis

  model-server:
    image: tracseq-model-server:latest
    ports:
      - "8081:8081"
    environment:
      - MODEL_REGISTRY_URL=http://mlops-api:8080

  db:
    image: postgres:14
    environment:
      POSTGRES_DB: mlops
      POSTGRES_USER: user
      POSTGRES_PASSWORD: pass
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:
```

### Kubernetes Deployment

1. **Deploy MLOps Platform**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mlops-platform
spec:
  replicas: 3
  selector:
    matchLabels:
      app: mlops-platform
  template:
    metadata:
      labels:
        app: mlops-platform
    spec:
      containers:
      - name: mlops-api
        image: tracseq-mlops:latest
        ports:
        - containerPort: 8080
        env:
        - name: MLOPS_DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: mlops-secrets
              key: database-url
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
```

### Scaling Considerations

1. **Database Scaling**
   - Use read replicas for heavy read workloads
   - Consider database partitioning for large datasets
   - Implement connection pooling

2. **Model Serving Scaling**
   - Use horizontal pod autoscaling
   - Implement model caching
   - Consider GPU nodes for deep learning models

3. **Storage Scaling**
   - Use object storage (S3, Azure Blob) for artifacts
   - Implement data lifecycle policies
   - Consider data compression

## 🔐 Security Best Practices

### Authentication and Authorization

1. **API Security**
```python
from mlops.security import JWTAuth, APIKeyAuth

# JWT Authentication
jwt_auth = JWTAuth(secret_key="your-secret")
api_key_auth = APIKeyAuth(valid_keys=["your-api-key"])
```

2. **Model Access Control**
```python
# Role-based access control
from mlops.auth import RoleBasedAuth

auth = RoleBasedAuth()
auth.add_permission("data_scientist", "models:read")
auth.add_permission("ml_engineer", "models:write")
auth.add_permission("admin", "models:admin")
```

### Data Security

1. **Encryption at Rest**
   - Encrypt model artifacts
   - Secure database connections
   - Use encrypted storage volumes

2. **Network Security**
   - Use TLS for all communications
   - Implement network policies
   - Secure service-to-service communication

### Audit Logging

```python
from mlops.audit import AuditLogger

audit = AuditLogger()
audit.log_model_access(user_id="user123", model_id="lab_classifier", action="predict")
audit.log_deployment(user_id="user123", model_id="lab_classifier", environment="production")
```

## 🧪 Testing

### Unit Tests

```bash
# Run unit tests
pytest lab_submission_rag/mlops/tests/unit/

# Run with coverage
pytest --cov=mlops --cov-report=html
```

### Integration Tests

```bash
# Run integration tests
pytest lab_submission_rag/mlops/tests/integration/

# Run end-to-end tests
pytest lab_submission_rag/mlops/tests/e2e/
```

### Performance Tests

```bash
# Load testing
locust -f mlops/tests/performance/load_test.py --host=http://localhost:8080
```

## 📈 Performance Optimization

### Database Optimization

1. **Indexing Strategy**
```sql
-- Create indexes for common queries
CREATE INDEX idx_model_registry_model_id ON model_registry(model_id);
CREATE INDEX idx_experiments_created_at ON experiments(created_at);
CREATE INDEX idx_metrics_experiment_id_step ON experiment_metrics(experiment_id, step);
```

2. **Query Optimization**
```python
# Use batch operations
await tracker.log_metrics_batch(metrics_list)

# Implement pagination
experiments = await tracker.list_experiments(limit=50, offset=100)
```

### Caching Strategy

```python
from mlops.cache import RedisCache

cache = RedisCache(redis_url="redis://localhost:6379")

# Cache model predictions
@cache.cached(ttl=3600)
async def get_model_prediction(model_id, input_data):
    return await model.predict(input_data)
```

### Async Processing

```python
import asyncio
from mlops.async_utils import AsyncBatchProcessor

# Process metrics in batches
processor = AsyncBatchProcessor(batch_size=100, max_wait_time=30)
await processor.add_metrics(metrics)
```

## 🐛 Troubleshooting

### Common Issues

1. **Database Connection Issues**
```bash
# Check database connectivity
python -c "from mlops.database import test_connection; test_connection()"

# Check connection pool
python -c "from mlops.database import get_connection_info; print(get_connection_info())"
```

2. **Model Loading Issues**
```python
# Debug model loading
from mlops.model_registry import ModelRegistry
registry = ModelRegistry("./models", "sqlite:///mlops.db")
model, metadata = await registry.get_model("lab_classifier", "1.0.0")
```

3. **Performance Issues**
```bash
# Profile application
python -m cProfile -o profile.stats mlops_example.py

# Analyze with snakeviz
pip install snakeviz
snakeviz profile.stats
```

### Logging Configuration

```python
import structlog
import logging

# Configure structured logging
logging.basicConfig(
    format="%(message)s",
    stream=sys.stdout,
    level=logging.INFO,
)

structlog.configure(
    processors=[
        structlog.stdlib.filter_by_level,
        structlog.stdlib.add_logger_name,
        structlog.stdlib.add_log_level,
        structlog.stdlib.PositionalArgumentsFormatter(),
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.StackInfoRenderer(),
        structlog.processors.format_exc_info,
        structlog.processors.UnicodeDecoder(),
        structlog.processors.JSONRenderer()
    ],
    context_class=dict,
    logger_factory=structlog.stdlib.LoggerFactory(),
    wrapper_class=structlog.stdlib.BoundLogger,
    cache_logger_on_first_use=True,
)
```

## 📚 Additional Resources

### Documentation
- [API Reference](./api/README.md)
- [Architecture Guide](./ARCHITECTURE.md)
- [Contributing Guide](./CONTRIBUTING.md)

### Examples
- [Complete Demo](../mlops_example.py)
- [Custom Training Pipeline](../examples/custom_training.py)
- [Advanced Monitoring](../examples/advanced_monitoring.py)

### Community
- [GitHub Issues](https://github.com/your-org/tracseq/issues)
- [Discussion Forum](https://github.com/your-org/tracseq/discussions)
- [Slack Channel](https://your-workspace.slack.com/channels/tracseq)

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](./CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone repository
git clone https://github.com/your-org/tracseq.git
cd tracseq

# Create virtual environment
python -m venv mlops-env
source mlops-env/bin/activate  # Linux/Mac
# or
mlops-env\Scripts\activate  # Windows

# Install development dependencies
pip install -r mlops_requirements.txt
pip install -r requirements-dev.txt

# Install pre-commit hooks
pre-commit install

# Run tests
pytest
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

---

*Context improved by Giga AI* 
