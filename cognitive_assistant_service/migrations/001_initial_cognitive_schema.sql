-- Cognitive Assistant Service Database Schema
-- This migration creates tables for storing AI query history and user sessions

-- Query history table for storing all AI interactions
CREATE TABLE IF NOT EXISTS query_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    query TEXT NOT NULL,
    response TEXT NOT NULL,
    confidence REAL NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
    response_time_ms BIGINT NOT NULL,
    query_type VARCHAR(50),
    user_role VARCHAR(50),
    context JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- User session table for maintaining conversation context
CREATE TABLE IF NOT EXISTS user_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    context JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_activity TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

-- AI insights cache for storing processed insights
CREATE TABLE IF NOT EXISTS ai_insights_cache (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    insight_type VARCHAR(100) NOT NULL,
    context_hash VARCHAR(64) NOT NULL, -- Hash of input context for cache key
    insights JSONB NOT NULL,
    confidence REAL NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '24 hours'
);

-- User preferences for AI assistant behavior
CREATE TABLE IF NOT EXISTS user_ai_preferences (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL UNIQUE,
    preferred_response_style VARCHAR(50) DEFAULT 'detailed', -- detailed, concise, technical
    notification_preferences JSONB DEFAULT '{}',
    proactive_suggestions_enabled BOOLEAN DEFAULT TRUE,
    preferred_confidence_threshold REAL DEFAULT 0.7,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Feedback table for improving AI responses
CREATE TABLE IF NOT EXISTS ai_response_feedback (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    query_history_id UUID NOT NULL REFERENCES query_history(id),
    user_id UUID NOT NULL,
    helpful BOOLEAN,
    accuracy_rating INTEGER CHECK (accuracy_rating >= 1 AND accuracy_rating <= 5),
    feedback_text TEXT,
    improvement_suggestions TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_query_history_user_id ON query_history(user_id);
CREATE INDEX IF NOT EXISTS idx_query_history_created_at ON query_history(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_query_history_confidence ON query_history(confidence);
CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_active ON user_sessions(is_active) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_ai_insights_cache_context_hash ON ai_insights_cache(context_hash);
CREATE INDEX IF NOT EXISTS idx_ai_insights_cache_expires_at ON ai_insights_cache(expires_at);
CREATE INDEX IF NOT EXISTS idx_ai_response_feedback_query_history_id ON ai_response_feedback(query_history_id);

-- Function to update last_activity timestamp
CREATE OR REPLACE FUNCTION update_session_activity()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_activity = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to automatically update session activity
CREATE TRIGGER update_session_activity_trigger
    BEFORE UPDATE ON user_sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_session_activity();

-- Function to clean up expired cache entries
CREATE OR REPLACE FUNCTION cleanup_expired_cache()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM ai_insights_cache WHERE expires_at < NOW();
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Sample data for testing (optional - can be removed in production)
INSERT INTO user_ai_preferences (user_id, preferred_response_style, proactive_suggestions_enabled)
VALUES 
    ('00000000-0000-0000-0000-000000000001', 'detailed', TRUE),
    ('00000000-0000-0000-0000-000000000002', 'concise', TRUE),
    ('00000000-0000-0000-0000-000000000003', 'technical', FALSE)
ON CONFLICT (user_id) DO NOTHING; 