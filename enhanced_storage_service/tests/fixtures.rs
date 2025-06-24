use chrono::{DateTime, Utc};
use fake::{Fake, Faker};
use uuid::Uuid;
use enhanced_storage_service::models::*;
use crate::test_utils::TestDataFactory;

/// Storage Location Fixtures
pub struct StorageLocationFixtures;

impl StorageLocationFixtures {
    pub fn create_location_request() -> CreateStorageLocationRequest {
        CreateStorageLocationRequest {
            name: format!("Test Location {}", fastrand::u32(1..=1000)),
            description: Some("Test storage location for automated testing".to_string()),
            location_type: "rack".to_string(),
            temperature_zone: "-20C".to_string(),
            max_capacity: 100,
            coordinates: Some(TestDataFactory::coordinates()),
            metadata: Some(serde_json::json!({
                "test": true,
                "created_by": "test_suite"
            })),
        }
    }

    pub fn create_location_request_with_zone(zone: &str) -> CreateStorageLocationRequest {
        let mut request = Self::create_location_request();
        request.temperature_zone = zone.to_string();
        request.name = format!("Test {} Location {}", zone, fastrand::u32(1..=1000));
        request
    }

    pub fn update_location_request() -> UpdateStorageLocationRequest {
        UpdateStorageLocationRequest {
            name: Some(format!("Updated Location {}", fastrand::u32(1..=1000))),
            description: Some("Updated description".to_string()),
            location_type: Some("freezer".to_string()),
            temperature_zone: Some("-80C".to_string()),
            max_capacity: Some(200),
            coordinates: Some(TestDataFactory::coordinates()),
            status: Some("maintenance".to_string()),
            metadata: Some(serde_json::json!({
                "updated": true,
                "test": true
            })),
        }
    }

    pub fn storage_location() -> StorageLocation {
        StorageLocation {
            id: TestDataFactory::uuid(),
            name: format!("Test Location {}", fastrand::u32(1..=1000)),
            description: Some("Test storage location".to_string()),
            location_type: "rack".to_string(),
            temperature_zone: "-20C".to_string(),
            max_capacity: 100,
            current_capacity: fastrand::i32(0..=50),
            coordinates: Some(TestDataFactory::coordinates()),
            status: "active".to_string(),
            metadata: serde_json::json!({"test": true}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

/// Sample Fixtures
pub struct SampleFixtures;

impl SampleFixtures {
    pub fn store_sample_request(location_id: Uuid) -> StoreSampleRequest {
        StoreSampleRequest {
            barcode: TestDataFactory::barcode(),
            sample_type: "blood".to_string(),
            storage_location_id: location_id,
            position: Some(TestDataFactory::coordinates()),
            temperature_requirements: Some("-20C".to_string()),
            metadata: Some(serde_json::json!({
                "patient_id": "P12345",
                "collection_date": "2024-01-15",
                "test": true
            })),
        }
    }

    pub fn store_sample_request_with_type(location_id: Uuid, sample_type: &str) -> StoreSampleRequest {
        let mut request = Self::store_sample_request(location_id);
        request.sample_type = sample_type.to_string();
        request
    }

    pub fn move_sample_request(new_location_id: Uuid) -> MoveSampleRequest {
        MoveSampleRequest {
            new_location_id,
            new_position: Some(TestDataFactory::coordinates()),
            reason: "Quality control testing".to_string(),
        }
    }

    pub fn sample() -> Sample {
        Sample {
            id: TestDataFactory::uuid(),
            barcode: TestDataFactory::barcode(),
            sample_type: "blood".to_string(),
            storage_location_id: Some(TestDataFactory::uuid()),
            position: Some(TestDataFactory::coordinates()),
            temperature_requirements: Some("-20C".to_string()),
            status: "stored".to_string(),
            metadata: serde_json::json!({"test": true}),
            chain_of_custody: serde_json::json!([{
                "action": "stored",
                "timestamp": Utc::now(),
                "location_id": TestDataFactory::uuid()
            }]),
            stored_at: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

/// IoT Sensor Fixtures
pub struct IoTSensorFixtures;

impl IoTSensorFixtures {
    pub fn iot_sensor() -> IoTSensor {
        IoTSensor {
            id: TestDataFactory::uuid(),
            sensor_id: TestDataFactory::sensor_id(),
            sensor_type: "temperature".to_string(),
            location_id: Some(TestDataFactory::uuid()),
            status: "active".to_string(),
            last_reading: Some(serde_json::json!({
                "temperature": -20.5,
                "humidity": 45.2,
                "timestamp": Utc::now()
            })),
            calibration_data: serde_json::json!({
                "last_calibration": Utc::now(),
                "calibration_offset": 0.1,
                "accuracy": 0.95
            }),
            maintenance_schedule: serde_json::json!({
                "next_maintenance": "2024-06-01",
                "maintenance_interval_days": 90
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn sensor_reading(sensor_id: &str) -> SensorReading {
        SensorReading {
            sensor_id: sensor_id.to_string(),
            readings: vec![
                SensorReadingValue {
                    reading_type: "temperature".to_string(),
                    value: TestDataFactory::temperature("-20C"),
                    unit: "celsius".to_string(),
                    quality_score: Some(0.98),
                },
                SensorReadingValue {
                    reading_type: "humidity".to_string(),
                    value: 45.0 + (fastrand::f64() * 10.0 - 5.0),
                    unit: "percent".to_string(),
                    quality_score: Some(0.95),
                },
            ],
            timestamp: Utc::now(),
        }
    }

    pub fn sensor_data() -> SensorData {
        SensorData {
            id: TestDataFactory::uuid(),
            sensor_id: TestDataFactory::uuid(),
            reading_type: "temperature".to_string(),
            value: TestDataFactory::temperature("-20C"),
            unit: "celsius".to_string(),
            quality_score: 0.98,
            metadata: serde_json::json!({"test": true}),
            recorded_at: Utc::now(),
        }
    }
}

/// Alert Fixtures
pub struct AlertFixtures;

impl AlertFixtures {
    pub fn create_alert_request() -> CreateAlertRequest {
        CreateAlertRequest {
            alert_type: "environmental".to_string(),
            severity: AlertSeverity::High,
            title: "Temperature Alert".to_string(),
            message: "Temperature deviation detected in storage location".to_string(),
            source_type: "sensor".to_string(),
            source_id: Some(TestDataFactory::uuid()),
            metadata: Some(serde_json::json!({
                "sensor_reading": -25.5,
                "threshold": -22.0,
                "deviation": 3.5
            })),
        }
    }

    pub fn alert() -> Alert {
        Alert {
            id: TestDataFactory::uuid(),
            alert_type: "environmental".to_string(),
            severity: "high".to_string(),
            title: "Temperature Alert".to_string(),
            message: "Temperature deviation detected".to_string(),
            source_type: "sensor".to_string(),
            source_id: Some(TestDataFactory::uuid()),
            status: "active".to_string(),
            acknowledged_by: None,
            acknowledged_at: None,
            resolved_at: None,
            metadata: serde_json::json!({"test": true}),
            created_at: Utc::now(),
        }
    }
}

/// Analytics Fixtures
pub struct AnalyticsFixtures;

impl AnalyticsFixtures {
    pub fn analytics_model() -> AnalyticsModel {
        AnalyticsModel {
            id: TestDataFactory::uuid(),
            model_name: "capacity_prediction".to_string(),
            model_type: "linear_regression".to_string(),
            version: "1.0.0".to_string(),
            model_data: serde_json::json!({
                "coefficients": [0.5, 0.3, 0.2],
                "intercept": 10.0,
                "features": ["utilization", "trend", "seasonality"]
            }),
            performance_metrics: serde_json::json!({
                "accuracy": 0.92,
                "rmse": 2.5,
                "mae": 1.8
            }),
            training_metadata: serde_json::json!({
                "training_samples": 10000,
                "training_date": Utc::now(),
                "validation_split": 0.2
            }),
            status: "active".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn prediction_request() -> PredictionRequest {
        PredictionRequest {
            model_type: "capacity_prediction".to_string(),
            input_data: serde_json::json!({
                "current_utilization": 0.75,
                "historical_trend": 0.05,
                "seasonal_factor": 1.1,
                "location_id": TestDataFactory::uuid()
            }),
            prediction_horizon: Some(30),
        }
    }

    pub fn prediction() -> Prediction {
        Prediction {
            id: TestDataFactory::uuid(),
            model_id: TestDataFactory::uuid(),
            prediction_type: "capacity".to_string(),
            input_data: serde_json::json!({
                "utilization": 0.75,
                "trend": 0.05
            }),
            prediction_result: serde_json::json!({
                "predicted_capacity": 85.2,
                "confidence_interval": [80.1, 90.3]
            }),
            confidence_score: Some(0.89),
            prediction_horizon: Some(30),
            metadata: serde_json::json!({"test": true}),
            created_at: Utc::now(),
        }
    }
}

/// Blockchain Fixtures
pub struct BlockchainFixtures;

impl BlockchainFixtures {
    pub fn blockchain_record() -> BlockchainRecord {
        BlockchainRecord {
            transaction_type: "sample_stored".to_string(),
            data: serde_json::json!({
                "sample_id": TestDataFactory::uuid(),
                "location_id": TestDataFactory::uuid(),
                "timestamp": Utc::now(),
                "user_id": TestDataFactory::uuid()
            }),
            timestamp: Utc::now(),
            previous_hash: Some("prev_hash_123".to_string()),
        }
    }

    pub fn blockchain_transaction() -> BlockchainTransaction {
        BlockchainTransaction {
            id: TestDataFactory::uuid(),
            transaction_hash: format!("hash_{}", fastrand::u64(..)),
            block_number: Some(fastrand::i64(1..=1000)),
            transaction_type: "sample_stored".to_string(),
            data_hash: format!("data_hash_{}", fastrand::u64(..)),
            previous_hash: Some(format!("prev_hash_{}", fastrand::u64(..))),
            timestamp: Utc::now(),
            signature: "test_signature".to_string(),
            metadata: serde_json::json!({"test": true}),
            created_at: Utc::now(),
        }
    }
}

/// Automation Fixtures
pub struct AutomationFixtures;

impl AutomationFixtures {
    pub fn create_automation_task_request() -> CreateAutomationTaskRequest {
        CreateAutomationTaskRequest {
            task_type: "sample_retrieval".to_string(),
            priority: Some(5),
            input_parameters: serde_json::json!({
                "sample_id": TestDataFactory::uuid(),
                "destination": "lab_bench_1"
            }),
            scheduled_at: Some(Utc::now()),
            metadata: Some(serde_json::json!({
                "requested_by": "test_user",
                "urgency": "normal"
            })),
        }
    }

    pub fn automation_task() -> AutomationTask {
        AutomationTask {
            id: TestDataFactory::uuid(),
            task_type: "sample_retrieval".to_string(),
            priority: 5,
            status: "pending".to_string(),
            input_parameters: serde_json::json!({
                "sample_id": TestDataFactory::uuid()
            }),
            output_results: None,
            assigned_robot_id: Some("ROBOT_001".to_string()),
            scheduled_at: Some(Utc::now()),
            started_at: None,
            completed_at: None,
            error_message: None,
            metadata: serde_json::json!({"test": true}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn robot_status() -> RobotStatus {
        RobotStatus {
            robot_id: "ROBOT_001".to_string(),
            status: "idle".to_string(),
            current_task: None,
            position: Some(serde_json::json!({
                "x": 10.5,
                "y": 20.3,
                "z": 1.0
            })),
            battery_level: Some(85.2),
            last_communication: Utc::now(),
        }
    }
}

/// Energy Fixtures
pub struct EnergyFixtures;

impl EnergyFixtures {
    pub fn energy_consumption() -> EnergyConsumption {
        EnergyConsumption {
            id: TestDataFactory::uuid(),
            location_id: Some(TestDataFactory::uuid()),
            equipment_type: "freezer".to_string(),
            consumption_kwh: 15.5,
            cost_usd: Some(2.33),
            efficiency_ratio: Some(0.92),
            optimization_suggestions: serde_json::json!([
                {
                    "type": "schedule_optimization",
                    "description": "Adjust defrost cycles",
                    "potential_savings": 2.1
                }
            ]),
            period_start: Utc::now() - chrono::Duration::hours(24),
            period_end: Utc::now(),
            metadata: serde_json::json!({"test": true}),
            recorded_at: Utc::now(),
        }
    }
}

/// Compliance Fixtures
pub struct ComplianceFixtures;

impl ComplianceFixtures {
    pub fn compliance_event() -> ComplianceEvent {
        ComplianceEvent {
            id: TestDataFactory::uuid(),
            event_type: "temperature_monitoring".to_string(),
            regulatory_standard: "FDA_CFR_21".to_string(),
            compliance_status: "compliant".to_string(),
            description: "Temperature monitoring within acceptable range".to_string(),
            affected_entity_type: "storage_location".to_string(),
            affected_entity_id: TestDataFactory::uuid(),
            remediation_required: false,
            remediation_actions: serde_json::json!([]),
            auditor_notes: Some("Automated compliance check passed".to_string()),
            metadata: serde_json::json!({"test": true}),
            occurred_at: Utc::now(),
            created_at: Utc::now(),
        }
    }
}

/// Pagination test fixtures
pub struct PaginationFixtures;

impl PaginationFixtures {
    pub fn pagination_query() -> (Option<i32>, Option<i32>) {
        (Some(1), Some(10))
    }

    pub fn pagination_info(total_items: i64) -> PaginationInfo {
        PaginationInfo {
            page: 1,
            per_page: 10,
            total_pages: (total_items as i32 + 9) / 10,
            total_items,
            has_next: total_items > 10,
            has_prev: false,
        }
    }
} 
