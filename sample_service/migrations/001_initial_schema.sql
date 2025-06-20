-- Initial Sample Service Database Schema
-- This migration creates all necessary tables for sample management functionality

-- Sample status enum
CREATE TYPE sample_status AS ENUM (
    'pending',
    'validated',
    'in_storage',
    'in_sequencing',
    'completed',
    'failed',
    'rejected',
    'archived',
    'deleted'
);

-- Main samples table
CREATE TABLE samples (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    barcode VARCHAR(100) NOT NULL UNIQUE,
    sample_type VARCHAR(100) NOT NULL,
    status sample_status NOT NULL DEFAULT 'pending',
    template_id UUID, -- Reference to template service
    
    -- Source information
    source_type VARCHAR(100),
    source_identifier VARCHAR(255),
    collection_date TIMESTAMPTZ,
    collection_location VARCHAR(255),
    collector VARCHAR(255),
    
    -- Measurements
    concentration DECIMAL(15,6),
    volume DECIMAL(15,6),
    unit VARCHAR(50),
    quality_score DECIMAL(5,3),
    
    -- Metadata and notes
    metadata JSONB NOT NULL DEFAULT '{}',
    notes TEXT,
    
    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by VARCHAR(255),
    updated_by VARCHAR(255),
    
    -- Constraints
    CONSTRAINT samples_name_check CHECK (length(trim(name)) > 0),
    CONSTRAINT samples_barcode_check CHECK (length(trim(barcode)) > 0),
    CONSTRAINT samples_sample_type_check CHECK (length(trim(sample_type)) > 0),
    CONSTRAINT samples_concentration_check CHECK (concentration IS NULL OR concentration >= 0),
    CONSTRAINT samples_volume_check CHECK (volume IS NULL OR volume >= 0),
    CONSTRAINT samples_quality_score_check CHECK (quality_score IS NULL OR (quality_score >= 0 AND quality_score <= 100))
);

-- Sample status history table
CREATE TABLE sample_status_history (
    id SERIAL PRIMARY KEY,
    sample_id UUID NOT NULL REFERENCES samples(id) ON DELETE CASCADE,
    old_status sample_status,
    new_status sample_status NOT NULL,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    changed_by VARCHAR(255),
    reason TEXT,
    metadata JSONB DEFAULT '{}'
);

-- Sample validation results table
CREATE TABLE sample_validation_results (
    id SERIAL PRIMARY KEY,
    sample_id UUID NOT NULL REFERENCES samples(id) ON DELETE CASCADE,
    is_valid BOOLEAN NOT NULL,
    validation_type VARCHAR(100) NOT NULL,
    errors JSONB DEFAULT '[]',
    warnings JSONB DEFAULT '[]',
    validated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    validated_by VARCHAR(255),
    validation_metadata JSONB DEFAULT '{}'
);

-- Sample audit log table
CREATE TABLE sample_audit_log (
    id SERIAL PRIMARY KEY,
    sample_id UUID REFERENCES samples(id) ON DELETE CASCADE,
    action VARCHAR(100) NOT NULL,
    old_values JSONB,
    new_values JSONB,
    performed_by VARCHAR(255),
    performed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    session_id VARCHAR(255),
    ip_address INET,
    user_agent TEXT
);

-- Sample relationships table (for linked samples)
CREATE TABLE sample_relationships (
    id SERIAL PRIMARY KEY,
    parent_sample_id UUID NOT NULL REFERENCES samples(id) ON DELETE CASCADE,
    child_sample_id UUID NOT NULL REFERENCES samples(id) ON DELETE CASCADE,
    relationship_type VARCHAR(100) NOT NULL, -- 'derived_from', 'split_from', 'pooled_from', etc.
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by VARCHAR(255),
    metadata JSONB DEFAULT '{}',
    
    UNIQUE(parent_sample_id, child_sample_id, relationship_type),
    CONSTRAINT no_self_reference CHECK (parent_sample_id != child_sample_id)
);

-- Sample attachments table (for files/documents)
CREATE TABLE sample_attachments (
    id SERIAL PRIMARY KEY,
    sample_id UUID NOT NULL REFERENCES samples(id) ON DELETE CASCADE,
    filename VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size BIGINT,
    mime_type VARCHAR(255),
    description TEXT,
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    uploaded_by VARCHAR(255)
);

-- Sample storage locations table (integration with storage service)
CREATE TABLE sample_storage_locations (
    id SERIAL PRIMARY KEY,
    sample_id UUID NOT NULL REFERENCES samples(id) ON DELETE CASCADE,
    location_id VARCHAR(255) NOT NULL,
    location_type VARCHAR(100), -- 'freezer', 'refrigerator', 'room_temp', etc.
    temperature_zone VARCHAR(50),
    rack_position VARCHAR(100),
    stored_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    stored_by VARCHAR(255),
    removed_at TIMESTAMPTZ,
    removed_by VARCHAR(255),
    is_current BOOLEAN NOT NULL DEFAULT TRUE
);

-- Batch processing results table
CREATE TABLE batch_processing_results (
    id SERIAL PRIMARY KEY,
    batch_id UUID NOT NULL DEFAULT gen_random_uuid(),
    total_samples INTEGER NOT NULL,
    successful_samples INTEGER NOT NULL DEFAULT 0,
    failed_samples INTEGER NOT NULL DEFAULT 0,
    processing_started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processing_completed_at TIMESTAMPTZ,
    processed_by VARCHAR(255),
    error_summary JSONB DEFAULT '[]',
    metadata JSONB DEFAULT '{}'
);

-- Create indexes for performance
CREATE INDEX idx_samples_barcode ON samples(barcode);
CREATE INDEX idx_samples_status ON samples(status);
CREATE INDEX idx_samples_sample_type ON samples(sample_type);
CREATE INDEX idx_samples_template_id ON samples(template_id);
CREATE INDEX idx_samples_created_at ON samples(created_at);
CREATE INDEX idx_samples_updated_at ON samples(updated_at);
CREATE INDEX idx_samples_created_by ON samples(created_by);
CREATE INDEX idx_samples_collection_date ON samples(collection_date);

-- Text search index for sample names and metadata
CREATE INDEX idx_samples_name_trgm ON samples USING gin(name gin_trgm_ops);
CREATE INDEX idx_samples_metadata_gin ON samples USING gin(metadata);

CREATE INDEX idx_status_history_sample_id ON sample_status_history(sample_id);
CREATE INDEX idx_status_history_changed_at ON sample_status_history(changed_at);
CREATE INDEX idx_status_history_new_status ON sample_status_history(new_status);

CREATE INDEX idx_validation_results_sample_id ON sample_validation_results(sample_id);
CREATE INDEX idx_validation_results_validated_at ON sample_validation_results(validated_at);
CREATE INDEX idx_validation_results_is_valid ON sample_validation_results(is_valid);

CREATE INDEX idx_audit_log_sample_id ON sample_audit_log(sample_id);
CREATE INDEX idx_audit_log_performed_at ON sample_audit_log(performed_at);
CREATE INDEX idx_audit_log_action ON sample_audit_log(action);
CREATE INDEX idx_audit_log_performed_by ON sample_audit_log(performed_by);

CREATE INDEX idx_relationships_parent ON sample_relationships(parent_sample_id);
CREATE INDEX idx_relationships_child ON sample_relationships(child_sample_id);
CREATE INDEX idx_relationships_type ON sample_relationships(relationship_type);

CREATE INDEX idx_attachments_sample_id ON sample_attachments(sample_id);
CREATE INDEX idx_attachments_uploaded_at ON sample_attachments(uploaded_at);

CREATE INDEX idx_storage_locations_sample_id ON sample_storage_locations(sample_id);
CREATE INDEX idx_storage_locations_location_id ON sample_storage_locations(location_id);
CREATE INDEX idx_storage_locations_is_current ON sample_storage_locations(is_current);
CREATE INDEX idx_storage_locations_stored_at ON sample_storage_locations(stored_at);

CREATE INDEX idx_batch_results_batch_id ON batch_processing_results(batch_id);
CREATE INDEX idx_batch_results_started_at ON batch_processing_results(processing_started_at);

-- Create function to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger to automatically update updated_at for samples table
CREATE TRIGGER update_samples_updated_at 
    BEFORE UPDATE ON samples 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Create function to automatically log status changes
CREATE OR REPLACE FUNCTION log_status_change()
RETURNS TRIGGER AS $$
BEGIN
    -- Only log if status actually changed
    IF OLD.status IS DISTINCT FROM NEW.status THEN
        INSERT INTO sample_status_history (
            sample_id, old_status, new_status, changed_by, reason
        ) VALUES (
            NEW.id, OLD.status, NEW.status, NEW.updated_by, 'Automatic status change'
        );
    END IF;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger to automatically log status changes
CREATE TRIGGER log_sample_status_changes
    AFTER UPDATE ON samples
    FOR EACH ROW
    WHEN (OLD.status IS DISTINCT FROM NEW.status)
    EXECUTE FUNCTION log_status_change();

-- Create function to validate sample transitions
CREATE OR REPLACE FUNCTION validate_status_transition()
RETURNS TRIGGER AS $$
DECLARE
    valid_transitions TEXT[];
BEGIN
    -- Define valid transitions based on current status
    CASE OLD.status
        WHEN 'pending' THEN
            valid_transitions := ARRAY['validated', 'rejected', 'deleted'];
        WHEN 'validated' THEN
            valid_transitions := ARRAY['in_storage', 'in_sequencing', 'rejected', 'deleted'];
        WHEN 'in_storage' THEN
            valid_transitions := ARRAY['in_sequencing', 'rejected', 'deleted'];
        WHEN 'in_sequencing' THEN
            valid_transitions := ARRAY['completed', 'failed'];
        WHEN 'completed' THEN
            valid_transitions := ARRAY['archived'];
        WHEN 'failed' THEN
            valid_transitions := ARRAY['pending', 'deleted'];
        WHEN 'rejected' THEN
            valid_transitions := ARRAY['pending', 'deleted'];
        WHEN 'archived' THEN
            valid_transitions := ARRAY[]::TEXT[]; -- No transitions from archived
        WHEN 'deleted' THEN
            valid_transitions := ARRAY[]::TEXT[]; -- No transitions from deleted
    END CASE;

    -- Check if the new status is valid
    IF NEW.status::TEXT = ANY(valid_transitions) OR OLD.status = NEW.status THEN
        RETURN NEW;
    ELSE
        RAISE EXCEPTION 'Invalid status transition from % to %', OLD.status, NEW.status;
    END IF;
END;
$$ language 'plpgsql';

-- Create trigger to validate status transitions
CREATE TRIGGER validate_sample_status_transitions
    BEFORE UPDATE ON samples
    FOR EACH ROW
    WHEN (OLD.status IS DISTINCT FROM NEW.status)
    EXECUTE FUNCTION validate_status_transition();

-- Create function to automatically update storage location status
CREATE OR REPLACE FUNCTION update_storage_location_status()
RETURNS TRIGGER AS $$
BEGIN
    -- Update current location when sample status changes to in_storage
    IF NEW.status = 'in_storage' AND OLD.status != 'in_storage' THEN
        -- This would integrate with storage service to update location
        -- For now, we just log the change
        NULL;
    END IF;
    
    -- Remove from storage when sample status changes from in_storage
    IF OLD.status = 'in_storage' AND NEW.status != 'in_storage' THEN
        UPDATE sample_storage_locations 
        SET is_current = FALSE, removed_at = NOW(), removed_by = NEW.updated_by
        WHERE sample_id = NEW.id AND is_current = TRUE;
    END IF;
    
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger to update storage location status
CREATE TRIGGER update_sample_storage_status
    AFTER UPDATE ON samples
    FOR EACH ROW
    WHEN (OLD.status IS DISTINCT FROM NEW.status)
    EXECUTE FUNCTION update_storage_location_status();

-- Create function to clean up old data
CREATE OR REPLACE FUNCTION cleanup_old_sample_data()
RETURNS void AS $$
BEGIN
    -- Delete old audit log entries (older than 2 years)
    DELETE FROM sample_audit_log 
    WHERE performed_at < NOW() - INTERVAL '2 years';
    
    -- Delete old validation results (older than 1 year)
    DELETE FROM sample_validation_results 
    WHERE validated_at < NOW() - INTERVAL '1 year';
    
    -- Delete old batch processing results (older than 6 months)
    DELETE FROM batch_processing_results 
    WHERE processing_started_at < NOW() - INTERVAL '6 months';
END;
$$ language 'plpgsql';

-- Create views for common queries

-- Sample summary view
CREATE VIEW sample_summary AS
SELECT 
    s.id,
    s.name,
    s.barcode,
    s.sample_type,
    s.status,
    s.created_at,
    s.updated_at,
    sl.location_id AS current_location,
    sl.temperature_zone,
    COALESCE(att_count.count, 0) AS attachment_count,
    COALESCE(rel_count.count, 0) AS relationship_count
FROM samples s
LEFT JOIN sample_storage_locations sl ON s.id = sl.sample_id AND sl.is_current = TRUE
LEFT JOIN (
    SELECT sample_id, COUNT(*) as count 
    FROM sample_attachments 
    GROUP BY sample_id
) att_count ON s.id = att_count.sample_id
LEFT JOIN (
    SELECT parent_sample_id as sample_id, COUNT(*) as count 
    FROM sample_relationships 
    GROUP BY parent_sample_id
    UNION ALL
    SELECT child_sample_id as sample_id, COUNT(*) as count 
    FROM sample_relationships 
    GROUP BY child_sample_id
) rel_count ON s.id = rel_count.sample_id;

-- Sample statistics view
CREATE VIEW sample_statistics AS
SELECT 
    status,
    sample_type,
    COUNT(*) as sample_count,
    COUNT(CASE WHEN created_at > NOW() - INTERVAL '24 hours' THEN 1 END) as created_last_24h,
    COUNT(CASE WHEN updated_at > NOW() - INTERVAL '24 hours' THEN 1 END) as updated_last_24h,
    AVG(CASE WHEN quality_score IS NOT NULL THEN quality_score END) as avg_quality_score,
    MIN(created_at) as first_created,
    MAX(created_at) as last_created
FROM samples
GROUP BY status, sample_type;

-- Recent activity view
CREATE VIEW recent_sample_activity AS
SELECT 
    'sample_created' as activity_type,
    s.id as sample_id,
    s.name as sample_name,
    s.barcode,
    s.created_at as activity_time,
    s.created_by as performed_by,
    NULL as details
FROM samples s
WHERE s.created_at > NOW() - INTERVAL '7 days'

UNION ALL

SELECT 
    'status_changed' as activity_type,
    sh.sample_id,
    s.name as sample_name,
    s.barcode,
    sh.changed_at as activity_time,
    sh.changed_by as performed_by,
    json_build_object('old_status', sh.old_status, 'new_status', sh.new_status) as details
FROM sample_status_history sh
JOIN samples s ON sh.sample_id = s.id
WHERE sh.changed_at > NOW() - INTERVAL '7 days'

ORDER BY activity_time DESC; 
