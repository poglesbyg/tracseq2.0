use config_service::{
    AppState, ConfigEntry, ConfigTemplate, CreateConfigRequest,
    UpdateConfigRequest, ConfigQueryParams, ConfigResponse,
    ServiceConfigResponse, HealthResponse,
};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

/// Factory for creating test configurations
pub struct ConfigFactory;

impl ConfigFactory {
    pub fn create_test_config(service: &str, key: &str, value: serde_json::Value) -> ConfigEntry {
        ConfigEntry {
            id: Uuid::new_v4(),
            service_name: service.to_string(),
            key: key.to_string(),
            value,
            environment: "test".to_string(),
            version: 1,
            encrypted: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec!["test".to_string()],
        }
    }

    pub fn create_auth_service_configs() -> Vec<ConfigEntry> {
        vec![
            Self::create_test_config("auth-service", "jwt_expiry_hours", serde_json::json!(24)),
            Self::create_test_config("auth-service", "max_login_attempts", serde_json::json!(5)),
            Self::create_test_config("auth-service", "session_timeout_minutes", serde_json::json!(30)),
        ]
    }

    pub fn create_storage_service_configs() -> Vec<ConfigEntry> {
        vec![
            Self::create_test_config("storage-service", "temperature_check_interval", serde_json::json!(5)),
            Self::create_test_config("storage-service", "capacity_warning_threshold", serde_json::json!(0.8)),
            Self::create_test_config("storage-service", "ai_optimization_enabled", serde_json::json!(true)),
        ]
    }

    pub fn create_config_with_tags(service: &str, key: &str, tags: Vec<String>) -> ConfigEntry {
        let mut config = Self::create_test_config(service, key, serde_json::json!("test_value"));
        config.tags = tags;
        config
    }

    pub fn create_config_with_environment(service: &str, key: &str, env: &str) -> ConfigEntry {
        let mut config = Self::create_test_config(service, key, serde_json::json!("test_value"));
        config.environment = env.to_string();
        config
    }
}

/// Factory for creating test requests
pub struct RequestFactory;

impl RequestFactory {
    pub fn create_config_request(service: &str, key: &str, value: serde_json::Value) -> CreateConfigRequest {
        CreateConfigRequest {
            service_name: service.to_string(),
            key: key.to_string(),
            value,
            environment: Some("test".to_string()),
            encrypted: Some(false),
            tags: Some(vec!["test".to_string()]),
        }
    }

    pub fn update_config_request(value: serde_json::Value) -> UpdateConfigRequest {
        UpdateConfigRequest {
            value,
            tags: None,
        }
    }

    pub fn update_config_request_with_tags(value: serde_json::Value, tags: Vec<String>) -> UpdateConfigRequest {
        UpdateConfigRequest {
            value,
            tags: Some(tags),
        }
    }
}

/// Test data generators
pub struct TestDataGenerator;

impl TestDataGenerator {
    pub fn generate_bulk_configs(service: &str, count: usize) -> HashMap<String, serde_json::Value> {
        let mut configs = HashMap::new();
        
        for i in 0..count {
            configs.insert(
                format!("test_key_{}", i),
                serde_json::json!(format!("test_value_{}", i))
            );
        }
        
        configs
    }

    pub fn generate_complex_config_value() -> serde_json::Value {
        serde_json::json!({
            "database": {
                "host": "localhost",
                "port": 5432,
                "pool_size": 10
            },
            "features": {
                "enable_cache": true,
                "cache_ttl": 300,
                "enable_logging": true
            },
            "limits": {
                "max_requests": 1000,
                "timeout_seconds": 30
            }
        })
    }

    pub fn generate_encrypted_config_value() -> serde_json::Value {
        serde_json::json!({
            "encrypted": true,
            "value": "base64_encrypted_string_here",
            "algorithm": "AES-256-GCM"
        })
    }
}

/// Assertions for config service tests
pub struct ConfigAssertions;

impl ConfigAssertions {
    pub fn assert_config_equal(actual: &ConfigEntry, expected: &ConfigEntry) {
        assert_eq!(actual.service_name, expected.service_name);
        assert_eq!(actual.key, expected.key);
        assert_eq!(actual.value, expected.value);
        assert_eq!(actual.environment, expected.environment);
        assert_eq!(actual.encrypted, expected.encrypted);
        assert_eq!(actual.tags, expected.tags);
    }

    pub fn assert_config_version_incremented(updated: &ConfigEntry, original: &ConfigEntry) {
        assert_eq!(updated.version, original.version + 1);
        assert!(updated.updated_at > original.updated_at);
    }

    pub fn assert_service_config_response(response: &ServiceConfigResponse, service: &str, env: &str) {
        assert_eq!(response.service_name, service);
        assert_eq!(response.environment, env);
        assert!(!response.config.is_empty());
    }

    pub fn assert_config_in_response(response: &ConfigResponse, config: &ConfigEntry) {
        let found = response.configs.iter()
            .any(|c| c.service_name == config.service_name && c.key == config.key);
        assert!(found, "Config {}/{} not found in response", config.service_name, config.key);
    }
}

/// Performance test utilities
pub struct PerformanceTestUtils;

impl PerformanceTestUtils {
    pub async fn measure_config_operation<F, Fut, T>(operation: F) -> (std::time::Duration, T)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let start = std::time::Instant::now();
        let result = operation().await;
        (start.elapsed(), result)
    }

    pub async fn benchmark_concurrent_reads(state: &AppState, read_count: usize) -> std::time::Duration {
        let start = std::time::Instant::now();
        let mut handles = Vec::new();

        for _ in 0..read_count {
            let state_clone = state.clone();
            let handle = tokio::spawn(async move {
                let _ = state_clone.config_store.read().await;
            });
            handles.push(handle);
        }

        futures::future::join_all(handles).await;
        start.elapsed()
    }

    pub async fn benchmark_concurrent_writes(state: &AppState, write_count: usize) -> std::time::Duration {
        let start = std::time::Instant::now();
        let mut handles = Vec::new();

        for i in 0..write_count {
            let state_clone = state.clone();
            let config = ConfigFactory::create_test_config(
                "perf-test",
                &format!("key_{}", i),
                serde_json::json!(i)
            );
            
            let handle = tokio::spawn(async move {
                let mut store = state_clone.config_store.write().await;
                store.insert(format!("perf-test-key_{}", i), config);
            });
            handles.push(handle);
        }

        futures::future::join_all(handles).await;
        start.elapsed()
    }
}

/// Helper to create test app state
pub fn create_test_app_state() -> AppState {
    AppState::new()
}

/// Helper to create test app state with initial configs
pub async fn create_app_state_with_configs(configs: Vec<ConfigEntry>) -> AppState {
    let state = AppState::new();
    let mut store = state.config_store.write().await;
    
    for config in configs {
        let key = format!("{}-{}", config.service_name, config.key);
        store.insert(key, config);
    }
    
    state
}

/// Mock template builder for testing
pub struct TemplateBuilder {
    template: ConfigTemplate,
}

impl TemplateBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            template: ConfigTemplate {
                id: Uuid::new_v4(),
                name: name.to_string(),
                description: "Test template".to_string(),
                template: HashMap::new(),
                required_keys: vec![],
                optional_keys: vec![],
            },
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.template.description = desc.to_string();
        self
    }

    pub fn with_required_key(mut self, key: &str, default_value: serde_json::Value) -> Self {
        self.template.template.insert(key.to_string(), default_value);
        self.template.required_keys.push(key.to_string());
        self
    }

    pub fn with_optional_key(mut self, key: &str, default_value: serde_json::Value) -> Self {
        self.template.template.insert(key.to_string(), default_value);
        self.template.optional_keys.push(key.to_string());
        self
    }

    pub fn build(self) -> ConfigTemplate {
        self.template
    }
}

/// Test macro for async config tests
#[macro_export]
macro_rules! config_test {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let state = crate::test_utils::create_test_app_state();
            $test_body(state).await;
        }
    };
}