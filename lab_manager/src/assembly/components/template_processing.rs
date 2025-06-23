use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;

use super::super::traits::{
    Component, ComponentError, Configurable, ServiceConsumer, ServiceProvider, ServiceRegistry,
};

/// Configuration for template processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateProcessingConfig {
    /// Supported template formats
    pub supported_formats: Vec<String>,
    /// Maximum template file size in bytes
    pub max_file_size: usize,
    /// Enable template validation
    pub enable_validation: bool,
    /// Batch processing size
    pub batch_size: usize,
    /// Template cache size
    pub cache_size: usize,
    /// Enable format auto-detection
    pub auto_detect_format: bool,
}

impl Default for TemplateProcessingConfig {
    fn default() -> Self {
        Self {
            supported_formats: vec!["csv".to_string(), "xlsx".to_string(), "json".to_string()],
            max_file_size: 25 * 1024 * 1024, // 25MB
            enable_validation: true,
            batch_size: 500,
            cache_size: 100,
            auto_detect_format: true,
        }
    }
}

/// Template processing stages
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateStage {
    Upload,
    FormatDetection,
    FormatValidation,
    DataExtraction,
    SchemaValidation,
    DataMapping,
    ValidationComplete,
    ProcessingError,
}

/// Template format types
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateFormat {
    CSV { delimiter: char, has_header: bool },
    Excel { sheet_name: Option<String> },
    JSON { schema_version: String },
    XML { root_element: String },
    Unknown,
}

/// Template processing result
#[derive(Debug, Clone)]
pub struct TemplateResult {
    pub stage: TemplateStage,
    pub format: TemplateFormat,
    pub extracted_data: Vec<HashMap<String, String>>,
    pub metadata: HashMap<String, String>,
    pub validation_errors: Vec<String>,
    pub warnings: Vec<String>,
    pub processing_stats: TemplateStats,
}

/// Processing statistics for templates
#[derive(Debug, Clone, Default)]
pub struct TemplateStats {
    pub rows_processed: usize,
    pub columns_detected: usize,
    pub data_completeness: f64, // Percentage of non-empty cells
    pub processing_duration: std::time::Duration,
    pub memory_usage_kb: usize,
}

/// Template processing component implementing laboratory data pipelines
pub struct TemplateProcessingComponent {
    config: TemplateProcessingConfig,
    template_cache: HashMap<String, CachedTemplate>,
    processing_stats: ComponentStats,
    database_service: Option<std::sync::Arc<dyn Any + Send + Sync>>,
    storage_service: Option<std::sync::Arc<dyn Any + Send + Sync>>,
    is_initialized: bool,
}

#[derive(Debug, Clone)]
struct CachedTemplate {
    template_id: String,
    format: TemplateFormat,
    schema: HashMap<String, String>,
    last_used: chrono::DateTime<chrono::Utc>,
    usage_count: u32,
}

#[derive(Debug, Default)]
struct ComponentStats {
    templates_processed: u64,
    successful_extractions: u64,
    format_detection_accuracy: f64,
    average_processing_time: std::time::Duration,
    cache_hit_rate: f64,
}

impl TemplateProcessingComponent {
    pub fn new(config: TemplateProcessingConfig) -> Self {
        Self {
            config,
            template_cache: HashMap::new(),
            processing_stats: ComponentStats::default(),
            database_service: None,
            storage_service: None,
            is_initialized: false,
        }
    }

    /// Process a template file through the complete pipeline
    pub async fn process_template(
        &mut self,
        data: &[u8],
        filename: &str,
    ) -> Result<TemplateResult, ComponentError> {
        if !self.is_initialized {
            return Err(ComponentError::InitializationFailed(
                "Component not initialized".to_string(),
            ));
        }

        let processing_start = std::time::Instant::now();

        let mut result = TemplateResult {
            stage: TemplateStage::Upload,
            format: TemplateFormat::Unknown,
            extracted_data: Vec::new(),
            metadata: HashMap::new(),
            validation_errors: Vec::new(),
            warnings: Vec::new(),
            processing_stats: TemplateStats::default(),
        };

        // Stage 1: Format detection
        result.stage = TemplateStage::FormatDetection;
        result.format = self.detect_format(data, filename)?;

        if result.format == TemplateFormat::Unknown {
            result.stage = TemplateStage::ProcessingError;
            result
                .validation_errors
                .push("Unable to detect template format".to_string());
            return Ok(result);
        }

        // Stage 2: Format validation
        result.stage = TemplateStage::FormatValidation;
        if self.config.enable_validation {
            if let Err(validation_error) = self.validate_format(&result.format, data) {
                result.validation_errors.push(validation_error);
                result.stage = TemplateStage::ProcessingError;
                return Ok(result);
            }
        }

        // Stage 3: Data extraction
        result.stage = TemplateStage::DataExtraction;
        match self.extract_data(&result.format, data).await {
            Ok(extracted_data) => {
                result.extracted_data = extracted_data;
            }
            Err(e) => {
                result
                    .validation_errors
                    .push(format!("Data extraction failed: {}", e));
                result.stage = TemplateStage::ProcessingError;
                return Ok(result);
            }
        }

        // Stage 4: Schema validation
        result.stage = TemplateStage::SchemaValidation;
        let schema_validation = self.validate_schema(&result.extracted_data);
        if !schema_validation.is_empty() {
            result.warnings.extend(schema_validation);
        }

        // Stage 5: Data mapping
        result.stage = TemplateStage::DataMapping;
        self.apply_data_mapping(&mut result.extracted_data);

        // Calculate processing statistics
        let processing_duration = processing_start.elapsed();
        result.processing_stats = TemplateStats {
            rows_processed: result.extracted_data.len(),
            columns_detected: result
                .extracted_data
                .first()
                .map(|row| row.len())
                .unwrap_or(0),
            data_completeness: self.calculate_data_completeness(&result.extracted_data),
            processing_duration,
            memory_usage_kb: self.estimate_memory_usage(&result.extracted_data),
        };

        // Update component statistics
        self.update_component_stats(&result, processing_duration)
            .await;

        // Add metadata
        result.metadata.insert(
            "template_format".to_string(),
            format!("{:?}", result.format),
        );
        result.metadata.insert(
            "processing_duration_ms".to_string(),
            processing_duration.as_millis().to_string(),
        );
        result.metadata.insert(
            "rows_extracted".to_string(),
            result.extracted_data.len().to_string(),
        );

        result.stage = TemplateStage::ValidationComplete;
        Ok(result)
    }

    /// Process multiple templates in batch
    pub async fn process_batch(
        &mut self,
        templates: Vec<(Vec<u8>, String)>,
    ) -> Result<Vec<TemplateResult>, ComponentError> {
        let mut results = Vec::new();
        let batch_size = std::cmp::min(self.config.batch_size, templates.len());

        for chunk in templates.chunks(batch_size) {
            let mut batch_results = Vec::new();

            for (data, filename) in chunk {
                let result = self.process_template(data, filename).await?;
                batch_results.push(result);
            }

            results.extend(batch_results);

            // Small delay between batches to prevent overwhelming the system
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        Ok(results)
    }

    /// Get processing statistics for the component
    pub fn get_stats(&self) -> &ComponentStats {
        &self.processing_stats
    }

    /// Clear template cache
    pub fn clear_cache(&mut self) {
        self.template_cache.clear();
        tracing::info!("Template cache cleared");
    }

    /// Get supported formats
    pub fn get_supported_formats(&self) -> &[String] {
        &self.config.supported_formats
    }

    // Private helper methods

    fn detect_format(&self, data: &[u8], filename: &str) -> Result<TemplateFormat, ComponentError> {
        // First try filename extension
        if let Some(extension) = std::path::Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
        {
            match extension.as_str() {
                "csv" => {
                    return Ok(TemplateFormat::CSV {
                        delimiter: ',',
                        has_header: true,
                    })
                }
                "xlsx" | "xls" => return Ok(TemplateFormat::Excel { sheet_name: None }),
                "json" => {
                    return Ok(TemplateFormat::JSON {
                        schema_version: "1.0".to_string(),
                    })
                }
                "xml" => {
                    return Ok(TemplateFormat::XML {
                        root_element: "root".to_string(),
                    })
                }
                _ => {}
            }
        }

        // If auto-detection is enabled, analyze content
        if self.config.auto_detect_format {
            if let Ok(content) = std::str::from_utf8(data) {
                let trimmed = content.trim();

                // JSON detection
                if (trimmed.starts_with('{') && trimmed.ends_with('}'))
                    || (trimmed.starts_with('[') && trimmed.ends_with(']'))
                {
                    return Ok(TemplateFormat::JSON {
                        schema_version: "auto".to_string(),
                    });
                }

                // XML detection
                if trimmed.starts_with('<') && trimmed.contains("</") {
                    return Ok(TemplateFormat::XML {
                        root_element: "auto".to_string(),
                    });
                }

                // CSV detection (look for common delimiters)
                if content.lines().count() > 1 {
                    let first_line = content.lines().next().unwrap_or("");
                    if first_line.contains(',') {
                        return Ok(TemplateFormat::CSV {
                            delimiter: ',',
                            has_header: true,
                        });
                    } else if first_line.contains('\t') {
                        return Ok(TemplateFormat::CSV {
                            delimiter: '\t',
                            has_header: true,
                        });
                    } else if first_line.contains(';') {
                        return Ok(TemplateFormat::CSV {
                            delimiter: ';',
                            has_header: true,
                        });
                    }
                }
            }
        }

        Ok(TemplateFormat::Unknown)
    }

    fn validate_format(&self, format: &TemplateFormat, data: &[u8]) -> Result<(), String> {
        // Check file size
        if data.len() > self.config.max_file_size {
            return Err(format!(
                "File size {} exceeds maximum {}",
                data.len(),
                self.config.max_file_size
            ));
        }

        // Format-specific validation
        match format {
            TemplateFormat::CSV { .. } => {
                if let Ok(content) = std::str::from_utf8(data) {
                    if content.lines().count() < 2 {
                        return Err(
                            "CSV file must have at least 2 lines (header + data)".to_string()
                        );
                    }
                }
            }
            TemplateFormat::JSON { .. } => {
                if let Ok(content) = std::str::from_utf8(data) {
                    if serde_json::from_str::<serde_json::Value>(content).is_err() {
                        return Err("Invalid JSON format".to_string());
                    }
                }
            }
            TemplateFormat::Excel { .. } => {
                // Excel validation would require external library
                // For now, just check if it's binary data
                if data.len() < 8 {
                    return Err("Excel file too small to be valid".to_string());
                }
            }
            TemplateFormat::XML { .. } => {
                if let Ok(content) = std::str::from_utf8(data) {
                    if !content.trim_start().starts_with('<') {
                        return Err("Invalid XML format - must start with <".to_string());
                    }
                }
            }
            TemplateFormat::Unknown => {
                return Err("Unknown format cannot be validated".to_string());
            }
        }

        Ok(())
    }

    async fn extract_data(
        &self,
        format: &TemplateFormat,
        data: &[u8],
    ) -> Result<Vec<HashMap<String, String>>, ComponentError> {
        match format {
            TemplateFormat::CSV {
                delimiter,
                has_header,
            } => self.extract_csv_data(data, *delimiter, *has_header).await,
            TemplateFormat::JSON { .. } => self.extract_json_data(data).await,
            TemplateFormat::Excel { .. } => self.extract_excel_data(data).await,
            TemplateFormat::XML { .. } => self.extract_xml_data(data).await,
            TemplateFormat::Unknown => Err(ComponentError::ConfigurationError(
                "Cannot extract data from unknown format".to_string(),
            )),
        }
    }

    async fn extract_csv_data(
        &self,
        data: &[u8],
        delimiter: char,
        has_header: bool,
    ) -> Result<Vec<HashMap<String, String>>, ComponentError> {
        let content = std::str::from_utf8(data).map_err(|e| {
            ComponentError::ConfigurationError(format!("UTF-8 decode error: {}", e))
        })?;

        let mut lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return Ok(Vec::new());
        }

        // Parse header
        let headers = if has_header {
            let header_line = lines.remove(0);
            header_line
                .split(delimiter)
                .map(|h| h.trim().to_string())
                .collect::<Vec<String>>()
        } else {
            // Generate column names
            let first_line = lines.first().unwrap_or(&"");
            let column_count = first_line.split(delimiter).count();
            (0..column_count)
                .map(|i| format!("Column_{}", i + 1))
                .collect()
        };

        // Parse data rows
        let mut extracted_data = Vec::new();
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }

            let values: Vec<&str> = line.split(delimiter).collect();
            let mut row = HashMap::new();

            for (i, value) in values.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    row.insert(header.clone(), value.trim().to_string());
                }
            }

            extracted_data.push(row);
        }

        Ok(extracted_data)
    }

    async fn extract_json_data(
        &self,
        data: &[u8],
    ) -> Result<Vec<HashMap<String, String>>, ComponentError> {
        let content = std::str::from_utf8(data).map_err(|e| {
            ComponentError::ConfigurationError(format!("UTF-8 decode error: {}", e))
        })?;

        let json_value: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| ComponentError::ConfigurationError(format!("JSON parse error: {}", e)))?;

        let mut extracted_data = Vec::new();

        match json_value {
            serde_json::Value::Array(array) => {
                for item in array {
                    if let serde_json::Value::Object(obj) = item {
                        let mut row = HashMap::new();
                        for (key, value) in obj {
                            row.insert(key, value.to_string().trim_matches('"').to_string());
                        }
                        extracted_data.push(row);
                    }
                }
            }
            serde_json::Value::Object(obj) => {
                let mut row = HashMap::new();
                for (key, value) in obj {
                    row.insert(key, value.to_string().trim_matches('"').to_string());
                }
                extracted_data.push(row);
            }
            _ => {
                return Err(ComponentError::ConfigurationError(
                    "JSON must be object or array of objects".to_string(),
                ));
            }
        }

        Ok(extracted_data)
    }

    async fn extract_excel_data(
        &self,
        _data: &[u8],
    ) -> Result<Vec<HashMap<String, String>>, ComponentError> {
        // Mock Excel extraction - in real implementation would use calamine or similar
        let mut extracted_data = Vec::new();

        // Simulate Excel data
        let headers = vec![
            "Sample_ID".to_string(),
            "Sample_Type".to_string(),
            "Patient_ID".to_string(),
        ];

        for i in 1..=5 {
            let mut row = HashMap::new();
            row.insert("Sample_ID".to_string(), format!("S{:03}", i));
            row.insert("Sample_Type".to_string(), "Blood".to_string());
            row.insert("Patient_ID".to_string(), format!("P{:03}", i));
            extracted_data.push(row);
        }

        Ok(extracted_data)
    }

    async fn extract_xml_data(
        &self,
        data: &[u8],
    ) -> Result<Vec<HashMap<String, String>>, ComponentError> {
        let content = std::str::from_utf8(data).map_err(|e| {
            ComponentError::ConfigurationError(format!("UTF-8 decode error: {}", e))
        })?;

        // Simple XML parsing (in real implementation would use xml-rs or quick-xml)
        let mut extracted_data = Vec::new();

        // Mock XML extraction - look for simple patterns
        if content.contains("<sample") {
            let mut row = HashMap::new();
            row.insert("Sample_ID".to_string(), "XML_S001".to_string());
            row.insert("Sample_Type".to_string(), "Blood".to_string());
            row.insert("Format".to_string(), "XML".to_string());
            extracted_data.push(row);
        }

        Ok(extracted_data)
    }

    fn validate_schema(&self, data: &[HashMap<String, String>]) -> Vec<String> {
        let mut warnings = Vec::new();

        if data.is_empty() {
            warnings.push("No data extracted from template".to_string());
            return warnings;
        }

        // Check for common laboratory fields
        let required_fields = ["sample_id", "patient_id", "sample_type"];
        let first_row = &data[0];

        for field in required_fields {
            let field_variations = [
                field.to_string(),
                field.to_uppercase(),
                field.replace('_', " "),
                field.replace('_', ""),
            ];

            if !field_variations.iter().any(|variation| {
                first_row
                    .keys()
                    .any(|key| key.to_lowercase() == variation.to_lowercase())
            }) {
                warnings.push(format!("Recommended field '{}' not found", field));
            }
        }

        warnings
    }

    fn apply_data_mapping(&self, data: &mut [HashMap<String, String>]) {
        // Apply standard laboratory field mappings
        for row in data {
            let mut new_row = HashMap::new();

            for (key, value) in row.iter() {
                let normalized_key = self.normalize_field_name(key);
                new_row.insert(normalized_key, value.clone());
            }

            *row = new_row;
        }
    }

    fn normalize_field_name(&self, field: &str) -> String {
        let normalized = field
            .to_lowercase()
            .replace([' ', '-', '_'], "_")
            .trim()
            .to_string();

        // Common field mappings
        match normalized.as_str() {
            "id" | "sampleid" | "sample_identifier" => "sample_id".to_string(),
            "patient" | "patientid" | "patient_identifier" => "patient_id".to_string(),
            "type" | "sampletype" | "specimen_type" => "sample_type".to_string(),
            "date" | "collection_date" | "collectiondate" => "collection_date".to_string(),
            "volume" | "sample_volume" | "vol" => "volume".to_string(),
            "temp" | "temperature" | "storage_temp" => "temperature".to_string(),
            _ => normalized,
        }
    }

    fn calculate_data_completeness(&self, data: &[HashMap<String, String>]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let total_cells = data.len() * data[0].len();
        let non_empty_cells = data
            .iter()
            .flat_map(|row| row.values())
            .filter(|value| !value.trim().is_empty())
            .count();

        (non_empty_cells as f64 / total_cells as f64) * 100.0
    }

    fn estimate_memory_usage(&self, data: &[HashMap<String, String>]) -> usize {
        data.iter()
            .map(|row| row.iter().map(|(k, v)| k.len() + v.len()).sum::<usize>())
            .sum::<usize>()
            / 1024 // Convert to KB
    }

    async fn update_component_stats(
        &mut self,
        result: &TemplateResult,
        duration: std::time::Duration,
    ) {
        self.processing_stats.templates_processed += 1;

        if result.stage == TemplateStage::ValidationComplete {
            self.processing_stats.successful_extractions += 1;
        }

        // Update average processing time
        let total_processed = self.processing_stats.templates_processed as f64;
        let current_avg_ms = self.processing_stats.average_processing_time.as_millis() as f64;
        let new_duration_ms = duration.as_millis() as f64;

        let new_avg_ms =
            (current_avg_ms * (total_processed - 1.0) + new_duration_ms) / total_processed;
        self.processing_stats.average_processing_time =
            std::time::Duration::from_millis(new_avg_ms as u64);

        // Update format detection accuracy
        if result.format != TemplateFormat::Unknown {
            let successful_detections = self.processing_stats.successful_extractions as f64;
            self.processing_stats.format_detection_accuracy =
                (successful_detections / total_processed) * 100.0;
        }
    }
}

#[async_trait]
impl Component for TemplateProcessingComponent {
    fn component_id(&self) -> &'static str {
        "template_processing"
    }

    fn component_name(&self) -> &'static str {
        "Laboratory Template Processing Pipeline"
    }

    async fn initialize(&mut self, context: &ServiceRegistry) -> Result<(), ComponentError> {
        if self.is_initialized {
            return Ok(());
        }

        tracing::info!("Initializing template processing component");

        // Try to get optional services
        if let Some(db_service) = context.get_service::<()>("database_pool") {
            self.database_service = Some(db_service);
            tracing::info!("Database service connected for template processing");
        }

        if let Some(storage_service) = context.get_service::<()>("storage") {
            self.storage_service = Some(storage_service);
            tracing::info!("Storage service connected for template processing");
        }

        self.is_initialized = true;
        tracing::info!("Template processing component initialized successfully");

        Ok(())
    }

    async fn health_check(&self) -> Result<(), ComponentError> {
        if !self.is_initialized {
            return Err(ComponentError::InitializationFailed(
                "Component not initialized".to_string(),
            ));
        }

        // Check configuration validity
        if self.config.supported_formats.is_empty() {
            return Err(ComponentError::ConfigurationError(
                "No supported formats configured".to_string(),
            ));
        }

        if self.config.max_file_size == 0 {
            return Err(ComponentError::ConfigurationError(
                "Invalid max file size".to_string(),
            ));
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), ComponentError> {
        tracing::info!("Shutting down template processing component");

        // Log final statistics
        tracing::info!(
            "Template processing statistics: {:?}",
            self.processing_stats
        );

        // Clear cache and resources
        self.template_cache.clear();
        self.database_service = None;
        self.storage_service = None;
        self.is_initialized = false;

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[async_trait]
impl ServiceProvider for TemplateProcessingComponent {
    fn provided_services(&self) -> Vec<&'static str> {
        vec![
            "template_processing",
            "format_detection",
            "data_extraction",
            "schema_validation",
            "batch_processing",
        ]
    }
}

#[async_trait]
impl ServiceConsumer for TemplateProcessingComponent {
    fn required_services(&self) -> Vec<&'static str> {
        vec![] // No required services - can work standalone
    }

    fn optional_services(&self) -> Vec<&'static str> {
        vec!["database_pool", "storage", "event_system"]
    }

    async fn inject_service(
        &mut self,
        service_type: &str,
        service: std::sync::Arc<dyn Any + Send + Sync>,
    ) -> Result<(), ComponentError> {
        match service_type {
            "database_pool" => {
                self.database_service = Some(service);
                tracing::info!("Database service injected into template processing");
            }
            "storage" => {
                self.storage_service = Some(service);
                tracing::info!("Storage service injected into template processing");
            }
            _ => {
                return Err(ComponentError::ServiceInjectionFailed(format!(
                    "Unknown service type: {}",
                    service_type
                )));
            }
        }
        Ok(())
    }
}

impl Configurable for TemplateProcessingComponent {
    type Config = TemplateProcessingConfig;

    fn configure(&mut self, config: Self::Config) -> Result<(), ComponentError> {
        if self.is_initialized {
            return Err(ComponentError::ConfigurationError(
                "Cannot reconfigure initialized component".to_string(),
            ));
        }

        // Validate configuration
        if config.supported_formats.is_empty() {
            return Err(ComponentError::ConfigurationError(
                "Must support at least one format".to_string(),
            ));
        }

        if config.max_file_size == 0 {
            return Err(ComponentError::ConfigurationError(
                "Max file size must be greater than 0".to_string(),
            ));
        }

        self.config = config;
        Ok(())
    }

    fn get_config(&self) -> &Self::Config {
        &self.config
    }
}

/// Builder for template processing components
pub struct TemplateProcessingBuilder {
    config: TemplateProcessingConfig,
}

impl TemplateProcessingBuilder {
    pub fn new() -> Self {
        Self {
            config: TemplateProcessingConfig::default(),
        }
    }

    pub fn with_formats(mut self, formats: Vec<String>) -> Self {
        self.config.supported_formats = formats;
        self
    }

    pub fn with_max_file_size(mut self, size: usize) -> Self {
        self.config.max_file_size = size;
        self
    }

    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.config.batch_size = size;
        self
    }

    pub fn with_validation(mut self, enabled: bool) -> Self {
        self.config.enable_validation = enabled;
        self
    }

    pub fn with_auto_detection(mut self, enabled: bool) -> Self {
        self.config.auto_detect_format = enabled;
        self
    }

    pub fn for_csv_only(mut self) -> Self {
        self.config.supported_formats = vec!["csv".to_string()];
        self.config.auto_detect_format = false;
        self
    }

    pub fn for_high_throughput(mut self) -> Self {
        self.config.max_file_size = 100 * 1024 * 1024; // 100MB
        self.config.batch_size = 1000;
        self.config.cache_size = 500;
        self.config.enable_validation = false; // Skip validation for speed
        self
    }

    pub fn for_strict_validation(mut self) -> Self {
        self.config.enable_validation = true;
        self.config.auto_detect_format = true;
        self.config.batch_size = 50; // Smaller batches for thorough validation
        self
    }

    pub fn build(self) -> TemplateProcessingComponent {
        TemplateProcessingComponent::new(self.config)
    }
}

impl Default for TemplateProcessingBuilder {
    fn default() -> Self {
        Self::new()
    }
}
