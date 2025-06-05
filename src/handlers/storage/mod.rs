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
