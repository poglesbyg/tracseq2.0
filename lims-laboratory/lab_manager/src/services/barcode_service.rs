use chrono::{DateTime, Datelike, Utc};
use rand::Rng;
use std::collections::HashSet;

use crate::models::storage::{BarcodeConfig, StorageValidationError};

/// Barcode generation service for laboratory sample tracking
pub struct BarcodeService {
    config: BarcodeConfig,
    used_barcodes: HashSet<String>, // In production, this would be handled by database
}

impl BarcodeService {
    pub fn new(config: BarcodeConfig) -> Self {
        Self {
            config,
            used_barcodes: HashSet::new(),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(BarcodeConfig::default())
    }

    /// Generate a unique barcode following laboratory conventions
    pub async fn generate_barcode(
        &mut self,
        sample_type: Option<&str>,
        location_id: Option<i32>,
    ) -> Result<String, StorageValidationError> {
        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 100;

        while attempts < MAX_ATTEMPTS {
            let barcode = self.create_barcode_candidate(sample_type, location_id);

            if self.validate_barcode(&barcode)? && !self.used_barcodes.contains(&barcode) {
                self.used_barcodes.insert(barcode.clone());
                return Ok(barcode);
            }

            attempts += 1;
        }

        Err(StorageValidationError::InvalidBarcode {
            barcode: "GENERATION_FAILED".to_string(),
            reason: format!(
                "Failed to generate unique barcode after {} attempts",
                MAX_ATTEMPTS
            ),
        })
    }

    /// Create a barcode candidate following the configured pattern
    fn create_barcode_candidate(
        &self,
        sample_type: Option<&str>,
        location_id: Option<i32>,
    ) -> String {
        let mut parts = Vec::new();

        // Add prefix
        parts.push(self.config.prefix.clone());

        // Add sample type if provided
        if let Some(sample_type) = sample_type {
            parts.push(sample_type.to_uppercase());
        }

        // Add date component if configured
        if self.config.include_date {
            let now = Utc::now();
            parts.push(format!(
                "{:04}{:02}{:02}",
                now.year(),
                now.month(),
                now.day()
            ));
        }

        // Add location component if provided
        if let Some(location_id) = location_id {
            parts.push(format!("L{:03}", location_id));
        }

        // Add sequence/random component if configured
        if self.config.include_sequence {
            let sequence = self.generate_sequence_component();
            parts.push(sequence);
        }

        let barcode = parts.join(&self.config.separator);

        // Ensure minimum length requirement
        if barcode.len() < self.config.min_length {
            let padding_needed = self.config.min_length - barcode.len();
            let random_suffix = self.generate_random_string(padding_needed);
            format!("{}{}{}", barcode, self.config.separator, random_suffix)
        } else {
            barcode
        }
    }

    /// Generate a sequence component (combination of timestamp and random)
    fn generate_sequence_component(&self) -> String {
        let timestamp_suffix = Utc::now().timestamp() % 10000; // Last 4 digits of timestamp
        let random_component: u16 = rand::rng().random_range(100..=999);
        format!("{:04}{:03}", timestamp_suffix, random_component)
    }

    /// Generate a random alphanumeric string
    fn generate_random_string(&self, length: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::rng();

        (0..length)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Validate a barcode against laboratory conventions
    pub fn validate_barcode(&self, barcode: &str) -> Result<bool, StorageValidationError> {
        // Check minimum length requirement (from barcode inventory system rule)
        if barcode.len() < self.config.min_length {
            return Err(StorageValidationError::InvalidBarcode {
                barcode: barcode.to_string(),
                reason: format!(
                    "Barcode must be at least {} characters long",
                    self.config.min_length
                ),
            });
        }

        // Check for valid characters (alphanumeric and separators only)
        if !barcode
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(StorageValidationError::InvalidBarcode {
                barcode: barcode.to_string(),
                reason:
                    "Barcode contains invalid characters. Only alphanumeric and separators allowed"
                        .to_string(),
            });
        }

        // Check for empty barcode
        if barcode.is_empty() {
            return Err(StorageValidationError::InvalidBarcode {
                barcode: barcode.to_string(),
                reason: "Barcode cannot be empty".to_string(),
            });
        }

        // Check for reasonable maximum length (prevent excessively long barcodes)
        if barcode.len() > 50 {
            return Err(StorageValidationError::InvalidBarcode {
                barcode: barcode.to_string(),
                reason: "Barcode exceeds maximum length of 50 characters".to_string(),
            });
        }

        Ok(true)
    }

    /// Check if a barcode already exists (in production, this would query the database)
    pub async fn is_barcode_unique(&self, barcode: &str) -> bool {
        !self.used_barcodes.contains(barcode)
    }

    /// Reserve a barcode (mark as used)
    pub fn reserve_barcode(&mut self, barcode: String) {
        self.used_barcodes.insert(barcode);
    }

    /// Release a barcode (mark as available again)
    pub fn release_barcode(&mut self, barcode: &str) {
        self.used_barcodes.remove(barcode);
    }

    /// Generate a barcode for a specific sample type with location context
    pub async fn generate_sample_barcode(
        &mut self,
        sample_type: &str,
        location_id: i32,
        template_name: Option<&str>,
    ) -> Result<String, StorageValidationError> {
        // Create a more specific sample type based on template if provided
        let enhanced_sample_type = if let Some(template) = template_name {
            // Extract key parts from template name for barcode
            let template_short = template
                .split_whitespace()
                .take(2)
                .map(|word| word.chars().take(3).collect::<String>())
                .collect::<Vec<_>>()
                .join("");
            format!("{}-{}", sample_type, template_short)
        } else {
            sample_type.to_string()
        };

        self.generate_barcode(Some(&enhanced_sample_type), Some(location_id))
            .await
    }

    /// Parse barcode to extract components
    pub fn parse_barcode(&self, barcode: &str) -> BarcodeInfo {
        let parts: Vec<&str> = barcode.split(&self.config.separator).collect();

        BarcodeInfo {
            full_barcode: barcode.to_string(),
            prefix: parts.first().map(|s| s.to_string()),
            sample_type: if parts.len() > 1 {
                parts.get(1).map(|s| s.to_string())
            } else {
                None
            },
            date_component: self.extract_date_component(&parts),
            location_component: self.extract_location_component(&parts),
            sequence_component: parts.last().map(|s| s.to_string()),
            is_valid: self.validate_barcode(barcode).unwrap_or(false),
        }
    }

    fn extract_date_component(&self, parts: &[&str]) -> Option<String> {
        parts
            .iter()
            .find(|part| part.len() == 8 && part.chars().all(|c| c.is_numeric()))
            .map(|s| s.to_string())
    }

    fn extract_location_component(&self, parts: &[&str]) -> Option<i32> {
        parts
            .iter()
            .find(|part| part.starts_with('L') && part[1..].chars().all(|c| c.is_numeric()))
            .and_then(|s| s[1..].parse().ok())
    }

    /// Get barcode generation statistics
    pub fn get_stats(&self) -> BarcodeStats {
        BarcodeStats {
            total_generated: self.used_barcodes.len(),
            config: self.config.clone(),
            last_generated: Utc::now(), // In production, track this properly
        }
    }
}

/// Information extracted from a barcode
#[derive(Debug, Clone)]
pub struct BarcodeInfo {
    pub full_barcode: String,
    pub prefix: Option<String>,
    pub sample_type: Option<String>,
    pub date_component: Option<String>,
    pub location_component: Option<i32>,
    pub sequence_component: Option<String>,
    pub is_valid: bool,
}

/// Barcode generation statistics
#[derive(Debug, Clone)]
pub struct BarcodeStats {
    pub total_generated: usize,
    pub config: BarcodeConfig,
    pub last_generated: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_barcode_generation() {
        let mut service = BarcodeService::with_default_config();

        let barcode = service
            .generate_barcode(Some("DNA"), Some(1))
            .await
            .unwrap();

        assert!(barcode.len() >= service.config.min_length);
        assert!(barcode.contains("LAB"));
        assert!(barcode.contains("DNA"));
    }

    #[tokio::test]
    async fn test_barcode_validation() {
        let service = BarcodeService::with_default_config();

        // Valid barcode
        assert!(service.validate_barcode("LAB-DNA-20240115-001").is_ok());

        // Invalid barcode (too short)
        assert!(service.validate_barcode("AB").is_err());

        // Invalid barcode (invalid characters)
        assert!(service.validate_barcode("LAB@DNA#001").is_err());
    }

    #[tokio::test]
    async fn test_barcode_uniqueness() {
        let mut service = BarcodeService::with_default_config();

        let barcode1 = service
            .generate_barcode(Some("DNA"), Some(1))
            .await
            .unwrap();
        let barcode2 = service
            .generate_barcode(Some("DNA"), Some(1))
            .await
            .unwrap();

        assert_ne!(barcode1, barcode2, "Generated barcodes should be unique");
    }
}
