-- Comprehensive schema fixes for all services
-- This script ensures all tables have the expected columns

-- Create enum types if they don't exist
DO $$ BEGIN
    CREATE TYPE user_role AS ENUM (
        'guest',
        'data_analyst', 
        'research_scientist',
        'lab_technician',
        'principal_investigator',
        'lab_administrator'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

DO $$ BEGIN
    CREATE TYPE user_status AS ENUM (
        'active',
        'inactive',
        'suspended',
        'deleted'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Auth Service fixes
-- Add missing columns to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS status user_status DEFAULT 'active';
ALTER TABLE users ADD COLUMN IF NOT EXISTS first_name VARCHAR(100) DEFAULT '';
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_name VARCHAR(100) DEFAULT '';
ALTER TABLE users ADD COLUMN IF NOT EXISTS role user_role DEFAULT 'guest';
ALTER TABLE users ADD COLUMN IF NOT EXISTS email_verified BOOLEAN DEFAULT FALSE;
ALTER TABLE users ADD COLUMN IF NOT EXISTS failed_login_attempts INTEGER DEFAULT 0;
ALTER TABLE users ADD COLUMN IF NOT EXISTS locked_until TIMESTAMPTZ;
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_login_at TIMESTAMPTZ;
ALTER TABLE users ADD COLUMN IF NOT EXISTS password_changed_at TIMESTAMPTZ;
ALTER TABLE users ADD COLUMN IF NOT EXISTS department VARCHAR(100);
ALTER TABLE users ADD COLUMN IF NOT EXISTS position VARCHAR(100);
ALTER TABLE users ADD COLUMN IF NOT EXISTS lab_affiliation VARCHAR(100);
ALTER TABLE users ADD COLUMN IF NOT EXISTS phone VARCHAR(50);
ALTER TABLE users ADD COLUMN IF NOT EXISTS verification_token VARCHAR(255);
ALTER TABLE users ADD COLUMN IF NOT EXISTS office_location VARCHAR(100);
ALTER TABLE users ADD COLUMN IF NOT EXISTS shibboleth_id VARCHAR(255);
ALTER TABLE users ADD COLUMN IF NOT EXISTS external_id VARCHAR(255);

-- Create user_sessions table if it doesn't exist
CREATE TABLE IF NOT EXISTS user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(64),
    refresh_token_hash VARCHAR(64),
    token VARCHAR(255) UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    device_info JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    revoked_at TIMESTAMPTZ,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create security_audit_log table if it doesn't exist
CREATE TABLE IF NOT EXISTS security_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL UNIQUE,
    event_type VARCHAR(50) NOT NULL,
    user_id UUID REFERENCES users(id),
    user_email VARCHAR(255),
    ip_address VARCHAR(45),
    user_agent TEXT,
    details JSONB,
    severity VARCHAR(20) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create user_activity_log table if it doesn't exist
CREATE TABLE IF NOT EXISTS user_activity_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50),
    resource_id UUID,
    ip_address VARCHAR(45),
    user_agent TEXT,
    details JSONB,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add missing timestamp column to sessions table
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS timestamp TIMESTAMPTZ DEFAULT NOW();

-- Add any other missing columns to sessions
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ DEFAULT NOW();
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ DEFAULT NOW();

-- Fix rate_limits table if it still has issues
ALTER TABLE rate_limits ADD COLUMN IF NOT EXISTS identifier VARCHAR(255);
ALTER TABLE rate_limits ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ DEFAULT NOW();
ALTER TABLE rate_limits ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ DEFAULT NOW();

-- Storage Service fixes
-- Ensure storage_locations table exists with all required columns
CREATE TABLE IF NOT EXISTS storage_locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    location_type VARCHAR(50) NOT NULL,
    temperature_zone VARCHAR(50),
    capacity INTEGER,
    current_usage INTEGER DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Ensure storage_items table exists
CREATE TABLE IF NOT EXISTS storage_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    item_type VARCHAR(50) NOT NULL,
    location_id UUID REFERENCES storage_locations(id),
    quantity INTEGER DEFAULT 1,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Event Service fixes
-- Ensure events table exists
CREATE TABLE IF NOT EXISTS events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(255) NOT NULL,
    aggregate_id UUID,
    aggregate_type VARCHAR(255),
    event_data JSONB NOT NULL,
    metadata JSONB DEFAULT '{}',
    user_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Ensure event_subscriptions table exists
CREATE TABLE IF NOT EXISTS event_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscriber_name VARCHAR(255) NOT NULL,
    event_types TEXT[] NOT NULL,
    webhook_url VARCHAR(500),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Notification Service fixes
-- Ensure notifications table exists
CREATE TABLE IF NOT EXISTS notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipient_id UUID NOT NULL,
    channel VARCHAR(50) NOT NULL,
    subject VARCHAR(255),
    content TEXT NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    sent_at TIMESTAMPTZ,
    error_message TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_timestamp ON sessions(timestamp);
CREATE INDEX IF NOT EXISTS idx_storage_locations_type ON storage_locations(location_type);
CREATE INDEX IF NOT EXISTS idx_storage_items_location ON storage_items(location_id);
CREATE INDEX IF NOT EXISTS idx_events_aggregate ON events(aggregate_id, aggregate_type);
CREATE INDEX IF NOT EXISTS idx_events_created ON events(created_at);
CREATE INDEX IF NOT EXISTS idx_notifications_recipient ON notifications(recipient_id);
CREATE INDEX IF NOT EXISTS idx_notifications_status ON notifications(status);

-- Create or update the update trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply update triggers to all tables with updated_at
DO $$
DECLARE
    t record;
BEGIN
    FOR t IN 
        SELECT table_name 
        FROM information_schema.columns 
        WHERE column_name = 'updated_at' 
        AND table_schema = 'public'
    LOOP
        EXECUTE format('DROP TRIGGER IF EXISTS update_%I_updated_at ON %I', t.table_name, t.table_name);
        EXECUTE format('CREATE TRIGGER update_%I_updated_at BEFORE UPDATE ON %I FOR EACH ROW EXECUTE FUNCTION update_updated_at_column()', t.table_name, t.table_name);
    END LOOP;
END$$;

-- Sample Service fixes
-- Create sample_status enum if it doesn't exist
DO $$ BEGIN
    CREATE TYPE sample_status AS ENUM (
        'pending',
        'validated', 
        'in_storage',
        'in_sequencing',
        'completed',
        'failed',
        'discarded',
        'deleted'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Add missing columns to samples table
ALTER TABLE samples ADD COLUMN IF NOT EXISTS barcode VARCHAR(100) UNIQUE;
ALTER TABLE samples ADD COLUMN IF NOT EXISTS template_id UUID;
ALTER TABLE samples ADD COLUMN IF NOT EXISTS source_type VARCHAR(50);
ALTER TABLE samples ADD COLUMN IF NOT EXISTS source_identifier VARCHAR(255);
ALTER TABLE samples ADD COLUMN IF NOT EXISTS collection_date TIMESTAMPTZ;
ALTER TABLE samples ADD COLUMN IF NOT EXISTS collection_location VARCHAR(255);
ALTER TABLE samples ADD COLUMN IF NOT EXISTS collector VARCHAR(255);
ALTER TABLE samples ADD COLUMN IF NOT EXISTS concentration DECIMAL(10,4);
ALTER TABLE samples ADD COLUMN IF NOT EXISTS volume DECIMAL(10,4);
ALTER TABLE samples ADD COLUMN IF NOT EXISTS unit VARCHAR(20);
ALTER TABLE samples ADD COLUMN IF NOT EXISTS quality_score DECIMAL(3,2);
ALTER TABLE samples ADD COLUMN IF NOT EXISTS notes TEXT;
ALTER TABLE samples ADD COLUMN IF NOT EXISTS created_by VARCHAR(255);
ALTER TABLE samples ADD COLUMN IF NOT EXISTS updated_by VARCHAR(255);

-- Update status column to use enum if it's still varchar
DO $$ 
BEGIN
    -- Check if status column exists and is not an enum
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'samples' 
        AND column_name = 'status' 
        AND data_type = 'character varying'
    ) THEN
        -- Add temporary column
        ALTER TABLE samples ADD COLUMN status_new sample_status;
        -- Copy and convert data
        UPDATE samples SET status_new = 
            CASE 
                WHEN status = 'pending' THEN 'pending'::sample_status
                WHEN status = 'validated' THEN 'validated'::sample_status
                WHEN status = 'in_storage' THEN 'in_storage'::sample_status
                WHEN status = 'in_sequencing' THEN 'in_sequencing'::sample_status
                WHEN status = 'completed' THEN 'completed'::sample_status
                WHEN status = 'failed' THEN 'failed'::sample_status
                WHEN status = 'discarded' THEN 'discarded'::sample_status
                ELSE 'pending'::sample_status
            END;
        -- Drop old column and rename new one
        ALTER TABLE samples DROP COLUMN status;
        ALTER TABLE samples RENAME COLUMN status_new TO status;
        ALTER TABLE samples ALTER COLUMN status SET DEFAULT 'pending'::sample_status;
        ALTER TABLE samples ALTER COLUMN status SET NOT NULL;
    END IF;
END $$;

-- Create sample workflow tables
CREATE TABLE IF NOT EXISTS sample_relationships (
    id SERIAL PRIMARY KEY,
    parent_sample_id UUID NOT NULL REFERENCES samples(id) ON DELETE CASCADE,
    child_sample_id UUID NOT NULL REFERENCES samples(id) ON DELETE CASCADE,
    relationship_type VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(parent_sample_id, child_sample_id, relationship_type)
);

CREATE TABLE IF NOT EXISTS sample_status_history (
    id SERIAL PRIMARY KEY,
    sample_id UUID NOT NULL REFERENCES samples(id) ON DELETE CASCADE,
    previous_status sample_status,
    new_status sample_status NOT NULL,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    changed_by VARCHAR(255),
    reason VARCHAR(500),
    automated BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB DEFAULT '{}'
);

-- Fix column name mismatch
DO $$ 
BEGIN
    -- Check if the column needs to be renamed
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'sample_status_history' 
        AND column_name = 'old_status'
    ) AND NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'sample_status_history' 
        AND column_name = 'previous_status'
    ) THEN
        ALTER TABLE sample_status_history RENAME COLUMN old_status TO previous_status;
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS sample_validation_rules (
    id SERIAL PRIMARY KEY,
    rule_name VARCHAR(100) NOT NULL UNIQUE,
    sample_type VARCHAR(50),
    rule_expression TEXT NOT NULL,
    error_message VARCHAR(500),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    severity VARCHAR(20) NOT NULL DEFAULT 'error',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS sample_validation_results (
    id SERIAL PRIMARY KEY,
    sample_id UUID NOT NULL REFERENCES samples(id) ON DELETE CASCADE,
    rule_id INTEGER REFERENCES sample_validation_rules(id),
    validation_passed BOOLEAN NOT NULL,
    error_message TEXT,
    validated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    validated_by VARCHAR(255)
);

CREATE TABLE IF NOT EXISTS barcode_sequences (
    id SERIAL PRIMARY KEY,
    prefix VARCHAR(20) NOT NULL,
    sequence_number BIGINT NOT NULL DEFAULT 1,
    last_generated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(prefix)
);

CREATE TABLE IF NOT EXISTS barcode_audit (
    id SERIAL PRIMARY KEY,
    barcode VARCHAR(100) NOT NULL,
    sample_id UUID REFERENCES samples(id),
    action VARCHAR(50) NOT NULL,
    performed_by VARCHAR(255),
    performed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS sample_audit_log (
    id SERIAL PRIMARY KEY,
    sample_id UUID REFERENCES samples(id),
    action VARCHAR(100) NOT NULL,
    old_values JSONB,
    new_values JSONB,
    performed_by VARCHAR(255),
    performed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    session_id VARCHAR(255),
    ip_address INET,
    user_agent TEXT
); 