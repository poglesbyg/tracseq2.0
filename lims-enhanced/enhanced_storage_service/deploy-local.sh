#!/bin/bash

# ========================================
# Enhanced Storage Service - Local Deployment Script
# ========================================

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_NAME="enhanced-storage-service"
DOCKER_COMPOSE_FILE="docker-compose.local.yml"
ENV_FILE="local.env"

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    
    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        log_error "Docker Compose is not installed. Please install Docker Compose first."
        exit 1
    fi
    
    # Check if Docker is running
    if ! docker info &> /dev/null; then
        log_error "Docker is not running. Please start Docker first."
        exit 1
    fi
    
    # Check Rust toolchain
    if ! command -v cargo &> /dev/null; then
        log_warning "Rust toolchain not found. Will rely on Docker build."
    fi
    
    log_success "Prerequisites check completed"
}

# Create necessary directories
create_directories() {
    log_info "Creating necessary directories..."
    
    mkdir -p config/mosquitto
    mkdir -p config/prometheus
    mkdir -p config/grafana/provisioning/dashboards
    mkdir -p config/grafana/provisioning/datasources
    mkdir -p config/grafana/dashboards
    mkdir -p config/nginx
    mkdir -p mocks/lims/mappings
    mkdir -p mocks/erp/mappings
    mkdir -p scripts
    mkdir -p frontend/src
    mkdir -p frontend/public
    
    log_success "Directories created"
}

# Create configuration files
create_config_files() {
    log_info "Creating configuration files..."
    
    # Mosquitto MQTT configuration
    cat > config/mosquitto/mosquitto.conf << 'EOF'
# Mosquitto MQTT Broker Configuration
listener 1883
allow_anonymous true
persistence true
persistence_location /mosquitto/data/
log_dest file /mosquitto/log/mosquitto.log
log_type error
log_type warning
log_type notice
log_type information
EOF

    # Prometheus configuration
    cat > config/prometheus/prometheus.yml << 'EOF'
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'enhanced-storage-service'
    static_configs:
      - targets: ['enhanced-storage-service:9100']
    scrape_interval: 5s
    metrics_path: /metrics

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres:5432']

  - job_name: 'redis'
    static_configs:
      - targets: ['redis:6379']
EOF

    # Grafana datasource configuration
    cat > config/grafana/provisioning/datasources/prometheus.yml << 'EOF'
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
EOF

    # Mock LIMS service mappings
    cat > mocks/lims/mappings/samples.json << 'EOF'
{
  "request": {
    "method": "GET",
    "urlPath": "/api/samples"
  },
  "response": {
    "status": 200,
    "headers": {
      "Content-Type": "application/json"
    },
    "jsonBody": {
      "samples": [
        {
          "id": "LIMS-001",
          "barcode": "SAMPLE-20241201-001",
          "status": "processed",
          "type": "blood",
          "created_at": "2024-12-01T10:00:00Z"
        }
      ],
      "total": 1
    }
  }
}
EOF

    # Mock ERP service mappings
    cat > mocks/erp/mappings/inventory.json << 'EOF'
{
  "request": {
    "method": "GET",
    "urlPath": "/api/inventory"
  },
  "response": {
    "status": 200,
    "headers": {
      "Content-Type": "application/json"
    },
    "jsonBody": {
      "inventory": [
        {
          "item_id": "REAGENT-001",
          "name": "DNA Extraction Kit",
          "quantity": 50,
          "unit": "units",
          "cost_per_unit": 25.99
        }
      ],
      "total": 1
    }
  }
}
EOF

    # Nginx configuration
    cat > config/nginx/nginx.conf << 'EOF'
events {
    worker_connections 1024;
}

http {
    upstream enhanced_storage_backend {
        server enhanced-storage-service:8082;
    }

    upstream enhanced_storage_frontend {
        server frontend:3000;
    }

    server {
        listen 80;
        server_name localhost;

        # Frontend routes
        location / {
            proxy_pass http://enhanced_storage_frontend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        # API routes
        location /api/ {
            proxy_pass http://enhanced_storage_backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        # WebSocket support
        location /ws/ {
            proxy_pass http://enhanced_storage_backend;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }
    }
}
EOF

    log_success "Configuration files created"
}

# Create a simple frontend placeholder
create_frontend() {
    log_info "Creating frontend placeholder..."
    
    # Dockerfile for frontend
    cat > frontend/Dockerfile << 'EOF'
FROM node:18-alpine

WORKDIR /app

# Create a simple React app placeholder
RUN npx create-react-app . --template typescript

# Install additional dependencies
RUN npm install axios recharts @mui/material @emotion/react @emotion/styled

EXPOSE 3000

CMD ["npm", "start"]
EOF

    log_success "Frontend placeholder created"
}

# Build the application
build_application() {
    log_info "Building Enhanced Storage Service..."
    
    # Build with Docker Compose
    if command -v docker-compose &> /dev/null; then
        docker-compose -f $DOCKER_COMPOSE_FILE build
    else
        docker compose -f $DOCKER_COMPOSE_FILE build
    fi
    
    log_success "Application built successfully"
}

# Start services
start_services() {
    log_info "Starting services..."
    
    if command -v docker-compose &> /dev/null; then
        docker-compose -f $DOCKER_COMPOSE_FILE up -d
    else
        docker compose -f $DOCKER_COMPOSE_FILE up -d
    fi
    
    log_success "Services started"
}

# Wait for services to be ready
wait_for_services() {
    log_info "Waiting for services to be ready..."
    
    # Wait for database
    log_info "Waiting for PostgreSQL..."
    until docker exec $(docker ps -q -f name=postgres) pg_isready -U postgres -d enhanced_storage &> /dev/null; do
        echo -n "."
        sleep 2
    done
    echo ""
    
    # Wait for main service
    log_info "Waiting for Enhanced Storage Service..."
    local count=0
    until curl -f http://localhost:8082/health &> /dev/null; do
        if [ $count -gt 30 ]; then
            log_error "Service health check timeout"
            return 1
        fi
        echo -n "."
        sleep 5
        count=$((count + 1))
    done
    echo ""
    
    log_success "All services are ready"
}

# Display service URLs
display_service_urls() {
    log_info "Deployment completed! Access your services:"
    echo ""
    echo "🚀 Enhanced Storage Service API: http://localhost:8082"
    echo "📊 Grafana Dashboard: http://localhost:3000 (admin/admin)"
    echo "📈 Prometheus Metrics: http://localhost:9090"
    echo "🔍 Jaeger Tracing: http://localhost:16686"
    echo "💾 MinIO Console: http://localhost:9001 (minioadmin/minioadmin)"
    echo "🌐 Main Application: http://localhost:80"
    echo "🔧 Mock LIMS: http://localhost:8090"
    echo "💼 Mock ERP: http://localhost:8091"
    echo ""
    echo "🔗 Key API Endpoints:"
    echo "  - Health Check: http://localhost:8082/health"
    echo "  - Storage Overview: http://localhost:8082/storage/overview"
    echo "  - IoT Sensors: http://localhost:8082/iot/sensors"
    echo "  - AI Platform: http://localhost:8082/ai/overview"
    echo "  - Integrations: http://localhost:8082/integrations/overview"
    echo ""
    echo "📝 To check logs: docker logs enhanced-storage-service_enhanced-storage-service_1"
    echo "🛑 To stop services: ./deploy-local.sh stop"
}

# Stop services
stop_services() {
    log_info "Stopping services..."
    
    if command -v docker-compose &> /dev/null; then
        docker-compose -f $DOCKER_COMPOSE_FILE down
    else
        docker compose -f $DOCKER_COMPOSE_FILE down
    fi
    
    log_success "Services stopped"
}

# Clean up everything
cleanup() {
    log_info "Cleaning up..."
    
    if command -v docker-compose &> /dev/null; then
        docker-compose -f $DOCKER_COMPOSE_FILE down -v --remove-orphans
    else
        docker compose -f $DOCKER_COMPOSE_FILE down -v --remove-orphans
    fi
    
    docker system prune -f
    
    log_success "Cleanup completed"
}

# Test the deployment
test_deployment() {
    log_info "Running deployment tests..."
    
    # Test health endpoint
    if curl -f http://localhost:8082/health &> /dev/null; then
        log_success "✅ Health check passed"
    else
        log_error "❌ Health check failed"
        return 1
    fi
    
    # Test API endpoints
    if curl -f http://localhost:8082/storage/overview &> /dev/null; then
        log_success "✅ Storage API accessible"
    else
        log_warning "⚠️  Storage API not accessible yet"
    fi
    
    if curl -f http://localhost:8082/ai/overview &> /dev/null; then
        log_success "✅ AI Platform API accessible"
    else
        log_warning "⚠️  AI Platform API not accessible yet"
    fi
    
    if curl -f http://localhost:8082/integrations/overview &> /dev/null; then
        log_success "✅ Integrations API accessible"
    else
        log_warning "⚠️  Integrations API not accessible yet"
    fi
    
    log_success "Deployment tests completed"
}

# Main execution
main() {
    case "${1:-deploy}" in
        "deploy")
            log_info "🚀 Starting Enhanced Storage Service local deployment..."
            check_prerequisites
            create_directories
            create_config_files
            create_frontend
            build_application
            start_services
            wait_for_services
            test_deployment
            display_service_urls
            ;;
        "start")
            start_services
            wait_for_services
            display_service_urls
            ;;
        "stop")
            stop_services
            ;;
        "restart")
            stop_services
            start_services
            wait_for_services
            display_service_urls
            ;;
        "test")
            test_deployment
            ;;
        "clean")
            cleanup
            ;;
        "logs")
            if command -v docker-compose &> /dev/null; then
                docker-compose -f $DOCKER_COMPOSE_FILE logs -f enhanced-storage-service
            else
                docker compose -f $DOCKER_COMPOSE_FILE logs -f enhanced-storage-service
            fi
            ;;
        *)
            echo "Usage: $0 {deploy|start|stop|restart|test|clean|logs}"
            echo ""
            echo "Commands:"
            echo "  deploy   - Full deployment (default)"
            echo "  start    - Start services"
            echo "  stop     - Stop services"
            echo "  restart  - Restart services"
            echo "  test     - Test deployment"
            echo "  clean    - Clean up everything"
            echo "  logs     - Show service logs"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@" 
