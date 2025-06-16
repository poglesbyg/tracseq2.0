use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::storage::{
    SampleLocation, StorageCapacityStats, StorageLocation, StorageRequirement, StorageState,
    StorageValidationError, TemperatureZone,
};
use crate::repositories::storage_repository::{
    CreateMovementHistory, CreateSampleLocation, CreateStorageLocation, StorageRepository,
    UpdateStorageLocation,
};
use crate::services::barcode_service::BarcodeService;

/// Storage management service for biological sample storage operations
pub struct StorageManagementService<R: StorageRepository> {
    storage_repo: Arc<R>,
    barcode_service: Arc<RwLock<BarcodeService>>,
}

impl<R: StorageRepository> StorageManagementService<R> {
    pub fn new(storage_repo: Arc<R>, barcode_service: Arc<RwLock<BarcodeService>>) -> Self {
        Self {
            storage_repo,
            barcode_service,
        }
    }

    /// Store a sample with automatic barcode generation and validation
    pub async fn store_sample(
        &self,
        sample_id: i32,
        location_id: i32,
        sample_type: &str,
        template_name: Option<&str>,
        stored_by: &str,
        position: Option<String>,
        requirements: Option<StorageRequirement>,
    ) -> Result<StoredSampleResult, StorageManagementError> {
        // Validate storage location exists and is available
        let location = self
            .storage_repo
            .get_storage_location(location_id)
            .await
            .map_err(StorageManagementError::DatabaseError)?
            .ok_or(StorageManagementError::LocationNotFound(location_id))?;

        // Validate storage requirements if provided
        if let Some(req) = &requirements {
            self.validate_storage_requirements(&location, req)?;
        }

        // Check capacity
        if !location.can_accommodate(1) {
            return Err(StorageManagementError::InsufficientCapacity {
                location_id,
                requested: 1,
                available: location.available_capacity(),
            });
        }

        // Generate unique barcode
        let mut barcode_service = self.barcode_service.write().await;
        let barcode = barcode_service
            .generate_sample_barcode(sample_type, location_id, template_name)
            .await
            .map_err(StorageManagementError::BarcodeGenerationError)?;

        // Store sample
        let sample_location = CreateSampleLocation {
            sample_id,
            location_id,
            barcode: barcode.clone(),
            position,
            storage_state: StorageState::InStorage,
            stored_by: Some(stored_by.to_string()),
            notes: requirements.as_ref().map(|req| {
                format!(
                    "Temperature: {}, Special conditions: {}",
                    req.temperature_zone.display_name(),
                    req.special_conditions.join(", ")
                )
            }),
        };

        let stored_sample = self
            .storage_repo
            .store_sample(sample_location)
            .await
            .map_err(StorageManagementError::DatabaseError)?;

        // Record movement history
        let movement = CreateMovementHistory {
            sample_id,
            barcode: barcode.clone(),
            from_location_id: None,
            to_location_id: location_id,
            from_state: Some(StorageState::Validated),
            to_state: StorageState::InStorage,
            movement_reason: "Initial storage".to_string(),
            moved_by: stored_by.to_string(),
            notes: Some("Sample initially stored in laboratory system".to_string()),
        };

        self.storage_repo
            .record_movement(movement)
            .await
            .map_err(StorageManagementError::DatabaseError)?;

        Ok(StoredSampleResult {
            sample_location: stored_sample,
            barcode,
            location,
            warnings: self.check_storage_warnings(location_id).await?,
        })
    }

    /// Move a sample from one location to another
    pub async fn move_sample(
        &self,
        sample_id: i32,
        new_location_id: i32,
        moved_by: &str,
        reason: &str,
        requirements: Option<StorageRequirement>,
    ) -> Result<SampleLocation, StorageManagementError> {
        // Validate new location
        let new_location = self
            .storage_repo
            .get_storage_location(new_location_id)
            .await
            .map_err(StorageManagementError::DatabaseError)?
            .ok_or(StorageManagementError::LocationNotFound(new_location_id))?;

        // Validate storage requirements if provided
        if let Some(req) = &requirements {
            self.validate_storage_requirements(&new_location, req)?;
        }

        // Check capacity
        if !new_location.can_accommodate(1) {
            return Err(StorageManagementError::InsufficientCapacity {
                location_id: new_location_id,
                requested: 1,
                available: new_location.available_capacity(),
            });
        }

        // Move sample
        let moved_sample = self
            .storage_repo
            .move_sample(sample_id, new_location_id, moved_by, reason)
            .await
            .map_err(StorageManagementError::DatabaseError)?;

        Ok(moved_sample)
    }

    /// Move a sample by barcode from one location to another
    pub async fn move_sample_by_barcode(
        &self,
        barcode: &str,
        new_location_id: i32,
        moved_by: &str,
        reason: &str,
        requirements: Option<StorageRequirement>,
    ) -> Result<SampleLocation, StorageManagementError> {
        // First get the sample by barcode
        let sample_location = self
            .storage_repo
            .get_sample_by_barcode(barcode)
            .await
            .map_err(StorageManagementError::DatabaseError)?
            .ok_or(StorageManagementError::BarcodeNotFound(barcode.to_string()))?;

        // Use the existing move_sample method
        self.move_sample(
            sample_location.sample_id,
            new_location_id,
            moved_by,
            reason,
            requirements,
        )
        .await
    }

    /// Remove a sample from storage
    pub async fn remove_sample(
        &self,
        barcode: &str,
        removed_by: &str,
        reason: &str,
    ) -> Result<RemovedSampleResult, StorageManagementError> {
        // First get the sample by barcode
        let sample_location = self
            .storage_repo
            .get_sample_by_barcode(barcode)
            .await
            .map_err(StorageManagementError::DatabaseError)?
            .ok_or(StorageManagementError::BarcodeNotFound(barcode.to_string()))?;

        // Get the location name for the response
        let location = self
            .storage_repo
            .get_storage_location(sample_location.location_id)
            .await
            .map_err(StorageManagementError::DatabaseError)?
            .ok_or(StorageManagementError::LocationNotFound(
                sample_location.location_id,
            ))?;

        // Remove the sample
        let removed_sample = self
            .storage_repo
            .remove_sample(sample_location.sample_id, removed_by, reason)
            .await
            .map_err(StorageManagementError::DatabaseError)?;

        Ok(RemovedSampleResult {
            sample_location: removed_sample,
            location,
        })
    }

    /// Update sample storage state
    pub async fn update_sample_state(
        &self,
        sample_id: i32,
        new_state: StorageState,
        updated_by: &str,
    ) -> Result<SampleLocation, StorageManagementError> {
        // Get current sample location to validate state transition
        let current_sample = self
            .storage_repo
            .get_sample_location(sample_id)
            .await
            .map_err(StorageManagementError::DatabaseError)?
            .ok_or(StorageManagementError::SampleNotFound(sample_id))?;

        // Validate state transition
        if !current_sample.storage_state.can_transition_to(new_state) {
            return Err(StorageManagementError::InvalidStateTransition {
                current_state: current_sample.storage_state,
                requested_state: new_state,
            });
        }

        // Update state
        let updated_sample = self
            .storage_repo
            .update_sample_state(sample_id, new_state, updated_by)
            .await
            .map_err(StorageManagementError::DatabaseError)?;

        Ok(updated_sample)
    }

    /// Scan barcode to get sample information
    pub async fn scan_barcode(
        &self,
        barcode: &str,
    ) -> Result<SampleScanResult, StorageManagementError> {
        let sample_location = self
            .storage_repo
            .get_sample_by_barcode(barcode)
            .await
            .map_err(StorageManagementError::DatabaseError)?
            .ok_or(StorageManagementError::BarcodeNotFound(barcode.to_string()))?;

        let location = self
            .storage_repo
            .get_storage_location(sample_location.location_id)
            .await
            .map_err(StorageManagementError::DatabaseError)?
            .ok_or(StorageManagementError::LocationNotFound(
                sample_location.location_id,
            ))?;

        let barcode_service = self.barcode_service.read().await;
        let barcode_info = barcode_service.parse_barcode(barcode);

        Ok(SampleScanResult {
            sample_location,
            location,
            barcode_info,
        })
    }

    /// Get storage capacity statistics
    pub async fn get_capacity_overview(&self) -> Result<CapacityOverview, StorageManagementError> {
        let stats = self
            .storage_repo
            .get_storage_capacity_stats()
            .await
            .map_err(StorageManagementError::DatabaseError)?;

        let total_capacity: i32 = stats.iter().map(|s| s.total_capacity).sum();
        let total_usage: i32 = stats.iter().map(|s| s.current_usage).sum();
        let locations_near_capacity = stats.iter().filter(|s| s.is_near_capacity).count();

        let warnings = self.generate_capacity_warnings(&stats);

        Ok(CapacityOverview {
            total_locations: stats.len() as i32,
            total_capacity,
            total_usage,
            total_available: total_capacity - total_usage,
            overall_utilization: if total_capacity > 0 {
                (total_usage as f64 / total_capacity as f64) * 100.0
            } else {
                0.0
            },
            locations_near_capacity: locations_near_capacity as i32,
            by_temperature: self.group_stats_by_temperature(&stats),
            warnings,
            location_stats: stats,
        })
    }

    /// Get storage locations by temperature zone
    pub async fn get_locations_by_temperature(
        &self,
        temperature_zone: TemperatureZone,
    ) -> Result<Vec<StorageLocation>, StorageManagementError> {
        self.storage_repo
            .get_storage_locations_by_temperature(temperature_zone)
            .await
            .map_err(StorageManagementError::DatabaseError)
    }

    /// Get samples in a specific location
    pub async fn get_samples_in_location(
        &self,
        location_id: i32,
    ) -> Result<Vec<SampleLocation>, StorageManagementError> {
        self.storage_repo
            .get_samples_in_location(location_id)
            .await
            .map_err(StorageManagementError::DatabaseError)
    }

    /// Create a new storage location
    pub async fn create_storage_location(
        &self,
        location_data: CreateStorageLocation,
    ) -> Result<StorageLocation, StorageManagementError> {
        self.storage_repo
            .create_storage_location(location_data)
            .await
            .map_err(StorageManagementError::DatabaseError)
    }

    /// Update storage location
    pub async fn update_storage_location(
        &self,
        location_id: i32,
        updates: UpdateStorageLocation,
    ) -> Result<StorageLocation, StorageManagementError> {
        self.storage_repo
            .update_storage_location(location_id, updates)
            .await
            .map_err(StorageManagementError::DatabaseError)
    }

    /// Private helper methods
    fn validate_storage_requirements(
        &self,
        location: &StorageLocation,
        requirements: &StorageRequirement,
    ) -> Result<(), StorageManagementError> {
        // Check temperature compatibility
        if location.temperature_zone != requirements.temperature_zone {
            return Err(StorageManagementError::IncompatibleTemperature {
                sample_temp_requirement: requirements.temperature_zone,
                location_temp: location.temperature_zone,
            });
        }

        // Check if location is active
        if !location.is_active {
            return Err(StorageManagementError::LocationInactive(location.id));
        }

        Ok(())
    }

    async fn check_storage_warnings(
        &self,
        location_id: i32,
    ) -> Result<Vec<String>, StorageManagementError> {
        let location_stats = self
            .storage_repo
            .get_location_capacity_stats(location_id)
            .await
            .map_err(StorageManagementError::DatabaseError)?;

        let mut warnings = Vec::new();

        if let Some(stats) = location_stats {
            if stats.is_near_capacity {
                warnings.push(format!(
                    "Location '{}' is at {:.1}% capacity",
                    stats.location_name, stats.utilization_percentage
                ));
            }

            if stats.utilization_percentage >= 95.0 {
                warnings.push(format!(
                    "Location '{}' is critically full (>95%)",
                    stats.location_name
                ));
            }
        }

        Ok(warnings)
    }

    fn generate_capacity_warnings(&self, stats: &[StorageCapacityStats]) -> Vec<String> {
        let mut warnings = Vec::new();

        let critical_locations: Vec<_> = stats
            .iter()
            .filter(|s| s.utilization_percentage >= 95.0)
            .collect();

        let near_capacity_locations: Vec<_> = stats
            .iter()
            .filter(|s| s.is_near_capacity && s.utilization_percentage < 95.0)
            .collect();

        if !critical_locations.is_empty() {
            warnings.push(format!(
                "{} location(s) are critically full (>95%): {}",
                critical_locations.len(),
                critical_locations
                    .iter()
                    .map(|s| s.location_name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        if !near_capacity_locations.is_empty() {
            warnings.push(format!(
                "{} location(s) are approaching capacity: {}",
                near_capacity_locations.len(),
                near_capacity_locations
                    .iter()
                    .map(|s| s.location_name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        warnings
    }

    fn group_stats_by_temperature(
        &self,
        stats: &[StorageCapacityStats],
    ) -> Vec<TemperatureGroupStats> {
        use std::collections::HashMap;

        let mut groups: HashMap<TemperatureZone, Vec<&StorageCapacityStats>> = HashMap::new();

        for stat in stats {
            groups.entry(stat.temperature_zone).or_default().push(stat);
        }

        groups
            .into_iter()
            .map(|(temp_zone, location_stats)| {
                let total_capacity: i32 = location_stats.iter().map(|s| s.total_capacity).sum();
                let total_usage: i32 = location_stats.iter().map(|s| s.current_usage).sum();

                TemperatureGroupStats {
                    temperature_zone: temp_zone,
                    location_count: location_stats.len() as i32,
                    total_capacity,
                    total_usage,
                    utilization_percentage: if total_capacity > 0 {
                        (total_usage as f64 / total_capacity as f64) * 100.0
                    } else {
                        0.0
                    },
                }
            })
            .collect()
    }
}

/// Result of storing a sample
#[derive(Debug, Clone)]
pub struct StoredSampleResult {
    pub sample_location: SampleLocation,
    pub barcode: String,
    pub location: StorageLocation,
    pub warnings: Vec<String>,
}

/// Result of scanning a barcode
#[derive(Debug, Clone)]
pub struct SampleScanResult {
    pub sample_location: SampleLocation,
    pub location: StorageLocation,
    pub barcode_info: crate::services::barcode_service::BarcodeInfo,
}

/// Result of removing a sample from storage
#[derive(Debug, Clone)]
pub struct RemovedSampleResult {
    pub sample_location: SampleLocation,
    pub location: StorageLocation,
}

/// Capacity overview statistics
#[derive(Debug, Clone, Serialize)]
pub struct CapacityOverview {
    pub total_locations: i32,
    pub total_capacity: i32,
    pub total_usage: i32,
    pub total_available: i32,
    pub overall_utilization: f64,
    pub locations_near_capacity: i32,
    pub by_temperature: Vec<TemperatureGroupStats>,
    pub warnings: Vec<String>,
    pub location_stats: Vec<StorageCapacityStats>,
}

/// Temperature group statistics
#[derive(Debug, Clone, Serialize)]
pub struct TemperatureGroupStats {
    pub temperature_zone: TemperatureZone,
    pub location_count: i32,
    pub total_capacity: i32,
    pub total_usage: i32,
    pub utilization_percentage: f64,
}

/// Storage management error types
#[derive(Debug, thiserror::Error)]
pub enum StorageManagementError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Barcode generation error: {0}")]
    BarcodeGenerationError(#[from] StorageValidationError),

    #[error("Location {0} not found")]
    LocationNotFound(i32),

    #[error("Sample {0} not found")]
    SampleNotFound(i32),

    #[error("Barcode {0} not found")]
    BarcodeNotFound(String),

    #[error("Insufficient capacity in location {location_id}: requested {requested}, available {available}")]
    InsufficientCapacity {
        location_id: i32,
        requested: i32,
        available: i32,
    },

    #[error("Incompatible temperature: sample requires {sample_temp_requirement:?}, location has {location_temp:?}")]
    IncompatibleTemperature {
        sample_temp_requirement: TemperatureZone,
        location_temp: TemperatureZone,
    },

    #[error("Location {0} is inactive")]
    LocationInactive(i32),

    #[error("Invalid state transition from {current_state:?} to {requested_state:?}")]
    InvalidStateTransition {
        current_state: StorageState,
        requested_state: StorageState,
    },
}
