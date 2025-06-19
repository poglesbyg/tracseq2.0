#!/bin/bash

# TracSeq 2.0 Microservices Migration Script
# This script helps migrate from monolithic to microservices architecture

set -e

# Configuration
MIGRATION_DIR="deploy/migration"
DOCKER_COMPOSE_FILE="$MIGRATION_DIR/docker-compose.migration.yml"
ENV_FILE="$MIGRATION_DIR/migration.env"
BACKUP_DIR="backups/migration"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default values
ACTION=""
FORCE=false
BACKUP=true

# Logging function
log_message() {
    local message="$1"
    local level="${2:-INFO}"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    case $level in
        "ERROR")   echo -e "[$timestamp] [${RED}$level${NC}] $message" ;;
        "WARN")    echo -e "[$timestamp] [${YELLOW}$level${NC}] $message" ;;
        "SUCCESS") echo -e "[$timestamp] [${GREEN}$level${NC}] $message" ;;
        *)         echo -e "[$timestamp] [$level] $message" ;;
    esac
}

# Show usage
show_usage() {
    echo "TracSeq 2.0 Microservices Migration Tool"
    echo ""
    echo "Usage: $0 <action> [options]"
    echo ""
    echo "Actions:"
    echo "  prepare    Prepare the migration environment"
    echo "  phase1     Start Phase 1 (Hybrid architecture)"
    echo "  phase2     Start Phase 2 (Partial migration)"
    echo "  phase3     Start Phase 3 (Full microservices)"
    echo "  rollback   Rollback to monolithic architecture"
    echo "  status     Check migration status"
    echo ""
    echo "Options:"
    echo "  --force       Force action without confirmation"
    echo "  --no-backup   Skip backup creation"
    echo "  --help        Show this help message"
    echo ""
}

# Parse command line arguments
parse_args() {
    if [ $# -eq 0 ]; then
        show_usage
        exit 1
    fi
    
    ACTION="$1"
    shift
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --force)
                FORCE=true
                shift
                ;;
            --no-backup)
                BACKUP=false
                shift
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                echo "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
    
    # Validate action
    if [[ ! "$ACTION" =~ ^(prepare|phase1|phase2|phase3|rollback|status)$ ]]; then
        echo "Invalid action: $ACTION"
        show_usage
        exit 1
    fi
}

# Check prerequisites
check_prerequisites() {
    log_message "Checking prerequisites..."
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_message "Docker is not installed or not in PATH" "ERROR"
        exit 1
    fi
    
    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null; then
        log_message "Docker Compose is not installed or not in PATH" "ERROR"
        exit 1
    fi
    
    # Check if migration files exist
    if [ ! -f "$DOCKER_COMPOSE_FILE" ]; then
        log_message "Migration Docker Compose file not found: $DOCKER_COMPOSE_FILE" "ERROR"
        exit 1
    fi
    
    if [ ! -f "$ENV_FILE" ]; then
        log_message "Migration environment file not found: $ENV_FILE" "ERROR"
        exit 1
    fi
    
    log_message "Prerequisites check passed" "SUCCESS"
}

# Backup current state
backup_current_state() {
    if [ "$BACKUP" = false ]; then
        log_message "Backup skipped (--no-backup flag)" "WARN"
        return
    fi
    
    log_message "Creating backup of current state..."
    
    local backup_timestamp=$(date '+%Y%m%d_%H%M%S')
    local backup_path="$BACKUP_DIR/$backup_timestamp"
    
    mkdir -p "$backup_path"
    
    # Backup databases
    log_message "Backing up databases..."
    
    # Export current monolith database
    local monolith_backup="$backup_path/monolith_db.sql"
    if docker ps | grep -q "lab_manager_db"; then
        docker exec lab_manager_db_1 pg_dump -U postgres -d lab_manager > "$monolith_backup"
    fi
    
    # Export volumes
    log_message "Backing up volumes..."
    if docker volume ls | grep -q "lab_manager_postgres_data"; then
        docker run --rm -v lab_manager_postgres_data:/data -v "${PWD}/${backup_path}:/backup" busybox tar czf /backup/postgres_data.tar.gz -C /data .
    fi
    
    if docker volume ls | grep -q "lab_manager_app_storage"; then
        docker run --rm -v lab_manager_app_storage:/data -v "${PWD}/${backup_path}:/backup" busybox tar czf /backup/app_storage.tar.gz -C /data .
    fi
    
    log_message "Backup completed: $backup_path" "SUCCESS"
}

# Start Migration Phase 1
start_migration_phase1() {
    log_message "Starting Migration Phase 1: Hybrid Architecture"
    
    # Update environment for Phase 1
    sed -i 's/MIGRATION_PHASE=.*/MIGRATION_PHASE=1/' "$ENV_FILE"
    
    # Start services with Phase 1 configuration
    log_message "Starting hybrid services..."
    
    cd "$MIGRATION_DIR"
    
    # Start core infrastructure
    docker-compose --env-file migration.env up -d \
        legacy-postgres \
        auth-postgres \
        storage-postgres \
        notification-postgres \
        rag-postgres \
        event-postgres \
        gateway-redis \
        event-redis \
        ollama
    
    sleep 10
    
    # Start microservices
    docker-compose --env-file migration.env up -d \
        auth-service \
        enhanced-storage-service \
        notification-service \
        enhanced-rag-service \
        event-service
    
    sleep 5
    
    # Start legacy monolith
    docker-compose --env-file migration.env up -d lab-manager
    
    sleep 5
    
    # Start API Gateway
    docker-compose --env-file migration.env up -d api-gateway
    
    sleep 5
    
    # Start Frontend
    docker-compose --env-file migration.env up -d frontend
    
    cd - > /dev/null
    
    log_message "Phase 1 deployment completed" "SUCCESS"
}

# Start Migration Phase 2
start_migration_phase2() {
    log_message "Starting Migration Phase 2: Partial Migration"
    
    # Update environment for Phase 2
    sed -i 's/MIGRATION_PHASE=.*/MIGRATION_PHASE=2/' "$ENV_FILE"
    
    cd "$MIGRATION_DIR"
    
    # Add more microservices
    docker-compose --env-file migration.env up -d \
        template-postgres \
        sequencing-postgres \
        template-service \
        sequencing-service
    
    # Update API Gateway routing
    docker-compose --env-file migration.env restart api-gateway
    
    cd - > /dev/null
    
    log_message "Phase 2 deployment completed" "SUCCESS"
}

# Start Migration Phase 3
start_migration_phase3() {
    log_message "Starting Migration Phase 3: Full Microservices"
    
    # Update environment for Phase 3
    sed -i 's/MIGRATION_PHASE=.*/MIGRATION_PHASE=3/' "$ENV_FILE"
    
    cd "$MIGRATION_DIR"
    
    # Add remaining microservices
    docker-compose --env-file migration.env up -d \
        sample-postgres \
        transaction-postgres \
        sample-service \
        transaction-service
    
    # Update API Gateway to route everything to microservices
    docker-compose --env-file migration.env restart api-gateway
    
    # Gracefully shut down legacy monolith
    log_message "Shutting down legacy monolith..."
    docker-compose --env-file migration.env stop lab-manager
    
    cd - > /dev/null
    
    log_message "Phase 3 deployment completed - Full microservices!" "SUCCESS"
}

# Get migration status
get_migration_status() {
    log_message "Checking migration status..."
    
    cd "$MIGRATION_DIR"
    
    local services=$(docker-compose --env-file migration.env ps --services)
    local running_services=$(docker-compose --env-file migration.env ps --filter "status=running" --services)
    
    echo -e "\n${CYAN}=== Migration Status ===${NC}"
    echo "Total Services: $(echo "$services" | wc -l)"
    echo "Running Services: $(echo "$running_services" | wc -l)"
    
    echo -e "\n${CYAN}=== Service Health ===${NC}"
    for service in $services; do
        local status=$(docker-compose --env-file migration.env ps "$service" --format "table {{.State}}" | tail -n 1)
        if [ "$status" = "running" ]; then
            echo -e "${GREEN}$service : $status${NC}"
        else
            echo -e "${RED}$service : $status${NC}"
        fi
    done
    
    # Check API Gateway health
    echo -e "\n${CYAN}=== API Gateway Health ===${NC}"
    if curl -f -s "http://localhost:8000/health" > /dev/null 2>&1; then
        echo -e "${GREEN}API Gateway: Healthy${NC}"
    else
        echo -e "${RED}API Gateway: Unhealthy${NC}"
    fi
    
    # Check Frontend
    echo -e "\n${CYAN}=== Frontend Health ===${NC}"
    if curl -f -s "http://localhost:8080" > /dev/null 2>&1; then
        echo -e "${GREEN}Frontend: Accessible${NC}"
    else
        echo -e "${RED}Frontend: Inaccessible${NC}"
    fi
    
    cd - > /dev/null
}

# Rollback migration
rollback_migration() {
    log_message "Rolling back migration..." "WARN"
    
    if [ "$FORCE" = false ]; then
        echo -n "This will stop all microservices and restore monolith. Continue? (y/N): "
        read -r confirmation
        if [[ ! "$confirmation" =~ ^[Yy]$ ]]; then
            log_message "Rollback cancelled" "INFO"
            return
        fi
    fi
    
    cd "$MIGRATION_DIR"
    
    # Stop all migration services
    docker-compose --env-file migration.env down
    
    # Restart original monolith
    cd ../..
    docker-compose up -d
    cd - > /dev/null
    
    log_message "Rollback completed" "SUCCESS"
}

# Prepare migration
prepare_migration() {
    log_message "Preparing migration environment..."
    
    # Create migration directories
    mkdir -p "$MIGRATION_DIR"
    mkdir -p "$BACKUP_DIR"
    
    # Stop current monolith
    log_message "Stopping current monolith services..."
    docker-compose down
    
    log_message "Migration preparation completed" "SUCCESS"
}

# Show next steps
show_next_steps() {
    case $ACTION in
        "phase1")
            echo -e "\n${CYAN}=== Next Steps ===${NC}"
            echo "1. Test the hybrid setup at http://localhost:8080"
            echo "2. Verify API Gateway at http://localhost:8000/health"
            echo "3. Run './migrate-to-microservices.sh phase2' when ready"
            ;;
        "phase2")
            echo -e "\n${CYAN}=== Next Steps ===${NC}"
            echo "1. Test additional services (Templates, Sequencing)"
            echo "2. Run './migrate-to-microservices.sh phase3' for full migration"
            ;;
        "phase3")
            echo -e "\n${GREEN}=== Congratulations! ===${NC}"
            echo "Full microservices migration completed!"
            echo "Your application is now running on distributed architecture."
            echo "Frontend: http://localhost:8080"
            echo "API Gateway: http://localhost:8000"
            ;;
    esac
}

# Main execution
main() {
    parse_args "$@"
    
    log_message "TracSeq 2.0 Microservices Migration Tool" "SUCCESS"
    log_message "Action: $ACTION"
    
    check_prerequisites
    
    case $ACTION in
        "prepare")
            prepare_migration
            ;;
        "phase1")
            if [ "$BACKUP" = true ]; then
                backup_current_state
            fi
            start_migration_phase1
            ;;
        "phase2")
            start_migration_phase2
            ;;
        "phase3")
            start_migration_phase3
            ;;
        "rollback")
            rollback_migration
            ;;
        "status")
            get_migration_status
            ;;
    esac
    
    if [[ "$ACTION" != "status" && "$ACTION" != "rollback" ]]; then
        log_message "Waiting for services to stabilize..."
        sleep 10
        get_migration_status
    fi
    
    log_message "Migration action '$ACTION' completed successfully!" "SUCCESS"
    
    show_next_steps
}

# Run main function with all arguments
main "$@" 
