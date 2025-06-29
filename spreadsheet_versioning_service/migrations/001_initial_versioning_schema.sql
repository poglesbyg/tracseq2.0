-- Spreadsheet Versioning Service - Initial Schema Migration
-- File: spreadsheet_versioning_service/migrations/001_initial_versioning_schema.sql

-- Create custom types
CREATE TYPE version_status AS ENUM ('draft', 'published', 'archived', 'superseded');
CREATE TYPE change_type AS ENUM ('create', 'update', 'delete', 'restore', 'merge');

-- Spreadsheet Versions table
CREATE TABLE spreadsheet_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    spreadsheet_id UUID NOT NULL, -- Reference to main spreadsheet
    version_number INTEGER NOT NULL,
    version_name VARCHAR(255),
    status version_status DEFAULT 'draft',
    content_hash VARCHAR(255) NOT NULL,
    file_size_bytes BIGINT NOT NULL,
    row_count INTEGER DEFAULT 0,
    column_count INTEGER DEFAULT 0,
    sheet_names JSONB DEFAULT '[]',
    content_summary JSONB DEFAULT '{}',
    metadata JSONB DEFAULT '{}',
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ,
    archived_at TIMESTAMPTZ,
    
    CONSTRAINT spreadsheet_versions_number_check CHECK (version_number > 0),
    CONSTRAINT spreadsheet_versions_size_check CHECK (file_size_bytes >= 0),
    UNIQUE(spreadsheet_id, version_number)
);

-- Version Changes table
CREATE TABLE version_changes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    version_id UUID NOT NULL,
    change_type change_type NOT NULL,
    sheet_name VARCHAR(255),
    cell_range VARCHAR(100), -- e.g., "A1:C10"
    old_value JSONB,
    new_value JSONB,
    change_summary TEXT,
    affected_rows JSONB DEFAULT '[]',
    affected_columns JSONB DEFAULT '[]',
    change_metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT version_changes_version_fkey FOREIGN KEY (version_id) REFERENCES spreadsheet_versions(id) ON DELETE CASCADE
);

-- Collaboration Sessions table
CREATE TABLE collaboration_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    spreadsheet_id UUID NOT NULL,
    session_name VARCHAR(255),
    active_users JSONB DEFAULT '[]',
    lock_status JSONB DEFAULT '{}', -- Cell/sheet locks
    last_activity TIMESTAMPTZ DEFAULT NOW(),
    session_metadata JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Version Comments table
CREATE TABLE version_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    version_id UUID NOT NULL,
    comment_text TEXT NOT NULL,
    cell_reference VARCHAR(50), -- e.g., "Sheet1!A1"
    commented_by UUID,
    parent_comment_id UUID, -- For replies
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT version_comments_version_fkey FOREIGN KEY (version_id) REFERENCES spreadsheet_versions(id) ON DELETE CASCADE,
    CONSTRAINT version_comments_parent_fkey FOREIGN KEY (parent_comment_id) REFERENCES version_comments(id)
);

-- Create indexes
CREATE INDEX idx_spreadsheet_versions_spreadsheet_id ON spreadsheet_versions(spreadsheet_id);
CREATE INDEX idx_spreadsheet_versions_status ON spreadsheet_versions(status);
CREATE INDEX idx_spreadsheet_versions_created_at ON spreadsheet_versions(created_at);

CREATE INDEX idx_version_changes_version_id ON version_changes(version_id);
CREATE INDEX idx_version_changes_type ON version_changes(change_type);
CREATE INDEX idx_version_changes_created_at ON version_changes(created_at);

CREATE INDEX idx_collaboration_sessions_spreadsheet_id ON collaboration_sessions(spreadsheet_id);
CREATE INDEX idx_collaboration_sessions_active ON collaboration_sessions(is_active);

CREATE INDEX idx_version_comments_version_id ON version_comments(version_id);
CREATE INDEX idx_version_comments_created_at ON version_comments(created_at);

COMMENT ON TABLE spreadsheet_versions IS 'Version history for spreadsheets';
COMMENT ON TABLE version_changes IS 'Detailed change tracking for spreadsheet versions';
COMMENT ON TABLE collaboration_sessions IS 'Active collaboration sessions for spreadsheets';
COMMENT ON TABLE version_comments IS 'Comments and annotations on spreadsheet versions'; 