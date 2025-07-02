# TracSeq Barcode Service

A standalone microservice for laboratory barcode generation, validation, and management.

## Setup

### Database URL for SQLx

This service uses SQLx with compile-time query verification. You need to set the `DATABASE_URL` environment variable before building:

```bash
# Option 1: Use the setup script
source setup-env.sh

# Option 2: Export directly
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/barcode_service"

# Option 3: Add to your shell profile (~/.bashrc or ~/.zshrc)
echo 'export DATABASE_URL="postgres://postgres:postgres@localhost:5432/barcode_service"' >> ~/.zshrc
```

After setting the DATABASE_URL, you can build the service:

```bash
cargo build
```

## Features

- **Barcode Generation**: Generate unique barcodes with configurable patterns
- **Validation**: Validate barcode format and uniqueness
- **Reservation System**: Reserve and release barcodes for laboratory workflows
- **Parsing**: Extract components from existing barcodes
- **Statistics**: Track barcode generation metrics
- **Health Monitoring**: Comprehensive health checks and monitoring

## API Endpoints

### Health Check
```
GET /health
```

### Barcode Operations
```
POST /api/v1/barcodes/generate     - Generate a new barcode
POST /api/v1/barcodes/validate     - Validate a barcode
POST /api/v1/barcodes/parse        - Parse barcode components
POST /api/v1/barcodes/reserve      - Reserve a barcode
POST /api/v1/barcodes/release      - Release a barcode
POST /api/v1/barcodes/check        - Check barcode uniqueness
GET  /api/v1/barcodes/stats        - Get generation statistics
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | Server host |
| `PORT` | `8090` | Server port |
| `DATABASE_URL` | `postgresql://...` | PostgreSQL connection string |
| `ENVIRONMENT` | `development` | Environment (development/production) |
| `BARCODE_PREFIX` | `LAB` | Default barcode prefix |
| `BARCODE_SEPARATOR` | `-` | Barcode component separator |
| `BARCODE_MIN_LENGTH` | `8` | Minimum barcode length |
| `BARCODE_INCLUDE_DATE` | `true` | Include date in barcode |
| `BARCODE_INCLUDE_SEQUENCE` | `true` | Include sequence in barcode |
| `BARCODE_VALIDATION_PATTERN` | `^[A-Z0-9\-_]+$` | Validation regex |

## Usage Examples

### Generate a Barcode
```bash
curl -X POST http://localhost:8090/api/v1/barcodes/generate \
  -H "Content-Type: application/json" \
  -d '{
    "sample_type": "DNA",
    "location_id": 42,
    "template_name": "Genomic DNA Extraction"
  }'
```

Response:
```json
{
  "barcode": "LAB-DNA-20240315-L042-1234567",
  "info": {
    "full_barcode": "LAB-DNA-20240315-L042-1234567",
    "prefix": "LAB",
    "sample_type": "DNA",
    "date_component": "20240315",
    "location_component": 42,
    "sequence_component": "1234567",
    "is_valid": true,
    "generated_at": "2024-03-15T10:30:00Z"
  }
}
```

### Validate a Barcode
```bash
curl -X POST http://localhost:8090/api/v1/barcodes/validate \
  -H "Content-Type: application/json" \
  -d '{
    "barcode": "LAB-DNA-20240315-L042-1234567"
  }'
```

Response:
```json
{
  "is_valid": true,
  "errors": [],
  "info": {
    "full_barcode": "LAB-DNA-20240315-L042-1234567",
    "prefix": "LAB",
    "sample_type": "DNA",
    "is_valid": true
  }
}
```

### Reserve a Barcode
```bash
curl -X POST http://localhost:8090/api/v1/barcodes/reserve \
  -H "Content-Type: application/json" \
  -d '{
    "barcode": "LAB-DNA-20240315-L042-1234567",
    "reserved_by": "lab_technician",
    "purpose": "Sample processing workflow"
  }'
```

### Check Uniqueness
```bash
curl -X POST http://localhost:8090/api/v1/barcodes/check \
  -H "Content-Type: application/json" \
  -d '{
    "barcode": "LAB-DNA-20240315-L042-1234567"
  }'
```

Response:
```json
{
  "is_unique": false,
  "is_reserved": true,
  "reserved_by": "lab_technician",
  "reserved_at": "2024-03-15T10:30:00Z"
}
```

## Development

### Prerequisites
- Rust 1.75+
- PostgreSQL 14+
- Docker (optional)

### Running Locally
```bash
# Set environment variables
export DATABASE_URL="postgresql://user:password@localhost:5432/barcode_db"
export RUST_LOG=debug

# Run the service
cargo run
```

### Building with Docker
```bash
# Build image
docker build -t tracseq-barcode-service .

# Run container
docker run -p 8090:8090 \
  -e DATABASE_URL="postgresql://user:password@host:5432/barcode_db" \
  tracseq-barcode-service
```

## Database Schema

The service creates the following table:

```sql
CREATE TABLE barcodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    barcode VARCHAR(255) NOT NULL UNIQUE,
    prefix VARCHAR(50),
    sample_type VARCHAR(100),
    location_id INTEGER,
    is_reserved BOOLEAN NOT NULL DEFAULT false,
    reserved_by VARCHAR(255),
    reserved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB
);
```

## Integration

### Service Communication
The barcode service can be integrated with other TracSeq services:

- **Sample Service**: Generate barcodes during sample creation
- **Storage Service**: Use location-based barcode generation
- **Template Service**: Include template information in barcodes
- **API Gateway**: Route barcode requests through centralized gateway

### Client Libraries
The service provides a REST API that can be consumed by any HTTP client. For Rust services, you can use the generated client models.

## Monitoring

### Health Checks
- Database connectivity
- Service availability
- Barcode generation capacity

### Metrics
- Total barcodes generated
- Generation rate per day
- Reservation statistics
- Error rates

### Logging
Structured logging with tracing for:
- Request/response tracking
- Database operations
- Error conditions
- Performance metrics

## Error Handling

The service provides detailed error responses:

```json
{
  "error": {
    "message": "Barcode must be at least 8 characters long",
    "type": "validation_error",
    "timestamp": "2024-03-15T10:30:00Z"
  }
}
```

Error types:
- `validation_error`: Invalid barcode format
- `generation_failed`: Could not generate unique barcode
- `barcode_not_found`: Barcode does not exist
- `barcode_already_reserved`: Barcode is already reserved
- `database_error`: Database operation failed

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

MIT License - see LICENSE file for details. 