-- Event Service - Initial Schema Migration
-- File: event_service/migrations/001_initial_event_schema.sql

-- Create custom types
CREATE TYPE event_status AS ENUM ('pending', 'processing', 'completed', 'failed', 'retrying', 'dead_letter');
CREATE TYPE event_priority AS ENUM ('low', 'normal', 'high', 'critical');

-- Event Streams table (event categories/topics)
CREATE TABLE event_streams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    schema_version VARCHAR(50) NOT NULL DEFAULT '1.0.0',
    retention_hours INTEGER DEFAULT 168, -- 7 days
    partition_key VARCHAR(100),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Events table (individual event messages)
CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id UUID NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    event_version VARCHAR(50) DEFAULT '1.0.0',
    source_service VARCHAR(100) NOT NULL,
    correlation_id VARCHAR(255),
    causation_id VARCHAR(255),
    aggregate_id VARCHAR(255),
    sequence_number BIGSERIAL,
    event_data JSONB NOT NULL,
    metadata JSONB DEFAULT '{}',
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ,
    
    CONSTRAINT events_stream_fkey FOREIGN KEY (stream_id) REFERENCES event_streams(id)
);

-- Event Handlers table (processors for events)
CREATE TABLE event_handlers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    handler_name VARCHAR(255) NOT NULL UNIQUE,
    stream_id UUID NOT NULL,
    handler_type VARCHAR(100) NOT NULL, -- 'sync', 'async', 'scheduled'
    event_filter JSONB DEFAULT '{}', -- Conditions for processing
    retry_policy JSONB DEFAULT '{"max_retries": 3, "backoff_ms": [1000, 5000, 15000]}',
    dead_letter_after INTEGER DEFAULT 3,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT event_handlers_stream_fkey FOREIGN KEY (stream_id) REFERENCES event_streams(id)
);

-- Event Processing table (tracking handler executions)
CREATE TABLE event_processing (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL,
    handler_id UUID NOT NULL,
    status event_status DEFAULT 'pending',
    priority event_priority DEFAULT 'normal',
    attempt_count INTEGER DEFAULT 0,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error_message TEXT,
    processing_duration_ms INTEGER,
    result_data JSONB,
    next_retry_at TIMESTAMPTZ,
    
    CONSTRAINT event_processing_event_fkey FOREIGN KEY (event_id) REFERENCES events(id),
    CONSTRAINT event_processing_handler_fkey FOREIGN KEY (handler_id) REFERENCES event_handlers(id),
    UNIQUE(event_id, handler_id)
);

-- Create indexes
CREATE INDEX idx_events_stream_id ON events(stream_id);
CREATE INDEX idx_events_type ON events(event_type);
CREATE INDEX idx_events_occurred_at ON events(occurred_at);
CREATE INDEX idx_events_correlation_id ON events(correlation_id);

CREATE INDEX idx_event_processing_status ON event_processing(status);
CREATE INDEX idx_event_processing_next_retry ON event_processing(next_retry_at);

-- Insert sample streams and handlers
INSERT INTO event_streams (stream_name, description) VALUES
    ('laboratory.samples', 'Sample lifecycle events'),
    ('laboratory.storage', 'Storage and location events'),
    ('laboratory.qc', 'Quality control events'),
    ('laboratory.notifications', 'Notification events');

COMMENT ON TABLE event_streams IS 'Event stream definitions and configurations';
COMMENT ON TABLE events IS 'Individual event messages in the system';
COMMENT ON TABLE event_handlers IS 'Event processing handlers and their configurations';
COMMENT ON TABLE event_processing IS 'Tracking of event processing attempts and status'; 