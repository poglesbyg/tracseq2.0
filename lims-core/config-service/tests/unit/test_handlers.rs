//! Unit tests for configuration service handlers

use config_service::{
    handlers::*, AppState, ConfigEntry, CreateConfigRequest,
    UpdateConfigRequest, ConfigQueryParams, ConfigResponse,
    ServiceConfigResponse, HealthResponse,
};
use crate::test_utils::*;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};

#[tokio::test]
async fn test_health_check_handler() {
    let state = create_app_state_with_configs(
        ConfigFactory::create_auth_service_configs()
    ).await;
    
    let response = health_check(State(state.clone())).await;
    let health = response.0;
    
    assert_eq!(health.status, "healthy");
    assert_eq!(health.config_count, 3); // 3 auth configs
    assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn test_get_configs_no_filters() {
    let configs = vec![
        ConfigFactory::create_test_config("service1", "key1", serde_json::json!("value1")),
        ConfigFactory::create_test_config("service2", "key2", serde_json::json!("value2")),
    ];
    
    let state = create_app_state_with_configs(configs.clone()).await;
    
    let params = ConfigQueryParams {
        environment: None,
        tags: None,
        version: None,
    };
    
    let response = get_configs(Query(params), State(state)).await;
    let config_response = response.0;
    
    assert_eq!(config_response.total, 2);
    assert_eq!(config_response.configs.len(), 2);
}

#[tokio::test]
async fn test_get_configs_with_environment_filter() {
    let configs = vec![
        ConfigFactory::create_config_with_environment("service1", "key1", "dev"),
        ConfigFactory::create_config_with_environment("service1", "key2", "prod"),
        ConfigFactory::create_config_with_environment("service2", "key1", "dev"),
    ];
    
    let state = create_app_state_with_configs(configs).await;
    
    let params = ConfigQueryParams {
        environment: Some("dev".to_string()),
        tags: None,
        version: None,
    };
    
    let response = get_configs(Query(params), State(state)).await;
    let config_response = response.0;
    
    assert_eq!(config_response.total, 2);
    assert!(config_response.configs.iter().all(|c| c.environment == "dev"));
}

#[tokio::test]
async fn test_get_configs_with_tag_filter() {
    let configs = vec![
        ConfigFactory::create_config_with_tags("service1", "key1", vec!["critical".to_string()]),
        ConfigFactory::create_config_with_tags("service1", "key2", vec!["optional".to_string()]),
        ConfigFactory::create_config_with_tags("service2", "key1", vec!["critical".to_string(), "encrypted".to_string()]),
    ];
    
    let state = create_app_state_with_configs(configs).await;
    
    let params = ConfigQueryParams {
        environment: None,
        tags: Some("critical".to_string()),
        version: None,
    };
    
    let response = get_configs(Query(params), State(state)).await;
    let config_response = response.0;
    
    assert_eq!(config_response.total, 2);
    assert!(config_response.configs.iter().all(|c| c.tags.contains(&"critical".to_string())));
}

#[tokio::test]
async fn test_get_service_config_success() {
    let configs = ConfigFactory::create_auth_service_configs();
    let state = create_app_state_with_configs(configs).await;
    
    let result = get_service_config(
        Path(("auth-service".to_string(), "test".to_string())),
        State(state)
    ).await;
    
    assert!(result.is_ok());
    let response = result.unwrap().0;
    
    ConfigAssertions::assert_service_config_response(&response, "auth-service", "test");
    assert_eq!(response.config.len(), 3); // 3 auth service configs
    assert!(response.config.contains_key("jwt_expiry_hours"));
}

#[tokio::test]
async fn test_get_service_config_not_found() {
    let state = create_test_app_state();
    
    let result = get_service_config(
        Path(("nonexistent-service".to_string(), "test".to_string())),
        State(state)
    ).await;
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_config_success() {
    let state = create_test_app_state();
    
    let request = RequestFactory::create_config_request(
        "new-service",
        "new-key",
        serde_json::json!("new-value")
    );
    
    let result = create_config(
        State(state.clone()),
        Json(request)
    ).await;
    
    assert!(result.is_ok());
    let created_config = result.unwrap().0;
    
    assert_eq!(created_config.service_name, "new-service");
    assert_eq!(created_config.key, "new-key");
    assert_eq!(created_config.value, serde_json::json!("new-value"));
    assert_eq!(created_config.version, 1);
    
    // Verify it was stored
    let store = state.config_store.read().await;
    assert!(store.contains_key("new-service-new-key"));
}

#[tokio::test]
async fn test_create_config_conflict() {
    let existing_config = ConfigFactory::create_test_config(
        "existing-service",
        "existing-key",
        serde_json::json!("existing-value")
    );
    
    let state = create_app_state_with_configs(vec![existing_config]).await;
    
    let request = RequestFactory::create_config_request(
        "existing-service",
        "existing-key",
        serde_json::json!("new-value")
    );
    
    let result = create_config(
        State(state),
        Json(request)
    ).await;
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_update_config_success() {
    let original_config = ConfigFactory::create_test_config(
        "update-service",
        "update-key",
        serde_json::json!("original-value")
    );
    let original_version = original_config.version;
    
    let state = create_app_state_with_configs(vec![original_config.clone()]).await;
    
    let update_request = RequestFactory::update_config_request(
        serde_json::json!("updated-value")
    );
    
    let result = update_config(
        Path(("update-service".to_string(), "update-key".to_string())),
        State(state.clone()),
        Json(update_request)
    ).await;
    
    assert!(result.is_ok());
    let updated_config = result.unwrap().0;
    
    assert_eq!(updated_config.value, serde_json::json!("updated-value"));
    ConfigAssertions::assert_config_version_incremented(&updated_config, &original_config);
}

#[tokio::test]
async fn test_update_config_with_tags() {
    let original_config = ConfigFactory::create_test_config(
        "tag-service",
        "tag-key",
        serde_json::json!("value")
    );
    
    let state = create_app_state_with_configs(vec![original_config]).await;
    
    let update_request = RequestFactory::update_config_request_with_tags(
        serde_json::json!("new-value"),
        vec!["updated".to_string(), "important".to_string()]
    );
    
    let result = update_config(
        Path(("tag-service".to_string(), "tag-key".to_string())),
        State(state),
        Json(update_request)
    ).await;
    
    assert!(result.is_ok());
    let updated_config = result.unwrap().0;
    
    assert_eq!(updated_config.tags, vec!["updated".to_string(), "important".to_string()]);
}

#[tokio::test]
async fn test_update_config_not_found() {
    let state = create_test_app_state();
    
    let update_request = RequestFactory::update_config_request(
        serde_json::json!("value")
    );
    
    let result = update_config(
        Path(("nonexistent".to_string(), "key".to_string())),
        State(state),
        Json(update_request)
    ).await;
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_config_success() {
    let config = ConfigFactory::create_test_config(
        "delete-service",
        "delete-key",
        serde_json::json!("to-delete")
    );
    
    let state = create_app_state_with_configs(vec![config]).await;
    
    let result = delete_config(
        Path(("delete-service".to_string(), "delete-key".to_string())),
        State(state.clone())
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), StatusCode::NO_CONTENT);
    
    // Verify it was deleted
    let store = state.config_store.read().await;
    assert!(!store.contains_key("delete-service-delete-key"));
}

#[tokio::test]
async fn test_delete_config_not_found() {
    let state = create_test_app_state();
    
    let result = delete_config(
        Path(("nonexistent".to_string(), "key".to_string())),
        State(state)
    ).await;
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_bulk_update_service_config() {
    // Setup existing configs
    let existing_configs = vec![
        ConfigFactory::create_test_config("bulk-service", "existing1", serde_json::json!("old1")),
        ConfigFactory::create_test_config("bulk-service", "existing2", serde_json::json!("old2")),
    ];
    
    let state = create_app_state_with_configs(existing_configs).await;
    
    // Prepare bulk update
    let bulk_configs = TestDataGenerator::generate_bulk_configs("bulk-service", 3);
    
    let result = bulk_update_service_config(
        Path(("bulk-service".to_string(), "test".to_string())),
        State(state.clone()),
        Json(bulk_configs)
    ).await;
    
    assert!(result.is_ok());
    let response = result.unwrap().0;
    
    assert_eq!(response.service_name, "bulk-service");
    assert_eq!(response.environment, "test");
    assert_eq!(response.config.len(), 5); // 2 existing + 3 new
    
    // Verify updates in store
    let store = state.config_store.read().await;
    assert_eq!(store.len(), 5);
}

#[tokio::test]
async fn test_complex_config_handling() {
    let complex_value = TestDataGenerator::generate_complex_config_value();
    let config = ConfigFactory::create_test_config(
        "complex-service",
        "complex-config",
        complex_value.clone()
    );
    
    let state = create_app_state_with_configs(vec![config]).await;
    
    let result = get_service_config(
        Path(("complex-service".to_string(), "test".to_string())),
        State(state)
    ).await;
    
    assert!(result.is_ok());
    let response = result.unwrap().0;
    
    let retrieved_value = response.config.get("complex-config").unwrap();
    assert_eq!(retrieved_value, &complex_value);
    
    // Verify nested structure is preserved
    assert_eq!(retrieved_value["database"]["host"], "localhost");
    assert_eq!(retrieved_value["features"]["enable_cache"], true);
}

#[tokio::test]
async fn test_concurrent_handler_operations() {
    let state = Arc::new(create_test_app_state());
    let mut handles = Vec::new();
    
    // Concurrent creates
    for i in 0..10 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            let request = RequestFactory::create_config_request(
                "concurrent-service",
                &format!("key-{}", i),
                serde_json::json!(i)
            );
            
            create_config(
                State(state_clone.as_ref().clone()),
                Json(request)
            ).await
        });
        handles.push(handle);
    }
    
    let results = futures::future::join_all(handles).await;
    
    // All should succeed
    assert!(results.iter().all(|r| r.is_ok() && r.as_ref().unwrap().is_ok()));
    
    // Verify all were created
    let store = state.config_store.read().await;
    assert_eq!(store.len(), 10);
}