-- Library Details Service - Initial Schema Migration
-- File: library_details_service/migrations/001_initial_library_schema.sql

-- Create custom types
CREATE TYPE library_type AS ENUM ('dna', 'rna', 'cdna', 'amplicon', 'bisulfite', 'chip_seq');
CREATE TYPE prep_status AS ENUM ('pending', 'in_progress', 'completed', 'failed', 'qc_failed');

-- Library Preparation Protocols table
CREATE TABLE library_protocols (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    protocol_name VARCHAR(255) NOT NULL UNIQUE,
    library_type library_type NOT NULL,
    version VARCHAR(50) NOT NULL,
    description TEXT,
    protocol_steps JSONB NOT NULL,
    required_equipment JSONB DEFAULT '[]',
    reagents JSONB DEFAULT '[]',
    quality_checkpoints JSONB DEFAULT '[]',
    expected_yield_range NUMRANGE,
    protocol_metadata JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Libraries table
CREATE TABLE libraries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    library_id VARCHAR(255) NOT NULL UNIQUE,
    sample_id UUID NOT NULL, -- Reference to samples table
    library_name VARCHAR(255),
    library_type library_type NOT NULL,
    protocol_id UUID,
    prep_status prep_status DEFAULT 'pending',
    concentration DECIMAL(8,3), -- ng/µL
    volume DECIMAL(8,3), -- µL
    fragment_size_bp INTEGER,
    fragment_size_range NUMRANGE,
    molarity DECIMAL(10,3), -- nM
    rqs_score DECIMAL(3,1), -- RNA Quality Score
    din_score DECIMAL(3,1), -- DNA Integrity Number
    a260_280_ratio DECIMAL(4,2),
    a260_230_ratio DECIMAL(4,2),
    prep_date TIMESTAMPTZ,
    prepared_by UUID,
    qc_metrics JSONB DEFAULT '{}',
    notes TEXT,
    storage_location VARCHAR(255),
    barcode VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT libraries_protocol_fkey FOREIGN KEY (protocol_id) REFERENCES library_protocols(id),
    CONSTRAINT libraries_concentration_check CHECK (concentration IS NULL OR concentration >= 0),
    CONSTRAINT libraries_volume_check CHECK (volume IS NULL OR volume >= 0)
);

-- Library Quality Metrics table
CREATE TABLE library_qc_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    library_id UUID NOT NULL,
    metric_type VARCHAR(100) NOT NULL, -- 'bioanalyzer', 'tapestation', 'qubit', 'qpcr'
    metric_value DECIMAL(10,4),
    metric_unit VARCHAR(50),
    measurement_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    instrument VARCHAR(100),
    operator_id UUID,
    pass_criteria BOOLEAN,
    qc_data JSONB DEFAULT '{}',
    
    CONSTRAINT library_qc_library_fkey FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE
);

-- Create indexes
CREATE INDEX idx_libraries_sample_id ON libraries(sample_id);
CREATE INDEX idx_libraries_type ON libraries(library_type);
CREATE INDEX idx_libraries_status ON libraries(prep_status);
CREATE INDEX idx_library_qc_library_id ON library_qc_metrics(library_id);

-- Insert sample protocols
INSERT INTO library_protocols (protocol_name, library_type, version, protocol_steps) VALUES
    ('TruSeq DNA PCR-Free', 'dna', 'v1.0', '["fragmentation", "end_repair", "adapter_ligation", "purification"]'),
    ('TruSeq Stranded mRNA', 'rna', 'v2.0', '["mrna_purification", "fragmentation", "cdna_synthesis", "adapter_ligation"]');

COMMENT ON TABLE library_protocols IS 'Library preparation protocol definitions';
COMMENT ON TABLE libraries IS 'Library preparation records and metadata';
COMMENT ON TABLE library_qc_metrics IS 'Quality control metrics for prepared libraries'; 