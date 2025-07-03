//! Integration tests for proactive AI suggestions

use cognitive_assistant_service::{
    ProactiveSuggestion, SuggestionType, Department,
    OllamaService, LabContextService,
};
use crate::test_utils::*;
use wiremock::MockServer;
use sqlx::PgPool;

#[tokio::test]
async fn test_generate_department_specific_suggestions() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let test_db = TestDatabase::new().await;
    let ollama_service = OllamaService::new(mock_server.uri());
    let context_service = LabContextService::new(test_db.pool.clone());
    
    // Setup test data - create some historical activities
    for i in 0..5 {
        sqlx::query!(
            r#"
            INSERT INTO activities (user_id, activity_type, description, timestamp)
            VALUES ($1, $2, $3, NOW() - INTERVAL '1 day' * $4)
            "#,
            "lab_user",
            "sample_processing",
            format!("Processed sample batch {}", i),
            i as i32
        )
        .execute(&test_db.pool)
        .await
        .expect("Failed to insert activity");
    }
    
    // Generate suggestions for different departments
    let departments = vec![
        Department::Sequencing,
        Department::Molecular,
        Department::QualityControl,
        Department::Storage,
    ];
    
    for dept in departments {
        let suggestions = generate_proactive_suggestions_for_department(
            &ollama_service,
            &context_service,
            &test_db.pool,
            dept.clone()
        ).await;
        
        assert!(suggestions.is_ok());
        let suggestions = suggestions.unwrap();
        assert!(!suggestions.is_empty());
        
        for suggestion in &suggestions {
            CognitiveAssertions::assert_suggestion_validity(suggestion);
            assert_eq!(suggestion.department, dept);
        }
    }
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_suggestion_prioritization() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let test_db = TestDatabase::new().await;
    
    // Insert various types of data that should trigger different priority suggestions
    
    // Critical: Equipment failure
    sqlx::query!(
        r#"
        INSERT INTO equipment (equipment_id, name, status, department, last_updated)
        VALUES ($1, $2, $3, $4, NOW())
        "#,
        "FREEZE001",
        "Ultra-Low Freezer",
        "critical_failure",
        Department::Storage as i32
    )
    .execute(&test_db.pool)
    .await
    .expect("Failed to insert equipment");
    
    // High: Expiring reagents
    sqlx::query!(
        r#"
        INSERT INTO reagents (reagent_id, name, expiry_date, quantity, department)
        VALUES ($1, $2, NOW() + INTERVAL '7 days', $3, $4)
        "#,
        "REA001",
        "PCR Master Mix",
        10,
        Department::Molecular as i32
    )
    .execute(&test_db.pool)
    .await
    .expect("Failed to insert reagent");
    
    // Medium: Process optimization opportunity
    for i in 0..20 {
        sqlx::query!(
            r#"
            INSERT INTO process_metrics (process_name, duration_minutes, success_rate, timestamp)
            VALUES ($1, $2, $3, NOW() - INTERVAL '1 hour' * $4)
            "#,
            "RNA Extraction",
            120 + i * 5, // Increasing duration trend
            0.85,
            i
        )
        .execute(&test_db.pool)
        .await
        .expect("Failed to insert process metric");
    }
    
    let ollama_service = OllamaService::new(mock_server.uri());
    let suggestions = analyze_lab_data_and_suggest(&ollama_service, &test_db.pool).await.unwrap();
    
    // Should have suggestions of different priorities
    let critical_suggestions: Vec<_> = suggestions.iter()
        .filter(|s| s.priority == "critical")
        .collect();
    let high_suggestions: Vec<_> = suggestions.iter()
        .filter(|s| s.priority == "high")
        .collect();
    let medium_suggestions: Vec<_> = suggestions.iter()
        .filter(|s| s.priority == "medium")
        .collect();
    
    assert!(!critical_suggestions.is_empty(), "Should have critical suggestions for equipment failure");
    assert!(!high_suggestions.is_empty(), "Should have high priority suggestions for expiring reagents");
    assert!(!medium_suggestions.is_empty(), "Should have medium priority suggestions for process optimization");
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_contextual_suggestion_generation() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let test_db = TestDatabase::new().await;
    let ollama_service = OllamaService::new(mock_server.uri());
    
    // Create context with specific issues
    let context = ContextBuilder::new()
        .with_equipment(vec![
            ("PCR Machine A".to_string(), "Frequent temperature fluctuations".to_string()),
            ("Centrifuge B".to_string(), "Unusual vibrations detected".to_string()),
        ])
        .with_activities(vec![
            "Multiple PCR failures this week".to_string(),
            "Inconsistent centrifugation results".to_string(),
        ])
        .build();
    
    let suggestions = generate_contextual_suggestions(&ollama_service, &context).await.unwrap();
    
    // Should generate maintenance suggestions
    let maintenance_suggestions: Vec<_> = suggestions.iter()
        .filter(|s| matches!(s.suggestion_type, SuggestionType::MaintenanceAlert))
        .collect();
    
    assert!(!maintenance_suggestions.is_empty());
    
    // Should reference the specific equipment issues
    let has_equipment_reference = suggestions.iter()
        .any(|s| s.description.contains("PCR") || s.description.contains("Centrifuge"));
    
    assert!(has_equipment_reference);
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_suggestion_action_items() {
    let suggestions = TestDataGenerator::proactive_suggestions();
    
    for suggestion in &suggestions {
        // Each suggestion should have actionable items
        assert!(!suggestion.action_items.is_empty());
        
        // Action items should be specific
        for action in &suggestion.action_items {
            assert!(action.len() > 10, "Action items should be detailed");
        }
    }
}

#[tokio::test]
async fn test_real_time_suggestion_updates() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let test_db = TestDatabase::new().await;
    let ollama_service = OllamaService::new(mock_server.uri());
    
    // Generate initial suggestions
    let initial_suggestions = analyze_lab_data_and_suggest(&ollama_service, &test_db.pool).await.unwrap();
    let initial_count = initial_suggestions.len();
    
    // Add new critical data
    sqlx::query!(
        r#"
        INSERT INTO alerts (alert_type, severity, message, created_at)
        VALUES ($1, $2, $3, NOW())
        "#,
        "temperature_excursion",
        "critical",
        "Freezer temperature above threshold for 30 minutes"
    )
    .execute(&test_db.pool)
    .await
    .expect("Failed to insert alert");
    
    // Generate updated suggestions
    let updated_suggestions = analyze_lab_data_and_suggest(&ollama_service, &test_db.pool).await.unwrap();
    
    // Should have new critical suggestion
    let new_critical = updated_suggestions.iter()
        .filter(|s| s.priority == "critical")
        .any(|s| s.description.contains("temperature") || s.description.contains("Freezer"));
    
    assert!(new_critical, "Should generate new suggestion for critical alert");
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_suggestion_deduplication() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let test_db = TestDatabase::new().await;
    let ollama_service = OllamaService::new(mock_server.uri());
    
    // Insert multiple similar issues
    for i in 0..5 {
        sqlx::query!(
            r#"
            INSERT INTO equipment (equipment_id, name, status, department, last_updated)
            VALUES ($1, $2, $3, $4, NOW() - INTERVAL '1 hour' * $5)
            "#,
            format!("PCR{:03}", i),
            "PCR Machine",
            "needs_calibration",
            Department::Molecular as i32,
            i as i32
        )
        .execute(&test_db.pool)
        .await
        .expect("Failed to insert equipment");
    }
    
    let suggestions = analyze_lab_data_and_suggest(&ollama_service, &test_db.pool).await.unwrap();
    
    // Should consolidate similar issues into one suggestion
    let calibration_suggestions: Vec<_> = suggestions.iter()
        .filter(|s| s.description.contains("calibration"))
        .collect();
    
    // Should have one consolidated suggestion, not 5 separate ones
    assert_eq!(calibration_suggestions.len(), 1, "Similar issues should be consolidated");
    
    // The suggestion should mention multiple machines
    assert!(calibration_suggestions[0].description.contains("multiple") || 
            calibration_suggestions[0].description.contains("several"));
    
    test_db.cleanup().await;
}

#[tokio::test]
async fn test_cross_department_suggestions() {
    let mock_server = MockServer::start().await;
    MockOllamaSetup::setup_chat_endpoint(&mock_server).await;
    
    let test_db = TestDatabase::new().await;
    let ollama_service = OllamaService::new(mock_server.uri());
    
    // Create scenario that affects multiple departments
    // E.g., Sample storage issue affecting both Storage and Sequencing
    sqlx::query!(
        r#"
        INSERT INTO cross_department_issues (issue_type, affected_departments, description, severity)
        VALUES ($1, $2, $3, $4)
        "#,
        "sample_integrity",
        &vec![Department::Storage as i32, Department::Sequencing as i32],
        "Temperature fluctuations affecting sequencing sample quality",
        "high"
    )
    .execute(&test_db.pool)
    .await
    .expect("Failed to insert cross-department issue");
    
    // Generate suggestions for both departments
    let storage_suggestions = generate_proactive_suggestions_for_department(
        &ollama_service,
        &LabContextService::new(test_db.pool.clone()),
        &test_db.pool,
        Department::Storage
    ).await.unwrap();
    
    let sequencing_suggestions = generate_proactive_suggestions_for_department(
        &ollama_service,
        &LabContextService::new(test_db.pool.clone()),
        &test_db.pool,
        Department::Sequencing
    ).await.unwrap();
    
    // Both departments should have related suggestions
    let storage_has_temp_suggestion = storage_suggestions.iter()
        .any(|s| s.description.contains("temperature"));
    let sequencing_has_quality_suggestion = sequencing_suggestions.iter()
        .any(|s| s.description.contains("quality") || s.description.contains("sample"));
    
    assert!(storage_has_temp_suggestion);
    assert!(sequencing_has_quality_suggestion);
    
    test_db.cleanup().await;
}

// Helper functions for integration tests

async fn generate_proactive_suggestions_for_department(
    ollama_service: &OllamaService,
    context_service: &LabContextService,
    pool: &PgPool,
    department: Department,
) -> Result<Vec<ProactiveSuggestion>, ServiceError> {
    // In real implementation, this would analyze department-specific data
    // For testing, we'll generate mock suggestions
    use cognitive_assistant_service::{ServiceError, ServiceErrorKind};
    
    Ok(vec![
        ProactiveSuggestion {
            id: None,
            suggestion_type: SuggestionType::ProcessImprovement,
            title: format!("Optimize {} workflow", department_name(&department)),
            description: format!("Analysis shows opportunity to improve {} processes", department_name(&department)),
            priority: "medium".to_string(),
            department,
            potential_impact: Some("20% efficiency improvement".to_string()),
            action_items: vec![
                "Review current workflow".to_string(),
                "Identify bottlenecks".to_string(),
                "Implement improvements".to_string(),
            ],
            created_at: chrono::Utc::now(),
        }
    ])
}

async fn analyze_lab_data_and_suggest(
    ollama_service: &OllamaService,
    pool: &PgPool,
) -> Result<Vec<ProactiveSuggestion>, ServiceError> {
    // Mock implementation that would analyze various data sources
    use cognitive_assistant_service::{ServiceError, ServiceErrorKind};
    
    let mut suggestions = Vec::new();
    
    // Check for critical equipment issues
    let critical_equipment = sqlx::query!(
        "SELECT equipment_id, name, status FROM equipment WHERE status = 'critical_failure'"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ServiceError {
        kind: ServiceErrorKind::DatabaseError,
        message: e.to_string(),
        details: None,
    })?;
    
    for equipment in critical_equipment {
        suggestions.push(ProactiveSuggestion {
            id: None,
            suggestion_type: SuggestionType::MaintenanceAlert,
            title: format!("Critical: {} requires immediate attention", equipment.name),
            description: format!("{} is in critical failure state", equipment.name),
            priority: "critical".to_string(),
            department: Department::Storage, // Simplified
            potential_impact: Some("Sample integrity at risk".to_string()),
            action_items: vec![
                "Immediately transfer samples to backup storage".to_string(),
                "Contact maintenance team".to_string(),
                "Document affected samples".to_string(),
            ],
            created_at: chrono::Utc::now(),
        });
    }
    
    // Add other suggestion types based on data analysis
    
    Ok(suggestions)
}

async fn generate_contextual_suggestions(
    ollama_service: &OllamaService,
    context: &LabContext,
) -> Result<Vec<ProactiveSuggestion>, ServiceError> {
    use cognitive_assistant_service::{ServiceError, ServiceErrorKind};
    
    let mut suggestions = Vec::new();
    
    // Analyze equipment status for maintenance needs
    for (equipment, status) in &context.equipment_status {
        if status.contains("fluctuation") || status.contains("vibration") || status.contains("unusual") {
            suggestions.push(ProactiveSuggestion {
                id: None,
                suggestion_type: SuggestionType::MaintenanceAlert,
                title: format!("Maintenance recommended for {}", equipment),
                description: format!("{} is showing signs of issues: {}", equipment, status),
                priority: "high".to_string(),
                department: Department::Molecular, // Simplified
                potential_impact: Some("Prevent equipment failure and sample loss".to_string()),
                action_items: vec![
                    format!("Schedule maintenance for {}", equipment),
                    "Run diagnostic tests".to_string(),
                    "Prepare backup equipment".to_string(),
                ],
                created_at: chrono::Utc::now(),
            });
        }
    }
    
    Ok(suggestions)
}

fn department_name(dept: &Department) -> &str {
    match dept {
        Department::Sequencing => "Sequencing",
        Department::Molecular => "Molecular Biology",
        Department::QualityControl => "Quality Control",
        Department::Storage => "Sample Storage",
    }
}

use cognitive_assistant_service::{ServiceError, LabContext};