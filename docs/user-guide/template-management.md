# Template Management

## Overview

The Template Management system allows you to create, upload, and manage spreadsheet templates for sample data collection. Templates define the structure and validation rules for sample metadata, ensuring consistent data entry across your laboratory.

## Template Structure

### Basic Template Format

Templates are defined using a structured JSON format:

```typescript
interface Template {
  name: string;           // Template name
  version: string;        // Template version
  description: string;    // Template description
  fields: TemplateField[]; // Template fields
  validation: {
    rules: ValidationRule[]; // Validation rules
    dependencies: FieldDependency[]; // Field dependencies
  };
}

interface TemplateField {
  name: string;          // Field name
  type: FieldType;       // Field type
  required: boolean;     // Required field
  description: string;   // Field description
  options?: string[];    // Options for select fields
  validation?: {
    min?: number;        // Minimum value
    max?: number;        // Maximum value
    pattern?: string;    // Regex pattern
  };
}

type FieldType = 
  | 'text'
  | 'number'
  | 'date'
  | 'select'
  | 'multiselect'
  | 'boolean';
```

### Example Template

```json
{
  "name": "DNA Sample Template",
  "version": "1.0.0",
  "description": "Template for DNA sample collection",
  "fields": [
    {
      "name": "sample_id",
      "type": "text",
      "required": true,
      "description": "Unique sample identifier"
    },
    {
      "name": "collection_date",
      "type": "date",
      "required": true,
      "description": "Date of sample collection"
    },
    {
      "name": "sample_type",
      "type": "select",
      "required": true,
      "description": "Type of DNA sample",
      "options": ["Blood", "Tissue", "Cell Culture", "Other"]
    }
  ]
}
```

## Using Templates

### Creating a New Template

1. Navigate to the Templates section
2. Click "Create New Template"
3. Fill in the template details:
   - Name
   - Version
   - Description
4. Add fields:
   - Click "Add Field"
   - Configure field properties
   - Set validation rules
5. Save the template

### Uploading a Template

1. Navigate to the Templates section
2. Click "Upload Template"
3. Select your template file (JSON or Excel format)
4. Review the template structure
5. Click "Validate" to check for errors
6. Click "Save" to store the template

### Template Validation

The system performs several validation checks:

1. **Structure Validation**
   - Required fields present
   - Correct data types
   - Valid field names

2. **Business Rules**
   - Field dependencies
   - Value ranges
   - Format requirements

3. **Cross-field Validation**
   - Related field consistency
   - Conditional requirements

## Template Operations

### Editing Templates

1. Find the template in the list
2. Click "Edit"
3. Modify fields or validation rules
4. Save changes

### Version Control

1. Create a new version:
   - Click "New Version"
   - Increment version number
   - Make changes
   - Save as new version

2. View version history:
   - Click "Version History"
   - Compare versions
   - Restore previous versions

### Template Export

1. Select a template
2. Click "Export"
3. Choose format:
   - JSON
   - Excel
   - CSV
4. Download the file

## Best Practices

1. **Template Design**
   - Use clear, descriptive field names
   - Include detailed descriptions
   - Set appropriate validation rules
   - Consider future extensibility

2. **Version Management**
   - Use semantic versioning
   - Document changes between versions
   - Maintain backward compatibility
   - Archive old versions

3. **Validation Rules**
   - Keep rules simple and clear
   - Test rules thoroughly
   - Document rule purposes
   - Consider edge cases

## Troubleshooting

### Common Issues

1. **Template Validation Errors**
   - Check field types
   - Verify required fields
   - Review validation rules
   - Check for circular dependencies

2. **Upload Problems**
   - Verify file format
   - Check file size
   - Ensure proper permissions
   - Validate JSON structure

3. **Version Conflicts**
   - Check version numbers
   - Review change history
   - Resolve conflicts
   - Update references

## Related Topics

- [Sample Processing](sample-processing.md)
- [API Documentation](../api/endpoints.md#templates)
- [Database Schema](../backend/database-schema.md#templates)

## Next Steps

1. Learn about [Sample Processing](sample-processing.md)
2. Review [API Documentation](../api/endpoints.md#templates)
3. Explore [Database Schema](../backend/database-schema.md#templates) 
