# Spreadsheet Versioning Service

The Spreadsheet Versioning Service is a microservice in the TracSeq 2.0 laboratory management system that provides comprehensive version control and difference tracking for spreadsheet files.

## 🎯 Features

### Core Versioning
- **Automatic version creation** when spreadsheets with the same name are uploaded
- **SHA-256 hash-based deduplication** to prevent storing identical files
- **Parent-child version relationships** for tracking change lineage
- **Metadata preservation** including file information and custom attributes

### Difference Engine
- **Cell-by-cell comparison** between any two versions
- **Structural change detection** (added/removed rows/columns)
- **Configurable diff options** (ignore whitespace, case sensitivity)
- **Comprehensive change summaries** with statistics

### Conflict Detection & Resolution
- **Three-way merge analysis** with common base versions
- **Automatic conflict detection** for concurrent changes
- **Multiple resolution strategies** (manual review, latest wins, auto-merge)
- **Conflict metadata tracking** for audit trails

## 🏗️ Architecture

### Database Schema
- `spreadsheet_versions` - Version metadata and file information
- `version_data` - Cell-by-cell spreadsheet content storage
- `version_diffs` - Computed differences between versions
- `version_conflicts` - Detected conflicts requiring resolution
- `version_merge_requests` - Merge workflow management

### Service Components
- **VersioningService** - Core version management operations
- **DiffEngine** - Advanced spreadsheet comparison algorithms
- **ConflictResolver** - Conflict detection and resolution logic
- **MergeEngine** - Version merging with multiple strategies

## 🚀 Getting Started

### Prerequisites
- Rust 1.75+
- PostgreSQL 15+
- Docker & Docker Compose (for containerized deployment)

### Environment Variables
```bash
# Server Configuration
PORT=8088
RUST_LOG=info

# Database
DATABASE_URL=postgresql://user:password@localhost/tracseq_versioning

# File Processing
MAX_FILE_SIZE_MB=100
MAX_VERSIONS_PER_SPREADSHEET=50

# Versioning
ENABLE_AUTO_VERSIONING=true
RETENTION_DAYS=365

# Diff Algorithm
DIFF_ALGORITHM=structural_aware  # cell_by_cell, structural_aware, semantic

# Conflict Resolution
CONFLICT_STRATEGY=manual_review   # latest_wins, auto_merge, custom_rules
AUTO_RESOLVE_THRESHOLD=0.95
REQUIRE_MANUAL_APPROVAL=true
```

### Development Setup
1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd spreadsheet_versioning_service
   ```

2. **Start the database**
   ```bash
   docker-compose up postgres-versioning -d
   ```

3. **Run migrations**
   ```bash
   cargo run --bin migrate
   ```

4. **Start the service**
   ```bash
   cargo run
   ```

### Docker Deployment
```bash
# Full service deployment
docker-compose up -d

# View logs
docker-compose logs -f versioning-service

# Health check
curl http://localhost:8088/health
```

## 📡 API Reference

### Version Management
```http
POST   /api/v1/versions                    # Create new version
GET    /api/v1/versions/{id}               # Get version by ID
PUT    /api/v1/versions/{id}               # Update version metadata
DELETE /api/v1/versions/{id}               # Delete version (soft delete)
```

### Spreadsheet Versioning
```http
GET    /api/v1/spreadsheets/{id}/versions           # List spreadsheet versions
POST   /api/v1/spreadsheets/{id}/versions           # Create version for spreadsheet
GET    /api/v1/spreadsheets/{id}/versions/{vid}     # Get specific version
```

### Difference Engine
```http
POST   /api/v1/diff/compare                # Compare two versions
POST   /api/v1/diff/merge                  # Merge versions
POST   /api/v1/diff/conflicts              # Detect conflicts
```

### Conflict Resolution
```http
GET    /api/v1/conflicts                   # List all conflicts
GET    /api/v1/conflicts/{id}              # Get conflict details
POST   /api/v1/conflicts/{id}/resolve      # Resolve conflict
```

## 🔄 Workflow Examples

### Basic Version Creation
```bash
# Upload new spreadsheet version
curl -X POST http://localhost:8088/api/v1/versions \
  -H "Content-Type: application/json" \
  -d '{
    "spreadsheet_id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Sample Data",
    "filename": "samples_v2.xlsx",
    "original_filename": "samples_v2.xlsx",
    "file_type": "xlsx",
    "file_data": "...base64_encoded_data...",
    "changes_summary": "Added 50 new samples"
  }'
```

### Generate Diff Between Versions
```bash
curl -X POST http://localhost:8088/api/v1/diff/compare \
  -H "Content-Type: application/json" \
  -d '{
    "from_version_id": "version-1-uuid",
    "to_version_id": "version-2-uuid",
    "diff_options": {
      "ignore_whitespace": false,
      "ignore_case": false,
      "include_metadata": true,
      "detailed_changes": true
    }
  }'
```

### Conflict Detection
```bash
curl -X POST http://localhost:8088/api/v1/diff/conflicts \
  -H "Content-Type: application/json" \
  -d '{
    "base_version_id": "base-version-uuid",
    "version_a_id": "version-a-uuid",
    "version_b_id": "version-b-uuid"
  }'
```

## 🔧 Configuration

### Diff Algorithm Options
- **cell_by_cell**: Basic cell-level comparison
- **structural_aware**: Detects row/column changes
- **semantic**: Content-aware comparison (future)

### Conflict Resolution Strategies
- **manual_review**: All conflicts require human review
- **latest_wins**: Most recent change takes precedence
- **auto_merge**: Automatic resolution for simple conflicts
- **custom_rules**: User-defined resolution logic

### Performance Tuning
```toml
# Cargo.toml optimizations
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

## 🏥 Health & Monitoring

### Health Endpoints
- `GET /health` - Basic health check
- `GET /health/ready` - Readiness probe (includes DB connectivity)

### Metrics (Planned)
- Version creation rate
- Diff generation time
- Conflict resolution success rate
- Storage utilization

## 🛡️ Security Considerations

- **Input validation** for all uploaded files
- **File size limits** to prevent DoS attacks
- **Hash verification** to ensure data integrity
- **Access control** integration with auth service
- **Audit logging** for all version operations

## 🔄 Integration

### Lab Manager Integration
The versioning service integrates with the main Lab Manager system:

```rust
// Example integration in lab_manager
use spreadsheet_versioning_client::VersioningClient;

let client = VersioningClient::new("http://versioning-service:8088");
let versions = client.list_versions(spreadsheet_id).await?;
```

### Event Integration
The service publishes events to the TracSeq event bus:
- `spreadsheet.version.created`
- `spreadsheet.version.updated`
- `spreadsheet.diff.generated`
- `spreadsheet.conflict.detected`
- `spreadsheet.conflict.resolved`

## 🧪 Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration

# Test with sample data
cargo test --test sample_data -- --nocapture

# Performance benchmarks
cargo bench
```

## 📈 Roadmap

### Phase 1 (Current)
- ✅ Basic version management
- ✅ File hash-based deduplication
- ✅ Simple diff generation
- 🔄 Conflict detection framework

### Phase 2 (Next)
- 🔄 Advanced merge strategies
- 🔄 Semantic diff algorithms
- 🔄 Real-time collaboration features
- 🔄 Version branching and tagging

### Phase 3 (Future)
- ⏳ Machine learning for intelligent merging
- ⏳ Visual diff representation
- ⏳ Integration with external version control
- ⏳ Advanced analytics and reporting

## 📞 Support

For questions or issues:
- Create an issue in the TracSeq repository
- Contact the development team
- Refer to the main TracSeq documentation

## 📄 License

This service is part of the TracSeq 2.0 laboratory management system. 
