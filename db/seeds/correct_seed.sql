-- TracSeq 2.0 Complete Database Seed (Corrected)
-- This script adds projects, reports, and other missing data

BEGIN;

-- Insert Projects (using actual user IDs for PI and project manager)
INSERT INTO projects (id, project_code, name, description, project_type, status, priority, start_date, target_end_date, principal_investigator_id, project_manager_id, department, budget_approved, budget_used, metadata, created_at, created_by) VALUES
('550e8400-e29b-41d4-a716-446655440301', 'CANCER-2024-001', 'Breast Cancer Genomics Study', 'Comprehensive genomic analysis of breast cancer samples to identify novel biomarkers', 'research', 'active', 'high', '2024-01-15', '2025-01-15', '550e8400-e29b-41d4-a716-446655440006', '550e8400-e29b-41d4-a716-446655440002', 'Oncology', 250000.00, 12500.00, '{"funding_source": "NIH Grant", "ethical_approval": "IRB-2024-001"}', NOW() - INTERVAL '6 months', '550e8400-e29b-41d4-a716-446655440001'),
('550e8400-e29b-41d4-a716-446655440302', 'COVID-2024-002', 'COVID-19 Variant Surveillance', 'Ongoing surveillance of COVID-19 variants in the local population', 'diagnostic', 'active', 'urgent', '2024-02-01', '2024-12-31', '550e8400-e29b-41d4-a716-446655440002', '550e8400-e29b-41d4-a716-446655440002', 'Infectious Disease', 500000.00, 150000.00, '{"public_health": true, "reporting_required": "weekly"}', NOW() - INTERVAL '5 months', '550e8400-e29b-41d4-a716-446655440001'),
('550e8400-e29b-41d4-a716-446655440303', 'MICROBIOME-2024-003', 'Gut Microbiome Diversity', 'Study of gut microbiome diversity in different dietary groups', 'research', 'active', 'medium', '2024-03-01', '2025-03-01', '550e8400-e29b-41d4-a716-446655440006', '550e8400-e29b-41d4-a716-446655440002', 'Microbiology', 150000.00, 25000.00, '{"sample_size": 100, "cohorts": 4}', NOW() - INTERVAL '4 months', '550e8400-e29b-41d4-a716-446655440001'),
('550e8400-e29b-41d4-a716-446655440304', 'RARE-2023-001', 'Rare Disease Genomics', 'Whole genome sequencing for rare disease diagnosis', 'clinical', 'completed', 'high', '2023-06-01', '2024-06-01', '550e8400-e29b-41d4-a716-446655440006', '550e8400-e29b-41d4-a716-446655440002', 'Genetics', 300000.00, 295000.00, '{"patients_enrolled": 50, "variants_identified": 12}', NOW() - INTERVAL '1 year', '550e8400-e29b-41d4-a716-446655440001'),
('550e8400-e29b-41d4-a716-446655440305', 'AGRI-2024-001', 'Crop Resistance Study', 'Genomic analysis of drought-resistant crop varieties', 'research', 'planning', 'low', '2024-09-01', '2025-09-01', '550e8400-e29b-41d4-a716-446655440006', NULL, 'Agricultural Sciences', 180000.00, 0.00, '{"crop_types": ["wheat", "corn", "soy"]}', NOW() - INTERVAL '1 month', '550e8400-e29b-41d4-a716-446655440001')
ON CONFLICT (id) DO NOTHING;

-- Add team members to projects
INSERT INTO project_team_members (project_id, user_id, role, joined_at) VALUES
('550e8400-e29b-41d4-a716-446655440301', '550e8400-e29b-41d4-a716-446655440003', 'lab_technician', NOW() - INTERVAL '5 months'),
('550e8400-e29b-41d4-a716-446655440301', '550e8400-e29b-41d4-a716-446655440004', 'data_analyst', NOW() - INTERVAL '5 months'),
('550e8400-e29b-41d4-a716-446655440302', '550e8400-e29b-41d4-a716-446655440003', 'lab_technician', NOW() - INTERVAL '4 months'),
('550e8400-e29b-41d4-a716-446655440302', '550e8400-e29b-41d4-a716-446655440005', 'lab_technician', NOW() - INTERVAL '4 months'),
('550e8400-e29b-41d4-a716-446655440303', '550e8400-e29b-41d4-a716-446655440005', 'lab_technician', NOW() - INTERVAL '3 months')
ON CONFLICT DO NOTHING;

-- Update samples to link to projects
UPDATE samples 
SET metadata = jsonb_set(metadata, '{project_id}', '"550e8400-e29b-41d4-a716-446655440301"')
WHERE metadata->>'project' = 'CANCER-2024-001';

UPDATE samples 
SET metadata = jsonb_set(metadata, '{project_id}', '"550e8400-e29b-41d4-a716-446655440302"')
WHERE metadata->>'project' = 'COVID-2024-002';

UPDATE samples 
SET metadata = jsonb_set(metadata, '{project_id}', '"550e8400-e29b-41d4-a716-446655440303"')
WHERE metadata->>'project' = 'MICROBIOME-2024-003';

UPDATE samples 
SET metadata = jsonb_set(metadata, '{project_id}', '"550e8400-e29b-41d4-a716-446655440304"')
WHERE metadata->>'project' = 'RARE-2023-001';

-- Insert Generated Reports (with corrected schema)
INSERT INTO generated_reports (id, definition_id, name, description, status, format, parameters, file_path, file_size, generated_by, started_at, completed_at, created_at) VALUES
('550e8400-e29b-41d4-a716-446655440401', NULL, 'Monthly Sequencing Summary - June 2024', 'Comprehensive summary of all sequencing activities for June 2024', 'completed', 'pdf', '{"month": "2024-06", "include_failed": false}', '/reports/2024/06/sequencing_summary.pdf', 1048576, '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '1 month', NOW() - INTERVAL '1 month' + INTERVAL '2 hours', NOW() - INTERVAL '1 month'),
('550e8400-e29b-41d4-a716-446655440402', NULL, 'Sample Quality Report - CANCER-2024-001', 'Quality assessment for breast cancer genomics project samples', 'completed', 'pdf', '{"project_code": "CANCER-2024-001", "date_range": "all"}', '/reports/projects/cancer_2024_001_quality.pdf', 524288, '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '2 weeks', NOW() - INTERVAL '2 weeks' + INTERVAL '30 minutes', NOW() - INTERVAL '2 weeks'),
('550e8400-e29b-41d4-a716-446655440403', NULL, 'Storage Capacity Analysis Q2 2024', 'Quarterly analysis of storage utilization and capacity', 'completed', 'excel', '{"quarter": "Q2", "year": 2024}', '/reports/2024/q2/storage_capacity.xlsx', 204800, '550e8400-e29b-41d4-a716-446655440001', NOW() - INTERVAL '1 week', NOW() - INTERVAL '1 week' + INTERVAL '1 hour', NOW() - INTERVAL '1 week'),
('550e8400-e29b-41d4-a716-446655440404', NULL, 'COVID Variant Detection Report', 'Analysis of detected COVID-19 variants in surveillance samples', 'completed', 'pdf', '{"project_code": "COVID-2024-002", "analysis_type": "variant_detection"}', '/reports/covid/variant_analysis_2024_06.pdf', 2097152, '550e8400-e29b-41d4-a716-446655440006', NOW() - INTERVAL '5 days', NOW() - INTERVAL '5 days' + INTERVAL '3 hours', NOW() - INTERVAL '5 days'),
('550e8400-e29b-41d4-a716-446655440405', NULL, 'Library Preparation Efficiency Q2 2024', 'Analysis of library preparation success rates and efficiency metrics', 'completed', 'pdf', '{"quarter": "Q2", "year": 2024, "group_by": "technician"}', '/reports/2024/q2/library_prep_efficiency.pdf', 786432, '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '3 weeks', NOW() - INTERVAL '3 weeks' + INTERVAL '45 minutes', NOW() - INTERVAL '3 weeks'),
('550e8400-e29b-41d4-a716-446655440406', NULL, 'Microbiome Diversity Preliminary Results', 'Initial analysis of microbiome diversity across dietary cohorts', 'processing', 'pdf', '{"project_code": "MICROBIOME-2024-003", "analysis_stage": "preliminary"}', NULL, NULL, '550e8400-e29b-41d4-a716-446655440006', NOW() - INTERVAL '3 days', NULL, NOW() - INTERVAL '3 days')
ON CONFLICT (id) DO NOTHING;

-- Insert Quality Reports (with corrected schema)
INSERT INTO quality_reports (id, entity_type, entity_id, report_type, overall_status, metrics_summary, recommendations, report_data, generated_at, created_at) VALUES
('550e8400-e29b-41d4-a716-446655440501', 'sample', '550e8400-e29b-41d4-a716-446655440101', 'initial_qc', 'pass', '{"concentration": 150.5, "integrity": 8.9, "purity": 1.85, "volume": 500}', '{"Continue with sequencing", "Consider duplicate aliquot for backup"}', '{"detailed_metrics": {"260_280_ratio": 1.85, "260_230_ratio": 2.1, "rin_score": 8.9}, "instrument": "Agilent Bioanalyzer", "technician": "Jane Smith"}', NOW() - INTERVAL '5 months', NOW() - INTERVAL '5 months'),
('550e8400-e29b-41d4-a716-446655440502', 'sample', '550e8400-e29b-41d4-a716-446655440102', 'initial_qc', 'pass', '{"concentration": 125.3, "integrity": 8.7, "purity": 1.83, "volume": 450}', '{}', '{"detailed_metrics": {"260_280_ratio": 1.83, "260_230_ratio": 2.05}}', NOW() - INTERVAL '5 months', NOW() - INTERVAL '5 months'),
('550e8400-e29b-41d4-a716-446655440503', 'sample', '550e8400-e29b-41d4-a716-446655440103', 'initial_qc', 'pass', '{"concentration": 85.2, "integrity": 9.1, "purity": 1.88, "volume": 1000}', '{"Excellent RNA quality", "Proceed with library prep immediately"}', '{"detailed_metrics": {"260_280_ratio": 1.88, "260_230_ratio": 2.15, "rin_score": 9.1}}', NOW() - INTERVAL '5 months', NOW() - INTERVAL '5 months'),
('550e8400-e29b-41d4-a716-446655440504', 'sample', '550e8400-e29b-41d4-a716-446655440104', 'viral_load', 'warning', '{"ct_value": 25.3, "copies_per_ml": 125000, "quality": "good"}', '{"High viral load detected", "Use appropriate PPE", "Consider dilution for sequencing"}', '{"gene_targets": {"N_gene": 25.3, "E_gene": 25.8}, "variant_preliminary": "likely Omicron"}', NOW() - INTERVAL '3 months', NOW() - INTERVAL '3 months')
ON CONFLICT (id) DO NOTHING;

-- Insert Report Templates
INSERT INTO report_templates (id, name, description, template_type, template_content, parameters_schema, created_by, created_at) VALUES
('550e8400-e29b-41d4-a716-446655440601', 'Monthly Sequencing Summary', 'Standard monthly summary of all sequencing activities', 'summary', '{"sections": ["overview", "by_project", "by_platform", "quality_metrics", "turnaround_time"]}', '{"month": {"type": "string", "format": "YYYY-MM"}}', '550e8400-e29b-41d4-a716-446655440001', NOW() - INTERVAL '1 year'),
('550e8400-e29b-41d4-a716-446655440602', 'Project Quality Report', 'Quality control summary for a specific project', 'quality', '{"sections": ["sample_summary", "qc_metrics", "failed_samples", "recommendations"]}', '{"project_code": {"type": "string"}}', '550e8400-e29b-41d4-a716-446655440001', NOW() - INTERVAL '8 months'),
('550e8400-e29b-41d4-a716-446655440603', 'Storage Utilization Report', 'Analysis of storage capacity and utilization', 'operational', '{"sections": ["current_usage", "by_temperature", "forecast", "recommendations"]}', '{"include_forecast": {"type": "boolean", "default": true}}', '550e8400-e29b-41d4-a716-446655440001', NOW() - INTERVAL '6 months')
ON CONFLICT (id) DO NOTHING;

-- Insert Batches (with corrected schema)
INSERT INTO batches (id, batch_number, project_id, batch_type, status, priority, sample_count, metadata, notes, created_by, created_at) VALUES
('550e8400-e29b-41d4-a716-446655440801', 'BATCH-2024-001', '550e8400-e29b-41d4-a716-446655440301', 'sequencing', 'completed', 'high', 24, '{"platform": "NovaSeq", "run_type": "PE150"}', 'Initial breast cancer samples - high priority', '550e8400-e29b-41d4-a716-446655440003', NOW() - INTERVAL '5 months'),
('550e8400-e29b-41d4-a716-446655440802', 'BATCH-2024-002', '550e8400-e29b-41d4-a716-446655440301', 'sequencing', 'completed', 'medium', 24, '{"platform": "NovaSeq", "run_type": "PE150"}', 'Second batch breast cancer samples', '550e8400-e29b-41d4-a716-446655440003', NOW() - INTERVAL '4 months'),
('550e8400-e29b-41d4-a716-446655440803', 'BATCH-2024-003', '550e8400-e29b-41d4-a716-446655440302', 'diagnostic', 'processing', 'urgent', 48, '{"platform": "NextSeq", "run_type": "SE75", "turnaround": "48h"}', 'COVID surveillance batch - rush processing', '550e8400-e29b-41d4-a716-446655440003', NOW() - INTERVAL '3 months'),
('550e8400-e29b-41d4-a716-446655440804', 'BATCH-2024-004', '550e8400-e29b-41d4-a716-446655440303', 'research', 'completed', 'medium', 36, '{"platform": "MiSeq", "run_type": "PE300"}', 'Microbiome cohort 1 - dietary intervention group', '550e8400-e29b-41d4-a716-446655440005', NOW() - INTERVAL '2 months'),
('550e8400-e29b-41d4-a716-446655440805', 'BATCH-2024-005', '550e8400-e29b-41d4-a716-446655440303', 'research', 'ready', 'medium', 36, '{"platform": "MiSeq", "run_type": "PE300"}', 'Microbiome cohort 2 - control group', '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '1 week')
ON CONFLICT (id) DO NOTHING;

-- Insert Sequencing Platforms (if they exist)
INSERT INTO sequencing_platforms (id, name, model, manufacturer, technology, max_read_length, max_output_gb, typical_run_time_hours, is_active, created_at) VALUES
('550e8400-e29b-41d4-a716-446655440901', 'NovaSeq 6000 #1', 'NovaSeq 6000', 'Illumina', 'SBS', 150, 6000, 44, true, NOW() - INTERVAL '2 years'),
('550e8400-e29b-41d4-a716-446655440902', 'MiSeq #1', 'MiSeq', 'Illumina', 'SBS', 300, 15, 24, true, NOW() - INTERVAL '3 years'),
('550e8400-e29b-41d4-a716-446655440903', 'NextSeq 2000 #1', 'NextSeq 2000', 'Illumina', 'SBS', 150, 360, 29, true, NOW() - INTERVAL '1 year')
ON CONFLICT (id) DO NOTHING;

-- Insert sequencing runs
INSERT INTO sequencing_runs (id, run_id, platform_id, name, run_type, read_length, status, started_at, completed_at, operator_id, metadata, created_at) VALUES
('550e8400-e29b-41d4-a716-446655441001', 'RUN-2024-0001', '550e8400-e29b-41d4-a716-446655440901', 'CANCER-2024-001 Batch 1', 'production', 150, 'completed', NOW() - INTERVAL '5 months', NOW() - INTERVAL '5 months' + INTERVAL '44 hours', '550e8400-e29b-41d4-a716-446655440003', '{"flow_cell": "HJKL5DRXY", "chemistry": "v1.5"}', NOW() - INTERVAL '5 months'),
('550e8400-e29b-41d4-a716-446655441002', 'RUN-2024-0002', '550e8400-e29b-41d4-a716-446655440901', 'CANCER-2024-001 Batch 2', 'production', 150, 'completed', NOW() - INTERVAL '4 months', NOW() - INTERVAL '4 months' + INTERVAL '44 hours', '550e8400-e29b-41d4-a716-446655440003', '{"flow_cell": "HJKL6DRXY", "chemistry": "v1.5"}', NOW() - INTERVAL '4 months'),
('550e8400-e29b-41d4-a716-446655441003', 'RUN-2024-0003', '550e8400-e29b-41d4-a716-446655440903', 'COVID-2024-002 Surveillance', 'diagnostic', 75, 'running', NOW() - INTERVAL '6 hours', NULL, '550e8400-e29b-41d4-a716-446655440005', '{"flow_cell": "AAXC7M5XY", "priority": "urgent"}', NOW() - INTERVAL '6 hours'),
('550e8400-e29b-41d4-a716-446655441004', 'RUN-2024-0004', '550e8400-e29b-41d4-a716-446655440902', 'MICROBIOME-2024-003 Cohort 1', 'research', 300, 'completed', NOW() - INTERVAL '2 months', NOW() - INTERVAL '2 months' + INTERVAL '24 hours', '550e8400-e29b-41d4-a716-446655440005', '{"flow_cell": "MJKL2DRXY", "chemistry": "v3"}', NOW() - INTERVAL '2 months')
ON CONFLICT (id) DO NOTHING;

-- Insert some transactions
INSERT INTO transactions (id, type, description, user_id, entity_type, entity_id, metadata, created_at) VALUES
('550e8400-e29b-41d4-a716-446655441101', 'sample_received', 'Received breast cancer tissue sample', '550e8400-e29b-41d4-a716-446655440003', 'sample', '550e8400-e29b-41d4-a716-446655440101', '{"location": "receiving", "temperature": "4C"}', NOW() - INTERVAL '5 months'),
('550e8400-e29b-41d4-a716-446655441102', 'sample_stored', 'Stored in ultra-low freezer', '550e8400-e29b-41d4-a716-446655440003', 'sample', '550e8400-e29b-41d4-a716-446655440101', '{"location": "ULT-01-A-03", "temperature": "-80C"}', NOW() - INTERVAL '5 months' + INTERVAL '1 hour'),
('550e8400-e29b-41d4-a716-446655441103', 'project_created', 'Created new research project', '550e8400-e29b-41d4-a716-446655440001', 'project', '550e8400-e29b-41d4-a716-446655440301', '{"funding_approved": true}', NOW() - INTERVAL '6 months'),
('550e8400-e29b-41d4-a716-446655441104', 'report_generated', 'Generated monthly summary report', '550e8400-e29b-41d4-a716-446655440002', 'report', '550e8400-e29b-41d4-a716-446655440401', '{"recipients": ["lab_manager@example.com", "pi@example.com"]}', NOW() - INTERVAL '1 month'),
('550e8400-e29b-41d4-a716-446655441105', 'sequencing_started', 'Started sequencing run', '550e8400-e29b-41d4-a716-446655440005', 'sequencing_run', '550e8400-e29b-41d4-a716-446655441003', '{"estimated_completion": "29 hours"}', NOW() - INTERVAL '6 hours')
ON CONFLICT (id) DO NOTHING;

COMMIT;
