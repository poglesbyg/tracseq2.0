-- Template Service - Add Missing Columns
-- File: template_service/migrations/003_add_missing_columns.sql

-- Create template_status enum if it doesn't exist
DO $$ BEGIN
    CREATE TYPE template_status AS ENUM ('Draft', 'Published', 'Archived', 'Deprecated');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Add missing columns to templates table
ALTER TABLE templates 
    ADD COLUMN IF NOT EXISTS category VARCHAR(100),
    ADD COLUMN IF NOT EXISTS status template_status DEFAULT 'Draft',
    ADD COLUMN IF NOT EXISTS version VARCHAR(20) DEFAULT '1.0.0',
    ADD COLUMN IF NOT EXISTS created_by VARCHAR(255) DEFAULT 'system',
    ADD COLUMN IF NOT EXISTS createdDate TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS updatedDate TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS isActive BOOLEAN DEFAULT true;

-- Update timestamps to match what frontend expects
UPDATE templates SET createdDate = created_at WHERE createdDate IS NULL;
UPDATE templates SET updatedDate = updated_at WHERE updatedDate IS NULL;

-- Create indexes for new columns
CREATE INDEX IF NOT EXISTS idx_templates_category ON templates(category);
CREATE INDEX IF NOT EXISTS idx_templates_status ON templates(status);

COMMENT ON COLUMN templates.category IS 'Template category for grouping';
COMMENT ON COLUMN templates.status IS 'Template lifecycle status';
COMMENT ON COLUMN templates.version IS 'Template version string'; 