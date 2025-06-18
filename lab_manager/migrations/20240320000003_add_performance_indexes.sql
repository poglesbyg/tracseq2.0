-- Add performance indexes for critical laboratory data lookups
-- Migration: 20240320000003_add_performance_indexes

-- =============================================================================
-- SAMPLE MANAGEMENT INDEXES
-- =============================================================================

-- Primary sample lookup by barcode (most frequent operation)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_samples_barcode 
ON samples (barcode) 
WHERE barcode IS NOT NULL;

-- Sample ID lookups for foreign key relationships
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_samples_sample_id 
ON samples (id);

-- Sample name searches (partial match support)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_samples_name_gin 
ON samples USING gin (name gin_trgm_ops) 
WHERE name IS NOT NULL;

-- Sample status filtering
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_samples_status 
ON samples (status) 
WHERE status IS NOT NULL;

-- Sample type filtering  
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_samples_sample_type 
ON samples (sample_type) 
WHERE sample_type IS NOT NULL;

-- Created date range queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_samples_created_at 
ON samples (created_at);

-- Updated date for change tracking
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_samples_updated_at 
ON samples (updated_at);

-- Storage location lookups
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_samples_storage_location 
ON samples (storage_location) 
WHERE storage_location IS NOT NULL;

-- Composite index for common filtered queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_samples_status_type_created 
ON samples (status, sample_type, created_at) 
WHERE status IS NOT NULL AND sample_type IS NOT NULL;

-- =============================================================================
-- USER MANAGEMENT INDEXES
-- =============================================================================

-- Email lookups for authentication (most critical for login performance)
CREATE UNIQUE INDEX CONCURRENTLY IF NOT EXISTS idx_users_email_unique 
ON users (email);

-- User ID lookups
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_id 
ON users (id);

-- Role-based queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_role 
ON users (role) 
WHERE role IS NOT NULL;

-- Active user filtering
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_is_active 
ON users (is_active);

-- Login tracking
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_last_login 
ON users (last_login) 
WHERE last_login IS NOT NULL;

-- Account lockout queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_locked_until 
ON users (locked_until) 
WHERE locked_until IS NOT NULL;

-- Failed login attempt tracking
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_failed_attempts 
ON users (failed_login_attempts) 
WHERE failed_login_attempts > 0;

-- =============================================================================
-- SEQUENCING JOB INDEXES
-- =============================================================================

-- Job ID lookups
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sequencing_jobs_id 
ON sequencing_jobs (id);

-- Job status filtering
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sequencing_jobs_status 
ON sequencing_jobs (status) 
WHERE status IS NOT NULL;

-- Job name searches
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sequencing_jobs_name 
ON sequencing_jobs (name) 
WHERE name IS NOT NULL;

-- Created date for job history
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sequencing_jobs_created_at 
ON sequencing_jobs (created_at);

-- Composite index for active job queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sequencing_jobs_status_created 
ON sequencing_jobs (status, created_at) 
WHERE status IS NOT NULL;

-- =============================================================================
-- TEMPLATE MANAGEMENT INDEXES
-- =============================================================================

-- Template name lookups
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_templates_name 
ON templates (name) 
WHERE name IS NOT NULL;

-- Template type filtering
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_templates_template_type 
ON templates (template_type) 
WHERE template_type IS NOT NULL;

-- Active template filtering
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_templates_is_active 
ON templates (is_active);

-- =============================================================================
-- STORAGE MANAGEMENT INDEXES
-- =============================================================================

-- Storage location lookups
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_storage_locations_location_id 
ON storage_locations (location_id) 
WHERE location_id IS NOT NULL;

-- Storage type filtering
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_storage_locations_storage_type 
ON storage_locations (storage_type) 
WHERE storage_type IS NOT NULL;

-- Capacity queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_storage_locations_capacity 
ON storage_locations (total_capacity, occupied_capacity);

-- Temperature condition filtering
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_storage_locations_temperature 
ON storage_locations (temperature_condition) 
WHERE temperature_condition IS NOT NULL;

-- =============================================================================
-- AUDIT AND SECURITY INDEXES
-- =============================================================================

-- Security audit log queries by user
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_security_audit_log_user_id 
ON security_audit_log (user_id) 
WHERE user_id IS NOT NULL;

-- Security audit log queries by event type
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_security_audit_log_event_type 
ON security_audit_log (event_type);

-- Security audit log time-based queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_security_audit_log_timestamp 
ON security_audit_log (timestamp);

-- Security audit log by severity
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_security_audit_log_severity 
ON security_audit_log (severity);

-- Composite index for security monitoring
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_security_audit_log_severity_timestamp 
ON security_audit_log (severity, timestamp);

-- =============================================================================
-- SESSION MANAGEMENT INDEXES
-- =============================================================================

-- User session lookups
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_sessions_user_id 
ON user_sessions (user_id);

-- Session token lookups
CREATE UNIQUE INDEX CONCURRENTLY IF NOT EXISTS idx_user_sessions_session_token 
ON user_sessions (session_token);

-- Active session filtering
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_sessions_is_active 
ON user_sessions (is_active);

-- Session expiry cleanup
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_sessions_expires_at 
ON user_sessions (expires_at);

-- =============================================================================
-- SPREADSHEET DATA INDEXES
-- =============================================================================

-- Spreadsheet ID lookups
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_spreadsheet_data_spreadsheet_id 
ON spreadsheet_data (spreadsheet_id);

-- Upload timestamp queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_spreadsheet_data_uploaded_at 
ON spreadsheet_data (uploaded_at);

-- Processing status filtering
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_spreadsheet_data_processing_status 
ON spreadsheet_data (processing_status) 
WHERE processing_status IS NOT NULL;

-- =============================================================================
-- PERFORMANCE MONITORING
-- =============================================================================

-- Enable pg_stat_statements extension for query performance monitoring
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- Enable pg_trgm extension for fuzzy text search
CREATE EXTENSION IF NOT EXISTS pg_trgm;

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
