//! TracSeq Event Service - Event-driven communication hub for microservices.

mod events;
mod handlers;
mod services;

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use services::event_bus::{EventBus, RedisEventBus};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{info, Level};
use tracing_subscriber;

use events::{Event, EventPublicationResult, SubscriptionConfig};

/// Application state
#[derive(Clone)]
struct AppState {
    event_bus: Arc<dyn EventBus>,
}

/// Health check response
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Event publication request
#[derive(Deserialize)]
struct PublishEventRequest {
    event_type: String,
    source_service: String,
    payload: serde_json::Value,
    subject: Option<String>,
    priority: Option<u8>,
    correlation_id: Option<uuid::Uuid>,
}

/// Event bus statistics response
#[derive(Serialize)]
struct StatsResponse {
    events_published: u64,
    events_consumed: u64,
    events_failed: u64,
    handlers_registered: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("🚀 Starting TracSeq Event Service");

    // Load configuration
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8087".to_string())
        .parse::<u16>()
        .unwrap_or(8087);

    // Initialize event bus
    info!("🔗 Connecting to Redis at {}", redis_url);
    let event_bus: Arc<dyn EventBus> = Arc::new(
        RedisEventBus::new(&redis_url).await
            .expect("Failed to initialize Redis event bus")
    );

    // Create application state
    let app_state = AppState { event_bus };

    // Build the application router
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/api/v1/events/publish", post(publish_event))
        .route("/api/v1/events/subscribe", post(subscribe_to_events))
        .route("/api/v1/stats", get(get_stats))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Start the server
    let bind_address = format!("{}:{}", host, port);
    info!("🌐 Starting server on {}", bind_address);
    
    let listener = TcpListener::bind(&bind_address).await
        .expect("Failed to bind to address");

    info!("✅ TracSeq Event Service is running on http://{}", bind_address);
    
    axum::serve(listener, app).await
        .expect("Server failed to start");

    Ok(())
}

/// Root endpoint
async fn root() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "service": "TracSeq Event Service",
        "version": "0.1.0",
        "description": "Event-driven communication hub for TracSeq microservices",
        "endpoints": {
            "health": "/health",
            "publish": "/api/v1/events/publish",
            "subscribe": "/api/v1/events/subscribe",
            "stats": "/api/v1/stats"
        }
    }))
}

/// Health check endpoint
async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let stats = state.event_bus.get_stats().await;
    
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: "0.1.0".to_string(),
        timestamp: chrono::Utc::now(),
    })
}

/// Publish an event
async fn publish_event(
    State(state): State<AppState>,
    Json(request): Json<PublishEventRequest>,
) -> Result<Json<EventPublicationResult>, StatusCode> {
    // Create event
    let mut event = Event::new(
        request.event_type,
        request.source_service,
        request.payload,
    );

    // Set optional fields
    if let Some(subject) = request.subject {
        event = event.with_subject(subject);
    }
    
    if let Some(priority) = request.priority {
        event = event.with_priority(priority);
    }
    
    if let Some(correlation_id) = request.correlation_id {
        event = event.with_correlation_id(correlation_id);
    }

    // Publish event
    match state.event_bus.publish(event).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Subscribe to events
async fn subscribe_to_events(
    State(state): State<AppState>,
    Json(config): Json<SubscriptionConfig>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.event_bus.subscribe(config.clone()).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "subscribed",
            "subscription_name": config.name,
            "event_types": config.event_types
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get event bus statistics
async fn get_stats(State(state): State<AppState>) -> Json<StatsResponse> {
    let stats = state.event_bus.get_stats().await;
    
    Json(StatsResponse {
        events_published: stats.events_published,
        events_consumed: stats.events_consumed,
        events_failed: stats.events_failed,
        handlers_registered: stats.handlers_registered,
    })
} 
