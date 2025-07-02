-- TracSeq 2.0 Seed Data - Part 3: Projects and Templates

-- Insert Projects
INSERT INTO projects (id, project_code, name, description, status, priority, principal_investigator_name, principal_investigator_email, institution, start_date, target_end_date, budget_amount, created_by, created_at) VALUES
('proj-001', 'CANCER-2024-001', 'Breast Cancer Genomics Study', 'Comprehensive genomic analysis of breast cancer samples to identify novel biomarkers', 'active', 'high', 'Dr. Emily Chen', 'emily.chen@medical.org', 'City Medical Center', '2024-01-15', '2025-01-15', 250000.00, '550e8400-e29b-41d4-a716-446655440006', NOW() - INTERVAL '6 months'),
('proj-002', 'COVID-2024-002', 'COVID-19 Variant Surveillance', 'Ongoing surveillance of COVID-19 variants in the local population', 'active', 'urgent', 'Dr. Michael Wang', 'michael.wang@health.gov', 'State Health Department', '2024-02-01', '2024-12-31', 500000.00, '550e8400-e29b-41d4-a716-446655440006', NOW() - INTERVAL '5 months'),
('proj-003', 'MICROBIOME-2024-003', 'Gut Microbiome Diversity', 'Study of gut microbiome diversity in different dietary groups', 'active', 'medium', 'Dr. Sarah Martinez', 'sarah.martinez@university.edu', 'State University', '2024-03-01', '2025-03-01', 150000.00, '550e8400-e29b-41d4-a716-446655440006', NOW() - INTERVAL '4 months'),
('proj-004', 'RARE-2023-001', 'Rare Disease Genomics', 'Whole genome sequencing for rare disease diagnosis', 'completed', 'high', 'Dr. James Liu', 'james.liu@genetics.org', 'Genetics Institute', '2023-06-01', '2024-06-01', 300000.00, '550e8400-e29b-41d4-a716-446655440006', NOW() - INTERVAL '1 year'),
('proj-005', 'AGRI-2024-001', 'Crop Resistance Study', 'Genomic analysis of drought-resistant crop varieties', 'planning', 'low', 'Dr. Lisa Anderson', 'lisa.anderson@agri.edu', 'Agricultural Research Center', '2024-09-01', '2025-09-01', 180000.00, '550e8400-e29b-41d4-a716-446655440006', NOW() - INTERVAL '1 month')
ON CONFLICT (id) DO NOTHING;

-- Insert Batches
INSERT INTO batches (id, batch_number, project_id, batch_type, status, sample_count, created_by, created_at) VALUES
('batch-001', 'BATCH-2024-001', 'proj-001', 'sample_receipt', 'completed', 24, '550e8400-e29b-41d4-a716-446655440003', NOW() - INTERVAL '5 months'),
('batch-002', 'BATCH-2024-002', 'proj-001', 'library_prep', 'completed', 24, '550e8400-e29b-41d4-a716-446655440003', NOW() - INTERVAL '4 months'),
('batch-003', 'BATCH-2024-003', 'proj-002', 'sample_receipt', 'in_progress', 48, '550e8400-e29b-41d4-a716-446655440003', NOW() - INTERVAL '3 months'),
('batch-004', 'BATCH-2024-004', 'proj-003', 'sample_receipt', 'completed', 36, '550e8400-e29b-41d4-a716-446655440005', NOW() - INTERVAL '2 months'),
('batch-005', 'BATCH-2024-005', 'proj-003', 'sequencing', 'pending_approval', 36, '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '1 week')
ON CONFLICT (id) DO NOTHING;

-- Insert Templates
INSERT INTO templates (id, name, description, category, content, version, is_active, created_by, created_at, file_type) VALUES
('tmpl-001', 'Sample Submission Form', 'Standard form for sample submission', 'submission', '{"fields": ["sample_name", "sample_type", "volume", "concentration"]}', '2.0', true, '550e8400-e29b-41d4-a716-446655440001', NOW() - INTERVAL '1 year', 'json'),
('tmpl-002', 'Library Prep Protocol', 'Standard library preparation protocol', 'protocol', '{"steps": ["DNA extraction", "Fragmentation", "Adapter ligation", "PCR amplification"]}', '1.5', true, '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '8 months', 'json'),
('tmpl-003', 'QC Report Template', 'Quality control report template', 'report', '{"sections": ["Sample Quality", "Library Quality", "Sequencing Metrics"]}', '1.0', true, '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '6 months', 'json'),
('tmpl-004', 'Sequencing Request', 'Template for sequencing service requests', 'request', '{"required_fields": ["coverage", "read_length", "platform"]}', '1.2', true, '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '4 months', 'json'),
('tmpl-005', 'Storage Label', 'Standard storage label template', 'label', '{"format": "barcode_128", "fields": ["sample_id", "date", "location"]}', '1.0', true, '550e8400-e29b-41d4-a716-446655440001', NOW() - INTERVAL '2 months', 'json'),
('tmpl-006', 'RNA Extraction Protocol', 'Protocol for RNA extraction from tissue samples', 'protocol', '{"steps": ["Tissue homogenization", "RNA isolation", "DNase treatment", "Quality assessment"]}', '2.1', true, '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '3 months', 'json'),
('tmpl-007', 'Sample Chain of Custody', 'Form for tracking sample custody transfers', 'tracking', '{"fields": ["transfer_date", "from_user", "to_user", "reason", "condition"]}', '1.0', true, '550e8400-e29b-41d4-a716-446655440001', NOW() - INTERVAL '5 months', 'json')
ON CONFLICT (id) DO NOTHING;
