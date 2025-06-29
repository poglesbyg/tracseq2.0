-- Template Service - Initial Schema Migration
-- File: template_service/migrations/001_initial_template_schema.sql

-- Create custom types
CREATE TYPE template_status AS ENUM ('draft', 'active', 'deprecated', 'archived');
CREATE TYPE field_type AS ENUM ('text', 'number', 'date', 'boolean', 'select', 'multiselect', 'file');

-- Templates table
CREATE TABLE templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100) NOT NULL, -- 'submission', 'report', 'qc', 'protocol'
    status template_status DEFAULT 'draft',
    version INTEGER DEFAULT 1,
    template_data JSONB NOT NULL,
    field_definitions JSONB DEFAULT '[]',
    validation_rules JSONB DEFAULT '{}',
    metadata JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT templates_version_check CHECK (version > 0)
);

-- Template Fields table
CREATE TABLE template_fields (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL,
    field_name VARCHAR(255) NOT NULL,
    field_type field_type NOT NULL,
    field_label VARCHAR(255) NOT NULL,
    field_description TEXT,
    is_required BOOLEAN DEFAULT false,
    field_order INTEGER DEFAULT 0,
    validation_rules JSONB DEFAULT '{}',
    default_value TEXT,
    field_options JSONB DEFAULT '[]', -- For select/multiselect
    field_metadata JSONB DEFAULT '{}',
    
    CONSTRAINT template_fields_template_fkey FOREIGN KEY (template_id) REFERENCES templates(id) ON DELETE CASCADE,
    UNIQUE(template_id, field_name)
);

-- Template Instances table (filled templates)
CREATE TABLE template_instances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL,
    instance_name VARCHAR(255),
    form_data JSONB NOT NULL,
    status VARCHAR(50) DEFAULT 'draft',
    submitted_by UUID,
    submitted_at TIMESTAMPTZ,
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT template_instances_template_fkey FOREIGN KEY (template_id) REFERENCES templates(id)
);

-- Create indexes
CREATE INDEX idx_templates_category ON templates(category);
CREATE INDEX idx_templates_status ON templates(status);
CREATE INDEX idx_template_fields_template_id ON template_fields(template_id);
CREATE INDEX idx_template_instances_template_id ON template_instances(template_id);

-- Insert sample templates
INSERT INTO templates (name, category, template_data, field_definitions) VALUES
    ('Sample Submission Form', 'submission', 
     '{"title": "Laboratory Sample Submission", "sections": ["sample_info", "requester", "analysis"]}',
     '[{"name": "sample_id", "type": "text", "required": true}, {"name": "sample_type", "type": "select", "options": ["DNA", "RNA", "Protein"]}]');

COMMENT ON TABLE templates IS 'Document templates and their definitions';
COMMENT ON TABLE template_fields IS 'Field definitions for templates';
COMMENT ON TABLE template_instances IS 'Filled template instances'; 