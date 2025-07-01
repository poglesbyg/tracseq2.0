use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn, error};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigEntry {
    pub id: Uuid,
    pub service_name: String,
    pub key: String,
    pub value: serde_json::Value,
    pub environment: String,
    pub version: u32,
    pub encrypted: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub template: HashMap<String, serde_json::Value>,
    pub required_keys: Vec<String>,
    pub optional_keys: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateConfigRequest {
    pub service_name: String,
    pub key: String,
    pub value: serde_json::Value,
    pub environment: Option<String>,
    pub encrypted: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateConfigRequest {
    pub value: serde_json::Value,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigQueryParams {
    pub environment: Option<String>,
    pub tags: Option<String>,
    pub version: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub configs: Vec<ConfigEntry>,
    pub total: usize,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ServiceConfigResponse {
    pub service_name: String,
    pub environment: String,
    pub config: HashMap<String, serde_json::Value>,
    pub version: u32,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub config_count: usize,
}

pub type ConfigStore = Arc<RwLock<HashMap<String, ConfigEntry>>>;
pub type TemplateStore = Arc<RwLock<HashMap<String, ConfigTemplate>>>;

#[derive(Clone)]
pub struct AppState {
    pub config_store: ConfigStore,
    pub template_store: TemplateStore,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config_store: Arc::new(RwLock::new(HashMap::new())),
            template_store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn initialize_default_configs(&self) {
        let mut store = self.config_store.write().await;
        
        // Default configurations for TracSeq microservices
        let default_configs = vec![
            // Auth Service Defaults
            ("auth-service", "jwt_expiry_hours", serde_json::json!(24)),
            ("auth-service", "max_login_attempts", serde_json::json!(5)),
            ("auth-service", "session_timeout_minutes", serde_json::json!(30)),
            ("auth-service", "enable_2fa", serde_json::json!(false)),
            
            // Sample Service Defaults
            ("sample-service", "max_batch_size", serde_json::json!(1000)),
            ("sample-service", "auto_approve_threshold", serde_json::json!(0.9)),
            ("sample-service", "default_storage_temp", serde_json::json!(-80)),
            ("sample-service", "enable_barcode_validation", serde_json::json!(true)),
            
            // Enhanced Storage Defaults
            ("enhanced-storage-service", "temperature_check_interval_minutes", serde_json::json!(5)),
            ("enhanced-storage-service", "capacity_warning_threshold", serde_json::json!(0.8)),
            ("enhanced-storage-service", "capacity_critical_threshold", serde_json::json!(0.95)),
            ("enhanced-storage-service", "ai_optimization_enabled", serde_json::json!(true)),
            
            // RAG Service Defaults
            ("enhanced-rag-service", "default_model", serde_json::json!("llama3.2:3b")),
            ("enhanced-rag-service", "confidence_threshold", serde_json::json!(0.7)),
            ("enhanced-rag-service", "max_document_size_mb", serde_json::json!(50)),
            ("enhanced-rag-service", "enable_caching", serde_json::json!(true)),
            
            // Notification Service Defaults
            ("notification-service", "email_enabled", serde_json::json!(true)),
            ("notification-service", "sms_enabled", serde_json::json!(false)),
            ("notification-service", "slack_enabled", serde_json::json!(true)),
            ("notification-service", "max_retry_attempts", serde_json::json!(3)),
            
            // Event Service Defaults
            ("event-service", "max_event_retention_days", serde_json::json!(30)),
            ("event-service", "batch_processing_size", serde_json::json!(100)),
            ("event-service", "enable_event_replay", serde_json::json!(true)),
            
            // API Gateway Defaults
            ("api-gateway", "rate_limit_per_minute", serde_json::json!(1000)),
            ("api-gateway", "enable_cors", serde_json::json!(true)),
            ("api-gateway", "request_timeout_seconds", serde_json::json!(30)),
            ("api-gateway", "enable_request_logging", serde_json::json!(true)),
        ];

        for (service, key, value) in default_configs {
            let config_id = format!("{}-{}", service, key);
            let config = ConfigEntry {
                id: Uuid::new_v4(),
                service_name: service.to_string(),
                key: key.to_string(),
                value,
                environment: "default".to_string(),
                version: 1,
                encrypted: false,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                tags: vec!["default".to_string()],
            };
            store.insert(config_id, config);
        }

        info!("Initialized {} default configurations", store.len());
    }
}

// Re-export handler functions for testing
pub mod handlers {
    use super::*;

    pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
        let config_count = state.config_store.read().await.len();
        
        Json(HealthResponse {
            status: "healthy".to_string(),
            timestamp: chrono::Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            config_count,
        })
    }

    pub async fn get_configs(
        Query(params): Query<ConfigQueryParams>,
        State(state): State<AppState>,
    ) -> Json<ConfigResponse> {
        let store = state.config_store.read().await;
        let mut configs: Vec<ConfigEntry> = store.values().cloned().collect();

        // Apply filters
        if let Some(env) = &params.environment {
            configs.retain(|c| c.environment == *env);
        }

        if let Some(tags_str) = &params.tags {
            let tags: Vec<String> = tags_str.split(',').map(|s| s.trim().to_string()).collect();
            configs.retain(|c| tags.iter().any(|tag| c.tags.contains(tag)));
        }

        if let Some(version) = params.version {
            configs.retain(|c| c.version == version);
        }

        let total = configs.len();
        
        Json(ConfigResponse { configs, total })
    }

    pub async fn get_service_config(
        Path((service_name, environment)): Path<(String, String)>,
        State(state): State<AppState>,
    ) -> Result<Json<ServiceConfigResponse>, StatusCode> {
        let store = state.config_store.read().await;
        
        let service_configs: Vec<&ConfigEntry> = store
            .values()
            .filter(|c| c.service_name == service_name && c.environment == environment)
            .collect();

        if service_configs.is_empty() {
            return Err(StatusCode::NOT_FOUND);
        }

        let mut config_map = HashMap::new();
        let mut max_version = 0;
        let mut last_updated = chrono::DateTime::<chrono::Utc>::MIN_UTC;

        for config in service_configs {
            config_map.insert(config.key.clone(), config.value.clone());
            max_version = max_version.max(config.version);
            last_updated = last_updated.max(config.updated_at);
        }

        Ok(Json(ServiceConfigResponse {
            service_name,
            environment,
            config: config_map,
            version: max_version,
            last_updated,
        }))
    }

    pub async fn create_config(
        State(state): State<AppState>,
        Json(request): Json<CreateConfigRequest>,
    ) -> Result<Json<ConfigEntry>, StatusCode> {
        let mut store = state.config_store.write().await;
        
        let config_id = format!("{}-{}", request.service_name, request.key);
        
        // Check if config already exists
        if store.contains_key(&config_id) {
            return Err(StatusCode::CONFLICT);
        }

        let config = ConfigEntry {
            id: Uuid::new_v4(),
            service_name: request.service_name,
            key: request.key,
            value: request.value,
            environment: request.environment.unwrap_or_else(|| "default".to_string()),
            version: 1,
            encrypted: request.encrypted.unwrap_or(false),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            tags: request.tags.unwrap_or_else(|| vec!["user-created".to_string()]),
        };

        store.insert(config_id, config.clone());
        info!("Created configuration: {}/{}", config.service_name, config.key);

        Ok(Json(config))
    }

    pub async fn update_config(
        Path((service_name, key)): Path<(String, String)>,
        State(state): State<AppState>,
        Json(request): Json<UpdateConfigRequest>,
    ) -> Result<Json<ConfigEntry>, StatusCode> {
        let mut store = state.config_store.write().await;
        
        let config_id = format!("{}-{}", service_name, key);
        
        match store.get_mut(&config_id) {
            Some(config) => {
                config.value = request.value;
                config.version += 1;
                config.updated_at = chrono::Utc::now();
                
                if let Some(tags) = request.tags {
                    config.tags = tags;
                }

                info!("Updated configuration: {}/{} (v{})", config.service_name, config.key, config.version);
                Ok(Json(config.clone()))
            },
            None => Err(StatusCode::NOT_FOUND),
        }
    }

    pub async fn delete_config(
        Path((service_name, key)): Path<(String, String)>,
        State(state): State<AppState>,
    ) -> Result<StatusCode, StatusCode> {
        let mut store = state.config_store.write().await;
        
        let config_id = format!("{}-{}", service_name, key);
        
        match store.remove(&config_id) {
            Some(_) => {
                info!("Deleted configuration: {}/{}", service_name, key);
                Ok(StatusCode::NO_CONTENT)
            },
            None => Err(StatusCode::NOT_FOUND),
        }
    }

    pub async fn bulk_update_service_config(
        Path((service_name, environment)): Path<(String, String)>,
        State(state): State<AppState>,
        Json(configs): Json<HashMap<String, serde_json::Value>>,
    ) -> Result<Json<ServiceConfigResponse>, StatusCode> {
        let mut store = state.config_store.write().await;
        
        let mut updated_configs = Vec::new();
        let now = chrono::Utc::now();

        for (key, value) in configs {
            let config_id = format!("{}-{}", service_name, key);
            
            match store.get_mut(&config_id) {
                Some(config) => {
                    // Update existing config
                    config.value = value;
                    config.version += 1;
                    config.updated_at = now;
                    updated_configs.push(config.clone());
                },
                None => {
                    // Create new config
                    let config = ConfigEntry {
                        id: Uuid::new_v4(),
                        service_name: service_name.clone(),
                        key: key.clone(),
                        value,
                        environment: environment.clone(),
                        version: 1,
                        encrypted: false,
                        created_at: now,
                        updated_at: now,
                        tags: vec!["bulk-update".to_string()],
                    };
                    store.insert(config_id, config.clone());
                    updated_configs.push(config);
                }
            }
        }

        let config_map: HashMap<String, serde_json::Value> = updated_configs
            .iter()
            .map(|c| (c.key.clone(), c.value.clone()))
            .collect();

        let max_version = updated_configs.iter().map(|c| c.version).max().unwrap_or(1);

        info!("Bulk updated {} configurations for service: {}", updated_configs.len(), service_name);

        Ok(Json(ServiceConfigResponse {
            service_name,
            environment,
            config: config_map,
            version: max_version,
            last_updated: now,
        }))
    }
}

pub fn create_app() -> Router {
    let state = AppState::new();

    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/configs", get(handlers::get_configs).post(handlers::create_config))
        .route("/configs/:service_name/:key", 
            put(handlers::update_config).delete(handlers::delete_config))
        .route("/configs/:service_name/:environment", get(handlers::get_service_config))
        .route("/configs/:service_name/:environment/bulk", put(handlers::bulk_update_service_config))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(state)
}