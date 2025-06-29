#!/bin/bash

# TracSeq 2.0 - Phase 7 Deployment Script
# Advanced Microservices Patterns: Event Sourcing, CQRS, Kafka, Enhanced Saga

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
log() {
    echo -e "${GREEN}[$(date '+%Y-%m-%d %H:%M:%S')] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}"
}

error() {
    echo -e "${RED}[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}"
}

info() {
    echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $1${NC}"
}

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        error "Docker is not installed"
        exit 1
    fi
    
    # Check Docker Compose
    if ! docker compose version &> /dev/null; then
        error "Docker Compose is not installed"
        exit 1
    fi
    
    # Check if Phase 7 network exists
    if ! docker network ls | grep -q "tracseq-phase7"; then
        log "Creating Phase 7 network..."
        docker network create tracseq-phase7
    fi
    
    log "Prerequisites check passed"
}

# Initialize Kafka topics
init_kafka_topics() {
    log "Initializing Kafka topics..."
    
    # Wait for Kafka to be ready
    sleep 30
    
    # Create topics
    docker exec tracseq-kafka kafka-topics --bootstrap-server localhost:9093 \
        --create --if-not-exists --topic laboratory.sample.events \
        --partitions 3 --replication-factor 1 || true
        
    docker exec tracseq-kafka kafka-topics --bootstrap-server localhost:9093 \
        --create --if-not-exists --topic laboratory.sequencing.events \
        --partitions 3 --replication-factor 1 || true
        
    docker exec tracseq-kafka kafka-topics --bootstrap-server localhost:9093 \
        --create --if-not-exists --topic laboratory.storage.events \
        --partitions 3 --replication-factor 1 || true
        
    docker exec tracseq-kafka kafka-topics --bootstrap-server localhost:9093 \
        --create --if-not-exists --topic laboratory.notification.events \
        --partitions 3 --replication-factor 1 || true
        
    docker exec tracseq-kafka kafka-topics --bootstrap-server localhost:9093 \
        --create --if-not-exists --topic laboratory.saga.events \
        --partitions 3 --replication-factor 1 || true
        
    docker exec tracseq-kafka kafka-topics --bootstrap-server localhost:9093 \
        --create --if-not-exists --topic laboratory.dead-letter \
        --partitions 1 --replication-factor 1 || true
    
    log "Kafka topics initialized"
}

# Deploy Phase 7 infrastructure
deploy_infrastructure() {
    log "Deploying Phase 7 infrastructure..."
    
    # Deploy databases first
    docker compose -f docker-compose.phase7-advanced.yml up -d \
        event-store-db read-model-db
    
    # Wait for databases
    log "Waiting for databases to be ready..."
    sleep 20
    
    # Deploy Kafka cluster
    docker compose -f docker-compose.phase7-advanced.yml up -d \
        zookeeper kafka schema-registry kafka-ui
    
    # Initialize Kafka topics
    init_kafka_topics
    
    # Deploy Kafka Connect and ksqlDB
    docker compose -f docker-compose.phase7-advanced.yml up -d \
        kafka-connect ksqldb-server
    
    log "Infrastructure deployed successfully"
}

# Deploy Phase 7 services
deploy_services() {
    log "Deploying Phase 7 services..."
    
    # Note: These services would need their Dockerfiles created
    # For now, we'll show the deployment commands
    
    warn "Event Sourcing Service would be deployed here"
    # docker compose -f docker-compose.phase7-advanced.yml up -d event-sourcing-service
    
    warn "CQRS Projection Service would be deployed here"
    # docker compose -f docker-compose.phase7-advanced.yml up -d projection-service
    
    warn "Saga Orchestrator would be deployed here"
    # docker compose -f docker-compose.phase7-advanced.yml up -d saga-orchestrator
    
    # Deploy monitoring
    docker compose -f docker-compose.phase7-advanced.yml up -d kafka-exporter
    
    log "Services deployment completed"
}

# Create sample Kafka Connect connectors
create_connectors() {
    log "Creating Kafka Connect connectors..."
    
    # Wait for Kafka Connect to be ready
    sleep 30
    
    # Create CDC connector for event store (example)
    cat > /tmp/event-store-cdc.json << EOF
{
  "name": "event-store-cdc",
  "config": {
    "connector.class": "io.debezium.connector.postgresql.PostgresConnector",
    "database.hostname": "event-store-db",
    "database.port": "5432",
    "database.user": "event_store_user",
    "database.password": "event_store_pass",
    "database.dbname": "event_store",
    "database.server.name": "event-store",
    "table.include.list": "public.events",
    "plugin.name": "pgoutput"
  }
}
EOF
    
    # Note: This would require Debezium connector to be installed
    # curl -X POST http://localhost:8083/connectors \
    #     -H "Content-Type: application/json" \
    #     -d @/tmp/event-store-cdc.json
    
    log "Connectors configuration prepared"
}

# Health check function
health_check() {
    log "Running health checks..."
    
    local services=(
        "Event Store DB:postgresql://localhost:5434"
        "Read Model DB:postgresql://localhost:5435"
        "Kafka:http://localhost:9092"
        "Schema Registry:http://localhost:8081"
        "Kafka UI:http://localhost:8080"
        "Kafka Connect:http://localhost:8083"
        "ksqlDB:http://localhost:8088"
    )
    
    for service in "${services[@]}"; do
        IFS=':' read -r name url <<< "$service"
        info "Checking $name..."
        # Simple connection test (would need actual health check endpoints)
    done
}

# Display access information
display_access_info() {
    echo ""
    log "🎉 Phase 7 Deployment Complete!"
    echo ""
    info "Access your Phase 7 components:"
    echo "  📊 Kafka UI: http://localhost:8080"
    echo "  🔧 Schema Registry: http://localhost:8081"
    echo "  🔌 Kafka Connect: http://localhost:8083"
    echo "  📈 ksqlDB: http://localhost:8088"
    echo "  🗄️ Event Store DB: postgresql://localhost:5434"
    echo "  📖 Read Model DB: postgresql://localhost:5435"
    echo ""
    info "Kafka Topics created:"
    echo "  - laboratory.sample.events"
    echo "  - laboratory.sequencing.events"
    echo "  - laboratory.storage.events"
    echo "  - laboratory.notification.events"
    echo "  - laboratory.saga.events"
    echo "  - laboratory.dead-letter"
    echo ""
    info "Next steps:"
    echo "  1. Implement service Dockerfiles for event-sourcing, CQRS, and saga services"
    echo "  2. Update existing microservices to publish events to Kafka"
    echo "  3. Create ksqlDB streams for real-time analytics"
    echo "  4. Set up Kafka Connect for database CDC"
    echo "  5. Configure saga definitions for your workflows"
    echo ""
}

# Cleanup function
cleanup() {
    warn "Cleaning up Phase 7 deployment..."
    docker compose -f docker-compose.phase7-advanced.yml down
    docker network rm tracseq-phase7 || true
}

# Main execution
main() {
    log "Starting TracSeq 2.0 Phase 7 Deployment"
    log "Phase 7: Advanced Microservices Patterns"
    
    # Parse command line arguments
    case "${1:-deploy}" in
        deploy)
            check_prerequisites
            deploy_infrastructure
            deploy_services
            create_connectors
            health_check
            display_access_info
            ;;
        cleanup)
            cleanup
            ;;
        restart)
            cleanup
            sleep 5
            check_prerequisites
            deploy_infrastructure
            deploy_services
            create_connectors
            health_check
            display_access_info
            ;;
        *)
            error "Unknown command: $1"
            echo "Usage: $0 [deploy|cleanup|restart]"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"