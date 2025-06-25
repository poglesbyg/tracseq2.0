use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database_url: String,
    pub auth_service_url: String,
    pub iot: IoTConfig,
    pub analytics: AnalyticsConfig,
    pub digital_twin: DigitalTwinConfig,
    pub blockchain: BlockchainConfig,
    pub automation: AutomationConfig,
    pub energy: EnergyConfig,
    pub mobile: MobileConfig,
    pub compliance: ComplianceConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoTConfig {
    pub enabled: bool,
    pub mqtt_broker_url: String,
    pub mqtt_username: String,
    pub mqtt_password: String,
    pub sensor_polling_interval_seconds: u64,
    pub alert_threshold_temperature: f32,
    pub alert_threshold_humidity: f32,
    pub calibration_enabled: bool,
    pub real_time_monitoring: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub enabled: bool,
    pub prediction_models_path: String,
    pub training_data_retention_days: u32,
    pub prediction_horizon_days: u32,
    pub anomaly_detection_enabled: bool,
    pub machine_learning_enabled: bool,
    pub time_series_analysis: bool,
    pub model_retraining_interval_hours: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalTwinConfig {
    pub enabled: bool,
    pub simulation_engine: String,
    pub update_interval_minutes: u32,
    pub physics_simulation: bool,
    pub thermal_modeling: bool,
    pub capacity_modeling: bool,
    pub optimization_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub enabled: bool,
    pub chain_id: String,
    pub private_key: String,
    pub block_size_limit: usize,
    pub mining_difficulty: u32,
    pub consensus_algorithm: String,
    pub immutable_records: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationConfig {
    pub enabled: bool,
    pub robot_integration: bool,
    pub automated_placement: bool,
    pub automated_retrieval: bool,
    pub scheduling_enabled: bool,
    pub max_concurrent_tasks: u32,
    pub safety_checks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyConfig {
    pub optimization_enabled: bool,
    pub smart_scheduling: bool,
    pub energy_monitoring: bool,
    pub cost_optimization: bool,
    pub renewable_integration: bool,
    pub efficiency_targets: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileConfig {
    pub enabled: bool,
    pub jwt_secret: String,
    pub barcode_scanning: bool,
    pub geolocation_tracking: bool,
    pub offline_support: bool,
    pub push_notifications: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    pub enabled: bool,
    pub regulatory_standards: Vec<String>,
    pub audit_logging: bool,
    pub chain_of_custody: bool,
    pub data_integrity: bool,
    pub access_controls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub default_temperature_zones: Vec<String>,
    pub capacity_warning_threshold: f32,
    pub capacity_critical_threshold: f32,
    pub auto_organization: bool,
    pub barcode_generation: bool,
    pub location_tracking: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            server: ServerConfig {
                host: env::var("STORAGE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("STORAGE_PORT")
                    .unwrap_or_else(|_| "8082".to_string())
                    .parse()
                    .unwrap_or(8082),
                workers: env::var("STORAGE_WORKERS")
                    .unwrap_or_else(|_| "4".to_string())
                    .parse()
                    .unwrap_or(4),
                max_connections: env::var("STORAGE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()
                    .unwrap_or(1000),
                timeout_seconds: env::var("STORAGE_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
            },
            database_url: env::var("STORAGE_DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://storage_user:password@localhost:5432/enhanced_storage_db".to_string()
            }),
            auth_service_url: env::var("AUTH_SERVICE_URL")
                .unwrap_or_else(|_| "http://auth-service:8080".to_string()),
            iot: IoTConfig {
                enabled: env::var("IOT_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                mqtt_broker_url: env::var("MQTT_BROKER_URL")
                    .unwrap_or_else(|_| "mqtt://localhost:1883".to_string()),
                mqtt_username: env::var("MQTT_USERNAME")
                    .unwrap_or_else(|_| "storage_service".to_string()),
                mqtt_password: env::var("MQTT_PASSWORD").unwrap_or_else(|_| "password".to_string()),
                sensor_polling_interval_seconds: env::var("SENSOR_POLLING_INTERVAL")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
                alert_threshold_temperature: env::var("ALERT_THRESHOLD_TEMP")
                    .unwrap_or_else(|_| "2.0".to_string())
                    .parse()
                    .unwrap_or(2.0),
                alert_threshold_humidity: env::var("ALERT_THRESHOLD_HUMIDITY")
                    .unwrap_or_else(|_| "5.0".to_string())
                    .parse()
                    .unwrap_or(5.0),
                calibration_enabled: env::var("SENSOR_CALIBRATION")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                real_time_monitoring: env::var("REAL_TIME_MONITORING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
            analytics: AnalyticsConfig {
                enabled: env::var("ANALYTICS_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                prediction_models_path: env::var("PREDICTION_MODELS_PATH")
                    .unwrap_or_else(|_| "./models".to_string()),
                training_data_retention_days: env::var("TRAINING_DATA_RETENTION")
                    .unwrap_or_else(|_| "365".to_string())
                    .parse()
                    .unwrap_or(365),
                prediction_horizon_days: env::var("PREDICTION_HORIZON")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
                anomaly_detection_enabled: env::var("ANOMALY_DETECTION")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                machine_learning_enabled: env::var("MACHINE_LEARNING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                time_series_analysis: env::var("TIME_SERIES_ANALYSIS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                model_retraining_interval_hours: env::var("MODEL_RETRAINING_INTERVAL")
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()
                    .unwrap_or(24),
            },
            digital_twin: DigitalTwinConfig {
                enabled: env::var("DIGITAL_TWIN_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                simulation_engine: env::var("SIMULATION_ENGINE")
                    .unwrap_or_else(|_| "physics".to_string()),
                update_interval_minutes: env::var("TWIN_UPDATE_INTERVAL")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .unwrap_or(5),
                physics_simulation: env::var("PHYSICS_SIMULATION")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                thermal_modeling: env::var("THERMAL_MODELING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                capacity_modeling: env::var("CAPACITY_MODELING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                optimization_enabled: env::var("TWIN_OPTIMIZATION")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
            blockchain: BlockchainConfig {
                enabled: env::var("BLOCKCHAIN_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                chain_id: env::var("BLOCKCHAIN_CHAIN_ID")
                    .unwrap_or_else(|_| "storage-chain".to_string()),
                private_key: env::var("BLOCKCHAIN_PRIVATE_KEY")
                    .unwrap_or_else(|_| "default-key".to_string()),
                block_size_limit: env::var("BLOCKCHAIN_BLOCK_SIZE")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()
                    .unwrap_or(1000),
                mining_difficulty: env::var("BLOCKCHAIN_DIFFICULTY")
                    .unwrap_or_else(|_| "4".to_string())
                    .parse()
                    .unwrap_or(4),
                consensus_algorithm: env::var("BLOCKCHAIN_CONSENSUS")
                    .unwrap_or_else(|_| "proof_of_authority".to_string()),
                immutable_records: env::var("IMMUTABLE_RECORDS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
            automation: AutomationConfig {
                enabled: env::var("AUTOMATION_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                robot_integration: env::var("ROBOT_INTEGRATION")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                automated_placement: env::var("AUTO_PLACEMENT")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                automated_retrieval: env::var("AUTO_RETRIEVAL")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                scheduling_enabled: env::var("AUTO_SCHEDULING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                max_concurrent_tasks: env::var("MAX_CONCURRENT_TASKS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                safety_checks: env::var("SAFETY_CHECKS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
            energy: EnergyConfig {
                optimization_enabled: env::var("ENERGY_OPTIMIZATION")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                smart_scheduling: env::var("SMART_SCHEDULING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                energy_monitoring: env::var("ENERGY_MONITORING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                cost_optimization: env::var("COST_OPTIMIZATION")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                renewable_integration: env::var("RENEWABLE_INTEGRATION")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                efficiency_targets: env::var("EFFICIENCY_TARGETS")
                    .unwrap_or_else(|_| "0.85".to_string())
                    .parse()
                    .unwrap_or(0.85),
            },
            mobile: MobileConfig {
                enabled: env::var("MOBILE_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                jwt_secret: env::var("MOBILE_JWT_SECRET")
                    .unwrap_or_else(|_| "mobile-storage-secret".to_string()),
                barcode_scanning: env::var("BARCODE_SCANNING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                geolocation_tracking: env::var("GEOLOCATION_TRACKING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                offline_support: env::var("OFFLINE_SUPPORT")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                push_notifications: env::var("PUSH_NOTIFICATIONS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
            compliance: ComplianceConfig {
                enabled: env::var("COMPLIANCE_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                regulatory_standards: env::var("REGULATORY_STANDARDS")
                    .unwrap_or_else(|_| "FDA,ISO,CLIA".to_string())
                    .split(',')
                    .map(|s| s.to_string())
                    .collect(),
                audit_logging: env::var("AUDIT_LOGGING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                chain_of_custody: env::var("CHAIN_OF_CUSTODY")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                data_integrity: env::var("DATA_INTEGRITY")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                access_controls: env::var("ACCESS_CONTROLS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
            storage: StorageConfig {
                default_temperature_zones: env::var("TEMPERATURE_ZONES")
                    .unwrap_or_else(|_| "-80,-20,4,RT,37".to_string())
                    .split(',')
                    .map(|s| s.to_string())
                    .collect(),
                capacity_warning_threshold: env::var("CAPACITY_WARNING")
                    .unwrap_or_else(|_| "0.8".to_string())
                    .parse()
                    .unwrap_or(0.8),
                capacity_critical_threshold: env::var("CAPACITY_CRITICAL")
                    .unwrap_or_else(|_| "0.95".to_string())
                    .parse()
                    .unwrap_or(0.95),
                auto_organization: env::var("AUTO_ORGANIZATION")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                barcode_generation: env::var("BARCODE_GENERATION")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                location_tracking: env::var("LOCATION_TRACKING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
        })
    }

    pub fn validate(&self) -> Result<()> {
        // Validate IoT configuration
        if self.iot.enabled {
            if self.iot.mqtt_broker_url.is_empty() {
                return Err(anyhow::anyhow!(
                    "MQTT broker URL is required when IoT is enabled"
                ));
            }
        }

        // Validate analytics configuration
        if self.analytics.enabled {
            if self.analytics.prediction_models_path.is_empty() {
                return Err(anyhow::anyhow!(
                    "Prediction models path is required when analytics is enabled"
                ));
            }
        }

        // Validate blockchain configuration
        if self.blockchain.enabled {
            if self.blockchain.private_key.is_empty() {
                return Err(anyhow::anyhow!(
                    "Private key is required when blockchain is enabled"
                ));
            }
        }

        Ok(())
    }

    /// Create test configuration for axum-test
    pub fn test_config() -> Self {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 0, // Let OS assign port for tests
                workers: 1,
                max_connections: 10,
                timeout_seconds: 5,
            },
            database_url: std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:postgres@localhost:5432/enhanced_storage_test".to_string()
            }),
            auth_service_url: "http://localhost:8001".to_string(),
            iot: IoTConfig {
                enabled: false, // Disabled for tests
                mqtt_broker_url: "mqtt://localhost:1883".to_string(),
                mqtt_username: "test".to_string(),
                mqtt_password: "test".to_string(),
                sensor_polling_interval_seconds: 60,
                alert_threshold_temperature: 5.0,
                alert_threshold_humidity: 10.0,
                calibration_enabled: false,
                real_time_monitoring: false,
            },
            analytics: AnalyticsConfig {
                enabled: false, // Disabled for tests
                prediction_models_path: "./test_models".to_string(),
                training_data_retention_days: 30,
                prediction_horizon_days: 7,
                anomaly_detection_enabled: false,
                machine_learning_enabled: false,
                time_series_analysis: false,
                model_retraining_interval_hours: 24,
            },
            digital_twin: DigitalTwinConfig {
                enabled: false, // Disabled for tests
                simulation_engine: "test".to_string(),
                update_interval_minutes: 10,
                physics_simulation: false,
                thermal_modeling: false,
                capacity_modeling: false,
                optimization_enabled: false,
            },
            blockchain: BlockchainConfig {
                enabled: false, // Disabled for tests
                chain_id: "test-chain".to_string(),
                private_key: "test-key".to_string(),
                block_size_limit: 100,
                mining_difficulty: 1,
                consensus_algorithm: "test".to_string(),
                immutable_records: false,
            },
            automation: AutomationConfig {
                enabled: false, // Disabled for tests
                robot_integration: false,
                automated_placement: false,
                automated_retrieval: false,
                scheduling_enabled: false,
                max_concurrent_tasks: 1,
                safety_checks: true,
            },
            energy: EnergyConfig {
                optimization_enabled: false,
                smart_scheduling: false,
                energy_monitoring: false,
                cost_optimization: false,
                renewable_integration: false,
                efficiency_targets: 0.5,
            },
            mobile: MobileConfig {
                enabled: false, // Disabled for tests
                jwt_secret: "test-mobile-secret-32-characters".to_string(),
                barcode_scanning: false,
                geolocation_tracking: false,
                offline_support: false,
                push_notifications: false,
            },
            compliance: ComplianceConfig {
                enabled: true, // Keep enabled for tests
                regulatory_standards: vec!["TEST".to_string()],
                audit_logging: true,
                chain_of_custody: true,
                data_integrity: true,
                access_controls: true,
            },
            storage: StorageConfig {
                default_temperature_zones: vec!["RT".to_string(), "4".to_string()],
                capacity_warning_threshold: 0.8,
                capacity_critical_threshold: 0.95,
                auto_organization: false, // Simplified for tests
                barcode_generation: true,
                location_tracking: true,
            },
        }
    }
}
