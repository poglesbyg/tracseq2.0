-- Create custom types for storage system
CREATE TYPE temperature_zone AS ENUM (
    '-80C',
    '-20C', 
    '4C',
    'RT',
    '37C'
);

CREATE TYPE container_type AS ENUM (
    'tube',
    'plate',
    'box',
    'rack',
    'bag'
);

CREATE TYPE storage_state AS ENUM (
    'pending',
    'validated',
    'instorage',
    'insequencing',
    'completed',
    'discarded'
);

-- Storage locations table
CREATE TABLE storage_locations (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    temperature_zone temperature_zone NOT NULL,
    capacity INTEGER NOT NULL DEFAULT 100,
    current_usage INTEGER NOT NULL DEFAULT 0,
    container_type container_type NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    location_path TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT storage_locations_name_unique UNIQUE (name),
    CONSTRAINT storage_locations_capacity_positive CHECK (capacity > 0),
    CONSTRAINT storage_locations_usage_valid CHECK (current_usage >= 0 AND current_usage <= capacity)
);

-- Sample locations table for tracking sample storage
CREATE TABLE sample_locations (
    id SERIAL PRIMARY KEY,
    sample_id INTEGER NOT NULL,
    location_id INTEGER NOT NULL REFERENCES storage_locations(id) ON DELETE RESTRICT,
    barcode VARCHAR(255) NOT NULL,
    position VARCHAR(50),
    storage_state storage_state NOT NULL DEFAULT 'pending',
    stored_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    stored_by VARCHAR(255),
    moved_at TIMESTAMPTZ,
    moved_by VARCHAR(255),
    notes TEXT,
    temperature_log TEXT, -- JSON log of temperature readings
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT sample_locations_barcode_unique UNIQUE (barcode),
    CONSTRAINT sample_locations_sample_unique UNIQUE (sample_id)
);

-- Storage movement history for audit trail
CREATE TABLE storage_movement_history (
    id SERIAL PRIMARY KEY,
    sample_id INTEGER NOT NULL,
    barcode VARCHAR(255) NOT NULL,
    from_location_id INTEGER REFERENCES storage_locations(id),
    to_location_id INTEGER NOT NULL REFERENCES storage_locations(id),
    from_state storage_state,
    to_state storage_state NOT NULL,
    movement_reason VARCHAR(500) NOT NULL,
    moved_by VARCHAR(255) NOT NULL,
    moved_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    notes TEXT
);

-- Create indexes for performance
CREATE INDEX idx_sample_locations_sample_id ON sample_locations(sample_id);
CREATE INDEX idx_sample_locations_location_id ON sample_locations(location_id);
CREATE INDEX idx_sample_locations_barcode ON sample_locations(barcode);
CREATE INDEX idx_sample_locations_storage_state ON sample_locations(storage_state);
CREATE INDEX idx_storage_movement_history_sample_id ON storage_movement_history(sample_id);
CREATE INDEX idx_storage_movement_history_moved_at ON storage_movement_history(moved_at);
CREATE INDEX idx_storage_locations_temperature_zone ON storage_locations(temperature_zone);
CREATE INDEX idx_storage_locations_is_active ON storage_locations(is_active);

-- Insert some default storage locations for testing
INSERT INTO storage_locations (name, description, temperature_zone, capacity, container_type, location_path) VALUES
('Freezer A (-80°C)', 'Ultra low temperature freezer for long-term biological sample storage', '-80C', 100, 'box', 'Building A/Room 101/Freezer A'),
('Freezer B (-20°C)', 'Standard freezer for short-term frozen storage', '-20C', 80, 'rack', 'Building A/Room 101/Freezer B'),
('Refrigerator (4°C)', 'Cold storage for reagents and samples', '4C', 50, 'rack', 'Building A/Room 101/Refrigerator'),
('Room Temperature Storage', 'Ambient storage for stable compounds', 'RT', 200, 'rack', 'Building A/Room 102/Shelves'),
('Incubator (37°C)', 'Temperature controlled environment for cultures', '37C', 30, 'plate', 'Building A/Room 103/Incubator');

-- Create trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_storage_locations_updated_at BEFORE UPDATE ON storage_locations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_sample_locations_updated_at BEFORE UPDATE ON sample_locations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column(); 
