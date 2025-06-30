use axum::{extract::State, Json};
use chrono::Utc;

use crate::{
    models::{
        CheckBarcodeUniqueRequest, CheckBarcodeUniqueResponse, GenerateBarcodeRequest,
        GenerateBarcodeResponse, HealthResponse, ParseBarcodeRequest, ParseBarcodeResponse,
        ReleaseBarcodeRequest, ReserveBarcodeRequest, ValidateBarcodeRequest,
        ValidateBarcodeResponse,
    },
    service::BarcodeService,
    error::{BarcodeError, Result},
};

/// Health check endpoint
pub async fn health_check(State(service): State<BarcodeService>) -> Result<Json<HealthResponse>> {
    let total_barcodes = service.health_check().await?;
    
    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        service: "barcode_service".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: Utc::now(),
        database_connected: true,
        total_barcodes,
    }))
}

/// Generate a new barcode
pub async fn generate_barcode(
    State(service): State<BarcodeService>,
    Json(request): Json<GenerateBarcodeRequest>,
) -> Result<Json<GenerateBarcodeResponse>> {
    let barcode = service
        .generate_barcode(
            request.sample_type.as_deref(),
            request.location_id,
            request.template_name.as_deref(),
            request.custom_prefix.as_deref(),
        )
        .await?;

    let info = service.parse_barcode(&barcode);

    Ok(Json(GenerateBarcodeResponse { barcode, info }))
}

/// Validate a barcode
pub async fn validate_barcode(
    State(service): State<BarcodeService>,
    Json(request): Json<ValidateBarcodeRequest>,
) -> Result<Json<ValidateBarcodeResponse>> {
    let mut errors = Vec::new();
    let mut is_valid = true;

    // Validate format
    if let Err(e) = service.validate_barcode_format(&request.barcode) {
        errors.push(e.to_string());
        is_valid = false;
    }

    // Parse barcode for additional info
    let info = if is_valid {
        Some(service.parse_barcode(&request.barcode))
    } else {
        None
    };

    Ok(Json(ValidateBarcodeResponse {
        is_valid,
        errors,
        info,
    }))
}

/// Parse a barcode to extract components
pub async fn parse_barcode(
    State(service): State<BarcodeService>,
    Json(request): Json<ParseBarcodeRequest>,
) -> Result<Json<ParseBarcodeResponse>> {
    let info = service.parse_barcode(&request.barcode);

    Ok(Json(ParseBarcodeResponse { info }))
}

/// Reserve a barcode
pub async fn reserve_barcode(
    State(service): State<BarcodeService>,
    Json(request): Json<ReserveBarcodeRequest>,
) -> Result<Json<serde_json::Value>> {
    service
        .reserve_barcode(
            &request.barcode,
            &request.reserved_by,
            request.purpose.as_deref(),
        )
        .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Barcode {} reserved successfully", request.barcode),
        "reserved_by": request.reserved_by,
        "reserved_at": Utc::now()
    })))
}

/// Release a barcode
pub async fn release_barcode(
    State(service): State<BarcodeService>,
    Json(request): Json<ReleaseBarcodeRequest>,
) -> Result<Json<serde_json::Value>> {
    service
        .release_barcode(&request.barcode, &request.released_by)
        .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Barcode {} released successfully", request.barcode),
        "released_by": request.released_by,
        "released_at": Utc::now()
    })))
}

/// Check if a barcode is unique
pub async fn check_barcode_unique(
    State(service): State<BarcodeService>,
    Json(request): Json<CheckBarcodeUniqueRequest>,
) -> Result<Json<CheckBarcodeUniqueResponse>> {
    let is_unique = service.check_barcode_unique(&request.barcode).await?;

    // Get reservation status if the barcode exists
    let (is_reserved, reserved_by, reserved_at) = if !is_unique {
        if let Some(stored_barcode) = service.get_barcode_status(&request.barcode).await? {
            (
                stored_barcode.is_reserved,
                stored_barcode.reserved_by,
                stored_barcode.reserved_at,
            )
        } else {
            (false, None, None)
        }
    } else {
        (false, None, None)
    };

    Ok(Json(CheckBarcodeUniqueResponse {
        is_unique,
        is_reserved,
        reserved_by,
        reserved_at,
    }))
}

/// Get barcode generation statistics
pub async fn get_stats(State(service): State<BarcodeService>) -> Result<Json<serde_json::Value>> {
    let stats = service.get_stats().await?;

    Ok(Json(serde_json::json!({
        "statistics": stats,
        "timestamp": Utc::now(),
        "service": "barcode_service",
        "version": env!("CARGO_PKG_VERSION")
    })))
} 