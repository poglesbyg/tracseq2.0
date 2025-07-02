-- TracSeq 2.0 Complete Database Seed
-- This script adds projects, reports, and other missing data

-- Insert Projects (using actual user IDs for PI and project manager)
INSERT INTO projects (id, project_code, name, description, project_type, status, priority, start_date, target_end_date, principal_investigator_id, project_manager_id, department, budget_approved, metadata, created_at) VALUES
('550e8400-e29b-41d4-a716-446655440301', 'CANCER-2024-001', 'Breast Cancer Genomics Study', 'Comprehensive genomic analysis of breast cancer samples to identify novel biomarkers', 'research', 'active', 'high', '2024-01-15', '2025-01-15', '550e8400-e29b-41d4-a716-446655440006', '550e8400-e29b-41d4-a716-446655440002', 'Oncology', 250000.00, '{"funding_source": "NIH Grant", "ethical_approval": "IRB-2024-001"}', NOW() - INTERVAL '6 months'),
('550e8400-e29b-41d4-a716-446655440302', 'COVID-2024-002', 'COVID-19 Variant Surveillance', 'Ongoing surveillance of COVID-19 variants in the local population', 'diagnostic', 'active', 'urgent', '2024-02-01', '2024-12-31', '550e8400-e29b-41d4-a716-446655440002', '550e8400-e29b-41d4-a716-446655440002', 'Infectious Disease', 500000.00, '{"public_health": true, "reporting_required": "weekly"}', NOW() - INTERVAL '5 months'),
('550e8400-e29b-41d4-a716-446655440303', 'MICROBIOME-2024-003', 'Gut Microbiome Diversity', 'Study of gut microbiome diversity in different dietary groups', 'research', 'active', 'medium', '2024-03-01', '2025-03-01', '550e8400-e29b-41d4-a716-446655440006', '550e8400-e29b-41d4-a716-446655440002', 'Microbiology', 150000.00, '{"sample_size": 100, "cohorts": 4}', NOW() - INTERVAL '4 months'),
('550e8400-e29b-41d4-a716-446655440304', 'RARE-2023-001', 'Rare Disease Genomics', 'Whole genome sequencing for rare disease diagnosis', 'clinical', 'completed', 'high', '2023-06-01', '2024-06-01', '550e8400-e29b-41d4-a716-446655440006', '550e8400-e29b-41d4-a716-446655440002', 'Genetics', 300000.00, '{"patients_enrolled": 50, "variants_identified": 12}', NOW() - INTERVAL '1 year'),
('550e8400-e29b-41d4-a716-446655440305', 'AGRI-2024-001', 'Crop Resistance Study', 'Genomic analysis of drought-resistant crop varieties', 'research', 'planning', 'low', '2024-09-01', '2025-09-01', '550e8400-e29b-41d4-a716-446655440006', NULL, 'Agricultural Sciences', 180000.00, '{"crop_types": ["wheat", "corn", "soy"]}', NOW() - INTERVAL '1 month')
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

-- Insert Generated Reports
INSERT INTO generated_reports (id, report_definition_id, name, file_path, file_size, format, parameters, metadata, generated_by, created_at) VALUES
('550e8400-e29b-41d4-a716-446655440401', NULL, 'Monthly Sequencing Summary - June 2024', '/reports/2024/06/sequencing_summary.pdf', 1048576, 'pdf', '{"month": "2024-06", "include_failed": false}', '{"total_runs": 15, "total_samples": 245, "success_rate": 98.5, "average_turnaround_days": 5.2}', '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '1 month'),
('550e8400-e29b-41d4-a716-446655440402', NULL, 'Sample Quality Report - CANCER-2024-001', '/reports/projects/cancer_2024_001_quality.pdf', 524288, 'pdf', '{"project_code": "CANCER-2024-001", "date_range": "all"}', '{"total_samples": 24, "passed_qc": 23, "failed_qc": 1, "average_quality_score": 8.7}', '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '2 weeks'),
('550e8400-e29b-41d4-a716-446655440403', NULL, 'Storage Capacity Analysis Q2 2024', '/reports/2024/q2/storage_capacity.xlsx', 204800, 'excel', '{"quarter": "Q2", "year": 2024}', '{"ultra_low_utilization": 65.5, "freezer_utilization": 78.2, "refrigerator_utilization": 45.8, "forecast_full_date": "2024-12-15"}', '550e8400-e29b-41d4-a716-446655440001', NOW() - INTERVAL '1 week'),
('550e8400-e29b-41d4-a716-446655440404', NULL, 'COVID Variant Detection Report', '/reports/covid/variant_analysis_2024_06.pdf', 2097152, 'pdf', '{"project_code": "COVID-2024-002", "analysis_type": "variant_detection"}', '{"variants_detected": ["Delta", "Omicron BA.2", "Omicron BA.5"], "dominant_variant": "Omicron BA.5", "sample_count": 48}', '550e8400-e29b-41d4-a716-446655440006', NOW() - INTERVAL '5 days'),
('550e8400-e29b-41d4-a716-446655440405', NULL, 'Library Preparation Efficiency Q2 2024', '/reports/2024/q2/library_prep_efficiency.pdf', 786432, 'pdf', '{"quarter": "Q2", "year": 2024, "group_by": "technician"}', '{"total_preps": 186, "success_rate": 97.3, "average_yield_ng": 28.5, "average_time_hours": 4.2}', '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '3 weeks'),
('550e8400-e29b-41d4-a716-446655440406', NULL, 'Microbiome Diversity Preliminary Results', '/reports/projects/microbiome_2024_003_preliminary.pdf', 3145728, 'pdf', '{"project_code": "MICROBIOME-2024-003", "analysis_stage": "preliminary"}', '{"samples_analyzed": 36, "diversity_index": 0.82, "dominant_phyla": ["Bacteroidetes", "Firmicutes"], "unique_otus": 1247}', '550e8400-e29b-41d4-a716-446655440006', NOW() - INTERVAL '3 days')
ON CONFLICT (id) DO NOTHING;

-- Insert Quality Reports (for samples)
INSERT INTO quality_reports (id, entity_type, entity_id, report_type, metrics, passed, notes, reviewed_by, reviewed_at, created_at) VALUES
('550e8400-e29b-41d4-a716-446655440501', 'sample', '550e8400-e29b-41d4-a716-446655440101', 'initial_qc', '{"concentration": 150.5, "integrity": 8.9, "purity": 1.85, "volume": 500}', true, 'High quality sample, suitable for sequencing', '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '5 months', NOW() - INTERVAL '5 months'),
('550e8400-e29b-41d4-a716-446655440502', 'sample', '550e8400-e29b-41d4-a716-446655440102', 'initial_qc', '{"concentration": 125.3, "integrity": 8.7, "purity": 1.83, "volume": 450}', true, NULL, '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '5 months', NOW() - INTERVAL '5 months'),
('550e8400-e29b-41d4-a716-446655440503', 'sample', '550e8400-e29b-41d4-a716-446655440103', 'initial_qc', '{"concentration": 85.2, "integrity": 9.1, "purity": 1.88, "volume": 1000}', true, 'Excellent RNA integrity', '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '5 months', NOW() - INTERVAL '5 months'),
('550e8400-e29b-41d4-a716-446655440504', 'sample', '550e8400-e29b-41d4-a716-446655440104', 'viral_load', '{"ct_value": 25.3, "copies_per_ml": 125000, "quality": "good"}', true, 'High viral load detected', '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '3 months', NOW() - INTERVAL '3 months')
ON CONFLICT (id) DO NOTHING;

-- Insert Report Templates
INSERT INTO report_templates (id, name, description, template_type, template_content, parameters_schema, created_by, created_at) VALUES
('550e8400-e29b-41d4-a716-446655440601', 'Monthly Sequencing Summary', 'Standard monthly summary of all sequencing activities', 'summary', '{"sections": ["overview", "by_project", "by_platform", "quality_metrics", "turnaround_time"]}', '{"month": {"type": "string", "format": "YYYY-MM"}}', '550e8400-e29b-41d4-a716-446655440001', NOW() - INTERVAL '1 year'),
('550e8400-e29b-41d4-a716-446655440602', 'Project Quality Report', 'Quality control summary for a specific project', 'quality', '{"sections": ["sample_summary", "qc_metrics", "failed_samples", "recommendations"]}', '{"project_code": {"type": "string"}}', '550e8400-e29b-41d4-a716-446655440001', NOW() - INTERVAL '8 months'),
('550e8400-e29b-41d4-a716-446655440603', 'Storage Utilization Report', 'Analysis of storage capacity and utilization', 'operational', '{"sections": ["current_usage", "by_temperature", "forecast", "recommendations"]}', '{"include_forecast": {"type": "boolean", "default": true}}', '550e8400-e29b-41d4-a716-446655440001', NOW() - INTERVAL '6 months')
ON CONFLICT (id) DO NOTHING;

-- Insert Library Prep Protocols (matching the library_preparation_protocols table that likely exists)
INSERT INTO library_preparation_protocols (id, name, version, description, protocol_steps, compatible_sample_types, created_by, created_at) VALUES
('550e8400-e29b-41d4-a716-446655440701', 'Illumina TruSeq DNA', 'v3.0', 'Standard Illumina TruSeq DNA library preparation', '["DNA extraction", "Fragmentation", "End repair", "A-tailing", "Adapter ligation", "PCR amplification", "Cleanup"]', '["tissue", "blood", "cell_culture"]', '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '1 year'),
('550e8400-e29b-41d4-a716-446655440702', 'NEBNext Ultra II DNA', 'v2.0', 'High efficiency DNA library prep for low input samples', '["DNA quantification", "Fragmentation", "End prep", "Adapter ligation", "U excision", "PCR amplification", "Cleanup"]', '["tissue", "blood", "cell_culture", "cfDNA"]', '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '10 months'),
('550e8400-e29b-41d4-a716-446655440703', 'Illumina RNA Prep', 'v2.0', 'RNA sequencing library preparation', '["RNA extraction", "mRNA enrichment", "Fragmentation", "First strand synthesis", "Second strand synthesis", "A-tailing", "Adapter ligation", "PCR"]', '["tissue", "blood", "cell_culture"]', '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '8 months')
ON CONFLICT (id) DO NOTHING;

-- Insert Batches (in the project_service schema)
INSERT INTO batches (id, batch_number, project_id, description, sample_count, status, created_by, created_at) VALUES
('550e8400-e29b-41d4-a716-446655440801', 'BATCH-2024-001', '550e8400-e29b-41d4-a716-446655440301', 'Initial breast cancer samples', 24, 'completed', '550e8400-e29b-41d4-a716-446655440003', NOW() - INTERVAL '5 months'),
('550e8400-e29b-41d4-a716-446655440802', 'BATCH-2024-002', '550e8400-e29b-41d4-a716-446655440301', 'Second batch breast cancer samples', 24, 'completed', '550e8400-e29b-41d4-a716-446655440003', NOW() - INTERVAL '4 months'),
('550e8400-e29b-41d4-a716-446655440803', 'BATCH-2024-003', '550e8400-e29b-41d4-a716-446655440302', 'COVID surveillance batch March', 48, 'processing', '550e8400-e29b-41d4-a716-446655440003', NOW() - INTERVAL '3 months'),
('550e8400-e29b-41d4-a716-446655440804', 'BATCH-2024-004', '550e8400-e29b-41d4-a716-446655440303', 'Microbiome cohort 1', 36, 'completed', '550e8400-e29b-41d4-a716-446655440005', NOW() - INTERVAL '2 months'),
('550e8400-e29b-41d4-a716-446655440805', 'BATCH-2024-005', '550e8400-e29b-41d4-a716-446655440303', 'Microbiome cohort 2', 36, 'ready', '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '1 week')
ON CONFLICT (id) DO NOTHING;

-- Insert Sequencing Platforms (in sequencing service)
INSERT INTO sequencing_platforms (id, name, model, manufacturer, technology, max_read_length, max_output_gb, typical_run_time_hours, is_active, created_at) VALUES
('550e8400-e29b-41d4-a716-446655440901', 'NovaSeq 6000 #1', 'NovaSeq 6000', 'Illumina', 'SBS', 150, 6000, 44, true, NOW() - INTERVAL '2 years'),
('550e8400-e29b-41d4-a716-446655440902', 'MiSeq #1', 'MiSeq', 'Illumina', 'SBS', 300, 15, 24, true, NOW() - INTERVAL '3 years'),
('550e8400-e29b-41d4-a716-446655440903', 'NextSeq 2000 #1', 'NextSeq 2000', 'Illumina', 'SBS', 150, 360, 29, true, NOW() - INTERVAL '1 year')
ON CONFLICT (id) DO NOTHING;

COMMIT;
