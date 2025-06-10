-- Migration for spreadsheet processing service
-- Creates tables for storing uploaded spreadsheet data and enabling search

-- Table to store information about uploaded spreadsheets/datasets
CREATE TABLE spreadsheet_datasets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    filename VARCHAR(255) NOT NULL,
    original_filename VARCHAR(255) NOT NULL,
    file_type VARCHAR(10) NOT NULL CHECK (file_type IN ('xlsx', 'csv')),
    file_size BIGINT NOT NULL,
    sheet_name VARCHAR(255), -- For Excel files with multiple sheets
    total_rows INTEGER NOT NULL DEFAULT 0,
    total_columns INTEGER NOT NULL DEFAULT 0,
    column_headers TEXT[] NOT NULL DEFAULT '{}',
    upload_status VARCHAR(20) NOT NULL DEFAULT 'processing' CHECK (upload_status IN ('processing', 'completed', 'failed')),
    error_message TEXT,
    uploaded_by VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- Table to store individual rows/records from spreadsheets
CREATE TABLE spreadsheet_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    dataset_id UUID NOT NULL REFERENCES spreadsheet_datasets(id) ON DELETE CASCADE,
    row_number INTEGER NOT NULL,
    row_data JSONB NOT NULL, -- Stores the actual row data as key-value pairs
    search_text TEXT, -- Concatenated searchable text from all fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_spreadsheet_datasets_filename ON spreadsheet_datasets(filename);
CREATE INDEX idx_spreadsheet_datasets_file_type ON spreadsheet_datasets(file_type);
CREATE INDEX idx_spreadsheet_datasets_upload_status ON spreadsheet_datasets(upload_status);
CREATE INDEX idx_spreadsheet_datasets_created_at ON spreadsheet_datasets(created_at DESC);

CREATE INDEX idx_spreadsheet_records_dataset_id ON spreadsheet_records(dataset_id);
CREATE INDEX idx_spreadsheet_records_row_number ON spreadsheet_records(dataset_id, row_number);

-- Full-text search index for searchable content
CREATE INDEX idx_spreadsheet_records_search_text ON spreadsheet_records USING gin(to_tsvector('english', search_text));

-- GIN index on row_data for JSON queries
CREATE INDEX idx_spreadsheet_records_row_data ON spreadsheet_records USING gin(row_data); 
