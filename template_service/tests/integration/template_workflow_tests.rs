use crate::test_utils::*;
use template_service::{
    models::*,
    handlers::*,
    services::*,
    create_app,
};
use axum_test::TestServer;
use serde_json::json;
use uuid::Uuid;

/// Integration tests for complete template and document generation workflows
#[tokio::test]
async fn test_complete_document_generation_lifecycle() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = TemplateTestClient::new(app);

    // Phase 1: Create document template
    let template_request = TemplateFactory::create_valid_template_request();
    let template_name = template_request.name.clone();
    
    let response = client.post_json("/api/templates", &template_request).await;
    TemplateAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let template_data: serde_json::Value = response.json();
    TemplateAssertions::assert_template_data(&template_data, &template_name);
    
    let template_id = Uuid::parse_str(template_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_template(template_id);

    // Phase 2: Validate template content
    let validation_request = json!({
        "template_id": template_id,
        "check_syntax": true,
        "check_variables": true
    });
    
    let response = client.post_json("/api/templates/validate", &validation_request).await;
    TemplateAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let validation_data: serde_json::Value = response.json();
    TemplateAssertions::assert_template_validation(&validation_data);
    assert_eq!(validation_data["data"]["valid"], true);

    // Phase 3: Generate document with sample data
    let mut generation_request = TemplateFactory::create_generation_request();
    generation_request.template_id = template_id;
    generation_request.data = TemplateFactory::sample_template_data();
    generation_request.output_format = OutputFormat::PDF;
    
    let response = client.post_json("/api/templates/generate", &generation_request).await;
    TemplateAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let generation_data: serde_json::Value = response.json();
    TemplateAssertions::assert_document_generation(&generation_data);
    
    let document_id = Uuid::parse_str(generation_data["data"]["document_id"].as_str().unwrap()).unwrap();
    test_db.track_document(document_id);

    // Phase 4: Download generated document
    let response = client.get(&format!("/api/templates/documents/{}/download", document_id)).await;
    TemplateAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let pdf_content = response.bytes();
    TemplateFileUtils::assert_pdf_structure(&pdf_content);

    // Phase 5: Generate multiple formats from same template
    let formats = vec![OutputFormat::HTML, OutputFormat::DOCX];
    for format in formats {
        let mut format_request = generation_request.clone();
        format_request.output_format = format;
        format_request.file_name = Some(format!("test_document.{}", format!("{:?}", format).to_lowercase()));
        
        let response = client.post_json("/api/templates/generate", &format_request).await;
        TemplateAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
        
        let format_data: serde_json::Value = response.json();
        TemplateAssertions::assert_document_generation(&format_data);
        
        let format_document_id = Uuid::parse_str(format_data["data"]["document_id"].as_str().unwrap()).unwrap();
        test_db.track_document(format_document_id);
    }

    // Phase 6: Query document generation history
    let response = client.get(&format!("/api/templates/{}/documents", template_id)).await;
    TemplateAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let history_data: serde_json::Value = response.json();
    assert_eq!(history_data["success"], true);
    assert!(history_data["data"]["documents"].is_array());
    
    let documents = history_data["data"]["documents"].as_array().unwrap();
    assert_eq!(documents.len(), 3); // PDF + HTML + DOCX

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_spreadsheet_template_generation() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = TemplateTestClient::new(app);

    // Create spreadsheet template
    let spreadsheet_template = TemplateFactory::create_spreadsheet_template_request();
    let template_name = spreadsheet_template.name.clone();
    
    let response = client.post_json("/api/templates", &spreadsheet_template).await;
    TemplateAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let template_data: serde_json::Value = response.json();
    TemplateAssertions::assert_template_data(&template_data, &template_name);
    
    let template_id = Uuid::parse_str(template_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_template(template_id);

    // Generate spreadsheet with bulk data
    let bulk_data = TemplateTestDataGenerator::generate_bulk_data(1000);
    let generation_request = GenerateDocumentRequest {
        template_id,
        data: bulk_data,
        output_format: OutputFormat::XLSX,
        file_name: Some("sample_data_export.xlsx".to_string()),
        options: Some(GenerationOptions {
            include_header: true,
            include_footer: false,
            page_size: PageSize::A4,
            orientation: PageOrientation::Landscape,
            margins: PageMargins {
                top: 15.0,
                bottom: 15.0,
                left: 20.0,
                right: 20.0,
            },
            font_size: 10,
            font_family: "Calibri".to_string(),
        }),
    };
    
    let start_time = std::time::Instant::now();
    let response = client.post_json("/api/templates/generate", &generation_request).await;
    let generation_time = start_time.elapsed();
    
    TemplateAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let generation_data: serde_json::Value = response.json();
    TemplateAssertions::assert_document_generation(&generation_data);
    
    let document_id = Uuid::parse_str(generation_data["data"]["document_id"].as_str().unwrap()).unwrap();
    test_db.track_document(document_id);

    // Verify generation performance
    assert!(generation_time.as_secs() < 30, "Large spreadsheet generation should complete within 30 seconds");

    // Download and validate spreadsheet
    let response = client.get(&format!("/api/templates/documents/{}/download", document_id)).await;
    TemplateAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let xlsx_content = response.bytes();
    TemplateFileUtils::assert_xlsx_structure(&xlsx_content);
    
    let file_size = xlsx_content.len();
    assert!(file_size > 10000, "Spreadsheet should have substantial content");

    // Generate CSV version for comparison
    let csv_request = GenerateDocumentRequest {
        template_id,
        data: TemplateTestDataGenerator::generate_bulk_data(100), // Smaller dataset for CSV
        output_format: OutputFormat::CSV,
        file_name: Some("sample_data_export.csv".to_string()),
        options: None,
    };
    
    let response = client.post_json("/api/templates/generate", &csv_request).await;
    let csv_data: serde_json::Value = response.json();
    let csv_document_id = Uuid::parse_str(csv_data["data"]["document_id"].as_str().unwrap()).unwrap();
    test_db.track_document(csv_document_id);

    // Download and validate CSV
    let response = client.get(&format!("/api/templates/documents/{}/download", csv_document_id)).await;
    let csv_content = String::from_utf8(response.bytes()).unwrap();
    TemplateFileUtils::assert_csv_structure(&csv_content);

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_laboratory_report_generation() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = TemplateTestClient::new(app);

    // Create comprehensive laboratory report template
    let report_template = CreateTemplateRequest {
        name: "Laboratory Analysis Report".to_string(),
        description: Some("Comprehensive laboratory analysis report with QC data".to_string()),
        template_type: TemplateType::Report,
        category: TemplateCategory::AnalysisReport,
        content: r#"
<!DOCTYPE html>
<html>
<head>
    <title>{{report.title}}</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .header { text-align: center; border-bottom: 3px solid #007acc; padding-bottom: 20px; margin-bottom: 30px; }
        .section { margin-bottom: 25px; page-break-inside: avoid; }
        .sample-info { background: #f8f9fa; padding: 15px; border-radius: 5px; }
        .results-table { width: 100%; border-collapse: collapse; margin: 15px 0; }
        .results-table th, .results-table td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        .results-table th { background-color: #007acc; color: white; }
        .pass { color: #28a745; font-weight: bold; }
        .fail { color: #dc3545; font-weight: bold; }
        .warning { color: #ffc107; font-weight: bold; }
        .qc-section { border-left: 4px solid #007acc; padding-left: 15px; }
        .footer { margin-top: 50px; padding-top: 20px; border-top: 1px solid #ddd; font-size: 12px; color: #666; }
    </style>
</head>
<body>
    <div class="header">
        <h1>{{report.title}}</h1>
        <h2>{{laboratory.name}}</h2>
        <p>Report Generated: {{report.generated_date}}</p>
        <p>Report ID: {{report.id}}</p>
    </div>

    <div class="section sample-info">
        <h3>Sample Information</h3>
        <table class="results-table">
            <tr><td><strong>Sample ID:</strong></td><td>{{sample.id}}</td></tr>
            <tr><td><strong>Sample Name:</strong></td><td>{{sample.name}}</td></tr>
            <tr><td><strong>Sample Type:</strong></td><td>{{sample.type}}</td></tr>
            <tr><td><strong>Collection Date:</strong></td><td>{{sample.collection_date}}</td></tr>
            <tr><td><strong>Received Date:</strong></td><td>{{sample.received_date}}</td></tr>
            <tr><td><strong>Analysis Completed:</strong></td><td>{{sample.analysis_completed}}</td></tr>
        </table>
    </div>

    <div class="section">
        <h3>Analysis Results</h3>
        <table class="results-table">
            <thead>
                <tr>
                    <th>Parameter</th>
                    <th>Result</th>
                    <th>Unit</th>
                    <th>Reference Range</th>
                    <th>Status</th>
                    <th>Method</th>
                </tr>
            </thead>
            <tbody>
                {{#each results}}
                <tr>
                    <td>{{parameter}}</td>
                    <td>{{result}}</td>
                    <td>{{unit}}</td>
                    <td>{{reference_range}}</td>
                    <td class="{{status_class}}">{{status}}</td>
                    <td>{{method}}</td>
                </tr>
                {{/each}}
            </tbody>
        </table>
    </div>

    <div class="section qc-section">
        <h3>Quality Control</h3>
        {{#each qc_data}}
        <h4>{{qc_type}}</h4>
        <table class="results-table">
            <thead>
                <tr>
                    <th>QC Parameter</th>
                    <th>Expected</th>
                    <th>Observed</th>
                    <th>% Recovery</th>
                    <th>Status</th>
                </tr>
            </thead>
            <tbody>
                {{#each parameters}}
                <tr>
                    <td>{{name}}</td>
                    <td>{{expected}}</td>
                    <td>{{observed}}</td>
                    <td>{{recovery}}</td>
                    <td class="{{status_class}}">{{status}}</td>
                </tr>
                {{/each}}
            </tbody>
        </table>
        {{/each}}
    </div>

    <div class="section">
        <h3>Summary and Interpretation</h3>
        <p><strong>Overall Result:</strong> <span class="{{overall_status_class}}">{{overall_status}}</span></p>
        
        {{#if comments}}
        <h4>Comments:</h4>
        <ul>
            {{#each comments}}
            <li>{{this}}</li>
            {{/each}}
        </ul>
        {{/if}}

        {{#if recommendations}}
        <h4>Recommendations:</h4>
        <ul>
            {{#each recommendations}}
            <li>{{this}}</li>
            {{/each}}
        </ul>
        {{/if}}
    </div>

    <div class="footer">
        <p><strong>Analyzed by:</strong> {{analyst.name}} ({{analyst.credentials}})</p>
        <p><strong>Reviewed by:</strong> {{reviewer.name}} ({{reviewer.credentials}})</p>
        <p><strong>Laboratory:</strong> {{laboratory.name}} | {{laboratory.address}}</p>
        <p><strong>Accreditation:</strong> {{laboratory.accreditation}}</p>
        <p>This report contains confidential information and is intended solely for the use of the client.</p>
    </div>
</body>
</html>
        "#.trim().to_string(),
        schema: Some(json!({
            "type": "object",
            "properties": {
                "report": {
                    "type": "object",
                    "properties": {
                        "title": {"type": "string"},
                        "id": {"type": "string"},
                        "generated_date": {"type": "string"}
                    }
                },
                "laboratory": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "address": {"type": "string"},
                        "accreditation": {"type": "string"}
                    }
                },
                "sample": {"type": "object"},
                "results": {"type": "array"},
                "qc_data": {"type": "array"},
                "overall_status": {"type": "string"},
                "analyst": {"type": "object"},
                "reviewer": {"type": "object"}
            }
        })),
        is_active: true,
        version: "1.0.0".to_string(),
        tags: Some(vec!["laboratory".to_string(), "analysis".to_string(), "report".to_string()]),
    };
    
    let response = client.post_json("/api/templates", &report_template).await;
    let template_data: serde_json::Value = response.json();
    let template_id = Uuid::parse_str(template_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_template(template_id);

    // Generate report with comprehensive laboratory data
    let laboratory_data = json!({
        "report": {
            "title": "Comprehensive Laboratory Analysis Report",
            "id": "RPT-2024-001",
            "generated_date": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        },
        "laboratory": {
            "name": "TracSeq Laboratory Services",
            "address": "123 Science Drive, Research City, RC 12345",
            "accreditation": "ISO/IEC 17025:2017"
        },
        "sample": {
            "id": "SAM-2024-001",
            "name": "Patient Sample Alpha",
            "type": "Blood Serum",
            "collection_date": "2024-01-15",
            "received_date": "2024-01-16",
            "analysis_completed": "2024-01-18"
        },
        "results": [
            {
                "parameter": "Glucose",
                "result": "95",
                "unit": "mg/dL",
                "reference_range": "70-110",
                "status": "Normal",
                "status_class": "pass",
                "method": "Enzymatic"
            },
            {
                "parameter": "Cholesterol",
                "result": "220",
                "unit": "mg/dL",
                "reference_range": "<200",
                "status": "Elevated",
                "status_class": "warning",
                "method": "Enzymatic"
            },
            {
                "parameter": "Triglycerides",
                "result": "180",
                "unit": "mg/dL",
                "reference_range": "<150",
                "status": "High",
                "status_class": "fail",
                "method": "Enzymatic"
            }
        ],
        "qc_data": [
            {
                "qc_type": "Internal Standards",
                "parameters": [
                    {
                        "name": "Glucose Control",
                        "expected": "100",
                        "observed": "98",
                        "recovery": "98.0",
                        "status": "Pass",
                        "status_class": "pass"
                    },
                    {
                        "name": "Cholesterol Control",
                        "expected": "200",
                        "observed": "201",
                        "recovery": "100.5",
                        "status": "Pass",
                        "status_class": "pass"
                    }
                ]
            }
        ],
        "overall_status": "Results Reviewed - Action Required",
        "overall_status_class": "warning",
        "comments": [
            "Cholesterol and triglyceride levels are elevated",
            "Recommend dietary modification and follow-up testing"
        ],
        "recommendations": [
            "Repeat lipid panel in 3 months",
            "Consider statin therapy consultation",
            "Lifestyle modification counseling"
        ],
        "analyst": {
            "name": "Dr. Sarah Johnson",
            "credentials": "PhD, Clinical Chemistry"
        },
        "reviewer": {
            "name": "Dr. Michael Chen",
            "credentials": "MD, Laboratory Director"
        }
    });
    
    let generation_request = GenerateDocumentRequest {
        template_id,
        data: laboratory_data,
        output_format: OutputFormat::PDF,
        file_name: Some("laboratory_analysis_report.pdf".to_string()),
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
    };
    
    let response = client.post_json("/api/templates/generate", &generation_request).await;
    TemplateAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let generation_data: serde_json::Value = response.json();
    TemplateAssertions::assert_document_generation(&generation_data);
    
    let document_id = Uuid::parse_str(generation_data["data"]["document_id"].as_str().unwrap()).unwrap();
    test_db.track_document(document_id);

    // Verify document content and quality
    let response = client.get(&format!("/api/templates/documents/{}/download", document_id)).await;
    let pdf_content = response.bytes();
    TemplateFileUtils::assert_pdf_structure(&pdf_content);
    
    // Verify substantial content (comprehensive report should be large)
    assert!(pdf_content.len() > 50000, "Comprehensive report should generate substantial PDF content");

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_concurrent_document_generation() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = TemplateTestClient::new(app);

    // Create template
    let template_request = TemplateFactory::create_valid_template_request();
    let response = client.post_json("/api/templates", &template_request).await;
    let template_data: serde_json::Value = response.json();
    let template_id = Uuid::parse_str(template_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_template(template_id);

    // Test concurrent document generation
    let concurrent_count = 10;
    let concurrent_results = TemplatePerformanceUtils::concurrent_generation_test(
        &client,
        template_id,
        concurrent_count,
    ).await;
    
    let successful_generations = concurrent_results.iter()
        .filter(|&status| *status == axum::http::StatusCode::CREATED)
        .count();
    
    assert!(successful_generations >= (concurrent_count * 80 / 100), "At least 80% of concurrent generations should succeed");

    // Test different format generation performance
    let formats = TemplateTestDataGenerator::output_formats();
    let mut generation_times = Vec::new();
    
    for format in formats {
        let generation_request = GenerateDocumentRequest {
            template_id,
            data: TemplateFactory::sample_template_data(),
            output_format: format,
            file_name: Some(format!("test.{}", format!("{:?}", format).to_lowercase())),
            options: None,
        };
        
        let generation_time = TemplatePerformanceUtils::measure_generation_time(
            &client,
            &generation_request,
        ).await;
        
        generation_times.push((format, generation_time));
        
        // Clean up generated document
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    // Verify all formats generate within reasonable time
    for (format, time) in generation_times {
        assert!(time.as_secs() < 15, "Format {:?} should generate within 15 seconds", format);
    }

    test_db.cleanup().await;
}

#[tokio::test]
async fn test_template_version_management() {
    let mut test_db = TestDatabase::new().await;
    let app = create_app().await;
    let client = TemplateTestClient::new(app);

    // Create initial template version
    let mut template_request = TemplateFactory::create_valid_template_request();
    template_request.version = "1.0.0".to_string();
    
    let response = client.post_json("/api/templates", &template_request).await;
    let template_data: serde_json::Value = response.json();
    let template_id = Uuid::parse_str(template_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_template(template_id);

    // Create updated version
    let updated_template = CreateTemplateRequest {
        name: template_request.name.clone(),
        version: "1.1.0".to_string(),
        content: template_request.content + "\n<!-- Updated version -->",
        ..template_request.clone()
    };
    
    let response = client.post_json(&format!("/api/templates/{}/versions", template_id), &updated_template).await;
    TemplateAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let version_data: serde_json::Value = response.json();
    let version_id = Uuid::parse_str(version_data["data"]["id"].as_str().unwrap()).unwrap();
    test_db.track_version(version_id);

    // Get version history
    let response = client.get(&format!("/api/templates/{}/versions", template_id)).await;
    TemplateAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::OK);
    
    let history_data: serde_json::Value = response.json();
    assert_eq!(history_data["success"], true);
    assert!(history_data["data"]["versions"].is_array());
    
    let versions = history_data["data"]["versions"].as_array().unwrap();
    assert_eq!(versions.len(), 2); // Original + updated

    // Generate document from specific version
    let generation_request = GenerateDocumentRequest {
        template_id: version_id, // Use specific version
        data: TemplateFactory::sample_template_data(),
        output_format: OutputFormat::HTML,
        file_name: Some("versioned_document.html".to_string()),
        options: None,
    };
    
    let response = client.post_json("/api/templates/generate", &generation_request).await;
    TemplateAssertions::assert_status_code(response.status_code(), axum::http::StatusCode::CREATED);
    
    let generation_data: serde_json::Value = response.json();
    let document_id = Uuid::parse_str(generation_data["data"]["document_id"].as_str().unwrap()).unwrap();
    test_db.track_document(document_id);

    // Verify document contains updated content
    let response = client.get(&format!("/api/templates/documents/{}/download", document_id)).await;
    let html_content = String::from_utf8(response.bytes()).unwrap();
    assert!(html_content.contains("Updated version"), "Document should contain updated template content");

    test_db.cleanup().await;
}