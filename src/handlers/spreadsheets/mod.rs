use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    models::spreadsheet::{SpreadsheetDataset, SpreadsheetSearchQuery, SpreadsheetSearchResult},
    services::Service,
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
    pub pool_filter: Option<String>,
    pub sample_filter: Option<String>,
    pub project_filter: Option<String>,
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
    State(components): State<crate::AppComponents>,
    Query(params): Query<UploadParams>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, StatusCode> {
    let service = &components.spreadsheet_service;
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
    State(components): State<crate::AppComponents>,
    Query(params): Query<SearchParams>,
    Query(raw_params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<SpreadsheetSearchResult>>, StatusCode> {
    let service = &components.spreadsheet_service;
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
        pool_filter: params.pool_filter,
        sample_filter: params.sample_filter,
        project_filter: params.project_filter,
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
    State(components): State<crate::AppComponents>,
    Path(dataset_id): Path<Uuid>,
) -> Result<Json<ApiResponse<SpreadsheetDataset>>, StatusCode> {
    let service = &components.spreadsheet_service;
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
    State(components): State<crate::AppComponents>,
    Query(params): Query<ListParams>,
) -> Result<Json<ApiResponse<Vec<SpreadsheetDataset>>>, StatusCode> {
    let service = &components.spreadsheet_service;
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
    State(components): State<crate::AppComponents>,
    Path(dataset_id): Path<Uuid>,
) -> Result<Json<ApiResponse<u64>>, StatusCode> {
    let service = &components.spreadsheet_service;
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
    State(components): State<crate::AppComponents>,
) -> Result<Json<crate::services::ServiceHealth>, StatusCode> {
    let service = &components.spreadsheet_service;
    let health = service.health_check().await;
    Ok(Json(health))
}

/// Get supported file types
pub async fn supported_types(
    State(components): State<crate::AppComponents>,
) -> Result<Json<ApiResponse<Vec<&'static str>>>, StatusCode> {
    let service = &components.spreadsheet_service;
    let types = service.supported_file_types();
    Ok(Json(ApiResponse::success(
        types,
        "Supported file types retrieved",
    )))
}

/// Get available filters for datasets
pub async fn get_available_filters(
    State(components): State<crate::AppComponents>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<crate::models::spreadsheet::AvailableFilters>>, StatusCode> {
    let service = &components.spreadsheet_service;
    info!("Received request to get available filters");

    let dataset_id = params
        .get("dataset_id")
        .and_then(|id| Uuid::parse_str(id).ok());

    match service.get_available_filters(dataset_id).await {
        Ok(filters) => {
            info!("Successfully retrieved available filters");
            Ok(Json(ApiResponse::success(
                filters,
                "Available filters retrieved successfully",
            )))
        }
        Err(e) => {
            error!("Failed to get available filters: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Analyze dataset structure and content
pub async fn analyze_dataset(
    State(components): State<crate::AppComponents>,
    Path(dataset_id): Path<Uuid>,
) -> Result<Json<ApiResponse<crate::models::spreadsheet::DatasetAnalysis>>, StatusCode> {
    let service = &components.spreadsheet_service;
    info!("Received request to analyze dataset: {}", dataset_id);

    match service.analyze_dataset(dataset_id).await {
        Ok(analysis) => {
            info!("Successfully analyzed dataset: {}", dataset_id);
            Ok(Json(ApiResponse::success(
                analysis,
                "Dataset analysis completed successfully",
            )))
        }
        Err(sqlx::Error::RowNotFound) => {
            warn!("Dataset not found for analysis: {}", dataset_id);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!("Failed to analyze dataset {}: {}", dataset_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Analyze individual column
pub async fn analyze_column(
    State(components): State<crate::AppComponents>,
    Path((dataset_id, column_name)): Path<(Uuid, String)>,
) -> Result<Json<ApiResponse<crate::models::spreadsheet::ColumnAnalysis>>, StatusCode> {
    let service = &components.spreadsheet_service;
    info!(
        "Received request to analyze column '{}' in dataset: {}",
        column_name, dataset_id
    );

    match service.analyze_column(dataset_id, &column_name).await {
        Ok(analysis) => {
            info!(
                "Successfully analyzed column '{}' in dataset: {}",
                column_name, dataset_id
            );
            Ok(Json(ApiResponse::success(
                analysis,
                "Column analysis completed successfully",
            )))
        }
        Err(sqlx::Error::RowNotFound) => {
            warn!(
                "Dataset or column not found: {} / {}",
                dataset_id, column_name
            );
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!(
                "Failed to analyze column '{}' in dataset {}: {}",
                column_name, dataset_id, e
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
