use axum::{
    routing::{get, post, put, delete},
    Router,
    Json,
};
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Starting Spreadsheet Versioning Service (Minimal)");

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8088".to_string())
        .parse::<u16>()
        .unwrap_or(8088);

    println!("📋 Port: {}", port);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/health/ready", get(health_check))
        .route("/api/v1/spreadsheets", get(list_spreadsheets))
        .route("/api/v1/spreadsheets", post(create_spreadsheet))
        .route("/api/v1/spreadsheets/:id", get(get_spreadsheet))
        .route("/api/v1/spreadsheets/:id", put(update_spreadsheet))
        .route("/api/v1/spreadsheets/:id", delete(delete_spreadsheet))
        .route("/api/v1/spreadsheets/:id/preview", get(preview_spreadsheet))
        // Frontend-expected endpoints
        .route("/api/spreadsheets/datasets", get(list_datasets))
        .route("/api/spreadsheets/datasets/:id", delete(delete_dataset));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🚀 Spreadsheet Versioning Service listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "spreadsheet-versioning-service",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now()
    }))
}

async fn list_spreadsheets() -> Json<serde_json::Value> {
    Json(json!({
        "spreadsheets": [
            {
                "id": "sheet-001",
                "name": "Sample Data Sheet",
                "status": "active",
                "created_at": chrono::Utc::now()
            }
        ],
        "total": 1
    }))
}

async fn create_spreadsheet() -> Json<serde_json::Value> {
    Json(json!({
        "id": "sheet-new",
        "created": true,
        "message": "Spreadsheet created successfully"
    }))
}

async fn get_spreadsheet() -> Json<serde_json::Value> {
    Json(json!({
        "id": "sheet-001",
        "name": "Sample Spreadsheet",
        "status": "active",
        "version": "1.0"
    }))
}

async fn update_spreadsheet() -> Json<serde_json::Value> {
    Json(json!({
        "updated": true,
        "message": "Spreadsheet updated successfully"
    }))
}

async fn delete_spreadsheet() -> Json<serde_json::Value> {
    Json(json!({
        "deleted": true,
        "message": "Spreadsheet deleted successfully"
    }))
}

async fn preview_spreadsheet() -> Json<serde_json::Value> {
    Json(json!({
        "preview": {
            "rows": 10,
            "columns": 5,
            "data": [
                ["Sample ID", "Type", "Date", "Status", "Notes"],
                ["SMPL001", "DNA", "2025-07-08", "Active", "Good quality"],
                ["SMPL002", "RNA", "2025-07-08", "Processing", "High concentration"]
            ]
        },
        "message": "Preview generated successfully"
    }))
}

async fn list_datasets() -> Json<serde_json::Value> {
    Json(json!({
        "data": [
            {
                "id": "dataset-001",
                "name": "Sample Metadata",
                "original_filename": "sample_metadata.xlsx",
                "file_type": "xlsx",
                "file_size": 45672,
                "total_rows": 150,
                "total_columns": 8,
                "column_headers": ["Sample ID", "Type", "Date", "Status", "Notes", "Concentration", "Volume", "Quality"],
                "upload_status": "completed",
                "uploaded_by": "lab_technician",
                "created_at": "2025-07-08T10:30:00Z",
                "sheet_name": "Samples"
            },
            {
                "id": "dataset-002", 
                "name": "QC Results",
                "original_filename": "qc_results.csv",
                "file_type": "csv",
                "file_size": 23456,
                "total_rows": 75,
                "total_columns": 6,
                "column_headers": ["Sample ID", "QC Score", "Pass/Fail", "Date", "Technician", "Notes"],
                "upload_status": "completed",
                "uploaded_by": "qc_specialist",
                "created_at": "2025-07-08T14:15:00Z"
            },
            {
                "id": "dataset-003",
                "name": "Library Prep Data",
                "original_filename": "library_prep.xlsx",
                "file_type": "xlsx", 
                "file_size": 34567,
                "total_rows": 200,
                "total_columns": 10,
                "column_headers": ["Sample ID", "Library ID", "Protocol", "Date", "Concentration", "Volume", "Index", "Status", "Notes", "Technician"],
                "upload_status": "processing",
                "uploaded_by": "lab_manager",
                "created_at": "2025-07-08T15:00:00Z",
                "sheet_name": "Library Prep"
            }
        ],
        "total": 3,
        "message": "Datasets retrieved successfully"
    }))
}

async fn delete_dataset() -> Json<serde_json::Value> {
    Json(json!({
        "deleted": true,
        "message": "Dataset deleted successfully"
    }))
} 