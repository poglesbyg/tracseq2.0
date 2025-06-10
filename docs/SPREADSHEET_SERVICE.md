# Spreadsheet Processing Service

The Spreadsheet Processing Service is a new component of the Lab Manager system that enables uploading, processing, and searching through CSV and Excel files containing laboratory data.

## Features

- **File Upload**: Support for CSV, XLSX, and XLS file formats
- **Data Processing**: Automatic parsing and validation of spreadsheet data
- **Database Storage**: Efficient storage with full-text search capabilities
- **Search API**: Powerful search with text queries and column filters
- **RESTful API**: Complete REST API for all operations
- **Health Monitoring**: Built-in health checks and monitoring

## Database Schema

The service creates two main tables:

### `spreadsheet_datasets`
Stores metadata about uploaded files:
- `id` - Unique dataset identifier
- `filename` - Internal filename for storage
- `original_filename` - Original uploaded filename
- `file_type` - File type (csv, xlsx, xls)
- `file_size` - Size in bytes
- `sheet_name` - Excel sheet name (if applicable)
- `total_rows` - Number of data rows
- `total_columns` - Number of columns
- `column_headers` - Array of column names
- `upload_status` - Processing status (processing, completed, failed)
- `error_message` - Error details if processing failed
- `uploaded_by` - User who uploaded the file
- `metadata` - Additional JSON metadata

### `spreadsheet_records`
Stores individual rows from spreadsheets:
- `id` - Unique record identifier
- `dataset_id` - Reference to parent dataset
- `row_number` - Row position in original file
- `row_data` - JSON object with column-value pairs
- `search_text` - Concatenated searchable text
- `created_at` - Timestamp

## API Endpoints

### Upload File
```http
POST /api/spreadsheets/upload
Content-Type: multipart/form-data

Form data:
- file: [spreadsheet file]
- sheet_name: [optional, for Excel files]
- uploaded_by: [optional, user identifier]
```

**Response:**
```json
{
  "success": true,
  "dataset": {
    "id": "uuid",
    "filename": "stored_filename.csv",
    "original_filename": "data.csv",
    "file_type": "csv",
    "total_rows": 1000,
    "total_columns": 15,
    "upload_status": "Completed"
  },
  "message": "File uploaded and processed successfully"
}
```

### Search Data
```http
GET /api/spreadsheets/search?search_term=LAB001&limit=50&offset=0
GET /api/spreadsheets/search?dataset_id=uuid&filter_Department=Oncology&filter_Priority=High
```

**Response:**
```json
{
  "success": true,
  "data": {
    "records": [
      {
        "id": "uuid",
        "dataset_id": "uuid",
        "row_number": 1,
        "row_data": {
          "Sample_ID": "LAB20240001",
          "Patient_ID": "P12345",
          "Department": "Oncology"
        },
        "created_at": "2024-01-01T00:00:00Z"
      }
    ],
    "total_count": 100,
    "dataset_info": null
  },
  "message": "Search completed successfully"
}
```

### List Datasets
```http
GET /api/spreadsheets/datasets?limit=50&offset=0
```

### Get Dataset
```http
GET /api/spreadsheets/datasets/{id}
```

### Delete Dataset
```http
DELETE /api/spreadsheets/datasets/{id}
```

### Health Check
```http
GET /api/spreadsheets/health
```

### Supported File Types
```http
GET /api/spreadsheets/supported-types
```

## Integration Example

### 1. Add to Main Application

```rust
use lab_manager::{
    models::spreadsheet::SpreadsheetDataManager,
    services::spreadsheet_service::SpreadsheetService,
    handlers::spreadsheets,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Database setup
    let pool = PgPool::connect(&database_url).await?;
    
    // Initialize spreadsheet service
    let spreadsheet_manager = SpreadsheetDataManager::new(pool.clone());
    let spreadsheet_service = SpreadsheetService::new(spreadsheet_manager);
    
    // Create router
    let app = Router::new()
        .nest("/api/spreadsheets", spreadsheets::create_router(spreadsheet_service))
        // ... other routes
        .layer(CorsLayer::permissive());
    
    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

### 2. Programmatic Usage

```rust
// Create service instance
let manager = SpreadsheetDataManager::new(pool);
let service = SpreadsheetService::new(manager);

// Upload and process a file
let dataset = service.process_upload(
    "unique_filename.csv".to_string(),
    "original.csv".to_string(),
    file_data_bytes,
    "csv".to_string(),
    None, // sheet_name
    Some("user@lab.com".to_string()),
).await?;

// Search for data
let query = SpreadsheetSearchQuery {
    search_term: Some("LAB001".to_string()),
    dataset_id: None,
    column_filters: None,
    limit: Some(50),
    offset: None,
};

let results = service.search_data(query).await?;
println!("Found {} records", results.total_count);
```

## Command Line Examples

### Upload a CSV file
```bash
curl -X POST http://localhost:3000/api/spreadsheets/upload \
  -F "file=@sample_data.csv" \
  -F "uploaded_by=researcher@lab.com"
```

### Upload an Excel file with specific sheet
```bash
curl -X POST http://localhost:3000/api/spreadsheets/upload \
  -F "file=@lab_results.xlsx" \
  -F "sheet_name=Results" \
  -F "uploaded_by=analyst@lab.com"
```

### Search for specific text
```bash
curl "http://localhost:3000/api/spreadsheets/search?search_term=LAB20240001&limit=10"
```

### Filter by column values
```bash
curl "http://localhost:3000/api/spreadsheets/search?filter_Department=Oncology&filter_Priority=High&limit=25"
```

### Search within specific dataset
```bash
curl "http://localhost:3000/api/spreadsheets/search?dataset_id=550e8400-e29b-41d4-a716-446655440000&search_term=blood"
```

### List all uploaded datasets
```bash
curl "http://localhost:3000/api/spreadsheets/datasets"
```

### Get dataset details
```bash
curl "http://localhost:3000/api/spreadsheets/datasets/550e8400-e29b-41d4-a716-446655440000"
```

### Delete a dataset (and all its records)
```bash
curl -X DELETE "http://localhost:3000/api/spreadsheets/datasets/550e8400-e29b-41d4-a716-446655440000"
```

## Search Capabilities

### Full-Text Search
The service uses PostgreSQL's full-text search capabilities:
```bash
curl "http://localhost:3000/api/spreadsheets/search?search_term=oncology high priority"
```

### Column-Specific Filters
Filter by exact column values:
```bash
curl "http://localhost:3000/api/spreadsheets/search?filter_Sample_Type=Blood&filter_Storage_Temp=-80°C"
```

### Combined Search
Combine text search with column filters:
```bash
curl "http://localhost:3000/api/spreadsheets/search?search_term=rush&filter_Department=Oncology&limit=20"
```

## Performance Considerations

- **Indexing**: The service automatically creates indexes for optimal search performance
- **Bulk Operations**: Records are inserted in batches for better performance
- **Search Limits**: API calls are limited to 1000 results per query
- **File Size**: Consider implementing file size limits based on your needs

## Error Handling

The service provides comprehensive error handling:

- **File Format Errors**: Invalid or corrupted files are rejected with descriptive messages
- **Database Errors**: Connection and query errors are logged and returned as HTTP 500
- **Validation Errors**: Invalid parameters return HTTP 400 with details
- **Not Found Errors**: Missing resources return HTTP 404

## Security Considerations

- **File Validation**: All uploaded files are validated before processing
- **SQL Injection Protection**: All queries use parameterized statements
- **Input Sanitization**: User inputs are sanitized and validated
- **Error Information**: Error messages don't expose sensitive system details

## Monitoring and Health Checks

The service includes built-in health monitoring:

```bash
curl "http://localhost:3000/api/spreadsheets/health"
```

Response includes:
- Database connectivity status
- Parsing capability verification
- Response times for each check
- Overall service health status

## Migration Setup

1. Run the database migration:
```bash
cd lab_manager
sqlx migrate run
```

2. The migration creates:
   - `spreadsheet_datasets` table
   - `spreadsheet_records` table  
   - All necessary indexes for performance
   - Full-text search configuration

## Dependencies

The service requires these Cargo dependencies (already included):
- `calamine = "0.22"` - Excel file parsing
- `csv = "1.3"` - CSV parsing
- `sqlx` - Database operations
- `serde_json` - JSON handling
- `axum` - Web framework with multipart support

## Supported File Formats

- **CSV**: Comma-separated values with headers
- **XLSX**: Excel 2007+ format (multiple sheets supported)
- **XLS**: Legacy Excel format

For Excel files with multiple sheets, specify the sheet name in the upload request, or the first sheet will be used by default.

*Context improved by Giga AI* 
