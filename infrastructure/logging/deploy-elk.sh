#!/bin/bash

# ELK Stack Deployment Script for TracSeq Laboratory System
# This script deploys Elasticsearch, Logstash, Kibana, and Beats for centralized logging

set -e

echo "🚀 Deploying ELK Stack for TracSeq Laboratory System"
echo "=================================================="

# Configuration
COMPOSE_FILE="docker-compose.elk.yml"
STACK_NAME="tracseq-elk"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed"
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        print_error "Docker Compose is not installed"
        exit 1
    fi
    
    # Check available memory (Elasticsearch needs at least 2GB)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        MEMORY_MB=$(sysctl -n hw.memsize | awk '{print int($1/1024/1024)}')
    else
        MEMORY_MB=$(free -m | awk 'NR==2{print $2}')
    fi
    
    if [ "$MEMORY_MB" -lt 4096 ]; then
        print_warning "System has less than 4GB RAM. ELK stack may not perform optimally."
    fi
    
    print_status "Prerequisites check completed"
}

# Setup system settings for Elasticsearch
setup_system_settings() {
    print_status "Setting up system settings for Elasticsearch..."
    
    # Increase vm.max_map_count for Elasticsearch
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        sudo sysctl -w vm.max_map_count=262144
        echo "vm.max_map_count=262144" | sudo tee -a /etc/sysctl.conf
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        # For macOS with Docker Desktop
        print_warning "On macOS, ensure Docker Desktop has sufficient memory allocated (at least 4GB)"
    fi
    
    print_status "System settings configured"
}

# Create necessary directories
create_directories() {
    print_status "Creating necessary directories..."
    
    mkdir -p logs/elasticsearch
    mkdir -p logs/logstash
    mkdir -p logs/kibana
    mkdir -p logs/filebeat
    mkdir -p logs/metricbeat
    
    # Set permissions
    chmod 755 logs/*
    
    print_status "Directories created"
}

# Deploy ELK stack
deploy_elk() {
    print_status "Deploying ELK stack..."
    
    # Pull images first
    print_status "Pulling Docker images..."
    docker-compose -f $COMPOSE_FILE pull
    
    # Start services in order
    print_status "Starting Elasticsearch..."
    docker-compose -f $COMPOSE_FILE up -d elasticsearch
    
    # Wait for Elasticsearch to be ready
    print_status "Waiting for Elasticsearch to be ready..."
    timeout=300
    while ! curl -s http://localhost:9200/_cluster/health &> /dev/null; do
        if [ $timeout -le 0 ]; then
            print_error "Elasticsearch failed to start within 5 minutes"
            exit 1
        fi
        sleep 5
        timeout=$((timeout - 5))
        echo -n "."
    done
    echo
    print_status "Elasticsearch is ready"
    
    # Start Logstash
    print_status "Starting Logstash..."
    docker-compose -f $COMPOSE_FILE up -d logstash
    
    # Wait for Logstash to be ready
    print_status "Waiting for Logstash to be ready..."
    timeout=180
    while ! curl -s http://localhost:9600/_node/stats &> /dev/null; do
        if [ $timeout -le 0 ]; then
            print_error "Logstash failed to start within 3 minutes"
            exit 1
        fi
        sleep 5
        timeout=$((timeout - 5))
        echo -n "."
    done
    echo
    print_status "Logstash is ready"
    
    # Start Kibana
    print_status "Starting Kibana..."
    docker-compose -f $COMPOSE_FILE up -d kibana
    
    # Wait for Kibana to be ready
    print_status "Waiting for Kibana to be ready..."
    timeout=300
    while ! curl -s http://localhost:5601/api/status &> /dev/null; do
        if [ $timeout -le 0 ]; then
            print_error "Kibana failed to start within 5 minutes"
            exit 1
        fi
        sleep 5
        timeout=$((timeout - 5))
        echo -n "."
    done
    echo
    print_status "Kibana is ready"
    
    # Start Beats
    print_status "Starting Filebeat and Metricbeat..."
    docker-compose -f $COMPOSE_FILE up -d filebeat metricbeat
    
    # Start APM Server
    print_status "Starting APM Server..."
    docker-compose -f $COMPOSE_FILE up -d apm-server
    
    print_status "ELK stack deployment completed successfully!"
}

# Create index templates
create_index_templates() {
    print_status "Creating index templates..."
    
    # Wait a bit for everything to settle
    sleep 10
    
    # Create index template for TracSeq logs
    curl -X PUT "localhost:9200/_index_template/tracseq-logs" \
        -H "Content-Type: application/json" \
        -d '{
          "index_patterns": ["tracseq-logs-*"],
          "template": {
            "settings": {
              "number_of_shards": 1,
              "number_of_replicas": 0,
              "index.refresh_interval": "5s"
            },
            "mappings": {
              "properties": {
                "@timestamp": {"type": "date"},
                "level": {"type": "keyword"},
                "service": {"type": "keyword"},
                "logger": {"type": "keyword"},
                "message": {"type": "text"},
                "request_id": {"type": "keyword"},
                "trace_id": {"type": "keyword"},
                "laboratory_entity": {"type": "keyword"},
                "entity_id": {"type": "keyword"},
                "processing_time_ms": {"type": "float"},
                "container_name": {"type": "keyword"},
                "container_image": {"type": "keyword"},
                "environment": {"type": "keyword"},
                "system": {"type": "keyword"}
              }
            }
          }
        }'
    
    print_status "Index templates created"
}

# Display access information
display_access_info() {
    print_status "ELK Stack Access Information:"
    echo "=============================================="
    echo "🔍 Elasticsearch: http://localhost:9200"
    echo "📊 Kibana: http://localhost:5601"
    echo "🔧 Logstash: http://localhost:9600"
    echo "📈 APM Server: http://localhost:8200"
    echo "=============================================="
    echo
    print_status "Service Status:"
    docker-compose -f $COMPOSE_FILE ps
    echo
    print_status "To view logs: docker-compose -f $COMPOSE_FILE logs -f [service_name]"
    print_status "To stop: docker-compose -f $COMPOSE_FILE down"
    print_status "To restart: docker-compose -f $COMPOSE_FILE restart [service_name]"
}

# Health check
health_check() {
    print_status "Performing health check..."
    
    # Check Elasticsearch
    if curl -s http://localhost:9200/_cluster/health | grep -q '"status":"green\|yellow"'; then
        print_status "✅ Elasticsearch is healthy"
    else
        print_error "❌ Elasticsearch is not healthy"
    fi
    
    # Check Kibana
    if curl -s http://localhost:5601/api/status | grep -q '"overall":{"level":"available"'; then
        print_status "✅ Kibana is healthy"
    else
        print_error "❌ Kibana is not healthy"
    fi
    
    # Check Logstash
    if curl -s http://localhost:9600/_node/stats | grep -q '"status":"green"'; then
        print_status "✅ Logstash is healthy"
    else
        print_warning "⚠️  Logstash health check inconclusive"
    fi
}

# Main execution
main() {
    case "${1:-deploy}" in
        "deploy")
            check_prerequisites
            setup_system_settings
            create_directories
            deploy_elk
            create_index_templates
            health_check
            display_access_info
            ;;
        "stop")
            print_status "Stopping ELK stack..."
            docker-compose -f $COMPOSE_FILE down
            ;;
        "restart")
            print_status "Restarting ELK stack..."
            docker-compose -f $COMPOSE_FILE restart
            ;;
        "status")
            docker-compose -f $COMPOSE_FILE ps
            health_check
            ;;
        "logs")
            docker-compose -f $COMPOSE_FILE logs -f "${2:-}"
            ;;
        *)
            echo "Usage: $0 {deploy|stop|restart|status|logs [service]}"
            exit 1
            ;;
    esac
}

# Run main function
main "$@" 