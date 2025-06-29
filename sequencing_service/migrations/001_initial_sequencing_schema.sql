-- Sequencing Service - Initial Schema Migration
-- File: sequencing_service/migrations/001_initial_sequencing_schema.sql

-- Create custom types
CREATE TYPE sequencing_platform AS ENUM ('illumina_novaseq', 'illumina_miseq', 'illumina_hiseq', 'oxford_nanopore', 'pacbio_sequel', 'ion_torrent');
CREATE TYPE job_status AS ENUM ('pending', 'queued', 'running', 'completed', 'failed', 'cancelled', 'paused');
CREATE TYPE priority_level AS ENUM ('low', 'normal', 'high', 'urgent', 'stat');
CREATE TYPE run_type AS ENUM ('whole_genome', 'exome', 'targeted_panel', 'rna_seq', 'chip_seq', 'bisulfite_seq', 'amplicon');

-- Sequencing Instruments table
CREATE TABLE sequencing_instruments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    instrument_id VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    platform sequencing_platform NOT NULL,
    model VARCHAR(255) NOT NULL,
    serial_number VARCHAR(255) UNIQUE,
    location VARCHAR(255),
    status VARCHAR(50) DEFAULT 'operational', -- 'operational', 'maintenance', 'offline', 'error'
    max_throughput_gb DECIMAL(10,2),
    max_read_length INTEGER,
    supported_run_types run_type[] NOT NULL,
    installation_date DATE,
    last_maintenance_date TIMESTAMPTZ,
    next_maintenance_date TIMESTAMPTZ,
    maintenance_schedule JSONB DEFAULT '{}',
    configuration JSONB DEFAULT '{}',
    utilization_stats JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Sequencing Jobs table
CREATE TABLE sequencing_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id VARCHAR(255) NOT NULL UNIQUE,
    job_name VARCHAR(255) NOT NULL,
    description TEXT,
    status job_status DEFAULT 'pending',
    priority priority_level DEFAULT 'normal',
    run_type run_type NOT NULL,
    platform sequencing_platform NOT NULL,
    instrument_id UUID,
    requester_id UUID, -- Reference to users table
    project_id VARCHAR(255),
    sample_count INTEGER NOT NULL DEFAULT 0,
    estimated_duration_hours INTEGER,
    actual_duration_hours DECIMAL(5,2),
    estimated_cost DECIMAL(10,2),
    actual_cost DECIMAL(10,2),
    sequencing_parameters JSONB DEFAULT '{}', -- Read length, coverage, etc.
    library_prep_protocol VARCHAR(255),
    index_strategy VARCHAR(100),
    quality_metrics JSONB DEFAULT '{}',
    output_specifications JSONB DEFAULT '{}',
    submission_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    scheduled_start TIMESTAMPTZ,
    actual_start TIMESTAMPTZ,
    completion_date TIMESTAMPTZ,
    data_delivery_date TIMESTAMPTZ,
    notes TEXT,
    error_message TEXT,
    cancellation_reason TEXT,
    approval_required BOOLEAN DEFAULT false,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT sequencing_jobs_instrument_fkey FOREIGN KEY (instrument_id) REFERENCES sequencing_instruments(id),
    CONSTRAINT sequencing_jobs_sample_count_check CHECK (sample_count >= 0),
    CONSTRAINT sequencing_jobs_duration_check CHECK (actual_duration_hours IS NULL OR actual_duration_hours >= 0)
);

-- Job Samples table (many-to-many between jobs and samples)
CREATE TABLE job_samples (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id UUID NOT NULL,
    sample_id UUID NOT NULL, -- Reference to samples table
    sample_position INTEGER,
    barcode_sequence VARCHAR(255),
    index_name VARCHAR(100),
    library_concentration DECIMAL(8,3),
    library_volume DECIMAL(8,3),
    target_coverage DECIMAL(6,2),
    actual_coverage DECIMAL(6,2),
    quality_score DECIMAL(5,2),
    pass_filter_reads BIGINT,
    total_reads BIGINT,
    sample_notes TEXT,
    processing_status VARCHAR(50) DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT job_samples_job_fkey FOREIGN KEY (job_id) REFERENCES sequencing_jobs(id) ON DELETE CASCADE,
    CONSTRAINT job_samples_position_check CHECK (sample_position IS NULL OR sample_position > 0),
    UNIQUE(job_id, sample_id)
);

-- Sequencing Runs table (actual instrument runs)
CREATE TABLE sequencing_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id VARCHAR(255) NOT NULL UNIQUE,
    instrument_id UUID NOT NULL,
    job_id UUID,
    flowcell_id VARCHAR(255),
    run_folder VARCHAR(500),
    run_parameters JSONB DEFAULT '{}',
    chemistry_version VARCHAR(100),
    cycle_count INTEGER,
    read_structure VARCHAR(255), -- e.g., "150T8B8B150T"
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    status VARCHAR(50) DEFAULT 'pending',
    cluster_density DECIMAL(8,2),
    percent_pf DECIMAL(5,2), -- Percent passing filter
    q30_score DECIMAL(5,2),
    total_yield_gb DECIMAL(10,3),
    run_metrics JSONB DEFAULT '{}',
    error_rate DECIMAL(5,4),
    phasing_prephasing JSONB DEFAULT '{}',
    demultiplexing_stats JSONB DEFAULT '{}',
    operator_id UUID,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT sequencing_runs_instrument_fkey FOREIGN KEY (instrument_id) REFERENCES sequencing_instruments(id),
    CONSTRAINT sequencing_runs_job_fkey FOREIGN KEY (job_id) REFERENCES sequencing_jobs(id),
    CONSTRAINT sequencing_runs_cycle_check CHECK (cycle_count IS NULL OR cycle_count > 0)
);

-- Analysis Workflows table
CREATE TABLE analysis_workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    workflow_type VARCHAR(100) NOT NULL, -- 'primary', 'secondary', 'tertiary'
    pipeline_version VARCHAR(100) NOT NULL,
    supported_run_types run_type[] NOT NULL,
    input_requirements JSONB DEFAULT '{}',
    output_specifications JSONB DEFAULT '{}',
    compute_requirements JSONB DEFAULT '{}',
    expected_runtime_hours DECIMAL(5,2),
    workflow_definition JSONB NOT NULL, -- Pipeline steps and parameters
    validation_criteria JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    is_default BOOLEAN DEFAULT false,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Analysis Jobs table (processing of sequencing data)
CREATE TABLE analysis_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    analysis_id VARCHAR(255) NOT NULL UNIQUE,
    sequencing_run_id UUID NOT NULL,
    workflow_id UUID NOT NULL,
    job_name VARCHAR(255) NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    priority INTEGER DEFAULT 5, -- 1 (highest) to 10 (lowest)
    input_data_path TEXT NOT NULL,
    output_data_path TEXT,
    compute_environment VARCHAR(100), -- 'local', 'cloud_aws', 'hpc_cluster'
    allocated_cpus INTEGER,
    allocated_memory_gb INTEGER,
    allocated_storage_gb INTEGER,
    actual_runtime_hours DECIMAL(5,2),
    cpu_hours_used DECIMAL(8,2),
    memory_peak_gb DECIMAL(8,2),
    storage_used_gb DECIMAL(10,2),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    analysis_parameters JSONB DEFAULT '{}',
    quality_metrics JSONB DEFAULT '{}',
    output_files JSONB DEFAULT '[]',
    log_files JSONB DEFAULT '[]',
    error_message TEXT,
    operator_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT analysis_jobs_run_fkey FOREIGN KEY (sequencing_run_id) REFERENCES sequencing_runs(id),
    CONSTRAINT analysis_jobs_workflow_fkey FOREIGN KEY (workflow_id) REFERENCES analysis_workflows(id),
    CONSTRAINT analysis_jobs_priority_check CHECK (priority >= 1 AND priority <= 10)
);

-- Data Deliverables table (final outputs)
CREATE TABLE data_deliverables (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    deliverable_name VARCHAR(255) NOT NULL,
    job_id UUID NOT NULL,
    analysis_job_id UUID,
    file_type VARCHAR(100) NOT NULL, -- 'fastq', 'bam', 'vcf', 'report', 'qc_metrics'
    file_path TEXT NOT NULL,
    file_size_bytes BIGINT,
    file_checksum VARCHAR(255),
    compression_type VARCHAR(50),
    delivery_method VARCHAR(100), -- 'download_link', 'sftp', 'cloud_storage', 'physical_media'
    access_url TEXT,
    expiration_date TIMESTAMPTZ,
    download_count INTEGER DEFAULT 0,
    last_accessed TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    is_sensitive BOOLEAN DEFAULT false,
    encryption_status VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT data_deliverables_job_fkey FOREIGN KEY (job_id) REFERENCES sequencing_jobs(id),
    CONSTRAINT data_deliverables_analysis_fkey FOREIGN KEY (analysis_job_id) REFERENCES analysis_jobs(id),
    CONSTRAINT data_deliverables_size_check CHECK (file_size_bytes IS NULL OR file_size_bytes >= 0)
);

-- Sequencing Statistics table (for monitoring and reporting)
CREATE TABLE sequencing_statistics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    date DATE NOT NULL,
    instrument_id UUID,
    platform sequencing_platform,
    total_jobs INTEGER DEFAULT 0,
    completed_jobs INTEGER DEFAULT 0,
    failed_jobs INTEGER DEFAULT 0,
    total_samples INTEGER DEFAULT 0,
    total_gigabases DECIMAL(12,3) DEFAULT 0,
    average_q30_score DECIMAL(5,2),
    instrument_utilization DECIMAL(5,2), -- Percentage
    turnaround_time_hours DECIMAL(6,2),
    success_rate DECIMAL(5,2),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT sequencing_statistics_instrument_fkey FOREIGN KEY (instrument_id) REFERENCES sequencing_instruments(id),
    UNIQUE(date, instrument_id, platform)
);

-- Create indexes for performance
CREATE INDEX idx_sequencing_instruments_platform ON sequencing_instruments(platform);
CREATE INDEX idx_sequencing_instruments_status ON sequencing_instruments(status);
CREATE INDEX idx_sequencing_instruments_active ON sequencing_instruments(is_active);

CREATE INDEX idx_sequencing_jobs_status ON sequencing_jobs(status);
CREATE INDEX idx_sequencing_jobs_priority ON sequencing_jobs(priority);
CREATE INDEX idx_sequencing_jobs_platform ON sequencing_jobs(platform);
CREATE INDEX idx_sequencing_jobs_run_type ON sequencing_jobs(run_type);
CREATE INDEX idx_sequencing_jobs_submission_date ON sequencing_jobs(submission_date);
CREATE INDEX idx_sequencing_jobs_requester_id ON sequencing_jobs(requester_id);
CREATE INDEX idx_sequencing_jobs_instrument_id ON sequencing_jobs(instrument_id);

CREATE INDEX idx_job_samples_job_id ON job_samples(job_id);
CREATE INDEX idx_job_samples_sample_id ON job_samples(sample_id);
CREATE INDEX idx_job_samples_status ON job_samples(processing_status);

CREATE INDEX idx_sequencing_runs_instrument_id ON sequencing_runs(instrument_id);
CREATE INDEX idx_sequencing_runs_job_id ON sequencing_runs(job_id);
CREATE INDEX idx_sequencing_runs_started_at ON sequencing_runs(started_at);
CREATE INDEX idx_sequencing_runs_status ON sequencing_runs(status);

CREATE INDEX idx_analysis_workflows_type ON analysis_workflows(workflow_type);
CREATE INDEX idx_analysis_workflows_active ON analysis_workflows(is_active);
CREATE INDEX idx_analysis_workflows_run_types_gin ON analysis_workflows USING gin(supported_run_types);

CREATE INDEX idx_analysis_jobs_run_id ON analysis_jobs(sequencing_run_id);
CREATE INDEX idx_analysis_jobs_workflow_id ON analysis_jobs(workflow_id);
CREATE INDEX idx_analysis_jobs_status ON analysis_jobs(status);
CREATE INDEX idx_analysis_jobs_priority ON analysis_jobs(priority);
CREATE INDEX idx_analysis_jobs_started_at ON analysis_jobs(started_at);

CREATE INDEX idx_data_deliverables_job_id ON data_deliverables(job_id);
CREATE INDEX idx_data_deliverables_analysis_job_id ON data_deliverables(analysis_job_id);
CREATE INDEX idx_data_deliverables_file_type ON data_deliverables(file_type);
CREATE INDEX idx_data_deliverables_created_at ON data_deliverables(created_at);

CREATE INDEX idx_sequencing_statistics_date ON sequencing_statistics(date);
CREATE INDEX idx_sequencing_statistics_instrument_id ON sequencing_statistics(instrument_id);
CREATE INDEX idx_sequencing_statistics_platform ON sequencing_statistics(platform);

-- Create trigger functions
CREATE OR REPLACE FUNCTION update_sequencing_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers
CREATE TRIGGER trigger_sequencing_instruments_updated_at
    BEFORE UPDATE ON sequencing_instruments
    FOR EACH ROW
    EXECUTE FUNCTION update_sequencing_updated_at();

CREATE TRIGGER trigger_sequencing_jobs_updated_at
    BEFORE UPDATE ON sequencing_jobs
    FOR EACH ROW
    EXECUTE FUNCTION update_sequencing_updated_at();

CREATE TRIGGER trigger_analysis_workflows_updated_at
    BEFORE UPDATE ON analysis_workflows
    FOR EACH ROW
    EXECUTE FUNCTION update_sequencing_updated_at();

-- Insert sample instruments
INSERT INTO sequencing_instruments (instrument_id, name, platform, model, supported_run_types, max_throughput_gb, max_read_length) VALUES
    ('NOVASEQ001', 'NovaSeq 6000 #1', 'illumina_novaseq', 'NovaSeq 6000', ARRAY['whole_genome', 'exome', 'rna_seq'], 6000.0, 250),
    ('MISEQ001', 'MiSeq #1', 'illumina_miseq', 'MiSeq v3', ARRAY['targeted_panel', 'amplicon', 'exome'], 15.0, 300),
    ('NANOPORE001', 'MinION #1', 'oxford_nanopore', 'MinION Mk1C', ARRAY['whole_genome', 'rna_seq'], 50.0, 2000000);

-- Insert sample workflows
INSERT INTO analysis_workflows (workflow_name, workflow_type, pipeline_version, supported_run_types, workflow_definition) VALUES
    ('Standard WGS Pipeline', 'secondary', 'v2.1.0', ARRAY['whole_genome'], 
     '{"steps": ["fastp_qc", "bwa_alignment", "gatk_variant_calling", "annotation", "qc_metrics"]}'),
    ('RNA-seq Analysis', 'secondary', 'v1.8.0', ARRAY['rna_seq'],
     '{"steps": ["star_alignment", "salmon_quantification", "deseq2_analysis", "pathway_analysis"]}'),
    ('Targeted Panel Analysis', 'secondary', 'v1.5.0', ARRAY['targeted_panel'],
     '{"steps": ["bwa_alignment", "variant_calling", "annotation", "coverage_analysis"]}');

COMMENT ON TABLE sequencing_instruments IS 'Sequencing instruments and their specifications';
COMMENT ON TABLE sequencing_jobs IS 'Sequencing job requests and tracking';
COMMENT ON TABLE job_samples IS 'Samples associated with sequencing jobs';
COMMENT ON TABLE sequencing_runs IS 'Actual sequencing runs on instruments';
COMMENT ON TABLE analysis_workflows IS 'Data analysis workflow definitions';
COMMENT ON TABLE analysis_jobs IS 'Data analysis job executions';
COMMENT ON TABLE data_deliverables IS 'Final data outputs and deliverables';
COMMENT ON TABLE sequencing_statistics IS 'Aggregated statistics for sequencing operations'; 