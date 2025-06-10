use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    models::spreadsheet::{SpreadsheetDataset, SpreadsheetSearchQuery, SpreadsheetSearchResult},
    services::{spreadsheet_service::SpreadsheetService, Service},
};

/// Upload request parameters
#[derive(Debug, Deserialize)]
pub struct UploadParams {
    pub sheet_name: Option<String>,
    pub uploaded_by: Option<String>,
}

/// Upload response
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub success: bool,
    pub dataset: Option<SpreadsheetDataset>,
    pub message: String,
}

/// Search request parameters
#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub search_term: Option<String>,
    pub dataset_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    // Column filters as query parameters like: ?filter_Sample_ID=LAB001&filter_Department=Oncology
}

/// List datasets parameters
#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Generic API response
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: &str) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: message.to_string(),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            message: message.to_string(),
        }
    }
}

/// Upload spreadsheet file
pub async fn upload_spreadsheet(
    State(service): State<SpreadsheetService>,
    Query(params): Query<UploadParams>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, StatusCode> {
    info!("Received spreadsheet upload request");

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Failed to read multipart field: {}", e);
        StatusCode::BAD_REQUEST
    })? {
        if field.name() == Some("file") {
            let filename = field.file_name().unwrap_or("unknown").to_string();
            let file_data = field.bytes().await.map_err(|e| {
                error!("Failed to read file data: {}", e);
                StatusCode::BAD_REQUEST
            })?;

            // Detect file type from filename
            let file_type = match service.detect_file_type(&filename) {
                Some(ft) => ft,
                None => {
                    warn!("Unsupported file type for file: {}", filename);
                    return Ok(Json(UploadResponse {
                        success: false,
                        dataset: None,
                        message: format!(
                            "Unsupported file type. Supported types: {:?}",
                            service.supported_file_types()
                        ),
                    }));
                }
            };

            // Validate file type
            if !service.is_supported_file_type(&file_type) {
                warn!("File type not supported: {}", file_type);
                return Ok(Json(UploadResponse {
                    success: false,
                    dataset: None,
                    message: format!("File type '{}' not supported", file_type),
                }));
            }

            // Generate unique filename for storage
            let stored_filename = format!("{}_{}", Uuid::new_v4(), filename);

            // Process the upload
            match service
                .process_upload(
                    stored_filename,
                    filename.clone(),
                    file_data.to_vec(),
                    file_type,
                    params.sheet_name,
                    params.uploaded_by,
                )
                .await
            {
                Ok(dataset) => {
                    info!("Successfully processed upload for file: {}", filename);
                    return Ok(Json(UploadResponse {
                        success: true,
                        dataset: Some(dataset),
                        message: "File uploaded and processed successfully".to_string(),
                    }));
                }
                Err(e) => {
                    error!("Failed to process upload for file {}: {}", filename, e);
                    return Ok(Json(UploadResponse {
                        success: false,
                        dataset: None,
                        message: format!("Failed to process file: {}", e),
                    }));
                }
            }
        }
    }

    warn!("No file field found in upload request");
    Ok(Json(UploadResponse {
        success: false,
        dataset: None,
        message: "No file provided in request".to_string(),
    }))
}

/// Search spreadsheet data
pub async fn search_data(
    State(service): State<SpreadsheetService>,
    Query(params): Query<SearchParams>,
    Query(raw_params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<SpreadsheetSearchResult>>, StatusCode> {
    info!("Received data search request");

    // Extract column filters from query parameters
    let mut column_filters = HashMap::new();
    for (key, value) in raw_params {
        if key.starts_with("filter_") {
            let column_name = key.trim_start_matches("filter_");
            column_filters.insert(column_name.to_string(), value);
        }
    }

    let query = SpreadsheetSearchQuery {
        search_term: params.search_term,
        dataset_id: params.dataset_id,
        column_filters: if column_filters.is_empty() {
            None
        } else {
            Some(column_filters)
        },
        limit: params.limit,
        offset: params.offset,
    };

    match service.search_data(query).await {
        Ok(result) => {
            info!(
                "Search completed successfully, found {} records",
                result.records.len()
            );
            Ok(Json(ApiResponse::success(
                result,
                "Search completed successfully",
            )))
        }
        Err(e) => {
            error!("Search failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get dataset by ID
pub async fn get_dataset(
    State(service): State<SpreadsheetService>,
    Path(dataset_id): Path<Uuid>,
) -> Result<Json<ApiResponse<SpreadsheetDataset>>, StatusCode> {
    info!("Received request to get dataset: {}", dataset_id);

    match service.get_dataset(dataset_id).await {
        Ok(dataset) => {
            info!("Successfully retrieved dataset: {}", dataset_id);
            Ok(Json(ApiResponse::success(
                dataset,
                "Dataset retrieved successfully",
            )))
        }
        Err(sqlx::Error::RowNotFound) => {
            warn!("Dataset not found: {}", dataset_id);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!("Failed to retrieve dataset {}: {}", dataset_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List datasets
pub async fn list_datasets(
    State(service): State<SpreadsheetService>,
    Query(params): Query<ListParams>,
) -> Result<Json<ApiResponse<Vec<SpreadsheetDataset>>>, StatusCode> {
    info!("Received request to list datasets");

    match service.list_datasets(params.limit, params.offset).await {
        Ok(datasets) => {
            info!("Successfully retrieved {} datasets", datasets.len());
            Ok(Json(ApiResponse::success(
                datasets,
                "Datasets retrieved successfully",
            )))
        }
        Err(e) => {
            error!("Failed to list datasets: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete dataset
pub async fn delete_dataset(
    State(service): State<SpreadsheetService>,
    Path(dataset_id): Path<Uuid>,
) -> Result<Json<ApiResponse<u64>>, StatusCode> {
    info!("Received request to delete dataset: {}", dataset_id);

    match service.delete_dataset(dataset_id).await {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                info!("Successfully deleted dataset: {}", dataset_id);
                Ok(Json(ApiResponse::success(
                    rows_affected,
                    "Dataset deleted successfully",
                )))
            } else {
                warn!("Dataset not found for deletion: {}", dataset_id);
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            error!("Failed to delete dataset {}: {}", dataset_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get service health
pub async fn health_check(
    State(service): State<SpreadsheetService>,
) -> Result<Json<crate::services::ServiceHealth>, StatusCode> {
    let health = service.health_check().await;
    Ok(Json(health))
}

/// Get supported file types
pub async fn supported_types(
    State(service): State<SpreadsheetService>,
) -> Result<Json<ApiResponse<Vec<&'static str>>>, StatusCode> {
    let types = service.supported_file_types();
    Ok(Json(ApiResponse::success(
        types,
        "Supported file types retrieved",
    )))
}

/// Create router for spreadsheet endpoints
pub fn create_router(service: SpreadsheetService) -> Router {
    Router::new()
        .route("/upload", post(upload_spreadsheet))
        .route("/search", get(search_data))
        .route("/datasets", get(list_datasets))
        .route("/datasets/:id", get(get_dataset))
        .route("/datasets/:id", delete(delete_dataset))
        .route("/health", get(health_check))
        .route("/supported-types", get(supported_types))
        .with_state(service)
}
