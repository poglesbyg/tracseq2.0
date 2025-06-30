# Sample Editing Feature

## Overview

The sample editing feature allows users to modify existing samples in the lab management system. This includes updating sample metadata, changing status, modifying locations, and editing barcodes.

## Backend Implementation

### New API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/samples/:id` | Retrieve a single sample by ID |
| `PUT` | `/api/samples/:id` | Update a sample by ID |

### Data Models

#### UpdateSample Structure
```rust
#[derive(Debug, Deserialize, Clone)]
pub struct UpdateSample {
    pub name: Option<String>,
    pub barcode: Option<String>,
    pub location: Option<String>,
    pub status: Option<SampleStatus>,
    pub metadata: Option<serde_json::Value>,
}
```

#### Sample Status Options
- `Pending` - Initial state
- `Validated` - Sample has been validated
- `InStorage` - Sample is stored
- `InSequencing` - Sample is being sequenced
- `Completed` - Sample processing complete

### Backend Features

✅ **Partial Updates** - Only sends changed fields to optimize database operations
✅ **Dynamic Query Building** - Constructs SQL queries based on provided fields
✅ **Timestamp Management** - Automatically updates `updated_at` timestamp
✅ **Error Handling** - Comprehensive error handling with meaningful messages

## Frontend Implementation

### Sample Edit Modal

The editing interface provides:

- **Form Validation** - Real-time validation with error messages
- **Status Selection** - Dropdown with color-coded status options
- **Change Detection** - Only submits if changes are made
- **Loading States** - Shows progress during API calls
- **Sample Information** - Displays read-only metadata (ID, timestamps)

### UI Features

✅ **Responsive Design** - Works on mobile and desktop
✅ **Real-time Updates** - Automatically refreshes sample list and dashboard
✅ **Error Handling** - User-friendly error messages
✅ **Accessibility** - Proper labels and keyboard navigation

## Usage Examples

### API Usage

#### Update Sample Name and Location
```bash
curl -X PUT http://localhost:3000/api/samples/{id} \
  -H "Content-Type: application/json" \
  -d '{"name": "New Sample Name", "location": "Lab Room A"}'
```

#### Update Sample Status
```bash
curl -X PUT http://localhost:3000/api/samples/{id} \
  -H "Content-Type: application/json" \
  -d '{"status": "Validated"}'
```

#### Partial Update (Only Changed Fields)
```bash
curl -X PUT http://localhost:3000/api/samples/{id} \
  -H "Content-Type: application/json" \
  -d '{"barcode": "NEW-001"}'
```

### Frontend Usage

1. **Navigate to Samples Page** - Visit `/samples`
2. **Click Edit Button** - Click "Edit" on any sample row
3. **Modify Fields** - Update name, barcode, location, or status
4. **Save Changes** - Click "Save Changes" to apply updates
5. **View Updates** - See changes reflected immediately in the table

## Technical Details

### Database Operations

The update functionality uses dynamic SQL query construction:

```rust
// Example: Only updating name and status
UPDATE samples 
SET name = $1, status = $2, updated_at = NOW()
WHERE id = $3
RETURNING id, name, barcode, location, status, created_at, updated_at, metadata
```

### Frontend State Management

- **React Query** for API state management and caching
- **Local State** for form data and validation errors
- **Optimistic Updates** for better user experience

### Integration Points

✅ **Dashboard Integration** - Stats update automatically after edits
✅ **Template Integration** - Maintains template relationships
✅ **Storage Integration** - Location changes tracked
✅ **Sequencing Integration** - Status changes affect sequencing workflows

## Testing

### API Testing
```bash
# Run the demo script
./scripts/demo_sample_editing.sh

# Test individual endpoints
curl -s http://localhost:3000/api/samples/{id}
curl -X PUT http://localhost:3000/api/samples/{id} -d '{...}'
```

### Frontend Testing
1. Visit `http://localhost:5173/samples`
2. Click "Edit" on any sample
3. Modify fields and save
4. Verify changes in table and dashboard

## Future Enhancements

### Planned Features
- **Bulk Editing** - Edit multiple samples simultaneously
- **Field History** - Track changes over time
- **Advanced Validation** - Custom validation rules
- **Audit Trail** - Log all modifications
- **Conditional Fields** - Show/hide fields based on status

### Integration Opportunities
- **Barcode Scanning** - Direct barcode updates via scanner
- **Template Constraints** - Validate against template requirements
- **Workflow Automation** - Auto-status updates based on actions
- **Notification System** - Alert users of important changes

## Security Considerations

✅ **Input Validation** - All fields validated on frontend and backend
✅ **SQL Injection Prevention** - Using parameterized queries
✅ **Error Handling** - No sensitive information in error messages
⚠️ **Authentication** - To be implemented in future versions
⚠️ **Authorization** - Role-based permissions needed

## Performance

- **Optimized Queries** - Only updates changed fields
- **Caching** - React Query caches results
- **Minimal Payloads** - Only sends necessary data
- **Batch Updates** - Database operations are efficient

## Troubleshooting

### Common Issues

1. **Sample Not Found** - Verify sample ID exists
2. **Validation Errors** - Check required fields
3. **Network Errors** - Ensure backend is running
4. **Permission Errors** - Check API access

### Debug Commands
```bash
# Check if sample exists
curl -s http://localhost:3000/api/samples/{id}

# Test connection
curl -s http://localhost:3000/health

# View all samples
curl -s http://localhost:3000/api/samples | jq .
``` 
