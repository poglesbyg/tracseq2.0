# Sample Processing

## Overview

The Sample Processing system manages the lifecycle of laboratory samples from submission to storage. It integrates with templates for data collection, generates unique barcodes, and tracks sample status throughout the process.

## Sample Lifecycle

### 1. Sample Submission

```typescript
interface SampleSubmission {
  template_id: string;      // Reference to template
  metadata: {
    [key: string]: any;    // Template-defined fields
  };
  source: {
    type: string;          // Source type (patient, study, etc.)
    identifier: string;    // Source identifier
  };
  collection: {
    date: string;          // Collection date
    location: string;      // Collection location
    collector: string;     // Collector identifier
  };
}
```

### 2. Barcode Generation

The system automatically generates unique barcodes in the format:
```
LAB-YYYY-XXXXX
Where:
- LAB: Laboratory identifier
- YYYY: Year
- XXXXX: Sequential number
```

### 3. Status Tracking

```typescript
type SampleStatus = 
  | 'submitted'           // Initial submission
  | 'validating'          // Validation in progress
  | 'validated'           // Passed validation
  | 'rejected'            // Failed validation
  | 'stored'             // In storage
  | 'in_use'             // Currently in use
  | 'archived'           // Archived
  | 'disposed';          // Disposed
```

## Sample Submission Process

### 1. Using the Submission Wizard

1. Navigate to Samples section
2. Click "New Sample"
3. Select a template
4. Fill in required fields:
   - Source information
   - Collection details
   - Sample metadata
5. Review and submit

### 2. Batch Submission

1. Prepare Excel/CSV file:
   ```csv
   template_id,source_type,source_id,collection_date,...
   DNA-001,patient,P123,2024-03-20,...
   ```

2. Upload file:
   - Click "Batch Upload"
   - Select file
   - Map columns
   - Validate data
   - Submit batch

### 3. API Submission

```typescript
// Example API submission
const sample = {
  template_id: "DNA-001",
  metadata: {
    sample_type: "Blood",
    volume: "5ml",
    collection_date: "2024-03-20"
  },
  source: {
    type: "patient",
    identifier: "P123"
  }
};

await api.post('/samples', sample);
```

## Validation Process

### 1. Template Validation

- Checks against template structure
- Validates required fields
- Verifies data types
- Applies business rules

### 2. Business Rules

```typescript
interface ValidationRule {
  field: string;
  condition: string;
  message: string;
}

// Example rules
const rules = [
  {
    field: "collection_date",
    condition: "date <= today",
    message: "Collection date cannot be in the future"
  },
  {
    field: "volume",
    condition: "value > 0",
    message: "Volume must be positive"
  }
];
```

### 3. Cross-field Validation

- Checks relationships between fields
- Validates dependent fields
- Ensures data consistency

## Sample Tracking

### 1. Status Updates

1. Automatic updates:
   - On submission
   - After validation
   - When stored
   - During use

2. Manual updates:
   - Status changes
   - Location updates
   - Notes and comments

### 2. Location Tracking

```typescript
interface Location {
  facility: string;
  building: string;
  room: string;
  storage_unit: string;
  position: string;
  barcode: string;
}
```

### 3. History Tracking

- Status changes
- Location movements
- User actions
- Timestamps

## Best Practices

1. **Data Entry**
   - Double-check critical fields
   - Use batch upload for efficiency
   - Validate before submission
   - Document any exceptions

2. **Barcode Management**
   - Print barcodes immediately
   - Verify barcode readability
   - Keep backup records
   - Use consistent format

3. **Status Management**
   - Update status promptly
   - Document status changes
   - Review status regularly
   - Archive completed samples

## Troubleshooting

### Common Issues

1. **Validation Failures**
   - Check template requirements
   - Verify data formats
   - Review business rules
   - Check for missing fields

2. **Barcode Problems**
   - Verify barcode format
   - Check for duplicates
   - Ensure proper printing
   - Update if damaged

3. **Status Issues**
   - Verify current status
   - Check update permissions
   - Review status history
   - Contact support if stuck

## Related Topics

- [Template Management](template-management.md)
- [Storage Management](storage-management.md)
- [API Documentation](../api/endpoints.md#samples)

## Next Steps

1. Learn about [Storage Management](storage-management.md)
2. Review [API Documentation](../api/endpoints.md#samples)
3. Explore [Sequencing Management](sequencing-management.md) 
