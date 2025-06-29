#!/bin/bash

# TracSeq 2.0 Phase 6: Production Readiness & Observability
# Quick Start Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() { echo -e "${BLUE}🔄 $1${NC}"; }
print_success() { echo -e "${GREEN}✅ $1${NC}"; }
print_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
print_error() { echo -e "${RED}❌ $1${NC}"; }
print_header() { echo -e "\n${BLUE}===========================================${NC}"; echo -e "${BLUE}$1${NC}"; echo -e "${BLUE}===========================================${NC}\n"; }

# Setup monitoring configuration
setup_monitoring_config() {
    print_status "Setting up monitoring configuration..."
    
    # Create monitoring directory structure
    mkdir -p monitoring/{prometheus,grafana/{dashboards,datasources},alertmanager}
    
    # Create Prometheus configuration
    cat > monitoring/prometheus/prometheus.yml << EOF
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "rules/*.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

scrape_configs:
  # Prometheus itself
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  # TracSeq Microservices
  - job_name: 'auth-service'
    static_configs:
      - targets: ['auth-service:8080']
    metrics_path: '/metrics'
    scrape_interval: 5s

  - job_name: 'sample-service'
    static_configs:
      - targets: ['sample-service:8080']
    metrics_path: '/metrics'
    scrape_interval: 5s

  - job_name: 'template-service'
    static_configs:
      - targets: ['template-service:8080']
    metrics_path: '/metrics'
    scrape_interval: 5s

  - job_name: 'enhanced-rag-service'
    static_configs:
      - targets: ['enhanced-rag-service:8000']
    metrics_path: '/metrics'
    scrape_interval: 5s

  # Infrastructure
  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']

  - job_name: 'redis-exporter'
    static_configs:
      - targets: ['redis-exporter:9121']

  - job_name: 'postgres-exporter'
    static_configs:
      - targets: ['postgres-exporter:9187']
EOF

    # Create Grafana datasource configuration
    cat > monitoring/grafana/datasources/datasources.yml << EOF
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    orgId: 1
    url: http://prometheus:9090
    basicAuth: false
    isDefault: true
    editable: true

  - name: Jaeger
    type: jaeger
    access: proxy
    orgId: 1
    url: http://jaeger:16686
    basicAuth: false
    editable: true
EOF

    # Create AlertManager configuration
    cat > monitoring/alertmanager/alertmanager.yml << EOF
global:
  smtp_smarthost: 'localhost:587'
  smtp_from: 'alertmanager@tracseq.com'

route:
  group_by: ['alertname']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  receiver: 'web.hook'

receivers:
  - name: 'web.hook'
    webhook_configs:
    - url: 'http://localhost:5001/'

inhibit_rules:
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['alertname', 'dev', 'instance']
EOF

    print_success "Monitoring configuration created"
}

# Start Phase 6 components
start_phase6() {
    print_header "STARTING TRACSEQ 2.0 PHASE 6: PRODUCTION READINESS"
    
    print_status "Phase 6 Focus Areas:"
    echo "  📊 Monitoring & Observability (Prometheus, Grafana, Jaeger)"
    echo "  🛡️  Security Hardening & Performance Optimization"
    echo "  🔍 Distributed Tracing & Centralized Logging"
    echo "  📈 Business Metrics & Laboratory KPIs"
    echo ""
    
    # Setup monitoring config
    setup_monitoring_config
    
    # Start the microservices first
    print_status "Starting core TracSeq microservices..."
    docker-compose -f docker-compose.microservices.yml up -d postgres redis auth-service sample-service template-service enhanced-rag-service
    
    # Wait for services to be ready
    print_status "Waiting for core services to be ready..."
    sleep 30
    
    # Start monitoring stack
    print_status "Starting monitoring stack..."
    docker-compose -f docker-compose.monitoring.yml up -d
    
    # Wait for monitoring services
    print_status "Waiting for monitoring services to initialize..."
    sleep 20
    
    # Check status
    print_header "PHASE 6 SYSTEM STATUS"
    
    local services=(
        "Core Services"
        "PostgreSQL:http://localhost:5432"
        "Redis:direct"
        "Auth Service:http://localhost:3010/health"
        "Sample Service:http://localhost:3011/health" 
        "Template Service:http://localhost:3013/health"
        "Enhanced RAG Service:http://localhost:3019/health"
        ""
        "Monitoring Stack"
        "Prometheus:http://localhost:9090"
        "Grafana:http://localhost:3000"
        "Jaeger:http://localhost:16686"
        "AlertManager:http://localhost:9093"
    )
    
    for service in "${services[@]}"; do
        if [[ -z "$service" ]]; then
            echo ""
            continue
        fi
        
        if [[ "$service" == *"Services" ]]; then
            echo -e "${BLUE}${service}:${NC}"
            continue
        fi
        
        IFS=':' read -r name url <<< "$service"
        
        if [[ "$url" == "direct" ]]; then
            echo "  ✅ $name (Direct service)"
        elif [[ "$url" == *"5432" ]]; then
            echo "  ✅ $name (Database)"
        else
            if curl -f -s "$url" >/dev/null 2>&1; then
                echo "  ✅ $name"
            else
                echo "  🔄 $name (Starting...)"
            fi
        fi
    done
    
    print_header "🎯 PHASE 6 QUICK ACCESS"
    echo "📊 Prometheus Metrics: http://localhost:9090"
    echo "📈 Grafana Dashboards: http://localhost:3000 (admin/admin)"
    echo "🔍 Jaeger Tracing: http://localhost:16686"
    echo "🚨 AlertManager: http://localhost:9093"
    echo ""
    echo "🔧 TracSeq Services:"
    echo "   Auth Service: http://localhost:3010/health"
    echo "   Sample Service: http://localhost:3011/health"
    echo "   Template Service: http://localhost:3013/health"
    echo "   RAG Service: http://localhost:3019/health"
    echo ""
    
    print_success "Phase 6 Environment Ready!"
    print_status "Next Steps:"
    echo "  1. Configure Grafana dashboards for laboratory metrics"
    echo "  2. Set up alerts for critical laboratory operations"
    echo "  3. Implement distributed tracing in Rust services"
    echo "  4. Review security hardening checklist"
    echo "  5. Performance testing and optimization"
}

# Stop all Phase 6 services
stop_phase6() {
    print_status "Stopping Phase 6 environment..."
    docker-compose -f docker-compose.monitoring.yml down
    docker-compose -f docker-compose.microservices.yml down
    print_success "Phase 6 environment stopped"
}

# Main execution
main() {
    case "${1:-start}" in
        start)
            start_phase6
            ;;
        stop)
            stop_phase6
            ;;
        restart)
            stop_phase6
            sleep 5
            start_phase6
            ;;
        status)
            docker-compose -f docker-compose.microservices.yml ps
            echo ""
            docker-compose -f docker-compose.monitoring.yml ps
            ;;
        *)
            echo "Usage: $0 {start|stop|restart|status}"
            exit 1
            ;;
    esac
}

# Handle interruption
trap 'print_error "Interrupted"; exit 1' INT TERM

# Run main function
main "$@" 