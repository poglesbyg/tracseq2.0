use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use thiserror;

/// Temperature zones for biological sample storage
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Hash)]
#[sqlx(type_name = "temperature_zone", rename_all = "lowercase")]
pub enum TemperatureZone {
    #[serde(rename = "-80C")]
    #[sqlx(rename = "-80C")]
    UltraLowFreezer, // -80°C
    #[serde(rename = "-20C")]
    #[sqlx(rename = "-20C")]
    Freezer, // -20°C
    #[serde(rename = "4C")]
    #[sqlx(rename = "4C")]
    Refrigerator, // 4°C
    #[serde(rename = "RT")]
    #[sqlx(rename = "RT")]
    RoomTemperature, // RT
    #[serde(rename = "37C")]
    #[sqlx(rename = "37C")]
    Incubator, // 37°C
}

impl TemperatureZone {
    pub fn temperature_celsius(&self) -> i32 {
        match self {
            TemperatureZone::UltraLowFreezer => -80,
            TemperatureZone::Freezer => -20,
            TemperatureZone::Refrigerator => 4,
            TemperatureZone::RoomTemperature => 25, // Assume 25°C for RT
            TemperatureZone::Incubator => 37,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            TemperatureZone::UltraLowFreezer => "Ultra Low Freezer (-80°C)",
            TemperatureZone::Freezer => "Freezer (-20°C)",
            TemperatureZone::Refrigerator => "Refrigerator (4°C)",
            TemperatureZone::RoomTemperature => "Room Temperature",
            TemperatureZone::Incubator => "Incubator (37°C)",
        }
    }
}

/// Storage container types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "container_type", rename_all = "lowercase")]
pub enum ContainerType {
    Tube,
    Plate,
    Box,
    Rack,
    Bag,
}

/// Sample storage state in the storage workflow
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "storage_state", rename_all = "lowercase")]
pub enum StorageState {
    Pending,      // Sample submitted but not yet validated
    Validated,    // Sample validated but not yet stored
    InStorage,    // Sample physically stored in location
    InSequencing, // Sample moved to sequencing
    Completed,    // Sample processing completed
    Discarded,    // Sample discarded or destroyed
}

impl StorageState {
    pub fn can_transition_to(&self, new_state: StorageState) -> bool {
        use StorageState::*;
        match (self, new_state) {
            (Pending, Validated) => true,
            (Validated, InStorage) => true,
            (InStorage, InSequencing) => true,
            (InSequencing, Completed) => true,
            (_, Discarded) => true, // Can discard from any state
            _ => false,
        }
    }
}

/// Storage location representation
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StorageLocation {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub temperature_zone: TemperatureZone,
    pub capacity: i32,
    pub current_usage: i32,
    pub container_type: ContainerType,
    pub is_active: bool,
    pub location_path: Option<String>, // e.g., "Building A/Room 101/Freezer 1/Shelf 2"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl StorageLocation {
    pub fn available_capacity(&self) -> i32 {
        self.capacity - self.current_usage
    }

    pub fn utilization_percentage(&self) -> f64 {
        if self.capacity > 0 {
            (self.current_usage as f64 / self.capacity as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn is_near_capacity(&self, threshold_percentage: f64) -> bool {
        self.utilization_percentage() >= threshold_percentage
    }

    pub fn can_accommodate(&self, sample_count: i32) -> bool {
        self.available_capacity() >= sample_count && self.is_active
    }
}

/// Sample location tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SampleLocation {
    pub id: i32,
    pub sample_id: i32,
    pub location_id: i32,
    pub barcode: String,
    pub position: Option<String>, // e.g., "A1", "Tube 15", etc.
    pub storage_state: StorageState,
    pub stored_at: DateTime<Utc>,
    pub stored_by: Option<String>, // User who stored the sample
    pub moved_at: Option<DateTime<Utc>>,
    pub moved_by: Option<String>,
    pub notes: Option<String>,
    pub temperature_log: Option<String>, // JSON log of temperature readings
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Storage movement history for audit trail
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StorageMovementHistory {
    pub id: i32,
    pub sample_id: i32,
    pub barcode: String,
    pub from_location_id: Option<i32>,
    pub to_location_id: i32,
    pub from_state: Option<StorageState>,
    pub to_state: StorageState,
    pub movement_reason: String,
    pub moved_by: String,
    pub moved_at: DateTime<Utc>,
    pub notes: Option<String>,
}

/// Storage capacity statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageCapacityStats {
    pub location_id: i32,
    pub location_name: String,
    pub temperature_zone: TemperatureZone,
    pub total_capacity: i32,
    pub current_usage: i32,
    pub available_capacity: i32,
    pub utilization_percentage: f64,
    pub sample_count: i32,
    pub is_near_capacity: bool,
}

/// Barcode generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarcodeConfig {
    pub prefix: String,
    pub min_length: usize,
    pub include_date: bool,
    pub include_sequence: bool,
    pub separator: String,
}

impl Default for BarcodeConfig {
    fn default() -> Self {
        Self {
            prefix: "LAB".to_string(),
            min_length: 6,
            include_date: true,
            include_sequence: true,
            separator: "-".to_string(),
        }
    }
}

/// Storage validation error types
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum StorageValidationError {
    #[error("Insufficient capacity in location {location_id}: requested {requested}, available {available}")]
    InsufficientCapacity {
        location_id: i32,
        requested: i32,
        available: i32,
    },
    #[error("Temperature incompatible: sample requires {sample_temp_requirement:?}, location has {location_temp:?}")]
    IncompatibleTemperature {
        sample_temp_requirement: TemperatureZone,
        location_temp: TemperatureZone,
    },
    #[error("Location {location_id} is inactive")]
    LocationInactive { location_id: i32 },
    #[error("Invalid state transition from {current_state:?} to {requested_state:?}")]
    InvalidStateTransition {
        current_state: StorageState,
        requested_state: StorageState,
    },
    #[error("Duplicate barcode: {barcode}")]
    DuplicateBarcode { barcode: String },
    #[error("Invalid barcode '{barcode}': {reason}")]
    InvalidBarcode { barcode: String, reason: String },
}

/// Sample storage requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRequirement {
    pub temperature_zone: TemperatureZone,
    pub container_type: Option<ContainerType>,
    pub special_conditions: Vec<String>,
    pub max_storage_duration_days: Option<i32>,
}
