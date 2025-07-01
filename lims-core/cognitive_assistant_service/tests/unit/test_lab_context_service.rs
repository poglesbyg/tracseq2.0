//! Unit tests for lab context service

use cognitive_assistant_service::{
    LabContextService, LabContext, Department, UserRole,
    ServiceError, ServiceErrorKind,
};
use crate::test_utils::*;
use sqlx::PgPool;
use std::sync::Arc;

#[tokio::test]
async fn test_lab_context_service_creation() {
    let test_db = TestDatabase::new().await;
    let service = LabContextService::new(test_db.pool.clone());
    
    // Service should be created successfully
    assert!(!service.pool.is_closed());
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_get_user_context() {
    let test_db = TestDatabase::new().await;
    let service = LabContextService::new(test_db.pool.clone());
    
    // Insert test user context
    sqlx::query!(
        r#"
        INSERT INTO user_contexts (user_id, role, department, preferences, created_at)
        VALUES ($1, $2, $3, $4, NOW())
        "#,
        "test_user",
        UserRole::Researcher as i32,
        Department::Molecular as i32,
        serde_json::json!({"theme": "dark", "notifications": true})
    )
    .execute(&test_db.pool)
    .await
    .expect("Failed to insert test user context");
    
    // Get context
    let result = service.get_user_context("test_user").await;
    
    assert!(result.is_ok());
    let context = result.unwrap();
    assert!(context.user_preferences.contains_key("theme"));
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_get_active_samples() {
    let test_db = TestDatabase::new().await;
    let service = LabContextService::new(test_db.pool.clone());
    
    // Insert test samples
    for i in 1..=3 {
        sqlx::query!(
            r#"
            INSERT INTO samples (sample_id, name, type, status, user_id, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            "#,
            format!("SAMP{:03}", i),
            format!("Test Sample {}", i),
            "DNA",
            "active",
            "test_user"
        )
        .execute(&test_db.pool)
        .await
        .expect("Failed to insert test sample");
    }
    
    // Get active samples
    let samples = service.get_active_samples("test_user").await.unwrap();
    
    assert_eq!(samples.len(), 3);
    assert!(samples.iter().all(|s| s.starts_with("SAMP")));
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_get_recent_activities() {
    let test_db = TestDatabase::new().await;
    let service = LabContextService::new(test_db.pool.clone());
    
    // Insert test activities
    let activities = vec![
        "DNA extraction started",
        "PCR amplification completed",
        "Gel electrophoresis in progress",
    ];
    
    for activity in &activities {
        sqlx::query!(
            r#"
            INSERT INTO activities (user_id, activity_type, description, timestamp)
            VALUES ($1, $2, $3, NOW())
            "#,
            "test_user",
            "lab_work",
            activity
        )
        .execute(&test_db.pool)
        .await
        .expect("Failed to insert test activity");
    }
    
    // Get recent activities
    let recent = service.get_recent_activities("test_user", 10).await.unwrap();
    
    assert_eq!(recent.len(), 3);
    for activity in activities {
        assert!(recent.contains(&activity.to_string()));
    }
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_get_equipment_status() {
    let test_db = TestDatabase::new().await;
    let service = LabContextService::new(test_db.pool.clone());
    
    // Insert test equipment
    let equipment = vec![
        ("PCR-001", "PCR Machine", "available"),
        ("CENT-001", "Centrifuge", "in_use"),
        ("FREEZE-001", "Freezer -80C", "maintenance"),
    ];
    
    for (id, name, status) in &equipment {
        sqlx::query!(
            r#"
            INSERT INTO equipment (equipment_id, name, status, department, last_updated)
            VALUES ($1, $2, $3, $4, NOW())
            "#,
            id,
            name,
            status,
            Department::Molecular as i32
        )
        .execute(&test_db.pool)
        .await
        .expect("Failed to insert test equipment");
    }
    
    // Get equipment status
    let status = service.get_equipment_status(Department::Molecular).await.unwrap();
    
    assert_eq!(status.len(), 3);
    assert_eq!(status.get("PCR Machine"), Some(&"available".to_string()));
    assert_eq!(status.get("Centrifuge"), Some(&"in_use".to_string()));
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_build_full_context() {
    let test_db = TestDatabase::new().await;
    let service = LabContextService::new(test_db.pool.clone());
    
    // Setup test data
    sqlx::query!(
        r#"
        INSERT INTO user_contexts (user_id, role, department, preferences, created_at)
        VALUES ($1, $2, $3, $4, NOW())
        "#,
        "test_user",
        UserRole::Technician as i32,
        Department::Sequencing as i32,
        serde_json::json!({"language": "en"})
    )
    .execute(&test_db.pool)
    .await
    .expect("Failed to insert user context");
    
    // Build full context
    let context = service.build_context_for_user("test_user", Department::Sequencing).await.unwrap();
    
    assert!(context.user_preferences.contains_key("language"));
    assert_eq!(context.active_samples.len(), 0); // No samples yet
    assert_eq!(context.recent_activities.len(), 0); // No activities yet
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_update_user_preferences() {
    let test_db = TestDatabase::new().await;
    let service = LabContextService::new(test_db.pool.clone());
    
    // Create initial context
    sqlx::query!(
        r#"
        INSERT INTO user_contexts (user_id, role, department, preferences, created_at)
        VALUES ($1, $2, $3, $4, NOW())
        "#,
        "test_user",
        UserRole::Researcher as i32,
        Department::Molecular as i32,
        serde_json::json!({"theme": "light"})
    )
    .execute(&test_db.pool)
    .await
    .expect("Failed to insert user context");
    
    // Update preferences
    let new_prefs = serde_json::json!({
        "theme": "dark",
        "notifications": true,
        "email_alerts": false
    });
    
    let result = service.update_user_preferences("test_user", new_prefs.clone()).await;
    assert!(result.is_ok());
    
    // Verify update
    let context = service.get_user_context("test_user").await.unwrap();
    assert_eq!(context.user_preferences.get("theme"), Some(&"dark".to_string()));
    assert_eq!(context.user_preferences.get("notifications"), Some(&"true".to_string()));
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_context_caching() {
    let test_db = TestDatabase::new().await;
    let service = LabContextService::new(test_db.pool.clone());
    
    // Note: This tests the concept - actual caching would be implemented in the service
    let user_id = "cached_user";
    
    // First call - would hit database
    let start = std::time::Instant::now();
    let _ = service.build_context_for_user(user_id, Department::QualityControl).await;
    let first_duration = start.elapsed();
    
    // Second call - should be faster if cached
    let start = std::time::Instant::now();
    let _ = service.build_context_for_user(user_id, Department::QualityControl).await;
    let second_duration = start.elapsed();
    
    // Both should complete quickly
    assert!(first_duration.as_millis() < 100);
    assert!(second_duration.as_millis() < 100);
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_error_handling_invalid_user() {
    let test_db = TestDatabase::new().await;
    let service = LabContextService::new(test_db.pool.clone());
    
    // Try to get context for non-existent user
    let result = service.get_user_context("non_existent_user").await;
    
    assert!(result.is_err());
    if let Err(error) = result {
        assert!(matches!(error.kind, ServiceErrorKind::DatabaseError));
    }
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_concurrent_context_queries() {
    let test_db = TestDatabase::new().await;
    let service = Arc::new(LabContextService::new(test_db.pool.clone()));
    
    // Create test user
    sqlx::query!(
        r#"
        INSERT INTO user_contexts (user_id, role, department, preferences, created_at)
        VALUES ($1, $2, $3, $4, NOW())
        "#,
        "concurrent_user",
        UserRole::Supervisor as i32,
        Department::Storage as i32,
        serde_json::json!({})
    )
    .execute(&test_db.pool)
    .await
    .expect("Failed to insert user context");
    
    let mut handles = Vec::new();
    
    // Spawn concurrent queries
    for _ in 0..10 {
        let service_clone = service.clone();
        let handle = tokio::spawn(async move {
            service_clone.get_user_context("concurrent_user").await
        });
        handles.push(handle);
    }
    
    let results = futures::future::join_all(handles).await;
    
    // All should succeed
    assert_eq!(results.len(), 10);
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
    
    test_db.cleanup().await;
}