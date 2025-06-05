# Template Editing Feature

## Overview

The template editing feature allows users to modify existing templates in the lab management system. This includes updating template names, descriptions, and metadata while preserving the original file data and relationships.

## Backend Implementation

### New API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/templates/:id` | Retrieve a single template by ID |
| `PUT` | `/api/templates/:id` | Update a template by ID |

### Data Models

#### UpdateTemplate Structure
```rust
#[derive(Debug, Deserialize, Clone)]
pub struct UpdateTemplate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}
```

#### Template Fields
- `name` - Template display name
- `description` - Optional template description
- `metadata` - JSON metadata including file information
- `file_path` - Physical file location (immutable)
- `file_type` - File type (.csv, .xlsx, etc.) (immutable)

### Backend Features

✅ **Partial Updates** - Only sends changed fields to optimize database operations
✅ **Dynamic Query Building** - Constructs SQL queries based on provided fields
✅ **Timestamp Management** - Automatically updates `updated_at` timestamp
✅ **File Preservation** - Original files remain unchanged during edits
✅ **Type Safety** - Full Rust type checking with UUIDs

## Frontend Implementation

### Template Edit Modal

The editing interface provides:

- **Form Validation** - Real-time validation with error messages
- **Change Detection** - Only submits if changes are made
- **Loading States** - Shows progress during API calls
- **Template Information** - Displays read-only metadata (ID, file info, timestamps)
- **Responsive Design** - Works on all screen sizes

### UI Features

✅ **Professional Interface** - Clean, modern design matching system aesthetics
✅ **Real-time Updates** - Automatically refreshes template list and dashboard
✅ **Error Handling** - User-friendly error messages and validation
✅ **Accessibility** - Proper labels and keyboard navigation

## Usage Examples

### API Usage

#### Update Template Name and Description
```bash
curl -X PUT http://localhost:3000/api/templates/{id} \
  -H "Content-Type: application/json" \
  -d '{"name": "New Template Name", "description": "Updated description"}'
```

#### Update Only Template Name
```bash
curl -X PUT http://localhost:3000/api/templates/{id} \
  -H "Content-Type: application/json" \
  -d '{"name": "New Template Name"}'
```

#### Update Only Description
```bash
curl -X PUT http://localhost:3000/api/templates/{id} \
  -H "Content-Type: application/json" \
  -d '{"description": "New description"}'
```

#### Get Single Template
```bash
curl -s http://localhost:3000/api/templates/{id}
```

### Frontend Usage

1. **Navigate to Templates Page** - Visit `/templates`
2. **Click Edit Button** - Click "Edit" on any template row
3. **Modify Fields** - Update name and/or description
4. **Save Changes** - Click "Save Changes" to apply updates
5. **View Updates** - See changes reflected immediately in the table

## Technical Details

### Database Operations

The update functionality uses dynamic SQL query construction:

```rust
// Example: Only updating name and description
UPDATE templates 
SET name = $1, description = $2, updated_at = NOW()
WHERE id = $3
RETURNING *
```

### Repository Pattern

The template editing leverages the existing repository pattern:

```rust
// Template service method
pub async fn update_template(
    &self, 
    template_id: Uuid, 
    updates: UpdateTemplate
) -> Result<Template, R::Error> {
    self.repository.update(template_id, updates).await
}
```

### Frontend State Management

- **React Query** for API state management and caching
- **Local State** for form data and validation errors
- **Optimistic Updates** for better user experience
- **Error Boundaries** for graceful error handling

### Integration Points

✅ **Dashboard Integration** - Template stats update automatically after edits
✅ **Sample Integration** - Maintains template-sample relationships
✅ **File System Integration** - Preserves original file data
✅ **Barcode Integration** - Template metadata preserved for barcode generation

## Template Data Preservation

### What Gets Updated
- ✅ Template name
- ✅ Template description  
- ✅ Custom metadata fields
- ✅ Updated timestamp

### What Stays Unchanged
- 🔒 Template ID (immutable)
- 🔒 Original file data
- 🔒 File path
- 🔒 File type
- 🔒 Creation timestamp
- 🔒 Spreadsheet parsing results

## Testing

### API Testing
```bash
# Run the demo script
./scripts/demo_template_editing.sh

# Test individual endpoints
curl -s http://localhost:3000/api/templates/{id}
curl -X PUT http://localhost:3000/api/templates/{id} -d '{...}'
```

### Frontend Testing
1. Visit `http://localhost:5173/templates`
2. Click "Edit" on any template
3. Modify fields and save
4. Verify changes in table and dashboard

## Future Enhancements

### Planned Features
- **Bulk Editing** - Edit multiple templates simultaneously
- **Field History** - Track changes over time
- **Template Versioning** - Maintain template versions
- **Advanced Validation** - Custom validation rules
- **Template Duplication** - Clone templates with modifications

### Integration Opportunities
- **File Re-upload** - Replace template files while preserving metadata
- **Template Merging** - Combine multiple templates
- **Validation Rules** - Add custom template validation
- **Import/Export** - Template metadata import/export

## Security Considerations

✅ **Input Validation** - All fields validated on frontend and backend
✅ **SQL Injection Prevention** - Using parameterized queries
✅ **File System Security** - Original files protected from modification
✅ **Error Handling** - No sensitive information in error messages
⚠️ **Authentication** - To be implemented in future versions
⚠️ **Authorization** - Role-based permissions needed

## Performance

- **Optimized Queries** - Only updates changed fields
- **Caching** - React Query caches results
- **Minimal Payloads** - Only sends necessary data
- **Database Efficiency** - Dynamic query construction
- **File System** - No file operations during edits

## Troubleshooting

### Common Issues

1. **Template Not Found** - Verify template ID exists
2. **Validation Errors** - Check required fields (name cannot be empty)
3. **Network Errors** - Ensure backend is running
4. **Permission Errors** - Check API access

### Debug Commands
```bash
# Check if template exists
curl -s http://localhost:3000/api/templates/{id}

# Test connection
curl -s http://localhost:3000/health

# View all templates
curl -s http://localhost:3000/api/templates | jq .
```

## Related Documentation

- [Template Management](../user-guide/template-management.md)
- [Sample Editing](SAMPLE_EDITING_FEATURE.md)
- [API Documentation](../api/endpoints.md#templates)
- [Database Schema](../backend/database-schema.md#templates)

## Migration Notes

### Upgrading from Previous Versions

1. **Database** - No schema changes required
2. **Frontend** - New edit modal automatically available
3. **API** - New endpoints backward compatible
4. **Templates** - Existing templates fully editable

### Backward Compatibility

✅ **Existing APIs** - All existing endpoints unchanged
✅ **File Format** - No changes to supported file formats
✅ **Template Structure** - Existing templates work without modification
✅ **Sample Integration** - No impact on existing sample workflows 
