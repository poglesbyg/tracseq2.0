-- Create spreadsheet-related tables for Excel/CSV data management
-- Migration: 20240320000009_create_spreadsheet_system.sql

-- Spreadsheet datasets table (main table for storing uploaded spreadsheet metadata)
CREATE TABLE IF NOT EXISTS spreadsheet_datasets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    filename VARCHAR(255) NOT NULL,
    file_path TEXT,
    file_size BIGINT,
    sheet_count INTEGER DEFAULT 1,
    total_rows INTEGER DEFAULT 0,
    total_columns INTEGER DEFAULT 0,
    file_type VARCHAR(50) NOT NULL, -- 'xlsx', 'xls', 'csv', etc.
    upload_status VARCHAR(50) DEFAULT 'pending', -- 'pending', 'processing', 'completed', 'failed'
    processing_errors TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    tags TEXT[],
    description TEXT,
    uploaded_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Spreadsheet sheets table (for multi-sheet Excel files)
CREATE TABLE IF NOT EXISTS spreadsheet_sheets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id UUID NOT NULL REFERENCES spreadsheet_datasets(id) ON DELETE CASCADE,
    sheet_name VARCHAR(255) NOT NULL,
    sheet_index INTEGER NOT NULL,
    row_count INTEGER DEFAULT 0,
    column_count INTEGER DEFAULT 0,
    headers JSONB, -- Array of column headers
    data_preview JSONB, -- First few rows for preview
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(dataset_id, sheet_name),
    UNIQUE(dataset_id, sheet_index)
);

-- Spreadsheet data table (stores actual cell data for querying)
CREATE TABLE IF NOT EXISTS spreadsheet_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sheet_id UUID NOT NULL REFERENCES spreadsheet_sheets(id) ON DELETE CASCADE,
    row_index INTEGER NOT NULL,
    column_index INTEGER NOT NULL,
    column_name VARCHAR(255),
    cell_value TEXT,
    data_type VARCHAR(50), -- 'text', 'number', 'date', 'boolean', 'formula'
    formatted_value TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(sheet_id, row_index, column_index)
);

-- Spreadsheet analysis results (for data analysis and insights)
CREATE TABLE IF NOT EXISTS spreadsheet_analysis (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id UUID NOT NULL REFERENCES spreadsheet_datasets(id) ON DELETE CASCADE,
    sheet_id UUID REFERENCES spreadsheet_sheets(id) ON DELETE CASCADE,
    analysis_type VARCHAR(100) NOT NULL, -- 'column_stats', 'data_quality', 'patterns', etc.
    column_name VARCHAR(255),
    results JSONB NOT NULL,
    confidence_score DECIMAL(5,3),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_spreadsheet_datasets_uploaded_by ON spreadsheet_datasets(uploaded_by);
CREATE INDEX IF NOT EXISTS idx_spreadsheet_datasets_created_at ON spreadsheet_datasets(created_at);
CREATE INDEX IF NOT EXISTS idx_spreadsheet_datasets_upload_status ON spreadsheet_datasets(upload_status);
CREATE INDEX IF NOT EXISTS idx_spreadsheet_datasets_file_type ON spreadsheet_datasets(file_type);

CREATE INDEX IF NOT EXISTS idx_spreadsheet_sheets_dataset_id ON spreadsheet_sheets(dataset_id);
CREATE INDEX IF NOT EXISTS idx_spreadsheet_sheets_sheet_name ON spreadsheet_sheets(sheet_name);

CREATE INDEX IF NOT EXISTS idx_spreadsheet_data_sheet_id ON spreadsheet_data(sheet_id);
CREATE INDEX IF NOT EXISTS idx_spreadsheet_data_row_column ON spreadsheet_data(row_index, column_index);
CREATE INDEX IF NOT EXISTS idx_spreadsheet_data_column_name ON spreadsheet_data(column_name);
CREATE INDEX IF NOT EXISTS idx_spreadsheet_data_data_type ON spreadsheet_data(data_type);

CREATE INDEX IF NOT EXISTS idx_spreadsheet_analysis_dataset_id ON spreadsheet_analysis(dataset_id);
CREATE INDEX IF NOT EXISTS idx_spreadsheet_analysis_sheet_id ON spreadsheet_analysis(sheet_id);
CREATE INDEX IF NOT EXISTS idx_spreadsheet_analysis_type ON spreadsheet_analysis(analysis_type);

-- Add GIN indexes for JSONB columns for fast queries
CREATE INDEX IF NOT EXISTS idx_spreadsheet_datasets_metadata_gin ON spreadsheet_datasets USING GIN(metadata);
CREATE INDEX IF NOT EXISTS idx_spreadsheet_sheets_headers_gin ON spreadsheet_sheets USING GIN(headers);
CREATE INDEX IF NOT EXISTS idx_spreadsheet_sheets_data_preview_gin ON spreadsheet_sheets USING GIN(data_preview);
CREATE INDEX IF NOT EXISTS idx_spreadsheet_analysis_results_gin ON spreadsheet_analysis USING GIN(results);

-- Add text search indexes for spreadsheet content
CREATE INDEX IF NOT EXISTS idx_spreadsheet_data_cell_value_trgm ON spreadsheet_data USING GIN(cell_value gin_trgm_ops);

-- Create trigger to update updated_at timestamps
CREATE OR REPLACE FUNCTION update_spreadsheet_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_spreadsheet_datasets_updated_at
    BEFORE UPDATE ON spreadsheet_datasets
    FOR EACH ROW EXECUTE FUNCTION update_spreadsheet_updated_at();

CREATE TRIGGER trigger_spreadsheet_sheets_updated_at
    BEFORE UPDATE ON spreadsheet_sheets
    FOR EACH ROW EXECUTE FUNCTION update_spreadsheet_updated_at();

CREATE TRIGGER trigger_spreadsheet_analysis_updated_at
    BEFORE UPDATE ON spreadsheet_analysis
    FOR EACH ROW EXECUTE FUNCTION update_spreadsheet_updated_at(); 
