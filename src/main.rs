mod config;
mod handlers;
mod models;
mod sample_submission;
mod sequencing;
mod storage;
pub mod tests;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::CorsLayer;

use crate::{
    config::database::create_pool, sample_submission::SampleSubmissionManager,
    sequencing::SequencingManager, storage::Storage,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub storage: Arc<Storage>,
    pub sample_manager: Arc<SampleSubmissionManager>,
    pub sequencing_manager: Arc<SequencingManager>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Create database connection pool
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    // Run migrations using the helper function
    config::database::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    // Initialize components
    let storage = Arc::new(Storage::new(
        std::env::var("STORAGE_PATH")
            .expect("STORAGE_PATH must be set")
            .into(),
    ));
    let sample_manager = Arc::new(SampleSubmissionManager::new(pool.clone()));
    let sequencing_manager = Arc::new(SequencingManager::new(pool.clone()));

    // Create app state
    let app_state = AppState {
        pool,
        storage,
        sample_manager,
        sequencing_manager,
    };

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/api/templates/upload", post(handlers::upload_template))
        .route("/api/templates", get(handlers::list_templates))
        .route("/api/samples", post(handlers::create_sample))
        .route("/api/samples", get(handlers::list_samples))
        .route("/api/samples/:id/validate", post(handlers::validate_sample))
        .route(
            "/api/sequencing/jobs",
            post(handlers::create_sequencing_job),
        )
        .route("/api/sequencing/jobs", get(handlers::list_sequencing_jobs))
        .route(
            "/api/sequencing/jobs/:id/status",
            post(handlers::update_job_status),
        )
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app.into_make_service(),
    )
    .await
    .unwrap();
}
