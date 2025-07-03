use axum::{
    body::Bytes,
    extract::{Path, Query},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{error, info};

use crate::services::proxy_service::{SERVICE_PROXY, ServiceHealth};

/// Health check response for all microservices
#[derive(Debug, Serialize)]
pub struct MicroservicesHealthResponse {
    pub status: String,
    pub services: HashMap<String, ServiceHealthInfo>,
}

#[derive(Debug, Serialize)]
pub struct ServiceHealthInfo {
    pub status: String,
    pub url: String,
}

/// Check health of all microservices
pub async fn check_microservices_health() -> impl IntoResponse {
    let health_statuses = SERVICE_PROXY.check_all_services_health().await;
    
    let mut services = HashMap::new();
    let mut overall_healthy = true;
    
    for (key, health) in health_statuses {
        let status = match health {
            ServiceHealth::Healthy => "healthy",
            ServiceHealth::Unhealthy => {
                overall_healthy = false;
                "unhealthy"
            }
            ServiceHealth::Unknown => {
                overall_healthy = false;
                "unknown"
            }
        };
        
        let config = SERVICE_PROXY.get_service_config(&key);
        let url = config.map(|c| c.url.clone()).unwrap_or_default();
        
        services.insert(
            key,
            ServiceHealthInfo {
                status: status.to_string(),
                url,
            },
        );
    }
    
    let response = MicroservicesHealthResponse {
        status: if overall_healthy { "healthy" } else { "degraded" }.to_string(),
        services,
    };
    
    Json(response)
}

/// List all registered microservices
pub async fn list_microservices() -> impl IntoResponse {
    let services = SERVICE_PROXY.list_services();
    let mut service_list = Vec::new();
    
    for key in services {
        if let Some(config) = SERVICE_PROXY.get_service_config(&key) {
            service_list.push(serde_json::json!({
                "key": key,
                "name": config.name,
                "url": config.url,
                "health_check_path": config.health_check_path,
            }));
        }
    }
    
    Json(serde_json::json!({
        "services": service_list,
        "count": service_list.len(),
    }))
}

/// Generic proxy handler for authentication endpoints
pub async fn proxy_auth_request(
    method: Method,
    path: Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, StatusCode> {
    let path_str = format!("/api/auth/{}", path.0);
    
    SERVICE_PROXY
        .proxy_request("auth", method, &path_str, headers, Some(body))
        .await
        .map_err(|e| {
            error!("Auth proxy error: {}", e);
            StatusCode::BAD_GATEWAY
        })
}

/// Proxy login request
pub async fn proxy_login(
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<Response, StatusCode> {
    let body = serde_json::to_vec(&payload)
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .into();
    
    SERVICE_PROXY
        .proxy_request("auth", Method::POST, "/api/auth/login", headers, Some(body))
        .await
        .map_err(|e| {
            error!("Login proxy error: {}", e);
            StatusCode::BAD_GATEWAY
        })
}

/// Proxy logout request
pub async fn proxy_logout(
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    SERVICE_PROXY
        .proxy_request("auth", Method::POST, "/api/auth/logout", headers, None)
        .await
        .map_err(|e| {
            error!("Logout proxy error: {}", e);
            StatusCode::BAD_GATEWAY
        })
}

/// Generic proxy handler for sample service endpoints
pub async fn proxy_sample_request(
    method: Method,
    path: Path<String>,
    headers: HeaderMap,
    body: Option<Bytes>,
) -> Result<Response, StatusCode> {
    let path_str = format!("/api/samples/{}", path.0);
    
    SERVICE_PROXY
        .proxy_request("sample", method, &path_str, headers, body)
        .await
        .map_err(|e| {
            error!("Sample proxy error: {}", e);
            StatusCode::BAD_GATEWAY
        })
}

/// Proxy create sample request
pub async fn proxy_create_sample(
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<Response, StatusCode> {
    let body = serde_json::to_vec(&payload)
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .into();
    
    SERVICE_PROXY
        .proxy_request("sample", Method::POST, "/api/samples", headers, Some(body))
        .await
        .map_err(|e| {
            error!("Create sample proxy error: {}", e);
            StatusCode::BAD_GATEWAY
        })
}

/// Proxy list samples request
pub async fn proxy_list_samples(
    headers: HeaderMap,
    query: Query<HashMap<String, String>>,
) -> Result<Response, StatusCode> {
    let query_string = serde_urlencoded::to_string(&query.0)
        .unwrap_or_default();
    let path = if query_string.is_empty() {
        "/api/samples".to_string()
    } else {
        format!("/api/samples?{}", query_string)
    };
    
    SERVICE_PROXY
        .proxy_request("sample", Method::GET, &path, headers, None)
        .await
        .map_err(|e| {
            error!("List samples proxy error: {}", e);
            StatusCode::BAD_GATEWAY
        })
}

/// Generic proxy handler for sequencing service endpoints
pub async fn proxy_sequencing_request(
    method: Method,
    path: Path<String>,
    headers: HeaderMap,
    body: Option<Bytes>,
) -> Result<Response, StatusCode> {
    let path_str = format!("/api/sequencing/{}", path.0);
    
    SERVICE_PROXY
        .proxy_request("sequencing", method, &path_str, headers, body)
        .await
        .map_err(|e| {
            error!("Sequencing proxy error: {}", e);
            StatusCode::BAD_GATEWAY
        })
}

/// Generic proxy handler for template service endpoints
pub async fn proxy_template_request(
    method: Method,
    path: Path<String>,
    headers: HeaderMap,
    body: Option<Bytes>,
) -> Result<Response, StatusCode> {
    // Template service expects paths without /api prefix
    let path_str = format!("/templates/{}", path.0);
    
    SERVICE_PROXY
        .proxy_request("template", method, &path_str, headers, body)
        .await
        .map_err(|e| {
            error!("Template proxy error: {}", e);
            StatusCode::BAD_GATEWAY
        })
}

/// Generic proxy handler for storage service endpoints
pub async fn proxy_storage_request(
    method: Method,
    path: Path<String>,
    headers: HeaderMap,
    body: Option<Bytes>,
) -> Result<Response, StatusCode> {
    let path_str = format!("/api/storage/{}", path.0);
    
    SERVICE_PROXY
        .proxy_request("storage", method, &path_str, headers, body)
        .await
        .map_err(|e| {
            error!("Storage proxy error: {}", e);
            StatusCode::BAD_GATEWAY
        })
}

/// Generic proxy handler for spreadsheet service endpoints
pub async fn proxy_spreadsheet_request(
    method: Method,
    path: Path<String>,
    headers: HeaderMap,
    body: Option<Bytes>,
) -> Result<Response, StatusCode> {
    let path_str = format!("/api/spreadsheets/{}", path.0);
    
    SERVICE_PROXY
        .proxy_request("spreadsheet", method, &path_str, headers, body)
        .await
        .map_err(|e| {
            error!("Spreadsheet proxy error: {}", e);
            StatusCode::BAD_GATEWAY
        })
}

/// Generic proxy handler for notifications service endpoints
pub async fn proxy_notifications_request(
    method: Method,
    path: Path<String>,
    headers: HeaderMap,
    body: Option<Bytes>,
) -> Result<Response, StatusCode> {
    let path_str = format!("/api/notifications/{}", path.0);
    
    SERVICE_PROXY
        .proxy_request("notifications", method, &path_str, headers, body)
        .await
        .map_err(|e| {
            error!("Notifications proxy error: {}", e);
            StatusCode::BAD_GATEWAY
        })
}

/// Generic proxy handler for events service endpoints
pub async fn proxy_events_request(
    method: Method,
    path: Path<String>,
    headers: HeaderMap,
    body: Option<Bytes>,
) -> Result<Response, StatusCode> {
    let path_str = format!("/api/events/{}", path.0);
    
    SERVICE_PROXY
        .proxy_request("events", method, &path_str, headers, body)
        .await
        .map_err(|e| {
            error!("Events proxy error: {}", e);
            StatusCode::BAD_GATEWAY
        })
}

/// Generic proxy handler for transactions service endpoints
pub async fn proxy_transactions_request(
    method: Method,
    path: Path<String>,
    headers: HeaderMap,
    body: Option<Bytes>,
) -> Result<Response, StatusCode> {
    let path_str = format!("/api/transactions/{}", path.0);
    
    SERVICE_PROXY
        .proxy_request("transactions", method, &path_str, headers, body)
        .await
        .map_err(|e| {
            error!("Transactions proxy error: {}", e);
            StatusCode::BAD_GATEWAY
        })
}

/// Generic proxy handler for qaqc service endpoints
pub async fn proxy_qaqc_request(
    method: Method,
    path: Path<String>,
    headers: HeaderMap,
    body: Option<Bytes>,
) -> Result<Response, StatusCode> {
    let path_str = format!("/api/qaqc/{}", path.0);
    
    SERVICE_PROXY
        .proxy_request("qaqc", method, &path_str, headers, body)
        .await
        .map_err(|e| {
            error!("QAQC proxy error: {}", e);
            StatusCode::BAD_GATEWAY
        })
}

/// Initialize the service proxy with circuit breakers
pub async fn initialize_proxy() {
    info!("Initializing service proxy with circuit breakers");
    SERVICE_PROXY.initialize_circuit_breakers().await;
    info!("Service proxy initialized successfully");
}

/// Feature flag check for proxy mode
pub fn is_proxy_mode_enabled() -> bool {
    std::env::var("ENABLE_PROXY_MODE")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false)
}

/// Service discovery endpoint
pub async fn service_discovery() -> impl IntoResponse {
    let services = SERVICE_PROXY.list_services();
    let mut discovery_info = Vec::new();
    
    for key in services {
        if let Some(config) = SERVICE_PROXY.get_service_config(&key) {
            let health = SERVICE_PROXY.check_service_health(&key).await;
            discovery_info.push(serde_json::json!({
                "service": key,
                "name": config.name,
                "url": config.url,
                "health_check": config.health_check_path,
                "status": match health {
                    ServiceHealth::Healthy => "healthy",
                    ServiceHealth::Unhealthy => "unhealthy",
                    ServiceHealth::Unknown => "unknown",
                },
                "timeout_seconds": config.timeout_seconds,
                "retry_attempts": config.retry_attempts,
            }));
        }
    }
    
    Json(serde_json::json!({
        "timestamp": chrono::Utc::now(),
        "services": discovery_info,
        "proxy_mode": is_proxy_mode_enabled(),
        "total_services": discovery_info.len(),
    }))
}