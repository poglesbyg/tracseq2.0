-- Complete Storage Schema for Enhanced Storage Service
-- This migration creates both basic storage tables and hierarchical storage

-- Create basic storage enums
CREATE TYPE storage_zone_type AS ENUM (
    'ultra_low_freezer',    -- -80°C
    'freezer',              -- -20°C
    'refrigerated',         -- 4°C
    'room_temperature',     -- 20-25°C
    'incubator'            -- 37°C
);

CREATE TYPE storage_status AS ENUM (
    'active',
    'maintenance',
    'offline',
    'decommissioned'
);

CREATE TYPE container_type AS ENUM (
    'freezer',
    'rack',
    'box',
    'position'
);

CREATE TYPE sample_position_status AS ENUM (
    'empty',
    'occupied',
    'reserved',
    'maintenance'
);

-- Basic storage locations table
CREATE TABLE storage_locations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    zone_type storage_zone_type NOT NULL,
    temperature_celsius DECIMAL(5,2),
    capacity INTEGER NOT NULL DEFAULT 0,
    current_usage INTEGER NOT NULL DEFAULT 0,
    status storage_status NOT NULL DEFAULT 'active',
    location_code VARCHAR(50) UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Hierarchical storage containers
CREATE TABLE storage_containers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    container_type container_type NOT NULL,
    parent_id UUID REFERENCES storage_containers(id) ON DELETE CASCADE,
    storage_location_id UUID REFERENCES storage_locations(id) ON DELETE SET NULL,
    
    -- Position within parent container
    position_x INTEGER,
    position_y INTEGER,
    position_z INTEGER,
    
    -- Capacity and dimensions
    capacity INTEGER NOT NULL DEFAULT 0,
    current_usage INTEGER NOT NULL DEFAULT 0,
    width INTEGER,
    height INTEGER,
    depth INTEGER,
    
    -- Temperature compatibility
    min_temperature_celsius DECIMAL(5,2),
    max_temperature_celsius DECIMAL(5,2),
    
    -- Status and metadata
    status storage_status NOT NULL DEFAULT 'active',
    barcode VARCHAR(255) UNIQUE,
    description TEXT,
    metadata JSONB,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT valid_position CHECK (
        (position_x IS NULL AND position_y IS NULL AND position_z IS NULL) OR
        (position_x IS NOT NULL AND position_y IS NOT NULL)
    ),
    CONSTRAINT valid_capacity CHECK (current_usage <= capacity),
    CONSTRAINT valid_temperature_range CHECK (
        (min_temperature_celsius IS NULL AND max_temperature_celsius IS NULL) OR
        (min_temperature_celsius <= max_temperature_celsius)
    )
);

-- Sample positions within containers
CREATE TABLE sample_positions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    container_id UUID NOT NULL REFERENCES storage_containers(id) ON DELETE CASCADE,
    sample_id UUID, -- References samples table
    
    -- Position coordinates
    position_x INTEGER NOT NULL,
    position_y INTEGER NOT NULL,
    position_z INTEGER DEFAULT 0,
    
    -- Status and metadata
    status sample_position_status NOT NULL DEFAULT 'empty',
    reserved_until TIMESTAMPTZ,
    reserved_by VARCHAR(255),
    
    -- Assignment details
    assigned_at TIMESTAMPTZ,
    assigned_by VARCHAR(255),
    notes TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    UNIQUE(container_id, position_x, position_y, position_z),
    CONSTRAINT valid_reservation CHECK (
        (status != 'reserved') OR (reserved_until IS NOT NULL AND reserved_by IS NOT NULL)
    ),
    CONSTRAINT valid_assignment CHECK (
        (status != 'occupied') OR (sample_id IS NOT NULL AND assigned_at IS NOT NULL)
    )
);

-- Create indexes for performance
CREATE INDEX idx_storage_locations_zone_type ON storage_locations(zone_type);
CREATE INDEX idx_storage_locations_status ON storage_locations(status);
CREATE INDEX idx_storage_containers_parent_id ON storage_containers(parent_id);
CREATE INDEX idx_storage_containers_storage_location_id ON storage_containers(storage_location_id);
CREATE INDEX idx_storage_containers_type ON storage_containers(container_type);
CREATE INDEX idx_storage_containers_status ON storage_containers(status);
CREATE INDEX idx_storage_containers_barcode ON storage_containers(barcode);
CREATE INDEX idx_sample_positions_container_id ON sample_positions(container_id);
CREATE INDEX idx_sample_positions_sample_id ON sample_positions(sample_id);
CREATE INDEX idx_sample_positions_status ON sample_positions(status);
CREATE INDEX idx_sample_positions_coordinates ON sample_positions(container_id, position_x, position_y, position_z);

-- Create hierarchical storage view
CREATE VIEW storage_hierarchy AS
WITH RECURSIVE hierarchy AS (
    -- Base case: root containers (no parent)
    SELECT 
        id,
        name,
        container_type,
        parent_id,
        storage_location_id,
        ARRAY[name] as path,
        0 as level,
        capacity,
        current_usage,
        status
    FROM storage_containers 
    WHERE parent_id IS NULL
    
    UNION ALL
    
    -- Recursive case: child containers
    SELECT 
        sc.id,
        sc.name,
        sc.container_type,
        sc.parent_id,
        sc.storage_location_id,
        h.path || sc.name,
        h.level + 1,
        sc.capacity,
        sc.current_usage,
        sc.status
    FROM storage_containers sc
    JOIN hierarchy h ON sc.parent_id = h.id
)
SELECT * FROM hierarchy;

-- Create storage capacity summary view
CREATE VIEW storage_capacity_summary AS
SELECT 
    sc.id,
    sc.name,
    sc.container_type,
    sc.capacity,
    sc.current_usage,
    ROUND((sc.current_usage::DECIMAL / NULLIF(sc.capacity, 0)) * 100, 2) as utilization_percentage,
    sl.zone_type,
    sl.temperature_celsius
FROM storage_containers sc
LEFT JOIN storage_locations sl ON sc.storage_location_id = sl.id;

-- Create detailed sample location view
CREATE VIEW sample_locations_detailed AS
SELECT 
    sp.id as position_id,
    sp.sample_id,
    sp.position_x,
    sp.position_y,
    sp.position_z,
    sp.status as position_status,
    sc.name as container_name,
    sc.container_type,
    sc.barcode as container_barcode,
    sl.name as location_name,
    sl.zone_type,
    sl.temperature_celsius,
    sp.assigned_at,
    sp.assigned_by,
    sp.notes
FROM sample_positions sp
JOIN storage_containers sc ON sp.container_id = sc.id
LEFT JOIN storage_locations sl ON sc.storage_location_id = sl.id;

-- Function to update container usage counts
CREATE OR REPLACE FUNCTION update_container_usage()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- Increment usage when sample is assigned
        IF NEW.status = 'occupied' THEN
            UPDATE storage_containers 
            SET current_usage = current_usage + 1,
                updated_at = NOW()
            WHERE id = NEW.container_id;
        END IF;
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        -- Handle status changes
        IF OLD.status != NEW.status THEN
            IF OLD.status = 'occupied' AND NEW.status != 'occupied' THEN
                -- Decrement usage when sample is removed
                UPDATE storage_containers 
                SET current_usage = current_usage - 1,
                    updated_at = NOW()
                WHERE id = NEW.container_id;
            ELSIF OLD.status != 'occupied' AND NEW.status = 'occupied' THEN
                -- Increment usage when sample is assigned
                UPDATE storage_containers 
                SET current_usage = current_usage + 1,
                    updated_at = NOW()
                WHERE id = NEW.container_id;
            END IF;
        END IF;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        -- Decrement usage when position is deleted
        IF OLD.status = 'occupied' THEN
            UPDATE storage_containers 
            SET current_usage = current_usage - 1,
                updated_at = NOW()
            WHERE id = OLD.container_id;
        END IF;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for automatic usage tracking
CREATE TRIGGER trigger_update_container_usage
    AFTER INSERT OR UPDATE OR DELETE ON sample_positions
    FOR EACH ROW
    EXECUTE FUNCTION update_container_usage();

-- Insert sample storage locations
INSERT INTO storage_locations (name, zone_type, temperature_celsius, capacity, location_code, description) VALUES
('Ultra Low Freezer A', 'ultra_low_freezer', -80.0, 1000, 'ULF-A', 'Main ultra-low freezer for long-term sample storage'),
('Ultra Low Freezer B', 'ultra_low_freezer', -80.0, 1000, 'ULF-B', 'Backup ultra-low freezer'),
('Freezer Room A', 'freezer', -20.0, 2000, 'FR-A', 'Main freezer room for sample storage'),
('Refrigerated Storage', 'refrigerated', 4.0, 500, 'REF-A', 'Refrigerated storage for active samples'),
('Room Temperature Storage', 'room_temperature', 22.0, 300, 'RT-A', 'Room temperature storage for stable samples');

-- Insert sample hierarchical containers
DO $$
DECLARE
    freezer_a_id UUID;
    freezer_b_id UUID;
    ref_storage_id UUID;
    rack_id UUID;
    box_id UUID;
    i INTEGER;
    j INTEGER;
    k INTEGER;
    l INTEGER;
BEGIN
    -- Get storage location IDs
    SELECT id INTO freezer_a_id FROM storage_locations WHERE location_code = 'ULF-A';
    SELECT id INTO freezer_b_id FROM storage_locations WHERE location_code = 'ULF-B';
    SELECT id INTO ref_storage_id FROM storage_locations WHERE location_code = 'REF-A';
    
    -- Create containers for Ultra Low Freezer A
    FOR i IN 1..3 LOOP
        INSERT INTO storage_containers (name, container_type, storage_location_id, capacity, min_temperature_celsius, max_temperature_celsius, barcode)
        VALUES (
            'ULF-A-Freezer-' || i,
            'freezer',
            freezer_a_id,
            400,
            -85.0,
            -75.0,
            'ULFA-FRZ-' || LPAD(i::TEXT, 3, '0')
        );
        
        -- Create racks for each freezer
        FOR j IN 1..4 LOOP
            INSERT INTO storage_containers (name, container_type, parent_id, capacity, position_x, position_y, barcode)
            VALUES (
                'ULF-A-F' || i || '-Rack-' || j,
                'rack',
                (SELECT id FROM storage_containers WHERE name = 'ULF-A-Freezer-' || i),
                100,
                j,
                1,
                'ULFA-F' || i || '-R' || LPAD(j::TEXT, 2, '0')
            );
            
            -- Create boxes for each rack
            FOR k IN 1..5 LOOP
                INSERT INTO storage_containers (name, container_type, parent_id, capacity, position_x, position_y, width, height, barcode)
                VALUES (
                    'ULF-A-F' || i || '-R' || j || '-Box-' || k,
                    'box',
                    (SELECT id FROM storage_containers WHERE name = 'ULF-A-F' || i || '-Rack-' || j),
                    100,
                    k,
                    1,
                    10,
                    10,
                    'ULFA-F' || i || 'R' || j || 'B' || LPAD(k::TEXT, 2, '0')
                );
                
                -- Create positions for each box (10x10 grid)
                FOR l IN 1..100 LOOP
                    INSERT INTO sample_positions (container_id, position_x, position_y, status)
                    VALUES (
                        (SELECT id FROM storage_containers WHERE name = 'ULF-A-F' || i || '-R' || j || '-Box-' || k),
                        ((l - 1) % 10) + 1,
                        ((l - 1) / 10) + 1,
                        'empty'
                    );
                END LOOP;
            END LOOP;
        END LOOP;
    END LOOP;
    
    -- Create containers for Refrigerated Storage
    INSERT INTO storage_containers (name, container_type, storage_location_id, capacity, min_temperature_celsius, max_temperature_celsius, barcode)
    VALUES (
        'REF-A-Refrigerator-1',
        'freezer',
        ref_storage_id,
        200,
        2.0,
        6.0,
        'REFA-REF-001'
    );
    
    -- Create racks for refrigerated storage
    FOR i IN 1..2 LOOP
        INSERT INTO storage_containers (name, container_type, parent_id, capacity, position_x, position_y, barcode)
        VALUES (
            'REF-A-R1-Rack-' || i,
            'rack',
            (SELECT id FROM storage_containers WHERE name = 'REF-A-Refrigerator-1'),
            100,
            i,
            1,
            'REFA-R1-R' || LPAD(i::TEXT, 2, '0')
        );
        
        -- Create boxes for refrigerated racks
        FOR j IN 1..4 LOOP
            INSERT INTO storage_containers (name, container_type, parent_id, capacity, position_x, position_y, width, height, barcode)
            VALUES (
                'REF-A-R1-R' || i || '-Box-' || j,
                'box',
                (SELECT id FROM storage_containers WHERE name = 'REF-A-R1-Rack-' || i),
                25,
                j,
                1,
                5,
                5,
                'REFA-R1R' || i || 'B' || LPAD(j::TEXT, 2, '0')
            );
            
            -- Create positions for refrigerated boxes (5x5 grid)
            FOR k IN 1..25 LOOP
                INSERT INTO sample_positions (container_id, position_x, position_y, status)
                VALUES (
                    (SELECT id FROM storage_containers WHERE name = 'REF-A-R1-R' || i || '-Box-' || j),
                    ((k - 1) % 5) + 1,
                    ((k - 1) / 5) + 1,
                    'empty'
                );
            END LOOP;
        END LOOP;
    END LOOP;
END $$;

-- Update storage location usage counts
UPDATE storage_locations 
SET current_usage = (
    SELECT COUNT(*) 
    FROM storage_containers sc 
    WHERE sc.storage_location_id = storage_locations.id
); 