-- Seed data for templates
INSERT INTO templates (id, name, description, file_path, file_type, metadata, created_at, updated_at) VALUES
-- DNA Extraction Templates
('550e8400-e29b-41d4-a716-446655440901', 
 'DNA Extraction Protocol - Qiagen DNeasy', 
 'Standard protocol for DNA extraction using Qiagen DNeasy Blood & Tissue Kit', 
 '/templates/extraction/dna_qiagen_dneasy_v2.docx', 
 'protocol',
 '{"version": "2.1", "kit": "Qiagen DNeasy", "sample_types": ["blood", "tissue"], "is_active": true}'::jsonb,
 NOW() - INTERVAL '90 days',
 NOW() - INTERVAL '10 days'),

('550e8400-e29b-41d4-a716-446655440902', 
 'RNA Extraction Protocol - TRIzol Method', 
 'RNA extraction protocol using TRIzol reagent for various sample types', 
 '/templates/extraction/rna_trizol_method_v3.docx', 
 'protocol',
 '{"version": "3.0", "reagent": "TRIzol", "sample_types": ["cells", "tissue"], "is_active": true}'::jsonb,
 NOW() - INTERVAL '60 days',
 NOW() - INTERVAL '5 days'),

-- Library Preparation Templates
('550e8400-e29b-41d4-a716-446655440903', 
 'Illumina TruSeq DNA Library Prep', 
 'Library preparation protocol for Illumina TruSeq DNA samples', 
 '/templates/library_prep/illumina_truseq_dna_v4.docx', 
 'protocol',
 '{"version": "4.2", "platform": "Illumina", "kit": "TruSeq DNA", "is_active": true}'::jsonb,
 NOW() - INTERVAL '45 days',
 NOW() - INTERVAL '2 days'),

('550e8400-e29b-41d4-a716-446655440904', 
 'RNA-Seq Library Preparation', 
 'Comprehensive protocol for RNA-Seq library preparation including rRNA depletion', 
 '/templates/library_prep/rnaseq_library_prep_v2.docx', 
 'protocol',
 '{"version": "2.5", "includes_rrna_depletion": true, "is_active": true}'::jsonb,
 NOW() - INTERVAL '30 days',
 NOW() - INTERVAL '1 day'),

-- Quality Control Templates
('550e8400-e29b-41d4-a716-446655440905', 
 'NGS Quality Control Checklist', 
 'Comprehensive checklist for next-generation sequencing quality control', 
 '/templates/qc/ngs_qc_checklist_v5.xlsx', 
 'checklist',
 '{"version": "5.0", "platform": "universal", "is_active": true}'::jsonb,
 NOW() - INTERVAL '120 days',
 NOW() - INTERVAL '15 days'),

('550e8400-e29b-41d4-a716-446655440906', 
 'Sample Quality Assessment Form', 
 'Standard form for documenting sample quality metrics', 
 '/templates/qc/sample_quality_form_v3.pdf', 
 'form',
 '{"version": "3.1", "required_fields": ["concentration", "purity", "integrity"], "is_active": true}'::jsonb,
 NOW() - INTERVAL '75 days',
 NOW() - INTERVAL '7 days'),

-- Sequencing Templates
('550e8400-e29b-41d4-a716-446655440907', 
 'Whole Genome Sequencing Workflow', 
 'Complete workflow template for human whole genome sequencing', 
 '/templates/sequencing/wgs_workflow_human_v6.docx', 
 'workflow',
 '{"version": "6.0", "organism": "human", "coverage": "30X", "is_active": true}'::jsonb,
 NOW() - INTERVAL '50 days',
 NOW() - INTERVAL '3 days'),

('550e8400-e29b-41d4-a716-446655440908', 
 'Targeted Panel Sequencing Protocol', 
 'Protocol for targeted sequencing using custom gene panels', 
 '/templates/sequencing/targeted_panel_protocol_v2.docx', 
 'protocol',
 '{"version": "2.3", "panel_size": "500 genes", "is_active": true}'::jsonb,
 NOW() - INTERVAL '40 days',
 NOW() - INTERVAL '4 days'),

-- Report Templates
('550e8400-e29b-41d4-a716-446655440909', 
 'Sequencing Results Report Template', 
 'Standard template for reporting sequencing results to clients', 
 '/templates/reports/sequencing_results_template_v4.docx', 
 'report',
 '{"version": "4.1", "sections": ["summary", "methods", "results", "quality"], "is_active": true}'::jsonb,
 NOW() - INTERVAL '100 days',
 NOW() - INTERVAL '20 days'),

('550e8400-e29b-41d4-a716-446655440910', 
 'Monthly Lab Activity Report', 
 'Template for generating monthly laboratory activity summaries', 
 '/templates/reports/monthly_activity_template_v2.xlsx', 
 'report',
 '{"version": "2.0", "automated": true, "frequency": "monthly", "is_active": true}'::jsonb,
 NOW() - INTERVAL '80 days',
 NOW() - INTERVAL '25 days'),

-- Sample Submission Templates
('550e8400-e29b-41d4-a716-446655440911', 
 'Sample Submission Form - Clinical', 
 'Clinical sample submission form with patient information fields', 
 '/templates/submission/clinical_submission_form_v7.pdf', 
 'form',
 '{"version": "7.2", "type": "clinical", "hipaa_compliant": true, "is_active": true}'::jsonb,
 NOW() - INTERVAL '110 days',
 NOW() - INTERVAL '12 days'),

('550e8400-e29b-41d4-a716-446655440912', 
 'Research Sample Submission Guide', 
 'Comprehensive guide for research sample submissions including metadata requirements', 
 '/templates/submission/research_submission_guide_v5.pdf', 
 'guide',
 '{"version": "5.5", "type": "research", "includes_metadata_schema": true, "is_active": true}'::jsonb,
 NOW() - INTERVAL '95 days',
 NOW() - INTERVAL '8 days');

-- Update timestamps to ensure consistency
UPDATE templates SET updated_at = created_at WHERE updated_at < created_at; 