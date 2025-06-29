use anyhow::Result;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use auth_service::{AppState, AuthServiceImpl, Config, DatabasePool, create_router};

#[tokio::main]
async fn main() -> Result<()> {
    println!("AUTH SERVICE: Starting main function");

    // Initialize tracing
    println!("AUTH SERVICE: Initializing tracing");
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "auth_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    println!("AUTH SERVICE: Tracing initialized");
    info!("Starting Authentication Service");

    // Load configuration
    println!("AUTH SERVICE: Loading configuration");
    let config = match Config::from_env() {
        Ok(config) => {
            println!("AUTH SERVICE: Configuration loaded successfully");
            config
        }
        Err(e) => {
            println!("AUTH SERVICE: Failed to load configuration: {e}");
            return Err(e);
        }
    };
    info!("Configuration loaded successfully");

    // Setup database connection
    println!(
        "AUTH SERVICE: Setting up database connection to: {}",
        config.database_url
    );
    let db_pool = match DatabasePool::new(&config.database_url).await {
        Ok(pool) => {
            println!("AUTH SERVICE: Database connection established");
            pool
        }
        Err(e) => {
            println!("AUTH SERVICE: Failed to establish database connection: {e}");
            return Err(e);
        }
    };
    info!("Database connection established");

    // Run database migrations
    println!("AUTH SERVICE: Running database migrations");
    if let Err(e) = db_pool.migrate().await {
        println!("AUTH SERVICE: Database migration failed: {e}");
        return Err(e);
    }
    println!("AUTH SERVICE: Database migrations completed");
    info!("Database migrations completed");

    // Initialize authentication service
    println!("AUTH SERVICE: Initializing authentication service");
    let auth_service = match AuthServiceImpl::new(db_pool.clone(), config.clone()) {
        Ok(service) => {
            println!("AUTH SERVICE: Authentication service initialized");
            service
        }
        Err(e) => {
            println!("AUTH SERVICE: Failed to initialize authentication service: {e}");
            return Err(e);
        }
    };
    info!("Authentication service initialized");

    // Setup application state
    println!("AUTH SERVICE: Setting up application state");
    let app_state = AppState {
        auth_service: std::sync::Arc::new(auth_service),
        config: std::sync::Arc::new(config.clone()),
        db_pool,
    };

    // Build the application router
    println!("AUTH SERVICE: Building application router");
    let app = create_router(app_state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    println!("AUTH SERVICE: Starting server on {addr}");
    info!("Authentication service listening on {addr}");

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            println!("AUTH SERVICE: Successfully bound to address {addr}");
            listener
        }
        Err(e) => {
            println!("AUTH SERVICE: Failed to bind to address {addr}: {e}");
            return Err(e.into());
        }
    };

    println!("AUTH SERVICE: About to start serving requests");
    if let Err(e) = axum::serve(listener, app).await {
        println!("AUTH SERVICE: Server failed with error: {e}");
        return Err(e.into());
    }

    println!("AUTH SERVICE: Server exited normally");
    Ok(())
}
