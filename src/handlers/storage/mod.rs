use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::AppComponents;

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageLocation {
    pub id: i32,
    pub name: String,
    pub capacity: i32,
    pub available: i32,
    pub samples: Vec<StoredSample>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoredSample {
    pub id: i32,
    pub name: String,
    pub barcode: String,
    pub template_id: i32,
    pub template_name: String,
    pub stored_at: String,
}

#[derive(Debug, Deserialize)]
pub struct MoveSampleRequest {
    pub sample_id: i32,
    pub location_id: i32,
}

#[derive(Debug, Serialize)]
pub struct MoveSampleResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ScannedSampleInfo {
    pub id: i32,
    pub name: String,
    pub barcode: String,
    pub location: String,
    pub template_name: String,
    pub stored_at: String,
}

/// List all available storage locations
/// TODO: This is a placeholder implementation. In production, this should fetch from database.
pub async fn list_storage_locations(
    State(_state): State<AppComponents>,
) -> Result<Json<Vec<StorageLocation>>, (StatusCode, String)> {
    // Mock data for now - this should be replaced with database queries
    let locations = vec![
        StorageLocation {
            id: 1,
            name: "Freezer A (-80°C)".to_string(),
            capacity: 100,
            available: 85,
            samples: vec![
                StoredSample {
                    id: 1,
                    name: "DNA Sample 001".to_string(),
                    barcode: "DNA001-2024-01".to_string(),
                    template_id: 1,
                    template_name: "DNA Extraction Template".to_string(),
                    stored_at: "2024-01-15T10:30:00Z".to_string(),
                },
                StoredSample {
                    id: 2,
                    name: "DNA Sample 002".to_string(),
                    barcode: "DNA002-2024-01".to_string(),
                    template_id: 1,
                    template_name: "DNA Extraction Template".to_string(),
                    stored_at: "2024-01-15T11:00:00Z".to_string(),
                },
            ],
        },
        StorageLocation {
            id: 2,
            name: "Freezer B (-20°C)".to_string(),
            capacity: 80,
            available: 65,
            samples: vec![StoredSample {
                id: 3,
                name: "RNA Sample 001".to_string(),
                barcode: "RNA001-2024-01".to_string(),
                template_id: 2,
                template_name: "RNA Isolation Template".to_string(),
                stored_at: "2024-01-16T09:15:00Z".to_string(),
            }],
        },
        StorageLocation {
            id: 3,
            name: "Refrigerator (4°C)".to_string(),
            capacity: 50,
            available: 45,
            samples: vec![],
        },
        StorageLocation {
            id: 4,
            name: "Room Temperature Storage".to_string(),
            capacity: 200,
            available: 180,
            samples: vec![StoredSample {
                id: 4,
                name: "Buffer Solution".to_string(),
                barcode: "BUF001-2024-01".to_string(),
                template_id: 3,
                template_name: "Buffer Template".to_string(),
                stored_at: "2024-01-10T14:20:00Z".to_string(),
            }],
        },
        StorageLocation {
            id: 5,
            name: "Incubator (37°C)".to_string(),
            capacity: 30,
            available: 25,
            samples: vec![],
        },
    ];

    Ok(Json(locations))
}

/// Move a sample from one storage location to another
pub async fn move_sample(
    State(_state): State<AppComponents>,
    Json(request): Json<MoveSampleRequest>,
) -> Result<Json<MoveSampleResponse>, (StatusCode, String)> {
    // TODO: Implement actual database operations for moving samples
    // For now, return a success response to prevent 404 errors

    // Validate input
    if request.sample_id <= 0 || request.location_id <= 0 {
        return Ok(Json(MoveSampleResponse {
            success: false,
            message: "Invalid sample ID or location ID".to_string(),
        }));
    }

    // Mock successful move operation
    Ok(Json(MoveSampleResponse {
        success: true,
        message: format!(
            "Sample {} successfully moved to location {}",
            request.sample_id, request.location_id
        ),
    }))
}

/// Scan a sample barcode to get its information
pub async fn scan_sample_barcode(
    State(_state): State<AppComponents>,
    axum::extract::Path(barcode): axum::extract::Path<String>,
) -> Result<Json<ScannedSampleInfo>, (StatusCode, String)> {
    // TODO: Implement actual database lookup for barcode
    // For now, return mock data based on the barcode pattern

    if barcode.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Barcode cannot be empty".to_string(),
        ));
    }

    // Mock scanned sample info based on barcode pattern
    let sample_info = if barcode.starts_with("DNA") {
        ScannedSampleInfo {
            id: 1,
            name: "DNA Sample".to_string(),
            barcode: barcode.clone(),
            location: "Freezer A (-80°C)".to_string(),
            template_name: "DNA Extraction Template".to_string(),
            stored_at: "2024-01-15T10:30:00Z".to_string(),
        }
    } else if barcode.starts_with("RNA") {
        ScannedSampleInfo {
            id: 2,
            name: "RNA Sample".to_string(),
            barcode: barcode.clone(),
            location: "Freezer B (-20°C)".to_string(),
            template_name: "RNA Isolation Template".to_string(),
            stored_at: "2024-01-16T09:15:00Z".to_string(),
        }
    } else {
        return Err((StatusCode::NOT_FOUND, "Sample not found".to_string()));
    };

    Ok(Json(sample_info))
}
