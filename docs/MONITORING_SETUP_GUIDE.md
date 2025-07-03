# Lab Manager Monitoring Setup Guide

This guide provides instructions for setting up and configuring Prometheus and Grafana monitoring for the Lab Manager system.

## Overview

The monitoring stack includes:
- **Prometheus**: Time-series database for metrics collection
- **Grafana**: Visualization and dashboard platform
- **Node Exporter**: System metrics exporter
- **cAdvisor**: Container metrics exporter
- **Alertmanager**: Alert handling and routing

## Prerequisites

- Docker and Docker Compose installed
- Lab Manager services running
- Basic understanding of monitoring concepts

## Quick Start

1. **Start the monitoring stack:**
   ```bash
   ./scripts/start-monitoring.sh
   ```

2. **Access the services:**
   - Prometheus: http://localhost:9090
   - Grafana: http://localhost:3002 (default: admin/admin)
   - Alertmanager: http://localhost:9093

## Configuration Files

### Directory Structure
```
monitoring/
├── prometheus/
│   ├── prometheus.yml          # Main Prometheus configuration
│   └── rules/
│       └── lab-manager-alerts.yml  # Alert rules
├── grafana/
│   ├── provisioning/
│   │   ├── datasources/
│   │   │   └── prometheus.yml  # Datasource configuration
│   │   └── dashboards/
│   │       └── default.yml     # Dashboard provisioning
│   └── dashboards/
│       └── lab-manager-overview.json  # Pre-built dashboard
└── alertmanager/
    └── alertmanager.yml        # Alert routing configuration
```

## Adding Metrics to Your Application

### Rust (Lab Manager)

1. **Add dependencies to Cargo.toml:**
   ```toml
   [dependencies]
   metrics = "0.21"
   metrics-exporter-prometheus = "0.13"
   ```

2. **Initialize metrics in your application:**
   ```rust
   use metrics_exporter_prometheus::PrometheusBuilder;
   
   // In your main function
   let builder = PrometheusBuilder::new();
   builder.install().expect("Failed to install Prometheus recorder");
   ```

3. **Add metrics endpoint:**
   ```rust
   // In your router configuration
   .route("/metrics", get(metrics_handler))
   
   async fn metrics_handler() -> impl IntoResponse {
       let encoder = TextEncoder::new();
       let metric_families = prometheus::gather();
       let mut buffer = vec![];
       encoder.encode(&metric_families, &mut buffer).unwrap();
       Response::builder()
           .header(CONTENT_TYPE, encoder.format_type())
           .body(buffer)
           .unwrap()
   }
   ```

4. **Record metrics:**
   ```rust
   use metrics::{counter, gauge, histogram};
   
   // Increment a counter
   counter!("http_requests_total", 1, "method" => "GET", "endpoint" => "/api/samples");
   
   // Set a gauge
   gauge!("active_connections", 42.0);
   
   // Record a histogram
   histogram!("http_request_duration_seconds", elapsed.as_secs_f64());
   ```

### Python (RAG Service)

1. **Install prometheus-client:**
   ```bash
   pip install prometheus-client
   ```

2. **Add metrics to your FastAPI app:**
   ```python
   from prometheus_client import Counter, Histogram, Gauge, generate_latest
   from fastapi import Response
   
   # Define metrics
   http_requests_total = Counter('http_requests_total', 'Total HTTP requests', ['method', 'endpoint'])
   request_duration = Histogram('http_request_duration_seconds', 'HTTP request latency')
   active_connections = Gauge('active_connections', 'Number of active connections')
   
   # Add metrics endpoint
   @app.get("/metrics")
   async def metrics():
       return Response(content=generate_latest(), media_type="text/plain")
   
   # Use metrics in your code
   @app.get("/api/process")
   @request_duration.time()  # Automatic timing
   async def process():
       http_requests_total.labels(method='GET', endpoint='/api/process').inc()
       # Your logic here
   ```

## Configuring Alerts

### Adding New Alert Rules

1. **Create a new rule file in `monitoring/prometheus/rules/`:**
   ```yaml
   groups:
     - name: custom_alerts
       interval: 30s
       rules:
         - alert: HighSampleProcessingTime
           expr: histogram_quantile(0.95, rate(sample_processing_duration_seconds_bucket[5m])) > 30
           for: 10m
           labels:
             severity: warning
           annotations:
             summary: "Sample processing taking too long"
             description: "95th percentile processing time is above 30 seconds"
   ```

2. **Reload Prometheus configuration:**
   ```bash
   curl -X POST http://localhost:9090/-/reload
   ```

### Configuring Alert Notifications

Edit `monitoring/alertmanager/alertmanager.yml` to add notification channels:

#### Slack Integration
```yaml
receivers:
  - name: 'slack-notifications'
    slack_configs:
      - api_url: 'YOUR_SLACK_WEBHOOK_URL'
        channel: '#alerts'
        title: 'Lab Manager Alert'
        text: '{{ range .Alerts }}{{ .Annotations.summary }}{{ end }}'
```

#### Email Integration
```yaml
receivers:
  - name: 'email-notifications'
    email_configs:
      - to: 'admin@example.com'
        from: 'alerts@labmanager.com'
        smarthost: 'smtp.gmail.com:587'
        auth_username: 'alerts@labmanager.com'
        auth_password: 'your-app-password'
```

## Creating Custom Dashboards

### Using Grafana UI

1. **Login to Grafana** at http://localhost:3002
2. **Create a new dashboard**: Click "+" → "Dashboard"
3. **Add panels** with queries like:
   - CPU Usage: `100 - (avg(irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)`
   - Memory Usage: `(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100`
   - Request Rate: `rate(http_requests_total[5m])`
   - Error Rate: `rate(http_requests_total{status=~"5.."}[5m])`

### Importing Pre-built Dashboards

1. **Go to Dashboards** → **Import**
2. **Upload JSON file** or paste dashboard JSON
3. **Select Prometheus datasource**
4. **Click Import**

Popular dashboard IDs from grafana.com:
- Node Exporter Full: 1860
- Docker Container: 193
- PostgreSQL: 9628

## Best Practices

### Metric Naming
- Use lowercase with underscores: `http_requests_total`
- Include unit in name: `_seconds`, `_bytes`, `_total`
- Be consistent across services

### Label Usage
- Keep cardinality low (avoid unique IDs)
- Use standard labels: `method`, `status`, `endpoint`
- Document label meanings

### Dashboard Design
- Group related metrics
- Use appropriate visualization types
- Set meaningful thresholds
- Include documentation panels

### Alert Configuration
- Set appropriate thresholds based on baselines
- Use multiple severity levels
- Include runbooks in annotations
- Test alerts before production

## Troubleshooting

### Prometheus Not Scraping Targets

1. **Check target status** at http://localhost:9090/targets
2. **Verify network connectivity:**
   ```bash
   docker exec prometheus wget -O- http://app:3000/metrics
   ```
3. **Check logs:**
   ```bash
   docker logs prometheus
   ```

### Grafana Connection Issues

1. **Verify datasource configuration**
2. **Test connection** in Grafana datasource settings
3. **Check network between containers:**
   ```bash
   docker network ls
   docker network inspect lab_network
   ```

### Missing Metrics

1. **Verify metric export** from application
2. **Check Prometheus scrape interval**
3. **Use Prometheus query browser** to explore available metrics

## Maintenance

### Backup Grafana Dashboards
```bash
# Export all dashboards
curl -s "http://admin:admin@localhost:3002/api/search" | \
  jq -r '.[] | .uid' | \
  xargs -I {} curl -s "http://admin:admin@localhost:3002/api/dashboards/uid/{}" | \
  jq -r '.dashboard' > dashboards_backup.json
```

### Clean Up Old Data
```bash
# Prometheus data retention (default: 15d)
# Set in docker-compose.monitoring.yml:
command:
  - '--storage.tsdb.retention.time=30d'
  - '--storage.tsdb.retention.size=10GB'
```

### Update Monitoring Stack
```bash
# Pull latest images
docker-compose -f docker-compose.monitoring.yml pull

# Restart services
docker-compose -f docker-compose.monitoring.yml up -d
```

## Security Considerations

1. **Change default passwords** for Grafana
2. **Use HTTPS** for production deployments
3. **Restrict metrics endpoints** with authentication
4. **Implement RBAC** in Grafana
5. **Secure Alertmanager webhooks**

## Additional Resources

- [Prometheus Documentation](https://prometheus.io/docs/)
- [Grafana Documentation](https://grafana.com/docs/)
- [PromQL Tutorial](https://prometheus.io/docs/prometheus/latest/querying/basics/)
- [Grafana Dashboard Best Practices](https://grafana.com/docs/grafana/latest/best-practices/)