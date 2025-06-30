use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tower_http::cors::CorsLayer;
use tracing::{info, warn};
use uuid::Uuid;

mod config;
mod handlers;
mod models;
mod services;

use crate::config::Settings;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub http_client: reqwest::Client,
    pub settings: Settings,
    pub cache: moka::future::Cache<String, DashboardData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardData {
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
    pub ttl_seconds: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("dashboard_service=debug,tower_http=debug,axum=debug")
        .init();

    info!("Starting Dashboard Service");

    // Load configuration
    let settings = Settings::new()?;

    // Create database pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&settings.database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Create HTTP client for calling other services
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    // Create cache with 5 minute TTL
    let cache = moka::future::Cache::builder()
        .time_to_live(Duration::from_secs(300))
        .max_capacity(1000)
        .build();

    // Create app state
    let state = Arc::new(AppState {
        pool,
        http_client,
        settings: settings.clone(),
        cache,
    });

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        // Dashboard endpoints
        .route("/api/dashboard", get(handlers::dashboard::get_main_dashboard))
        .route("/api/dashboard/metrics", get(handlers::metrics::get_system_metrics))
        .route("/api/dashboard/kpis", get(handlers::kpis::get_kpis))
        .route("/api/dashboard/services", get(handlers::services::get_service_status))
        .route("/api/dashboard/analytics", get(handlers::analytics::get_analytics))
        .route("/api/dashboard/performance", get(handlers::performance::get_performance_metrics))
        .route("/api/dashboard/alerts", get(handlers::alerts::get_active_alerts))
        .route("/api/dashboard/usage", get(handlers::usage::get_usage_stats))
        // Custom dashboard endpoints
        .route("/api/dashboard/custom", post(handlers::custom::create_custom_dashboard))
        .route("/api/dashboard/custom/:id", get(handlers::custom::get_custom_dashboard))
        .route("/api/dashboard/widgets", get(handlers::widgets::list_available_widgets))
        // Lab-specific dashboards
        .route("/api/dashboard/lab/samples", get(handlers::lab::get_sample_metrics))
        .route("/api/dashboard/lab/sequencing", get(handlers::lab::get_sequencing_metrics))
        .route("/api/dashboard/lab/storage", get(handlers::lab::get_storage_metrics))
        .route("/api/dashboard/lab/throughput", get(handlers::lab::get_throughput_metrics))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], settings.port));
    info!("Dashboard Service listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "dashboard-service",
        "timestamp": Utc::now()
    }))
}

// Stub modules - will be implemented next
mod config {
    use serde::{Deserialize, Serialize};
    use config::{Config, ConfigError, Environment, File};

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Settings {
        pub database_url: String,
        pub port: u16,
        pub service_urls: ServiceUrls,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct ServiceUrls {
        pub auth_service: String,
        pub sample_service: String,
        pub storage_service: String,
        pub sequencing_service: String,
        pub notification_service: String,
        pub rag_service: String,
        pub barcode_service: String,
        pub qaqc_service: String,
        pub library_service: String,
        pub event_service: String,
        pub transaction_service: String,
        pub spreadsheet_service: String,
    }

    impl Settings {
        pub fn new() -> Result<Self, ConfigError> {
            let s = Config::builder()
                .set_default("port", 3025)?
                .set_default("database_url", "postgresql://dashboard_user:dashboard_pass@postgres:5432/tracseq_dashboard")?
                .set_default("service_urls.auth_service", "http://auth-service:8080")?
                .set_default("service_urls.sample_service", "http://sample-service:8081")?
                .set_default("service_urls.storage_service", "http://enhanced-storage-service:8082")?
                .set_default("service_urls.sequencing_service", "http://sequencing-service:8084")?
                .set_default("service_urls.notification_service", "http://notification-service:8085")?
                .set_default("service_urls.rag_service", "http://enhanced-rag-service:8086")?
                .set_default("service_urls.barcode_service", "http://barcode-service:3020")?
                .set_default("service_urls.qaqc_service", "http://qaqc-service:3018")?
                .set_default("service_urls.library_service", "http://library-details-service:3021")?
                .set_default("service_urls.event_service", "http://event-service:3017")?
                .set_default("service_urls.transaction_service", "http://transaction-service:8088")?
                .set_default("service_urls.spreadsheet_service", "http://spreadsheet-versioning-service:3015")?
                .add_source(Environment::with_prefix("DASHBOARD"))
                .build()?;

            s.try_deserialize()
        }
    }
}

mod handlers {
    pub mod dashboard {
        use axum::{extract::State, Json};
        use std::sync::Arc;
        use crate::AppState;

        pub async fn get_main_dashboard(
            State(state): State<Arc<AppState>>,
        ) -> Json<serde_json::Value> {
            // Aggregate data from multiple services
            Json(serde_json::json!({
                "dashboard": "main",
                "sections": {
                    "overview": {
                        "total_samples": 1234,
                        "active_sequencing": 56,
                        "storage_utilization": 78.5,
                        "system_health": "operational"
                    },
                    "recent_activity": [],
                    "alerts": []
                }
            }))
        }
    }

    pub mod metrics {
        use axum::{extract::State, Json};
        use std::sync::Arc;
        use crate::AppState;

        pub async fn get_system_metrics(
            State(state): State<Arc<AppState>>,
        ) -> Json<serde_json::Value> {
            Json(serde_json::json!({
                "metrics": {
                    "cpu_usage": 45.2,
                    "memory_usage": 62.8,
                    "disk_usage": 71.3,
                    "network_io": {
                        "incoming": 123456,
                        "outgoing": 654321
                    }
                }
            }))
        }
    }

    pub mod kpis {
        use axum::{extract::State, Json};
        use std::sync::Arc;
        use crate::AppState;

        pub async fn get_kpis(
            State(state): State<Arc<AppState>>,
        ) -> Json<serde_json::Value> {
            Json(serde_json::json!({
                "kpis": {
                    "sample_throughput": {
                        "daily": 234,
                        "weekly": 1567,
                        "monthly": 6234
                    },
                    "turnaround_time": {
                        "average_hours": 48.5,
                        "median_hours": 42.0
                    },
                    "success_rate": 97.8,
                    "cost_per_sample": 125.50
                }
            }))
        }
    }

    pub mod services {
        use axum::{extract::State, Json};
        use std::sync::Arc;
        use crate::AppState;

        pub async fn get_service_status(
            State(state): State<Arc<AppState>>,
        ) -> Json<serde_json::Value> {
            Json(serde_json::json!({
                "services": {
                    "auth": "healthy",
                    "sample": "healthy",
                    "storage": "healthy",
                    "sequencing": "healthy",
                    "notification": "healthy",
                    "rag": "healthy"
                }
            }))
        }
    }

    pub mod analytics {
        use axum::{extract::State, Json};
        use std::sync::Arc;
        use crate::AppState;

        pub async fn get_analytics(
            State(state): State<Arc<AppState>>,
        ) -> Json<serde_json::Value> {
            Json(serde_json::json!({
                "analytics": {}
            }))
        }
    }

    pub mod performance {
        use axum::{extract::State, Json};
        use std::sync::Arc;
        use crate::AppState;

        pub async fn get_performance_metrics(
            State(state): State<Arc<AppState>>,
        ) -> Json<serde_json::Value> {
            Json(serde_json::json!({
                "performance": {}
            }))
        }
    }

    pub mod alerts {
        use axum::{extract::State, Json};
        use std::sync::Arc;
        use crate::AppState;

        pub async fn get_active_alerts(
            State(state): State<Arc<AppState>>,
        ) -> Json<serde_json::Value> {
            Json(serde_json::json!({
                "alerts": []
            }))
        }
    }

    pub mod usage {
        use axum::{extract::State, Json};
        use std::sync::Arc;
        use crate::AppState;

        pub async fn get_usage_stats(
            State(state): State<Arc<AppState>>,
        ) -> Json<serde_json::Value> {
            Json(serde_json::json!({
                "usage": {}
            }))
        }
    }

    pub mod custom {
        use axum::{extract::{State, Path}, Json};
        use std::sync::Arc;
        use crate::AppState;

        pub async fn create_custom_dashboard(
            State(state): State<Arc<AppState>>,
            Json(payload): Json<serde_json::Value>,
        ) -> Json<serde_json::Value> {
            Json(serde_json::json!({
                "id": "custom-123",
                "created": true
            }))
        }

        pub async fn get_custom_dashboard(
            State(state): State<Arc<AppState>>,
            Path(id): Path<String>,
        ) -> Json<serde_json::Value> {
            Json(serde_json::json!({
                "id": id,
                "dashboard": {}
            }))
        }
    }

    pub mod widgets {
        use axum::{extract::State, Json};
        use std::sync::Arc;
        use crate::AppState;

        pub async fn list_available_widgets(
            State(state): State<Arc<AppState>>,
        ) -> Json<serde_json::Value> {
            Json(serde_json::json!({
                "widgets": []
            }))
        }
    }

    pub mod lab {
        use axum::{extract::State, Json};
        use std::sync::Arc;
        use crate::AppState;

        pub async fn get_sample_metrics(
            State(state): State<Arc<AppState>>,
        ) -> Json<serde_json::Value> {
            Json(serde_json::json!({
                "sample_metrics": {}
            }))
        }

        pub async fn get_sequencing_metrics(
            State(state): State<Arc<AppState>>,
        ) -> Json<serde_json::Value> {
            Json(serde_json::json!({
                "sequencing_metrics": {}
            }))
        }

        pub async fn get_storage_metrics(
            State(state): State<Arc<AppState>>,
        ) -> Json<serde_json::Value> {
            Json(serde_json::json!({
                "storage_metrics": {}
            }))
        }

        pub async fn get_throughput_metrics(
            State(state): State<Arc<AppState>>,
        ) -> Json<serde_json::Value> {
            Json(serde_json::json!({
                "throughput_metrics": {}
            }))
        }
    }
}

mod models {
    // Dashboard models will be added here
}

mod services {
    // Service aggregation logic will be added here
}
