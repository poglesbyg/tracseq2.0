CREATE TYPE job_status AS ENUM ('pending', 'in_progress', 'completed', 'failed');

CREATE TABLE sequencing_jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    status job_status NOT NULL DEFAULT 'pending',
    sample_sheet_path VARCHAR(1024) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX idx_sequencing_jobs_status ON sequencing_jobs(status);
CREATE INDEX idx_sequencing_jobs_created_at ON sequencing_jobs(created_at); 
