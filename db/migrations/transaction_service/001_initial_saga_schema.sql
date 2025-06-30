-- Initial schema for distributed transaction saga persistence
-- This migration creates the core tables for storing saga state and execution history

-- Enable UUID extension for PostgreSQL
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Saga execution statuses as enum
CREATE TYPE saga_status AS ENUM (
    'Created',
    'Executing',
    'Compensating', 
    'Completed',
    'Compensated',
    'Failed',
    'Paused',
    'Cancelled',
    'TimedOut'
);

-- Step execution statuses as enum
CREATE TYPE step_status AS ENUM (
    'Pending',
    'Executing',
    'Completed',
    'Failed',
    'Skipped',
    'Retrying'
);

-- Compensation execution statuses as enum
CREATE TYPE compensation_status AS ENUM (
    'Pending',
    'Executing',
    'Completed',
    'Failed',
    'Skipped',
    'Retrying'
);

-- Main saga table for storing saga execution state
CREATE TABLE sagas (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    status saga_status NOT NULL DEFAULT 'Created',
    transaction_id UUID NOT NULL,
    user_id UUID,
    correlation_id UUID,
    
    -- Execution tracking
    completed_steps INTEGER NOT NULL DEFAULT 0,
    total_steps INTEGER NOT NULL DEFAULT 0,
    current_step VARCHAR(255),
    failed_step VARCHAR(255),
    
    -- Timing information
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Configuration
    timeout_ms BIGINT NOT NULL DEFAULT 300000,
    max_retries INTEGER NOT NULL DEFAULT 3,
    
    -- Serialized context and metadata
    transaction_context JSONB NOT NULL DEFAULT '{}',
    metadata JSONB NOT NULL DEFAULT '{}',
    custom_data JSONB NOT NULL DEFAULT '{}',
    
    -- Execution metrics
    execution_time_ms BIGINT DEFAULT 0,
    retry_attempts INTEGER DEFAULT 0,
    
    -- Error information
    error_message TEXT,
    error_category VARCHAR(100),
    compensation_errors JSONB DEFAULT '[]',
    
    -- Indexes for performance
    CONSTRAINT valid_completed_steps CHECK (completed_steps >= 0),
    CONSTRAINT valid_total_steps CHECK (total_steps >= 0),
    CONSTRAINT valid_timeout CHECK (timeout_ms > 0),
    CONSTRAINT valid_retries CHECK (max_retries >= 0)
);

-- Step execution history table
CREATE TABLE saga_steps (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    saga_id UUID NOT NULL REFERENCES sagas(id) ON DELETE CASCADE,
    step_name VARCHAR(255) NOT NULL,
    step_index INTEGER NOT NULL,
    status step_status NOT NULL DEFAULT 'Pending',
    
    -- Execution details
    execution_id UUID NOT NULL DEFAULT uuid_generate_v4(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    retry_count INTEGER DEFAULT 0,
    
    -- Step data and results
    input_data JSONB DEFAULT '{}',
    output_data JSONB DEFAULT '{}',
    step_metadata JSONB DEFAULT '{}',
    
    -- Error information
    error_message TEXT,
    
    -- Timing metrics
    execution_duration_ms BIGINT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT unique_saga_step_index UNIQUE(saga_id, step_index),
    CONSTRAINT valid_step_index CHECK (step_index >= 0),
    CONSTRAINT valid_retry_count CHECK (retry_count >= 0)
);

-- Compensation step execution history table
CREATE TABLE saga_compensations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    saga_id UUID NOT NULL REFERENCES sagas(id) ON DELETE CASCADE,
    step_name VARCHAR(255) NOT NULL,
    step_index INTEGER NOT NULL,
    status compensation_status NOT NULL DEFAULT 'Pending',
    
    -- Execution details
    execution_id UUID NOT NULL DEFAULT uuid_generate_v4(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    retry_count INTEGER DEFAULT 0,
    
    -- Compensation data and results
    input_data JSONB DEFAULT '{}',
    output_data JSONB DEFAULT '{}',
    compensation_metadata JSONB DEFAULT '{}',
    
    -- Error information
    error_message TEXT,
    skip_reason TEXT,
    
    -- Timing metrics
    execution_duration_ms BIGINT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT unique_saga_compensation_index UNIQUE(saga_id, step_index),
    CONSTRAINT valid_compensation_index CHECK (step_index >= 0),
    CONSTRAINT valid_compensation_retry_count CHECK (retry_count >= 0)
);

-- Saga checkpoints for recovery
CREATE TABLE saga_checkpoints (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    saga_id UUID NOT NULL REFERENCES sagas(id) ON DELETE CASCADE,
    checkpoint_type VARCHAR(50) NOT NULL,
    step_index INTEGER NOT NULL,
    
    -- Checkpoint data
    state_snapshot JSONB NOT NULL,
    checkpoint_metadata JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT valid_checkpoint_index CHECK (step_index >= 0)
);

-- Saga events for audit trail
CREATE TABLE saga_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    saga_id UUID NOT NULL REFERENCES sagas(id) ON DELETE CASCADE,
    event_type VARCHAR(100) NOT NULL,
    event_data JSONB NOT NULL DEFAULT '{}',
    
    -- Event metadata
    event_source VARCHAR(100),
    correlation_id UUID,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Performance index
    INDEX idx_saga_events_saga_id_created (saga_id, created_at),
    INDEX idx_saga_events_type_created (event_type, created_at)
);

-- Performance indexes
CREATE INDEX idx_sagas_status ON sagas(status);
CREATE INDEX idx_sagas_created_at ON sagas(created_at);
CREATE INDEX idx_sagas_updated_at ON sagas(updated_at);
CREATE INDEX idx_sagas_transaction_id ON sagas(transaction_id);
CREATE INDEX idx_sagas_user_id ON sagas(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX idx_sagas_correlation_id ON sagas(correlation_id) WHERE correlation_id IS NOT NULL;

CREATE INDEX idx_saga_steps_saga_id ON saga_steps(saga_id);
CREATE INDEX idx_saga_steps_status ON saga_steps(status);
CREATE INDEX idx_saga_steps_created_at ON saga_steps(created_at);

CREATE INDEX idx_saga_compensations_saga_id ON saga_compensations(saga_id);
CREATE INDEX idx_saga_compensations_status ON saga_compensations(status);
CREATE INDEX idx_saga_compensations_created_at ON saga_compensations(created_at);

CREATE INDEX idx_saga_checkpoints_saga_id ON saga_checkpoints(saga_id);
CREATE INDEX idx_saga_checkpoints_created_at ON saga_checkpoints(created_at);

-- Triggers for automatic timestamp updates
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_sagas_updated_at 
    BEFORE UPDATE ON sagas 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_saga_steps_updated_at 
    BEFORE UPDATE ON saga_steps 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_saga_compensations_updated_at 
    BEFORE UPDATE ON saga_compensations 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Views for common queries
CREATE VIEW active_sagas AS 
SELECT 
    id,
    name,
    status,
    transaction_id,
    user_id,
    completed_steps,
    total_steps,
    current_step,
    created_at,
    started_at,
    updated_at,
    CASE 
        WHEN total_steps > 0 THEN (completed_steps::FLOAT / total_steps::FLOAT) * 100
        ELSE 0
    END as progress_percentage,
    EXTRACT(EPOCH FROM (COALESCE(completed_at, NOW()) - started_at)) * 1000 as execution_time_ms
FROM sagas 
WHERE status IN ('Created', 'Executing', 'Compensating', 'Paused');

CREATE VIEW completed_sagas AS
SELECT 
    id,
    name,
    status,
    transaction_id,
    user_id,
    completed_steps,
    total_steps,
    created_at,
    started_at,
    completed_at,
    execution_time_ms,
    retry_attempts,
    error_message
FROM sagas 
WHERE status IN ('Completed', 'Compensated', 'Failed', 'Cancelled', 'TimedOut');

-- Function to cleanup old completed sagas
CREATE OR REPLACE FUNCTION cleanup_old_sagas(older_than_hours INTEGER DEFAULT 24)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM sagas 
    WHERE status IN ('Completed', 'Compensated', 'Failed', 'Cancelled', 'TimedOut')
    AND completed_at < NOW() - INTERVAL '1 hour' * older_than_hours;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Grant permissions (adjust as needed for your setup)
-- GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO tracseq_transaction_user;
-- GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO tracseq_transaction_user;
-- GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO tracseq_transaction_user; 
