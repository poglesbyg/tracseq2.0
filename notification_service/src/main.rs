use anyhow::Result;
use axum::{
    middleware,
    routing::{get, post, put, delete},
    Router,
};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    compression::CompressionLayer,
};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod database;
mod error;
mod handlers;
mod models;
mod services;
mod middleware;
mod clients;

use config::Config;
use database::DatabasePool;
use services::NotificationServiceImpl;
use clients::{AuthClient, SlackClient, TeamsClient, EmailClient, SmsClient};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "notification_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Notification Service");

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded successfully");

    // Setup database connection
    let db_pool = DatabasePool::new(&config.database_url).await?;
    info!("Database connection established");

    // Run database migrations
    db_pool.migrate().await?;
    info!("Database migrations completed");

    // Initialize external service clients
    let auth_client = AuthClient::new(config.auth_service_url.clone());
    let slack_client = SlackClient::new(config.slack.clone());
    let teams_client = TeamsClient::new(config.teams.clone());
    let email_client = EmailClient::new(config.email.clone());
    let sms_client = SmsClient::new(config.sms.clone());

    // Initialize notification service
    let notification_service = NotificationServiceImpl::new(
        db_pool.clone(),
        config.clone(),
        auth_client.clone(),
        slack_client,
        teams_client,
        email_client,
        sms_client,
    ).await?;
    info!("Notification service initialized");

    // Setup application state
    let app_state = AppState {
        notification_service,
        config: config.clone(),
        db_pool,
        auth_client,
    };

    // Build the application router
    let app = create_app(app_state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Notification service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub notification_service: NotificationServiceImpl,
    pub config: Config,
    pub db_pool: DatabasePool,
    pub auth_client: AuthClient,
}

/// Create the application router with all routes and middleware
fn create_app(state: AppState) -> Router {
    // Health check routes (no auth required)
    let health_routes = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/health/ready", get(handlers::health::readiness_check))
        .route("/health/metrics", get(handlers::health::metrics));

    // Notification management routes
    let notification_routes = Router::new()
        .route("/notifications", post(handlers::notifications::send_notification))
        .route("/notifications", get(handlers::notifications::list_notifications))
        .route("/notifications/:notification_id", get(handlers::notifications::get_notification))
        .route("/notifications/:notification_id/status", get(handlers::notifications::get_notification_status))
        .route("/notifications/:notification_id/retry", post(handlers::notifications::retry_notification))
        .route("/notifications/bulk", post(handlers::notifications::send_bulk_notifications))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // Channel management routes
    let channel_routes = Router::new()
        .route("/channels", get(handlers::channels::list_channels))
        .route("/channels/email/test", post(handlers::channels::test_email_channel))
        .route("/channels/sms/test", post(handlers::channels::test_sms_channel))
        .route("/channels/slack/test", post(handlers::channels::test_slack_channel))
        .route("/channels/teams/test", post(handlers::channels::test_teams_channel))
        .route("/channels/:channel_type/config", get(handlers::channels::get_channel_config))
        .route("/channels/:channel_type/config", put(handlers::channels::update_channel_config))
        .route("/channels/email/templates", get(handlers::channels::list_email_templates))
        .route("/channels/slack/webhooks", post(handlers::channels::create_slack_webhook))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // Template management routes
    let template_routes = Router::new()
        .route("/templates", post(handlers::templates::create_template))
        .route("/templates", get(handlers::templates::list_templates))
        .route("/templates/:template_id", get(handlers::templates::get_template))
        .route("/templates/:template_id", put(handlers::templates::update_template))
        .route("/templates/:template_id", delete(handlers::templates::delete_template))
        .route("/templates/:template_id/preview", post(handlers::templates::preview_template))
        .route("/templates/:template_id/validate", post(handlers::templates::validate_template))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // Subscription management routes
    let subscription_routes = Router::new()
        .route("/subscriptions", post(handlers::subscriptions::create_subscription))
        .route("/subscriptions", get(handlers::subscriptions::list_subscriptions))
        .route("/subscriptions/:subscription_id", get(handlers::subscriptions::get_subscription))
        .route("/subscriptions/:subscription_id", put(handlers::subscriptions::update_subscription))
        .route("/subscriptions/:subscription_id", delete(handlers::subscriptions::delete_subscription))
        .route("/subscriptions/user/:user_id", get(handlers::subscriptions::get_user_subscriptions))
        .route("/subscriptions/event/:event_type", get(handlers::subscriptions::get_event_subscriptions))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // Integration routes
    let integration_routes = Router::new()
        .route("/integration/lab-events", post(handlers::integration::handle_lab_event))
        .route("/integration/sample-events", post(handlers::integration::handle_sample_event))
        .route("/integration/sequencing-events", post(handlers::integration::handle_sequencing_event))
        .route("/integration/template-events", post(handlers::integration::handle_template_event))
        .route("/integration/system-alerts", post(handlers::integration::handle_system_alert))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // Admin routes (require admin privileges)
    let admin_routes = Router::new()
        .route("/admin/statistics", get(handlers::admin::get_notification_statistics))
        .route("/admin/failed-notifications", get(handlers::admin::get_failed_notifications))
        .route("/admin/retry-failed", post(handlers::admin::retry_failed_notifications))
        .route("/admin/cleanup", post(handlers::admin::cleanup_old_notifications))
        .route("/admin/channels/health", get(handlers::admin::check_channel_health))
        .route("/admin/rate-limits", get(handlers::admin::get_rate_limits))
        .route("/admin/rate-limits", put(handlers::admin::update_rate_limits))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            middleware::admin_middleware,
        ));

    // Combine all routes
    Router::new()
        .merge(health_routes)
        .merge(notification_routes)
        .merge(channel_routes)
        .merge(template_routes)
        .merge(subscription_routes)
        .merge(integration_routes)
        .merge(admin_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(CorsLayer::permissive()) // Configure CORS as needed
        )
        .with_state(state)
} 
