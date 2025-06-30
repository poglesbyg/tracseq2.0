-- Add performance indexes for critical laboratory data lookups
-- Migration: 20240320000003_add_performance_indexes

-- =============================================================================
-- REQUIRED EXTENSIONS (MUST BE FIRST)
-- =============================================================================

-- Enable pg_trgm extension for fuzzy text search
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Enable pg_stat_statements extension for query performance monitoring
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- =============================================================================
-- SAMPLE MANAGEMENT INDEXES
-- =============================================================================

-- Primary sample lookup by barcode (most frequent operation)
CREATE INDEX IF NOT EXISTS idx_samples_barcode 
ON samples (barcode) 
WHERE barcode IS NOT NULL;

-- Sample ID lookups for foreign key relationships
CREATE INDEX IF NOT EXISTS idx_samples_sample_id 
ON samples (id);

-- Sample name searches (partial match support)
CREATE INDEX IF NOT EXISTS idx_samples_name_gin 
ON samples USING gin (name gin_trgm_ops) 
WHERE name IS NOT NULL;

-- Sample status filtering
CREATE INDEX IF NOT EXISTS idx_samples_status 
ON samples (status) 
WHERE status IS NOT NULL;

-- Sample location filtering (note: column is named 'location' not 'storage_location')
CREATE INDEX IF NOT EXISTS idx_samples_location 
ON samples (location) 
WHERE location IS NOT NULL;

-- Created date range queries
CREATE INDEX IF NOT EXISTS idx_samples_created_at 
ON samples (created_at);

-- Updated date for change tracking
CREATE INDEX IF NOT EXISTS idx_samples_updated_at 
ON samples (updated_at);

-- Note: Storage location index already exists as idx_samples_location above
-- Skipping duplicate storage location index

-- Composite index for common filtered queries
CREATE INDEX IF NOT EXISTS idx_samples_status_location_created 
ON samples (status, location, created_at) 
WHERE status IS NOT NULL AND location IS NOT NULL;

-- =============================================================================
-- USER MANAGEMENT INDEXES
-- =============================================================================

-- Email lookups for authentication (most critical for login performance)
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email_unique 
ON users (email);

-- User ID lookups
CREATE INDEX IF NOT EXISTS idx_users_id 
ON users (id);

-- Role-based queries
CREATE INDEX IF NOT EXISTS idx_users_role 
ON users (role) 
WHERE role IS NOT NULL;

-- User status filtering (note: column is named 'status' not 'is_active')
CREATE INDEX IF NOT EXISTS idx_users_status 
ON users (status);

-- Login tracking
CREATE INDEX IF NOT EXISTS idx_users_last_login 
ON users (last_login) 
WHERE last_login IS NOT NULL;

-- Account lockout queries
CREATE INDEX IF NOT EXISTS idx_users_locked_until 
ON users (locked_until) 
WHERE locked_until IS NOT NULL;

-- Failed login attempt tracking
CREATE INDEX IF NOT EXISTS idx_users_failed_attempts 
ON users (failed_login_attempts) 
WHERE failed_login_attempts > 0;

-- =============================================================================
-- SEQUENCING JOB INDEXES
-- =============================================================================

-- Job ID lookups
CREATE INDEX IF NOT EXISTS idx_sequencing_jobs_id 
ON sequencing_jobs (id);

-- Job status filtering
CREATE INDEX IF NOT EXISTS idx_sequencing_jobs_status 
ON sequencing_jobs (status) 
WHERE status IS NOT NULL;

-- Job name searches
CREATE INDEX IF NOT EXISTS idx_sequencing_jobs_name 
ON sequencing_jobs (name) 
WHERE name IS NOT NULL;

-- Created date for job history
CREATE INDEX IF NOT EXISTS idx_sequencing_jobs_created_at 
ON sequencing_jobs (created_at);

-- Composite index for active job queries
CREATE INDEX IF NOT EXISTS idx_sequencing_jobs_status_created 
ON sequencing_jobs (status, created_at) 
WHERE status IS NOT NULL;

-- =============================================================================
-- TEMPLATE MANAGEMENT INDEXES
-- =============================================================================

-- Template name lookups
CREATE INDEX IF NOT EXISTS idx_templates_name 
ON templates (name) 
WHERE name IS NOT NULL;

-- Template file type filtering
CREATE INDEX IF NOT EXISTS idx_templates_file_type 
ON templates (file_type) 
WHERE file_type IS NOT NULL;

-- Note: templates table doesn't have is_active column - all templates are active by default

-- =============================================================================
-- STORAGE MANAGEMENT INDEXES
-- =============================================================================

-- Storage location name lookups
CREATE INDEX IF NOT EXISTS idx_storage_locations_name 
ON storage_locations (name) 
WHERE name IS NOT NULL;

-- Container type filtering (note: column is named 'container_type' not 'storage_type')
CREATE INDEX IF NOT EXISTS idx_storage_locations_container_type 
ON storage_locations (container_type) 
WHERE container_type IS NOT NULL;

-- Capacity queries (note: columns are 'capacity' and 'current_usage')
CREATE INDEX IF NOT EXISTS idx_storage_locations_capacity 
ON storage_locations (capacity, current_usage);

-- Temperature zone filtering (note: column is named 'temperature_zone' not 'temperature_condition')
CREATE INDEX IF NOT EXISTS idx_storage_locations_temperature_zone 
ON storage_locations (temperature_zone) 
WHERE temperature_zone IS NOT NULL;

-- Active storage locations
CREATE INDEX IF NOT EXISTS idx_storage_locations_is_active 
ON storage_locations (is_active);

-- =============================================================================
-- AUDIT AND SECURITY INDEXES
-- =============================================================================
-- Note: security_audit_log indexes are created in the security_audit_log migration itself

-- =============================================================================
-- SESSION MANAGEMENT INDEXES
-- =============================================================================
-- Note: user_sessions indexes are created in the security_audit_log migration itself

-- =============================================================================
-- SPREADSHEET DATA INDEXES
-- =============================================================================
-- Note: spreadsheet_data table doesn't exist in current schema - skipping these indexes

-- =============================================================================
-- PERFORMANCE MONITORING
-- =============================================================================

-- =============================================================================
-- INDEX MAINTENANCE NOTES
-- =============================================================================

-- CONCURRENTLY option prevents table locking during index creation
-- These indexes should be monitored for usage with pg_stat_user_indexes
-- Consider REINDEX CONCURRENTLY for maintenance if indexes become bloated
-- Monitor index size growth and query performance impact regularly

-- Query to check index usage:
-- SELECT schemaname, tablename, indexname, idx_scan, idx_tup_read, idx_tup_fetch
-- FROM pg_stat_user_indexes 
-- ORDER BY idx_scan DESC; 
