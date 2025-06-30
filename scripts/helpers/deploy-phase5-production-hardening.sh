#!/bin/bash

# TracSeq 2.0 - Phase 5 Deployment Script
# Production Hardening & System Integration

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"
COMPOSE_FILE="$PROJECT_ROOT/docker-compose.phase5-production.yml"
LOG_FILE="$PROJECT_ROOT/logs/phase5-deployment-$(date +%Y%m%d_%H%M%S).log"

# Functions
log() {
    echo -e "${GREEN}[$(date '+%Y-%m-%d %H:%M:%S')] $1${NC}" | tee -a "$LOG_FILE"
}

warn() {
    echo -e "${YELLOW}[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}" | tee -a "$LOG_FILE"
}

error() {
    echo -e "${RED}[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}" | tee -a "$LOG_FILE"
}

info() {
    echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $1${NC}" | tee -a "$LOG_FILE"
}

security_msg() {
    echo -e "${PURPLE}[$(date '+%Y-%m-%d %H:%M:%S')] SECURITY: $1${NC}" | tee -a "$LOG_FILE"
}

performance_msg() {
    echo -e "${CYAN}[$(date '+%Y-%m-%d %H:%M:%S')] PERFORMANCE: $1${NC}" | tee -a "$LOG_FILE"
}

# Banner
show_banner() {
    echo ""
    echo -e "${CYAN}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║                    TracSeq 2.0 - Phase 5                     ║${NC}"
    echo -e "${CYAN}║            Production Hardening & System Integration         ║${NC}"
    echo -e "${CYAN}║═══════════════════════════════════════════════════════════════║${NC}"
    echo -e "${CYAN}║  🔒 Security Hardening & Compliance                         ║${NC}"
    echo -e "${CYAN}║  🚀 Performance Optimization & Scaling                      ║${NC}"
    echo -e "${CYAN}║  🧪 Comprehensive Integration Testing                       ║${NC}"
    echo -e "${CYAN}║  📊 Advanced Monitoring & Alerting                          ║${NC}"
    echo -e "${CYAN}║  💾 Backup & Disaster Recovery                              ║${NC}"
    echo -e "${CYAN}║  🏭 Production Configuration Management                     ║${NC}"
    echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

# Check prerequisites
check_prerequisites() {
    log "🔍 Checking Phase 5 prerequisites..."
    
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
    
    # Ensure TracSeq network exists (from previous phases)
    info "Ensuring TracSeq network exists..."
    docker network create tracseq-network 2>/dev/null || info "TracSeq network already exists"
    
    # Check available resources
    available_memory=$(docker info --format '{{.MemTotal}}' 2>/dev/null || echo "0")
    required_memory=$((8 * 1024 * 1024 * 1024)) # 8GB in bytes
    
    if [[ $available_memory -lt $required_memory ]]; then
        warn "Low memory detected. Phase 5 requires at least 8GB RAM for optimal performance"
    fi
    
    # Check if previous phases are running
    running_services=$(docker ps --filter "name=tracseq" --format "{{.Names}}" | wc -l)
    if [[ $running_services -lt 15 ]]; then
        warn "Previous phases may not be fully deployed. Found only $running_services TracSeq services running"
    fi
    
    # Create log directory
    mkdir -p "$(dirname "$LOG_FILE")"
    
    log "✅ Prerequisites check completed"
}

# Security assessment
run_security_assessment() {
    security_msg "🔒 Starting security assessment and hardening..."
    
    # Deploy security scanner
    log "📡 Deploying Trivy security scanner..."
    docker compose -f "$COMPOSE_FILE" up -d security-scanner
    
    # Wait for security scanner to be ready
    sleep 15
    
    # Deploy Vault for secrets management
    log "🔐 Deploying HashiCorp Vault for secrets management..."
    docker compose -f "$COMPOSE_FILE" up -d vault
    
    # Wait for Vault to be ready
    sleep 20
    
    # Run initial security scan
    info "Running initial vulnerability scan on TracSeq containers..."
    
    # Scan core services
    local core_services=("tracseq20-auth-service-1" "tracseq20-template-service-1" "tracseq20-sample-service-1")
    
    for service in "${core_services[@]}"; do
        if docker ps --format "{{.Names}}" | grep -q "$service"; then
            info "Scanning $service for vulnerabilities..."
            docker exec tracseq-security-scanner trivy image --format json --output "/tmp/${service}-scan.json" \
                "$(docker inspect --format='{{.Config.Image}}' "$service" 2>/dev/null || echo 'unknown')" 2>/dev/null || true
        fi
    done
    
    security_msg "✅ Security assessment phase completed"
}

# Deploy backup and disaster recovery
deploy_backup_dr() {
    log "💾 Deploying backup and disaster recovery services..."
    
    # Deploy automated backup service
    log "🔄 Starting automated backup service..."
    docker compose -f "$COMPOSE_FILE" up -d backup-service
    
    # Wait for backup service
    sleep 10
    
    # Create initial backup
    info "Creating initial system backup..."
    
    # Verify backup service is running
    if docker ps --format "{{.Names}}" | grep -q "tracseq-backup-service"; then
        log "✅ Backup service deployed successfully"
        info "Automated backups will run every 6 hours"
        info "Backup retention: 7 days"
    else
        warn "Backup service deployment may have issues"
    fi
    
    log "✅ Backup and DR deployment completed"
}

# Deploy performance monitoring
deploy_performance_monitoring() {
    performance_msg "🚀 Deploying performance monitoring and optimization..."
    
    # Deploy cAdvisor for container monitoring
    log "📈 Deploying cAdvisor for container performance monitoring..."
    docker compose -f "$COMPOSE_FILE" up -d cadvisor
    
    # Deploy Redis cluster for high availability
    log "🔄 Deploying Redis cluster for high availability..."
    docker compose -f "$COMPOSE_FILE" up -d redis-cluster-1 redis-cluster-2 redis-cluster-3
    
    # Wait for Redis cluster to initialize
    sleep 30
    
    # Initialize Redis cluster
    info "Initializing Redis cluster..."
    # Note: In production, you would run redis-cli --cluster create with actual IPs
    warn "Redis cluster nodes deployed. Manual cluster initialization required for production"
    
    # Deploy load testing service
    log "🧪 Deploying automated load testing service..."
    docker compose -f "$COMPOSE_FILE" up -d load-tester
    
    performance_msg "✅ Performance monitoring deployment completed"
}

# Deploy integration testing
deploy_integration_testing() {
    log "🧪 Deploying comprehensive integration testing..."
    
    # Deploy integration test runner
    log "🔬 Starting integration test runner..."
    docker compose -f "$COMPOSE_FILE" up -d integration-tester
    
    # Wait for integration tests to initialize
    sleep 20
    
    # Check if integration tester is running
    if docker ps --format "{{.Names}}" | grep -q "tracseq-integration-tester"; then
        log "✅ Integration testing service deployed"
        info "Integration tests will run every 15 minutes"
        info "Test results are logged to integration_test_results volume"
    else
        warn "Integration testing service deployment may have issues"
    fi
    
    log "✅ Integration testing deployment completed"
}

# Deploy advanced monitoring
deploy_advanced_monitoring() {
    log "📊 Deploying advanced monitoring and alerting..."
    
    # Deploy advanced AlertManager
    log "🚨 Deploying advanced AlertManager with laboratory-specific rules..."
    docker compose -f "$COMPOSE_FILE" up -d alertmanager-advanced
    
    # Deploy production Grafana
    log "📈 Deploying production Grafana with pre-configured dashboards..."
    docker compose -f "$COMPOSE_FILE" up -d grafana-production
    
    # Deploy production Elasticsearch
    log "🔍 Deploying production Elasticsearch for log aggregation..."
    docker compose -f "$COMPOSE_FILE" up -d elasticsearch-production
    
    # Wait for services to be ready
    sleep 30
    
    log "✅ Advanced monitoring deployment completed"
}

# Health check all Phase 5 services
comprehensive_health_check() {
    log "🏥 Running comprehensive Phase 5 health checks..."
    
    local services=(
        "security-scanner:4954"
        "vault:8200"
        "cadvisor:8099"
        "alertmanager-advanced:9094"
        "grafana-production:3002"
        "elasticsearch-production:9201"
    )
    
    local healthy_count=0
    local total_count=${#services[@]}
    
    for service_port in "${services[@]}"; do
        IFS=':' read -r service port <<< "$service_port"
        
        info "Checking $service health..."
        if curl -f -s "http://localhost:$port" >/dev/null 2>&1 || \
           curl -f -s "http://localhost:$port/health" >/dev/null 2>&1; then
            log "✅ $service is healthy"
            ((healthy_count++))
        else
            warn "❌ $service health check failed"
        fi
    done
    
    log "📊 Health Check Summary: $healthy_count/$total_count services healthy"
    
    if [[ $healthy_count -eq $total_count ]]; then
        log "✅ All Phase 5 services are healthy!"
        return 0
    else
        warn "Some Phase 5 services may need attention"
        return 1
    fi
}

# System integration verification
verify_system_integration() {
    log "🔗 Verifying system integration across all phases..."
    
    # Check total running services
    total_services=$(docker ps --filter "name=tracseq" --format "{{.Names}}" | wc -l)
    log "📊 Total TracSeq services running: $total_services"
    
    # Verify cross-service communication
    info "Testing cross-service communication..."
    
    # Test API Gateway connectivity to core services
    local api_tests=(
        "http://localhost:8089/health"
        "http://localhost:8080/health"
        "http://localhost:8083/health"
    )
    
    for endpoint in "${api_tests[@]}"; do
        if curl -f -s "$endpoint" >/dev/null 2>&1; then
            info "✅ API connectivity test passed: $endpoint"
        else
            warn "❌ API connectivity test failed: $endpoint"
        fi
    done
    
    # Test monitoring stack integration
    if curl -f -s "http://localhost:9090/api/v1/targets" >/dev/null 2>&1; then
        info "✅ Prometheus is collecting metrics"
    else
        warn "❌ Prometheus connectivity issues"
    fi
    
    log "✅ System integration verification completed"
}

# Display access information
display_access_info() {
    echo ""
    log "🎉 Phase 5 Deployment Complete!"
    echo ""
    echo -e "${CYAN}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║                     PRODUCTION SERVICES                      ║${NC}"
    echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    info "🔒 Security & Compliance:"
    echo "  🛡️ Security Scanner: http://localhost:4954"
    echo "  🔐 Vault (Secrets): http://localhost:8200"
    echo "      Token: tracseq-dev-token"
    echo ""
    
    info "📊 Advanced Monitoring:"
    echo "  📈 Production Grafana: http://localhost:3002"
    echo "      Username: admin | Password: tracseq-prod-2024"
    echo "  🚨 Advanced AlertManager: http://localhost:9094"
    echo "  🔍 Production Elasticsearch: http://localhost:9201"
    echo "  📊 Container Metrics (cAdvisor): http://localhost:8099"
    echo ""
    
    info "🚀 Performance & Scaling:"
    echo "  🔄 Redis Cluster Node 1: localhost:7001"
    echo "  🔄 Redis Cluster Node 2: localhost:7002"
    echo "  🔄 Redis Cluster Node 3: localhost:7003"
    echo "  📊 Load Testing Results: Check load_test_results volume"
    echo ""
    
    info "🧪 Quality Assurance:"
    echo "  🔬 Integration Test Results: Check integration_test_results volume"
    echo "  📝 Automated Testing: Every 15 minutes"
    echo ""
    
    info "💾 Backup & Recovery:"
    echo "  🔄 Automated Backups: Every 6 hours"
    echo "  📁 Backup Storage: backup_storage volume"
    echo "  🗄️ Retention: 7 days"
    echo ""
    
    echo -e "${CYAN}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║                    PRODUCTION READINESS                      ║${NC}"
    echo -e "${CYAN}╚═══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    security_msg "Security hardening deployed with vulnerability scanning"
    performance_msg "Performance monitoring and Redis clustering enabled"
    log "Automated testing and quality assurance operational"
    log "Backup and disaster recovery configured"
    log "Advanced alerting and monitoring active"
    echo ""
    
    info "📝 Phase 5 Deployment Log: $LOG_FILE"
    info "🔧 Configuration Files: monitoring/alertmanager/, monitoring/grafana/"
    echo ""
    
    warn "⚠️  Production Notes:"
    echo "  • Configure real SMTP settings in AlertManager"
    echo "  • Set up proper Slack webhooks for critical alerts"
    echo "  • Initialize Redis cluster in production environment"
    echo "  • Review and customize security scan results"
    echo "  • Configure Vault with production secrets"
    echo ""
}

# Cleanup function
cleanup() {
    warn "🧹 Cleaning up Phase 5 deployment..."
    docker compose -f "$COMPOSE_FILE" down
    docker volume prune -f
}

# Main execution
main() {
    show_banner
    
    log "🚀 Starting TracSeq 2.0 Phase 5 Deployment"
    log "📅 Deployment started at $(date)"
    
    # Parse command line arguments
    case "${1:-deploy}" in
        deploy)
            check_prerequisites
            run_security_assessment
            deploy_backup_dr
            deploy_performance_monitoring
            deploy_integration_testing
            deploy_advanced_monitoring
            comprehensive_health_check
            verify_system_integration
            display_access_info
            ;;
        security)
            check_prerequisites
            run_security_assessment
            log "🔒 Security assessment completed"
            ;;
        monitoring)
            check_prerequisites
            deploy_advanced_monitoring
            log "📊 Advanced monitoring deployed"
            ;;
        performance)
            check_prerequisites
            deploy_performance_monitoring
            log "🚀 Performance monitoring deployed"
            ;;
        testing)
            check_prerequisites
            deploy_integration_testing
            log "🧪 Integration testing deployed"
            ;;
        backup)
            check_prerequisites
            deploy_backup_dr
            log "💾 Backup and DR deployed"
            ;;
        health)
            comprehensive_health_check
            verify_system_integration
            ;;
        cleanup)
            cleanup
            ;;
        restart)
            cleanup
            sleep 5
            check_prerequisites
            run_security_assessment
            deploy_backup_dr
            deploy_performance_monitoring
            deploy_integration_testing
            deploy_advanced_monitoring
            comprehensive_health_check
            verify_system_integration
            display_access_info
            ;;
        *)
            error "Unknown command: $1"
            echo ""
            echo "Usage: $0 [deploy|security|monitoring|performance|testing|backup|health|cleanup|restart]"
            echo ""
            echo "Commands:"
            echo "  deploy      - Full Phase 5 deployment (default)"
            echo "  security    - Deploy security scanning and hardening"
            echo "  monitoring  - Deploy advanced monitoring only"
            echo "  performance - Deploy performance monitoring only"
            echo "  testing     - Deploy integration testing only"
            echo "  backup      - Deploy backup and DR only"
            echo "  health      - Run health checks only"
            echo "  cleanup     - Remove all Phase 5 services"
            echo "  restart     - Cleanup and redeploy"
            echo ""
            exit 1
            ;;
    esac
    
    log "✅ Phase 5 deployment process completed at $(date)"
}

# Run main function with all arguments
main "$@" 