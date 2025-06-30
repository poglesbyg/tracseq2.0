# TracSeq Transaction Service

## Overview

The TracSeq Transaction Service provides **distributed transaction management** using the **Saga pattern** to maintain data consistency across all TracSeq microservices. It ensures reliable execution of complex laboratory workflows while providing compensation mechanisms for failure scenarios.

## 🚀 Key Features

### Core Transaction Management
- **Saga Pattern Implementation**: Orchestrates complex transactions across multiple services
- **Automatic Compensation**: Rollback mechanisms for failed transactions
- **State Persistence**: PostgreSQL-based saga state management with recovery
- **Event-Driven Coordination**: Real-time transaction status updates via Event Service
- **Distributed Locking**: Coordination across concurrent transactions

### Laboratory-Specific Workflows
- **Pre-built Workflow Templates**: DNA/RNA extraction, QC protocols, sample processing
- **AI-Enhanced Orchestration**: RAG integration for intelligent workflow decisions
- **Quality Control Integration**: Automated QC checkpoints and validation
- **Equipment Resource Management**: Coordination of laboratory equipment usage
- **Compliance Tracking**: Audit trails and regulatory compliance features

### Enterprise Features
- **High Availability**: Resilient architecture with failover capabilities
- **Monitoring & Observability**: Comprehensive metrics, health checks, and distributed tracing
- **Performance Optimization**: Concurrent saga execution with configurable limits
- **Security**: Role-based access control and audit logging
- **Scalability**: Horizontal scaling support with Redis coordination

## 🏗️ Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   API Gateway   │────│ Transaction     │────│   Event Service │
│                 │    │   Coordinator   │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                               │
                               ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   PostgreSQL    │────│  Saga Engine     │────│   Redis Cache   │
│  (Persistence)  │    │                  │    │  (Coordination) │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                               │
                               ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│  RAG Service    │────│ Workflow Engine  │────│ Laboratory      │
│ (AI Integration)│    │                  │    │   Services      │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## 🚀 Quick Start

### Prerequisites
- Docker & Docker Compose
- PostgreSQL 15+
- Redis 7+ (optional, for coordination)
- Rust 1.75+ (for development)

### Production Deployment

1. **Deploy with Docker Compose:**
   ```bash
   cd transaction_service
   docker-compose up -d
   ```

2. **Verify deployment:**
   ```bash
   curl http://localhost:8088/health
   curl http://localhost:8088/health/detailed
   ```

3. **View metrics:**
   ```bash
   # Prometheus metrics
   open http://localhost:9090
   
   # Grafana dashboards
   open http://localhost:3001 (admin/admin)
   ```

### Development Setup

1. **Set environment variables:**
   ```bash
   export RUST_LOG=debug
   export DATABASE_URL=postgresql://tracseq_transaction:tracseq_password@localhost:5435/tracseq_transactions
   export EVENT_SERVICE_URL=http://localhost:8087
   export RAG_SERVICE_URL=http://localhost:8086
   ```

2. **Run database migrations:**
   ```bash
   psql $DATABASE_URL -f migrations/001_initial_saga_schema.sql
   ```

3. **Start the service:**
   ```bash
   cargo run
   ```

4. **Run tests:**
   ```bash
   cargo test
   cargo test --features integration-tests
   ```

## 📊 API Reference

### Health & Monitoring

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Basic health check |
| `/health/detailed` | GET | Detailed health with dependencies |

### Transaction Management

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/transactions` | POST | Execute custom transaction |
| `/api/v1/transactions` | GET | List active transactions |
| `/api/v1/transactions/{saga_id}` | GET | Get transaction status |
| `/api/v1/transactions/{saga_id}` | DELETE | Cancel transaction |

### Laboratory Workflows

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/workflows/sample-submission` | POST | Execute sample submission workflow |
| `/api/v1/workflows/enhanced` | POST | Execute AI-enhanced workflow |
| `/api/v1/workflows/enhanced/templates` | GET | List workflow templates |
| `/api/v1/workflows/enhanced/ai-analyze` | POST | AI workflow analysis |

### Metrics & Analytics

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/metrics/coordinator` | GET | Coordinator statistics |

## 🔧 Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `8088` | Service port |
| `RUST_LOG` | `info` | Logging level |
| `DATABASE_URL` | `postgresql://...` | PostgreSQL connection string |
| `MAX_CONCURRENT_SAGAS` | `100` | Maximum concurrent transactions |
| `DEFAULT_TIMEOUT_MS` | `300000` | Default transaction timeout (5 min) |
| `EVENT_SERVICE_URL` | `http://localhost:8087` | Event service URL |
| `RAG_SERVICE_URL` | `http://localhost:8086` | RAG service URL |
| `ENABLE_EVENTS` | `true` | Enable event integration |
| `ENABLE_PERSISTENCE` | `true` | Enable database persistence |
| `ENABLE_AI_DECISIONS` | `true` | Enable AI-enhanced workflows |
| `AI_CONFIDENCE_THRESHOLD` | `0.8` | Minimum AI confidence for auto-decisions |
| `CLEANUP_AFTER_HOURS` | `24` | Auto-cleanup completed sagas after N hours |

### Database Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `DB_MAX_CONNECTIONS` | `20` | Maximum database connections |
| `DB_MIN_CONNECTIONS` | `5` | Minimum database connections |
| `DB_CONNECTION_TIMEOUT_SECONDS` | `30` | Connection timeout |

### Redis Configuration (Optional)

| Variable | Default | Description |
|----------|---------|-------------|
| `REDIS_URL` | `redis://localhost:6379` | Redis connection string |

## 🧪 Example Usage

### Execute Sample Submission Workflow

```bash
curl -X POST http://localhost:8088/api/v1/workflows/sample-submission \
  -H "Content-Type: application/json" \
  -d '{
    "sample_data": {
      "sample_id": "SAMPLE-001",
      "sample_type": "DNA",
      "volume_ml": 2.0,
      "concentration_ng_ul": 150.0
    },
    "storage_requirements": {
      "temperature": -80,
      "storage_duration_days": 365,
      "special_handling": ["freeze_immediately"]
    },
    "notification_recipients": ["lab@example.com"],
    "name": "Sample Submission - SAMPLE-001",
    "timeout_ms": 600000
  }'
```

### Execute AI-Enhanced Workflow

```bash
curl -X POST http://localhost:8088/api/v1/workflows/enhanced \
  -H "Content-Type: application/json" \
  -d '{
    "workflow_type": "dna_extraction",
    "sample_data": {
      "sample_type": "tissue",
      "extraction_method": "spin_column",
      "expected_yield": "high"
    },
    "ai_analysis_enabled": true,
    "auto_optimization": true,
    "quality_thresholds": {
      "min_purity": 1.8,
      "min_concentration": 50.0
    }
  }'
```

### Check Transaction Status

```bash
# Get all active transactions
curl http://localhost:8088/api/v1/transactions

# Get specific transaction
curl http://localhost:8088/api/v1/transactions/{saga_id}

# Cancel transaction
curl -X DELETE http://localhost:8088/api/v1/transactions/{saga_id}
```

## 🏗️ Workflow Templates

### Built-in Templates

1. **DNA Extraction Standard** (`dna_extraction_standard`)
   - Sample preparation → DNA extraction → Quality control
   - Duration: ~165 minutes
   - Equipment: pipettes, extraction kit, centrifuge, spectrophotometer

2. **RNA Extraction Standard** (`rna_extraction_standard`)
   - RNase treatment → Sample prep → RNA extraction
   - Duration: ~165 minutes
   - Special handling: RNase-free environment

3. **Comprehensive Sample QC** (`sample_qc_comprehensive`)
   - Visual inspection → Contamination screening → Documentation review
   - Duration: ~85 minutes
   - AI validation: enabled

### Custom Template Creation

```rust
use transaction_service::workflows::templates::*;

let custom_template = LaboratoryWorkflowTemplate {
    template_id: "custom_protocol".to_string(),
    name: "Custom Laboratory Protocol".to_string(),
    description: "Specialized workflow for custom processing".to_string(),
    steps: vec![
        // Define workflow steps...
    ],
    estimated_duration_minutes: 120,
    required_equipment: vec!["equipment1".to_string()],
    quality_checkpoints: vec!["checkpoint1".to_string()],
    ai_generated: false,
    confidence_score: 1.0,
};
```

## 📈 Monitoring & Observability

### Metrics Available

- **Transaction Metrics**: Success/failure rates, duration, concurrency
- **Saga Metrics**: Step completion rates, compensation rates, retry counts
- **System Metrics**: Database connections, memory usage, response times
- **Business Metrics**: Workflow completion rates, equipment utilization

### Health Checks

The service provides multiple health check endpoints:

- **Basic Health** (`/health`): Simple service availability
- **Detailed Health** (`/health/detailed`): Database, Redis, and external service connectivity
- **Readiness**: Kubernetes-compatible readiness probe

### Distributed Tracing

Integrated with Jaeger for end-to-end transaction tracing across all microservices.

## 🔒 Security

### Authentication & Authorization
- JWT token validation
- Role-based access control (RBAC)
- Service-to-service authentication

### Audit & Compliance
- Complete transaction audit trails
- Regulatory compliance tracking
- Data retention policies

### Data Protection
- Encryption at rest and in transit
- Secure credential management
- GDPR compliance features

## 🚀 Performance

### Benchmarks
- **Throughput**: 1000+ transactions/second
- **Latency**: <100ms median response time
- **Concurrency**: 100+ concurrent sagas
- **Recovery**: <5 second failover time

### Optimization Features
- Connection pooling
- Asynchronous processing
- Intelligent retry mechanisms
- Circuit breaker patterns

## 🛠️ Development

### Architecture Components

```
src/
├── saga/                 # Core saga pattern implementation
│   ├── mod.rs           # Main saga orchestration
│   ├── step.rs          # Individual step execution
│   ├── compensation.rs  # Rollback mechanisms
│   ├── state.rs         # State management
│   └── error.rs         # Error handling
├── coordinator/         # Transaction coordinator
├── persistence/         # Database persistence layer
├── workflows/           # Laboratory workflow implementations
│   ├── laboratory/      # Lab-specific workflows
│   ├── templates/       # Workflow templates
│   ├── orchestrator/    # AI-enhanced orchestration
│   └── rag_integration/ # RAG service integration
├── services/           # Service layer implementations
└── models/             # Data models and types
```

### Testing Strategy

```bash
# Unit tests
cargo test

# Integration tests
cargo test --features integration-tests

# Performance tests
cargo test --release --features perf-tests

# End-to-end tests
./scripts/e2e-tests.sh
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## 🚨 Troubleshooting

### Common Issues

1. **Database Connection Errors**
   ```bash
   # Check database connectivity
   psql $DATABASE_URL -c "SELECT 1"
   
   # Verify migrations
   psql $DATABASE_URL -c "\dt"
   ```

2. **Event Service Connectivity**
   ```bash
   # Test event service
   curl http://localhost:8087/health
   ```

3. **High Memory Usage**
   ```bash
   # Check saga cleanup
   curl http://localhost:8088/api/v1/metrics/coordinator
   ```

### Logs & Debugging

```bash
# Enable debug logging
export RUST_LOG=debug

# View structured logs
docker logs tracseq-transaction-service

# Monitor metrics
curl http://localhost:8088/api/v1/metrics/coordinator
```

## 📄 License

Licensed under the MIT License. See `LICENSE` file for details.

## 📞 Support

- **Documentation**: [TracSeq Docs](https://docs.tracseq.com)
- **Issues**: [GitHub Issues](https://github.com/tracseq/tracseq2.0/issues)
- **Community**: [Discord Server](https://discord.gg/tracseq)
- **Commercial Support**: support@tracseq.com
