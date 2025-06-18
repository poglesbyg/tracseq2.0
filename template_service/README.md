# Template Management Service

A comprehensive microservice for managing laboratory templates, dynamic form generation, and data validation.

## Features

- **Template Management**: Full CRUD operations with versioning
- **Dynamic Form Builder**: Runtime form generation from templates
- **Advanced Validation**: Field-level and cross-field validation rules
- **File Processing**: Excel, CSV, JSON import/export
- **Version Control**: Semantic versioning with rollback
- **Integration APIs**: Seamless service communication

## Quick Start

### Prerequisites

- Rust 1.70+
- PostgreSQL 14+
- Docker & Docker Compose (optional)

### Environment Variables

```bash
# Server Configuration
TEMPLATE_HOST=0.0.0.0
TEMPLATE_PORT=8083

# Database
TEMPLATE_DATABASE_URL=postgresql://template_user:password@localhost:5432/template_db

# Service URLs
AUTH_SERVICE_URL=http://auth-service:8080
SAMPLE_SERVICE_URL=http://sample-service:8081

# Features
FEATURE_FORM_BUILDER=true
FEATURE_TEMPLATE_VERSIONING=true
FEATURE_FILE_UPLOAD=true

# Template Configuration
TEMPLATE_MAX_PER_USER=50
TEMPLATE_MAX_FIELDS=100
TEMPLATE_CACHE_ENABLED=true

# Validation Configuration
VALIDATION_STRICT_MODE=true
VALIDATION_CROSS_FIELD_ENABLED=true

# File Configuration
FILE_UPLOAD_PATH=./uploads
FILE_MAX_SIZE_MB=10
FILE_ALLOWED_EXTENSIONS=xlsx,csv,json,xml
```

### Development Setup

1. **Clone and build**:
   ```bash
   cd template_service
   cargo build
   ```

2. **Setup database**:
   ```bash
   # Create database
   createdb template_db
   
   # Run migrations (once implemented)
   cargo run --bin migrate
   ```

3. **Run the service**:
   ```bash
   cargo run
   ```

### Docker Setup

```bash
# Build and run with Docker Compose
docker-compose up --build
```

## API Endpoints

### Template Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/templates` | Create new template |
| `GET` | `/templates` | List templates with filtering |
| `GET` | `/templates/{id}` | Get template details |
| `PUT` | `/templates/{id}` | Update template |
| `DELETE` | `/templates/{id}` | Delete template |
| `POST` | `/templates/{id}/clone` | Clone template |

### Field Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/templates/{id}/fields` | List template fields |
| `POST` | `/templates/{id}/fields` | Create field |
| `GET` | `/templates/{id}/fields/{field_id}` | Get field details |
| `PUT` | `/templates/{id}/fields/{field_id}` | Update field |
| `DELETE` | `/templates/{id}/fields/{field_id}` | Delete field |
| `POST` | `/templates/{id}/fields/reorder` | Reorder fields |

### Form Generation

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/forms/{id}/generate` | Generate dynamic form |
| `POST` | `/forms/{id}/validate` | Validate form submission |
| `GET` | `/forms/{id}/preview` | Preview form rendering |
| `POST` | `/forms/{id}/render` | Render form with data |

### Validation Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/templates/{id}/validation` | Get validation rules |
| `POST` | `/templates/{id}/validation` | Create validation rule |
| `PUT` | `/templates/{id}/validation/{rule_id}` | Update validation rule |
| `DELETE` | `/templates/{id}/validation/{rule_id}` | Delete validation rule |
| `POST` | `/templates/{id}/validate-data` | Validate form data |

### Version Control

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/templates/{id}/versions` | List template versions |
| `POST` | `/templates/{id}/versions` | Create new version |
| `GET` | `/templates/{id}/versions/{version}` | Get specific version |
| `DELETE` | `/templates/{id}/versions/{version}` | Delete version |
| `POST` | `/templates/{id}/versions/{version}/restore` | Restore version |

### Health & Monitoring

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Basic health check |
| `GET` | `/health/ready` | Readiness check |
| `GET` | `/health/metrics` | Application metrics |

## Template Structure

### Basic Template

```json
{
  "name": "Sample Collection Template",
  "description": "Template for laboratory sample collection",
  "template_type": "sample_collection",
  "category": "laboratory",
  "tags": ["samples", "collection", "lab"],
  "is_public": false,
  "fields": [
    {
      "name": "sample_name",
      "label": "Sample Name",
      "field_type": "text",
      "is_required": true,
      "validation_rules": [
        {
          "rule_type": "pattern",
          "rule_value": "^[A-Z]{2}\\d{4}$",
          "error_message": "Format: XX0000"
        }
      ]
    }
  ]
}
```

### Field Types

- `text` - Single line text input
- `textarea` - Multi-line text input
- `number` - Numeric input
- `email` - Email address
- `phone` - Phone number
- `url` - URL input
- `date` - Date picker
- `datetime` - Date and time picker
- `select` - Dropdown selection
- `multiselect` - Multiple selection
- `radio` - Radio buttons
- `checkbox` - Checkboxes
- `boolean` - Yes/No toggle
- `file` - File upload
- `password` - Password input
- `hidden` - Hidden field

### Validation Rules

- `required` - Field is mandatory
- `min_length` - Minimum character length
- `max_length` - Maximum character length
- `pattern` - Regular expression pattern
- `min_value` - Minimum numeric value
- `max_value` - Maximum numeric value
- `email` - Valid email format
- `phone` - Valid phone format
- `url` - Valid URL format
- `date` - Valid date format
- `custom` - Custom validation function
- `cross_field` - Cross-field validation

## Form Generation

### Generate Form

```bash
GET /forms/{template_id}/generate?format=html&theme=modern
```

Response:
```json
{
  "template_id": "uuid",
  "form_html": "<form>...</form>",
  "form_config": {
    "fields": [...],
    "validation": {...},
    "dependencies": [...]
  },
  "validation_schema": {...},
  "metadata": {...}
}
```

### Validate Form Data

```bash
POST /forms/{template_id}/validate
```

Request:
```json
{
  "form_data": {
    "sample_name": "LAB-001",
    "collection_date": "2024-03-20",
    "sample_type": "DNA"
  },
  "validate_dependencies": true,
  "strict_mode": false
}
```

Response:
```json
{
  "is_valid": true,
  "field_errors": {},
  "global_errors": [],
  "warnings": [],
  "validated_data": {...}
}
```

## Integration

### Sample Service Integration

Create sample from template:
```bash
POST /integration/samples/create
{
  "template_id": "uuid",
  "sample_data": {...},
  "generate_barcode": true
}
```

Validate sample data:
```bash
POST /integration/samples/validate
{
  "template_id": "uuid",
  "sample_data": {...},
  "strict_mode": true
}
```

### Authentication

All endpoints (except health checks) require authentication via JWT token:

```bash
Authorization: Bearer <jwt_token>
```

The service validates tokens with the authentication service.

## Configuration

### Template Limits

- Maximum templates per user: 50 (configurable)
- Maximum fields per template: 100 (configurable)
- Maximum validation rules per field: 10 (configurable)

### File Limits

- Maximum file size: 10MB (configurable)
- Allowed extensions: xlsx, csv, json, xml (configurable)
- Upload path: `./uploads` (configurable)

### Performance

- Template caching: Enabled by default
- Cache TTL: 3600 seconds (configurable)
- Form generation timeout: 30 seconds (configurable)
- Validation timeout: 10 seconds (configurable)

## Error Handling

The service returns structured error responses:

```json
{
  "error": "validation_error",
  "message": "Field validation failed",
  "details": {...},
  "timestamp": "2024-03-20T10:30:00Z",
  "request_id": "req_123"
}
```

### Common Error Codes

- `validation_error` - Input validation failed
- `template_not_found` - Template does not exist
- `field_not_found` - Field does not exist
- `authentication_error` - Invalid authentication
- `authorization_error` - Insufficient permissions
- `business_rule_violation` - Business rule violation
- `external_service_error` - External service unavailable

## Development

### Project Structure

```
template_service/
├── src/
│   ├── main.rs              # Application entry point
│   ├── config.rs            # Configuration management
│   ├── database.rs          # Database connection
│   ├── models.rs            # Data models
│   ├── error.rs             # Error handling
│   ├── services.rs          # Business logic
│   ├── middleware.rs        # Authentication middleware
│   ├── clients.rs           # External service clients
│   └── handlers/            # API endpoint handlers
│       ├── templates.rs     # Template operations
│       ├── fields.rs        # Field operations
│       ├── forms.rs         # Form generation
│       ├── validation.rs    # Validation operations
│       ├── versions.rs      # Version control
│       ├── files.rs         # File operations
│       ├── integration.rs   # Service integration
│       ├── health.rs        # Health checks
│       └── admin.rs         # Admin operations
├── migrations/              # Database migrations
├── tests/                   # Test files
├── Cargo.toml              # Dependencies
├── Dockerfile              # Container configuration
└── docker-compose.yml      # Local development setup
```

### Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration

# Run with coverage
cargo tarpaulin --out html
```

### Code Quality

```bash
# Format code
cargo fmt

# Check for issues
cargo clippy

# Security audit
cargo audit
```

## Production Deployment

### Docker Deployment

```yaml
services:
  template-service:
    image: template-service:latest
    ports:
      - "8083:8083"
    environment:
      - DATABASE_URL=${DATABASE_URL}
      - AUTH_SERVICE_URL=${AUTH_SERVICE_URL}
      - SAMPLE_SERVICE_URL=${SAMPLE_SERVICE_URL}
    depends_on:
      - template-db
      - auth-service
    volumes:
      - template_uploads:/app/uploads
      - template_backups:/app/backups
    restart: unless-stopped
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: template-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: template-service
  template:
    metadata:
      labels:
        app: template-service
    spec:
      containers:
      - name: template-service
        image: template-service:latest
        ports:
        - containerPort: 8083
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: template-service-secrets
              key: database-url
        livenessProbe:
          httpGet:
            path: /health
            port: 8083
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8083
```

### Monitoring

The service exposes metrics at `/health/metrics` for monitoring systems like Prometheus.

Key metrics include:
- Request count and latency
- Template operations (create, update, delete)
- Form generation performance
- Validation success rates
- Database connection health
- External service availability

### Logging

Structured logging with configurable levels:
- Error: Critical errors and failures
- Warn: Warning conditions
- Info: General information (default)
- Debug: Detailed debugging information
- Trace: Very detailed tracing

## Support

For issues and questions:
- Check the logs for error details
- Review the configuration settings
- Verify service dependencies are running
- Check database connectivity
- Validate authentication service availability

## License

This project is part of the Laboratory Management System and follows the same licensing terms. 
