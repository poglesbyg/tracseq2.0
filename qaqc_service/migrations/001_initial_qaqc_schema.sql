-- QAQC Service - Initial Schema Migration
-- File: qaqc_service/migrations/001_initial_qaqc_schema.sql

-- Create custom types
CREATE TYPE qc_status AS ENUM ('pending', 'in_progress', 'passed', 'failed', 'warning', 'cancelled');
CREATE TYPE qc_priority AS ENUM ('low', 'normal', 'high', 'critical');
CREATE TYPE validation_type AS ENUM ('automatic', 'manual', 'hybrid');
CREATE TYPE compliance_standard AS ENUM ('iso_15189', 'cap', 'clia', 'glp', 'gcp', 'internal');

-- Quality Control Checks table
CREATE TABLE qc_checks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    check_type VARCHAR(100) NOT NULL, -- 'sample_integrity', 'data_validation', 'equipment_verification'
    validation_type validation_type DEFAULT 'automatic',
    priority qc_priority DEFAULT 'normal',
    check_criteria JSONB NOT NULL, -- Validation rules and criteria
    acceptance_criteria JSONB NOT NULL, -- What constitutes pass/fail
    compliance_standards compliance_standard[] DEFAULT ARRAY['internal'],
    is_mandatory BOOLEAN DEFAULT true,
    is_active BOOLEAN DEFAULT true,
    frequency_hours INTEGER, -- How often to run (for periodic checks)
    timeout_minutes INTEGER DEFAULT 30,
    retry_count INTEGER DEFAULT 3,
    version INTEGER DEFAULT 1,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT qc_checks_version_check CHECK (version > 0),
    CONSTRAINT qc_checks_frequency_check CHECK (frequency_hours IS NULL OR frequency_hours > 0)
);

-- QC Check Executions table (individual runs of QC checks)
CREATE TABLE qc_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    check_id UUID NOT NULL,
    execution_reference VARCHAR(255), -- External reference (sample ID, batch ID, etc.)
    reference_type VARCHAR(100), -- 'sample', 'batch', 'equipment', 'manual'
    status qc_status DEFAULT 'pending',
    started_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    duration_seconds INTEGER,
    executed_by UUID,
    execution_parameters JSONB DEFAULT '{}',
    raw_results JSONB DEFAULT '{}',
    processed_results JSONB DEFAULT '{}',
    pass_criteria_met BOOLEAN,
    score DECIMAL(5,2), -- 0.00 to 100.00
    confidence_level DECIMAL(3,2), -- 0.00 to 1.00
    error_message TEXT,
    warnings JSONB DEFAULT '[]',
    notes TEXT,
    reviewed_by UUID,
    reviewed_at TIMESTAMPTZ,
    review_notes TEXT,
    correlation_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT qc_executions_check_fkey FOREIGN KEY (check_id) REFERENCES qc_checks(id),
    CONSTRAINT qc_executions_score_check CHECK (score IS NULL OR (score >= 0.0 AND score <= 100.0)),
    CONSTRAINT qc_executions_confidence_check CHECK (confidence_level IS NULL OR (confidence_level >= 0.0 AND confidence_level <= 1.0))
);

-- Validation Rules table (reusable validation logic)
CREATE TABLE validation_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    rule_type VARCHAR(100) NOT NULL, -- 'range_check', 'pattern_match', 'reference_lookup', 'custom_logic'
    data_type VARCHAR(50) NOT NULL, -- 'numeric', 'text', 'date', 'boolean', 'json'
    rule_definition JSONB NOT NULL, -- Rule parameters and logic
    error_message_template TEXT NOT NULL,
    warning_message_template TEXT,
    severity VARCHAR(20) DEFAULT 'error', -- 'error', 'warning', 'info'
    is_active BOOLEAN DEFAULT true,
    usage_count INTEGER DEFAULT 0,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Sample Quality Assessments table
CREATE TABLE sample_quality_assessments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sample_id UUID NOT NULL, -- Reference to samples table
    assessment_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    overall_status qc_status DEFAULT 'pending',
    overall_score DECIMAL(5,2), -- 0.00 to 100.00
    visual_inspection JSONB DEFAULT '{}',
    quantitative_metrics JSONB DEFAULT '{}',
    contamination_check JSONB DEFAULT '{}',
    integrity_assessment JSONB DEFAULT '{}',
    storage_compliance JSONB DEFAULT '{}',
    chain_of_custody_verified BOOLEAN DEFAULT false,
    acceptance_decision VARCHAR(50), -- 'accept', 'reject', 'conditional_accept', 'retest'
    rejection_reason TEXT,
    assessed_by UUID,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    comments TEXT,
    corrective_actions JSONB DEFAULT '[]',
    follow_up_required BOOLEAN DEFAULT false,
    follow_up_date TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT sample_quality_score_check CHECK (overall_score IS NULL OR (overall_score >= 0.0 AND overall_score <= 100.0))
);

-- Compliance Reports table
CREATE TABLE compliance_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_name VARCHAR(255) NOT NULL,
    standard compliance_standard NOT NULL,
    reporting_period_start DATE NOT NULL,
    reporting_period_end DATE NOT NULL,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    generated_by UUID,
    status VARCHAR(50) DEFAULT 'draft', -- 'draft', 'final', 'submitted', 'approved'
    total_checks INTEGER NOT NULL DEFAULT 0,
    passed_checks INTEGER NOT NULL DEFAULT 0,
    failed_checks INTEGER NOT NULL DEFAULT 0,
    warning_checks INTEGER NOT NULL DEFAULT 0,
    compliance_percentage DECIMAL(5,2),
    critical_findings JSONB DEFAULT '[]',
    recommendations JSONB DEFAULT '[]',
    summary_statistics JSONB DEFAULT '{}',
    report_data JSONB DEFAULT '{}',
    file_path TEXT,
    submitted_to VARCHAR(255),
    submitted_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    next_report_due TIMESTAMPTZ,
    
    CONSTRAINT compliance_reports_compliance_check CHECK (compliance_percentage IS NULL OR (compliance_percentage >= 0.0 AND compliance_percentage <= 100.0)),
    CONSTRAINT compliance_reports_dates_check CHECK (reporting_period_end >= reporting_period_start)
);

-- Audit Trails table (for compliance and tracking)
CREATE TABLE qc_audit_trails (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(100) NOT NULL, -- 'qc_check', 'execution', 'assessment', 'report'
    entity_id UUID NOT NULL,
    action VARCHAR(100) NOT NULL, -- 'created', 'updated', 'approved', 'rejected', 'deleted'
    performed_by UUID,
    performed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    old_values JSONB,
    new_values JSONB,
    change_reason TEXT,
    ip_address INET,
    user_agent TEXT,
    session_id VARCHAR(255),
    correlation_id VARCHAR(255)
);

-- Equipment Calibration Records table
CREATE TABLE equipment_calibrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    equipment_id VARCHAR(255) NOT NULL,
    equipment_name VARCHAR(255) NOT NULL,
    calibration_date TIMESTAMPTZ NOT NULL,
    next_calibration_due TIMESTAMPTZ NOT NULL,
    calibration_type VARCHAR(100) NOT NULL, -- 'initial', 'periodic', 'maintenance', 'verification'
    performed_by UUID,
    calibration_standard VARCHAR(255),
    calibration_procedure VARCHAR(255),
    calibration_results JSONB DEFAULT '{}',
    tolerance_met BOOLEAN NOT NULL,
    certificate_number VARCHAR(255),
    certificate_path TEXT,
    status VARCHAR(50) DEFAULT 'valid', -- 'valid', 'expired', 'out_of_tolerance', 'pending'
    comments TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT equipment_calibrations_dates_check CHECK (next_calibration_due > calibration_date)
);

-- QC Metrics and Statistics table
CREATE TABLE qc_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_date DATE NOT NULL,
    metric_hour INTEGER NOT NULL,
    check_type VARCHAR(100) NOT NULL,
    total_executions INTEGER NOT NULL DEFAULT 0,
    passed_executions INTEGER NOT NULL DEFAULT 0,
    failed_executions INTEGER NOT NULL DEFAULT 0,
    warning_executions INTEGER NOT NULL DEFAULT 0,
    average_score DECIMAL(5,2),
    average_duration_seconds INTEGER,
    pass_rate DECIMAL(5,2),
    trend_direction VARCHAR(20), -- 'improving', 'declining', 'stable'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT qc_metrics_hour_check CHECK (metric_hour >= 0 AND metric_hour <= 23),
    CONSTRAINT qc_metrics_executions_check CHECK (
        total_executions >= 0 AND passed_executions >= 0 AND failed_executions >= 0 AND warning_executions >= 0 AND
        (passed_executions + failed_executions + warning_executions) <= total_executions
    ),
    CONSTRAINT qc_metrics_pass_rate_check CHECK (pass_rate IS NULL OR (pass_rate >= 0.0 AND pass_rate <= 100.0)),
    UNIQUE(metric_date, metric_hour, check_type)
);

-- Create indexes for performance
CREATE INDEX idx_qc_checks_check_type ON qc_checks(check_type);
CREATE INDEX idx_qc_checks_active ON qc_checks(is_active);
CREATE INDEX idx_qc_checks_priority ON qc_checks(priority);
CREATE INDEX idx_qc_checks_compliance_gin ON qc_checks USING gin(compliance_standards);

CREATE INDEX idx_qc_executions_check_id ON qc_executions(check_id);
CREATE INDEX idx_qc_executions_status ON qc_executions(status);
CREATE INDEX idx_qc_executions_reference ON qc_executions(execution_reference);
CREATE INDEX idx_qc_executions_started_at ON qc_executions(started_at);
CREATE INDEX idx_qc_executions_correlation_id ON qc_executions(correlation_id);

CREATE INDEX idx_validation_rules_type ON validation_rules(rule_type);
CREATE INDEX idx_validation_rules_active ON validation_rules(is_active);

CREATE INDEX idx_sample_quality_sample_id ON sample_quality_assessments(sample_id);
CREATE INDEX idx_sample_quality_status ON sample_quality_assessments(overall_status);
CREATE INDEX idx_sample_quality_assessment_date ON sample_quality_assessments(assessment_date);
CREATE INDEX idx_sample_quality_decision ON sample_quality_assessments(acceptance_decision);

CREATE INDEX idx_compliance_reports_standard ON compliance_reports(standard);
CREATE INDEX idx_compliance_reports_period ON compliance_reports(reporting_period_start, reporting_period_end);
CREATE INDEX idx_compliance_reports_status ON compliance_reports(status);

CREATE INDEX idx_qc_audit_trails_entity ON qc_audit_trails(entity_type, entity_id);
CREATE INDEX idx_qc_audit_trails_performed_at ON qc_audit_trails(performed_at);
CREATE INDEX idx_qc_audit_trails_performed_by ON qc_audit_trails(performed_by);

CREATE INDEX idx_equipment_calibrations_equipment_id ON equipment_calibrations(equipment_id);
CREATE INDEX idx_equipment_calibrations_due_date ON equipment_calibrations(next_calibration_due);
CREATE INDEX idx_equipment_calibrations_status ON equipment_calibrations(status);

CREATE INDEX idx_qc_metrics_date_hour ON qc_metrics(metric_date, metric_hour);
CREATE INDEX idx_qc_metrics_check_type ON qc_metrics(check_type);

-- Create trigger functions
CREATE OR REPLACE FUNCTION update_qc_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers
CREATE TRIGGER trigger_qc_checks_updated_at
    BEFORE UPDATE ON qc_checks
    FOR EACH ROW
    EXECUTE FUNCTION update_qc_updated_at();

CREATE TRIGGER trigger_validation_rules_updated_at
    BEFORE UPDATE ON validation_rules
    FOR EACH ROW
    EXECUTE FUNCTION update_qc_updated_at();

-- Insert sample QC checks
INSERT INTO qc_checks (name, check_type, validation_type, priority, check_criteria, acceptance_criteria, compliance_standards) VALUES
    ('Sample Volume Verification', 'sample_integrity', 'automatic', 'high', 
     '{"min_volume": 50, "max_volume": 1000, "unit": "µL"}',
     '{"volume_variance": 5, "unit": "percent"}',
     ARRAY['iso_15189', 'internal']),
     
    ('DNA Concentration Range Check', 'sample_integrity', 'automatic', 'normal',
     '{"min_concentration": 1.0, "max_concentration": 1000.0, "unit": "ng/µL"}',
     '{"acceptable_range": [1.0, 500.0], "warning_range": [500.0, 1000.0]}',
     ARRAY['glp', 'internal']),
     
    ('Chain of Custody Verification', 'data_validation', 'manual', 'critical',
     '{"required_signatures": 2, "required_timestamps": true, "required_conditions": ["temperature", "integrity"]}',
     '{"all_signatures_present": true, "timestamps_valid": true, "conditions_documented": true}',
     ARRAY['iso_15189', 'cap', 'clia']);

-- Insert sample validation rules
INSERT INTO validation_rules (name, rule_type, data_type, rule_definition, error_message_template, severity) VALUES
    ('Positive Number Range', 'range_check', 'numeric', 
     '{"min": 0, "max": null, "inclusive": true}',
     'Value must be positive (greater than 0)', 'error'),
     
    ('Email Format', 'pattern_match', 'text',
     '{"pattern": "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"}',
     'Invalid email format', 'error'),
     
    ('Sample ID Format', 'pattern_match', 'text',
     '{"pattern": "^[A-Z]{3}-\\d{8}-[A-Z0-9]{6}$"}',
     'Sample ID must follow format: ABC-12345678-XYZ123', 'error');

COMMENT ON TABLE qc_checks IS 'Quality control check definitions and configurations';
COMMENT ON TABLE qc_executions IS 'Individual executions of quality control checks';
COMMENT ON TABLE validation_rules IS 'Reusable validation rules for data quality checks';
COMMENT ON TABLE sample_quality_assessments IS 'Quality assessments for laboratory samples';
COMMENT ON TABLE compliance_reports IS 'Compliance reports for regulatory standards';
COMMENT ON TABLE qc_audit_trails IS 'Audit trail for QC activities and changes';
COMMENT ON TABLE equipment_calibrations IS 'Equipment calibration records and schedules';
COMMENT ON TABLE qc_metrics IS 'Aggregated metrics and statistics for QC performance'; 