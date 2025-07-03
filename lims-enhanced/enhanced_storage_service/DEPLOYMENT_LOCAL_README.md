# Enhanced Storage Service - Local Deployment Guide

## 🚀 Quick Start

Deploy the complete Enhanced Storage Service locally with one command:

**Linux/macOS:**
```bash
./deploy-local.sh
```

**Windows:**
```powershell
.\deploy-local.ps1
```

## 📋 Prerequisites

### Required Software
- **Docker Desktop** (v20.10+) with Docker Compose
- **8GB+ RAM** (12GB+ recommended for full stack)
- **20GB+ free disk space**
- **Internet connection** for downloading images

### System Requirements
- **CPU**: 4+ cores recommended
- **RAM**: 8GB minimum, 12GB recommended
- **Storage**: 20GB available space
- **Network**: Ports 80, 3000, 5432, 6379, 8082, 8090-8091, 9000-9001, 9090, 11434, 16686

## 🏗️ Architecture Overview

The Enhanced Storage Service consists of three main phases:

### Phase 1: Core Platform (91 endpoints)
- **Storage Management**: Location and sample tracking
- **IoT Integration**: Sensor networks and monitoring
- **Analytics Engine**: Business intelligence and reporting
- **Automation Platform**: Robotic sample handling
- **Blockchain Security**: Chain of custody and audit trails
- **Digital Twin**: Virtual facility simulation
- **Energy Management**: Power optimization
- **Mobile APIs**: Mobile application support

### Phase 2: AI/ML Platform (9 endpoints)
- **Predictive Maintenance**: Equipment failure prediction
- **Intelligent Routing**: AI-powered sample placement
- **Anomaly Detection**: Real-time system monitoring
- **AI Model Management**: Training pipelines and lifecycle

### Phase 3: Enterprise Integration (9 endpoints)
- **LIMS Integration**: Laboratory Information Management System
- **ERP Integration**: Enterprise Resource Planning
- **Multi-Cloud Platform**: AWS, Azure, GCP connectivity
- **Data Orchestration**: Real-time synchronization

## 🐳 Services Included

| Service | Purpose | Port | Credentials |
|---------|---------|------|-------------|
| **Enhanced Storage Service** | Main API | 8082 | - |
| **PostgreSQL** | Primary Database | 5432 | postgres/postgres |
| **Redis** | Caching & Sessions | 6379 | Password: redis_password |
| **Grafana** | Monitoring Dashboard | 3000 | admin/admin |
| **Prometheus** | Metrics Collection | 9090 | - |
| **Jaeger** | Distributed Tracing | 16686 | - |
| **MinIO** | S3-Compatible Storage | 9000/9001 | minioadmin/minioadmin |
| **Mosquitto** | MQTT Broker (IoT) | 1883/9001 | - |
| **LocalStack** | AWS Services Mock | 4566 | - |
| **Mock LIMS** | LIMS Service Mock | 8090 | - |
| **Mock ERP** | ERP Service Mock | 8091 | - |
| **Ganache** | Blockchain Test Network | 8545 | - |
| **Nginx** | Load Balancer | 80/443 | - |

## 📁 Directory Structure

```
enhanced_storage_service/
├── docker-compose.local.yml     # Main compose file
├── local.env                    # Environment variables
├── deploy-local.sh             # Linux/macOS deployment script
├── deploy-local.ps1            # Windows deployment script
├── Dockerfile                  # Service container definition
├── config/                     # Service configurations
│   ├── mosquitto/
│   ├── prometheus/
│   ├── grafana/
│   └── nginx/
├── mocks/                      # Mock service configurations
│   ├── lims/
│   └── erp/
├── scripts/
│   └── init-db.sql            # Database initialization
└── src/                       # Rust source code
    ├── handlers/              # API endpoint handlers
    ├── ai/                    # AI/ML modules
    ├── integrations/          # Enterprise integrations
    └── ...
```

## 🚀 Deployment Commands

### Full Deployment
```bash
# Linux/macOS
./deploy-local.sh deploy

# Windows
.\deploy-local.ps1 deploy
```

### Service Management
```bash
# Start services
./deploy-local.sh start

# Stop services
./deploy-local.sh stop

# Restart services
./deploy-local.sh restart

# View logs
./deploy-local.sh logs

# Clean up everything
./deploy-local.sh clean

# Test deployment
./deploy-local.sh test
```

## 🔗 Access Points

Once deployed, access your services at:

### Main Application
- **API Gateway**: http://localhost:8082
- **Frontend Application**: http://localhost:80
- **Health Check**: http://localhost:8082/health

### Core Platform APIs (Phase 1)
- **Storage Overview**: http://localhost:8082/storage/overview
- **IoT Sensors**: http://localhost:8082/iot/sensors
- **Analytics Dashboard**: http://localhost:8082/analytics/overview
- **Automation Status**: http://localhost:8082/automation/status
- **Blockchain Audit**: http://localhost:8082/blockchain/audit-trail
- **Digital Twin**: http://localhost:8082/digital-twin/overview
- **Energy Management**: http://localhost:8082/energy/overview
- **Mobile APIs**: http://localhost:8082/mobile/overview

### AI/ML Platform APIs (Phase 2)
- **AI Platform**: http://localhost:8082/ai/overview
- **Predictive Maintenance**: http://localhost:8082/ai/predict/equipment-failure
- **Intelligent Routing**: http://localhost:8082/ai/optimize/sample-routing
- **Anomaly Detection**: http://localhost:8082/ai/detect/anomalies

### Enterprise Integration APIs (Phase 3)
- **Integration Hub**: http://localhost:8082/integrations/overview
- **LIMS Sync**: http://localhost:8082/integrations/lims/samples/sync
- **ERP Integration**: http://localhost:8082/integrations/erp/purchase-requisitions
- **Cloud Storage**: http://localhost:8082/integrations/cloud/upload

### Monitoring & Tools
- **Grafana Dashboards**: http://localhost:3000 (admin/admin)
- **Prometheus Metrics**: http://localhost:9090
- **Jaeger Tracing**: http://localhost:16686
- **MinIO Console**: http://localhost:9001 (minioadmin/minioadmin)

### Mock Services
- **Mock LIMS**: http://localhost:8090
- **Mock ERP**: http://localhost:8091

## 🧪 Testing the Deployment

### Health Checks
```bash
# Service health
curl http://localhost:8082/health

# Database connectivity
curl http://localhost:8082/storage/overview

# AI platform status
curl http://localhost:8082/ai/overview

# Integration status
curl http://localhost:8082/integrations/overview
```

### Sample API Calls
```bash
# Get storage locations
curl http://localhost:8082/storage/locations

# Get IoT sensor data
curl http://localhost:8082/iot/sensors

# AI equipment prediction
curl -X POST http://localhost:8082/ai/predict/equipment-failure \
  -H "Content-Type: application/json" \
  -d '{"equipment_id": "FREEZER_001", "metrics": {"temperature": -75.5}}'

# LIMS synchronization
curl -X POST http://localhost:8082/integrations/lims/samples/sync \
  -H "Content-Type: application/json" \
  -d '{"sample_ids": ["SAMPLE-001"]}'
```

## 🛠️ Configuration

### Environment Variables
Key configuration options in `local.env`:

```bash
# Database
DATABASE_URL=postgres://postgres:postgres@localhost:5432/enhanced_storage

# AI/ML Configuration
AI_CONFIDENCE_THRESHOLD=0.85
AI_ENABLE_REAL_TIME_TRAINING=true

# Integration URLs
LIMS_BASE_URL=http://localhost:8090
ERP_BASE_URL=http://localhost:8091

# Security
JWT_SECRET=enhanced-storage-local-jwt-secret-key-2024
ENCRYPTION_KEY=enhanced-storage-encryption-key-32bit

# IoT Settings
MQTT_BROKER_URL=mqtt://localhost:1883
IOT_SENSOR_POLLING_INTERVAL_SECONDS=30

# Blockchain
BLOCKCHAIN_NODE_URL=http://localhost:8545

# Energy Management
ENERGY_MONITORING_ENABLED=true
CARBON_TRACKING_ENABLED=true
```

### Customization
1. **Modify ports**: Edit `docker-compose.local.yml`
2. **Add services**: Extend the compose file
3. **Change configurations**: Update files in `config/`
4. **Mock responses**: Modify files in `mocks/`

## 🔧 Troubleshooting

### Common Issues

#### Port Conflicts
```bash
# Check port usage
netstat -tulpn | grep :8082

# Change ports in docker-compose.local.yml
```

#### Memory Issues
```bash
# Check memory usage
docker stats

# Increase Docker Desktop memory allocation
```

#### Service Won't Start
```bash
# Check service logs
docker logs enhanced-storage-service_enhanced-storage-service_1

# Restart specific service
docker-compose -f docker-compose.local.yml restart enhanced-storage-service
```

#### Database Connection Issues
```bash
# Check database logs
docker logs enhanced-storage-service_postgres_1

# Reset database
docker-compose -f docker-compose.local.yml down -v
```

### Reset Everything
```bash
# Complete reset
./deploy-local.sh clean
./deploy-local.sh deploy
```

## 📊 Performance Metrics

Expected performance on recommended hardware:

- **API Response Time**: < 100ms (average)
- **Database Query Time**: < 50ms (average)
- **AI Inference Time**: < 1s (typical)
- **Memory Usage**: ~4-6GB (full stack)
- **CPU Usage**: 10-30% (idle), 50-80% (heavy load)

## 🔒 Security Notes

### Local Development Security
- Default passwords are used (change for production)
- SSL/TLS disabled (enable for production)
- All services exposed (restrict for production)
- No authentication on monitoring tools

### Production Considerations
- Change all default passwords
- Enable SSL/TLS certificates
- Implement proper authentication
- Restrict network access
- Enable audit logging
- Regular security updates

## 📚 API Documentation

Once deployed, interactive API documentation is available at:
- **OpenAPI/Swagger**: http://localhost:8082/docs
- **Redoc**: http://localhost:8082/redoc

## 🤝 Support

### Getting Help
1. Check the logs: `./deploy-local.sh logs`
2. Run tests: `./deploy-local.sh test`
3. Review configuration files
4. Check Docker Desktop status
5. Verify system requirements

### Advanced Configuration
For advanced configuration options, see:
- `docker-compose.local.yml` - Service definitions
- `local.env` - Environment variables
- `config/` directory - Service configurations
- `scripts/init-db.sql` - Database schema

---

**🎉 You now have a complete Enhanced Storage Service running locally with 109 API endpoints across all three phases!**

The system includes core laboratory management, advanced AI capabilities, and enterprise integration - representing the most comprehensive laboratory storage management platform available. 
