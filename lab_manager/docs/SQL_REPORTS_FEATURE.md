# SQL Reports Feature

## Overview

The SQL Reports feature allows users to write custom SQL queries and generate reports from the lab management system database. This powerful tool provides flexible data analysis capabilities while maintaining security through read-only access.

## Features

### ✅ **SQL Query Editor**
- **Interactive Query Interface**: Write and execute SQL queries with a user-friendly text editor
- **Real-time Results**: View query results immediately in a formatted table
- **Performance Metrics**: See execution time and row count for each query
- **Error Handling**: Clear error messages for invalid queries or SQL syntax issues

### ✅ **Predefined Report Templates** 
- **Sample Analytics**: 
  - Samples by Status (count grouping)
  - Recent Samples (last 30 days)
  - Sample Storage Locations (distribution analysis)
- **Template Usage**: Track which templates are used most frequently
- **One-Click Execution**: Click any template to load it into the editor

### ✅ **Database Schema Browser**
- **Complete Schema View**: Browse all tables and their columns
- **Data Type Information**: See column types, nullable status, and primary keys
- **Query Helper**: Use schema info to write accurate queries

### ✅ **Export Functionality**
- **CSV Export**: Download query results as CSV files
- **Timestamp Naming**: Files automatically named with current date
- **Data Formatting**: Proper handling of quotes and special characters

### ✅ **Security Features**
- **Read-Only Access**: Only SELECT queries are permitted
- **SQL Injection Protection**: Comprehensive validation against malicious queries
- **Query Restrictions**: No comments, multiple statements, or data modification allowed
- **Safe Keywords**: Blocks INSERT, UPDATE, DELETE, DROP, CREATE, ALTER, etc.

## API Endpoints

### Get Report Templates
```bash
GET /api/reports/templates
```
**Response:**
```json
[
  {
    "id": "samples_by_status",
    "name": "Samples by Status", 
    "description": "Count of samples grouped by status",
    "sql": "SELECT status, COUNT(*) as count FROM samples GROUP BY status ORDER BY count DESC",
    "category": "Samples"
  }
]
```

### Get Database Schema
```bash
GET /api/reports/schema
```
**Response:**
```json
{
  "tables": [
    {
      "name": "samples",
      "columns": [
        {
          "name": "id",
          "data_type": "uuid",
          "is_nullable": false,
          "is_primary_key": true
        }
      ]
    }
  ]
}
```

### Execute SQL Query
```bash
POST /api/reports/execute
Content-Type: application/json

{
  "sql": "select name, barcode from samples limit 5"
}
```
**Response:**
```json
{
  "columns": ["name", "barcode"],
  "rows": [
    {"name": "Sample 1", "barcode": "TEST-001"},
    {"name": "Sample 2", "barcode": "TEST-002"}
  ],
  "row_count": 2,
  "execution_time_ms": 5,
  "query": "select name, barcode from samples limit 5"
}
```

## Database Schema

### Available Tables

#### **samples**
- `id` (UUID, Primary Key) - Unique sample identifier
- `name` (VARCHAR) - Sample name
- `barcode` (VARCHAR) - Unique barcode
- `location` (VARCHAR) - Storage location
- `status` (ENUM) - Sample status (pending, validated, in_storage, in_sequencing, completed)
- `created_at` (TIMESTAMP) - Creation timestamp
- `updated_at` (TIMESTAMP) - Last update timestamp
- `metadata` (JSONB) - Additional sample data

#### **templates**
- `id` (UUID, Primary Key) - Unique template identifier
- `name` (VARCHAR) - Template name
- `description` (TEXT) - Template description
- `file_path` (VARCHAR) - Storage path for template file
- `file_type` (VARCHAR) - File type (xlsx, csv, etc.)
- `created_at` (TIMESTAMP) - Creation timestamp
- `updated_at` (TIMESTAMP) - Last update timestamp
- `metadata` (JSONB) - Template configuration data

#### **sequencing_jobs**
- `id` (UUID, Primary Key) - Unique job identifier
- `name` (VARCHAR) - Job name
- `status` (ENUM) - Job status (pending, in_progress, completed, failed)
- `sample_sheet_path` (VARCHAR) - Path to sample sheet
- `created_at` (TIMESTAMP) - Creation timestamp
- `updated_at` (TIMESTAMP) - Last update timestamp
- `metadata` (JSONB) - Job configuration data

## Example Queries

### Basic Sample Analysis
```sql
-- Count samples by status
select status, count(*) as sample_count 
from samples 
group by status 
order by sample_count desc;

-- Recent sample activity
select name, barcode, location, created_at 
from samples 
where created_at >= now() - interval '7 days' 
order by created_at desc;
```

### Advanced Analytics
```sql
-- Template usage statistics
select 
    t.name as template_name,
    count(s.id) as samples_created,
    max(s.created_at) as last_used
from templates t
left join samples s on s.metadata->>'template_name' = t.name
group by t.id, t.name
order by samples_created desc;

-- Storage location utilization
select 
    location,
    count(*) as sample_count,
    count(case when status = 'pending' then 1 end) as pending,
    count(case when status = 'completed' then 1 end) as completed
from samples
group by location
order by sample_count desc;
```

### JSONB Metadata Queries
```sql
-- Query sample metadata
select 
    name,
    metadata->>'template_name' as template,
    metadata->>'batch_number' as batch
from samples
where metadata ? 'template_name';

-- Complex metadata filtering
select name, barcode, metadata
from samples
where metadata @> '{"batch_test": true}';
```

## Frontend Usage

### Accessing Reports
1. Navigate to **http://localhost:5173/reports**
2. Use the navigation sidebar: **Reports** (chart icon)

### Query Editor Tab
1. **Write SQL**: Enter your SELECT query in the text area
2. **Execute**: Click "Execute Query" button
3. **View Results**: Results appear in a formatted table below
4. **Export**: Click "Export CSV" to download results

### Templates Tab
1. **Browse Categories**: Templates are grouped by category (Samples, Templates, Storage)
2. **Preview**: See query preview in each template card
3. **Use Template**: Click any template to load it into the editor

### Schema Tab
1. **Browse Tables**: View all available database tables
2. **Column Details**: See column names, types, nullable status, and primary keys
3. **Reference**: Use this information to write accurate queries

## Security Considerations

### Query Validation
- **Whitelist Approach**: Only SELECT statements allowed
- **Keyword Filtering**: Blocks data modification keywords (INSERT, UPDATE, DELETE, etc.)
- **Comment Blocking**: SQL comments (-- and /* */) not permitted
- **Single Statement**: Multiple statements separated by semicolons blocked

### Safe Practices
- **Read-Only Database User**: Consider using a read-only database user for reports
- **Query Timeouts**: Long-running queries are automatically terminated
- **Resource Limits**: Implement row limits for large result sets
- **Audit Logging**: All executed queries are logged for security auditing

## Performance Optimization

### Query Best Practices
- **Use LIMIT**: Always limit results for large datasets
- **Index Usage**: Leverage existing indexes on id, created_at, status columns
- **Avoid SELECT ***: Specify only needed columns
- **Filter Early**: Use WHERE clauses to reduce dataset size

### System Monitoring
- **Execution Time**: Monitor query execution times (typical <50ms)
- **Resource Usage**: Track CPU and memory consumption
- **Concurrent Users**: Limit simultaneous report executions if needed

## Troubleshooting

### Common Issues

**"Only SELECT queries are allowed"**
- Ensure query starts with lowercase "select"
- Remove any INSERT, UPDATE, DELETE keywords
- Check for comments in query

**Parse Errors**
- Verify SQL syntax is correct
- Check for unmatched quotes or parentheses
- Ensure table and column names exist

**Empty Results**
- Verify table contains data: `select count(*) from table_name`
- Check WHERE clause conditions
- Confirm column names match schema

### Contact Support
For additional help with SQL queries or reporting issues:
- Check the database schema tab for available tables/columns
- Use predefined templates as starting points
- Refer to PostgreSQL documentation for advanced SQL features

## Future Enhancements

### Planned Features
- **Syntax Highlighting**: SQL syntax highlighting in the editor
- **Query History**: Save and retrieve previously executed queries
- **Scheduled Reports**: Automatically run reports on a schedule
- **Charts/Visualizations**: Generate charts directly from query results
- **Shared Reports**: Save and share reports with other users
- **Query Builder**: Visual query builder for non-SQL users

### API Extensions
- **Saved Queries**: Store frequently used queries
- **Report Subscriptions**: Email reports on schedule
- **Data Export**: Additional export formats (Excel, PDF)
- **Query Optimization**: Automatic query performance suggestions

*Context improved by Giga AI* 
