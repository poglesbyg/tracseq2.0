-- Enhanced Storage Service - Initial Schema Migration
-- File: enhanced_storage_service/migrations/001_initial_storage_schema.sql

-- Create custom types
CREATE TYPE storage_zone_type AS ENUM ('ultra_low_freezer', 'standard_freezer', 'refrigerated', 'room_temperature', 'incubator');
CREATE TYPE sensor_type AS ENUM ('temperature', 'humidity', 'pressure', 'co2', 'vibration');
CREATE TYPE sensor_status AS ENUM ('active', 'inactive', 'maintenance', 'error');
CREATE TYPE automation_status AS ENUM ('pending', 'running', 'completed', 'failed', 'paused');
CREATE TYPE movement_type AS ENUM ('storage', 'retrieval', 'relocation', 'maintenance');

-- Storage Locations table
CREATE TABLE storage_locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    zone_type storage_zone_type NOT NULL,
    zone_identifier VARCHAR(100) NOT NULL,
    temperature_range NUMRANGE NOT NULL,
    humidity_range NUMRANGE,
    capacity INTEGER NOT NULL DEFAULT 0,
    occupied INTEGER NOT NULL DEFAULT 0,
    coordinates JSONB, -- Physical coordinates: {x, y, z, room, building}
    equipment_id VARCHAR(255),
    equipment_model VARCHAR(255),
    operational_status VARCHAR(50) DEFAULT 'operational',
    last_maintenance_date TIMESTAMPTZ,
    next_maintenance_date TIMESTAMPTZ,
    location_metadata JSONB DEFAULT '{}',
    barcode VARCHAR(255) UNIQUE,
    parent_location_id UUID,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT storage_locations_capacity_check CHECK (capacity >= 0),
    CONSTRAINT storage_locations_occupied_check CHECK (occupied >= 0 AND occupied <= capacity),
    CONSTRAINT storage_locations_parent_fkey FOREIGN KEY (parent_location_id) REFERENCES storage_locations(id)
);

-- IoT Sensors table
CREATE TABLE iot_sensors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sensor_id VARCHAR(255) NOT NULL UNIQUE,
    sensor_type sensor_type NOT NULL,
    location_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    model VARCHAR(255),
    manufacturer VARCHAR(255),
    firmware_version VARCHAR(100),
    calibration_date TIMESTAMPTZ,
    next_calibration_date TIMESTAMPTZ,
    status sensor_status DEFAULT 'active',
    threshold_min DECIMAL(10,4),
    threshold_max DECIMAL(10,4),
    alert_threshold_min DECIMAL(10,4),
    alert_threshold_max DECIMAL(10,4),
    sampling_interval INTEGER DEFAULT 60, -- seconds
    last_reading DECIMAL(10,4),
    last_reading_time TIMESTAMPTZ,
    configuration JSONB DEFAULT '{}',
    is_critical BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT iot_sensors_location_fkey FOREIGN KEY (location_id) REFERENCES storage_locations(id) ON DELETE CASCADE,
    CONSTRAINT iot_sensors_sampling_check CHECK (sampling_interval > 0)
);

-- Sensor Readings table (time-series data)
CREATE TABLE sensor_readings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sensor_id UUID NOT NULL,
    reading_value DECIMAL(10,4) NOT NULL,
    reading_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    unit VARCHAR(20),
    quality_score DECIMAL(3,2) DEFAULT 1.0, -- 0.0 to 1.0
    is_anomaly BOOLEAN DEFAULT false,
    anomaly_score DECIMAL(3,2),
    metadata JSONB DEFAULT '{}',
    
    CONSTRAINT sensor_readings_sensor_fkey FOREIGN KEY (sensor_id) REFERENCES iot_sensors(id) ON DELETE CASCADE,
    CONSTRAINT sensor_readings_quality_check CHECK (quality_score >= 0.0 AND quality_score <= 1.0)
);

-- Storage Movement History table
CREATE TABLE storage_movement_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sample_id UUID, -- Reference to samples table (from sample_service)
    movement_type movement_type NOT NULL,
    from_location_id UUID,
    to_location_id UUID,
    moved_by UUID, -- Reference to users table
    movement_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reason TEXT,
    container_barcode VARCHAR(255),
    position_before VARCHAR(100),
    position_after VARCHAR(100),
    temperature_at_movement DECIMAL(5,2),
    chain_of_custody_signed BOOLEAN DEFAULT false,
    movement_metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT storage_movement_from_fkey FOREIGN KEY (from_location_id) REFERENCES storage_locations(id),
    CONSTRAINT storage_movement_to_fkey FOREIGN KEY (to_location_id) REFERENCES storage_locations(id),
    CONSTRAINT storage_movement_check CHECK (from_location_id IS NOT NULL OR to_location_id IS NOT NULL)
);

-- Automation Tasks table
CREATE TABLE automation_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_name VARCHAR(255) NOT NULL,
    task_type VARCHAR(100) NOT NULL, -- 'sample_movement', 'inventory_check', 'maintenance', etc.
    status automation_status DEFAULT 'pending',
    priority INTEGER DEFAULT 5, -- 1 (highest) to 10 (lowest)
    location_id UUID,
    target_sample_id UUID,
    robot_id VARCHAR(255),
    scheduled_time TIMESTAMPTZ,
    started_time TIMESTAMPTZ,
    completed_time TIMESTAMPTZ,
    estimated_duration_minutes INTEGER,
    actual_duration_minutes INTEGER,
    task_parameters JSONB DEFAULT '{}',
    task_results JSONB DEFAULT '{}',
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT automation_tasks_location_fkey FOREIGN KEY (location_id) REFERENCES storage_locations(id),
    CONSTRAINT automation_tasks_priority_check CHECK (priority >= 1 AND priority <= 10),
    CONSTRAINT automation_tasks_retry_check CHECK (retry_count >= 0 AND retry_count <= max_retries)
);

-- Digital Twin Models table (for virtual storage environment)
CREATE TABLE digital_twin_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_name VARCHAR(255) NOT NULL,
    model_version VARCHAR(50) NOT NULL,
    location_id UUID NOT NULL,
    model_type VARCHAR(100) NOT NULL, -- 'thermal', 'airflow', 'capacity', 'predictive'
    model_parameters JSONB NOT NULL DEFAULT '{}',
    training_data_hash VARCHAR(255),
    accuracy_score DECIMAL(5,4),
    last_training_date TIMESTAMPTZ,
    next_training_date TIMESTAMPTZ,
    prediction_horizon_hours INTEGER DEFAULT 24,
    model_status VARCHAR(50) DEFAULT 'active',
    model_artifacts JSONB DEFAULT '{}', -- Serialized model data
    validation_metrics JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT digital_twin_location_fkey FOREIGN KEY (location_id) REFERENCES storage_locations(id) ON DELETE CASCADE,
    CONSTRAINT digital_twin_accuracy_check CHECK (accuracy_score >= 0.0 AND accuracy_score <= 1.0),
    UNIQUE(model_name, model_version, location_id)
);

-- Predictive Analytics Results table
CREATE TABLE predictive_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_id UUID NOT NULL,
    prediction_type VARCHAR(100) NOT NULL, -- 'capacity_full', 'maintenance_due', 'temperature_excursion'
    predicted_value DECIMAL(10,4),
    confidence_score DECIMAL(3,2),
    prediction_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_until TIMESTAMPTZ,
    actual_value DECIMAL(10,4), -- Filled in later for model validation
    prediction_metadata JSONB DEFAULT '{}',
    alert_generated BOOLEAN DEFAULT false,
    alert_level VARCHAR(20), -- 'info', 'warning', 'critical'
    
    CONSTRAINT predictive_analytics_model_fkey FOREIGN KEY (model_id) REFERENCES digital_twin_models(id) ON DELETE CASCADE,
    CONSTRAINT predictive_analytics_confidence_check CHECK (confidence_score >= 0.0 AND confidence_score <= 1.0)
);

-- Environmental Alerts table
CREATE TABLE environmental_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_type VARCHAR(100) NOT NULL,
    severity VARCHAR(20) NOT NULL, -- 'low', 'medium', 'high', 'critical'
    location_id UUID,
    sensor_id UUID,
    message TEXT NOT NULL,
    current_value DECIMAL(10,4),
    threshold_value DECIMAL(10,4),
    alert_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    acknowledged_at TIMESTAMPTZ,
    acknowledged_by UUID,
    resolved_at TIMESTAMPTZ,
    resolved_by UUID,
    resolution_notes TEXT,
    escalation_level INTEGER DEFAULT 1,
    notification_sent BOOLEAN DEFAULT false,
    alert_metadata JSONB DEFAULT '{}',
    
    CONSTRAINT environmental_alerts_location_fkey FOREIGN KEY (location_id) REFERENCES storage_locations(id),
    CONSTRAINT environmental_alerts_sensor_fkey FOREIGN KEY (sensor_id) REFERENCES iot_sensors(id),
    CONSTRAINT environmental_alerts_escalation_check CHECK (escalation_level >= 1 AND escalation_level <= 5)
);

-- Storage Capacity History (for analytics and predictions)
CREATE TABLE storage_capacity_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    location_id UUID NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    total_capacity INTEGER NOT NULL,
    occupied_capacity INTEGER NOT NULL,
    available_capacity INTEGER NOT NULL,
    utilization_percentage DECIMAL(5,2) NOT NULL,
    projected_full_date TIMESTAMPTZ,
    capacity_trend VARCHAR(20), -- 'increasing', 'decreasing', 'stable'
    
    CONSTRAINT capacity_history_location_fkey FOREIGN KEY (location_id) REFERENCES storage_locations(id) ON DELETE CASCADE,
    CONSTRAINT capacity_history_utilization_check CHECK (utilization_percentage >= 0.0 AND utilization_percentage <= 100.0)
);

-- Create indexes for performance
CREATE INDEX idx_storage_locations_zone_type ON storage_locations(zone_type);
CREATE INDEX idx_storage_locations_operational_status ON storage_locations(operational_status);
CREATE INDEX idx_storage_locations_barcode ON storage_locations(barcode);
CREATE INDEX idx_storage_locations_coordinates_gin ON storage_locations USING gin(coordinates);

CREATE INDEX idx_iot_sensors_location_id ON iot_sensors(location_id);
CREATE INDEX idx_iot_sensors_sensor_type ON iot_sensors(sensor_type);
CREATE INDEX idx_iot_sensors_status ON iot_sensors(status);
CREATE INDEX idx_iot_sensors_last_reading_time ON iot_sensors(last_reading_time);

CREATE INDEX idx_sensor_readings_sensor_id ON sensor_readings(sensor_id);
CREATE INDEX idx_sensor_readings_time ON sensor_readings(reading_time);
CREATE INDEX idx_sensor_readings_time_sensor ON sensor_readings(reading_time, sensor_id);
CREATE INDEX idx_sensor_readings_anomaly ON sensor_readings(is_anomaly) WHERE is_anomaly = true;

CREATE INDEX idx_movement_history_sample_id ON storage_movement_history(sample_id);
CREATE INDEX idx_movement_history_movement_time ON storage_movement_history(movement_time);
CREATE INDEX idx_movement_history_from_location ON storage_movement_history(from_location_id);
CREATE INDEX idx_movement_history_to_location ON storage_movement_history(to_location_id);

CREATE INDEX idx_automation_tasks_status ON automation_tasks(status);
CREATE INDEX idx_automation_tasks_scheduled_time ON automation_tasks(scheduled_time);
CREATE INDEX idx_automation_tasks_priority ON automation_tasks(priority);
CREATE INDEX idx_automation_tasks_location_id ON automation_tasks(location_id);

CREATE INDEX idx_digital_twin_location_id ON digital_twin_models(location_id);
CREATE INDEX idx_digital_twin_model_type ON digital_twin_models(model_type);
CREATE INDEX idx_digital_twin_status ON digital_twin_models(model_status);

CREATE INDEX idx_predictive_analytics_model_id ON predictive_analytics(model_id);
CREATE INDEX idx_predictive_analytics_prediction_time ON predictive_analytics(prediction_time);
CREATE INDEX idx_predictive_analytics_prediction_type ON predictive_analytics(prediction_type);

CREATE INDEX idx_environmental_alerts_location_id ON environmental_alerts(location_id);
CREATE INDEX idx_environmental_alerts_sensor_id ON environmental_alerts(sensor_id);
CREATE INDEX idx_environmental_alerts_severity ON environmental_alerts(severity);
CREATE INDEX idx_environmental_alerts_alert_time ON environmental_alerts(alert_time);
CREATE INDEX idx_environmental_alerts_acknowledged ON environmental_alerts(acknowledged_at);

CREATE INDEX idx_capacity_history_location_id ON storage_capacity_history(location_id);
CREATE INDEX idx_capacity_history_timestamp ON storage_capacity_history(timestamp);

-- Create trigger functions for updated_at timestamps
CREATE OR REPLACE FUNCTION update_storage_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers
CREATE TRIGGER trigger_storage_locations_updated_at
    BEFORE UPDATE ON storage_locations
    FOR EACH ROW
    EXECUTE FUNCTION update_storage_updated_at();

CREATE TRIGGER trigger_iot_sensors_updated_at
    BEFORE UPDATE ON iot_sensors
    FOR EACH ROW
    EXECUTE FUNCTION update_storage_updated_at();

CREATE TRIGGER trigger_automation_tasks_updated_at
    BEFORE UPDATE ON automation_tasks
    FOR EACH ROW
    EXECUTE FUNCTION update_storage_updated_at();

CREATE TRIGGER trigger_digital_twin_models_updated_at
    BEFORE UPDATE ON digital_twin_models
    FOR EACH ROW
    EXECUTE FUNCTION update_storage_updated_at();

-- Insert sample data for testing
INSERT INTO storage_locations (name, zone_type, zone_identifier, temperature_range, capacity, equipment_id, equipment_model) VALUES
    ('Ultra-Low Freezer Zone A', 'ultra_low_freezer', 'ULF-A1', '[-85.0, -75.0]', 500, 'ULF-001', 'Thermo Fisher ULT-2586'),
    ('Standard Freezer Zone B', 'standard_freezer', 'SF-B1', '[-25.0, -15.0]', 300, 'SF-001', 'Fisher Scientific Freezer'),
    ('Refrigerated Storage Zone C', 'refrigerated', 'REF-C1', '[2.0, 8.0]', 200, 'REF-001', 'Lab Refrigerator Unit'),
    ('Room Temperature Zone D', 'room_temperature', 'RT-D1', '[18.0, 25.0]', 150, 'RT-001', 'Climate Controlled Cabinet'),
    ('Incubator Zone E', 'incubator', 'INC-E1', '[35.0, 39.0]', 100, 'INC-001', 'CO2 Incubator');

-- Insert sample IoT sensors
INSERT INTO iot_sensors (sensor_id, sensor_type, location_id, name, threshold_min, threshold_max, alert_threshold_min, alert_threshold_max) 
SELECT 
    'TEMP-' || substr(sl.zone_identifier, 1, 10) || '-01',
    'temperature',
    sl.id,
    'Temperature Sensor - ' || sl.name,
    lower(sl.temperature_range),
    upper(sl.temperature_range),
    lower(sl.temperature_range) - 2.0,
    upper(sl.temperature_range) + 2.0
FROM storage_locations sl;

COMMENT ON TABLE storage_locations IS 'Storage locations with temperature zones and capacity tracking';
COMMENT ON TABLE iot_sensors IS 'IoT sensors for environmental monitoring';
COMMENT ON TABLE sensor_readings IS 'Time-series data from IoT sensors';
COMMENT ON TABLE storage_movement_history IS 'Audit trail of sample movements between storage locations';
COMMENT ON TABLE automation_tasks IS 'Robotic automation tasks for sample handling';
COMMENT ON TABLE digital_twin_models IS 'Digital twin models for predictive analytics';
COMMENT ON TABLE predictive_analytics IS 'Results from predictive analytics models';
COMMENT ON TABLE environmental_alerts IS 'Environmental condition alerts and notifications';
COMMENT ON TABLE storage_capacity_history IS 'Historical capacity utilization for analytics'; 