use crate::auth::{AuthError, AuthService, Claims, LoginResponse, User};
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn auth_routes(auth_service: Arc<AuthService>) -> Router {
    Router::new()
        .route("/sso/login", get(initiate_sso))
        .route("/sso/callback", post(handle_sso_callback))
        .route("/me", get(get_current_user))
        .with_state(auth_service)
}

async fn initiate_sso(
    State(auth_service): State<Arc<AuthService>>,
) -> Result<Json<SsoInitiateResponse>, AuthError> {
    let redirect_url = auth_service.initiate_sso().await?;
    Ok(Json(SsoInitiateResponse { redirect_url }))
}

async fn handle_sso_callback(
    State(auth_service): State<Arc<AuthService>>,
    Json(payload): Json<SsoCallbackRequest>,
) -> Result<Json<LoginResponse>, AuthError> {
    auth_service
        .handle_sso_callback(&payload.saml_response)
        .await
        .map(Json)
}

async fn get_current_user(
    State(auth_service): State<Arc<AuthService>>,
    claims: Claims,
) -> Result<Json<User>, AuthError> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, unc_pid, email, display_name, role, status
        FROM users
        WHERE id = $1
        "#,
        claims.sub
    )
    .fetch_one(&auth_service.db)
    .await?;

    Ok(Json(user))
}

#[derive(Debug, Serialize)]
struct SsoInitiateResponse {
    redirect_url: String,
}

#[derive(Debug, Deserialize)]
struct SsoCallbackRequest {
    saml_response: String,
}
