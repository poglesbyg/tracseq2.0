-- Create security audit log table for authentication and authorization tracking
-- Migration: 20240320000004_create_security_audit_log

-- =============================================================================
-- SECURITY AUDIT LOG TABLE
-- =============================================================================

CREATE TABLE IF NOT EXISTS security_audit_log (
    -- Primary identifier
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Event classification
    event_type VARCHAR(50) NOT NULL,
    
    -- User information (nullable for anonymous events)
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    user_email VARCHAR(255),
    
    -- Request context
    ip_address INET,
    user_agent TEXT,
    
    -- Event details and metadata
    details JSONB DEFAULT '{}',
    
    -- Security severity level
    severity VARCHAR(20) NOT NULL CHECK (severity IN ('LOW', 'MEDIUM', 'HIGH', 'CRITICAL')),
    
    -- Timestamp
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================================================
-- USER SESSIONS TABLE (for session management)
-- =============================================================================

CREATE TABLE IF NOT EXISTS user_sessions (
    -- Session identifier
    session_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- User reference
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Session token (hashed)
    session_token VARCHAR(255) UNIQUE NOT NULL,
    
    -- Session metadata
    ip_address INET,
    user_agent TEXT,
    
    -- Session lifecycle
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_accessed TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- Logout tracking
    logout_reason VARCHAR(50),
    logged_out_at TIMESTAMPTZ
);

-- =============================================================================
-- QUERY LOG TABLE (for RAG system queries)
-- =============================================================================

CREATE TABLE IF NOT EXISTS query_log (
    -- Query identifier
    query_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Query content
    query_text TEXT NOT NULL,
    session_id VARCHAR(255),
    
    -- Response
    response_text TEXT,
    
    -- Performance metrics
    processing_time DECIMAL(10,3),
    chunks_retrieved INTEGER DEFAULT 0,
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================================================
-- EXTRACTION RESULTS TABLE (for RAG document processing)
-- =============================================================================

CREATE TABLE IF NOT EXISTS extraction_results (
    -- Extraction identifier
    extraction_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Associated submission
    submission_id VARCHAR(255),
    
    -- Extraction results
    success BOOLEAN NOT NULL DEFAULT FALSE,
    confidence_score DECIMAL(3,2),
    missing_fields TEXT[],
    warnings TEXT[],
    
    -- Performance
    processing_time DECIMAL(10,3),
    source_document TEXT,
    
    -- Extracted data
    extracted_data JSONB,
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================================================
-- DOCUMENT CHUNKS TABLE (for RAG vector storage metadata)
-- =============================================================================

CREATE TABLE IF NOT EXISTS document_chunks (
    -- Chunk identifier  
    chunk_id VARCHAR(255) PRIMARY KEY,
    
    -- Document reference
    document_id UUID,
    
    -- Chunk content and metadata
    content TEXT NOT NULL,
    chunk_index INTEGER NOT NULL,
    page_number INTEGER DEFAULT 1,
    
    -- Vector embedding (stored as array for PostgreSQL compatibility)
    embedding DECIMAL(8,6)[],
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================================================
-- DOCUMENTS TABLE (for RAG document tracking)
-- =============================================================================

CREATE TABLE IF NOT EXISTS documents (
    -- Document identifier
    document_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Associated submission
    submission_id VARCHAR(255),
    
    -- File information
    filename VARCHAR(255) NOT NULL,
    file_path TEXT,
    file_type VARCHAR(10),
    file_size BIGINT,
    
    -- Processing status
    processed BOOLEAN NOT NULL DEFAULT FALSE,
    processing_time DECIMAL(10,3),
    chunk_count INTEGER DEFAULT 0,
    
    -- Timestamps
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ
);

-- =============================================================================
-- INDEXES FOR PERFORMANCE
-- =============================================================================

-- Security audit log indexes (already created in previous migration)
-- CREATE INDEX IF NOT EXISTS idx_security_audit_log_user_id ON security_audit_log (user_id);
-- CREATE INDEX IF NOT EXISTS idx_security_audit_log_event_type ON security_audit_log (event_type);
-- CREATE INDEX IF NOT EXISTS idx_security_audit_log_timestamp ON security_audit_log (timestamp);
-- CREATE INDEX IF NOT EXISTS idx_security_audit_log_severity ON security_audit_log (severity);

-- User sessions indexes
-- CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions (user_id);
-- CREATE UNIQUE INDEX IF NOT EXISTS idx_user_sessions_session_token ON user_sessions (session_token);
-- CREATE INDEX IF NOT EXISTS idx_user_sessions_is_active ON user_sessions (is_active);
-- CREATE INDEX IF NOT EXISTS idx_user_sessions_expires_at ON user_sessions (expires_at);

-- Query log indexes
CREATE INDEX IF NOT EXISTS idx_query_log_session_id ON query_log (session_id);
CREATE INDEX IF NOT EXISTS idx_query_log_created_at ON query_log (created_at);

-- Document indexes
CREATE INDEX IF NOT EXISTS idx_documents_submission_id ON documents (submission_id);
CREATE INDEX IF NOT EXISTS idx_documents_processed ON documents (processed);
CREATE INDEX IF NOT EXISTS idx_documents_uploaded_at ON documents (uploaded_at);

-- Document chunks indexes
CREATE INDEX IF NOT EXISTS idx_document_chunks_document_id ON document_chunks (document_id);
CREATE INDEX IF NOT EXISTS idx_document_chunks_chunk_index ON document_chunks (chunk_index);

-- Extraction results indexes
CREATE INDEX IF NOT EXISTS idx_extraction_results_submission_id ON extraction_results (submission_id);
CREATE INDEX IF NOT EXISTS idx_extraction_results_success ON extraction_results (success);
CREATE INDEX IF NOT EXISTS idx_extraction_results_created_at ON extraction_results (created_at);

-- =============================================================================
-- DATA RETENTION POLICIES
-- =============================================================================

-- Security audit log retention (keep for 2 years for compliance)
-- Note: Implement as scheduled job or use pg_partman for partitioning
-- DELETE FROM security_audit_log WHERE timestamp < NOW() - INTERVAL '2 years';

-- Session cleanup (remove expired sessions daily)
-- DELETE FROM user_sessions WHERE expires_at < NOW() AND is_active = FALSE;

-- Query log retention (keep for 90 days)
-- DELETE FROM query_log WHERE created_at < NOW() - INTERVAL '90 days';

-- =============================================================================
-- COMMENTS AND DOCUMENTATION
-- =============================================================================

COMMENT ON TABLE security_audit_log IS 'Comprehensive audit trail for security events and user actions';
COMMENT ON TABLE user_sessions IS 'Active user sessions for authentication and session management';
COMMENT ON TABLE query_log IS 'RAG system query history for analytics and debugging';
COMMENT ON TABLE extraction_results IS 'Document processing results from RAG system';
COMMENT ON TABLE document_chunks IS 'Processed document chunks for vector search';
COMMENT ON TABLE documents IS 'Document metadata and processing status';

COMMENT ON COLUMN security_audit_log.severity IS 'Security event severity: LOW, MEDIUM, HIGH, CRITICAL';
COMMENT ON COLUMN user_sessions.session_token IS 'Hashed session token for security';
COMMENT ON COLUMN document_chunks.embedding IS 'Vector embedding for semantic search';
COMMENT ON COLUMN extraction_results.confidence_score IS 'Confidence score between 0.00 and 1.00'; 
