-- Fix spreadsheet schema to match the expected model
-- Migration: 20240320000010_fix_spreadsheet_schema.sql

-- Add missing columns to spreadsheet_datasets table
ALTER TABLE spreadsheet_datasets 
    ADD COLUMN IF NOT EXISTS original_filename VARCHAR(255),
    ADD COLUMN IF NOT EXISTS sheet_name VARCHAR(255),
    ADD COLUMN IF NOT EXISTS column_headers JSONB DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS error_message TEXT;

-- Create the spreadsheet_records table that the model expects
CREATE TABLE IF NOT EXISTS spreadsheet_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id UUID NOT NULL REFERENCES spreadsheet_datasets(id) ON DELETE CASCADE,
    row_number INTEGER NOT NULL,
    row_data JSONB NOT NULL,
    search_text TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(dataset_id, row_number)
);

-- Create indexes for spreadsheet_records
CREATE INDEX IF NOT EXISTS idx_spreadsheet_records_dataset_id ON spreadsheet_records(dataset_id);
CREATE INDEX IF NOT EXISTS idx_spreadsheet_records_row_number ON spreadsheet_records(row_number);
CREATE INDEX IF NOT EXISTS idx_spreadsheet_records_search_text ON spreadsheet_records USING GIN(to_tsvector('english', search_text));

-- Add GIN index for row_data JSONB searches
CREATE INDEX IF NOT EXISTS idx_spreadsheet_records_row_data_gin ON spreadsheet_records USING GIN(row_data);

-- Update column constraints and defaults where needed
-- Make sure original_filename has same constraint as filename if needed
UPDATE spreadsheet_datasets SET original_filename = filename WHERE original_filename IS NULL;
UPDATE spreadsheet_datasets SET sheet_name = 'Sheet1' WHERE sheet_name IS NULL;
UPDATE spreadsheet_datasets SET column_headers = '[]'::jsonb WHERE column_headers IS NULL; 
