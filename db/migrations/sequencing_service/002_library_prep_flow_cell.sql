-- Migration: Add Library Preparation and Flow Cell Design tables
-- Version: 002
-- Description: Adds support for library prep workflow and flow cell design features

-- Library preparation protocols table
CREATE TABLE library_prep_protocols (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    version VARCHAR(50) NOT NULL,
    protocol_type VARCHAR(100) NOT NULL, -- 'DNA', 'RNA', 'ChIP', 'ATAC', etc.
    kit_name VARCHAR(255),
    kit_manufacturer VARCHAR(255),
    input_requirements JSONB NOT NULL, -- min/max concentration, volume, quality metrics
    protocol_steps JSONB NOT NULL, -- detailed steps with timing
    reagents JSONB NOT NULL, -- required reagents and volumes
    equipment_required TEXT[],
    estimated_duration_hours DECIMAL(5,2),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by UUID,
    CONSTRAINT library_prep_protocols_unique_name_version UNIQUE(name, version)
);

-- Library preparations table
CREATE TABLE library_preparations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    batch_id VARCHAR(255) NOT NULL UNIQUE,
    project_id UUID NOT NULL,
    protocol_id UUID NOT NULL,
    sample_ids UUID[] NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- 'pending', 'in_progress', 'completed', 'failed', 'qc_review'
    prep_date DATE NOT NULL,
    operator_id UUID NOT NULL,
    input_metrics JSONB, -- concentration, volume, quality scores
    output_metrics JSONB, -- final library concentration, size distribution
    reagent_lots JSONB, -- lot numbers for reagents used
    notes TEXT,
    qc_status VARCHAR(50), -- 'pending', 'passed', 'failed', 'conditional'
    qc_metrics JSONB,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT library_preparations_protocol_fkey FOREIGN KEY (protocol_id) REFERENCES library_prep_protocols(id),
    CONSTRAINT library_preparations_project_fkey FOREIGN KEY (project_id) REFERENCES projects(id)
);

-- Flow cell types table
CREATE TABLE flow_cell_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    manufacturer VARCHAR(255) NOT NULL,
    model VARCHAR(255) NOT NULL,
    lane_count INTEGER NOT NULL,
    reads_per_lane BIGINT,
    chemistry_version VARCHAR(100),
    compatible_sequencers TEXT[],
    specifications JSONB,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Flow cell designs table
CREATE TABLE flow_cell_designs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    flow_cell_type_id UUID NOT NULL,
    project_id UUID NOT NULL,
    sequencing_run_id UUID,
    design_version INTEGER NOT NULL DEFAULT 1,
    status VARCHAR(50) NOT NULL DEFAULT 'draft', -- 'draft', 'approved', 'in_sequencing', 'completed'
    lane_assignments JSONB NOT NULL, -- array of lane configurations
    pooling_strategy JSONB, -- how samples are pooled
    expected_coverage JSONB, -- expected coverage per sample
    ai_optimization_score DECIMAL(5,2), -- AI-suggested optimization score
    ai_suggestions JSONB, -- AI recommendations for improvement
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by UUID NOT NULL,
    CONSTRAINT flow_cell_designs_type_fkey FOREIGN KEY (flow_cell_type_id) REFERENCES flow_cell_types(id),
    CONSTRAINT flow_cell_designs_project_fkey FOREIGN KEY (project_id) REFERENCES projects(id),
    CONSTRAINT flow_cell_designs_run_fkey FOREIGN KEY (sequencing_run_id) REFERENCES sequencing_runs(id)
);

-- Flow cell lane assignments table (for detailed lane tracking)
CREATE TABLE flow_cell_lanes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    flow_cell_design_id UUID NOT NULL,
    lane_number INTEGER NOT NULL,
    library_prep_ids UUID[] NOT NULL,
    sample_sheet_data JSONB, -- demultiplexing information
    target_reads BIGINT,
    index_type VARCHAR(100), -- 'single', 'dual', 'udi'
    index_sequences JSONB,
    loading_concentration_pm DECIMAL(10,2),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT flow_cell_lanes_design_fkey FOREIGN KEY (flow_cell_design_id) REFERENCES flow_cell_designs(id),
    CONSTRAINT flow_cell_lanes_unique_lane UNIQUE(flow_cell_design_id, lane_number)
);

-- Indexes for performance
CREATE INDEX idx_library_prep_protocols_active ON library_prep_protocols(is_active);
CREATE INDEX idx_library_prep_protocols_type ON library_prep_protocols(protocol_type);
CREATE INDEX idx_library_preparations_batch_id ON library_preparations(batch_id);
CREATE INDEX idx_library_preparations_project_id ON library_preparations(project_id);
CREATE INDEX idx_library_preparations_status ON library_preparations(status);
CREATE INDEX idx_library_preparations_prep_date ON library_preparations(prep_date);
CREATE INDEX idx_flow_cell_types_active ON flow_cell_types(is_active);
CREATE INDEX idx_flow_cell_designs_project_id ON flow_cell_designs(project_id);
CREATE INDEX idx_flow_cell_designs_status ON flow_cell_designs(status);
CREATE INDEX idx_flow_cell_lanes_design_id ON flow_cell_lanes(flow_cell_design_id);

-- Add GIN indexes for JSONB columns
CREATE INDEX idx_library_preparations_sample_ids_gin ON library_preparations USING gin(sample_ids);
CREATE INDEX idx_flow_cell_designs_lane_assignments_gin ON flow_cell_designs USING gin(lane_assignments);

-- Triggers for updated_at
CREATE TRIGGER trigger_library_prep_protocols_updated_at
    BEFORE UPDATE ON library_prep_protocols
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trigger_library_preparations_updated_at
    BEFORE UPDATE ON library_preparations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trigger_flow_cell_types_updated_at
    BEFORE UPDATE ON flow_cell_types
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trigger_flow_cell_designs_updated_at
    BEFORE UPDATE ON flow_cell_designs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trigger_flow_cell_lanes_updated_at
    BEFORE UPDATE ON flow_cell_lanes
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- Insert default flow cell types
INSERT INTO flow_cell_types (name, manufacturer, model, lane_count, reads_per_lane, chemistry_version, compatible_sequencers) VALUES
('S4', 'Illumina', 'NovaSeq 6000 S4', 8, 2500000000, 'v1.5', ARRAY['NovaSeq 6000']),
('S2', 'Illumina', 'NovaSeq 6000 S2', 2, 1650000000, 'v1.5', ARRAY['NovaSeq 6000']),
('S1', 'Illumina', 'NovaSeq 6000 S1', 2, 650000000, 'v1.5', ARRAY['NovaSeq 6000']),
('SP', 'Illumina', 'NovaSeq 6000 SP', 2, 325000000, 'v1.5', ARRAY['NovaSeq 6000']),
('NextSeq 2000 P3', 'Illumina', 'NextSeq 2000 P3', 1, 400000000, 'v3', ARRAY['NextSeq 2000', 'NextSeq 1000']),
('NextSeq 2000 P2', 'Illumina', 'NextSeq 2000 P2', 1, 100000000, 'v3', ARRAY['NextSeq 2000', 'NextSeq 1000']),
('MiSeq v3', 'Illumina', 'MiSeq v3', 1, 25000000, 'v3', ARRAY['MiSeq']);

-- Comments
COMMENT ON TABLE library_prep_protocols IS 'Library preparation protocol definitions';
COMMENT ON TABLE library_preparations IS 'Library preparation batch records';
COMMENT ON TABLE flow_cell_types IS 'Flow cell specifications and configurations';
COMMENT ON TABLE flow_cell_designs IS 'Flow cell design and lane assignment plans';
COMMENT ON TABLE flow_cell_lanes IS 'Detailed lane assignments for flow cells';