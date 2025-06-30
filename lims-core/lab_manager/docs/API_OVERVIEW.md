# 🔧 API Overview - Lab Manager REST API

## Base Information

**Base URL**: `http://localhost:3000/api`  
**API Version**: v1  
**Authentication**: JWT Bearer tokens  
**Content-Type**: `application/json`

## 🔐 Authentication

### Authentication Flow
```bash
# 1. Login to get JWT token
POST /api/auth/login
{
  "email": "user@lab.local",
  "password": "password"
}

# Response includes access_token
{
  "success": true,
  "data": {
    "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
    "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
    "user": { ... }
  }
}

# 2. Use token in subsequent requests
Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...
```

### Authentication Endpoints
```bash
POST   /api/auth/login              # User login
POST   /api/auth/logout             # User logout  
POST   /api/auth/refresh            # Refresh access token
GET    /api/auth/me                 # Get current user
PUT    /api/auth/password           # Change password
POST   /api/auth/password/reset     # Request password reset
POST   /api/auth/password/confirm   # Confirm password reset
```

## 🧪 Sample Management API

### Core Sample Operations
```bash
# Sample CRUD operations
GET    /api/samples                 # List samples with pagination/filtering
POST   /api/samples                 # Create new sample
GET    /api/samples/{id}            # Get sample details
PUT    /api/samples/{id}            # Update sample
DELETE /api/samples/{id}            # Delete sample

# Sample state management  
PUT    /api/samples/{id}/state      # Update sample state
GET    /api/samples/{id}/history    # Get sample state history
POST   /api/samples/{id}/validate   # Validate sample data

# Batch operations
POST   /api/samples/batch           # Create multiple samples
PUT    /api/samples/batch/state     # Update multiple sample states
```

### Sample Search & Filtering
```bash
# Advanced search
GET    /api/samples/search?q=LAB001&status=validated&storage_temp=-80
GET    /api/samples/search?barcode=LAB-20240101-001
GET    /api/samples/search?project_id=proj-123&date_from=2024-01-01

# Filter options
GET    /api/samples?limit=50&offset=0&order_by=created_at&order=desc
GET    /api/samples?status=in_storage&temperature_zone=-80c
GET    /api/samples?department=oncology&created_by=user-123
```

### Sample Validation
```bash
# Validation endpoints
POST   /api/samples/validate        # Validate sample data before creation
GET    /api/samples/validation/rules # Get validation rules
POST   /api/samples/{id}/revalidate # Re-run validation on existing sample
```

## 🏪 Storage Management API

### Storage Location Management
```bash
# Storage location CRUD
GET    /api/storage/locations       # List storage locations
POST   /api/storage/locations       # Create storage location
GET    /api/storage/locations/{id}  # Get location details
PUT    /api/storage/locations/{id}  # Update location
DELETE /api/storage/locations/{id}  # Delete location

# Capacity and utilization
GET    /api/storage/capacity        # Get capacity overview
GET    /api/storage/locations/{id}/capacity # Get location capacity
GET    /api/storage/utilization     # System-wide utilization stats
```

### Sample Storage Operations
```bash
# Sample storage assignments
POST   /api/storage/assign          # Assign sample to location
PUT    /api/storage/move            # Move sample between locations
DELETE /api/storage/remove          # Remove sample from storage

# Storage queries
GET    /api/storage/locations/{id}/samples  # Get samples in location
GET    /api/storage/samples/{id}/location   # Get sample's location
GET    /api/storage/samples/{id}/history    # Get sample movement history
```

### Temperature Zone Management
```bash
# Temperature zone operations
GET    /api/storage/temperature-zones       # List temperature zones
GET    /api/storage/temperature-zones/{zone}/locations # Locations by zone
GET    /api/storage/temperature-zones/{zone}/capacity  # Zone capacity stats
```

## 📊 Spreadsheet Processing API

### File Upload & Processing
```bash
# Spreadsheet upload and processing
POST   /api/spreadsheets/upload     # Upload spreadsheet file
GET    /api/spreadsheets/datasets   # List processed datasets  
GET    /api/spreadsheets/datasets/{id} # Get dataset details
DELETE /api/spreadsheets/datasets/{id} # Delete dataset

# File processing status
GET    /api/spreadsheets/datasets/{id}/status # Get processing status
POST   /api/spreadsheets/datasets/{id}/reprocess # Reprocess dataset
```

### Data Search & Export
```bash
# Search across spreadsheet data
GET    /api/spreadsheets/search     # Search data across all uploads
GET    /api/spreadsheets/search?dataset_id={id}&search_term=LAB001
GET    /api/spreadsheets/search?filter_Department=Oncology&limit=50

# Data export
GET    /api/spreadsheets/datasets/{id}/export?format=csv
GET    /api/spreadsheets/datasets/{id}/export?format=json
POST   /api/spreadsheets/search/export # Export search results
```

### Template Management
```bash
# Template operations  
GET    /api/templates               # List templates
POST   /api/templates               # Create template
GET    /api/templates/{id}          # Get template details
PUT    /api/templates/{id}          # Update template
DELETE /api/templates/{id}          # Delete template

# Template validation
POST   /api/templates/validate      # Validate template structure
GET    /api/templates/{id}/samples  # Get samples created from template
```

## 🤖 RAG Integration API

### Document Processing
```bash
# RAG document processing
POST   /api/rag/documents           # Upload document for processing
GET    /api/rag/documents/{id}      # Get processing status
GET    /api/rag/documents/{id}/results # Get extracted data

# Confidence scoring
GET    /api/rag/documents/{id}/confidence # Get confidence scores
POST   /api/rag/documents/{id}/review     # Review/approve extracted data
```

### AI Model Management
```bash
# Model operations
GET    /api/rag/models              # List available models
POST   /api/rag/models/download     # Download new model
GET    /api/rag/health              # Check RAG service health
```

## 👥 User Management API

### User CRUD Operations
```bash
# User management (admin only)
GET    /api/users                   # List users
POST   /api/users                   # Create user
GET    /api/users/{id}              # Get user details  
PUT    /api/users/{id}              # Update user
DELETE /api/users/{id}              # Delete user

# User roles and permissions
GET    /api/users/{id}/permissions  # Get user permissions
PUT    /api/users/{id}/role         # Update user role
GET    /api/roles                   # List available roles
```

### Session Management
```bash
# Session operations
GET    /api/users/{id}/sessions     # Get user sessions
DELETE /api/users/{id}/sessions/{session_id} # Revoke session
DELETE /api/users/{id}/sessions     # Revoke all sessions
```

## 📊 Reports & Analytics API

### System Reports
```bash
# System-wide reports
GET    /api/reports/samples         # Sample statistics
GET    /api/reports/storage         # Storage utilization
GET    /api/reports/activity        # User activity report
GET    /api/reports/audit           # Audit trail

# Custom SQL reports
POST   /api/reports/sql             # Execute custom SQL query
GET    /api/reports/templates       # List report templates
```

### Export & Scheduling
```bash
# Report export
GET    /api/reports/{id}/export?format=pdf
GET    /api/reports/{id}/export?format=excel
GET    /api/reports/{id}/export?format=csv

# Scheduled reports
GET    /api/reports/scheduled       # List scheduled reports
POST   /api/reports/schedule        # Schedule report
DELETE /api/reports/scheduled/{id}  # Cancel scheduled report
```

## 🔧 System Administration API

### Health & Monitoring
```bash
# System health
GET    /api/health                  # Basic health check
GET    /api/health/detailed         # Detailed system status
GET    /api/health/database         # Database connection status
GET    /api/health/storage          # Storage system status
GET    /api/health/rag              # RAG service status

# System metrics
GET    /api/metrics                 # Performance metrics
GET    /api/metrics/database        # Database performance
GET    /api/metrics/storage         # Storage metrics
```

### Configuration
```bash
# System configuration
GET    /api/config                  # Get system configuration
PUT    /api/config                  # Update configuration
GET    /api/config/validation       # Get validation rules
PUT    /api/config/validation       # Update validation rules
```

## 📋 Request/Response Examples

### Create Sample
```bash
POST /api/samples
Content-Type: application/json
Authorization: Bearer {token}

{
  "name": "Blood Sample 001",
  "barcode": "BLD-20240101-001",
  "sample_type": "blood",
  "volume": 5.0,
  "concentration": 25.5,
  "temperature_requirement": "-80c",
  "metadata": {
    "patient_id": "PAT-001",
    "collection_date": "2024-01-01",
    "department": "Oncology"
  }
}
```

### Response
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Blood Sample 001",
    "barcode": "BLD-20240101-001",
    "status": "validated",
    "created_at": "2024-01-01T10:00:00Z",
    "storage_location": null,
    "metadata": { ... }
  }
}
```

### Error Response
```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_FAILED",
    "message": "Sample validation failed",
    "details": [
      {
        "field": "barcode",
        "message": "Barcode already exists"
      }
    ]
  }
}
```

## 🔒 Authorization Levels

### Role-Based Access Control
```bash
# Access levels by role
LabAdmin:     Full access to all endpoints
PI:           Read/write access to project samples
Technician:   Sample processing and storage operations  
Scientist:    Read access + sample submission
Analyst:      Read-only access to data and reports
Guest:        Limited read access
```

### Permission Matrix
| Endpoint Category | LabAdmin | PI | Technician | Scientist | Analyst | Guest |
|------------------|----------|----|-----------|-----------|---------| ------|
| User Management  | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Sample CRUD      | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| Storage Mgmt     | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| Data Search      | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Reports          | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| System Config    | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |

## 📈 Rate Limiting

### Rate Limits by Endpoint Type
```bash
# Standard endpoints: 100 requests/minute
GET    /api/samples
POST   /api/samples

# File upload endpoints: 10 requests/minute  
POST   /api/spreadsheets/upload
POST   /api/rag/documents

# Authentication endpoints: 5 requests/minute
POST   /api/auth/login
POST   /api/auth/password/reset

# Admin endpoints: 50 requests/minute
POST   /api/users
PUT    /api/config
```

## 🔧 SDK & Integration

### Official SDK Libraries
```bash
# Python SDK
pip install lab-manager-sdk
from lab_manager import LabManagerClient

# JavaScript/Node.js SDK  
npm install @lab-manager/sdk
import { LabManagerClient } from '@lab-manager/sdk'

# Rust SDK
cargo add lab-manager-client
use lab_manager_client::Client;
```

### Webhook Support
```bash
# Configure webhooks for events
POST   /api/webhooks               # Create webhook
GET    /api/webhooks               # List webhooks  
DELETE /api/webhooks/{id}          # Delete webhook

# Supported events
sample.created, sample.updated, sample.state_changed
storage.assigned, storage.moved, storage.capacity_warning
user.created, user.login, user.permission_changed
```

---

*For detailed endpoint documentation, see the [OpenAPI specification](./openapi.yaml) or visit the interactive API docs at `/docs` when running the development server.*

*Context added by Giga sample-processing-algorithms* 
