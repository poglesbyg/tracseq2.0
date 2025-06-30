# Sequencing Management Service

## Overview

The Sequencing Management Service is a comprehensive microservice designed to handle all aspects of laboratory sequencing operations, from job creation and workflow management to quality control and data export. It integrates seamlessly with other laboratory services and provides advanced bioinformatics pipeline support.

## Features

### 🧬 Core Capabilities
- **Advanced Workflow Management**: Multi-stage sequencing pipelines with dependency tracking
- **Sample Sheet Generation**: Automated sample sheet creation with validation
- **Quality Control Integration**: Real-time QC monitoring and reporting
- **Bioinformatics Pipelines**: Integrated analysis workflows with custom algorithms
- **Device Integration**: Direct connection to sequencing instruments
- **Job Scheduling**: Advanced scheduling with resource optimization
- **Run Monitoring**: Real-time run tracking with performance metrics
- **Data Export**: Multiple format export with compression

### 🔬 Advanced Features
- **State Machine Workflows**: Complex workflow orchestration
- **Real-time QC Metrics**: Live quality control monitoring
- **Custom Analysis Pipelines**: Configurable bioinformatics workflows
- **Resource Optimization**: Intelligent resource allocation
- **Integration APIs**: Seamless integration with other lab services

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              Sequencing Management Service                 │
│                     (Port 8084)                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   Workflow  │  │  Quality    │  │   Sample    │        │
│  │   Engine    │  │  Control    │  │   Sheets    │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │ Bioinform   │  │ Scheduling  │  │ Integration │        │
│  │ Pipelines   │  │   Engine    │  │   Layer     │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
          │                    │                    │
          ▼                    ▼                    ▼
  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
  │   Sample    │    │ Notification│    │  Template   │
  │  Service    │    │  Service    │    │  Service    │
  │ (Port 8081) │    │ (Port 8085) │    │ (Port 8083) │
  └─────────────┘    └─────────────┘    └─────────────┘
```

## API Endpoints

### Sequencing Job Management

#### Create Sequencing Job
```http
POST /jobs
Content-Type: application/json

{
    "name": "WGS Batch 2024-001",
    "description": "Whole genome sequencing for cancer research project",
    "sample_ids": ["uuid1", "uuid2", "uuid3"],
    "sequencing_type": "whole_genome",
    "platform": "illumina_novaseq",
    "priority": "high",
    "metadata": {
        "project_id": "PROJ-001",
        "researcher": "Dr. Smith"
    }
}
```

#### List Sequencing Jobs
```http
GET /jobs?status=pending&platform=illumina&limit=10&offset=0
```

#### Get Job Details
```http
GET /jobs/{job_id}
```

#### Update Job Status
```http
PUT /jobs/{job_id}/status
Content-Type: application/json

{
    "status": "running",
    "notes": "Job started on NovaSeq instrument #2"
}
```

#### Clone Existing Job
```http
POST /jobs/{job_id}/clone
Content-Type: application/json

{
    "name": "WGS Batch 2024-002",
    "sample_ids": ["uuid4", "uuid5", "uuid6"]
}
```

### Workflow Management

#### List Available Workflows
```http
GET /workflows
```

#### Execute Workflow
```http
POST /workflows/{workflow_id}/execute
Content-Type: application/json

{
    "job_id": "job_uuid",
    "parameters": {
        "reference_genome": "hg38",
        "quality_threshold": 30
    }
}
```

#### Pause/Resume/Abort Workflow
```http
POST /workflows/{workflow_id}/pause
POST /workflows/{workflow_id}/resume
POST /workflows/{workflow_id}/abort
```

### Sample Sheet Management

#### Create Sample Sheet
```http
POST /sample-sheets
Content-Type: application/json

{
    "name": "NovaSeq_Run_001",
    "platform": "illumina_novaseq",
    "samples": [
        {
            "sample_id": "Sample_1",
            "sample_name": "Patient_001_DNA",
            "index": "AGTCAACT",
            "index2": "TCGTGGAC"
        }
    ],
    "run_parameters": {
        "read_length": 150,
        "paired_end": true,
        "cycles": 300
    }
}
```

#### Download Sample Sheet
```http
GET /sample-sheets/{sheet_id}/download
Accept: text/csv
```

#### Validate Sample Sheet
```http
POST /sample-sheets/{sheet_id}/validate
```

### Quality Control

#### Get Real-time QC Metrics
```http
GET /qc/metrics/real-time?run_id={run_id}
```

**Response:**
```json
{
    "run_id": "run_001",
    "timestamp": "2024-03-20T15:30:00Z",
    "metrics": {
        "cluster_density": 875.2,
        "pf_clusters": 94.8,
        "q30_score": 92.1,
        "error_rate": 0.65,
        "phasing": 0.18,
        "prephasing": 0.12
    },
    "status": "passing",
    "alerts": []
}
```

#### Get QC Thresholds
```http
GET /qc/thresholds
```

#### Update QC Thresholds
```http
PUT /qc/thresholds
Content-Type: application/json

{
    "cluster_density_min": 800,
    "cluster_density_max": 1000,
    "q30_threshold": 85,
    "error_rate_max": 1.0
}
```

### Analysis Pipelines

#### List Analysis Pipelines
```http
GET /analysis/pipelines
```

#### Execute Custom Pipeline
```http
POST /analysis/pipelines/custom
Content-Type: application/json

{
    "name": "Custom RNA-seq Pipeline",
    "steps": [
        {
            "name": "quality_trimming",
            "tool": "trimmomatic",
            "parameters": {
                "min_length": 36,
                "quality_threshold": 30
            }
        },
        {
            "name": "alignment",
            "tool": "star",
            "parameters": {
                "reference_genome": "hg38",
                "max_mismatches": 2
            }
        },
        {
            "name": "quantification",
            "tool": "featurecounts",
            "parameters": {
                "feature_type": "exon",
                "attribute": "gene_id"
            }
        }
    ],
    "compute_requirements": {
        "cpu_cores": 16,
        "memory_gb": 64,
        "storage_gb": 500
    }
}
```

#### Get Analysis Results
```http
GET /analysis/jobs/{job_id}/results
```

### Scheduling

#### Schedule Job
```http
POST /schedule/jobs
Content-Type: application/json

{
    "job_id": "job_uuid",
    "scheduled_time": "2024-03-21T09:00:00Z",
    "priority": "high",
    "resource_requirements": {
        "instrument": "novaseq_001",
        "estimated_duration_hours": 24
    }
}
```

#### Get Schedule Calendar
```http
GET /schedule/calendar?start_date=2024-03-20&end_date=2024-03-27
```

### Integration

#### Validate Samples for Sequencing
```http
POST /integration/samples/validate
Content-Type: application/json

{
    "sample_ids": ["uuid1", "uuid2"],
    "sequencing_type": "whole_genome",
    "quality_requirements": {
        "min_concentration": 10.0,
        "min_volume": 50.0,
        "purity_ratio": 1.8
    }
}
```

#### Get Sequencing Templates
```http
GET /integration/templates/sequencing?platform=illumina
```

### Data Export

#### Export Job Data
```http
GET /export/jobs?format=csv&start_date=2024-01-01&end_date=2024-03-20
```

#### Export Run Metrics
```http
GET /export/runs/{run_id}/metrics?format=json
```

#### Export Results
```http
GET /export/results/{job_id}?format=fastq&compression=gzip
```

## Configuration

### Environment Variables

```bash
# Server Configuration
SEQUENCING_HOST=0.0.0.0
SEQUENCING_PORT=8084
SEQUENCING_WORKERS=4

# Database
SEQUENCING_DATABASE_URL=postgresql://sequencing_user:password@localhost:5432/sequencing_db

# Service Integration
AUTH_SERVICE_URL=http://auth-service:8080
SAMPLE_SERVICE_URL=http://sample-service:8081
NOTIFICATION_SERVICE_URL=http://notification-service:8085
TEMPLATE_SERVICE_URL=http://template-service:8083

# Workflow Configuration
WORKFLOW_MAX_CONCURRENT_JOBS=10
WORKFLOW_DEFAULT_TIMEOUT_HOURS=48
WORKFLOW_RETRY_ATTEMPTS=3

# Quality Control
QC_REAL_TIME_MONITORING=true
QC_ALERT_THRESHOLD_VIOLATIONS=true
QC_AUTO_FAIL_ON_CRITICAL=true

# Bioinformatics
BIOINFORMATICS_TOOLS_PATH=/opt/bioinformatics
REFERENCE_GENOMES_PATH=/data/references
ANALYSIS_SCRATCH_PATH=/tmp/analysis

# Instrument Integration
INSTRUMENT_API_ENABLED=true
INSTRUMENT_POLLING_INTERVAL_SECONDS=30
INSTRUMENT_DATA_PATH=/data/sequencing_runs

# Storage and Caching
DATA_RETENTION_DAYS=365
CACHE_SIZE_MB=1024
TEMP_FILE_CLEANUP_HOURS=24

# Security
JWT_SECRET=your-sequencing-service-jwt-secret
API_RATE_LIMIT_PER_MINUTE=1000
CORS_ALLOWED_ORIGINS=http://localhost:3000,https://lab.company.com

# Logging
LOG_LEVEL=info
LOG_FORMAT=json
AUDIT_LOGGING_ENABLED=true
```

### Database Schema

```sql
-- Sequencing jobs
CREATE TABLE sequencing_jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status sequencing_job_status NOT NULL DEFAULT 'pending',
    sequencing_type VARCHAR(100) NOT NULL,
    platform VARCHAR(100) NOT NULL,
    priority priority_level NOT NULL DEFAULT 'medium',
    sample_ids UUID[] NOT NULL,
    workflow_id UUID REFERENCES workflows(id),
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    estimated_duration_hours INTEGER,
    actual_duration_hours DECIMAL
);

-- Workflows
CREATE TABLE workflows (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    workflow_type VARCHAR(100) NOT NULL,
    steps JSONB NOT NULL,
    default_parameters JSONB NOT NULL DEFAULT '{}',
    compute_requirements JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT true
);

-- Sample sheets
CREATE TABLE sample_sheets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    platform VARCHAR(100) NOT NULL,
    job_id UUID REFERENCES sequencing_jobs(id),
    file_path VARCHAR(1024),
    samples JSONB NOT NULL,
    run_parameters JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    validated BOOLEAN DEFAULT false,
    validation_errors JSONB
);

-- Sequencing runs
CREATE TABLE sequencing_runs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    job_id UUID NOT NULL REFERENCES sequencing_jobs(id),
    run_name VARCHAR(255) NOT NULL,
    instrument_id VARCHAR(100) NOT NULL,
    status run_status NOT NULL DEFAULT 'pending',
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    run_parameters JSONB NOT NULL DEFAULT '{}',
    metrics JSONB NOT NULL DEFAULT '{}',
    output_path VARCHAR(1024),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Quality control metrics
CREATE TABLE qc_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    run_id UUID NOT NULL REFERENCES sequencing_runs(id),
    metric_name VARCHAR(100) NOT NULL,
    metric_value DECIMAL NOT NULL,
    metric_unit VARCHAR(50),
    threshold_min DECIMAL,
    threshold_max DECIMAL,
    status qc_status NOT NULL,
    measured_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Analysis jobs
CREATE TABLE analysis_jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    sequencing_job_id UUID NOT NULL REFERENCES sequencing_jobs(id),
    pipeline_id UUID NOT NULL REFERENCES workflows(id),
    status analysis_status NOT NULL DEFAULT 'pending',
    parameters JSONB NOT NULL DEFAULT '{}',
    results_path VARCHAR(1024),
    log_path VARCHAR(1024),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_sequencing_jobs_status ON sequencing_jobs(status);
CREATE INDEX idx_sequencing_jobs_created_at ON sequencing_jobs(created_at);
CREATE INDEX idx_sequencing_jobs_priority ON sequencing_jobs(priority);
CREATE INDEX idx_sequencing_runs_status ON sequencing_runs(status);
CREATE INDEX idx_qc_metrics_run_id ON qc_metrics(run_id);
CREATE INDEX idx_analysis_jobs_status ON analysis_jobs(status);
```

## Development

### Building

```bash
# Build the service
cargo build --release

# Run tests
cargo test

# Run with development configuration
cargo run
```

### Docker

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/sequencing_service /usr/local/bin/

EXPOSE 8084

CMD ["sequencing_service"]
```

### Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_tests

# Performance tests
cargo test --test performance_tests --release
```

## Monitoring

### Health Checks

- `GET /health` - Basic health check
- `GET /health/ready` - Readiness probe
- `GET /health/metrics` - Prometheus metrics

### Metrics

The service exposes Prometheus metrics including:

- `sequencing_jobs_total` - Total number of jobs
- `sequencing_jobs_duration_seconds` - Job duration histogram
- `sequencing_runs_total` - Total number of runs
- `qc_metrics_violations_total` - QC threshold violations
- `workflow_execution_duration_seconds` - Workflow execution time
- `api_requests_total` - Total API requests
- `api_request_duration_seconds` - API request duration

### Logging

The service provides structured logging with configurable levels:

```json
{
    "timestamp": "2024-03-20T15:30:00Z",
    "level": "INFO",
    "service": "sequencing_service",
    "message": "Sequencing job created",
    "job_id": "uuid",
    "user_id": "uuid",
    "request_id": "uuid"
}
```

## Troubleshooting

### Common Issues

1. **Job Stuck in Pending State**
   - Check instrument availability
   - Verify resource requirements
   - Review scheduling conflicts

2. **QC Metrics Not Updating**
   - Verify instrument API connection
   - Check QC polling configuration
   - Review instrument data path

3. **Analysis Pipeline Failures**
   - Check bioinformatics tool availability
   - Verify reference genome paths
   - Review compute resource allocation

### Debug Mode

Enable debug logging:

```bash
export LOG_LEVEL=debug
export RUST_LOG=sequencing_service=debug
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

---

*Advanced sequencing workflow management for modern laboratories* 🧬 
