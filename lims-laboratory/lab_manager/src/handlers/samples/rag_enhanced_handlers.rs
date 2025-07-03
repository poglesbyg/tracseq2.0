use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use std::path::Path;
use tokio::fs;
use uuid::Uuid;

use crate::{
    assembly::AppComponents,
    handlers::samples::{BatchCreateSamplesResponse, BatchError},
    services::rag_integration_service::{
        RagConfig, RagEnhancedSampleResult, RagIntegrationService,
    },
};

/// Process a laboratory document using RAG and create samples from extracted data
pub async fn process_document_and_create_samples(
    State(_state): State<AppComponents>,
    mut multipart: Multipart,
) -> Result<Json<RagEnhancedSampleResult>, (StatusCode, String)> {
    let start_time = std::time::Instant::now();

    // Extract file from multipart form
    let mut file_path = None;
    let mut confidence_threshold = 0.7; // Default threshold

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to read multipart field: {}", e),
        )
    })? {
        let field_name = field.name().unwrap_or("");

        match field_name {
            "file" => {
                let file_name = field
                    .file_name()
                    .ok_or((StatusCode::BAD_REQUEST, "No filename provided".to_string()))?
                    .to_string();

                // Save uploaded file temporarily
                let upload_dir = Path::new("uploads");
                fs::create_dir_all(upload_dir).await.map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to create upload directory: {}", e),
                    )
                })?;

                let file_uuid = Uuid::new_v4();
                let temp_file_path = upload_dir.join(format!("{}_{}", file_uuid, file_name));

                let file_data = field.bytes().await.map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        format!("Failed to read file data: {}", e),
                    )
                })?;

                fs::write(&temp_file_path, file_data).await.map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to save uploaded file: {}", e),
                    )
                })?;

                file_path = Some(temp_file_path.to_string_lossy().to_string());
            }
            "confidence_threshold" => {
                let threshold_str = field.text().await.map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        format!("Failed to read confidence threshold: {}", e),
                    )
                })?;

                confidence_threshold = threshold_str.parse().unwrap_or(0.7);
            }
            _ => {
                // Skip unknown fields
            }
        }
    }

    let document_path = file_path.ok_or((
        StatusCode::BAD_REQUEST,
        "No document file provided".to_string(),
    ))?;

    // Initialize RAG service using app config
    let rag_config = RagConfig {
        base_url: _state.config.rag.base_url.clone(),
        timeout_seconds: _state.config.rag.timeout_seconds,
        max_file_size_mb: _state.config.rag.max_file_size_mb,
        supported_formats: _state.config.rag.supported_formats.clone(),
    };
    let rag_service = RagIntegrationService::new(rag_config);

    // Process document with RAG
    let extraction_result = rag_service
        .process_document(&document_path)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("RAG processing failed: {}", e),
            )
        })?;

    // Clean up temporary file
    let _ = fs::remove_file(&document_path).await;

    // Check confidence threshold
    if extraction_result.confidence_score < confidence_threshold {
        return Ok(Json(RagEnhancedSampleResult {
            samples: Vec::new(),
            extraction_result: Some(extraction_result.clone()),
            confidence_score: extraction_result.confidence_score,
            validation_warnings: vec![format!(
                "Extraction confidence ({:.2}) below threshold ({:.2}). Review required.",
                extraction_result.confidence_score, confidence_threshold
            )],
            processing_time: start_time.elapsed().as_secs_f64(),
        }));
    }

    // Convert RAG results to sample format
    let mut samples = rag_service
        .convert_to_samples(&extraction_result)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to convert RAG data to samples: {}", e),
            )
        })?;

    // Validate and enhance sample data
    let mut validation_warnings = Vec::new();
    for sample in &mut samples {
        // Validate barcode length (minimum 6 characters as per lab requirements)
        if sample.barcode.len() < 6 {
            sample.barcode = format!("{:0>6}", sample.barcode); // Pad with zeros
            validation_warnings.push(format!(
                "Barcode for sample '{}' was padded to meet minimum length requirement",
                sample.name
            ));
        }

        // Validate location format
        if sample.location.is_empty() || sample.location == "Unknown-Location" {
            sample.location = "Pending-Assignment".to_string();
            validation_warnings.push(format!(
                "Storage location for sample '{}' requires manual assignment",
                sample.name
            ));
        }
    }

    // Add extraction warnings
    validation_warnings.extend(extraction_result.warnings.clone());

    let processing_time = start_time.elapsed().as_secs_f64();

    Ok(Json(RagEnhancedSampleResult {
        samples,
        extraction_result: Some(extraction_result.clone()),
        confidence_score: extraction_result.confidence_score,
        validation_warnings,
        processing_time,
    }))
}

/// Create samples from RAG-processed data with validation
pub async fn create_samples_from_rag_data(
    State(state): State<AppComponents>,
    Json(rag_result): Json<RagEnhancedSampleResult>,
) -> Result<Json<BatchCreateSamplesResponse>, (StatusCode, String)> {
    if rag_result.samples.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "No samples provided in RAG result".to_string(),
        ));
    }

    let mut created_samples = Vec::new();
    let mut errors = Vec::new();

    for (index, sample_data) in rag_result.samples.iter().enumerate() {
        // Additional validation for RAG-extracted samples
        if sample_data.name.trim().is_empty() {
            errors.push(BatchError {
                index,
                error: "Sample name cannot be empty".to_string(),
            });
            continue;
        }

        if sample_data.barcode.trim().is_empty() {
            errors.push(BatchError {
                index,
                error: "Sample barcode cannot be empty".to_string(),
            });
            continue;
        }

        if sample_data.barcode.len() < 6 {
            errors.push(BatchError {
                index,
                error: format!(
                    "Barcode '{}' must be at least 6 characters long",
                    sample_data.barcode
                ),
            });
            continue;
        }

        if sample_data.location.trim().is_empty() {
            errors.push(BatchError {
                index,
                error: "Sample location cannot be empty".to_string(),
            });
            continue;
        }

        // Create the sample
        match state
            .sample_processing
            .manager
            .create_sample(sample_data.clone())
            .await
        {
            Ok(sample) => {
                tracing::info!(
                    "Created RAG-extracted sample: {} with barcode: {} (confidence: {:.2})",
                    sample.name,
                    sample.barcode,
                    rag_result.confidence_score
                );
                created_samples.push(sample);
            }
            Err(e) => {
                let error_msg =
                    if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                        format!("Barcode '{}' already exists", sample_data.barcode)
                    } else {
                        e.to_string()
                    };

                tracing::warn!(
                    "Failed to create RAG sample at index {}: {}",
                    index,
                    error_msg
                );
                errors.push(BatchError {
                    index,
                    error: error_msg,
                });
            }
        }
    }

    let response = BatchCreateSamplesResponse {
        created: created_samples.len(),
        failed: errors.len(),
        stored_in_storage: 0, // RAG samples are not automatically stored in storage
        samples: created_samples,
        storage_errors: Vec::new(), // No storage operations attempted
        errors,
    };

    tracing::info!(
        "RAG batch creation completed: {} created, {} failed, confidence: {:.2}",
        response.created,
        response.failed,
        rag_result.confidence_score
    );

    Ok(Json(response))
}

/// Query the RAG system for information about submitted samples
pub async fn query_submission_information(
    State(_state): State<AppComponents>,
    Json(query_request): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, (StatusCode, String)> {
    // Initialize RAG service using app config
    let rag_config = RagConfig {
        base_url: _state.config.rag.base_url.clone(),
        timeout_seconds: _state.config.rag.timeout_seconds,
        max_file_size_mb: _state.config.rag.max_file_size_mb,
        supported_formats: _state.config.rag.supported_formats.clone(),
    };
    let rag_service = RagIntegrationService::new(rag_config);

    // Query the enhanced RAG system with session support
    let answer = rag_service
        .query_submissions_with_session(
            &query_request.query,
            &query_request
                .session_id
                .unwrap_or_else(|| format!("rust_session_{}", chrono::Utc::now().timestamp())),
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("RAG query failed: {}", e),
            )
        })?;

    Ok(Json(QueryResponse {
        query: query_request.query,
        answer,
        timestamp: chrono::Utc::now(),
    }))
}

/// Get RAG system health and status
pub async fn get_rag_system_status(
    State(_state): State<AppComponents>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let rag_config = RagConfig {
        base_url: _state.config.rag.base_url.clone(),
        timeout_seconds: _state.config.rag.timeout_seconds,
        max_file_size_mb: _state.config.rag.max_file_size_mb,
        supported_formats: _state.config.rag.supported_formats.clone(),
    };
    let rag_service = RagIntegrationService::new(rag_config);

    let health_data = rag_service.check_health().await.map_err(|e| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("RAG system health check failed: {}", e),
        )
    })?;

    Ok(Json(health_data))
}

/// Process document in preview mode (no sample creation)
pub async fn preview_document_extraction(
    State(_state): State<AppComponents>,
    mut multipart: Multipart,
) -> Result<Json<RagEnhancedSampleResult>, (StatusCode, String)> {
    let start_time = std::time::Instant::now();

    // Extract file from multipart form (similar to process_document_and_create_samples)
    let mut file_path = None;
    let mut confidence_threshold = 0.7; // Default threshold

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to read multipart field: {}", e),
        )
    })? {
        let field_name = field.name().unwrap_or("");

        match field_name {
            "file" => {
                let file_name = field
                    .file_name()
                    .ok_or((StatusCode::BAD_REQUEST, "No filename provided".to_string()))?
                    .to_string();

                let upload_dir = Path::new("uploads");
                fs::create_dir_all(upload_dir).await.map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to create upload directory: {}", e),
                    )
                })?;

                let file_uuid = Uuid::new_v4();
                let temp_file_path =
                    upload_dir.join(format!("preview_{}_{}", file_uuid, file_name));

                let file_data = field.bytes().await.map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        format!("Failed to read file data: {}", e),
                    )
                })?;

                fs::write(&temp_file_path, file_data).await.map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to save uploaded file: {}", e),
                    )
                })?;

                file_path = Some(temp_file_path.to_string_lossy().to_string());
            }
            "confidence_threshold" => {
                let threshold_str = field.text().await.map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        format!("Failed to read confidence threshold: {}", e),
                    )
                })?;

                confidence_threshold = threshold_str.parse().unwrap_or(0.7);
            }
            _ => {
                // Skip unknown fields
            }
        }
    }

    let document_path = file_path.ok_or((
        StatusCode::BAD_REQUEST,
        "No document file provided".to_string(),
    ))?;

    // Initialize RAG service and process document
    let rag_config = RagConfig {
        base_url: _state.config.rag.base_url.clone(),
        timeout_seconds: _state.config.rag.timeout_seconds,
        max_file_size_mb: _state.config.rag.max_file_size_mb,
        supported_formats: _state.config.rag.supported_formats.clone(),
    };
    let rag_service = RagIntegrationService::new(rag_config);

    let extraction_result = rag_service
        .process_document(&document_path)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("RAG processing failed: {}", e),
            )
        })?;

    // Clean up temporary file
    let _ = fs::remove_file(&document_path).await;

    // Convert to sample format for preview
    let samples = if extraction_result.success
        && extraction_result.confidence_score >= confidence_threshold
    {
        rag_service
            .convert_to_samples(&extraction_result)
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let processing_time = start_time.elapsed().as_secs_f64();

    // Add confidence threshold warning if needed
    let mut validation_warnings = extraction_result.warnings.clone();
    if extraction_result.confidence_score < confidence_threshold {
        validation_warnings.push(format!(
            "Extraction confidence ({:.2}) below threshold ({:.2}). Review required.",
            extraction_result.confidence_score, confidence_threshold
        ));
    }

    Ok(Json(RagEnhancedSampleResult {
        samples,
        extraction_result: Some(extraction_result.clone()),
        confidence_score: extraction_result.confidence_score,
        validation_warnings,
        processing_time,
    }))
}

#[derive(Debug, serde::Deserialize)]
pub struct QueryRequest {
    pub query: String,
    pub session_id: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct QueryResponse {
    pub query: String,
    pub answer: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
