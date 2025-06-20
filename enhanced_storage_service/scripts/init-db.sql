-- Enhanced Storage Service Database Initialization
-- This script creates the necessary tables and initial data for local development

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ================================
-- CORE STORAGE TABLES (Phase 1)
-- ================================

-- Storage locations table
CREATE TABLE IF NOT EXISTS storage_locations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    location_type VARCHAR(100) NOT NULL,
    temperature_min DECIMAL(5,2),
    temperature_max DECIMAL(5,2),
    capacity INTEGER NOT NULL DEFAULT 0,
    available_capacity INTEGER NOT NULL DEFAULT 0,
    coordinates JSONB,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Samples table
CREATE TABLE IF NOT EXISTS samples (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    barcode VARCHAR(255) UNIQUE NOT NULL,
    sample_type VARCHAR(100) NOT NULL,
    location_id UUID REFERENCES storage_locations(id),
    project_id VARCHAR(255),
    status VARCHAR(50) DEFAULT 'pending',
    volume_ml DECIMAL(10,3),
    temperature DECIMAL(5,2),
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- IoT sensors table
CREATE TABLE IF NOT EXISTS iot_sensors (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    sensor_id VARCHAR(255) UNIQUE NOT NULL,
    sensor_type VARCHAR(100) NOT NULL,
    location_id UUID REFERENCES storage_locations(id),
    status VARCHAR(50) DEFAULT 'active',
    last_reading JSONB,
    last_reading_at TIMESTAMP WITH TIME ZONE,
    configuration JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Sensor readings table
CREATE TABLE IF NOT EXISTS sensor_readings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    sensor_id UUID REFERENCES iot_sensors(id),
    reading_type VARCHAR(100) NOT NULL,
    value DECIMAL(10,4) NOT NULL,
    unit VARCHAR(20),
    quality_score DECIMAL(3,2),
    metadata JSONB,
    recorded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ================================
-- AUTOMATION TABLES (Phase 1)
-- ================================

-- Robots table
CREATE TABLE IF NOT EXISTS robots (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    robot_id VARCHAR(255) UNIQUE NOT NULL,
    robot_type VARCHAR(100) NOT NULL,
    status VARCHAR(50) DEFAULT 'idle',
    current_location JSONB,
    capabilities JSONB,
    maintenance_schedule JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Automation jobs table
CREATE TABLE IF NOT EXISTS automation_jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    job_type VARCHAR(100) NOT NULL,
    status VARCHAR(50) DEFAULT 'queued',
    robot_id UUID REFERENCES robots(id),
    sample_id UUID REFERENCES samples(id),
    parameters JSONB,
    result JSONB,
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ================================
-- BLOCKCHAIN TABLES (Phase 1)
-- ================================

-- Blockchain transactions table
CREATE TABLE IF NOT EXISTS blockchain_transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    transaction_hash VARCHAR(255) UNIQUE NOT NULL,
    block_number BIGINT,
    sample_id UUID REFERENCES samples(id),
    event_type VARCHAR(100) NOT NULL,
    event_data JSONB,
    gas_used BIGINT,
    transaction_fee DECIMAL(20,8),
    confirmed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Chain of custody events table
CREATE TABLE IF NOT EXISTS custody_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    sample_id UUID REFERENCES samples(id),
    event_type VARCHAR(100) NOT NULL,
    actor VARCHAR(255) NOT NULL,
    location VARCHAR(255),
    digital_signature TEXT,
    blockchain_tx_id UUID REFERENCES blockchain_transactions(id),
    metadata JSONB,
    occurred_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ================================
-- ANALYTICS TABLES (Phase 1)
-- ================================

-- Analytics data table
CREATE TABLE IF NOT EXISTS analytics_data (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    metric_name VARCHAR(255) NOT NULL,
    metric_value DECIMAL(15,4),
    dimensions JSONB,
    calculated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    valid_until TIMESTAMP WITH TIME ZONE
);

-- Energy consumption table
CREATE TABLE IF NOT EXISTS energy_consumption (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    equipment_id VARCHAR(255) NOT NULL,
    power_kw DECIMAL(10,4) NOT NULL,
    energy_kwh DECIMAL(10,4),
    cost_usd DECIMAL(10,2),
    carbon_footprint_kg DECIMAL(10,4),
    recorded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ================================
-- AI/ML TABLES (Phase 2)
-- ================================

-- AI models table
CREATE TABLE IF NOT EXISTS ai_models (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    model_name VARCHAR(255) UNIQUE NOT NULL,
    model_type VARCHAR(100) NOT NULL,
    version VARCHAR(50) NOT NULL,
    accuracy DECIMAL(5,4),
    status VARCHAR(50) DEFAULT 'active',
    model_path TEXT,
    training_data JSONB,
    metrics JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- AI predictions table
CREATE TABLE IF NOT EXISTS ai_predictions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    model_name VARCHAR(255) NOT NULL,
    prediction_type VARCHAR(100) NOT NULL,
    input_data JSONB NOT NULL,
    prediction_result JSONB NOT NULL,
    confidence DECIMAL(5,4),
    inference_time_ms INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Anomaly detection table
CREATE TABLE IF NOT EXISTS anomaly_detections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    anomaly_type VARCHAR(100) NOT NULL,
    severity VARCHAR(50) NOT NULL,
    description TEXT,
    affected_equipment JSONB,
    detection_confidence DECIMAL(5,4),
    resolved BOOLEAN DEFAULT FALSE,
    resolved_at TIMESTAMP WITH TIME ZONE,
    detected_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ================================
-- INTEGRATION TABLES (Phase 3)
-- ================================

-- Integration status table
CREATE TABLE IF NOT EXISTS integration_status (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    integration_name VARCHAR(255) UNIQUE NOT NULL,
    integration_type VARCHAR(100) NOT NULL,
    status VARCHAR(50) DEFAULT 'active',
    last_sync TIMESTAMP WITH TIME ZONE,
    error_count INTEGER DEFAULT 0,
    last_error TEXT,
    configuration JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- LIMS synchronization table
CREATE TABLE IF NOT EXISTS lims_sync_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    sample_id UUID REFERENCES samples(id),
    lims_id VARCHAR(255),
    sync_status VARCHAR(50) DEFAULT 'pending',
    sync_direction VARCHAR(20) DEFAULT 'bidirectional',
    last_sync_at TIMESTAMP WITH TIME ZONE,
    sync_errors JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ERP synchronization table
CREATE TABLE IF NOT EXISTS erp_sync_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    entity_type VARCHAR(100) NOT NULL,
    entity_id VARCHAR(255) NOT NULL,
    erp_id VARCHAR(255),
    sync_status VARCHAR(50) DEFAULT 'pending',
    last_sync_at TIMESTAMP WITH TIME ZONE,
    sync_data JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Cloud storage table
CREATE TABLE IF NOT EXISTS cloud_storage (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    file_name VARCHAR(255) NOT NULL,
    cloud_provider VARCHAR(50) NOT NULL,
    storage_path TEXT NOT NULL,
    file_size BIGINT,
    content_type VARCHAR(100),
    encryption_enabled BOOLEAN DEFAULT TRUE,
    backup_status VARCHAR(50) DEFAULT 'pending',
    uploaded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ================================
-- INDEXES FOR PERFORMANCE
-- ================================

-- Core indexes
CREATE INDEX IF NOT EXISTS idx_samples_barcode ON samples(barcode);
CREATE INDEX IF NOT EXISTS idx_samples_location_id ON samples(location_id);
CREATE INDEX IF NOT EXISTS idx_samples_status ON samples(status);
CREATE INDEX IF NOT EXISTS idx_sensor_readings_sensor_id ON sensor_readings(sensor_id);
CREATE INDEX IF NOT EXISTS idx_sensor_readings_recorded_at ON sensor_readings(recorded_at);

-- Automation indexes
CREATE INDEX IF NOT EXISTS idx_automation_jobs_status ON automation_jobs(status);
CREATE INDEX IF NOT EXISTS idx_automation_jobs_robot_id ON automation_jobs(robot_id);

-- Blockchain indexes
CREATE INDEX IF NOT EXISTS idx_blockchain_tx_hash ON blockchain_transactions(transaction_hash);
CREATE INDEX IF NOT EXISTS idx_custody_events_sample_id ON custody_events(sample_id);

-- Analytics indexes
CREATE INDEX IF NOT EXISTS idx_analytics_data_metric_name ON analytics_data(metric_name);
CREATE INDEX IF NOT EXISTS idx_energy_consumption_equipment_id ON energy_consumption(equipment_id);
CREATE INDEX IF NOT EXISTS idx_energy_consumption_recorded_at ON energy_consumption(recorded_at);

-- AI/ML indexes
CREATE INDEX IF NOT EXISTS idx_ai_predictions_model_name ON ai_predictions(model_name);
CREATE INDEX IF NOT EXISTS idx_ai_predictions_created_at ON ai_predictions(created_at);
CREATE INDEX IF NOT EXISTS idx_anomaly_detections_severity ON anomaly_detections(severity);
CREATE INDEX IF NOT EXISTS idx_anomaly_detections_resolved ON anomaly_detections(resolved);

-- Integration indexes
CREATE INDEX IF NOT EXISTS idx_lims_sync_sample_id ON lims_sync_records(sample_id);
CREATE INDEX IF NOT EXISTS idx_erp_sync_entity_type ON erp_sync_records(entity_type);
CREATE INDEX IF NOT EXISTS idx_cloud_storage_provider ON cloud_storage(cloud_provider);

-- ================================
-- SAMPLE DATA FOR DEVELOPMENT
-- ================================

-- Insert sample storage locations
INSERT INTO storage_locations (name, location_type, temperature_min, temperature_max, capacity, available_capacity, coordinates) VALUES
('Ultra-Low Freezer A1', 'freezer', -86.0, -70.0, 100, 85, '{"x": 10, "y": 20, "z": 5}'),
('Standard Freezer B1', 'freezer', -25.0, -15.0, 200, 150, '{"x": 30, "y": 20, "z": 5}'),
('Refrigerator C1', 'refrigerator', 2.0, 8.0, 300, 250, '{"x": 50, "y": 20, "z": 5}'),
('Room Temperature D1', 'ambient', 18.0, 25.0, 500, 400, '{"x": 70, "y": 20, "z": 5}'),
('Incubator E1', 'incubator', 35.0, 40.0, 150, 120, '{"x": 90, "y": 20, "z": 5}')
ON CONFLICT DO NOTHING;

-- Insert sample IoT sensors
INSERT INTO iot_sensors (sensor_id, sensor_type, location_id, status, configuration) 
SELECT 
    'TEMP_001', 'temperature', id, 'active', '{"threshold_min": -86, "threshold_max": -70, "polling_interval": 30}'
FROM storage_locations WHERE name = 'Ultra-Low Freezer A1'
ON CONFLICT DO NOTHING;

INSERT INTO iot_sensors (sensor_id, sensor_type, location_id, status, configuration) 
SELECT 
    'HUMID_001', 'humidity', id, 'active', '{"threshold_min": 30, "threshold_max": 70, "polling_interval": 60}'
FROM storage_locations WHERE name = 'Standard Freezer B1'
ON CONFLICT DO NOTHING;

-- Insert sample robots
INSERT INTO robots (robot_id, robot_type, status, current_location, capabilities) VALUES
('ROBOT_001', 'sample_handler', 'idle', '{"x": 0, "y": 0, "z": 0}', '{"max_payload_kg": 5, "max_reach_m": 2.5, "precision_mm": 0.1}'),
('ROBOT_002', 'mobile_platform', 'active', '{"x": 15, "y": 10, "z": 0}', '{"max_speed_ms": 1.5, "max_payload_kg": 50, "navigation": "autonomous"}')
ON CONFLICT DO NOTHING;

-- Insert sample AI models
INSERT INTO ai_models (model_name, model_type, version, accuracy, status, metrics) VALUES
('equipment_failure_prediction', 'predictive_maintenance', '1.0.0', 0.94, 'active', '{"f1_score": 0.91, "precision": 0.89, "recall": 0.94}'),
('sample_routing_optimization', 'intelligent_routing', '1.0.0', 0.92, 'active', '{"optimization_score": 0.88, "average_time_saved": 0.15}'),
('system_anomaly_detection', 'anomaly_detection', '1.0.0', 0.89, 'active', '{"false_positive_rate": 0.05, "detection_accuracy": 0.89}')
ON CONFLICT DO NOTHING;

-- Insert sample integration status
INSERT INTO integration_status (integration_name, integration_type, status, configuration) VALUES
('lims_primary', 'LIMS', 'active', '{"base_url": "http://mock-lims:8090", "sync_interval": 15}'),
('erp_primary', 'ERP', 'active', '{"base_url": "http://mock-erp:8091", "sync_interval": 60}'),
('aws_cloud', 'Cloud', 'active', '{"provider": "aws", "region": "us-west-2"}'),
('azure_cloud', 'Cloud', 'active', '{"provider": "azure", "region": "eastus"}'),
('gcp_cloud', 'Cloud', 'active', '{"provider": "gcp", "region": "us-central1"}')
ON CONFLICT DO NOTHING;

-- ================================
-- UPDATE TIMESTAMPS TRIGGER
-- ================================

-- Create function to update timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply trigger to tables with updated_at column
CREATE TRIGGER update_storage_locations_updated_at BEFORE UPDATE ON storage_locations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_samples_updated_at BEFORE UPDATE ON samples FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_iot_sensors_updated_at BEFORE UPDATE ON iot_sensors FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_robots_updated_at BEFORE UPDATE ON robots FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_ai_models_updated_at BEFORE UPDATE ON ai_models FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_integration_status_updated_at BEFORE UPDATE ON integration_status FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ================================
-- COMPLETION MESSAGE
-- ================================

DO $$
BEGIN
    RAISE NOTICE 'Enhanced Storage Service database initialization completed successfully!';
    RAISE NOTICE 'Database includes tables for:';
    RAISE NOTICE '  - Core Platform (Phase 1): Storage, IoT, Analytics, Automation, Blockchain';
    RAISE NOTICE '  - AI/ML Platform (Phase 2): Models, Predictions, Anomaly Detection';
    RAISE NOTICE '  - Enterprise Integration (Phase 3): LIMS, ERP, Cloud Storage';
    RAISE NOTICE 'Sample data has been inserted for development and testing.';
END $$; 
