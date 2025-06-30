# Getting Started with Lab Manager

## Overview

Lab Manager is a comprehensive scientific sample management system that helps you manage templates, samples, sequencing jobs, and storage locations. This guide will help you get started with the basic features.

## Prerequisites

Before you begin, ensure you have:
- Docker and Docker Compose installed
- Required ports available (80, 3000, 3001, 5173, 5432)
- Basic understanding of laboratory workflows

## Installation

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd lab_manager
   ```

2. Run the setup script:
   ```bash
   ./scripts/run.sh
   ```

3. Access the application:
   - Frontend: http://localhost
   - API: http://localhost/api

## Basic Workflow

### 1. Template Management

1. Navigate to the Templates section
2. Upload a new template using the provided format
3. Review and validate the template
4. Save the template for future use

### 2. Sample Submission

1. Go to the Samples section
2. Click "New Sample"
3. Fill in the required information
4. Generate barcodes
5. Submit the sample

### 3. Storage Management

1. Access the Storage section
2. Scan or enter barcodes
3. Assign storage locations
4. Track sample movements

### 4. Sequencing Jobs

1. Navigate to Sequencing
2. Create a new job
3. Select samples
4. Configure sequencing parameters
5. Submit the job

## Common Tasks

### Uploading a Template

```typescript
// Example template format
interface Template {
  name: string;
  fields: {
    name: string;
    type: string;
    required: boolean;
  }[];
}
```

### Creating a Sample

```typescript
// Example sample creation
const sample = {
  name: "Sample-001",
  type: "DNA",
  source: "Patient-123",
  storage: {
    location: "Freezer-A",
    position: "Shelf-1"
  }
};
```

## Troubleshooting

### Common Issues

1. **Port Conflicts**
   - Check if required ports are available
   - Use `lsof -i :<port>` to check port usage

2. **Database Connection**
   - Verify PostgreSQL is running
   - Check database credentials in .env

3. **File Upload Issues**
   - Verify file format
   - Check file size limits
   - Ensure proper permissions

## Related Topics

- [Template Management](template-management.md)
- [Sample Processing](sample-processing.md)
- [Storage Management](storage-management.md)
- [Sequencing Management](sequencing-management.md)

## Next Steps

1. Review the [Template Management](template-management.md) guide
2. Learn about [Sample Processing](sample-processing.md)
3. Explore [Storage Management](storage-management.md)
4. Understand [Sequencing Management](sequencing-management.md) 
