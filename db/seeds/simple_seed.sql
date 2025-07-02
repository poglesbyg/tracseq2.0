-- TracSeq 2.0 Simple Seed Data
-- This works with the existing database structure

-- Insert test users (password hash is for 'password123')
INSERT INTO users (id, username, email, password_hash, full_name, is_active, created_at, updated_at) VALUES
('550e8400-e29b-41d4-a716-446655440001', 'admin', 'admin@tracseq.lab', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'System Administrator', true, NOW() - INTERVAL '1 year', NOW()),
('550e8400-e29b-41d4-a716-446655440002', 'jsmith', 'john.smith@tracseq.lab', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'John Smith', true, NOW() - INTERVAL '6 months', NOW()),
('550e8400-e29b-41d4-a716-446655440003', 'mjohnson', 'mary.johnson@tracseq.lab', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'Mary Johnson', true, NOW() - INTERVAL '8 months', NOW()),
('550e8400-e29b-41d4-a716-446655440004', 'dwilliams', 'david.williams@tracseq.lab', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'David Williams', true, NOW() - INTERVAL '4 months', NOW()),
('550e8400-e29b-41d4-a716-446655440005', 'sbrown', 'sarah.brown@tracseq.lab', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'Sarah Brown', true, NOW() - INTERVAL '3 months', NOW()),
('550e8400-e29b-41d4-a716-446655440006', 'rjones', 'robert.jones@university.edu', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'Dr. Robert Jones', true, NOW() - INTERVAL '2 months', NOW())
ON CONFLICT (id) DO NOTHING;

-- Insert Projects
INSERT INTO projects (id, project_code, name, description, status, priority, principal_investigator_name, principal_investigator_email, institution, start_date, target_end_date, budget_amount, created_by, created_at) VALUES
('proj-001', 'CANCER-2024-001', 'Breast Cancer Genomics Study', 'Comprehensive genomic analysis of breast cancer samples to identify novel biomarkers', 'active', 'high', 'Dr. Emily Chen', 'emily.chen@medical.org', 'City Medical Center', '2024-01-15', '2025-01-15', 250000.00, '550e8400-e29b-41d4-a716-446655440006', NOW() - INTERVAL '6 months'),
('proj-002', 'COVID-2024-002', 'COVID-19 Variant Surveillance', 'Ongoing surveillance of COVID-19 variants in the local population', 'active', 'urgent', 'Dr. Michael Wang', 'michael.wang@health.gov', 'State Health Department', '2024-02-01', '2024-12-31', 500000.00, '550e8400-e29b-41d4-a716-446655440006', NOW() - INTERVAL '5 months'),
('proj-003', 'MICROBIOME-2024-003', 'Gut Microbiome Diversity', 'Study of gut microbiome diversity in different dietary groups', 'active', 'medium', 'Dr. Sarah Martinez', 'sarah.martinez@university.edu', 'State University', '2024-03-01', '2025-03-01', 150000.00, '550e8400-e29b-41d4-a716-446655440006', NOW() - INTERVAL '4 months'),
('proj-004', 'RARE-2023-001', 'Rare Disease Genomics', 'Whole genome sequencing for rare disease diagnosis', 'completed', 'high', 'Dr. James Liu', 'james.liu@genetics.org', 'Genetics Institute', '2023-06-01', '2024-06-01', 300000.00, '550e8400-e29b-41d4-a716-446655440006', NOW() - INTERVAL '1 year'),
('proj-005', 'AGRI-2024-001', 'Crop Resistance Study', 'Genomic analysis of drought-resistant crop varieties', 'planning', 'low', 'Dr. Lisa Anderson', 'lisa.anderson@agri.edu', 'Agricultural Research Center', '2024-09-01', '2025-09-01', 180000.00, '550e8400-e29b-41d4-a716-446655440006', NOW() - INTERVAL '1 month')
ON CONFLICT (id) DO NOTHING;

-- Insert Templates
INSERT INTO templates (id, name, description, category, content, version, is_active, created_by, created_at) VALUES
('tmpl-001', 'Sample Submission Form', 'Standard form for sample submission', 'submission', '{"fields": ["sample_name", "sample_type", "volume", "concentration"]}', '2.0', true, '550e8400-e29b-41d4-a716-446655440001', NOW() - INTERVAL '1 year'),
('tmpl-002', 'Library Prep Protocol', 'Standard library preparation protocol', 'protocol', '{"steps": ["DNA extraction", "Fragmentation", "Adapter ligation", "PCR amplification"]}', '1.5', true, '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '8 months'),
('tmpl-003', 'QC Report Template', 'Quality control report template', 'report', '{"sections": ["Sample Quality", "Library Quality", "Sequencing Metrics"]}', '1.0', true, '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '6 months'),
('tmpl-004', 'Sequencing Request', 'Template for sequencing service requests', 'request', '{"required_fields": ["coverage", "read_length", "platform"]}', '1.2', true, '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '4 months'),
('tmpl-005', 'Storage Label', 'Standard storage label template', 'label', '{"format": "barcode_128", "fields": ["sample_id", "date", "location"]}', '1.0', true, '550e8400-e29b-41d4-a716-446655440001', NOW() - INTERVAL '2 months')
ON CONFLICT (id) DO NOTHING;

-- Insert Storage Locations
INSERT INTO storage_locations (id, name, location_type, zone, shelf, rack, box, position, temperature_celsius, is_occupied, created_at) VALUES
('loc-001', 'ULF-A-01-01-01-A1', 'box_position', 'ultra_low_freezer', '01', '01', '01', 'A1', -78.5, true, NOW() - INTERVAL '6 months'),
('loc-002', 'ULF-A-01-01-01-A2', 'box_position', 'ultra_low_freezer', '01', '01', '01', 'A2', -78.5, true, NOW() - INTERVAL '5 months'),
('loc-003', 'ULF-A-01-01-01-A3', 'box_position', 'ultra_low_freezer', '01', '01', '01', 'A3', -78.5, true, NOW() - INTERVAL '4 months'),
('loc-004', 'ULF-A-01-01-01-A4', 'box_position', 'ultra_low_freezer', '01', '01', '01', 'A4', -78.5, false, NOW() - INTERVAL '4 months'),
('loc-005', 'ULF-A-01-01-02-B1', 'box_position', 'ultra_low_freezer', '01', '01', '02', 'B1', -78.5, true, NOW() - INTERVAL '3 months'),
('loc-006', 'REF-1-02-03-01-C1', 'box_position', 'refrigerator', '02', '03', '01', 'C1', 4.5, true, NOW() - INTERVAL '2 months'),
('loc-007', 'REF-1-02-03-01-C2', 'box_position', 'refrigerator', '02', '03', '01', 'C2', 4.5, true, NOW() - INTERVAL '1 month')
ON CONFLICT (id) DO NOTHING;

-- Insert Samples
INSERT INTO samples (id, name, barcode, description, volume_ul, concentration_ng_ul, storage_location_id, submitted_by, status, created_at) VALUES
('samp-001', 'Breast Cancer Sample 001', 'BC2024001', 'Primary tumor tissue from patient BC001', 500.0, 150.5, 'loc-001', '550e8400-e29b-41d4-a716-446655440006', 'stored', NOW() - INTERVAL '5 months'),
('samp-002', 'Breast Cancer Sample 002', 'BC2024002', 'Primary tumor tissue from patient BC002', 450.0, 125.3, 'loc-002', '550e8400-e29b-41d4-a716-446655440006', 'stored', NOW() - INTERVAL '5 months'),
('samp-003', 'Breast Cancer Sample 003', 'BC2024003', 'Blood sample from patient BC003', 1000.0, 85.2, 'loc-003', '550e8400-e29b-41d4-a716-446655440006', 'in_sequencing', NOW() - INTERVAL '5 months'),
('samp-004', 'COVID Sample 001', 'COV2024001', 'Nasopharyngeal swab from patient COV001', 200.0, 45.6, 'loc-006', '550e8400-e29b-41d4-a716-446655440006', 'stored', NOW() - INTERVAL '3 months'),
('samp-005', 'COVID Sample 002', 'COV2024002', 'Nasopharyngeal swab from patient COV002', 180.0, 38.9, 'loc-007', '550e8400-e29b-41d4-a716-446655440006', 'in_processing', NOW() - INTERVAL '3 months'),
('samp-006', 'Microbiome Sample 001', 'MB2024001', 'Stool sample from participant MB001', 300.0, 95.4, 'loc-005', '550e8400-e29b-41d4-a716-446655440006', 'ready_for_sequencing', NOW() - INTERVAL '2 months')
ON CONFLICT (id) DO NOTHING;

-- Insert Sample Storage Locations (linking table)
INSERT INTO sample_storage_locations (sample_id, location_id, stored_at, stored_by) VALUES
('samp-001', 'loc-001', NOW() - INTERVAL '5 months', '550e8400-e29b-41d4-a716-446655440003'),
('samp-002', 'loc-002', NOW() - INTERVAL '5 months', '550e8400-e29b-41d4-a716-446655440003'),
('samp-003', 'loc-003', NOW() - INTERVAL '5 months', '550e8400-e29b-41d4-a716-446655440003'),
('samp-004', 'loc-006', NOW() - INTERVAL '3 months', '550e8400-e29b-41d4-a716-446655440003'),
('samp-005', 'loc-007', NOW() - INTERVAL '3 months', '550e8400-e29b-41d4-a716-446655440003'),
('samp-006', 'loc-005', NOW() - INTERVAL '2 months', '550e8400-e29b-41d4-a716-446655440003')
ON CONFLICT DO NOTHING;

-- Insert Library Preparations
INSERT INTO library_preparations (id, sample_id, protocol_name, protocol_version, kit_name, kit_lot_number, input_amount_ng, final_concentration_ng_ul, final_volume_ul, fragment_size_bp, quality_score, prepared_by, preparation_date, status, created_at) VALUES
('lib-001', 'samp-001', 'Illumina TruSeq DNA', 'v3.0', 'TruSeq DNA Library Prep Kit', 'LOT2024A123', 100.0, 25.5, 50.0, 350, 9.2, '550e8400-e29b-41d4-a716-446655440003', NOW() - INTERVAL '4 months', 'completed', NOW() - INTERVAL '4 months'),
('lib-002', 'samp-002', 'Illumina TruSeq DNA', 'v3.0', 'TruSeq DNA Library Prep Kit', 'LOT2024A123', 100.0, 22.8, 50.0, 340, 8.9, '550e8400-e29b-41d4-a716-446655440003', NOW() - INTERVAL '4 months', 'completed', NOW() - INTERVAL '4 months'),
('lib-003', 'samp-003', 'NEBNext Ultra II DNA', 'v2.0', 'NEBNext Ultra II DNA Library Prep Kit', 'LOT2024B456', 50.0, 18.9, 40.0, 360, 8.7, '550e8400-e29b-41d4-a716-446655440005', NOW() - INTERVAL '4 months', 'completed', NOW() - INTERVAL '4 months')
ON CONFLICT (id) DO NOTHING;

-- Insert Flow Cells
INSERT INTO flow_cells (id, flow_cell_barcode, flow_cell_type, platform, run_name, lanes_count, samples_per_lane, created_by, created_at, status) VALUES
('fc-001', 'FC2024001', 'S4', 'NovaSeq 6000', 'Run_2024_06_15_NovaSeq', 4, 12, '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '3 months', 'completed'),
('fc-002', 'FC2024002', 'SP', 'NovaSeq 6000', 'Run_2024_07_01_NovaSeq', 2, 24, '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '2 months', 'in_sequencing'),
('fc-003', 'FC2024003', 'S2', 'NovaSeq 6000', 'Run_2024_07_15_NovaSeq', 2, 16, '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '1 month', 'planning')
ON CONFLICT (id) DO NOTHING;

-- Insert Sequencing Jobs
INSERT INTO sequencing_jobs (id, name, sample_id, run_id, platform, read_type, read_length, requested_coverage, actual_coverage, status, submitted_by, created_at) VALUES
('seq-001', 'BC Sample 001 Sequencing', 'samp-001', 'fc-001', 'NovaSeq 6000', 'paired_end', 150, 30.0, 32.5, 'completed', '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '3 months'),
('seq-002', 'BC Sample 002 Sequencing', 'samp-002', 'fc-001', 'NovaSeq 6000', 'paired_end', 150, 30.0, 31.8, 'completed', '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '3 months'),
('seq-003', 'BC Sample 003 Sequencing', 'samp-003', 'fc-002', 'NovaSeq 6000', 'paired_end', 150, 30.0, NULL, 'running', '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '1 week')
ON CONFLICT (id) DO NOTHING;

-- Insert Quality Reports
INSERT INTO quality_reports (id, sample_id, report_type, metrics, passed, reviewed_by, reviewed_at, created_at) VALUES
('qr-001', 'samp-001', 'initial_qc', '{"concentration": 150.5, "integrity": 8.9, "purity": 1.85}', true, '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '5 months', NOW() - INTERVAL '5 months'),
('qr-002', 'samp-002', 'initial_qc', '{"concentration": 125.3, "integrity": 8.7, "purity": 1.83}', true, '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '5 months', NOW() - INTERVAL '5 months'),
('qr-003', 'samp-003', 'initial_qc', '{"concentration": 85.2, "integrity": 9.1, "purity": 1.88}', true, '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '5 months', NOW() - INTERVAL '5 months')
ON CONFLICT (id) DO NOTHING;

-- Insert Reports
INSERT INTO generated_reports (id, title, description, report_type, content, metadata, created_by, created_at) VALUES
('rep-001', 'Monthly Sequencing Summary - June 2024', 'Summary of all sequencing runs completed in June 2024', 'summary', '{"total_runs": 15, "total_samples": 245, "success_rate": 98.5}', '{"period": "2024-06"}', '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '1 month'),
('rep-002', 'Sample Quality Report - Project CANCER-2024-001', 'Quality control summary for breast cancer genomics project', 'quality_control', '{"total_samples": 24, "passed_qc": 23, "failed_qc": 1}', '{"project_id": "proj-001"}', '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '2 weeks'),
('rep-003', 'Storage Capacity Analysis', 'Current storage utilization and capacity forecast', 'capacity', '{"ultra_low_utilization": 65.5, "freezer_utilization": 78.2}', '{"generated_date": "2024-06-30"}', '550e8400-e29b-41d4-a716-446655440001', NOW() - INTERVAL '1 week')
ON CONFLICT (id) DO NOTHING;

-- Insert Notifications
INSERT INTO notifications (id, user_id, title, message, priority, status, created_at) VALUES
('notif-001', '550e8400-e29b-41d4-a716-446655440003', 'Sample Ready for Processing', 'Sample BC2024003 is ready for library preparation', 'medium', 'unread', NOW() - INTERVAL '2 days'),
('notif-002', '550e8400-e29b-41d4-a716-446655440002', 'Sequencing Run Completed', 'Sequencing run FC2024001 has completed successfully', 'high', 'read', NOW() - INTERVAL '3 months'),
('notif-003', '550e8400-e29b-41d4-a716-446655440004', 'QC Review Required', 'Quality control review needed for new samples', 'high', 'unread', NOW() - INTERVAL '1 day'),
('notif-004', '550e8400-e29b-41d4-a716-446655440001', 'Storage Temperature Alert', 'Temperature excursion detected in Ultra Low Freezer A', 'urgent', 'read', NOW() - INTERVAL '12 hours'),
('notif-005', '550e8400-e29b-41d4-a716-446655440006', 'Report Published', 'Your monthly report has been published', 'low', 'unread', NOW() - INTERVAL '5 days')
ON CONFLICT (id) DO NOTHING;

-- Insert Events
INSERT INTO events (id, event_type, event_category, description, severity, metadata, created_at) VALUES
('evt-001', 'sample_received', 'samples', 'New sample BC2024003 received and logged', 'info', '{"sample_id": "samp-003", "barcode": "BC2024003"}', NOW() - INTERVAL '5 months'),
('evt-002', 'sequencing_started', 'sequencing', 'Sequencing run started for flow cell FC2024001', 'info', '{"flow_cell_id": "fc-001", "sample_count": 48}', NOW() - INTERVAL '3 months'),
('evt-003', 'temperature_alert', 'storage', 'Temperature exceeded threshold in ultra low freezer', 'warning', '{"location": "ULF-A", "temperature": -68.5}', NOW() - INTERVAL '12 hours'),
('evt-004', 'report_generated', 'reports', 'Monthly sequencing summary report generated', 'info', '{"report_id": "rep-001", "type": "summary"}', NOW() - INTERVAL '1 month'),
('evt-005', 'user_login', 'auth', 'User admin logged in successfully', 'info', '{"user_id": "550e8400-e29b-41d4-a716-446655440001"}', NOW() - INTERVAL '1 hour')
ON CONFLICT (id) DO NOTHING;

-- Update storage location occupancy
UPDATE storage_locations SET is_occupied = true WHERE id IN ('loc-001', 'loc-002', 'loc-003', 'loc-005', 'loc-006', 'loc-007');

-- Insert sample quality metrics
INSERT INTO quality_metrics (id, sample_id, metric_type, value, unit, threshold_min, threshold_max, passed, measured_at) VALUES
('qm-001', 'samp-001', 'concentration', 150.5, 'ng/ul', 100.0, 500.0, true, NOW() - INTERVAL '5 months'),
('qm-002', 'samp-001', 'rin', 8.9, 'score', 7.0, 10.0, true, NOW() - INTERVAL '5 months'),
('qm-003', 'samp-002', 'concentration', 125.3, 'ng/ul', 100.0, 500.0, true, NOW() - INTERVAL '5 months'),
('qm-004', 'samp-003', 'a260_280', 1.85, 'ratio', 1.8, 2.0, true, NOW() - INTERVAL '5 months')
ON CONFLICT (id) DO NOTHING;

COMMIT;
