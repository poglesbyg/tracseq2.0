-- TracSeq 2.0 Seed Data - Part 2: Storage

-- Insert Storage Zones
INSERT INTO storage_zones (id, name, zone_type, temperature_range_min, temperature_range_max, current_temperature, capacity, created_at) VALUES
('zone-001', 'Ultra Low Freezer A', 'ultra_low_freezer', -80, -70, -78.5, 1000, NOW() - INTERVAL '2 years'),
('zone-002', 'Ultra Low Freezer B', 'ultra_low_freezer', -80, -70, -77.8, 1000, NOW() - INTERVAL '2 years'),
('zone-003', 'Standard Freezer 1', 'freezer', -20, -15, -18.2, 500, NOW() - INTERVAL '1 year'),
('zone-004', 'Refrigerator Unit 1', 'refrigerator', 2, 8, 4.5, 300, NOW() - INTERVAL '1 year'),
('zone-005', 'Room Temperature Storage', 'room_temperature', 20, 25, 22.1, 2000, NOW() - INTERVAL '2 years'),
('zone-006', 'Incubator 1', 'incubator', 35, 37, 36.8, 200, NOW() - INTERVAL '6 months')
ON CONFLICT (id) DO NOTHING;

-- Insert Storage Locations
INSERT INTO storage_locations (id, zone_id, location_code, location_type, shelf, rack, box, position, is_occupied, created_at) VALUES
-- Ultra Low Freezer A locations
('loc-001', 'zone-001', 'ULF-A-01-01-01-A1', 'box_position', '01', '01', '01', 'A1', true, NOW() - INTERVAL '6 months'),
('loc-002', 'zone-001', 'ULF-A-01-01-01-A2', 'box_position', '01', '01', '01', 'A2', true, NOW() - INTERVAL '5 months'),
('loc-003', 'zone-001', 'ULF-A-01-01-01-A3', 'box_position', '01', '01', '01', 'A3', true, NOW() - INTERVAL '4 months'),
('loc-004', 'zone-001', 'ULF-A-01-01-01-A4', 'box_position', '01', '01', '01', 'A4', false, NOW() - INTERVAL '4 months'),
('loc-005', 'zone-001', 'ULF-A-01-01-02-B1', 'box_position', '01', '01', '02', 'B1', true, NOW() - INTERVAL '3 months'),
-- Refrigerator locations
('loc-006', 'zone-004', 'REF-1-02-03-01-C1', 'box_position', '02', '03', '01', 'C1', true, NOW() - INTERVAL '2 months'),
('loc-007', 'zone-004', 'REF-1-02-03-01-C2', 'box_position', '02', '03', '01', 'C2', true, NOW() - INTERVAL '1 month'),
-- Room temperature locations
('loc-008', 'zone-005', 'RT-01-05-02', 'shelf', '01', '05', '02', NULL, true, NOW() - INTERVAL '2 weeks'),
('loc-009', 'zone-005', 'RT-01-05-03', 'shelf', '01', '05', '03', NULL, false, NOW() - INTERVAL '1 week'),
-- Additional locations for better demo
('loc-010', 'zone-001', 'ULF-A-02-01-01-A1', 'box_position', '02', '01', '01', 'A1', false, NOW() - INTERVAL '2 months'),
('loc-011', 'zone-002', 'ULF-B-01-01-01-A1', 'box_position', '01', '01', '01', 'A1', false, NOW() - INTERVAL '1 month'),
('loc-012', 'zone-003', 'FRZ-1-01-01-01-A1', 'box_position', '01', '01', '01', 'A1', false, NOW() - INTERVAL '3 weeks')
ON CONFLICT (id) DO NOTHING;

-- Insert Storage Temperature Logs
INSERT INTO storage_temperature_logs (zone_id, temperature, humidity, recorded_at) 
SELECT 
    'zone-001' as zone_id,
    -78.5 + (random() * 2 - 1) as temperature,
    45 + (random() * 5) as humidity,
    NOW() - (interval '1 hour' * generate_series(1, 24))
ON CONFLICT DO NOTHING;

INSERT INTO storage_temperature_logs (zone_id, temperature, humidity, recorded_at) 
SELECT 
    'zone-004' as zone_id,
    4.5 + (random() * 1 - 0.5) as temperature,
    65 + (random() * 5) as humidity,
    NOW() - (interval '1 hour' * generate_series(1, 24))
ON CONFLICT DO NOTHING;
