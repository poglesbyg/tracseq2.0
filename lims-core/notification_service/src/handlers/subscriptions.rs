use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppState, error::Result, models::*};

#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub user_id: Uuid,
    pub event_type: String,
    pub channels: Vec<Channel>,
    pub enabled: bool,
    pub filters: Option<serde_json::Value>,
    pub preferences: Option<NotificationPreferences>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSubscriptionRequest {
    pub event_type: Option<String>,
    pub channels: Option<Vec<Channel>>,
    pub enabled: Option<bool>,
    pub filters: Option<serde_json::Value>,
    pub preferences: Option<NotificationPreferences>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub event_type: String,
    pub channels: Vec<Channel>,
    pub enabled: bool,
    pub filters: serde_json::Value,
    pub preferences: NotificationPreferences,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Create a new subscription
/// POST /subscriptions
pub async fn create_subscription(
    State(state): State<AppState>,
    Json(request): Json<CreateSubscriptionRequest>,
) -> Result<Json<SubscriptionResponse>> {
    let subscription = state
        .notification_service
        .create_subscription(
            request.user_id,
            request.event_type,
            request.channels,
            request.enabled,
            request.filters.unwrap_or_default(),
            request.preferences.unwrap_or_default(),
        )
        .await?;

    Ok(Json(SubscriptionResponse {
        id: subscription.id,
        user_id: subscription.user_id,
        event_type: subscription.event_type,
        channels: subscription.channels,
        enabled: subscription.enabled,
        filters: subscription.filters,
        preferences: subscription.preferences,
        created_at: subscription.created_at,
        updated_at: subscription.updated_at,
    }))
}

/// List subscriptions
/// GET /subscriptions
pub async fn list_subscriptions(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<SubscriptionResponse>>> {
    let limit = query.limit.unwrap_or(50).min(1000);
    let offset = query.offset.unwrap_or(0);

    let subscriptions = state
        .notification_service
        .list_subscriptions(limit, offset)
        .await?;

    let responses = subscriptions
        .into_iter()
        .map(|sub| SubscriptionResponse {
            id: sub.id,
            user_id: sub.user_id,
            event_type: sub.event_type,
            channels: sub.channels,
            enabled: sub.enabled,
            filters: sub.filters,
            preferences: sub.preferences,
            created_at: sub.created_at,
            updated_at: sub.updated_at,
        })
        .collect();

    Ok(Json(responses))
}

/// Get a specific subscription
/// GET /subscriptions/{id}
pub async fn get_subscription(
    State(state): State<AppState>,
    Path(subscription_id): Path<Uuid>,
) -> Result<Json<SubscriptionResponse>> {
    let subscription = state
        .notification_service
        .get_subscription(subscription_id)
        .await?;

    Ok(Json(SubscriptionResponse {
        id: subscription.id,
        user_id: subscription.user_id,
        event_type: subscription.event_type,
        channels: subscription.channels,
        enabled: subscription.enabled,
        filters: subscription.filters,
        preferences: subscription.preferences,
        created_at: subscription.created_at,
        updated_at: subscription.updated_at,
    }))
}

/// Update a subscription
/// PUT /subscriptions/{id}
pub async fn update_subscription(
    State(state): State<AppState>,
    Path(subscription_id): Path<Uuid>,
    Json(request): Json<UpdateSubscriptionRequest>,
) -> Result<Json<SubscriptionResponse>> {
    let subscription = state
        .notification_service
        .update_subscription(
            subscription_id,
            request.event_type,
            request.channels,
            request.enabled,
            request.filters,
            request.preferences,
        )
        .await?;

    Ok(Json(SubscriptionResponse {
        id: subscription.id,
        user_id: subscription.user_id,
        event_type: subscription.event_type,
        channels: subscription.channels,
        enabled: subscription.enabled,
        filters: subscription.filters,
        preferences: subscription.preferences,
        created_at: subscription.created_at,
        updated_at: subscription.updated_at,
    }))
}

/// Delete a subscription
/// DELETE /subscriptions/{id}
pub async fn delete_subscription(
    State(state): State<AppState>,
    Path(subscription_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    state
        .notification_service
        .delete_subscription(subscription_id)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "Subscription deleted successfully",
        "subscription_id": subscription_id
    })))
}

/// Get user subscriptions
/// GET /subscriptions/user/{user_id}
pub async fn get_user_subscriptions(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<SubscriptionResponse>>> {
    let subscriptions = state
        .notification_service
        .get_user_subscriptions(user_id)
        .await?;

    let responses = subscriptions
        .into_iter()
        .map(|sub| SubscriptionResponse {
            id: sub.id,
            user_id: sub.user_id,
            event_type: sub.event_type,
            channels: sub.channels,
            enabled: sub.enabled,
            filters: sub.filters,
            preferences: sub.preferences,
            created_at: sub.created_at,
            updated_at: sub.updated_at,
        })
        .collect();

    Ok(Json(responses))
}

/// Get subscriptions for event type
/// GET /subscriptions/event/{event_type}
pub async fn get_event_subscriptions(
    State(state): State<AppState>,
    Path(event_type): Path<String>,
) -> Result<Json<Vec<SubscriptionResponse>>> {
    let subscriptions = state
        .notification_service
        .get_event_subscriptions(&event_type)
        .await?;

    let responses = subscriptions
        .into_iter()
        .map(|sub| SubscriptionResponse {
            id: sub.id,
            user_id: sub.user_id,
            event_type: sub.event_type,
            channels: sub.channels,
            enabled: sub.enabled,
            filters: sub.filters,
            preferences: sub.preferences,
            created_at: sub.created_at,
            updated_at: sub.updated_at,
        })
        .collect();

    Ok(Json(responses))
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
