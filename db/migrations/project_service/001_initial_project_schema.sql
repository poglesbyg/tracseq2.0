-- Migration: Create Project Management schema
-- Version: 001
-- Description: Initial schema for project management, batches, and approval workflows

-- Create schema if not exists
CREATE SCHEMA IF NOT EXISTS project_service;

-- Projects table
CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_code VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    project_type VARCHAR(100) NOT NULL, -- 'research', 'clinical', 'validation', 'development'
    status VARCHAR(50) NOT NULL DEFAULT 'planning', -- 'planning', 'active', 'on_hold', 'completed', 'cancelled'
    priority VARCHAR(20) DEFAULT 'medium', -- 'low', 'medium', 'high', 'urgent'
    start_date DATE,
    target_end_date DATE,
    actual_end_date DATE,
    principal_investigator_id UUID NOT NULL,
    project_manager_id UUID,
    department VARCHAR(255),
    budget_approved DECIMAL(12,2),
    budget_used DECIMAL(12,2) DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by UUID NOT NULL
);

-- Project team members
CREATE TABLE project_team_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL,
    user_id UUID NOT NULL,
    role VARCHAR(100) NOT NULL, -- 'investigator', 'analyst', 'technician', 'collaborator'
    permissions JSONB DEFAULT '{}', -- specific permissions for this project
    joined_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    left_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT project_team_members_project_fkey FOREIGN KEY (project_id) REFERENCES projects(id),
    CONSTRAINT project_team_members_unique_user_project UNIQUE(project_id, user_id)
);

-- Batches table
CREATE TABLE batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    batch_number VARCHAR(100) NOT NULL UNIQUE,
    project_id UUID NOT NULL,
    batch_type VARCHAR(100) NOT NULL, -- 'sample_receipt', 'library_prep', 'sequencing', 'analysis'
    status VARCHAR(50) NOT NULL DEFAULT 'created', -- 'created', 'in_progress', 'pending_approval', 'approved', 'completed', 'failed'
    priority VARCHAR(20) DEFAULT 'medium',
    sample_count INTEGER DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by UUID NOT NULL,
    CONSTRAINT batches_project_fkey FOREIGN KEY (project_id) REFERENCES projects(id)
);

-- Batch workflow steps
CREATE TABLE batch_workflow_steps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    batch_id UUID NOT NULL,
    step_order INTEGER NOT NULL,
    step_name VARCHAR(255) NOT NULL,
    step_type VARCHAR(100) NOT NULL, -- 'process', 'qc', 'approval', 'documentation'
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- 'pending', 'in_progress', 'completed', 'failed', 'skipped'
    required BOOLEAN DEFAULT true,
    assigned_to UUID,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    completed_by UUID,
    results JSONB,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT batch_workflow_steps_batch_fkey FOREIGN KEY (batch_id) REFERENCES batches(id),
    CONSTRAINT batch_workflow_steps_unique_order UNIQUE(batch_id, step_order)
);

-- Project sign-offs table
CREATE TABLE project_signoffs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL,
    batch_id UUID,
    signoff_type VARCHAR(100) NOT NULL, -- 'project_initiation', 'batch_release', 'qc_approval', 'final_delivery'
    signoff_level VARCHAR(50) NOT NULL, -- 'technical', 'supervisor', 'manager', 'director'
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- 'pending', 'approved', 'rejected', 'conditional'
    required_by UUID NOT NULL,
    signed_by UUID,
    signed_at TIMESTAMPTZ,
    comments TEXT,
    conditions JSONB, -- any conditions for conditional approval
    expiry_date DATE, -- for time-limited approvals
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT project_signoffs_project_fkey FOREIGN KEY (project_id) REFERENCES projects(id),
    CONSTRAINT project_signoffs_batch_fkey FOREIGN KEY (batch_id) REFERENCES batches(id)
);

-- Template repository table
CREATE TABLE template_repository (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    category VARCHAR(100) NOT NULL, -- 'submission', 'protocol', 'report', 'analysis', 'documentation'
    file_type VARCHAR(50) NOT NULL, -- 'xlsx', 'docx', 'pdf', 'csv', 'json'
    version VARCHAR(50) NOT NULL,
    description TEXT,
    file_path VARCHAR(500) NOT NULL,
    file_size_bytes BIGINT,
    checksum VARCHAR(64), -- SHA-256 hash
    tags TEXT[],
    metadata JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    download_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by UUID NOT NULL,
    CONSTRAINT template_repository_unique_name_version UNIQUE(name, version)
);

-- Project files table
CREATE TABLE project_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL,
    batch_id UUID,
    parent_folder_id UUID,
    name VARCHAR(255) NOT NULL,
    file_type VARCHAR(20) NOT NULL, -- 'file' or 'folder'
    file_extension VARCHAR(50),
    file_path VARCHAR(1000) NOT NULL,
    file_size_bytes BIGINT,
    mime_type VARCHAR(255),
    checksum VARCHAR(64),
    metadata JSONB DEFAULT '{}',
    is_deleted BOOLEAN DEFAULT false,
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by UUID NOT NULL,
    CONSTRAINT project_files_project_fkey FOREIGN KEY (project_id) REFERENCES projects(id),
    CONSTRAINT project_files_batch_fkey FOREIGN KEY (batch_id) REFERENCES batches(id),
    CONSTRAINT project_files_parent_fkey FOREIGN KEY (parent_folder_id) REFERENCES project_files(id)
);

-- Permission queue for batch progression
CREATE TABLE permission_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    batch_id UUID NOT NULL,
    permission_type VARCHAR(100) NOT NULL, -- 'proceed_to_sequencing', 'release_results', 'archive_data'
    requested_by UUID NOT NULL,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- 'pending', 'approved', 'rejected', 'expired'
    priority VARCHAR(20) DEFAULT 'medium',
    reason TEXT NOT NULL,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    rejection_reason TEXT,
    expires_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT permission_queue_batch_fkey FOREIGN KEY (batch_id) REFERENCES batches(id)
);

-- Indexes for performance
CREATE INDEX idx_projects_status ON projects(status);
CREATE INDEX idx_projects_project_code ON projects(project_code);
CREATE INDEX idx_projects_pi_id ON projects(principal_investigator_id);
CREATE INDEX idx_project_team_members_user_id ON project_team_members(user_id);
CREATE INDEX idx_project_team_members_project_id ON project_team_members(project_id);
CREATE INDEX idx_batches_batch_number ON batches(batch_number);
CREATE INDEX idx_batches_project_id ON batches(project_id);
CREATE INDEX idx_batches_status ON batches(status);
CREATE INDEX idx_batch_workflow_steps_batch_id ON batch_workflow_steps(batch_id);
CREATE INDEX idx_batch_workflow_steps_status ON batch_workflow_steps(status);
CREATE INDEX idx_project_signoffs_project_id ON project_signoffs(project_id);
CREATE INDEX idx_project_signoffs_batch_id ON project_signoffs(batch_id);
CREATE INDEX idx_project_signoffs_status ON project_signoffs(status);
CREATE INDEX idx_template_repository_category ON template_repository(category);
CREATE INDEX idx_template_repository_active ON template_repository(is_active);
CREATE INDEX idx_project_files_project_id ON project_files(project_id);
CREATE INDEX idx_project_files_batch_id ON project_files(batch_id);
CREATE INDEX idx_project_files_parent_folder_id ON project_files(parent_folder_id);
CREATE INDEX idx_project_files_name_trgm ON project_files USING gin(name gin_trgm_ops);
CREATE INDEX idx_permission_queue_batch_id ON permission_queue(batch_id);
CREATE INDEX idx_permission_queue_status ON permission_queue(status);

-- Full text search indexes
CREATE INDEX idx_projects_search ON projects USING gin(to_tsvector('english', name || ' ' || COALESCE(description, '')));
CREATE INDEX idx_template_repository_search ON template_repository USING gin(to_tsvector('english', name || ' ' || COALESCE(description, '')));

-- Triggers for updated_at
CREATE TRIGGER trigger_projects_updated_at
    BEFORE UPDATE ON projects
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trigger_project_team_members_updated_at
    BEFORE UPDATE ON project_team_members
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trigger_batches_updated_at
    BEFORE UPDATE ON batches
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trigger_batch_workflow_steps_updated_at
    BEFORE UPDATE ON batch_workflow_steps
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trigger_project_signoffs_updated_at
    BEFORE UPDATE ON project_signoffs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trigger_template_repository_updated_at
    BEFORE UPDATE ON template_repository
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trigger_project_files_updated_at
    BEFORE UPDATE ON project_files
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trigger_permission_queue_updated_at
    BEFORE UPDATE ON permission_queue
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- Comments
COMMENT ON TABLE projects IS 'Project management and tracking';
COMMENT ON TABLE project_team_members IS 'Project team assignments and permissions';
COMMENT ON TABLE batches IS 'Batch tracking for samples and workflows';
COMMENT ON TABLE batch_workflow_steps IS 'Workflow steps and progress for batches';
COMMENT ON TABLE project_signoffs IS 'Approval and sign-off tracking';
COMMENT ON TABLE template_repository IS 'Downloadable template files';
COMMENT ON TABLE project_files IS 'File management for projects';
COMMENT ON TABLE permission_queue IS 'Permission requests for batch progression';