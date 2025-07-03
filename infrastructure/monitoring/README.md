# TracSeq 2.0 Monitoring Stack

This directory contains a comprehensive monitoring and observability stack for the TracSeq 2.0 laboratory management system.

## Overview

The monitoring stack provides:
- **Metrics Collection**: Prometheus for time-series metrics
- **Visualization**: Grafana for dashboards and analytics
- **Log Aggregation**: Loki for centralized logging
- **Distributed Tracing**: Jaeger for request tracing
- **Alerting**: AlertManager for alert routing and management
- **Uptime Monitoring**: Uptime Kuma for service availability
- **Long-term Storage**: Mimir for metrics retention

## Quick Start

### 1. Start the Monitoring Stack

```bash
# Start the enhanced monitoring stack
docker-compose -f docker-compose.monitoring-enhanced.yml up -d

# Or use the basic stack
docker-compose -f docker-compose.monitoring.yml up -d
```

### 2. Access the Services

- **Grafana**: http://localhost:3001 (admin/tracseq-admin)
- **Prometheus**: http://localhost:9090
- **AlertManager**: http://localhost:9093
- **Jaeger**: http://localhost:16686
- **Loki**: http://localhost:3100
- **Uptime Kuma**: http://localhost:3002

## Components

### Prometheus
- Collects metrics from all microservices
- Configured with service discovery for dynamic environments
- Recording rules for performance optimization
- Comprehensive alerting rules

### Grafana
- Pre-configured dashboards:
  - Service Overview
  - Infrastructure Metrics
  - Laboratory Operations
  - AI Services Performance
  - Business KPIs
- Multiple data sources (Prometheus, Loki, Jaeger, PostgreSQL)
- Unified alerting system

### Loki & Promtail
- Centralized log aggregation
- Automatic log parsing for different service types
- Integration with distributed tracing
- Log-based alerting capabilities

### Jaeger
- End-to-end distributed tracing
- Service dependency analysis
- Performance bottleneck identification
- Integration with logs and metrics

### Exporters
- **Node Exporter**: System-level metrics
- **cAdvisor**: Container metrics
- **PostgreSQL Exporter**: Database metrics
- **Redis Exporter**: Cache metrics
- **Blackbox Exporter**: Endpoint monitoring
- **Pushgateway**: Batch job metrics

## Configuration

### Adding New Services to Monitoring

1. **Metrics Collection**:
   Add to `prometheus/prometheus-enhanced.yml`:
   ```yaml
   - job_name: 'new-service'
     static_configs:
       - targets: ['new-service:port']
         labels:
           service: 'new-service'
           tier: 'backend'
   ```

2. **Log Collection**:
   Services are automatically discovered if running in Docker.

3. **Tracing**:
   Configure your service to send traces to:
   - Jaeger: `jaeger:14268` (HTTP) or `jaeger:14250` (gRPC)
   - OTLP: `jaeger:4317` (gRPC) or `jaeger:4318` (HTTP)

### Custom Dashboards

Dashboards are organized by category:
- `/grafana/dashboards/tracseq/`: Core service dashboards
- `/grafana/dashboards/infrastructure/`: Infrastructure monitoring
- `/grafana/dashboards/laboratory/`: Lab-specific metrics
- `/grafana/dashboards/ai-services/`: AI/ML service monitoring
- `/grafana/dashboards/business/`: Business metrics

### Alert Configuration

1. **Alert Rules**: Located in `/prometheus/alerts/`
2. **Recording Rules**: Located in `/prometheus/rules/`
3. **AlertManager Config**: `/alertmanager/alertmanager.yml`

## Monitoring Guidelines

### What to Monitor

1. **Golden Signals**:
   - Latency (response times)
   - Traffic (request rates)
   - Errors (error rates)
   - Saturation (resource utilization)

2. **Business Metrics**:
   - Sample processing rates
   - Storage utilization
   - Sequencing success rates
   - User activity

3. **Infrastructure**:
   - CPU, Memory, Disk, Network
   - Container health
   - Database performance
   - Message queue depth

### Best Practices

1. **Naming Conventions**:
   - Metrics: `service_component_measurement_unit`
   - Labels: Use consistent labeling across services
   - Dashboards: Descriptive names with service prefixes

2. **Retention Policies**:
   - Raw metrics: 15 days (Prometheus)
   - Aggregated metrics: 1 year (Mimir)
   - Logs: 7 days (Loki)
   - Traces: 7 days (Jaeger)

3. **Performance**:
   - Use recording rules for expensive queries
   - Limit cardinality of metrics
   - Configure appropriate scrape intervals

## Troubleshooting

### Common Issues

1. **High Memory Usage**:
   ```bash
   # Check Prometheus memory usage
   curl -s http://localhost:9090/api/v1/query?query=prometheus_tsdb_symbol_table_size_bytes
   
   # Compact TSDB
   curl -X POST http://localhost:9090/api/v1/admin/tsdb/clean_tombstones
   ```

2. **Missing Metrics**:
   ```bash
   # Check scrape targets
   curl http://localhost:9090/api/v1/targets
   
   # Verify service discovery
   curl http://localhost:9090/api/v1/service-discovery
   ```

3. **Dashboard Not Loading**:
   - Check data source configuration
   - Verify time range selection
   - Review Grafana logs: `docker logs tracseq-grafana`

### Useful Queries

1. **Service Availability**:
   ```promql
   up{job=~".*-service|api-gateway"}
   ```

2. **Error Rate by Service**:
   ```promql
   sum by (service) (rate(http_requests_total{status=~"5.."}[5m])) / 
   sum by (service) (rate(http_requests_total[5m]))
   ```

3. **Resource Usage**:
   ```promql
   # CPU Usage
   sum by (service) (rate(container_cpu_usage_seconds_total[5m])) * 100
   
   # Memory Usage
   sum by (service) (container_memory_working_set_bytes)
   ```

## Maintenance

### Backup

```bash
# Backup Prometheus data
docker exec tracseq-prometheus tar -czf - /prometheus > prometheus-backup.tar.gz

# Backup Grafana dashboards
docker exec tracseq-grafana tar -czf - /var/lib/grafana > grafana-backup.tar.gz
```

### Updates

1. Update image versions in docker-compose files
2. Test in staging environment
3. Plan maintenance window
4. Perform rolling updates

### Monitoring the Monitoring Stack

- Prometheus self-monitoring: http://localhost:9090/metrics
- Grafana metrics: http://localhost:3001/metrics
- Use Uptime Kuma to monitor the monitoring services themselves

## Integration with CI/CD

### Deployment Metrics

Push deployment events to Prometheus:
```bash
echo "deployment_info{service=\"$SERVICE\",version=\"$VERSION\",environment=\"production\"} $(date +%s)" | \
  curl --data-binary @- http://localhost:9091/metrics/job/deployments
```

### Performance Testing

Export test results to Prometheus:
```bash
cat performance-results.txt | \
  curl --data-binary @- http://localhost:9091/metrics/job/performance-tests
```

## Security Considerations

1. **Authentication**:
   - Change default passwords
   - Enable authentication for all services
   - Use TLS for external access

2. **Network Security**:
   - Restrict access to monitoring endpoints
   - Use internal networks for service communication
   - Enable firewall rules

3. **Data Protection**:
   - Encrypt sensitive metrics
   - Implement retention policies
   - Regular security audits

## Support

For issues or questions:
1. Check service logs: `docker-compose logs <service-name>`
2. Review documentation in service directories
3. Contact the platform team

*Context improved by Giga AI*