# Sample Management Service

A specialized microservice for managing laboratory sample lifecycle, from submission through completion, with comprehensive barcode management, validation, and workflow tracking.

## **🎯 Features**

### **Core Sample Management**
- ✅ **Complete Lifecycle**: Pending → Validated → InStorage → InSequencing → Completed
- ✅ **Barcode Generation**: Configurable format with prefix, timestamp, sequence
- ✅ **Batch Processing**: Efficient handling of multiple samples with rollback
- ✅ **Rich Metadata**: JSONB storage for flexible sample properties
- ✅ **Quality Tracking**: Concentration, volume, quality scores

### **Workflow & Validation**
- ✅ **State Transitions**: Enforced workflow with validation rules
- ✅ **Business Rules**: Configurable validation engine
- ✅ **Template Integration**: Lab-specific field definitions
- ✅ **Audit Trail**: Complete change history with user attribution

### **Integration & Security**
- ✅ **Authentication**: JWT-based with role-based access control
- ✅ **Storage Service**: Automated physical location management
- ✅ **Health Monitoring**: Comprehensive health checks and metrics
- ✅ **Error Handling**: Detailed error responses with proper HTTP status codes

---

## **🚀 Quick Start**

### **Environment Variables**
```bash
# Required
DATABASE_URL=postgresql://sample_user:password@localhost:5432/sample_db
AUTH_SERVICE_URL=http://auth-service:8080
STORAGE_SERVICE_URL=http://storage-service:8082

# Optional
SAMPLE_PORT=8081
BARCODE_PREFIX=LAB
SAMPLE_MAX_BATCH_SIZE=100
LOG_LEVEL=info
```

### **Docker Deployment**
```bash
# Build the service
docker build -t sample-service .

# Run with environment file
docker run --env-file .env -p 8081:8081 sample-service

# Or with docker-compose
docker-compose up sample-service
```

### **Local Development**
```bash
# Install dependencies
cargo build

# Run database migrations
cargo run --bin migrate

# Start the service
RUST_LOG=debug cargo run
```

---

## **📡 API Reference**

### **Sample Operations**

#### Create Sample
```bash
POST /samples
Content-Type: application/json
Authorization: Bearer <jwt_token>

{
  "name": "DNA Sample 001",
  "sample_type": "DNA",
  "source_type": "patient",
  "source_identifier": "P12345",
  "collection_date": "2024-03-20T10:00:00Z",
  "collection_location": "Lab A",
  "collector": "Dr. Smith",
  "concentration": 250.5,
  "volume": 50.0,
  "unit": "μL",
  "metadata": {
    "study_id": "STUDY_001",
    "consent_form": "CF-2024-001"
  }
}
```

#### List Samples
```bash
GET /samples?status=pending&limit=50&offset=0
Authorization: Bearer <jwt_token>
```

#### Update Sample Status
```bash
PUT /samples/{sample_id}/status
Content-Type: application/json
Authorization: Bearer <jwt_token>

{
  "new_status": "validated",
  "reason": "All validation checks passed",
  "notify": true
}
```

### **Barcode Operations**

#### Generate Barcode
```bash
POST /barcodes/generate
Content-Type: application/json
Authorization: Bearer <jwt_token>

{
  "prefix": "LAB",
  "sample_type": "DNA",
  "include_timestamp": true,
  "include_sequence": true
}
```

#### Scan Barcode
```bash
POST /barcodes/scan
Content-Type: application/json
Authorization: Bearer <jwt_token>

{
  "barcode": "LAB-2024-001234"
}
```

### **Batch Operations**

#### Create Batch Samples
```bash
POST /samples/batch
Content-Type: application/json
Authorization: Bearer <jwt_token>

{
  "samples": [
    {
      "name": "Sample 001",
      "sample_type": "DNA",
      "source_identifier": "P001"
    },
    {
      "name": "Sample 002", 
      "sample_type": "RNA",
      "source_identifier": "P002"
    }
  ],
  "auto_generate_barcodes": true,
  "batch_name": "Batch 2024-03-20"
}
```

### **Workflow Management**

#### Get Valid Transitions
```bash
GET /workflow/transitions?status=pending
Authorization: Bearer <jwt_token>
```

#### Get Sample History
```bash
GET /workflow/history/{sample_id}
Authorization: Bearer <jwt_token>
```

---

## **🔧 Configuration**

### **Sample Configuration**
```rust
pub struct SampleConfig {
    pub max_batch_size: usize,           // Default: 100
    pub default_status: String,          // Default: "pending"
    pub auto_generate_barcode: bool,     // Default: true
    pub validation_timeout_seconds: u64, // Default: 30
    pub metadata_max_size_kb: usize,     // Default: 64
}
```

### **Barcode Configuration**
```rust
pub struct BarcodeConfig {
    pub prefix: String,                  // Default: "LAB"
    pub length: usize,                   // Default: 12
    pub include_timestamp: bool,         // Default: true
    pub include_sequence: bool,          // Default: true
    pub separator: String,               // Default: "-"
    pub checksum: bool,                  // Default: false
}
```

### **Environment Variables Reference**
| Variable | Description | Default |
|----------|-------------|---------|
| `SAMPLE_HOST` | Server bind address | `0.0.0.0` |
| `SAMPLE_PORT` | Server port | `8081` |
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `AUTH_SERVICE_URL` | Authentication service URL | Required |
| `STORAGE_SERVICE_URL` | Storage service URL | Required |
| `BARCODE_PREFIX` | Default barcode prefix | `LAB` |
| `SAMPLE_MAX_BATCH_SIZE` | Maximum samples per batch | `100` |
| `LOG_LEVEL` | Logging level | `info` |

---

## **🗄️ Database Schema**

### **Core Tables**
```sql
-- Sample entity
CREATE TABLE samples (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    barcode VARCHAR(100) NOT NULL UNIQUE,
    sample_type VARCHAR(50) NOT NULL,
    status sample_status NOT NULL DEFAULT 'pending',
    template_id UUID,
    source_type VARCHAR(50),
    source_identifier VARCHAR(255),
    collection_date TIMESTAMPTZ,
    collection_location VARCHAR(255),
    collector VARCHAR(255),
    concentration DECIMAL(10,4),
    volume DECIMAL(10,4),
    unit VARCHAR(20),
    quality_score DECIMAL(3,2),
    metadata JSONB DEFAULT '{}',
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by VARCHAR(255),
    updated_by VARCHAR(255)
);

-- Status history
CREATE TABLE sample_status_history (
    id SERIAL PRIMARY KEY,
    sample_id UUID NOT NULL REFERENCES samples(id),
    previous_status sample_status,
    new_status sample_status NOT NULL,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    changed_by VARCHAR(255),
    reason VARCHAR(500),
    automated BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB DEFAULT '{}'
);

-- Validation rules
CREATE TABLE sample_validation_rules (
    id SERIAL PRIMARY KEY,
    rule_name VARCHAR(100) NOT NULL UNIQUE,
    sample_type VARCHAR(50),
    rule_expression TEXT NOT NULL,
    error_message VARCHAR(500),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    severity VARCHAR(20) NOT NULL DEFAULT 'error'
);
```

---

## **🔍 Health Monitoring**

### **Health Endpoints**
```bash
# Basic health check
GET /health

# Readiness check (includes dependencies)
GET /health/ready

# Application metrics
GET /health/metrics
```

### **Sample Health Response**
```json
{
  "status": "healthy",
  "timestamp": "2024-03-20T10:00:00Z",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "dependencies": {
    "database": {
      "status": "healthy",
      "response_time_ms": 5
    },
    "auth_service": {
      "status": "healthy", 
      "response_time_ms": 12
    },
    "storage_service": {
      "status": "healthy",
      "response_time_ms": 8
    }
  }
}
```

---

## **🛡️ Security**

### **Authentication**
- **JWT Tokens**: Required for all non-health endpoints
- **Role-Based Access**: Different permissions per user role
- **Service-to-Service**: Inter-service authentication

### **Authorization Levels**
| Role | Permissions |
|------|-------------|
| **Guest** | Read-only access to public samples |
| **Technician** | Create/update samples, scan barcodes |
| **Scientist** | Full sample management, template operations |
| **PI** | Project management, batch operations |
| **Lab Admin** | Full system access, configuration |

### **Rate Limiting**
- **Sample Creation**: 100 requests/minute per user
- **Batch Operations**: 10 requests/minute per user
- **Barcode Generation**: 1000 requests/minute per user

---

## **🐳 Docker Configuration**

### **Dockerfile**
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/sample_service /usr/local/bin/
EXPOSE 8081
CMD ["sample_service"]
```

### **Docker Compose**
```yaml
version: '3.8'
services:
  sample-service:
    build: .
    ports:
      - "8081:8081"
    environment:
      - DATABASE_URL=postgresql://sample_user:password@sample-db:5432/sample_db
      - AUTH_SERVICE_URL=http://auth-service:8080
      - STORAGE_SERVICE_URL=http://storage-service:8082
      - RUST_LOG=info
    depends_on:
      - sample-db
      - auth-service
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8081/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  sample-db:
    image: postgres:15
    environment:
      - POSTGRES_DB=sample_db
      - POSTGRES_USER=sample_user
      - POSTGRES_PASSWORD=password
    volumes:
      - sample_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

volumes:
  sample_data:
```

---

## **📊 Metrics & Monitoring**

### **Application Metrics**
- **Sample Creation Rate**: Samples created per minute
- **Validation Success Rate**: Percentage of samples passing validation
- **Barcode Generation Rate**: Barcodes generated per minute
- **API Response Times**: 95th percentile response times
- **Error Rates**: 4xx and 5xx error percentages

### **Business Metrics**
- **Sample Throughput**: Daily sample processing volume
- **Workflow Efficiency**: Average time per status transition
- **Batch Success Rate**: Percentage of successful batch operations
- **Template Usage**: Most popular template types

---

## **🔧 Development**

### **Running Tests**
```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_tests

# With test output
cargo test -- --nocapture
```

### **Code Quality**
```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Security audit
cargo audit
```

### **Database Migrations**
```bash
# Create new migration
sqlx migrate add create_new_table

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

---

## **🐛 Troubleshooting**

### **Common Issues**

#### Database Connection Failed
```bash
# Check database URL format
DATABASE_URL=postgresql://user:password@host:port/database

# Test connection
psql $DATABASE_URL -c "SELECT 1;"
```

#### Authentication Service Unreachable
```bash
# Check service URL
curl http://auth-service:8080/health

# Check network connectivity
docker network ls
```

#### High Memory Usage
```bash
# Check sample batch size
SAMPLE_MAX_BATCH_SIZE=50

# Enable memory monitoring
RUST_LOG=debug,sample_service::memory=trace
```

### **Debug Mode**
```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Trace specific modules
RUST_LOG=sample_service::services=trace cargo run
```

---

## **📈 Performance Tuning**

### **Database Optimization**
- **Connection Pool**: 10-20 connections for typical loads
- **Query Optimization**: Use indexes on frequently queried fields
- **Batch Operations**: Process up to 100 samples per batch

### **Memory Management**
- **Metadata Size**: Limit to 64KB per sample
- **Batch Processing**: Stream large datasets
- **Connection Reuse**: HTTP client connection pooling

---

## **🤝 Contributing**

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### **Code Standards**
- Follow Rust formatting guidelines
- Add tests for new functionality
- Update documentation for API changes
- Ensure all health checks pass

---

## **📄 License**

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

---

*Sample Management Service - Part of the Laboratory Management Microservices Ecosystem* 
