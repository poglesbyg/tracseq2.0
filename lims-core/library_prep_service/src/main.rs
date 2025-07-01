use axum::{routing::{get, post, put}, Router};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

mod models;
mod handlers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Run migrations unless SKIP_MIGRATIONS is set
    if std::env::var("SKIP_MIGRATIONS").unwrap_or_default() != "true" {
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;
    }

    // Build application
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/protocols", get(handlers::list_protocols))
        .route("/api/v1/protocols", post(handlers::create_protocol))
        .route("/api/v1/protocols/:id", get(handlers::get_protocol))
        .route("/api/v1/protocols/:id", put(handlers::update_protocol))
        .route("/api/v1/preparations", get(handlers::list_library_preps))
        .route("/api/v1/preparations", post(handlers::create_library_prep))
        .route("/api/v1/preparations/:id", get(handlers::get_library_prep))
        .route("/api/v1/preparations/:id", put(handlers::update_library_prep))
        .route("/api/v1/preparations/:id/complete", post(handlers::complete_library_prep))
        .route("/api/v1/stats", get(handlers::get_library_prep_stats))
        .route("/api/v1/search", get(handlers::search_library_preps))
        .with_state(pool)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // Start server
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8085".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");
    
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Library Prep Service listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "healthy"
} 