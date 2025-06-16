# Lab Manager Database Integration Guide

This guide explains how to integrate the Laboratory Submission RAG system with your existing `lab_manager` application database.

## Overview

The RAG system has been designed to work alongside your existing lab_manager application by:
- Using table prefixes (`rag_` by default) to avoid naming conflicts
- Connecting to the same PostgreSQL database
- Providing APIs that can complement your existing lab management workflow

## Prerequisites

- Your lab_manager application running via Docker
- PostgreSQL database accessible from Docker containers
- Network connectivity between containers

## Integration Steps

### 1. Configure Database Connection

#### Option A: Using Docker Compose Override (Recommended)

Create a `.env` file with your lab_manager database credentials:

```bash
# Lab Manager Database Configuration
LAB_MANAGER_DB_HOST=lab_manager_postgres_1  # Your postgres container name
LAB_MANAGER_DB_PORT=5432
LAB_MANAGER_DB_NAME=lab_manager              # Your database name
LAB_MANAGER_DB_USER=your_username
LAB_MANAGER_DB_PASSWORD=your_password
RAG_TABLE_PREFIX=rag_                        # Prefix to avoid table conflicts
```

Then start the RAG system with the override file:

```bash
docker-compose -f docker-compose.yml -f docker-compose.override.yml up -d
```

#### Option B: Connect to External Database

If your lab_manager database is not in Docker, update the environment variables:

```bash
# External Database Configuration
DATABASE_URL=postgresql+asyncpg://user:password@your-db-host:5432/lab_manager
DATABASE_HOST=your-db-host
DATABASE_PORT=5432
DATABASE_NAME=lab_manager
DATABASE_USER=your_username
DATABASE_PASSWORD=your_password
TABLE_PREFIX=rag_
```

### 2. Analyze Existing Schema

Run the integration script to analyze your existing database:

```bash
# From within the rag-service container or locally
python scripts/integrate_lab_manager.py
```

This script will:
- Check database connectivity
- List existing tables
- Identify potential conflicts
- Suggest integration strategies
- Optionally create RAG tables

### 3. Network Configuration

#### Connect to Lab Manager Docker Network

Find your lab_manager network:

```bash
docker network ls | grep lab_manager
```

Update the `docker-compose.override.yml` with the correct network name:

```yaml
networks:
  lab_manager_default:  # Replace with your actual network name
    external: true
```

#### Link to PostgreSQL Container

Find your lab_manager PostgreSQL container:

```bash
docker ps | grep postgres
```

Update the external links in `docker-compose.override.yml`:

```yaml
external_links:
  - your_postgres_container_name:postgres
```

### 4. Table Structure

The RAG system creates the following tables with the specified prefix:

- `rag_lab_submissions` - RAG-processed laboratory submissions
- `rag_samples` - Individual samples from RAG extraction
- `rag_documents` - Uploaded and processed documents
- `rag_document_chunks` - Text chunks for vector search
- `rag_extraction_results` - LLM extraction results
- `rag_query_logs` - User query analytics
- `rag_pooling_info` - Sample pooling information
- `rag_sequence_generation` - Sequencing parameters
- `rag_informatics_info` - Analysis requirements

### 5. API Integration

The RAG system provides APIs that can complement your lab_manager:

#### Sample Information APIs

```bash
# Get sample count
GET /samples/count?sample_type=dna&storage_condition=frozen

# Get sample statistics
GET /samples/statistics

# Search samples
GET /samples/search?q=patient123

# List submissions
GET /submissions?status=completed&limit=50
```

#### Query Interface

```bash
# Natural language queries
POST /query
{
  "query": "How many DNA samples are stored frozen?",
  "session_id": "user123"
}
```

#### Database Status

```bash
# Check database connection and statistics
GET /database/status
```

### 6. Data Synchronization (Optional)

If you want to sync data between lab_manager and the RAG system, you can:

#### Create Database Views

```sql
-- Example: Create a view to access lab_manager samples from RAG system
CREATE VIEW rag_lab_manager_samples AS
SELECT 
    id,
    sample_id,
    patient_id,
    sample_type,
    created_at
FROM lab_manager_samples;  -- Replace with your actual table name
```

#### Create Triggers

```sql
-- Example: Trigger to sync new samples to RAG system
CREATE OR REPLACE FUNCTION sync_to_rag()
RETURNS TRIGGER AS $$
BEGIN
    -- Insert logic to sync data to RAG tables
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER sync_samples_to_rag
    AFTER INSERT ON lab_manager_samples
    FOR EACH ROW
    EXECUTE FUNCTION sync_to_rag();
```

## Configuration Options

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `postgresql+asyncpg://user:password@host.docker.internal:5432/lab_manager` | Full database connection URL |
| `DATABASE_HOST` | `host.docker.internal` | Database host |
| `DATABASE_PORT` | `5432` | Database port |
| `DATABASE_NAME` | `lab_manager` | Database name |
| `DATABASE_USER` | `user` | Database username |
| `DATABASE_PASSWORD` | `password` | Database password |
| `TABLE_PREFIX` | `rag_` | Prefix for RAG tables |
| `DATABASE_SCHEMA` | `None` | Optional schema name |

### Table Prefix Configuration

To avoid conflicts with existing tables, the RAG system uses a configurable table prefix:

```bash
# Change the prefix if needed
TABLE_PREFIX=lab_rag_
# or
TABLE_PREFIX=ai_
```

## Troubleshooting

### Common Issues

1. **Connection Refused**
   - Check if lab_manager PostgreSQL is accessible
   - Verify network connectivity between containers
   - Ensure correct host and port configuration

2. **Table Conflicts**
   - Change the `TABLE_PREFIX` environment variable
   - Run the integration script to check conflicts

3. **Permission Denied**
   - Ensure the database user has CREATE TABLE permissions
   - Check if the database exists

4. **Network Issues**
   - Verify Docker network configuration
   - Check external links in docker-compose.override.yml

### Debugging Commands

```bash
# Check container logs
docker-compose logs rag-service

# Test database connection
docker-compose exec rag-service python scripts/integrate_lab_manager.py

# Check network connectivity
docker-compose exec rag-service ping lab_manager_postgres_1

# Inspect database
docker-compose exec rag-service psql -h postgres -U user -d lab_manager -c "\dt"
```

## Security Considerations

1. **Database Credentials**: Store sensitive credentials in environment files, not in code
2. **Network Isolation**: Use Docker networks to isolate database access
3. **User Permissions**: Create a dedicated database user for the RAG system with minimal required permissions
4. **SSL/TLS**: Enable SSL connections for production databases

## Performance Optimization

1. **Connection Pooling**: Adjust `DATABASE_POOL_SIZE` and `DATABASE_MAX_OVERFLOW`
2. **Indexing**: Add indexes on frequently queried columns
3. **Query Optimization**: Monitor slow queries and optimize as needed

## Support

If you encounter issues during integration:

1. Run the integration script for diagnostics
2. Check the troubleshooting section
3. Review container logs for error messages
4. Ensure all prerequisites are met

The RAG system is designed to coexist peacefully with your lab_manager application while providing enhanced document processing and intelligent querying capabilities. 
