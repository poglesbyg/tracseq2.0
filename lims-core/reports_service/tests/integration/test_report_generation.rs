//! Integration tests for report generation workflow

use crate::test_utils::*;
use reqwest::StatusCode;
use serde_json::{json, Value};
use std::time::Duration;

#[tokio::test]
async fn test_complete_report_generation_workflow() {
    let app = create_test_app().await;
    
    // Set up mock service responses
    MockServiceResponses::setup_sample_service_mocks(&app.mock_server).await;
    MockServiceResponses::setup_storage_service_mocks(&app.mock_server).await;
    MockServiceResponses::setup_sequencing_service_mocks(&app.mock_server).await;
    
    // 1. Generate a new report
    let request = ReportFactory::create_report_request();
    let response = app.post("/api/reports/generate", &request).await;
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    let report_id = body["id"].as_str().unwrap();
    
    // 2. Check report status - should be generating
    let status_response = app.get(&format!("/api/reports/{}", report_id)).await;
    assert_eq!(status_response.status(), StatusCode::OK);
    
    // 3. Simulate report generation completion
    // In real implementation, this would be done by a background job
    sqlx::query!(
        r#"
        UPDATE reports 
        SET status = 'completed', 
            completed_at = NOW(),
            file_path = $1,
            size_bytes = $2
        WHERE id = $3
        "#,
        format!("/data/reports/{}.pdf", report_id),
        1024576i64, // 1MB
        report_id
    )
    .execute(&app.test_db.pool)
    .await
    .expect("Failed to update report status");
    
    // 4. Download the completed report
    let download_response = app.get(&format!("/api/reports/{}/download", report_id)).await;
    assert_eq!(download_response.status(), StatusCode::OK);
    
    // 5. Verify report metadata
    let metadata = sqlx::query!(
        r#"
        SELECT title, template_id, status, format, file_path, size_bytes, 
               completed_at - created_at as generation_time
        FROM reports
        WHERE id = $1
        "#,
        report_id
    )
    .fetch_one(&app.test_db.pool)
    .await
    .expect("Failed to fetch report metadata");
    
    assert_eq!(metadata.status, "completed");
    assert_eq!(metadata.format, "pdf");
    assert!(metadata.file_path.is_some());
    assert!(metadata.size_bytes.unwrap() > 0);
    assert!(metadata.generation_time.is_some());
}

#[tokio::test]
async fn test_sample_analytics_report_generation() {
    let app = create_test_app().await;
    
    // Set up sample service mock
    MockServiceResponses::setup_sample_service_mocks(&app.mock_server).await;
    
    // Request sample analytics report
    let response = app.get("/api/reports/analytics/samples").await;
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let analytics: Value = response.json().await.expect("Failed to parse JSON");
    ReportAssertions::assert_analytics_structure(&analytics);
    
    // Generate report from analytics
    let report_request = json!({
        "template_id": "sample-summary",
        "title": "Sample Analytics Report",
        "parameters": {
            "start_date": "2024-01-01",
            "end_date": "2024-01-31",
            "analytics_data": analytics
        },
        "format": "pdf"
    });
    
    let generate_response = app.post("/api/reports/generate", &report_request).await;
    assert_eq!(generate_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_storage_utilization_report() {
    let app = create_test_app().await;
    
    // Set up storage service mock
    MockServiceResponses::setup_storage_service_mocks(&app.mock_server).await;
    
    // Get storage analytics
    let response = app.get("/api/reports/analytics/storage").await;
    assert_eq!(response.status(), StatusCode::OK);
    
    let storage_data: Value = response.json().await.expect("Failed to parse JSON");
    
    // Generate storage report
    let report_request = json!({
        "template_id": "storage-utilization",
        "title": "Monthly Storage Utilization Report",
        "parameters": {
            "report_date": "2024-01-31",
            "storage_data": storage_data
        },
        "format": "excel"
    });
    
    let generate_response = app.post("/api/reports/generate", &report_request).await;
    assert_eq!(generate_response.status(), StatusCode::OK);
    
    let body: Value = generate_response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["status"], "generating");
}

#[tokio::test]
async fn test_sequencing_metrics_report() {
    let app = create_test_app().await;
    
    // Set up sequencing service mock
    MockServiceResponses::setup_sequencing_service_mocks(&app.mock_server).await;
    
    // Get sequencing analytics
    let response = app.get("/api/reports/analytics/sequencing").await;
    assert_eq!(response.status(), StatusCode::OK);
    
    // Generate sequencing report with specific platform filter
    let report_request = json!({
        "template_id": "sequencing-metrics",
        "title": "Illumina Platform Performance Report",
        "parameters": {
            "platform": "illumina",
            "quality_threshold": 30
        },
        "format": "pdf"
    });
    
    let generate_response = app.post("/api/reports/generate", &report_request).await;
    assert_eq!(generate_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_financial_report_generation() {
    let app = create_test_app().await;
    
    // Get financial analytics
    let response = app.get("/api/reports/analytics/financial").await;
    assert_eq!(response.status(), StatusCode::OK);
    
    // Generate financial report
    let report_request = json!({
        "template_id": "financial-summary",
        "title": "Q1 2024 Financial Summary",
        "parameters": {
            "fiscal_period": "Q1-2024",
            "cost_center": "laboratory-operations"
        },
        "format": "excel"
    });
    
    let generate_response = app.post("/api/reports/generate", &report_request).await;
    assert_eq!(generate_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_performance_analytics_report() {
    let app = create_test_app().await;
    
    // Get performance analytics
    let response = app.get("/api/reports/analytics/performance").await;
    assert_eq!(response.status(), StatusCode::OK);
    
    let perf_data: Value = response.json().await.expect("Failed to parse JSON");
    
    // Create custom performance report
    let report_request = json!({
        "template_id": "sample-summary", // Using sample template for now
        "title": "Laboratory Performance Metrics",
        "parameters": {
            "start_date": "2024-01-01",
            "end_date": "2024-01-31",
            "performance_data": perf_data
        },
        "format": "pdf"
    });
    
    let generate_response = app.post("/api/reports/generate", &report_request).await;
    assert_eq!(generate_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_report_generation_failure_handling() {
    let app = create_test_app().await;
    
    // Don't set up mocks - simulate service failures
    
    // Generate report
    let request = ReportFactory::create_report_request();
    let response = app.post("/api/reports/generate", &request).await;
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    let report_id = body["id"].as_str().unwrap();
    
    // Simulate generation failure
    sqlx::query!(
        r#"
        UPDATE reports 
        SET status = 'failed',
            error_message = $1,
            completed_at = NOW()
        WHERE id = $2
        "#,
        "Failed to fetch data from sample service",
        report_id
    )
    .execute(&app.test_db.pool)
    .await
    .expect("Failed to update report status");
    
    // Check failed status
    let status_response = app.get(&format!("/api/reports/{}", report_id)).await;
    assert_eq!(status_response.status(), StatusCode::OK);
    
    // Verify error details in database
    let error_details = sqlx::query!(
        "SELECT status, error_message FROM reports WHERE id = $1",
        report_id
    )
    .fetch_one(&app.test_db.pool)
    .await
    .expect("Failed to fetch report");
    
    assert_eq!(error_details.status, "failed");
    assert!(error_details.error_message.is_some());
}

#[tokio::test]
async fn test_concurrent_report_generation_stress() {
    let app = create_test_app().await;
    
    // Set up mocks
    MockServiceResponses::setup_sample_service_mocks(&app.mock_server).await;
    
    // Generate multiple reports concurrently
    let results = PerformanceTestUtils::concurrent_report_generation(&app, 10).await;
    
    // All should succeed
    assert_eq!(results.len(), 10);
    for (duration, status) in &results {
        assert_eq!(*status, StatusCode::OK);
        assert!(*duration < Duration::from_secs(1)); // Should be fast
    }
    
    // Verify all reports were created
    let count = sqlx::query!("SELECT COUNT(*) as count FROM reports")
        .fetch_one(&app.test_db.pool)
        .await
        .expect("Failed to count reports");
    
    assert_eq!(count.count.unwrap(), 10);
}

#[tokio::test]
async fn test_report_generation_with_large_dataset() {
    let app = create_test_app().await;
    
    // Create a report with large parameters
    let mut large_data = Vec::new();
    for i in 0..1000 {
        large_data.push(json!({
            "sample_id": format!("SAMPLE-{:04}", i),
            "type": if i % 2 == 0 { "DNA" } else { "RNA" },
            "status": "completed",
            "quality_score": 35.0 + (i % 10) as f64
        }));
    }
    
    let report_request = json!({
        "template_id": "sample-summary",
        "title": "Large Dataset Report",
        "parameters": {
            "start_date": "2024-01-01",
            "end_date": "2024-01-31",
            "sample_data": large_data
        },
        "format": "excel"
    });
    
    let response = app.post("/api/reports/generate", &report_request).await;
    assert_eq!(response.status(), StatusCode::OK);
    
    // Verify large parameters were stored
    let body: Value = response.json().await.expect("Failed to parse JSON");
    let report_id = body["id"].as_str().unwrap();
    
    let stored_params = sqlx::query!(
        "SELECT parameters FROM reports WHERE id = $1",
        report_id
    )
    .fetch_one(&app.test_db.pool)
    .await
    .expect("Failed to fetch report");
    
    let sample_data = &stored_params.parameters["sample_data"];
    assert_eq!(sample_data.as_array().unwrap().len(), 1000);
}

#[tokio::test]
async fn test_report_generation_with_template_rendering() {
    let app = create_test_app().await;
    
    // Create a custom template with complex rendering
    let template_id = "test-render-template";
    sqlx::query!(
        r#"
        INSERT INTO report_templates (id, name, description, category, template_content, fields, parameters)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        template_id,
        "Test Render Template",
        "Template for testing rendering",
        "testing",
        r#"
        <h1>{{title}}</h1>
        <p>Report generated on {{report_date}}</p>
        
        <h2>Summary</h2>
        <ul>
        {% for item in summary_items %}
            <li>{{item.name}}: {{item.value}}</li>
        {% endfor %}
        </ul>
        
        {% if show_details %}
        <h2>Detailed Analysis</h2>
        <p>{{detailed_analysis}}</p>
        {% endif %}
        "#,
        json!(["title", "report_date", "summary_items", "detailed_analysis"]),
        json!({
            "title": "required",
            "report_date": "required",
            "summary_items": "required",
            "show_details": "optional",
            "detailed_analysis": "optional"
        })
    )
    .execute(&app.test_db.pool)
    .await
    .expect("Failed to insert template");
    
    // Generate report with template
    let report_request = json!({
        "template_id": template_id,
        "title": "Complex Template Report",
        "parameters": {
            "title": "Laboratory Performance Summary",
            "report_date": "2024-01-31",
            "summary_items": [
                {"name": "Total Samples", "value": "1,500"},
                {"name": "Success Rate", "value": "96.7%"},
                {"name": "Average TAT", "value": "48.5 hours"}
            ],
            "show_details": true,
            "detailed_analysis": "Performance exceeded expectations with improved turnaround times."
        },
        "format": "pdf"
    });
    
    let response = app.post("/api/reports/generate", &report_request).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_report_generation_idempotency() {
    let app = create_test_app().await;
    
    // Generate same report multiple times with idempotency key
    let idempotency_key = uuid::Uuid::new_v4().to_string();
    let request = ReportFactory::create_report_request();
    
    // First request
    let response1 = app.client
        .post(&app.url("/api/reports/generate"))
        .header("Idempotency-Key", &idempotency_key)
        .json(&request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response1.status(), StatusCode::OK);
    let body1: Value = response1.json().await.expect("Failed to parse JSON");
    
    // Second request with same idempotency key
    let response2 = app.client
        .post(&app.url("/api/reports/generate"))
        .header("Idempotency-Key", &idempotency_key)
        .json(&request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response2.status(), StatusCode::OK);
    let body2: Value = response2.json().await.expect("Failed to parse JSON");
    
    // Should return same report ID (when idempotency is implemented)
    // For now, this documents expected behavior
    println!("Report ID 1: {}", body1["id"]);
    println!("Report ID 2: {}", body2["id"]);
}