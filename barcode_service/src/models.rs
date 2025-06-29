use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request to generate a new barcode
#[derive(Debug, Deserialize)]
pub struct GenerateBarcodeRequest {
    pub sample_type: Option<String>,
    pub location_id: Option<i32>,
    pub template_name: Option<String>,
    pub custom_prefix: Option<String>,
}

/// Response containing a generated barcode
#[derive(Debug, Serialize)]
pub struct GenerateBarcodeResponse {
    pub barcode: String,
    pub info: BarcodeInfo,
}

/// Request to validate a barcode
#[derive(Debug, Deserialize)]
pub struct ValidateBarcodeRequest {
    pub barcode: String,
}

/// Response for barcode validation
#[derive(Debug, Serialize)]
pub struct ValidateBarcodeResponse {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub info: Option<BarcodeInfo>,
}

/// Request to parse a barcode
#[derive(Debug, Deserialize)]
pub struct ParseBarcodeRequest {
    pub barcode: String,
}

/// Response containing parsed barcode information
#[derive(Debug, Serialize)]
pub struct ParseBarcodeResponse {
    pub info: BarcodeInfo,
}

/// Request to reserve a barcode
#[derive(Debug, Deserialize)]
pub struct ReserveBarcodeRequest {
    pub barcode: String,
    pub reserved_by: String,
    pub purpose: Option<String>,
}

/// Request to release a barcode
#[derive(Debug, Deserialize)]
pub struct ReleaseBarcodeRequest {
    pub barcode: String,
    pub released_by: String,
}

/// Request to check if a barcode is unique
#[derive(Debug, Deserialize)]
pub struct CheckBarcodeUniqueRequest {
    pub barcode: String,
}

/// Response for barcode uniqueness check
#[derive(Debug, Serialize)]
pub struct CheckBarcodeUniqueResponse {
    pub is_unique: bool,
    pub is_reserved: bool,
    pub reserved_by: Option<String>,
    pub reserved_at: Option<DateTime<Utc>>,
}

/// Information extracted from a barcode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarcodeInfo {
    pub full_barcode: String,
    pub prefix: Option<String>,
    pub sample_type: Option<String>,
    pub date_component: Option<String>,
    pub location_component: Option<i32>,
    pub sequence_component: Option<String>,
    pub is_valid: bool,
    pub generated_at: Option<DateTime<Utc>>,
}

/// Barcode generation statistics
#[derive(Debug, Serialize)]
pub struct BarcodeStats {
    pub total_generated: i64,
    pub total_reserved: i64,
    pub total_unique_prefixes: i64,
    pub most_recent_barcode: Option<String>,
    pub generation_rate_per_day: f64,
    pub config_summary: BarcodeConfigSummary,
}

/// Summary of barcode configuration
#[derive(Debug, Serialize)]
pub struct BarcodeConfigSummary {
    pub prefix: String,
    pub min_length: usize,
    pub include_date: bool,
    pub include_sequence: bool,
}

/// Database model for stored barcodes
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StoredBarcode {
    pub id: Uuid,
    pub barcode: String,
    pub prefix: Option<String>,
    pub sample_type: Option<String>,
    pub location_id: Option<i32>,
    pub is_reserved: bool,
    pub reserved_by: Option<String>,
    pub reserved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub database_connected: bool,
    pub total_barcodes: i64,
} 