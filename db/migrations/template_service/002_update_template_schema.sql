-- Template Service - Update Schema to Match Rust Model
-- File: template_service/migrations/002_update_template_schema.sql

-- Rename template_status enum values to match Rust model
ALTER TYPE template_status RENAME VALUE 'draft' TO 'Draft';
ALTER TYPE template_status RENAME VALUE 'active' TO 'Published';
ALTER TYPE template_status ADD VALUE 'Archived';
ALTER TYPE template_status ADD VALUE 'Deprecated';

-- Add missing columns to templates table
ALTER TABLE templates 
    ADD COLUMN IF NOT EXISTS template_type VARCHAR(100) NOT NULL DEFAULT 'form',
    ADD COLUMN IF NOT EXISTS tags TEXT[] DEFAULT '{}',
    ADD COLUMN IF NOT EXISTS is_public BOOLEAN DEFAULT false,
    ADD COLUMN IF NOT EXISTS is_system BOOLEAN DEFAULT false,
    ADD COLUMN IF NOT EXISTS form_config JSONB DEFAULT '{}',
    ADD COLUMN IF NOT EXISTS updated_by VARCHAR(255),
    ADD COLUMN IF NOT EXISTS published_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS published_by VARCHAR(255);

-- Rename existing columns to match Rust model
ALTER TABLE templates 
    RENAME COLUMN template_data TO form_config;

-- Drop columns that don't exist in the Rust model
ALTER TABLE templates 
    DROP COLUMN IF EXISTS field_definitions,
    DROP COLUMN IF EXISTS validation_rules;

-- Update created_by to be VARCHAR instead of UUID to match Rust model
ALTER TABLE templates 
    ALTER COLUMN created_by TYPE VARCHAR(255);

-- Add validation_rule_type enum
CREATE TYPE validation_rule_type AS ENUM (
    'Required', 'MinLength', 'MaxLength', 'Pattern', 
    'MinValue', 'MaxValue', 'Email', 'Phone', 
    'Url', 'Date', 'Custom', 'CrossField'
);

-- Create indexes for new columns
CREATE INDEX IF NOT EXISTS idx_templates_template_type ON templates(template_type);
CREATE INDEX IF NOT EXISTS idx_templates_is_public ON templates(is_public);
CREATE INDEX IF NOT EXISTS idx_templates_is_system ON templates(is_system);
CREATE INDEX IF NOT EXISTS idx_templates_tags ON templates USING GIN(tags);

-- Update existing data to have proper template_type
UPDATE templates SET template_type = 'submission' WHERE category = 'submission';
UPDATE templates SET template_type = 'report' WHERE category = 'report';
UPDATE templates SET template_type = 'protocol' WHERE category = 'protocol';
UPDATE templates SET template_type = 'form' WHERE template_type = 'form';

-- Set default version value
ALTER TABLE templates ALTER COLUMN version SET DEFAULT '1.0.0';
UPDATE templates SET version = '1.0.0' WHERE version = 1;

-- Change version column to VARCHAR to match Rust model
ALTER TABLE templates ALTER COLUMN version TYPE VARCHAR(20);

COMMENT ON COLUMN templates.template_type IS 'Type of template (form, spreadsheet, document, etc.)';
COMMENT ON COLUMN templates.tags IS 'Array of tags for categorization';
COMMENT ON COLUMN templates.is_public IS 'Whether template is publicly accessible';
COMMENT ON COLUMN templates.is_system IS 'Whether template is a system template';
COMMENT ON COLUMN templates.form_config IS 'JSON configuration for form rendering'; 