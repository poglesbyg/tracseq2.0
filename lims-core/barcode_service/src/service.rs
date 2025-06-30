use chrono::{DateTime, Datelike, Utc};
use rand::Rng;
use regex::Regex;
use std::collections::HashMap;

use crate::{
    config::BarcodeConfig,
    database::DatabasePool,
    error::{BarcodeError, Result},
    models::{BarcodeInfo, BarcodeStats, BarcodeConfigSummary, StoredBarcode},
};

/// Core barcode generation service
#[derive(Clone)]
pub struct BarcodeService {
    db_pool: DatabasePool,
    config: BarcodeConfig,
    validation_regex: Regex,
}

impl BarcodeService {
    /// Create a new barcode service instance
    pub async fn new(db_pool: DatabasePool, config: BarcodeConfig) -> Result<Self> {
        let validation_regex = Regex::new(&config.validation_pattern)
            .map_err(|e| BarcodeError::ConfigurationError(format!("Invalid regex pattern: {}", e)))?;

        Ok(Self {
            db_pool,
            config,
            validation_regex,
        })
    }

    /// Generate a unique barcode
    pub async fn generate_barcode(
        &self,
        sample_type: Option<&str>,
        location_id: Option<i32>,
        template_name: Option<&str>,
        custom_prefix: Option<&str>,
    ) -> Result<String> {
        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 100;

        while attempts < MAX_ATTEMPTS {
            let barcode = self.create_barcode_candidate(
                sample_type,
                location_id,
                template_name,
                custom_prefix,
            );

            if self.validate_barcode_format(&barcode).is_ok() {
                if self.check_barcode_unique(&barcode).await? {
                    // Store the generated barcode
                    self.store_barcode(&barcode, sample_type, location_id).await?;
                    return Ok(barcode);
                }
            }

            attempts += 1;
        }

        Err(BarcodeError::GenerationFailed(format!(
            "Failed to generate unique barcode after {} attempts",
            MAX_ATTEMPTS
        )))
    }

    /// Create a barcode candidate following the configured pattern
    fn create_barcode_candidate(
        &self,
        sample_type: Option<&str>,
        location_id: Option<i32>,
        template_name: Option<&str>,
        custom_prefix: Option<&str>,
    ) -> String {
        let mut parts = Vec::new();

        // Add prefix (custom or default)
        let prefix = custom_prefix.unwrap_or(&self.config.prefix);
        parts.push(prefix.to_string());

        // Add sample type if provided
        if let Some(sample_type) = sample_type {
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
            parts.push(enhanced_sample_type.to_uppercase());
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
        let random_component: u16 = rand::thread_rng().gen_range(100..=999);
        format!("{:04}{:03}", timestamp_suffix, random_component)
    }

    /// Generate a random alphanumeric string
    fn generate_random_string(&self, length: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();

        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Validate a barcode format against laboratory conventions
    pub fn validate_barcode_format(&self, barcode: &str) -> Result<()> {
        // Check minimum length requirement
        if barcode.len() < self.config.min_length {
            return Err(BarcodeError::ValidationError(format!(
                "Barcode must be at least {} characters long",
                self.config.min_length
            )));
        }

        // Check for empty barcode
        if barcode.is_empty() {
            return Err(BarcodeError::ValidationError(
                "Barcode cannot be empty".to_string(),
            ));
        }

        // Check against validation regex
        if !self.validation_regex.is_match(barcode) {
            return Err(BarcodeError::ValidationError(
                "Barcode contains invalid characters".to_string(),
            ));
        }

        // Check for reasonable maximum length (prevent excessively long barcodes)
        if barcode.len() > 50 {
            return Err(BarcodeError::ValidationError(
                "Barcode exceeds maximum length of 50 characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if a barcode is unique in the database
    pub async fn check_barcode_unique(&self, barcode: &str) -> Result<bool> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM barcodes WHERE barcode = $1",
            barcode
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(BarcodeError::DatabaseError)?;

        Ok(count.unwrap_or(0) == 0)
    }

    /// Store a generated barcode in the database
    async fn store_barcode(
        &self,
        barcode: &str,
        sample_type: Option<&str>,
        location_id: Option<i32>,
    ) -> Result<()> {
        let info = self.parse_barcode(barcode);

        sqlx::query!(
            r#"
            INSERT INTO barcodes (id, barcode, prefix, sample_type, location_id, is_reserved, created_at, metadata)
            VALUES ($1, $2, $3, $4, $5, false, $6, $7)
            "#,
            uuid::Uuid::new_v4(),
            barcode,
            info.prefix,
            sample_type,
            location_id,
            Utc::now(),
            serde_json::json!({
                "generated_by": "barcode_service",
                "validation_passed": true
            })
        )
        .execute(&self.db_pool)
        .await
        .map_err(BarcodeError::DatabaseError)?;

        Ok(())
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
            is_valid: self.validate_barcode_format(barcode).is_ok(),
            generated_at: Some(Utc::now()),
        }
    }

    /// Extract date component from barcode parts
    fn extract_date_component(&self, parts: &[&str]) -> Option<String> {
        parts
            .iter()
            .find(|part| part.len() == 8 && part.chars().all(|c| c.is_numeric()))
            .map(|s| s.to_string())
    }

    /// Extract location component from barcode parts
    fn extract_location_component(&self, parts: &[&str]) -> Option<i32> {
        parts
            .iter()
            .find(|part| part.starts_with('L') && part[1..].chars().all(|c| c.is_numeric()))
            .and_then(|s| s[1..].parse().ok())
    }

    /// Reserve a barcode
    pub async fn reserve_barcode(&self, barcode: &str, reserved_by: &str, purpose: Option<&str>) -> Result<()> {
        // Check if barcode exists
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM barcodes WHERE barcode = $1)",
            barcode
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(BarcodeError::DatabaseError)?;

        if !exists.unwrap_or(false) {
            return Err(BarcodeError::BarcodeNotFound(barcode.to_string()));
        }

        // Reserve the barcode
        let rows_affected = sqlx::query!(
            r#"
            UPDATE barcodes 
            SET is_reserved = true, reserved_by = $1, reserved_at = $2, metadata = metadata || $3
            WHERE barcode = $4 AND NOT is_reserved
            "#,
            reserved_by,
            Utc::now(),
            serde_json::json!({
                "reservation_purpose": purpose
            }),
            barcode
        )
        .execute(&self.db_pool)
        .await
        .map_err(BarcodeError::DatabaseError)?
        .rows_affected();

        if rows_affected == 0 {
            return Err(BarcodeError::BarcodeAlreadyReserved(barcode.to_string()));
        }

        Ok(())
    }

    /// Release a barcode
    pub async fn release_barcode(&self, barcode: &str, released_by: &str) -> Result<()> {
        let rows_affected = sqlx::query!(
            r#"
            UPDATE barcodes 
            SET is_reserved = false, reserved_by = NULL, reserved_at = NULL, metadata = metadata || $1
            WHERE barcode = $2 AND is_reserved
            "#,
            serde_json::json!({
                "released_by": released_by,
                "released_at": Utc::now()
            }),
            barcode
        )
        .execute(&self.db_pool)
        .await
        .map_err(BarcodeError::DatabaseError)?
        .rows_affected();

        if rows_affected == 0 {
            return Err(BarcodeError::BarcodeNotReserved(barcode.to_string()));
        }

        Ok(())
    }

    /// Get barcode reservation status
    pub async fn get_barcode_status(&self, barcode: &str) -> Result<Option<StoredBarcode>> {
        let stored_barcode = sqlx::query_as!(
            StoredBarcode,
            "SELECT * FROM barcodes WHERE barcode = $1",
            barcode
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(BarcodeError::DatabaseError)?;

        Ok(stored_barcode)
    }

    /// Get barcode generation statistics
    pub async fn get_stats(&self) -> Result<BarcodeStats> {
        let total_generated = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM barcodes"
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(BarcodeError::DatabaseError)?
        .unwrap_or(0);

        let total_reserved = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM barcodes WHERE is_reserved = true"
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(BarcodeError::DatabaseError)?
        .unwrap_or(0);

        let total_unique_prefixes = sqlx::query_scalar!(
            "SELECT COUNT(DISTINCT prefix) FROM barcodes WHERE prefix IS NOT NULL"
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(BarcodeError::DatabaseError)?
        .unwrap_or(0);

        let most_recent_barcode = sqlx::query_scalar!(
            "SELECT barcode FROM barcodes ORDER BY created_at DESC LIMIT 1"
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(BarcodeError::DatabaseError)?;

        // Calculate generation rate per day (last 30 days)
        let thirty_days_ago = Utc::now() - chrono::Duration::days(30);
        let recent_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM barcodes WHERE created_at >= $1",
            thirty_days_ago
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(BarcodeError::DatabaseError)?
        .unwrap_or(0);

        let generation_rate_per_day = recent_count as f64 / 30.0;

        Ok(BarcodeStats {
            total_generated,
            total_reserved,
            total_unique_prefixes,
            most_recent_barcode,
            generation_rate_per_day,
            config_summary: BarcodeConfigSummary {
                prefix: self.config.prefix.clone(),
                min_length: self.config.min_length,
                include_date: self.config.include_date,
                include_sequence: self.config.include_sequence,
            },
        })
    }

    /// Check database connectivity
    pub async fn health_check(&self) -> Result<i64> {
        let count = sqlx::query_scalar!("SELECT COUNT(*) FROM barcodes")
            .fetch_one(&self.db_pool)
            .await
            .map_err(BarcodeError::DatabaseError)?;

        Ok(count.unwrap_or(0))
    }
} 