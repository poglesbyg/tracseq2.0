use axum::{routing::get, Router};
use lab_manager::{
    handlers::spreadsheets,
    models::spreadsheet::{SpreadsheetDataManager, SpreadsheetSearchQuery},
    services::{spreadsheet_service::SpreadsheetService, Service},
};
use sqlx::PgPool;
use std::collections::HashMap;
use tower_http::cors::CorsLayer;
use tracing::{info, Level};
use tracing_subscriber;

/// Example demonstrating how to integrate the Spreadsheet Service
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Spreadsheet Service Example");

    // Database connection (replace with your actual connection string)
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:password@localhost/lab_manager".to_string());

    let pool = PgPool::connect(&database_url).await?;
    info!("Connected to database");

    // Run migrations (in a real app, you'd run this separately)
    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("Database migrations completed");

    // Initialize the spreadsheet service
    let spreadsheet_manager = SpreadsheetDataManager::new(pool.clone());
    let spreadsheet_service = SpreadsheetService::new(spreadsheet_manager);

    // Perform health check
    let health = spreadsheet_service.health_check().await;
    info!("Spreadsheet service health: {:?}", health);

    // Create the API router
    let spreadsheet_router = spreadsheets::create_router(spreadsheet_service.clone());

    // Build the main application router
    let app = Router::new()
        .route("/", get(|| async { "Lab Manager Spreadsheet Service API" }))
        .nest("/api/spreadsheets", spreadsheet_router)
        .layer(CorsLayer::permissive());

    info!("API routes configured");

    // Example of programmatic usage (without HTTP)
    demonstrate_service_usage(spreadsheet_service).await?;

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Server listening on http://0.0.0.0:3000");

    println!("\n=== API Endpoints ===");
    println!("POST   /api/spreadsheets/upload          - Upload CSV/Excel files");
    println!("GET    /api/spreadsheets/search          - Search data across all uploads");
    println!("GET    /api/spreadsheets/datasets        - List all datasets");
    println!("GET    /api/spreadsheets/datasets/:id    - Get specific dataset");
    println!("DELETE /api/spreadsheets/datasets/:id    - Delete dataset");
    println!("GET    /api/spreadsheets/health          - Service health check");
    println!("GET    /api/spreadsheets/supported-types - Get supported file types");

    println!("\n=== Example Usage ===");
    println!("# Upload a file:");
    println!("curl -X POST http://localhost:3000/api/spreadsheets/upload \\");
    println!("  -F \"file=@sample_data.csv\" \\");
    println!("  -F \"uploaded_by=researcher@lab.com\"");

    println!("\n# Search for data:");
    println!("curl \"http://localhost:3000/api/spreadsheets/search?search_term=LAB001&limit=10\"");

    println!("\n# Search with column filters:");
    println!("curl \"http://localhost:3000/api/spreadsheets/search?filter_Department=Oncology&filter_Priority=High\"");

    println!("\n# List all datasets:");
    println!("curl http://localhost:3000/api/spreadsheets/datasets");

    axum::serve(listener, app).await?;

    Ok(())
}

/// Demonstrate programmatic usage of the spreadsheet service
async fn demonstrate_service_usage(
    service: SpreadsheetService,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Demonstrating Spreadsheet Service Usage ===");

    // Example CSV data (simulating the lab template)
    let csv_data = r#"Sample_ID,Patient_ID,Sample_Type,Collection_Date,Submitter,Department,Priority,Analysis_Type,Storage_Temp,Volume_mL,Concentration_ng_uL,Quality_Score,Notes
LAB20240001,P12345,Blood,2024-01-15,Dr. Smith,Oncology,High,WGS,-80°C,2.5,250.5,8.2,Rush analysis needed
LAB20240002,P23456,Saliva,2024-01-14,Dr. Johnson,Cardiology,Medium,WES,-80°C,1.8,180.3,7.9,Standard processing
LAB20240003,P34567,Tissue,2024-01-13,Dr. Brown,Neurology,High,Targeted Panel,-80°C,3.2,320.7,8.5,Handle with care"#;

    // 1. Test CSV parsing
    info!("1. Testing CSV parsing...");
    let parsed_data = service.parse_csv_data(csv_data.as_bytes())?;
    info!(
        "Parsed {} rows with {} columns",
        parsed_data.total_rows, parsed_data.total_columns
    );
    info!("Headers: {:?}", parsed_data.headers);

    // 2. Test file type detection
    info!("2. Testing file type detection...");
    let test_files = vec!["data.csv", "results.xlsx", "analysis.xls", "unknown.txt"];
    for filename in test_files {
        match service.detect_file_type(filename) {
            Some(file_type) => info!("File '{}' detected as: {}", filename, file_type),
            None => info!("File '{}' has unsupported type", filename),
        }
    }

    // 3. Test supported file types
    info!(
        "3. Supported file types: {:?}",
        service.supported_file_types()
    );

    // 4. Simulate file upload and processing
    info!("4. Testing file upload simulation...");
    match service
        .process_upload(
            "demo_data.csv".to_string(),
            "Sample Lab Data.csv".to_string(),
            csv_data.as_bytes().to_vec(),
            "csv".to_string(),
            None,
            Some("example_user@lab.com".to_string()),
        )
        .await
    {
        Ok(dataset) => {
            info!("Successfully processed upload. Dataset ID: {}", dataset.id);
            info!("Dataset status: {:?}", dataset.upload_status);
            info!(
                "Total rows: {}, Total columns: {}",
                dataset.total_rows, dataset.total_columns
            );

            // 5. Test search functionality
            info!("5. Testing search functionality...");

            // Simple text search
            let search_query = SpreadsheetSearchQuery {
                search_term: Some("LAB20240001".to_string()),
                dataset_id: Some(dataset.id),
                column_filters: None,
                limit: Some(10),
                offset: None,
            };

            match service.search_data(search_query).await {
                Ok(results) => {
                    info!("Text search found {} records", results.records.len());
                    for record in &results.records[..2.min(results.records.len())] {
                        info!("Record {}: {:?}", record.row_number, record.row_data);
                    }
                }
                Err(e) => info!("Search failed: {}", e),
            }

            // Column filter search
            let mut column_filters = HashMap::new();
            column_filters.insert("Department".to_string(), "Oncology".to_string());

            let filter_query = SpreadsheetSearchQuery {
                search_term: None,
                dataset_id: Some(dataset.id),
                column_filters: Some(column_filters),
                limit: Some(10),
                offset: None,
            };

            match service.search_data(filter_query).await {
                Ok(results) => {
                    info!(
                        "Column filter search found {} records",
                        results.records.len()
                    );
                }
                Err(e) => info!("Filter search failed: {}", e),
            }

            // 6. Test dataset operations
            info!("6. Testing dataset operations...");

            // List datasets
            match service.list_datasets(Some(10), None).await {
                Ok(datasets) => {
                    info!("Found {} datasets", datasets.len());
                    for ds in &datasets[..1.min(datasets.len())] {
                        info!("Dataset: {} ({})", ds.original_filename, ds.file_type);
                    }
                }
                Err(e) => info!("Failed to list datasets: {}", e),
            }

            // Get specific dataset
            match service.get_dataset(dataset.id).await {
                Ok(retrieved_dataset) => {
                    info!("Retrieved dataset: {}", retrieved_dataset.original_filename);
                }
                Err(e) => info!("Failed to get dataset: {}", e),
            }

            // Clean up - delete the test dataset
            info!("7. Cleaning up test data...");
            match service.delete_dataset(dataset.id).await {
                Ok(rows_deleted) => {
                    info!("Deleted {} dataset record(s)", rows_deleted);
                }
                Err(e) => info!("Failed to delete dataset: {}", e),
            }
        }
        Err(e) => {
            info!("Upload simulation failed: {}", e);
        }
    }

    info!("=== Service demonstration completed ===");
    Ok(())
}
