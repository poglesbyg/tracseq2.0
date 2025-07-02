-- TracSeq 2.0 Final Seed Data
-- Matches actual database schema with correct enum values

-- Insert test users (password hash is for 'password123')
INSERT INTO users (id, email, password_hash, first_name, last_name, role, status, lab_affiliation, department, position, email_verified) VALUES
('550e8400-e29b-41d4-a716-446655440001', 'admin@tracseq.lab', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'System', 'Administrator', 'lab_administrator', 'active', 'TracSeq Lab', 'IT', 'System Administrator', true),
('550e8400-e29b-41d4-a716-446655440002', 'john.smith@tracseq.lab', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'John', 'Smith', 'principal_investigator', 'active', 'TracSeq Lab', 'Operations', 'Lab Manager', true),
('550e8400-e29b-41d4-a716-446655440003', 'mary.johnson@tracseq.lab', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'Mary', 'Johnson', 'lab_technician', 'active', 'TracSeq Lab', 'Sequencing', 'Lab Technician', true),
('550e8400-e29b-41d4-a716-446655440004', 'david.williams@tracseq.lab', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'David', 'Williams', 'data_analyst', 'active', 'TracSeq Lab', 'Bioinformatics', 'Data Analyst', true),
('550e8400-e29b-41d4-a716-446655440005', 'sarah.brown@tracseq.lab', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'Sarah', 'Brown', 'lab_technician', 'active', 'TracSeq Lab', 'Sample Prep', 'Lab Technician', true),
('550e8400-e29b-41d4-a716-446655440006', 'robert.jones@university.edu', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'Robert', 'Jones', 'guest', 'active', 'State University', 'Biology', 'Principal Investigator', true)
ON CONFLICT (id) DO NOTHING;

-- Insert Samples with metadata
INSERT INTO samples (id, name, barcode, location, status, metadata, created_at) VALUES
('550e8400-e29b-41d4-a716-446655440101', 'Breast Cancer Sample 001', 'BC2024001', 'Ultra Low Freezer A, Shelf 1, Rack 1', 'in_storage', 
 '{"sample_type": "tissue", "volume_ul": 500, "concentration_ng_ul": 150.5, "project": "CANCER-2024-001", "patient_id": "BC001", "collection_date": "2024-01-20", "description": "Primary tumor tissue"}', NOW() - INTERVAL '5 months'),
('550e8400-e29b-41d4-a716-446655440102', 'Breast Cancer Sample 002', 'BC2024002', 'Ultra Low Freezer A, Shelf 1, Rack 1', 'in_storage',
 '{"sample_type": "tissue", "volume_ul": 450, "concentration_ng_ul": 125.3, "project": "CANCER-2024-001", "patient_id": "BC002", "collection_date": "2024-01-20", "description": "Primary tumor tissue"}', NOW() - INTERVAL '5 months'),
('550e8400-e29b-41d4-a716-446655440103', 'Breast Cancer Sample 003', 'BC2024003', 'Ultra Low Freezer A, Shelf 1, Rack 2', 'in_sequencing',
 '{"sample_type": "blood", "volume_ul": 1000, "concentration_ng_ul": 85.2, "project": "CANCER-2024-001", "patient_id": "BC003", "collection_date": "2024-01-21", "description": "Blood sample"}', NOW() - INTERVAL '5 months'),
('550e8400-e29b-41d4-a716-446655440104', 'COVID Sample 001', 'COV2024001', 'Refrigerator 1, Shelf 2', 'in_storage',
 '{"sample_type": "swab", "volume_ul": 200, "concentration_ng_ul": 45.6, "project": "COVID-2024-002", "patient_id": "COV001", "collection_date": "2024-02-15", "description": "Nasopharyngeal swab"}', NOW() - INTERVAL '3 months'),
('550e8400-e29b-41d4-a716-446655440105', 'COVID Sample 002', 'COV2024002', 'Refrigerator 1, Shelf 2', 'validated',
 '{"sample_type": "swab", "volume_ul": 180, "concentration_ng_ul": 38.9, "project": "COVID-2024-002", "patient_id": "COV002", "collection_date": "2024-02-15", "description": "Nasopharyngeal swab"}', NOW() - INTERVAL '3 months'),
('550e8400-e29b-41d4-a716-446655440106', 'Microbiome Sample 001', 'MB2024001', 'Room Temperature Storage, Shelf 5', 'pending',
 '{"sample_type": "stool", "volume_ul": 300, "concentration_ng_ul": 95.4, "project": "MICROBIOME-2024-003", "participant_id": "MB001", "collection_date": "2024-03-10", "description": "Stool sample"}', NOW() - INTERVAL '2 months'),
('550e8400-e29b-41d4-a716-446655440107', 'Microbiome Sample 002', 'MB2024002', 'Room Temperature Storage, Shelf 5', 'validated',
 '{"sample_type": "stool", "volume_ul": 280, "concentration_ng_ul": 88.7, "project": "MICROBIOME-2024-003", "participant_id": "MB002", "collection_date": "2024-03-10", "description": "Stool sample"}', NOW() - INTERVAL '2 months'),
('550e8400-e29b-41d4-a716-446655440108', 'Rare Disease Sample 001', 'RD2024001', 'Ultra Low Freezer B, Shelf 2', 'completed',
 '{"sample_type": "blood", "volume_ul": 750, "concentration_ng_ul": 110.2, "project": "RARE-2023-001", "patient_id": "RD001", "collection_date": "2023-06-15", "description": "Whole blood for WGS"}', NOW() - INTERVAL '1 year')
ON CONFLICT (id) DO NOTHING;

-- Insert Templates (simplified schema)
INSERT INTO templates (id, name, description, version, is_active, created_by, created_at) VALUES
('550e8400-e29b-41d4-a716-446655440201', 'Sample Submission Form', 'Standard form for sample submission', '2.0', true, '550e8400-e29b-41d4-a716-446655440001', NOW() - INTERVAL '1 year'),
('550e8400-e29b-41d4-a716-446655440202', 'Library Prep Protocol', 'Standard library preparation protocol', '1.5', true, '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '8 months'),
('550e8400-e29b-41d4-a716-446655440203', 'QC Report Template', 'Quality control report template', '1.0', true, '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '6 months'),
('550e8400-e29b-41d4-a716-446655440204', 'Sequencing Request', 'Template for sequencing service requests', '1.2', true, '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '4 months'),
('550e8400-e29b-41d4-a716-446655440205', 'Storage Label', 'Standard storage label template', '1.0', true, '550e8400-e29b-41d4-a716-446655440001', NOW() - INTERVAL '2 months')
ON CONFLICT (id) DO NOTHING;

-- Insert Sequencing Jobs (simplified)
INSERT INTO sequencing_jobs (id, name, run_id, read_type, read_length, requested_coverage, actual_coverage, status, submitted_by, created_at) VALUES
('550e8400-e29b-41d4-a716-446655440301', 'BC Sample 001 Sequencing', 'RUN2024001', 'paired_end', 150, 30.0, 32.5, 'completed', '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '3 months'),
('550e8400-e29b-41d4-a716-446655440302', 'BC Sample 002 Sequencing', 'RUN2024001', 'paired_end', 150, 30.0, 31.8, 'completed', '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '3 months'),
('550e8400-e29b-41d4-a716-446655440303', 'BC Sample 003 Sequencing', 'RUN2024002', 'paired_end', 150, 30.0, NULL, 'running', '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '1 week'),
('550e8400-e29b-41d4-a716-446655440304', 'COVID RNA Sequencing', 'RUN2024003', 'single_end', 100, 20.0, 22.3, 'running', '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '2 days'),
('550e8400-e29b-41d4-a716-446655440305', 'Microbiome 16S Sequencing', 'RUN2024004', 'paired_end', 250, 50.0, NULL, 'pending', '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '1 day')
ON CONFLICT (id) DO NOTHING;

-- Insert Notifications
INSERT INTO notifications (title, message, priority, status, created_at) VALUES
('Sample Ready for Processing', 'Sample BC2024003 is ready for library preparation', 'medium', 'sent', NOW() - INTERVAL '2 days'),
('Sequencing Run Completed', 'Sequencing run RUN2024001 has completed successfully', 'high', 'delivered', NOW() - INTERVAL '3 months'),
('QC Review Required', 'Quality control review needed for new samples', 'high', 'pending', NOW() - INTERVAL '1 day'),
('Storage Temperature Alert', 'Temperature excursion detected in Ultra Low Freezer A', 'urgent', 'sent', NOW() - INTERVAL '12 hours'),
('Report Published', 'Your monthly report has been published', 'low', 'pending', NOW() - INTERVAL '5 days'),
('New Sample Submission', 'New batch of samples received for project MICROBIOME-2024-003', 'medium', 'pending', NOW() - INTERVAL '3 hours'),
('Maintenance Reminder', 'Scheduled maintenance for NovaSeq 6000 next Monday', 'low', 'scheduled', NOW() - INTERVAL '1 hour')
ON CONFLICT DO NOTHING;

-- Insert Events
INSERT INTO events (event_type, severity, metadata, created_at) VALUES
('sample_received', 'info', '{"sample_barcode": "BC2024003", "location": "Ultra Low Freezer A", "project": "CANCER-2024-001"}', NOW() - INTERVAL '5 months'),
('sequencing_started', 'info', '{"run_id": "RUN2024001", "sample_count": 48, "platform": "NovaSeq 6000"}', NOW() - INTERVAL '3 months'),
('temperature_alert', 'warning', '{"location": "ULF-A", "temperature": -68.5, "threshold": -70, "zone": "ultra_low_freezer"}', NOW() - INTERVAL '12 hours'),
('report_generated', 'info', '{"report_type": "summary", "period": "2024-06", "format": "pdf"}', NOW() - INTERVAL '1 month'),
('user_login', 'info', '{"user_email": "admin@tracseq.lab", "ip": "192.168.1.100", "success": true}', NOW() - INTERVAL '1 hour'),
('qc_passed', 'info', '{"sample_id": "BC2024001", "qc_type": "concentration", "value": 150.5, "threshold": 100}', NOW() - INTERVAL '4 months'),
('storage_capacity_warning', 'warning', '{"zone": "ultra_low_freezer", "current_capacity": 78.5, "threshold": 80}', NOW() - INTERVAL '2 days')
ON CONFLICT DO NOTHING;

-- Insert Sample Sheets
INSERT INTO sample_sheets (name, created_by, metadata, created_at) VALUES
('June 2024 Sequencing Run', '550e8400-e29b-41d4-a716-446655440002', 
 '{"instrument": "NovaSeq 6000", "run_date": "2024-06-15", "operator": "John Smith", "samples": 48, "lanes": 4}', NOW() - INTERVAL '3 months'),
('COVID Variant Analysis', '550e8400-e29b-41d4-a716-446655440003',
 '{"project": "COVID-2024-002", "urgency": "high", "special_instructions": "Handle with BSL-2 precautions", "samples": 24}', NOW() - INTERVAL '2 months'),
('Microbiome Diversity Study', '550e8400-e29b-41d4-a716-446655440002',
 '{"project": "MICROBIOME-2024-003", "sample_type": "16S rRNA", "analysis_pipeline": "QIIME2", "samples": 36}', NOW() - INTERVAL '1 month')
ON CONFLICT DO NOTHING;

-- Insert Quality Metrics
INSERT INTO quality_metrics (metric_type, value, unit, threshold_min, threshold_max, passed, measured_at) VALUES
('concentration', 150.5, 'ng/ul', 100.0, 500.0, true, NOW() - INTERVAL '5 months'),
('rin', 8.9, 'score', 7.0, 10.0, true, NOW() - INTERVAL '5 months'),
('a260_280', 1.85, 'ratio', 1.8, 2.0, true, NOW() - INTERVAL '5 months'),
('fragment_size', 350, 'bp', 300, 400, true, NOW() - INTERVAL '4 months'),
('q30_percent', 92.5, 'percent', 80.0, 100.0, true, NOW() - INTERVAL '3 months'),
('cluster_density', 285.5, 'K/mm2', 250.0, 350.0, true, NOW() - INTERVAL '3 months')
ON CONFLICT DO NOTHING;

-- Insert Sequencing Runs
INSERT INTO sequencing_runs (id, run_id, name, instrument, status, total_reads, total_bases, metadata, created_at) VALUES
('550e8400-e29b-41d4-a716-446655440401', 'RUN2024001', 'June NovaSeq Run', 'NovaSeq 6000', 'completed', 4500000000, 675000000000,
 '{"flow_cell": "FC2024001", "chemistry": "v1.5", "run_time_hours": 44, "lanes": 4}', NOW() - INTERVAL '3 months'),
('550e8400-e29b-41d4-a716-446655440402', 'RUN2024002', 'July NovaSeq Run', 'NovaSeq 6000', 'running', 2000000000, 300000000000,
 '{"flow_cell": "FC2024002", "chemistry": "v1.5", "run_time_hours": 20, "lanes": 2}', NOW() - INTERVAL '1 week'),
('550e8400-e29b-41d4-a716-446655440403', 'RUN2024003', 'COVID MiSeq Run', 'MiSeq', 'running', 25000000, 7500000000,
 '{"flow_cell": "MS2024001", "chemistry": "v3", "run_time_hours": 24, "lanes": 1}', NOW() - INTERVAL '2 days')
ON CONFLICT (id) DO NOTHING;

COMMIT;
