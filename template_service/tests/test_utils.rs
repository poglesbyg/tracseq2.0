use template_service::{models::*, Config, TemplateService};
use axum::{http::StatusCode, Router};
use axum_test::TestServer;
use fake::{Fake, Faker};
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use tempfile::TempDir;
use std::path::PathBuf;

/// Test database manager for isolated template testing
pub struct TestDatabase {
    pub pool: PgPool,
    pub cleanup_templates: Vec<Uuid>,
    pub cleanup_documents: Vec<Uuid>,
    pub cleanup_versions: Vec<Uuid>,
}

impl TestDatabase {
    pub async fn new() -> Self {
        let pool = get_test_db().await.clone();
        Self {
            pool,
            cleanup_templates: Vec::new(),
            cleanup_documents: Vec::new(),
            cleanup_versions: Vec::new(),
        }
    }

    pub async fn cleanup(&mut self) {
        for version_id in &self.cleanup_versions {
            let _ = sqlx::query("DELETE FROM template_versions WHERE id = $1")
                .bind(version_id)
                .execute(&self.pool)
                .await;
        }

        for document_id in &self.cleanup_documents {
            let _ = sqlx::query("DELETE FROM generated_documents WHERE id = $1")
                .bind(document_id)
                .execute(&self.pool)
                .await;
        }

        for template_id in &self.cleanup_templates {
            let _ = sqlx::query("DELETE FROM document_templates WHERE id = $1")
                .bind(template_id)
                .execute(&self.pool)
                .await;
        }

        self.cleanup_templates.clear();
        self.cleanup_documents.clear();
        self.cleanup_versions.clear();
    }

    pub fn track_template(&mut self, template_id: Uuid) {
        self.cleanup_templates.push(template_id);
    }

    pub fn track_document(&mut self, document_id: Uuid) {
        self.cleanup_documents.push(document_id);
    }

    pub fn track_version(&mut self, version_id: Uuid) {
        self.cleanup_versions.push(version_id);
    }
}

/// Factory for creating test template entities
pub struct TemplateFactory;

impl TemplateFactory {
    pub fn create_valid_template_request() -> CreateTemplateRequest {
        CreateTemplateRequest {
            name: format!("Test Template {}", Faker.fake::<String>()),
            description: Some("Test template for document generation".to_string()),
            template_type: TemplateType::Document,
            category: TemplateCategory::SampleForm,
            content: Self::sample_document_template(),
            schema: Some(Self::sample_template_schema()),
            is_active: true,
            version: "1.0.0".to_string(),
            tags: Some(vec!["test".to_string(), "sample".to_string()]),
        }
    }

    pub fn create_spreadsheet_template_request() -> CreateTemplateRequest {
        CreateTemplateRequest {
            name: format!("Spreadsheet Template {}", Faker.fake::<String>()),
            template_type: TemplateType::Spreadsheet,
            category: TemplateCategory::DataExport,
            content: Self::sample_spreadsheet_template(),
            schema: Some(Self::sample_spreadsheet_schema()),
            ..Self::create_valid_template_request()
        }
    }

    pub fn create_generation_request() -> GenerateDocumentRequest {
        GenerateDocumentRequest {
            template_id: Uuid::new_v4(),
            data: Self::sample_template_data(),
            output_format: OutputFormat::PDF,
            file_name: Some("test_document.pdf".to_string()),
            options: Some(GenerationOptions {
                include_header: true,
                include_footer: true,
                page_size: PageSize::A4,
                orientation: PageOrientation::Portrait,
                margins: PageMargins {
                    top: 25.0,
                    bottom: 25.0,
                    left: 25.0,
                    right: 25.0,
                },
                font_size: 12,
                font_family: "Arial".to_string(),
            }),
        }
    }

    pub fn sample_document_template() -> String {
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>{{title}}</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .header { text-align: center; margin-bottom: 30px; }
        .section { margin-bottom: 20px; }
        .field { margin-bottom: 10px; }
        .label { font-weight: bold; }
    </style>
</head>
<body>
    <div class="header">
        <h1>{{title}}</h1>
        <h2>{{subtitle}}</h2>
    </div>
    
    <div class="section">
        <h3>Sample Information</h3>
        <div class="field">
            <span class="label">Sample ID:</span> {{sample.id}}
        </div>
        <div class="field">
            <span class="label">Name:</span> {{sample.name}}
        </div>
        <div class="field">
            <span class="label">Type:</span> {{sample.type}}
        </div>
        <div class="field">
            <span class="label">Collection Date:</span> {{sample.collection_date}}
        </div>
    </div>
    
    <div class="section">
        <h3>Processing Details</h3>
        {{#each processing_steps}}
        <div class="field">
            <span class="label">{{name}}:</span> {{description}}
        </div>
        {{/each}}
    </div>
    
    <div class="section">
        <h3>Results</h3>
        <table border="1" style="width: 100%; border-collapse: collapse;">
            <thead>
                <tr>
                    <th>Metric</th>
                    <th>Value</th>
                    <th>Unit</th>
                    <th>Status</th>
                </tr>
            </thead>
            <tbody>
                {{#each results}}
                <tr>
                    <td>{{metric}}</td>
                    <td>{{value}}</td>
                    <td>{{unit}}</td>
                    <td>{{status}}</td>
                </tr>
                {{/each}}
            </tbody>
        </table>
    </div>
</body>
</html>
        "#.trim().to_string()
    }

    pub fn sample_spreadsheet_template() -> String {
        serde_json::json!({
            "sheets": [
                {
                    "name": "Sample Data",
                    "headers": ["Sample ID", "Name", "Type", "Collection Date", "Status"],
                    "data_source": "samples",
                    "formatting": {
                        "header_style": {
                            "font_weight": "bold",
                            "background_color": "#4CAF50",
                            "text_color": "#FFFFFF"
                        },
                        "alternate_row_color": "#F5F5F5"
                    }
                },
                {
                    "name": "Results Summary",
                    "headers": ["Metric", "Count", "Percentage"],
                    "data_source": "summary",
                    "charts": [
                        {
                            "type": "pie",
                            "title": "Sample Distribution",
                            "position": "E2"
                        }
                    ]
                }
            ]
        }).to_string()
    }

    pub fn sample_template_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "Document title"
                },
                "subtitle": {
                    "type": "string",
                    "description": "Document subtitle"
                },
                "sample": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "string"},
                        "name": {"type": "string"},
                        "type": {"type": "string"},
                        "collection_date": {"type": "string", "format": "date"}
                    },
                    "required": ["id", "name", "type"]
                },
                "processing_steps": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"},
                            "description": {"type": "string"}
                        }
                    }
                },
                "results": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "metric": {"type": "string"},
                            "value": {"type": "number"},
                            "unit": {"type": "string"},
                            "status": {"type": "string"}
                        }
                    }
                }
            },
            "required": ["title", "sample"]
        })
    }

    pub fn sample_spreadsheet_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "samples": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "string"},
                            "name": {"type": "string"},
                            "type": {"type": "string"},
                            "collection_date": {"type": "string"},
                            "status": {"type": "string"}
                        }
                    }
                },
                "summary": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "metric": {"type": "string"},
                            "count": {"type": "integer"},
                            "percentage": {"type": "number"}
                        }
                    }
                }
            }
        })
    }

    pub fn sample_template_data() -> serde_json::Value {
        serde_json::json!({
            "title": "Lab Sample Analysis Report",
            "subtitle": "Comprehensive Analysis Results",
            "sample": {
                "id": "SAM-2024-001",
                "name": "Test Sample Alpha",
                "type": "DNA",
                "collection_date": "2024-01-15"
            },
            "processing_steps": [
                {
                    "name": "Extraction",
                    "description": "DNA extraction completed successfully"
                },
                {
                    "name": "Quantification",
                    "description": "Sample concentration measured at 150 ng/µL"
                },
                {
                    "name": "Quality Control",
                    "description": "Quality metrics within acceptable range"
                }
            ],
            "results": [
                {
                    "metric": "Concentration",
                    "value": 150.5,
                    "unit": "ng/µL",
                    "status": "Pass"
                },
                {
                    "metric": "Purity (260/280)",
                    "value": 1.85,
                    "unit": "ratio",
                    "status": "Pass"
                },
                {
                    "metric": "Purity (260/230)",
                    "value": 2.1,
                    "unit": "ratio",
                    "status": "Pass"
                }
            ]
        })
    }
}

/// HTTP test client wrapper for template API testing
pub struct TemplateTestClient {
    pub server: TestServer,
    pub auth_token: Option<String>,
}

impl TemplateTestClient {
    pub fn new(app: Router) -> Self {
        let server = TestServer::new(app).unwrap();
        Self {
            server,
            auth_token: None,
        }
    }

    pub fn with_auth_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    pub async fn post_json<T: serde::Serialize>(&self, path: &str, body: &T) -> axum_test::TestResponse {
        let mut request = self.server.post(path).json(body);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", format!("Bearer {}", token).parse().unwrap());
        }
        request.await
    }

    pub async fn get(&self, path: &str) -> axum_test::TestResponse {
        let mut request = self.server.get(path);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", format!("Bearer {}", token).parse().unwrap());
        }
        request.await
    }

    pub async fn put_json<T: serde::Serialize>(&self, path: &str, body: &T) -> axum_test::TestResponse {
        let mut request = self.server.put(path).json(body);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", format!("Bearer {}", token).parse().unwrap());
        }
        request.await
    }

    pub async fn delete(&self, path: &str) -> axum_test::TestResponse {
        let mut request = self.server.delete(path);
        if let Some(token) = &self.auth_token {
            request = request.add_header("Authorization", format!("Bearer {}", token).parse().unwrap());
        }
        request.await
    }
}

/// Common assertions for template testing
pub struct TemplateAssertions;

impl TemplateAssertions {
    pub fn assert_successful_creation(response: &Value) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["created_at"].is_string());
    }

    pub fn assert_template_data(response: &Value, expected_name: &str) {
        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["name"], expected_name);
        assert!(response["data"]["id"].is_string());
        assert!(response["data"]["template_type"].is_string());
    }

    pub fn assert_document_generation(response: &Value) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["document_id"].is_string());
        assert!(response["data"]["file_path"].is_string());
        assert!(response["data"]["file_size"].is_number());
        assert!(response["data"]["generated_at"].is_string());
    }

    pub fn assert_template_validation(response: &Value) {
        assert_eq!(response["success"], true);
        assert!(response["data"]["valid"].is_boolean());
        if !response["data"]["valid"].as_bool().unwrap() {
            assert!(response["data"]["errors"].is_array());
        }
    }

    pub fn assert_validation_error(response: &Value) {
        assert_eq!(response["success"], false);
        assert!(response["error"].is_string());
    }
}

/// Test data generators for template scenarios
pub struct TemplateTestDataGenerator;

impl TemplateTestDataGenerator {
    pub fn template_types() -> Vec<TemplateType> {
        vec![
            TemplateType::Document,
            TemplateType::Spreadsheet,
            TemplateType::Report,
            TemplateType::Label,
            TemplateType::Certificate,
        ]
    }

    pub fn template_categories() -> Vec<TemplateCategory> {
        vec![
            TemplateCategory::SampleForm,
            TemplateCategory::AnalysisReport,
            TemplateCategory::QualityControl,
            TemplateCategory::DataExport,
            TemplateCategory::Certificate,
        ]
    }

    pub fn output_formats() -> Vec<OutputFormat> {
        vec![
            OutputFormat::PDF,
            OutputFormat::HTML,
            OutputFormat::DOCX,
            OutputFormat::XLSX,
            OutputFormat::CSV,
        ]
    }

    pub fn page_sizes() -> Vec<PageSize> {
        vec![
            PageSize::A4,
            PageSize::A5,
            PageSize::Letter,
            PageSize::Legal,
        ]
    }

    pub fn generate_bulk_data(count: usize) -> serde_json::Value {
        let samples: Vec<serde_json::Value> = (0..count)
            .map(|i| serde_json::json!({
                "id": format!("SAM-2024-{:03}", i + 1),
                "name": format!("Sample {}", i + 1),
                "type": if i % 2 == 0 { "DNA" } else { "RNA" },
                "collection_date": "2024-01-15",
                "status": if i % 3 == 0 { "Completed" } else { "In Progress" }
            }))
            .collect();

        serde_json::json!({
            "samples": samples,
            "summary": [
                {"metric": "Total Samples", "count": count, "percentage": 100.0},
                {"metric": "DNA Samples", "count": (count + 1) / 2, "percentage": 50.0},
                {"metric": "RNA Samples", "count": count / 2, "percentage": 50.0}
            ]
        })
    }

    pub fn invalid_template_content() -> Vec<String> {
        vec![
            "".to_string(),
            "{{unclosed_variable".to_string(),
            "{{#if condition}} {{/for}}".to_string(), // Mismatched helpers
            "{{invalid-variable-name}}".to_string(),
        ]
    }
}

/// Performance testing utilities
pub struct TemplatePerformanceUtils;

impl TemplatePerformanceUtils {
    pub async fn measure_generation_time(
        client: &TemplateTestClient,
        request: &GenerateDocumentRequest,
    ) -> std::time::Duration {
        let start = std::time::Instant::now();
        let _ = client.post_json("/api/templates/generate", request).await;
        start.elapsed()
    }

    pub async fn concurrent_generation_test(
        client: &TemplateTestClient,
        template_id: Uuid,
        concurrent_count: usize,
    ) -> Vec<StatusCode> {
        let tasks: Vec<_> = (0..concurrent_count)
            .map(|i| {
                let request = GenerateDocumentRequest {
                    template_id,
                    data: TemplateFactory::sample_template_data(),
                    output_format: OutputFormat::PDF,
                    file_name: Some(format!("concurrent_test_{}.pdf", i)),
                    options: None,
                };
                async move {
                    client.post_json("/api/templates/generate", &request).await.status_code()
                }
            })
            .collect();

        futures::future::join_all(tasks).await
    }
}

/// File utilities for template testing
pub struct TemplateFileUtils;

impl TemplateFileUtils {
    pub fn create_temp_template_file(content: &str, extension: &str) -> (TempDir, PathBuf) {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join(format!("template.{}", extension));
        
        std::fs::write(&file_path, content).expect("Failed to write template file");
        
        (temp_dir, file_path)
    }

    pub fn assert_pdf_structure(pdf_bytes: &[u8]) {
        assert!(pdf_bytes.starts_with(b"%PDF"), "File should start with PDF header");
        assert!(pdf_bytes.len() > 100, "PDF should have substantial content");
    }

    pub fn assert_xlsx_structure(xlsx_bytes: &[u8]) {
        // Basic XLSX format validation
        assert!(xlsx_bytes.len() > 22, "XLSX file should have minimum size");
        // XLSX files are ZIP archives, check for ZIP signature
        assert_eq!(&xlsx_bytes[0..4], b"PK\x03\x04", "XLSX should have ZIP signature");
    }

    pub fn assert_csv_structure(csv_content: &str) {
        let lines: Vec<&str> = csv_content.lines().collect();
        assert!(!lines.is_empty(), "CSV should have at least one line");
        
        if lines.len() > 1 {
            let header_cols = lines[0].split(',').count();
            for (i, line) in lines.iter().enumerate().skip(1) {
                let cols = line.split(',').count();
                assert_eq!(
                    cols, header_cols,
                    "Line {} should have {} columns like header", i + 1, header_cols
                );
            }
        }
    }
}