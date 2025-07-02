-- TracSeq 2.0 Seed Data - Based on Actual Schema
-- This matches the real database structure

-- Insert test users (password hash is for 'password123')
INSERT INTO users (id, email, password_hash, first_name, last_name, role, status, lab_affiliation, department, position) VALUES
('550e8400-e29b-41d4-a716-446655440001', 'admin@tracseq.lab', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'System', 'Administrator', 'lab_admin', 'active', 'TracSeq Lab', 'IT', 'System Administrator'),
('550e8400-e29b-41d4-a716-446655440002', 'john.smith@tracseq.lab', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'John', 'Smith', 'lab_manager', 'active', 'TracSeq Lab', 'Operations', 'Lab Manager'),
('550e8400-e29b-41d4-a716-446655440003', 'mary.johnson@tracseq.lab', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'Mary', 'Johnson', 'lab_technician', 'active', 'TracSeq Lab', 'Sequencing', 'Lab Technician'),
('550e8400-e29b-41d4-a716-446655440004', 'david.williams@tracseq.lab', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'David', 'Williams', 'data_analyst', 'active', 'TracSeq Lab', 'Bioinformatics', 'Data Analyst'),
('550e8400-e29b-41d4-a716-446655440005', 'sarah.brown@tracseq.lab', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'Sarah', 'Brown', 'lab_technician', 'active', 'TracSeq Lab', 'Sample Prep', 'Lab Technician'),
('550e8400-e29b-41d4-a716-446655440006', 'robert.jones@university.edu', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewYpfQk6F8DpHwCu', 'Robert', 'Jones', 'guest', 'active', 'State University', 'Biology', 'Principal Investigator')
ON CONFLICT (id) DO NOTHING;

-- Insert Samples with metadata
INSERT INTO samples (id, name, barcode, location, status, metadata, created_at) VALUES
('550e8400-e29b-41d4-a716-446655440101', 'Breast Cancer Sample 001', 'BC2024001', 'Ultra Low Freezer A, Shelf 1, Rack 1', 'stored', 
 '{"sample_type": "tissue", "volume_ul": 500, "concentration_ng_ul": 150.5, "project": "CANCER-2024-001", "patient_id": "BC001", "collection_date": "2024-01-20", "description": "Primary tumor tissue"}', NOW() - INTERVAL '5 months'),
('550e8400-e29b-41d4-a716-446655440102', 'Breast Cancer Sample 002', 'BC2024002', 'Ultra Low Freezer A, Shelf 1, Rack 1', 'stored',
 '{"sample_type": "tissue", "volume_ul": 450, "concentration_ng_ul": 125.3, "project": "CANCER-2024-001", "patient_id": "BC002", "collection_date": "2024-01-20", "description": "Primary tumor tissue"}', NOW() - INTERVAL '5 months'),
('550e8400-e29b-41d4-a716-446655440103', 'Breast Cancer Sample 003', 'BC2024003', 'Ultra Low Freezer A, Shelf 1, Rack 2', 'processing',
 '{"sample_type": "blood", "volume_ul": 1000, "concentration_ng_ul": 85.2, "project": "CANCER-2024-001", "patient_id": "BC003", "collection_date": "2024-01-21", "description": "Blood sample"}', NOW() - INTERVAL '5 months'),
('550e8400-e29b-41d4-a716-446655440104', 'COVID Sample 001', 'COV2024001', 'Refrigerator 1, Shelf 2', 'stored',
 '{"sample_type": "swab", "volume_ul": 200, "concentration_ng_ul": 45.6, "project": "COVID-2024-002", "patient_id": "COV001", "collection_date": "2024-02-15", "description": "Nasopharyngeal swab"}', NOW() - INTERVAL '3 months'),
('550e8400-e29b-41d4-a716-446655440105', 'COVID Sample 002', 'COV2024002', 'Refrigerator 1, Shelf 2', 'processing',
 '{"sample_type": "swab", "volume_ul": 180, "concentration_ng_ul": 38.9, "project": "COVID-2024-002", "patient_id": "COV002", "collection_date": "2024-02-15", "description": "Nasopharyngeal swab"}', NOW() - INTERVAL '3 months'),
('550e8400-e29b-41d4-a716-446655440106', 'Microbiome Sample 001', 'MB2024001', 'Room Temperature Storage, Shelf 5', 'pending',
 '{"sample_type": "stool", "volume_ul": 300, "concentration_ng_ul": 95.4, "project": "MICROBIOME-2024-003", "participant_id": "MB001", "collection_date": "2024-03-10", "description": "Stool sample"}', NOW() - INTERVAL '2 months')
ON CONFLICT (id) DO NOTHING;

-- Insert Templates (using the actual schema)
INSERT INTO templates (id, name, description, content, version, is_active, created_by, created_at) VALUES
('550e8400-e29b-41d4-a716-446655440201', 'Sample Submission Form', 'Standard form for sample submission', 
 '{"fields": ["sample_name", "sample_type", "volume", "concentration"], "required": ["sample_name", "sample_type"]}', '2.0', true, '550e8400-e29b-41d4-a716-446655440001', NOW() - INTERVAL '1 year'),
('550e8400-e29b-41d4-a716-446655440202', 'Library Prep Protocol', 'Standard library preparation protocol',
 '{"steps": ["DNA extraction", "Fragmentation", "Adapter ligation", "PCR amplification"], "duration_hours": 6}', '1.5', true, '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '8 months'),
('550e8400-e29b-41d4-a716-446655440203', 'QC Report Template', 'Quality control report template',
 '{"sections": ["Sample Quality", "Library Quality", "Sequencing Metrics"], "format": "pdf"}', '1.0', true, '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '6 months'),
('550e8400-e29b-41d4-a716-446655440204', 'Sequencing Request', 'Template for sequencing service requests',
 '{"required_fields": ["coverage", "read_length", "platform"], "platforms": ["NovaSeq", "MiSeq", "NextSeq"]}', '1.2', true, '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '4 months')
ON CONFLICT (id) DO NOTHING;

-- Insert Storage Locations (using integer IDs)
INSERT INTO storage_locations (name, zone, shelf, rack, box, position, temperature_celsius, is_occupied, created_at) VALUES
('ULF-A-01-01-01-A1', 'ultra_low_freezer', '01', '01', '01', 'A1', -78.5, true, NOW() - INTERVAL '6 months'),
('ULF-A-01-01-01-A2', 'ultra_low_freezer', '01', '01', '01', 'A2', -78.5, true, NOW() - INTERVAL '5 months'),
('ULF-A-01-01-01-A3', 'ultra_low_freezer', '01', '01', '01', 'A3', -78.5, true, NOW() - INTERVAL '4 months'),
('REF-1-02-03-01-C1', 'refrigerator', '02', '03', '01', 'C1', 4.5, true, NOW() - INTERVAL '2 months'),
('REF-1-02-03-01-C2', 'refrigerator', '02', '03', '01', 'C2', 4.5, true, NOW() - INTERVAL '1 month'),
('RT-01-05-02', 'room_temperature', '01', '05', '02', NULL, 22.0, true, NOW() - INTERVAL '2 months')
ON CONFLICT DO NOTHING;

-- Insert Sequencing Jobs (using actual schema)
INSERT INTO sequencing_jobs (id, name, run_id, platform, read_type, read_length, requested_coverage, actual_coverage, status, submitted_by, created_at) VALUES
('550e8400-e29b-41d4-a716-446655440301', 'BC Sample 001 Sequencing', 'RUN2024001', 'NovaSeq 6000', 'paired_end', 150, 30.0, 32.5, 'completed', '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '3 months'),
('550e8400-e29b-41d4-a716-446655440302', 'BC Sample 002 Sequencing', 'RUN2024001', 'NovaSeq 6000', 'paired_end', 150, 30.0, 31.8, 'completed', '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '3 months'),
('550e8400-e29b-41d4-a716-446655440303', 'BC Sample 003 Sequencing', 'RUN2024002', 'NovaSeq 6000', 'paired_end', 150, 30.0, NULL, 'running', '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '1 week')
ON CONFLICT (id) DO NOTHING;

-- Insert Notifications
INSERT INTO notifications (title, message, priority, status, created_at) VALUES
('Sample Ready for Processing', 'Sample BC2024003 is ready for library preparation', 'medium', 'unread', NOW() - INTERVAL '2 days'),
('Sequencing Run Completed', 'Sequencing run RUN2024001 has completed successfully', 'high', 'read', NOW() - INTERVAL '3 months'),
('QC Review Required', 'Quality control review needed for new samples', 'high', 'unread', NOW() - INTERVAL '1 day'),
('Storage Temperature Alert', 'Temperature excursion detected in Ultra Low Freezer A', 'urgent', 'read', NOW() - INTERVAL '12 hours'),
('Report Published', 'Your monthly report has been published', 'low', 'unread', NOW() - INTERVAL '5 days')
ON CONFLICT DO NOTHING;

-- Insert Events
INSERT INTO events (event_type, description, severity, metadata, created_at) VALUES
('sample_received', 'New sample BC2024003 received and logged', 'info', '{"sample_barcode": "BC2024003", "location": "Ultra Low Freezer A"}', NOW() - INTERVAL '5 months'),
('sequencing_started', 'Sequencing run started for RUN2024001', 'info', '{"run_id": "RUN2024001", "sample_count": 48}', NOW() - INTERVAL '3 months'),
('temperature_alert', 'Temperature exceeded threshold in ultra low freezer', 'warning', '{"location": "ULF-A", "temperature": -68.5, "threshold": -70}', NOW() - INTERVAL '12 hours'),
('report_generated', 'Monthly sequencing summary report generated', 'info', '{"report_type": "summary", "period": "2024-06"}', NOW() - INTERVAL '1 month'),
('user_login', 'User admin@tracseq.lab logged in successfully', 'info', '{"user_email": "admin@tracseq.lab", "ip": "192.168.1.100"}', NOW() - INTERVAL '1 hour')
ON CONFLICT DO NOTHING;

-- Insert Sample Sheets (example data)
INSERT INTO sample_sheets (name, description, created_by, metadata, created_at) VALUES
('June 2024 Sequencing Run', 'Sample sheet for June 2024 NovaSeq run', '550e8400-e29b-41d4-a716-446655440002', 
 '{"instrument": "NovaSeq 6000", "run_date": "2024-06-15", "operator": "John Smith"}', NOW() - INTERVAL '3 months'),
('COVID Variant Analysis', 'Sample sheet for COVID variant sequencing', '550e8400-e29b-41d4-a716-446655440003',
 '{"project": "COVID-2024-002", "urgency": "high", "special_instructions": "Handle with BSL-2 precautions"}', NOW() - INTERVAL '2 months')
ON CONFLICT DO NOTHING;

COMMIT;
