# TracSeq Transaction Service

## Overview

The TracSeq Transaction Service provides **distributed transaction management** using the **Saga pattern** to maintain data consistency across all TracSeq microservices. It ensures reliable execution of complex laboratory workflows while providing compensation mechanisms for failure scenarios.

## Key Features

- **Saga Pattern Implementation**: Orchestrates complex transactions across multiple services
- **Automatic Compensation**: Rollback mechanisms for failed transactions
- **Event-Driven Coordination**: Real-time transaction status updates
- **Laboratory-Specific Workflows**: Pre-built workflows for sample processing
- **Monitoring & Observability**: Comprehensive metrics and health monitoring

## Quick Start

### Development Setup

1. **Start with Docker Compose:**
   ```bash
   cd tracseq2.0/transaction_service
   docker-compose up -d
   ```

2. **Local development:**
   ```bash
   export RUST_LOG=info
   export EVENT_SERVICE_URL=http://localhost:8087
   cargo run
   ```

3. **Run tests:**
   ```bash
   cargo test
   ```

## API Endpoints

### Health & Status
- `GET /health` - Basic health check
- `GET /health/detailed` - Detailed health with dependencies

### Transaction Management
- `POST /api/v1/transactions` - Execute custom transaction
- `GET /api/v1/transactions` - List active transactions
- `GET /api/v1/transactions/{saga_id}` - Get transaction status
- `DELETE /api/v1/transactions/{saga_id}` - Cancel transaction

### Laboratory Workflows
- `POST /api/v1/workflows/sample-submission` - Execute sample submission workflow

### Metrics
- `GET /api/v1/metrics/coordinator` - Get coordinator statistics

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `8088` | Service port |
| `MAX_CONCURRENT_SAGAS` | `100` | Maximum concurrent transactions |
| `DEFAULT_TIMEOUT_MS` | `300000` | Default transaction timeout |
| `EVENT_SERVICE_URL` | `http://localhost:8087` | Event service URL |

## Architecture

The service implements the Saga pattern with:
- **Transaction Coordinator**: Manages saga execution
- **Step Execution**: Individual transaction steps with retry logic
- **Compensation Logic**: Automatic rollback for failed transactions
- **Event Integration**: Real-time coordination with other services

## License

Licensed under the MIT License.
