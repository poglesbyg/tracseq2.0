-- Migration: Extended QC schema for library prep and sequencing QC
-- Version: 002
-- Description: Adds comprehensive QC tracking for library prep and sequencing workflows

-- QC metrics definitions table
CREATE TABLE qc_metric_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    metric_type VARCHAR(100) NOT NULL, -- 'library_prep', 'sequencing', 'sample_quality', 'data_quality'
    data_type VARCHAR(50) NOT NULL, -- 'numeric', 'boolean', 'text', 'json'
    unit VARCHAR(50),
    min_value DECIMAL(20,6),
    max_value DECIMAL(20,6),
    warning_threshold_low DECIMAL(20,6),
    warning_threshold_high DECIMAL(20,6),
    fail_threshold_low DECIMAL(20,6),
    fail_threshold_high DECIMAL(20,6),
    description TEXT,
    calculation_method TEXT,
    is_required BOOLEAN DEFAULT true,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT qc_metric_definitions_unique_name_type UNIQUE(name, metric_type)
);

-- Library prep QC results
CREATE TABLE library_prep_qc (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    library_prep_id UUID NOT NULL,
    qc_batch_id VARCHAR(255) NOT NULL,
    qc_date DATE NOT NULL,
    performed_by UUID NOT NULL,
    overall_status VARCHAR(50) NOT NULL DEFAULT 'pending', -- 'pending', 'passed', 'failed', 'conditional', 'in_review'
    concentration_ngul DECIMAL(10,4),
    volume_ul DECIMAL(10,2),
    total_yield_ng DECIMAL(10,2),
    fragment_size_bp INTEGER,
    fragment_size_cv DECIMAL(5,2),
    bioanalyzer_rin DECIMAL(3,1),
    bioanalyzer_trace_path VARCHAR(500),
    qubit_concentration DECIMAL(10,4),
    nanodrop_260_280 DECIMAL(4,2),
    nanodrop_260_230 DECIMAL(4,2),
    contamination_status VARCHAR(50), -- 'none_detected', 'minor', 'significant'
    adapter_contamination DECIMAL(5,2), -- percentage
    notes TEXT,
    raw_data JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT library_prep_qc_library_prep_fkey FOREIGN KEY (library_prep_id) REFERENCES library_preparations(id)
);

-- Sequencing run QC metrics
CREATE TABLE sequencing_run_qc (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sequencing_run_id UUID NOT NULL,
    flow_cell_id VARCHAR(255),
    qc_type VARCHAR(100) NOT NULL, -- 'pre_run', 'mid_run', 'post_run', 'demux'
    qc_timestamp TIMESTAMPTZ NOT NULL,
    overall_status VARCHAR(50) NOT NULL DEFAULT 'pending',
    cluster_density_k_per_mm2 DECIMAL(10,2),
    cluster_pf_percent DECIMAL(5,2),
    phix_aligned_percent DECIMAL(5,2),
    error_rate_percent DECIMAL(5,3),
    q30_percent DECIMAL(5,2),
    total_reads_pf_m DECIMAL(10,2),
    total_yield_gb DECIMAL(10,2),
    intensity_cycle_1 JSONB, -- per-lane intensities
    index_metrics JSONB, -- index balance and representation
    lane_metrics JSONB, -- per-lane detailed metrics
    run_summary JSONB,
    alerts TEXT[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT sequencing_run_qc_run_fkey FOREIGN KEY (sequencing_run_id) REFERENCES sequencing_runs(id)
);

-- QC review and approval workflow
CREATE TABLE qc_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(100) NOT NULL, -- 'library_prep', 'sequencing_run', 'sample', 'data'
    entity_id UUID NOT NULL,
    review_type VARCHAR(50) NOT NULL, -- 'automatic', 'manual', 'supervisor', 'external'
    reviewer_id UUID,
    review_status VARCHAR(50) NOT NULL DEFAULT 'pending', -- 'pending', 'in_progress', 'completed'
    decision VARCHAR(50), -- 'approved', 'rejected', 'conditional', 'repeat_required'
    review_criteria JSONB,
    review_results JSONB,
    comments TEXT,
    conditions_for_approval TEXT,
    review_started_at TIMESTAMPTZ,
    review_completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- QC metric history for trend analysis
CREATE TABLE qc_metric_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_definition_id UUID NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    entity_id UUID NOT NULL,
    metric_value DECIMAL(20,6),
    metric_value_json JSONB,
    status VARCHAR(50) NOT NULL, -- 'pass', 'warning', 'fail'
    recorded_at TIMESTAMPTZ NOT NULL,
    recorded_by UUID,
    instrument_id VARCHAR(255),
    batch_id VARCHAR(255),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT qc_metric_history_definition_fkey FOREIGN KEY (metric_definition_id) REFERENCES qc_metric_definitions(id)
);

-- QC control samples tracking
CREATE TABLE qc_control_samples (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    control_type VARCHAR(100) NOT NULL, -- 'positive', 'negative', 'spike_in', 'reference'
    control_name VARCHAR(255) NOT NULL,
    lot_number VARCHAR(255),
    expected_values JSONB NOT NULL,
    tolerance_range JSONB,
    expiry_date DATE,
    storage_location VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- QC control results
CREATE TABLE qc_control_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    control_sample_id UUID NOT NULL,
    run_id UUID NOT NULL,
    run_type VARCHAR(100) NOT NULL, -- 'library_prep', 'sequencing'
    measured_values JSONB NOT NULL,
    passed BOOLEAN NOT NULL,
    deviation_from_expected JSONB,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT qc_control_results_control_fkey FOREIGN KEY (control_sample_id) REFERENCES qc_control_samples(id)
);

-- Indexes for performance
CREATE INDEX idx_qc_metric_definitions_type ON qc_metric_definitions(metric_type);
CREATE INDEX idx_qc_metric_definitions_active ON qc_metric_definitions(is_active);
CREATE INDEX idx_library_prep_qc_library_prep_id ON library_prep_qc(library_prep_id);
CREATE INDEX idx_library_prep_qc_status ON library_prep_qc(overall_status);
CREATE INDEX idx_library_prep_qc_date ON library_prep_qc(qc_date);
CREATE INDEX idx_sequencing_run_qc_run_id ON sequencing_run_qc(sequencing_run_id);
CREATE INDEX idx_sequencing_run_qc_type ON sequencing_run_qc(qc_type);
CREATE INDEX idx_sequencing_run_qc_status ON sequencing_run_qc(overall_status);
CREATE INDEX idx_qc_reviews_entity ON qc_reviews(entity_type, entity_id);
CREATE INDEX idx_qc_reviews_status ON qc_reviews(review_status);
CREATE INDEX idx_qc_metric_history_entity ON qc_metric_history(entity_type, entity_id);
CREATE INDEX idx_qc_metric_history_recorded_at ON qc_metric_history(recorded_at);
CREATE INDEX idx_qc_control_results_control_id ON qc_control_results(control_sample_id);
CREATE INDEX idx_qc_control_results_run ON qc_control_results(run_type, run_id);

-- Triggers for updated_at
CREATE TRIGGER trigger_qc_metric_definitions_updated_at
    BEFORE UPDATE ON qc_metric_definitions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trigger_library_prep_qc_updated_at
    BEFORE UPDATE ON library_prep_qc
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trigger_sequencing_run_qc_updated_at
    BEFORE UPDATE ON sequencing_run_qc
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trigger_qc_reviews_updated_at
    BEFORE UPDATE ON qc_reviews
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trigger_qc_control_samples_updated_at
    BEFORE UPDATE ON qc_control_samples
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- Insert default QC metric definitions
INSERT INTO qc_metric_definitions (name, metric_type, data_type, unit, min_value, max_value, warning_threshold_low, warning_threshold_high, fail_threshold_low, fail_threshold_high, description, is_required) VALUES
-- Library prep metrics
('Library Concentration', 'library_prep', 'numeric', 'ng/μL', 0, 1000, 2, 100, 1, 200, 'Post-prep library concentration', true),
('Fragment Size', 'library_prep', 'numeric', 'bp', 100, 1000, 250, 450, 200, 500, 'Average fragment size', true),
('Library Yield', 'library_prep', 'numeric', 'ng', 0, 10000, 50, 5000, 25, 10000, 'Total library yield', true),
('RIN Score', 'library_prep', 'numeric', NULL, 0, 10, 7, 10, 6, 10, 'RNA Integrity Number', false),
('260/280 Ratio', 'library_prep', 'numeric', NULL, 0, 3, 1.8, 2.0, 1.7, 2.1, 'Nucleic acid purity', true),

-- Sequencing metrics
('Cluster Density', 'sequencing', 'numeric', 'K/mm²', 0, 3000, 800, 1200, 600, 1400, 'Cluster density on flow cell', true),
('% PF Clusters', 'sequencing', 'numeric', '%', 0, 100, 80, 100, 75, 100, 'Percentage of passing filter clusters', true),
('% Q30', 'sequencing', 'numeric', '%', 0, 100, 80, 100, 75, 100, 'Percentage of bases with Q30 or higher', true),
('Error Rate', 'sequencing', 'numeric', '%', 0, 10, 0, 0.5, 0, 1, 'Sequencing error rate', true),
('PhiX Alignment', 'sequencing', 'numeric', '%', 0, 100, 0.5, 5, 0.1, 10, 'Percentage aligned to PhiX control', false);

-- Comments
COMMENT ON TABLE qc_metric_definitions IS 'QC metric definitions and thresholds';
COMMENT ON TABLE library_prep_qc IS 'QC results for library preparations';
COMMENT ON TABLE sequencing_run_qc IS 'QC metrics for sequencing runs';
COMMENT ON TABLE qc_reviews IS 'QC review and approval workflow';
COMMENT ON TABLE qc_metric_history IS 'Historical QC metrics for trend analysis';
COMMENT ON TABLE qc_control_samples IS 'QC control sample definitions';
COMMENT ON TABLE qc_control_results IS 'QC control sample results';