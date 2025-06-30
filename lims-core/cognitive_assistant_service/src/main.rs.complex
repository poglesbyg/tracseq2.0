use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};
use uuid::Uuid;

mod handlers;
mod models;
mod services;

use handlers::cognitive_handler;
use models::{LabQuery, IntelligentResponse};
use services::{ollama_service::OllamaService, lab_context_service::LabContextService};

#[derive(Clone)]
pub struct AppState {
    pub ollama_service: Arc<OllamaService>,
    pub lab_context_service: Arc<LabContextService>,
    pub database: sqlx::PgPool,
    pub redis: redis::aio::MultiplexedConnection,
}

#[derive(Serialize)]
struct HealthResponse {
    service: String,
    status: String,
    version: String,
    ollama_connected: bool,
    timestamp: chrono::DateTime<chrono::Utc>,
}

async fn health_check(State(state): State<AppState>) -> Result<Json<HealthResponse>, StatusCode> {
    // Check Ollama connectivity
    let ollama_connected = state.ollama_service.check_connection().await.is_ok();
    
    let response = HealthResponse {
        service: "cognitive_assistant_service".to_string(),
        status: if ollama_connected { "healthy" } else { "degraded" }.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        ollama_connected,
        timestamp: chrono::Utc::now(),
    };
    
    Ok(Json(response))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("cognitive_assistant_service=debug,info")
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Get configuration from environment
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:tracseq_password@postgres:5432/tracseq".to_string());
    
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://redis:6379/0".to_string());
    
    let ollama_url = std::env::var("OLLAMA_BASE_URL")
        .unwrap_or_else(|_| "http://ollama:11434".to_string());
    
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8090".to_string())
        .parse::<u16>()?;

    info!("🧠 Starting Cognitive Laboratory Assistant Service on port {}", port);
    info!("🔗 Connecting to database: {}", database_url);
    info!("🔗 Connecting to Redis: {}", redis_url);
    info!("🤖 Connecting to Ollama: {}", ollama_url);

    // Connect to database
    let database = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&database).await?;

    // Connect to Redis
    let redis_client = redis::Client::open(redis_url)?;
    let redis_connection = redis_client.get_multiplexed_async_connection().await?;

    // Initialize services
    let ollama_service = Arc::new(OllamaService::new(&ollama_url));
    let lab_context_service = Arc::new(LabContextService::new(database.clone()));

    // Verify Ollama connection
    match ollama_service.check_connection().await {
        Ok(_) => info!("✅ Successfully connected to Ollama"),
        Err(e) => warn!("⚠️ Failed to connect to Ollama: {}", e),
    }

    // Create application state
    let app_state = AppState {
        ollama_service,
        lab_context_service,
        database,
        redis: redis_connection,
    };

    // Build our application routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ask", post(cognitive_handler::handle_intelligent_query))
        .route("/suggest", post(cognitive_handler::handle_proactive_suggestions))
        .route("/analyze", post(cognitive_handler::handle_context_analysis))
        .route("/predict", post(cognitive_handler::handle_predictive_insights))
        .route("/chat", post(cognitive_handler::handle_lab_chat))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Start the server
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    
    info!("🚀 Cognitive Laboratory Assistant Service listening on port {}", port);
    info!("📋 Available endpoints:");
    info!("   • GET  /health - Service health check");
    info!("   • POST /ask - Intelligent laboratory queries");
    info!("   • POST /suggest - Proactive suggestions");
    info!("   • POST /analyze - Context analysis");
    info!("   • POST /predict - Predictive insights");
    info!("   • POST /chat - Natural language lab chat");

    axum::serve(listener, app).await?;

    Ok(())
} 