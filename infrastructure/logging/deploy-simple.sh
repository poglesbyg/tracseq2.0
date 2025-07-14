#!/bin/bash

# Simple ELK Stack Deployment Script for TracSeq Laboratory System
# This script deploys a simplified version of Elasticsearch, Logstash, and Kibana

set -e

echo "🚀 Deploying Simple ELK Stack for TracSeq Laboratory System"
echo "=========================================================="

# Configuration
COMPOSE_FILE="docker-compose.simple.yml"

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

# Deploy simple ELK stack
deploy_elk() {
    print_status "Deploying simple ELK stack..."
    
    # Start services
    print_status "Starting ELK services..."
    docker-compose -f $COMPOSE_FILE up -d
    
    # Wait for services to be ready
    print_status "Waiting for services to be ready..."
    sleep 30
    
    # Check Elasticsearch
    print_status "Checking Elasticsearch..."
    timeout=120
    while ! curl -s http://localhost:9200/_cluster/health &> /dev/null; do
        if [ $timeout -le 0 ]; then
            print_error "Elasticsearch failed to start within 2 minutes"
            exit 1
        fi
        sleep 5
        timeout=$((timeout - 5))
        echo -n "."
    done
    echo
    print_status "Elasticsearch is ready"
    
    # Check Kibana
    print_status "Checking Kibana..."
    timeout=120
    while ! curl -s http://localhost:5601/api/status &> /dev/null; do
        if [ $timeout -le 0 ]; then
            print_error "Kibana failed to start within 2 minutes"
            exit 1
        fi
        sleep 5
        timeout=$((timeout - 5))
        echo -n "."
    done
    echo
    print_status "Kibana is ready"
    
    print_status "Simple ELK stack deployment completed successfully!"
}

# Create basic index template
create_index_template() {
    print_status "Creating basic index template..."
    
    sleep 5
    
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
                "environment": {"type": "keyword"},
                "system": {"type": "keyword"}
              }
            }
          }
        }' 2>/dev/null || print_warning "Failed to create index template"
    
    print_status "Index template created"
}

# Display access information
display_access_info() {
    print_status "Simple ELK Stack Access Information:"
    echo "=============================================="
    echo "🔍 Elasticsearch: http://localhost:9200"
    echo "📊 Kibana: http://localhost:5601"
    echo "🔧 Logstash TCP: localhost:5000"
    echo "🔧 Logstash Beats: localhost:5044"
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
            deploy_elk
            create_index_template
            health_check
            display_access_info
            ;;
        "stop")
            print_status "Stopping simple ELK stack..."
            docker-compose -f $COMPOSE_FILE down
            ;;
        "restart")
            print_status "Restarting simple ELK stack..."
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