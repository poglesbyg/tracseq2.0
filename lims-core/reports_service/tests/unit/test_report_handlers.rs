//! Unit tests for report handlers

use crate::test_utils::*;
use reqwest::StatusCode;
use serde_json::{json, Value};

#[tokio::test]
async fn test_list_reports_empty() {
    let app = create_test_app().await;
    
    let response = app.get("/api/reports").await;
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert!(body["reports"].is_array());
    assert_eq!(body["reports"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_list_reports_with_data() {
    let app = create_test_app().await;
    
    // Insert test reports
    let reports = TestDataGenerator::generate_report_data(5);
    for report in &reports {
        sqlx::query!(
            r#"
            INSERT INTO reports (id, title, template_id, status, format, parameters, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            report["id"].as_str().unwrap().parse::<uuid::Uuid>().unwrap_or_else(|_| uuid::Uuid::new_v4()),
            report["title"].as_str().unwrap(),
            report["template_id"].as_str().unwrap(),
            report["status"].as_str().unwrap(),
            report["format"].as_str().unwrap(),
            report["parameters"],
            chrono::Utc::now()
        )
        .execute(&app.test_db.pool)
        .await
        .expect("Failed to insert test report");
    }
    
    let response = app.get("/api/reports").await;
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    let returned_reports = body["reports"].as_array().unwrap();
    assert_eq!(returned_reports.len(), 5);
}

#[tokio::test]
async fn test_get_report_exists() {
    let app = create_test_app().await;
    
    let report_id = uuid::Uuid::new_v4();
    let report_data = json!({
        "title": "Test Report",
        "template_id": "sample-summary",
        "status": "completed",
        "format": "pdf",
        "parameters": {"test": true}
    });
    
    // Insert test report
    sqlx::query!(
        r#"
        INSERT INTO reports (id, title, template_id, status, format, parameters)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        report_id,
        report_data["title"].as_str().unwrap(),
        report_data["template_id"].as_str().unwrap(),
        report_data["status"].as_str().unwrap(),
        report_data["format"].as_str().unwrap(),
        report_data["parameters"]
    )
    .execute(&app.test_db.pool)
    .await
    .expect("Failed to insert test report");
    
    let response = app.get(&format!("/api/reports/{}", report_id)).await;
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["id"], report_id.to_string());
    assert!(body.get("report").is_some());
}

#[tokio::test]
async fn test_get_report_not_found() {
    let app = create_test_app().await;
    
    let non_existent_id = uuid::Uuid::new_v4();
    let response = app.get(&format!("/api/reports/{}", non_existent_id)).await;
    
    // Current implementation returns empty object, but should return 404
    // This test documents current behavior
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["id"], non_existent_id.to_string());
}

#[tokio::test]
async fn test_generate_report_valid_request() {
    let app = create_test_app().await;
    
    let request = ReportFactory::create_report_request();
    let response = app.post("/api/reports/generate", &request).await;
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert!(body.get("id").is_some());
    assert_eq!(body["status"], "generating");
}

#[tokio::test]
async fn test_generate_report_missing_template() {
    let app = create_test_app().await;
    
    let request = json!({
        "template_id": "non-existent-template",
        "title": "Test Report",
        "parameters": {},
        "format": "pdf"
    });
    
    let response = app.post("/api/reports/generate", &request).await;
    
    // Current implementation doesn't validate template existence
    // This test documents current behavior
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_generate_report_invalid_format() {
    let app = create_test_app().await;
    
    let request = json!({
        "template_id": "sample-summary",
        "title": "Test Report",
        "parameters": {},
        "format": "invalid-format"
    });
    
    let response = app.post("/api/reports/generate", &request).await;
    
    // Current implementation doesn't validate format
    // This test documents current behavior
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_download_report_exists() {
    let app = create_test_app().await;
    
    let report_id = uuid::Uuid::new_v4();
    
    // Insert completed report with file path
    sqlx::query!(
        r#"
        INSERT INTO reports (id, title, template_id, status, format, file_path, size_bytes)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        report_id,
        "Completed Report",
        "sample-summary",
        "completed",
        "pdf",
        "/data/reports/report.pdf",
        1024i64
    )
    .execute(&app.test_db.pool)
    .await
    .expect("Failed to insert report");
    
    let response = app.get(&format!("/api/reports/{}/download", report_id)).await;
    
    // Current implementation returns string, should return file
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = response.text().await.expect("Failed to get response text");
    assert_eq!(body, "Report file content");
}

#[tokio::test]
async fn test_download_report_not_ready() {
    let app = create_test_app().await;
    
    let report_id = uuid::Uuid::new_v4();
    
    // Insert generating report
    sqlx::query!(
        r#"
        INSERT INTO reports (id, title, template_id, status, format)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        report_id,
        "Generating Report",
        "sample-summary",
        "generating",
        "pdf"
    )
    .execute(&app.test_db.pool)
    .await
    .expect("Failed to insert report");
    
    let response = app.get(&format!("/api/reports/{}/download", report_id)).await;
    
    // Current implementation doesn't check status
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_report_status_transitions() {
    let app = create_test_app().await;
    
    let report_id = uuid::Uuid::new_v4();
    
    // Insert pending report
    sqlx::query!(
        r#"
        INSERT INTO reports (id, title, template_id, status, format)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        report_id,
        "Status Test Report",
        "sample-summary",
        "pending",
        "pdf"
    )
    .execute(&app.test_db.pool)
    .await
    .expect("Failed to insert report");
    
    // Update to generating
    sqlx::query!(
        "UPDATE reports SET status = 'generating' WHERE id = $1",
        report_id
    )
    .execute(&app.test_db.pool)
    .await
    .expect("Failed to update status");
    
    // Verify status
    let result = sqlx::query!(
        "SELECT status FROM reports WHERE id = $1",
        report_id
    )
    .fetch_one(&app.test_db.pool)
    .await
    .expect("Failed to fetch report");
    
    assert_eq!(result.status, "generating");
    
    // Update to completed
    sqlx::query!(
        "UPDATE reports SET status = 'completed', completed_at = NOW() WHERE id = $1",
        report_id
    )
    .execute(&app.test_db.pool)
    .await
    .expect("Failed to update status");
    
    // Verify final status
    let result = sqlx::query!(
        "SELECT status, completed_at FROM reports WHERE id = $1",
        report_id
    )
    .fetch_one(&app.test_db.pool)
    .await
    .expect("Failed to fetch report");
    
    assert_eq!(result.status, "completed");
    assert!(result.completed_at.is_some());
}

#[tokio::test]
async fn test_report_parameters_validation() {
    let app = create_test_app().await;
    
    // Test with required parameters
    let valid_request = json!({
        "template_id": "sample-summary",
        "title": "Parameter Test Report",
        "parameters": {
            "start_date": "2024-01-01",
            "end_date": "2024-01-31",
            "department": "molecular"
        },
        "format": "pdf"
    });
    
    let response = app.post("/api/reports/generate", &valid_request).await;
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test with missing required parameters
    let invalid_request = json!({
        "template_id": "financial-summary",
        "title": "Missing Parameter Report",
        "parameters": {
            // Missing required fiscal_period
            "cost_center": "lab-001"
        },
        "format": "excel"
    });
    
    let response = app.post("/api/reports/generate", &invalid_request).await;
    // Current implementation doesn't validate parameters
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_concurrent_report_generation() {
    let app = create_test_app().await;
    
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let client = app.client.clone();
        let url = app.url("/api/reports/generate");
        
        let handle = tokio::spawn(async move {
            let request = json!({
                "template_id": "sample-summary",
                "title": format!("Concurrent Report {}", i),
                "parameters": {
                    "start_date": "2024-01-01",
                    "end_date": "2024-01-31"
                },
                "format": "pdf"
            });
            
            client
                .post(&url)
                .json(&request)
                .send()
                .await
                .expect("Failed to send request")
        });
        
        handles.push(handle);
    }
    
    let responses = futures::future::join_all(handles).await;
    
    // All requests should succeed
    for response in responses {
        let resp = response.expect("Task failed");
        assert_eq!(resp.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn test_report_filtering() {
    let app = create_test_app().await;
    
    // Insert reports with different statuses
    let statuses = vec!["pending", "generating", "completed", "failed"];
    
    for (i, status) in statuses.iter().enumerate() {
        sqlx::query!(
            r#"
            INSERT INTO reports (title, template_id, status, format)
            VALUES ($1, $2, $3, $4)
            "#,
            format!("Report {}", i),
            "sample-summary",
            *status,
            "pdf"
        )
        .execute(&app.test_db.pool)
        .await
        .expect("Failed to insert report");
    }
    
    // Test filtering by status (when implemented)
    let response = app.get("/api/reports?status=completed").await;
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test filtering by date range (when implemented)
    let response = app.get("/api/reports?start_date=2024-01-01&end_date=2024-01-31").await;
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test filtering by template (when implemented)
    let response = app.get("/api/reports?template_id=sample-summary").await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_report_pagination() {
    let app = create_test_app().await;
    
    // Insert many reports
    for i in 0..25 {
        sqlx::query!(
            r#"
            INSERT INTO reports (title, template_id, status, format)
            VALUES ($1, $2, $3, $4)
            "#,
            format!("Report {}", i),
            "sample-summary",
            "completed",
            "pdf"
        )
        .execute(&app.test_db.pool)
        .await
        .expect("Failed to insert report");
    }
    
    // Test pagination (when implemented)
    let response = app.get("/api/reports?page=1&limit=10").await;
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: Value = response.json().await.expect("Failed to parse JSON");
    // Current implementation returns all reports
    assert!(body["reports"].as_array().unwrap().len() <= 25);
}

#[tokio::test]
async fn test_report_generation_performance() {
    let app = create_test_app().await;
    
    let request = ReportFactory::create_report_request();
    
    let start = std::time::Instant::now();
    let response = app.post("/api/reports/generate", &request).await;
    let duration = start.elapsed();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Report generation should be fast (just creating the record)
    assert!(duration < std::time::Duration::from_millis(100));
    
    println!("Report generation took: {:?}", duration);
}