-- TracSeq 2.0 - Event Store Database Schema
-- Event sourcing infrastructure for laboratory operations

-- Create events table
CREATE TABLE IF NOT EXISTS events (
    id UUID PRIMARY KEY,
    aggregate_id UUID NOT NULL,
    aggregate_type VARCHAR(255) NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    event_version INTEGER NOT NULL,
    event_data JSONB NOT NULL,
    metadata JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sequence_number BIGSERIAL UNIQUE NOT NULL,
    
    -- Indexes for performance
    CONSTRAINT unique_aggregate_version UNIQUE (aggregate_id, event_version)
);

-- Indexes for event queries
CREATE INDEX idx_events_aggregate_id ON events(aggregate_id);
CREATE INDEX idx_events_event_type ON events(event_type);
CREATE INDEX idx_events_created_at ON events(created_at);
CREATE INDEX idx_events_aggregate_type ON events(aggregate_type);
CREATE INDEX idx_events_sequence_number ON events(sequence_number);

-- Create snapshots table for performance optimization
CREATE TABLE IF NOT EXISTS snapshots (
    id UUID PRIMARY KEY,
    aggregate_id UUID NOT NULL,
    aggregate_type VARCHAR(255) NOT NULL,
    version INTEGER NOT NULL,
    snapshot_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Ensure only one snapshot per aggregate at a given version
    CONSTRAINT unique_aggregate_snapshot_version UNIQUE (aggregate_id, version)
);

-- Indexes for snapshot queries
CREATE INDEX idx_snapshots_aggregate_id ON snapshots(aggregate_id);
CREATE INDEX idx_snapshots_version ON snapshots(version DESC);

-- Create projections metadata table
CREATE TABLE IF NOT EXISTS projections (
    id UUID PRIMARY KEY,
    projection_name VARCHAR(255) NOT NULL UNIQUE,
    last_processed_sequence BIGINT NOT NULL DEFAULT 0,
    last_processed_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create event subscriptions table
CREATE TABLE IF NOT EXISTS event_subscriptions (
    id UUID PRIMARY KEY,
    subscription_name VARCHAR(255) NOT NULL UNIQUE,
    event_types TEXT[] NOT NULL,
    last_processed_sequence BIGINT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    retry_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create saga state table for distributed transactions
CREATE TABLE IF NOT EXISTS saga_states (
    id UUID PRIMARY KEY,
    saga_type VARCHAR(255) NOT NULL,
    saga_data JSONB NOT NULL,
    current_step VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL, -- 'running', 'completed', 'failed', 'compensating'
    correlation_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    
    -- Index for finding active sagas
    CONSTRAINT idx_saga_correlation UNIQUE (correlation_id)
);

CREATE INDEX idx_saga_states_status ON saga_states(status);
CREATE INDEX idx_saga_states_type ON saga_states(saga_type);

-- Function to update timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers for updated_at
CREATE TRIGGER update_projections_updated_at BEFORE UPDATE ON projections
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_event_subscriptions_updated_at BEFORE UPDATE ON event_subscriptions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_saga_states_updated_at BEFORE UPDATE ON saga_states
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Partitioning for events table (optional, for high-volume scenarios)
-- Example: Partition by month
/*
CREATE TABLE events_2024_01 PARTITION OF events
    FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');
    
CREATE TABLE events_2024_02 PARTITION OF events
    FOR VALUES FROM ('2024-02-01') TO ('2024-03-01');
*/

-- View for latest aggregate states
CREATE OR REPLACE VIEW latest_aggregate_versions AS
SELECT 
    aggregate_id,
    aggregate_type,
    MAX(event_version) as latest_version,
    COUNT(*) as event_count,
    MIN(created_at) as first_event_at,
    MAX(created_at) as last_event_at
FROM events
GROUP BY aggregate_id, aggregate_type;

-- Function to get aggregate history
CREATE OR REPLACE FUNCTION get_aggregate_history(
    p_aggregate_id UUID,
    p_from_version INTEGER DEFAULT NULL,
    p_to_version INTEGER DEFAULT NULL
) RETURNS TABLE (
    event_id UUID,
    event_type VARCHAR,
    event_version INTEGER,
    event_data JSONB,
    created_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        e.id,
        e.event_type,
        e.event_version,
        e.event_data,
        e.created_at
    FROM events e
    WHERE e.aggregate_id = p_aggregate_id
        AND (p_from_version IS NULL OR e.event_version >= p_from_version)
        AND (p_to_version IS NULL OR e.event_version <= p_to_version)
    ORDER BY e.event_version ASC;
END;
$$ LANGUAGE plpgsql;