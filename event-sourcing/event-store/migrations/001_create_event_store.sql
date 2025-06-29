-- Event Store Database Schema for TracSeq 2.0
-- This migration creates the foundational tables for event sourcing

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create event streams table
CREATE TABLE event_streams (
    stream_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    stream_name VARCHAR(255) NOT NULL UNIQUE,
    stream_type VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

-- Create events table
CREATE TABLE events (
    event_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    stream_id UUID NOT NULL REFERENCES event_streams(stream_id),
    event_type VARCHAR(100) NOT NULL,
    event_version INTEGER NOT NULL,
    event_data JSONB NOT NULL,
    event_metadata JSONB DEFAULT '{}',
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sequence_number BIGSERIAL,
    correlation_id UUID,
    causation_id UUID,
    
    -- Ensure event ordering within a stream
    UNIQUE(stream_id, sequence_number)
);

-- Create snapshots table for event sourcing optimization
CREATE TABLE snapshots (
    snapshot_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    stream_id UUID NOT NULL REFERENCES event_streams(stream_id),
    snapshot_version INTEGER NOT NULL,
    snapshot_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Only one snapshot per stream version
    UNIQUE(stream_id, snapshot_version)
);

-- Create indexes for performance
CREATE INDEX idx_events_stream_id ON events(stream_id);
CREATE INDEX idx_events_event_type ON events(event_type);
CREATE INDEX idx_events_occurred_at ON events(occurred_at);
CREATE INDEX idx_events_correlation_id ON events(correlation_id);
CREATE INDEX idx_events_sequence_number ON events(sequence_number);

CREATE INDEX idx_event_streams_name ON event_streams(stream_name);
CREATE INDEX idx_event_streams_type ON event_streams(stream_type);

CREATE INDEX idx_snapshots_stream_id ON snapshots(stream_id);
CREATE INDEX idx_snapshots_version ON snapshots(snapshot_version);

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger for event_streams updated_at
CREATE TRIGGER update_event_streams_updated_at
    BEFORE UPDATE ON event_streams
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Insert initial laboratory event stream types
INSERT INTO event_streams (stream_name, stream_type, metadata) VALUES 
('laboratory-samples', 'SampleAggregate', '{"description": "Sample lifecycle events"}'),
('laboratory-storage', 'StorageAggregate', '{"description": "Storage location and temperature events"}'),
('laboratory-sequencing', 'SequencingAggregate', '{"description": "Sequencing workflow events"}'),
('laboratory-notifications', 'NotificationAggregate', '{"description": "Notification and alert events"}'),
('laboratory-audit', 'AuditAggregate', '{"description": "System audit and compliance events"}');

-- Create a view for event stream summaries
CREATE VIEW event_stream_summary AS
SELECT 
    es.stream_id,
    es.stream_name,
    es.stream_type,
    es.created_at,
    es.updated_at,
    COUNT(e.event_id) as event_count,
    MAX(e.sequence_number) as latest_sequence,
    MAX(e.occurred_at) as latest_event_time
FROM event_streams es
LEFT JOIN events e ON es.stream_id = e.stream_id
GROUP BY es.stream_id, es.stream_name, es.stream_type, es.created_at, es.updated_at;

-- Grant permissions (assuming event_store_user exists)
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO event_store_user;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO event_store_user; 