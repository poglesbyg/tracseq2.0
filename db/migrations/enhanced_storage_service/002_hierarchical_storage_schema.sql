-- Enhanced Storage Service - Hierarchical Storage Schema Migration
-- File: enhanced_storage_service/migrations/002_hierarchical_storage_schema.sql

-- Create custom types for hierarchical storage
CREATE TYPE storage_container_type AS ENUM ('freezer', 'rack', 'box', 'position');
CREATE TYPE position_status AS ENUM ('available', 'occupied', 'reserved', 'maintenance');

-- Hierarchical Storage Containers table
CREATE TABLE storage_containers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    container_type storage_container_type NOT NULL,
    parent_container_id UUID,
    location_id UUID, -- Reference to storage_locations for top-level containers
    
    -- Physical properties
    grid_position JSONB, -- {row: 1, column: 1} for positioning within parent
    dimensions JSONB, -- {width: 10, height: 10, depth: 5} in cm
    capacity INTEGER DEFAULT 1, -- Number of child containers or samples it can hold
    occupied_count INTEGER DEFAULT 0,
    
    -- Container-specific metadata
    temperature_zone VARCHAR(50),
    barcode VARCHAR(255) UNIQUE,
    description TEXT,
    
    -- Status and tracking
    status VARCHAR(50) DEFAULT 'active',
    installation_date TIMESTAMPTZ DEFAULT NOW(),
    last_maintenance_date TIMESTAMPTZ,
    next_maintenance_date TIMESTAMPTZ,
    
    -- Metadata and configuration
    container_metadata JSONB DEFAULT '{}',
    access_restrictions JSONB DEFAULT '{}', -- {required_clearance: 'level_2', restricted_hours: []}
    
    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    
    -- Constraints
    CONSTRAINT storage_containers_capacity_check CHECK (capacity >= 0),
    CONSTRAINT storage_containers_occupied_check CHECK (occupied_count >= 0 AND occupied_count <= capacity),
    CONSTRAINT storage_containers_parent_fkey FOREIGN KEY (parent_container_id) REFERENCES storage_containers(id) ON DELETE CASCADE,
    CONSTRAINT storage_containers_location_fkey FOREIGN KEY (location_id) REFERENCES storage_locations(id) ON DELETE SET NULL
);

-- Sample Positions table (replaces simple storage assignments)
CREATE TABLE sample_positions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sample_id UUID NOT NULL, -- Reference to samples table
    container_id UUID NOT NULL, -- Reference to storage_containers (should be type 'position')
    position_identifier VARCHAR(100), -- e.g., "A1", "B2", custom identifier within container
    
    -- Assignment details
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assigned_by UUID, -- Reference to users table
    removed_at TIMESTAMPTZ,
    removed_by UUID,
    
    -- Position status
    status position_status DEFAULT 'occupied',
    reservation_expires_at TIMESTAMPTZ,
    
    -- Sample-specific storage information
    storage_conditions JSONB DEFAULT '{}', -- {temperature: -80, humidity: 45}
    special_requirements JSONB DEFAULT '{}', -- {hazardous: false, light_sensitive: true}
    
    -- Chain of custody
    chain_of_custody JSONB DEFAULT '[]',
    
    -- Metadata
    position_metadata JSONB DEFAULT '{}',
    notes TEXT,
    
    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT sample_positions_sample_unique UNIQUE (sample_id), -- One sample per position
    CONSTRAINT sample_positions_container_fkey FOREIGN KEY (container_id) REFERENCES storage_containers(id) ON DELETE CASCADE,
    CONSTRAINT sample_positions_status_check CHECK (
        (status = 'occupied' AND sample_id IS NOT NULL) OR 
        (status != 'occupied' AND sample_id IS NULL)
    )
);

-- Storage Container Hierarchy View (for easy navigation)
CREATE VIEW storage_hierarchy AS
WITH RECURSIVE hierarchy AS (
    -- Base case: top-level containers (freezers)
    SELECT 
        id,
        name,
        container_type,
        parent_container_id,
        location_id,
        grid_position,
        capacity,
        occupied_count,
        temperature_zone,
        barcode,
        status,
        1 as level,
        ARRAY[name] as path,
        name as full_path
    FROM storage_containers 
    WHERE parent_container_id IS NULL
    
    UNION ALL
    
    -- Recursive case: child containers
    SELECT 
        sc.id,
        sc.name,
        sc.container_type,
        sc.parent_container_id,
        sc.location_id,
        sc.grid_position,
        sc.capacity,
        sc.occupied_count,
        sc.temperature_zone,
        sc.barcode,
        sc.status,
        h.level + 1,
        h.path || sc.name,
        h.full_path || ' > ' || sc.name
    FROM storage_containers sc
    JOIN hierarchy h ON sc.parent_container_id = h.id
)
SELECT * FROM hierarchy;

-- Storage Capacity Summary View
CREATE VIEW storage_capacity_summary AS
SELECT 
    sc.id,
    sc.name,
    sc.container_type,
    sc.capacity,
    sc.occupied_count,
    sc.capacity - sc.occupied_count as available_count,
    CASE 
        WHEN sc.capacity > 0 THEN ROUND((sc.occupied_count::decimal / sc.capacity::decimal) * 100, 2)
        ELSE 0 
    END as utilization_percentage,
    CASE 
        WHEN sc.capacity > 0 THEN
            CASE 
                WHEN (sc.occupied_count::decimal / sc.capacity::decimal) >= 0.95 THEN 'critical'
                WHEN (sc.occupied_count::decimal / sc.capacity::decimal) >= 0.80 THEN 'warning'
                ELSE 'normal'
            END
        ELSE 'normal'
    END as capacity_status,
    sc.temperature_zone,
    sc.status,
    sc.created_at,
    sc.updated_at
FROM storage_containers sc;

-- Sample Location View (combines sample positions with full hierarchy path)
CREATE VIEW sample_locations_detailed AS
SELECT 
    sp.id as position_id,
    sp.sample_id,
    sp.position_identifier,
    sp.assigned_at,
    sp.assigned_by,
    sp.status as position_status,
    sp.storage_conditions,
    sp.special_requirements,
    sp.notes,
    
    -- Container information
    sc.id as container_id,
    sc.name as container_name,
    sc.container_type,
    sc.barcode as container_barcode,
    sc.temperature_zone,
    
    -- Hierarchy information
    sh.full_path,
    sh.level,
    sh.path,
    
    -- Location information
    sl.name as location_name,
    sl.zone_type as location_zone_type
    
FROM sample_positions sp
JOIN storage_containers sc ON sp.container_id = sc.id
JOIN storage_hierarchy sh ON sc.id = sh.id
LEFT JOIN storage_locations sl ON sh.location_id = sl.id
WHERE sp.removed_at IS NULL;

-- Create indexes for performance
CREATE INDEX idx_storage_containers_parent_id ON storage_containers(parent_container_id);
CREATE INDEX idx_storage_containers_location_id ON storage_containers(location_id);
CREATE INDEX idx_storage_containers_type ON storage_containers(container_type);
CREATE INDEX idx_storage_containers_status ON storage_containers(status);
CREATE INDEX idx_storage_containers_barcode ON storage_containers(barcode);
CREATE INDEX idx_storage_containers_temperature_zone ON storage_containers(temperature_zone);
CREATE INDEX idx_storage_containers_grid_position_gin ON storage_containers USING gin(grid_position);

CREATE INDEX idx_sample_positions_sample_id ON sample_positions(sample_id);
CREATE INDEX idx_sample_positions_container_id ON sample_positions(container_id);
CREATE INDEX idx_sample_positions_status ON sample_positions(status);
CREATE INDEX idx_sample_positions_assigned_at ON sample_positions(assigned_at);
CREATE INDEX idx_sample_positions_removed_at ON sample_positions(removed_at);

-- Create trigger functions for capacity updates
CREATE OR REPLACE FUNCTION update_container_capacity()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- Increase parent container occupied count
        UPDATE storage_containers 
        SET occupied_count = occupied_count + 1, updated_at = NOW()
        WHERE id = NEW.container_id;
        
        -- Update all parent containers in the hierarchy
        WITH RECURSIVE parent_hierarchy AS (
            SELECT parent_container_id, 1 as level
            FROM storage_containers 
            WHERE id = NEW.container_id AND parent_container_id IS NOT NULL
            
            UNION ALL
            
            SELECT sc.parent_container_id, ph.level + 1
            FROM storage_containers sc
            JOIN parent_hierarchy ph ON sc.id = ph.parent_container_id
            WHERE sc.parent_container_id IS NOT NULL
        )
        UPDATE storage_containers 
        SET occupied_count = occupied_count + 1, updated_at = NOW()
        WHERE id IN (SELECT parent_container_id FROM parent_hierarchy);
        
        RETURN NEW;
        
    ELSIF TG_OP = 'DELETE' THEN
        -- Decrease parent container occupied count
        UPDATE storage_containers 
        SET occupied_count = occupied_count - 1, updated_at = NOW()
        WHERE id = OLD.container_id;
        
        -- Update all parent containers in the hierarchy
        WITH RECURSIVE parent_hierarchy AS (
            SELECT parent_container_id, 1 as level
            FROM storage_containers 
            WHERE id = OLD.container_id AND parent_container_id IS NOT NULL
            
            UNION ALL
            
            SELECT sc.parent_container_id, ph.level + 1
            FROM storage_containers sc
            JOIN parent_hierarchy ph ON sc.id = ph.parent_container_id
            WHERE sc.parent_container_id IS NOT NULL
        )
        UPDATE storage_containers 
        SET occupied_count = occupied_count - 1, updated_at = NOW()
        WHERE id IN (SELECT parent_container_id FROM parent_hierarchy);
        
        RETURN OLD;
    END IF;
    
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for automatic capacity updates
CREATE TRIGGER trigger_sample_position_capacity_update
    AFTER INSERT OR DELETE ON sample_positions
    FOR EACH ROW
    EXECUTE FUNCTION update_container_capacity();

-- Create trigger for updated_at timestamps
CREATE TRIGGER trigger_storage_containers_updated_at
    BEFORE UPDATE ON storage_containers
    FOR EACH ROW
    EXECUTE FUNCTION update_storage_updated_at();

CREATE TRIGGER trigger_sample_positions_updated_at
    BEFORE UPDATE ON sample_positions
    FOR EACH ROW
    EXECUTE FUNCTION update_storage_updated_at();

-- Insert sample hierarchical storage data
DO $$
DECLARE
    freezer_id UUID;
    rack_id UUID;
    box_id UUID;
    position_id UUID;
    location_id UUID;
    i INTEGER;
    j INTEGER;
    k INTEGER;
BEGIN
    -- Get a location ID for reference
    SELECT id INTO location_id FROM storage_locations WHERE zone_type = 'ultra_low_freezer' LIMIT 1;
    
    -- Create sample freezers
    FOR i IN 1..3 LOOP
        INSERT INTO storage_containers (name, container_type, location_id, capacity, temperature_zone, barcode, description)
        VALUES (
            'Freezer-' || i,
            'freezer',
            location_id,
            20, -- 20 racks per freezer
            '-80C',
            'FRZ-' || LPAD(i::text, 3, '0'),
            'Ultra-low temperature freezer unit ' || i
        ) RETURNING id INTO freezer_id;
        
        -- Create racks in each freezer
        FOR j IN 1..4 LOOP
            INSERT INTO storage_containers (name, container_type, parent_container_id, capacity, temperature_zone, barcode, grid_position, description)
            VALUES (
                'Rack-' || j,
                'rack',
                freezer_id,
                25, -- 25 boxes per rack
                '-80C',
                'FRZ-' || LPAD(i::text, 3, '0') || '-R' || LPAD(j::text, 2, '0'),
                jsonb_build_object('row', (j-1) / 2 + 1, 'column', (j-1) % 2 + 1),
                'Storage rack ' || j || ' in freezer ' || i
            ) RETURNING id INTO rack_id;
            
            -- Create boxes in each rack
            FOR k IN 1..5 LOOP
                INSERT INTO storage_containers (name, container_type, parent_container_id, capacity, temperature_zone, barcode, grid_position, description)
                VALUES (
                    'Box-' || k,
                    'box',
                    rack_id,
                    100, -- 100 positions per box (10x10 grid)
                    '-80C',
                    'FRZ-' || LPAD(i::text, 3, '0') || '-R' || LPAD(j::text, 2, '0') || '-B' || LPAD(k::text, 2, '0'),
                    jsonb_build_object('row', (k-1) / 5 + 1, 'column', (k-1) % 5 + 1),
                    'Storage box ' || k || ' in rack ' || j || ' of freezer ' || i
                ) RETURNING id INTO box_id;
                
                -- Create positions in each box (10x10 grid)
                FOR row IN 1..10 LOOP
                    FOR col IN 1..10 LOOP
                        INSERT INTO storage_containers (name, container_type, parent_container_id, capacity, temperature_zone, barcode, grid_position, description)
                        VALUES (
                            CHR(64 + row) || col::text, -- A1, A2, B1, B2, etc.
                            'position',
                            box_id,
                            1, -- 1 sample per position
                            '-80C',
                            'FRZ-' || LPAD(i::text, 3, '0') || '-R' || LPAD(j::text, 2, '0') || '-B' || LPAD(k::text, 2, '0') || '-' || CHR(64 + row) || col::text,
                            jsonb_build_object('row', row, 'column', col),
                            'Position ' || CHR(64 + row) || col::text || ' in box ' || k
                        );
                    END LOOP;
                END LOOP;
            END LOOP;
        END LOOP;
    END LOOP;
END $$;

-- Create sample data for refrigerated storage
DO $$
DECLARE
    freezer_id UUID;
    rack_id UUID;
    box_id UUID;
    location_id UUID;
    i INTEGER;
    j INTEGER;
    k INTEGER;
BEGIN
    -- Get a refrigerated location ID
    SELECT id INTO location_id FROM storage_locations WHERE zone_type = 'refrigerated' LIMIT 1;
    
    -- Create sample refrigerated units
    FOR i IN 1..2 LOOP
        INSERT INTO storage_containers (name, container_type, location_id, capacity, temperature_zone, barcode, description)
        VALUES (
            'Refrigerator-' || i,
            'freezer',
            location_id,
            10, -- 10 racks per refrigerator
            '4C',
            'REF-' || LPAD(i::text, 3, '0'),
            'Refrigerated storage unit ' || i
        ) RETURNING id INTO freezer_id;
        
        -- Create racks in each refrigerator
        FOR j IN 1..3 LOOP
            INSERT INTO storage_containers (name, container_type, parent_container_id, capacity, temperature_zone, barcode, grid_position, description)
            VALUES (
                'Shelf-' || j,
                'rack',
                freezer_id,
                15, -- 15 boxes per shelf
                '4C',
                'REF-' || LPAD(i::text, 3, '0') || '-S' || LPAD(j::text, 2, '0'),
                jsonb_build_object('row', j, 'column', 1),
                'Storage shelf ' || j || ' in refrigerator ' || i
            ) RETURNING id INTO rack_id;
            
            -- Create boxes on each shelf
            FOR k IN 1..3 LOOP
                INSERT INTO storage_containers (name, container_type, parent_container_id, capacity, temperature_zone, barcode, grid_position, description)
                VALUES (
                    'Tray-' || k,
                    'box',
                    rack_id,
                    24, -- 24 positions per tray (4x6 grid)
                    '4C',
                    'REF-' || LPAD(i::text, 3, '0') || '-S' || LPAD(j::text, 2, '0') || '-T' || LPAD(k::text, 2, '0'),
                    jsonb_build_object('row', 1, 'column', k),
                    'Storage tray ' || k || ' on shelf ' || j
                ) RETURNING id INTO box_id;
                
                -- Create positions in each tray (4x6 grid)
                FOR row IN 1..4 LOOP
                    FOR col IN 1..6 LOOP
                        INSERT INTO storage_containers (name, container_type, parent_container_id, capacity, temperature_zone, barcode, grid_position, description)
                        VALUES (
                            CHR(64 + row) || col::text, -- A1, A2, B1, B2, etc.
                            'position',
                            box_id,
                            1, -- 1 sample per position
                            '4C',
                            'REF-' || LPAD(i::text, 3, '0') || '-S' || LPAD(j::text, 2, '0') || '-T' || LPAD(k::text, 2, '0') || '-' || CHR(64 + row) || col::text,
                            jsonb_build_object('row', row, 'column', col),
                            'Position ' || CHR(64 + row) || col::text || ' in tray ' || k
                        );
                    END LOOP;
                END LOOP;
            END LOOP;
        END LOOP;
    END LOOP;
END $$;

-- Add comments
COMMENT ON TABLE storage_containers IS 'Hierarchical storage containers: freezers, racks, boxes, and positions';
COMMENT ON TABLE sample_positions IS 'Sample assignments to specific storage positions with full audit trail';
COMMENT ON VIEW storage_hierarchy IS 'Recursive view of storage container hierarchy with full paths';
COMMENT ON VIEW storage_capacity_summary IS 'Summary of storage capacity and utilization at all levels';
COMMENT ON VIEW sample_locations_detailed IS 'Detailed view of sample locations with full hierarchy information'; 