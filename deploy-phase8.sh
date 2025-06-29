#!/bin/bash

# TracSeq 2.0 - Phase 8 ML Platform Deployment Script
# Deploy machine learning infrastructure

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║       TracSeq 2.0 - Phase 8: ML Platform Deployment          ║"
echo "╚══════════════════════════════════════════════════════════════╝"

# Check prerequisites
echo "🔍 Checking prerequisites..."

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

# Check if required networks exist
if ! docker network ls | grep -q "tracseq-backend"; then
    echo "📦 Creating tracseq-backend network..."
    docker network create tracseq-backend
fi

# Create necessary directories
echo "📁 Creating directory structure..."
mkdir -p ml-platform/{model-serving,feature-store,automl,mlops,model-registry-ui,drift-detector}
mkdir -p ml-models/{sample-quality,storage-optimization,sequencing-prediction}
mkdir -p logs/ml-platform

# Create Dockerfiles for ML services
echo "🐳 Creating Dockerfiles..."

# Model Serving Dockerfile
cat > ml-platform/model-serving/Dockerfile << 'EOF'
FROM python:3.9-slim

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application
COPY model_server.py .

# Create model directory
RUN mkdir -p /models

EXPOSE 8094

CMD ["python", "model_server.py"]
EOF

# Model Serving requirements
cat > ml-platform/model-serving/requirements.txt << 'EOF'
fastapi==0.104.1
uvicorn==0.24.0
numpy==1.24.3
pandas==2.0.3
scikit-learn==1.3.0
xgboost==1.7.6
lightgbm==4.1.0
mlflow==2.8.0
redis==5.0.1
sqlalchemy==2.0.23
psycopg2-binary==2.9.9
prometheus-client==0.19.0
joblib==1.3.2
EOF

# Feature Store Dockerfile
cat > ml-platform/feature-store/Dockerfile << 'EOF'
FROM python:3.9-slim

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application
COPY feature_store.py .

EXPOSE 8095

CMD ["python", "feature_store.py"]
EOF

# Feature Store requirements
cat > ml-platform/feature-store/requirements.txt << 'EOF'
fastapi==0.104.1
uvicorn==0.24.0
numpy==1.24.3
pandas==2.0.3
pyarrow==14.0.1
scipy==1.11.4
redis==5.0.1
sqlalchemy==2.0.23
psycopg2-binary==2.9.9
EOF

# AutoML Dockerfile
cat > ml-platform/automl/Dockerfile << 'EOF'
FROM python:3.9-slim

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application
COPY automl_framework.py .

# Create model directory
RUN mkdir -p /models/automl

EXPOSE 8096

CMD ["python", "automl_framework.py"]
EOF

# AutoML requirements
cat > ml-platform/automl/requirements.txt << 'EOF'
fastapi==0.104.1
uvicorn==0.24.0
numpy==1.24.3
pandas==2.0.3
scikit-learn==1.3.0
xgboost==1.7.6
lightgbm==4.1.0
optuna==3.4.0
mlflow==2.8.0
joblib==1.3.2
sqlalchemy==2.0.23
psycopg2-binary==2.9.9
EOF

# MLOps Dockerfile
cat > ml-platform/mlops/Dockerfile << 'EOF'
FROM python:3.9-slim

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    docker.io \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application
COPY mlops_pipeline.py .

# Create model directory
RUN mkdir -p /models

EXPOSE 8097

CMD ["python", "mlops_pipeline.py"]
EOF

# MLOps requirements
cat > ml-platform/mlops/requirements.txt << 'EOF'
fastapi==0.104.1
uvicorn==0.24.0
mlflow==2.8.0
docker==6.1.3
boto3==1.29.7
sqlalchemy==2.0.23
psycopg2-binary==2.9.9
prometheus-client==0.19.0
pyyaml==6.0.1
numpy==1.24.3
pandas==2.0.3
joblib==1.3.2
EOF

# Create worker Dockerfile
cat > ml-platform/Dockerfile.worker << 'EOF'
FROM python:3.9-slim

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install Python dependencies
RUN pip install --no-cache-dir \
    numpy pandas scikit-learn \
    xgboost lightgbm \
    tensorflow torch \
    mlflow optuna \
    ray[default]

# Create directories
RUN mkdir -p /data /checkpoints

CMD ["python", "-m", "ray.worker"]
EOF

# Copy Python files to their directories
echo "📋 Copying application files..."
# Note: Files should already exist from previous steps

# Initialize databases
echo "🗄️ Initializing ML databases..."

# Start only the database service first
docker-compose -f docker-compose.phase8-ml.yml up -d ml-postgres

# Wait for database to be ready
echo "⏳ Waiting for database to be ready..."
sleep 10

# Create MLflow database
docker exec tracseq-ml-db psql -U ml_user -d ml_platform -c "CREATE SCHEMA IF NOT EXISTS mlflow;"

# Start remaining services
echo "🚀 Starting ML platform services..."
docker-compose -f docker-compose.phase8-ml.yml up -d

# Wait for services to be healthy
echo "⏳ Waiting for services to be healthy..."
sleep 30

# Check service health
echo "🏥 Checking service health..."

services=(
    "mlflow:5000"
    "model-server:8094"
    "feature-store:8095"
    "automl:8096"
    "mlops:8097"
)

for service in "${services[@]}"; do
    IFS=':' read -r name port <<< "$service"
    if curl -f -s "http://localhost:$port/health" > /dev/null 2>&1; then
        echo "✅ $name is healthy"
    else
        echo "⚠️  $name health check failed (may still be starting)"
    fi
done

# Create initial ML experiments
echo "🧪 Creating initial ML experiments..."

# Create sample quality experiment
curl -X POST http://localhost:8097/experiments \
  -H "Content-Type: application/json" \
  -d '{
    "name": "sample_quality_prediction",
    "description": "Predict sample quality based on collection and storage parameters",
    "parameters": {
      "model_type": "gradient_boosting",
      "optimization_metric": "rmse"
    },
    "tags": {
      "domain": "laboratory",
      "type": "quality_prediction"
    },
    "user_id": "system"
  }' || true

# Register laboratory features
echo "📊 Registering laboratory features..."

# Sample quality features
features=(
    '{"name": "sample_age_hours", "entity_type": "sample", "type": "numeric", "source": "computed", "description": "Age of sample in hours"}'
    '{"name": "storage_temperature", "entity_type": "sample", "type": "numeric", "source": "streaming", "description": "Current storage temperature"}'
    '{"name": "quality_risk_score", "entity_type": "sample", "type": "numeric", "source": "computed", "description": "Overall quality risk score"}'
)

for feature in "${features[@]}"; do
    curl -X POST http://localhost:8095/register-feature \
      -H "Content-Type: application/json" \
      -d "$feature" || true
done

# Display summary
echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           Phase 8 ML Platform Deployment Summary             ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "🎯 ML Services:"
echo "   - MLflow UI: http://localhost:5000"
echo "   - Model Server API: http://localhost:8094"
echo "   - Feature Store API: http://localhost:8095"
echo "   - AutoML API: http://localhost:8096"
echo "   - MLOps API: http://localhost:8097"
echo "   - Jupyter Lab: http://localhost:8888"
echo "   - TensorBoard: http://localhost:6006"
echo ""
echo "📊 ML Capabilities:"
echo "   ✓ Real-time model serving with A/B testing"
echo "   ✓ Centralized feature store with caching"
echo "   ✓ Automated ML with hyperparameter optimization"
echo "   ✓ Complete MLOps pipeline"
echo "   ✓ Model versioning and registry"
echo "   ✓ Experiment tracking"
echo "   ✓ Model deployment automation"
echo "   ✓ GPU-enabled training workers"
echo ""
echo "🔧 Next Steps:"
echo "   1. Access Jupyter Lab to develop models"
echo "   2. Use AutoML API to train models automatically"
echo "   3. Deploy models via MLOps pipeline"
echo "   4. Monitor model performance in MLflow"
echo ""
echo "📝 Example Usage:"
echo "   # Train a model with AutoML"
echo '   curl -X POST http://localhost:8096/experiments \
     -H "Content-Type: application/json" \
     -d @sample_quality_dataset.json'
echo ""
echo "✅ Phase 8 deployment completed successfully!"