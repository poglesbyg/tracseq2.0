-- TracSeq 2.0 Seed Data - Part 5: Reports, Notifications, and Events

-- Insert Reports
INSERT INTO reports (id, title, description, report_type, category, status, data, created_by, created_at) VALUES
('rep-001', 'Monthly Sequencing Summary - June 2024', 'Summary of all sequencing runs completed in June 2024', 'summary', 'sequencing', 'published', '{"total_runs": 15, "total_samples": 245, "success_rate": 98.5, "average_turnaround_days": 5.2}', '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '1 month'),
('rep-002', 'Sample Quality Report - Project CANCER-2024-001', 'Quality control summary for breast cancer genomics project', 'quality_control', 'project', 'published', '{"total_samples": 24, "passed_qc": 23, "failed_qc": 1, "average_quality_score": 8.7}', '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '2 weeks'),
('rep-003', 'Storage Capacity Analysis', 'Current storage utilization and capacity forecast', 'capacity', 'storage', 'draft', '{"ultra_low_utilization": 65.5, "freezer_utilization": 78.2, "refrigerator_utilization": 45.8}', '550e8400-e29b-41d4-a716-446655440001', NOW() - INTERVAL '1 week'),
('rep-004', 'COVID Variant Detection Report', 'Analysis of detected COVID-19 variants in recent samples', 'analysis', 'research', 'published', '{"variants_detected": ["Delta", "Omicron BA.2", "Omicron BA.5"], "dominant_variant": "Omicron BA.5"}', '550e8400-e29b-41d4-a716-446655440006', NOW() - INTERVAL '5 days'),
('rep-005', 'Library Preparation Efficiency Q2 2024', 'Analysis of library preparation success rates and efficiency metrics', 'performance', 'operations', 'published', '{"total_preps": 186, "success_rate": 97.3, "average_yield_ng": 28.5, "average_time_hours": 4.2}', '550e8400-e29b-41d4-a716-446655440002', NOW() - INTERVAL '3 weeks'),
('rep-006', 'Microbiome Diversity Analysis', 'Preliminary results from gut microbiome diversity study', 'analysis', 'research', 'draft', '{"samples_analyzed": 36, "diversity_index": 0.82, "dominant_phyla": ["Bacteroidetes", "Firmicutes"]}', '550e8400-e29b-41d4-a716-446655440006', NOW() - INTERVAL '3 days')
ON CONFLICT (id) DO NOTHING;

-- Insert Notifications
INSERT INTO notifications (id, user_id, title, message, type, priority, status, related_entity_type, related_entity_id, created_at) VALUES
('notif-001', '550e8400-e29b-41d4-a716-446655440003', 'Sample Ready for Processing', 'Sample BC2024003 is ready for library preparation', 'task', 'medium', 'unread', 'sample', 'samp-003', NOW() - INTERVAL '2 days'),
('notif-002', '550e8400-e29b-41d4-a716-446655440002', 'Sequencing Run Completed', 'Sequencing run FC2024001 has completed successfully', 'info', 'high', 'read', 'sequencing_job', 'seq-001', NOW() - INTERVAL '3 months'),
('notif-003', '550e8400-e29b-41d4-a716-446655440004', 'QC Review Required', 'Quality control review needed for batch BATCH-2024-005', 'task', 'high', 'unread', 'batch', 'batch-005', NOW() - INTERVAL '1 day'),
('notif-004', '550e8400-e29b-41d4-a716-446655440001', 'Storage Temperature Alert', 'Temperature excursion detected in Ultra Low Freezer A', 'alert', 'urgent', 'read', 'storage_zone', 'zone-001', NOW() - INTERVAL '12 hours'),
('notif-005', '550e8400-e29b-41d4-a716-446655440006', 'Report Published', 'Your COVID variant analysis report has been published', 'info', 'low', 'unread', 'report', 'rep-004', NOW() - INTERVAL '5 days'),
('notif-006', '550e8400-e29b-41d4-a716-446655440003', 'Library Prep Deadline', 'Library preparation for samples in batch BATCH-2024-005 due tomorrow', 'reminder', 'medium', 'unread', 'batch', 'batch-005', NOW() - INTERVAL '6 hours'),
('notif-007', '550e8400-e29b-41d4-a716-446655440002', 'New Sample Submission', 'New sample batch received for project MICROBIOME-2024-003', 'info', 'medium', 'unread', 'project', 'proj-003', NOW() - INTERVAL '3 hours'),
('notif-008', '550e8400-e29b-41d4-a716-446655440005', 'Maintenance Scheduled', 'NovaSeq 6000 scheduled for maintenance next Monday', 'info', 'low', 'unread', NULL, NULL, NOW() - INTERVAL '1 hour')
ON CONFLICT (id) DO NOTHING;

-- Insert Events (for event monitoring)
INSERT INTO events (id, event_type, event_name, description, severity, source_service, user_id, metadata, created_at) VALUES
('evt-001', 'sample_received', 'Sample Received', 'New sample BC2024003 received and logged', 'info', 'sample_service', '550e8400-e29b-41d4-a716-446655440003', '{"sample_id": "samp-003", "barcode": "BC2024003"}', NOW() - INTERVAL '5 months'),
('evt-002', 'sequencing_started', 'Sequencing Started', 'Sequencing run started for flow cell FC2024001', 'info', 'sequencing_service', '550e8400-e29b-41d4-a716-446655440003', '{"flow_cell_id": "fc-001", "sample_count": 48}', NOW() - INTERVAL '3 months'),
('evt-003', 'temperature_alert', 'Temperature Excursion', 'Temperature exceeded threshold in zone-001', 'warning', 'storage_service', NULL, '{"zone_id": "zone-001", "temperature": -68.5, "threshold": -70}', NOW() - INTERVAL '12 hours'),
('evt-004', 'login_failed', 'Failed Login Attempt', 'Multiple failed login attempts for user jsmith', 'warning', 'auth_service', '550e8400-e29b-41d4-a716-446655440002', '{"attempts": 3, "ip_address": "192.168.1.100"}', NOW() - INTERVAL '6 hours'),
('evt-005', 'report_generated', 'Report Generated', 'Monthly sequencing summary report generated', 'info', 'reports_service', '550e8400-e29b-41d4-a716-446655440002', '{"report_id": "rep-001", "type": "summary"}', NOW() - INTERVAL '1 month'),
('evt-006', 'storage_capacity_warning', 'Storage Capacity Warning', 'Ultra Low Freezer A approaching 80% capacity', 'warning', 'storage_service', NULL, '{"zone_id": "zone-001", "current_capacity": 78.5, "threshold": 80}', NOW() - INTERVAL '2 days'),
('evt-007', 'qc_failed', 'QC Failed', 'Sample failed quality control check', 'error', 'qaqc_service', '550e8400-e29b-41d4-a716-446655440004', '{"sample_id": "samp-005", "qc_type": "concentration", "reason": "Below threshold"}', NOW() - INTERVAL '3 months'),
('evt-008', 'batch_approved', 'Batch Approved', 'Sample batch approved for sequencing', 'info', 'sample_service', '550e8400-e29b-41d4-a716-446655440002', '{"batch_id": "batch-002", "approved_samples": 24}', NOW() - INTERVAL '4 months'),
('evt-009', 'user_created', 'User Created', 'New user account created', 'info', 'auth_service', '550e8400-e29b-41d4-a716-446655440001', '{"user_id": "550e8400-e29b-41d4-a716-446655440006", "role": "researcher"}', NOW() - INTERVAL '2 months'),
('evt-010', 'data_export', 'Data Exported', 'Sequencing data exported for analysis', 'info', 'reports_service', '550e8400-e29b-41d4-a716-446655440006', '{"project_id": "proj-001", "format": "FASTQ", "size_gb": 125.5}', NOW() - INTERVAL '1 week')
ON CONFLICT (id) DO NOTHING;

-- Insert QC Reviews
INSERT INTO qc_reviews (id, sample_id, review_type, status, reviewer_id, review_date, results, notes, created_at) VALUES
('qcr-001', 'samp-001', 'initial_qc', 'approved', '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '5 months', '{"concentration": "pass", "integrity": "pass", "purity": "pass"}', 'High quality sample, suitable for sequencing', NOW() - INTERVAL '5 months'),
('qcr-002', 'samp-002', 'initial_qc', 'approved', '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '5 months', '{"concentration": "pass", "integrity": "pass", "purity": "pass"}', NULL, NOW() - INTERVAL '5 months'),
('qcr-003', 'samp-006', 'library_qc', 'pending', '550e8400-e29b-41d4-a716-446655440004', NULL, NULL, NULL, NOW() - INTERVAL '2 days'),
('qcr-004', 'samp-005', 'initial_qc', 'rejected', '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '3 months', '{"concentration": "fail", "integrity": "pass", "purity": "pass"}', 'Concentration below minimum threshold', NOW() - INTERVAL '3 months'),
('qcr-005', 'samp-003', 'pre_sequencing', 'approved', '550e8400-e29b-41d4-a716-446655440004', NOW() - INTERVAL '2 months', '{"library_quality": "pass", "fragment_size": "pass", "adapter_contamination": "none"}', 'Ready for sequencing', NOW() - INTERVAL '2 months')
ON CONFLICT (id) DO NOTHING;

-- Insert Spreadsheets
INSERT INTO spreadsheets (id, name, description, category, owner_id, is_template, created_at) VALUES
('sheet-001', 'Sample Tracking Master', 'Master spreadsheet for tracking all samples', 'tracking', '550e8400-e29b-41d4-a716-446655440002', false, NOW() - INTERVAL '6 months'),
('sheet-002', 'QC Results Template', 'Template for recording QC results', 'quality_control', '550e8400-e29b-41d4-a716-446655440004', true, NOW() - INTERVAL '4 months'),
('sheet-003', 'Sequencing Run Log', 'Log of all sequencing runs', 'sequencing', '550e8400-e29b-41d4-a716-446655440002', false, NOW() - INTERVAL '3 months'),
('sheet-004', 'Storage Inventory', 'Current inventory of all storage locations', 'storage', '550e8400-e29b-41d4-a716-446655440001', false, NOW() - INTERVAL '2 months'),
('sheet-005', 'Project Budget Tracker', 'Budget tracking for all active projects', 'finance', '550e8400-e29b-41d4-a716-446655440001', false, NOW() - INTERVAL '5 months')
ON CONFLICT (id) DO NOTHING;

-- Insert recent Audit Logs
INSERT INTO audit_logs (user_id, action, entity_type, entity_id, changes, ip_address, user_agent, created_at) VALUES
('550e8400-e29b-41d4-a716-446655440003', 'create', 'sample', 'samp-010', '{"status": "created", "barcode": "MB2024003"}', '192.168.1.50', 'Mozilla/5.0', NOW() - INTERVAL '2 hours'),
('550e8400-e29b-41d4-a716-446655440002', 'update', 'sequencing_job', 'seq-003', '{"status": "pending -> in_progress"}', '192.168.1.51', 'Mozilla/5.0', NOW() - INTERVAL '1 hour'),
('550e8400-e29b-41d4-a716-446655440001', 'login', 'user', '550e8400-e29b-41d4-a716-446655440001', '{"success": true}', '192.168.1.10', 'Mozilla/5.0', NOW() - INTERVAL '30 minutes'),
('550e8400-e29b-41d4-a716-446655440004', 'approve', 'qc_review', 'qcr-005', '{"status": "pending -> approved"}', '192.168.1.52', 'Mozilla/5.0', NOW() - INTERVAL '15 minutes'),
('550e8400-e29b-41d4-a716-446655440006', 'view', 'report', 'rep-004', '{"action": "downloaded"}', '192.168.1.60', 'Mozilla/5.0', NOW() - INTERVAL '5 minutes')
ON CONFLICT DO NOTHING;
