-- TracSeq 2.0 Seed Data - Part 4: Samples and Sequencing

-- Insert Samples
INSERT INTO samples (id, barcode, name, sample_type, description, volume_ul, concentration_ng_ul, quality_score, collection_date, received_date, storage_location_id, project_id, batch_id, submitted_by, status, created_at) VALUES
-- Breast Cancer samples
('samp-001', 'BC2024001', 'Breast Cancer Sample 001', 'tissue', 'Primary tumor tissue from patient BC001', 500.0, 150.5, 8.9, '2024-01-20', '2024-01-22', 'loc-001', 'proj-001', 'batch-001', '550e8400-e29b-41d4-a716-446655440006', 'stored', NOW() - INTERVAL '5 months'),
('samp-002', 'BC2024002', 'Breast Cancer Sample 002', 'tissue', 'Primary tumor tissue from patient BC002', 450.0, 125.3, 8.7, '2024-01-20', '2024-01-22', 'loc-002', 'proj-001', 'batch-001', '550e8400-e29b-41d4-a716-446655440006', 'stored', NOW() - INTERVAL '5 months'),
('samp-003', 'BC2024003', 'Breast Cancer Sample 003', 'blood', 'Blood sample from patient BC003', 1000.0, 85.2, 9.1, '2024-01-21', '2024-01-22', 'loc-003', 'proj-001', 'batch-001', '550e8400-e29b-41d4-a716-446655440006', 'in_sequencing', NOW() - INTERVAL '5 months'),
-- COVID samples
('samp-004', 'COV2024001', 'COVID Sample 001', 'swab', 'Nasopharyngeal swab from patient COV001', 200.0, 45.6, 7.8, '2024-02-15', '2024-02-15', 'loc-006', 'proj-002', 'batch-003', '550e8400-e29b-41d4-a716-446655440006', 'stored', NOW() - INTERVAL '3 months'),
('samp-005', 'COV2024002', 'COVID Sample 002', 'swab', 'Nasopharyngeal swab from patient COV002', 180.0, 38.9, 7.5, '2024-02-15', '2024-02-15', 'loc-007', 'proj-002', 'batch-003', '550e8400-e29b-41d4-a716-446655440006', 'in_processing', NOW() - INTERVAL '3 months'),
-- Microbiome samples
('samp-006', 'MB2024001', 'Microbiome Sample 001', 'stool', 'Stool sample from participant MB001', 300.0, 95.4, 8.2, '2024-03-10', '2024-03-11', 'loc-005', 'proj-003', 'batch-004', '550e8400-e29b-41d4-a716-446655440006', 'ready_for_sequencing', NOW() - INTERVAL '2 months'),
('samp-007', 'MB2024002', 'Microbiome Sample 002', 'stool', 'Stool sample from participant MB002', 280.0, 88.7, 8.0, '2024-03-10', '2024-03-11', 'loc-008', 'proj-003', 'batch-004', '550e8400-e29b-41d4-a716-446655440006', 'stored', NOW() - INTERVAL '2 months'),
-- Additional samples
('samp-008', 'BC2024004', 'Breast Cancer Sample 004', 'tissue', 'Metastatic tissue from patient BC004', 400.0, 110.2, 8.5, '2024-01-25', '2024-01-26', 'loc-010', 'proj-001', 'batch-001', '550e8400-e29b-41d4-a716-446655440006', 'stored', NOW() - INTERVAL '5 months'),
('samp-009', 'COV2024003', 'COVID Sample 003', 'saliva', 'Saliva sample from patient COV003', 300.0, 55.3, 8.0, '2024-02-20', '2024-02-20', 'loc-011', 'proj-002', 'batch-003', '550e8400-e29b-41d4-a716-446655440006', 'ready_for_sequencing', NOW() - INTERVAL '3 months'),
('samp-010', 'MB2024003', 'Microbiome Sample 003', 'stool', 'Stool sample from participant MB003', 250.0, 92.1, 7.9, '2024-03-15', '2024-03-16', 'loc-012', 'proj-003', 'batch-004', '550e8400-e29b-41d4-a716-446655440006', 'stored', NOW() - INTERVAL '2 months')
ON CONFLICT (id) DO NOTHING;

-- Insert Sample Quality Control
INSERT INTO sample_quality_control (id, sample_id, qc_type, result, passed, performed_by, performed_at, notes) VALUES
('qc-001', 'samp-001', 'concentration', '{"measured": 150.5, "expected_min": 100, "unit": "ng/ul"}', true, '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '5 months', 'Good quality DNA'),
('qc-002', 'samp-001', 'integrity', '{"rin": 8.9, "threshold": 7.0}', true, '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '5 months', 'High integrity'),
('qc-003', 'samp-002', 'concentration', '{"measured": 125.3, "expected_min": 100, "unit": "ng/ul"}', true, '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '5 months', NULL),
('qc-004', 'samp-003', 'purity', '{"a260_280": 1.85, "acceptable_range": [1.8, 2.0]}', true, '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '5 months', 'Pure DNA'),
('qc-005', 'samp-004', 'viral_load', '{"ct_value": 25.3, "positive_threshold": 35}', true, '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '3 months', 'High viral load'),
('qc-006', 'samp-006', 'contamination', '{"bacterial_contamination": false, "host_dna": 2.1}', true, '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '2 months', 'No contamination detected'),
('qc-007', 'samp-009', 'viral_load', '{"ct_value": 28.7, "positive_threshold": 35}', true, '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '3 months', 'Moderate viral load')
ON CONFLICT (id) DO NOTHING;

-- Insert Library Preparations
INSERT INTO library_preparations (id, sample_id, protocol_name, protocol_version, kit_name, kit_lot_number, input_amount_ng, final_concentration_ng_ul, final_volume_ul, fragment_size_bp, quality_score, prepared_by, preparation_date, status, created_at) VALUES
('lib-001', 'samp-001', 'Illumina TruSeq DNA', 'v3.0', 'TruSeq DNA Library Prep Kit', 'LOT2024A123', 100.0, 25.5, 50.0, 350, 9.2, '550e8400-e29b-41d4-a716-446655440003', NOW() - INTERVAL '4 months', 'completed', NOW() - INTERVAL '4 months'),
('lib-002', 'samp-002', 'Illumina TruSeq DNA', 'v3.0', 'TruSeq DNA Library Prep Kit', 'LOT2024A123', 100.0, 22.8, 50.0, 340, 8.9, '550e8400-e29b-41d4-a716-446655440003', NOW() - INTERVAL '4 months', 'completed', NOW() - INTERVAL '4 months'),
('lib-003', 'samp-003', 'NEBNext Ultra II DNA', 'v2.0', 'NEBNext Ultra II DNA Library Prep Kit', 'LOT2024B456', 50.0, 18.9, 40.0, 360, 8.7, '550e8400-e29b-41d4-a716-446655440005', NOW() - INTERVAL '4 months', 'completed', NOW() - INTERVAL '4 months'),
('lib-004', 'samp-006', 'Illumina TruSeq DNA', 'v3.0', 'TruSeq DNA Library Prep Kit', 'LOT2024C789', 75.0, 20.2, 45.0, 345, 8.8, '550e8400-e29b-41d4-a716-446655440003', NOW() - INTERVAL '1 month', 'ready_for_sequencing', NOW() - INTERVAL '1 month'),
('lib-005', 'samp-009', 'Illumina RNA Prep', 'v2.0', 'Illumina RNA Prep Kit', 'LOT2024D012', 50.0, 15.5, 40.0, 300, 8.5, '550e8400-e29b-41d4-a716-446655440005', NOW() - INTERVAL '2 months', 'ready_for_sequencing', NOW() - INTERVAL '2 months')
ON CONFLICT (id) DO NOTHING;

-- Insert Flow Cells
INSERT INTO flow_cells (id, flow_cell_barcode, flow_cell_type, platform, run_name, lanes_count, samples_per_lane, created_by, created_at, status) VALUES
('fc-001', 'FC2024001', 'S4', 'NovaSeq 6000', 'Run_2024_06_15_NovaSeq', 4, 12, '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '3 months', 'completed'),
('fc-002', 'FC2024002', 'SP', 'NovaSeq 6000', 'Run_2024_07_01_NovaSeq', 2, 24, '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '2 months', 'in_sequencing'),
('fc-003', 'FC2024003', 'S2', 'NovaSeq 6000', 'Run_2024_07_15_NovaSeq', 2, 16, '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '1 month', 'planning')
ON CONFLICT (id) DO NOTHING;

-- Insert Sequencing Jobs
INSERT INTO sequencing_jobs (id, job_name, sample_id, library_prep_id, flow_cell_id, lane_number, platform, read_type, read_length, requested_coverage, actual_coverage, status, submitted_by, assigned_to, started_at, completed_at, created_at) VALUES
('seq-001', 'BC Sample 001 Sequencing', 'samp-001', 'lib-001', 'fc-001', 1, 'NovaSeq 6000', 'paired_end', 150, 30.0, 32.5, 'completed', '550e8400-e29b-41d4-a716-446655440002', '550e8400-e29b-41d4-a716-446655440003', NOW() - INTERVAL '3 months', NOW() - INTERVAL '3 months' + INTERVAL '2 days', NOW() - INTERVAL '3 months'),
('seq-002', 'BC Sample 002 Sequencing', 'samp-002', 'lib-002', 'fc-001', 1, 'NovaSeq 6000', 'paired_end', 150, 30.0, 31.8, 'completed', '550e8400-e29b-41d4-a716-446655440002', '550e8400-e29b-41d4-a716-446655440003', NOW() - INTERVAL '3 months', NOW() - INTERVAL '3 months' + INTERVAL '2 days', NOW() - INTERVAL '3 months'),
('seq-003', 'BC Sample 003 Sequencing', 'samp-003', 'lib-003', 'fc-002', 1, 'NovaSeq 6000', 'paired_end', 150, 30.0, NULL, 'in_progress', '550e8400-e29b-41d4-a716-446655440002', '550e8400-e29b-41d4-a716-446655440003', NOW() - INTERVAL '2 days', NULL, NOW() - INTERVAL '1 week'),
('seq-004', 'Microbiome Sample 001', 'samp-006', 'lib-004', 'fc-003', 2, 'NovaSeq 6000', 'paired_end', 150, 50.0, NULL, 'pending', '550e8400-e29b-41d4-a716-446655440002', NULL, NULL, NULL, NOW() - INTERVAL '3 days'),
('seq-005', 'COVID Sample RNA-Seq', 'samp-009', 'lib-005', 'fc-002', 2, 'NovaSeq 6000', 'paired_end', 100, 20.0, 22.3, 'in_progress', '550e8400-e29b-41d4-a716-446655440002', '550e8400-e29b-41d4-a716-446655440003', NOW() - INTERVAL '1 day', NULL, NOW() - INTERVAL '2 weeks')
ON CONFLICT (id) DO NOTHING;

-- Insert Sequencing Quality Metrics
INSERT INTO sequencing_quality_metrics (id, sequencing_job_id, metric_type, value, threshold, passed, created_at) VALUES
('sqm-001', 'seq-001', 'total_reads', 450000000, 400000000, true, NOW() - INTERVAL '3 months'),
('sqm-002', 'seq-001', 'q30_percent', 92.5, 80.0, true, NOW() - INTERVAL '3 months'),
('sqm-003', 'seq-001', 'mean_quality_score', 35.8, 30.0, true, NOW() - INTERVAL '3 months'),
('sqm-004', 'seq-002', 'total_reads', 425000000, 400000000, true, NOW() - INTERVAL '3 months'),
('sqm-005', 'seq-002', 'q30_percent', 91.2, 80.0, true, NOW() - INTERVAL '3 months'),
('sqm-006', 'seq-002', 'cluster_density', 285.5, 250.0, true, NOW() - INTERVAL '3 months')
ON CONFLICT (id) DO NOTHING;

-- Insert Storage Movement History
INSERT INTO storage_movement_history (id, sample_id, from_location_id, to_location_id, moved_by, reason, created_at) VALUES
('move-001', 'samp-001', NULL, 'loc-001', '550e8400-e29b-41d4-a716-446655440003', 'Initial storage after receipt', NOW() - INTERVAL '5 months'),
('move-002', 'samp-002', NULL, 'loc-002', '550e8400-e29b-41d4-a716-446655440003', 'Initial storage after receipt', NOW() - INTERVAL '5 months'),
('move-003', 'samp-006', 'loc-008', 'loc-005', '550e8400-e29b-41d4-a716-446655440005', 'Moved for library preparation', NOW() - INTERVAL '1 month'),
('move-004', 'samp-003', 'loc-003', 'loc-001', '550e8400-e29b-41d4-a716-446655440003', 'Consolidating samples', NOW() - INTERVAL '2 weeks')
ON CONFLICT (id) DO NOTHING;
