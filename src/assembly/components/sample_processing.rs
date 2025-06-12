use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::sync::Arc;

use super::super::traits::{
    Component, ComponentError, Configurable, ServiceConsumer, ServiceProvider, ServiceRegistry,
};

/// Configuration for the sample processing component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleProcessingConfig {
    /// Enable RAG processing for document analysis
    pub enable_rag: bool,
    /// Minimum confidence score for auto-validation
    pub confidence_threshold: f64,
    /// Maximum file size for document uploads (in bytes)
    pub max_file_size: usize,
    /// Supported document formats
    pub supported_formats: Vec<String>,
    /// Enable automatic barcode generation
    pub auto_generate_barcodes: bool,
    /// Barcode prefix for laboratory naming convention
    pub barcode_prefix: String,
}

impl Default for SampleProcessingConfig {
    fn default() -> Self {
        Self {
            enable_rag: true,
            confidence_threshold: 0.8,
            max_file_size: 10 * 1024 * 1024, // 10MB
            supported_formats: vec![
                "pdf".to_string(),
                "docx".to_string(),
                "txt".to_string(),
                "csv".to_string(),
            ],
            auto_generate_barcodes: true,
            barcode_prefix: "LAB".to_string(),
        }
    }
}

/// Processing pipeline stages
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingStage {
    DocumentUpload,
    FormatValidation,
    RagProcessing,
    ConfidenceScoring,
    DataExtraction,
    SampleCreation,
    BarcodeGeneration,
    ValidationComplete,
}

/// Processing result with confidence and metadata
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    pub stage: ProcessingStage,
    pub confidence: f64,
    pub extracted_data: serde_json::Value,
    pub metadata: std::collections::HashMap<String, String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Sample processing pipeline component
pub struct SampleProcessingComponent {
    config: SampleProcessingConfig,
    database_service: Option<Arc<dyn Any + Send + Sync>>,
    storage_service: Option<Arc<dyn Any + Send + Sync>>,
    is_initialized: bool,
    processing_stats: ProcessingStats,
}

#[derive(Debug, Default)]
struct ProcessingStats {
    documents_processed: u64,
    samples_created: u64,
    average_confidence: f64,
    processing_errors: u64,
}

impl SampleProcessingComponent {
    pub fn new(config: SampleProcessingConfig) -> Self {
        Self {
            config,
            database_service: None,
            storage_service: None,
            is_initialized: false,
            processing_stats: ProcessingStats::default(),
        }
    }

    /// Process a document through the sample submission pipeline
    pub async fn process_document(
        &mut self,
        document_data: &[u8],
        filename: &str,
    ) -> Result<ProcessingResult, ComponentError> {
        if !self.is_initialized {
            return Err(ComponentError::InitializationFailed(
                "Component not initialized".to_string(),
            ));
        }

        let mut result = ProcessingResult {
            stage: ProcessingStage::DocumentUpload,
            confidence: 0.0,
            extracted_data: serde_json::Value::Null,
            metadata: std::collections::HashMap::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Stage 1: Format validation
        result.stage = ProcessingStage::FormatValidation;
        if let Err(validation_error) = self.validate_document_format(filename) {
            result.errors.push(validation_error);
            return Ok(result);
        }

        // Stage 2: Size validation
        if document_data.len() > self.config.max_file_size {
            result.errors.push(format!(
                "Document size {} exceeds maximum allowed size {}",
                document_data.len(),
                self.config.max_file_size
            ));
            return Ok(result);
        }

        // Stage 3: RAG processing (if enabled)
        if self.config.enable_rag {
            result.stage = ProcessingStage::RagProcessing;
            match self.process_with_rag(document_data, filename).await {
                Ok((confidence, extracted_data)) => {
                    result.confidence = confidence;
                    result.extracted_data = extracted_data;
                    result.stage = ProcessingStage::ConfidenceScoring;
                }
                Err(e) => {
                    result.errors.push(format!("RAG processing failed: {}", e));
                    return Ok(result);
                }
            }
        }

        // Stage 4: Data extraction and validation
        result.stage = ProcessingStage::DataExtraction;
        if result.confidence >= self.config.confidence_threshold {
            result.stage = ProcessingStage::SampleCreation;

            // Stage 5: Barcode generation (if enabled)
            if self.config.auto_generate_barcodes {
                result.stage = ProcessingStage::BarcodeGeneration;
                let barcode = self.generate_barcode();
                result.metadata.insert("barcode".to_string(), barcode);
            }

            result.stage = ProcessingStage::ValidationComplete;
            self.processing_stats.samples_created += 1;
        } else {
            result.warnings.push(format!(
                "Confidence {} below threshold {}, manual review required",
                result.confidence, self.config.confidence_threshold
            ));
        }

        // Update processing stats
        self.processing_stats.documents_processed += 1;
        self.update_average_confidence(result.confidence);

        Ok(result)
    }

    /// Get processing statistics
    pub fn get_stats(&self) -> &ProcessingStats {
        &self.processing_stats
    }

    /// Reset processing statistics
    pub fn reset_stats(&mut self) {
        self.processing_stats = ProcessingStats::default();
    }

    // Private helper methods

    fn validate_document_format(&self, filename: &str) -> Result<(), String> {
        let extension = std::path::Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .ok_or_else(|| "Unable to determine file extension".to_string())?;

        if !self.config.supported_formats.contains(&extension) {
            return Err(format!(
                "Unsupported file format: {}. Supported formats: {:?}",
                extension, self.config.supported_formats
            ));
        }

        Ok(())
    }

    async fn process_with_rag(
        &self,
        _document_data: &[u8],
        _filename: &str,
    ) -> Result<(f64, serde_json::Value), ComponentError> {
        // Mock RAG processing - in real implementation, this would:
        // 1. Extract text from document
        // 2. Process with RAG pipeline
        // 3. Extract structured data
        // 4. Calculate confidence scores

        // Simulate processing time
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Mock extracted data
        let extracted_data = serde_json::json!({
            "sample_type": "blood",
            "collection_date": "2024-01-15",
            "patient_id": "P12345",
            "laboratory": "Central Lab",
            "volume": "5ml",
            "temperature": "4C"
        });

        // Mock confidence score
        let confidence = 0.92;

        Ok((confidence, extracted_data))
    }

    fn generate_barcode(&self) -> String {
        use fastrand;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let random_part = fastrand::u32(1000..9999);

        format!(
            "{}-{}-{}",
            self.config.barcode_prefix, timestamp, random_part
        )
    }

    fn update_average_confidence(&mut self, new_confidence: f64) {
        let total_docs = self.processing_stats.documents_processed as f64;
        let current_avg = self.processing_stats.average_confidence;

        // Calculate running average
        self.processing_stats.average_confidence =
            (current_avg * (total_docs - 1.0) + new_confidence) / total_docs;
    }
}

#[async_trait]
impl Component for SampleProcessingComponent {
    fn component_id(&self) -> &'static str {
        "sample_processing"
    }

    fn component_name(&self) -> &'static str {
        "Sample Processing Pipeline"
    }

    async fn initialize(&mut self, _context: &ServiceRegistry) -> Result<(), ComponentError> {
        self.is_initialized = true;
        tracing::info!("Sample processing component initialized");
        Ok(())
    }

    async fn health_check(&self) -> Result<(), ComponentError> {
        if !self.is_initialized {
            return Err(ComponentError::InitializationFailed(
                "Component not initialized".to_string(),
            ));
        }
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), ComponentError> {
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
impl ServiceProvider for SampleProcessingComponent {
    fn provided_services(&self) -> Vec<&'static str> {
        vec![
            "sample_processing",
            "document_processing",
            "barcode_generation",
        ]
    }
}

#[async_trait]
impl ServiceConsumer for SampleProcessingComponent {
    fn required_services(&self) -> Vec<&'static str> {
        vec!["database_pool", "storage"]
    }

    async fn inject_service(
        &mut self,
        service_type: &str,
        service: Arc<dyn Any + Send + Sync>,
    ) -> Result<(), ComponentError> {
        match service_type {
            "database_pool" => {
                self.database_service = Some(service);
                tracing::info!("Database service injected");
            }
            "storage" => {
                self.storage_service = Some(service);
                tracing::info!("Storage service injected");
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

impl Configurable for SampleProcessingComponent {
    type Config = SampleProcessingConfig;

    fn configure(&mut self, config: Self::Config) -> Result<(), ComponentError> {
        if self.is_initialized {
            return Err(ComponentError::ConfigurationError(
                "Cannot reconfigure initialized component".to_string(),
            ));
        }

        // Validate configuration
        if config.confidence_threshold < 0.0 || config.confidence_threshold > 1.0 {
            return Err(ComponentError::ConfigurationError(
                "Confidence threshold must be between 0.0 and 1.0".to_string(),
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

/// Builder for creating sample processing components
pub struct SampleProcessingBuilder {
    config: SampleProcessingConfig,
}

impl SampleProcessingBuilder {
    pub fn new() -> Self {
        Self {
            config: SampleProcessingConfig::default(),
        }
    }

    /// Enable or disable RAG processing
    pub fn with_rag(mut self, enabled: bool) -> Self {
        self.config.enable_rag = enabled;
        self
    }

    /// Set confidence threshold
    pub fn with_confidence_threshold(mut self, threshold: f64) -> Self {
        self.config.confidence_threshold = threshold;
        self
    }

    /// Set maximum file size
    pub fn with_max_file_size(mut self, size: usize) -> Self {
        self.config.max_file_size = size;
        self
    }

    /// Set supported formats
    pub fn with_formats(mut self, formats: Vec<String>) -> Self {
        self.config.supported_formats = formats;
        self
    }

    /// Set barcode configuration
    pub fn with_barcode_config(mut self, auto_generate: bool, prefix: String) -> Self {
        self.config.auto_generate_barcodes = auto_generate;
        self.config.barcode_prefix = prefix;
        self
    }

    /// Configure for high-throughput processing
    pub fn for_high_throughput(mut self) -> Self {
        self.config.max_file_size = 50 * 1024 * 1024; // 50MB
        self.config.confidence_threshold = 0.7; // Lower threshold for speed
        self.config.supported_formats = vec![
            "pdf".to_string(),
            "docx".to_string(),
            "txt".to_string(),
            "csv".to_string(),
            "xlsx".to_string(),
        ];
        self
    }

    /// Configure for high-accuracy processing
    pub fn for_high_accuracy(mut self) -> Self {
        self.config.confidence_threshold = 0.95; // Higher threshold for accuracy
        self.config.enable_rag = true;
        self
    }

    /// Build the component
    pub fn build(self) -> SampleProcessingComponent {
        SampleProcessingComponent::new(self.config)
    }
}

impl Default for SampleProcessingBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience macro for creating sample processing components
#[macro_export]
macro_rules! sample_processing_component {
    // Default configuration
    () => {
        $crate::assembly::components::sample_processing::SampleProcessingBuilder::new().build()
    };

    // High-throughput configuration
    (high_throughput) => {
        $crate::assembly::components::sample_processing::SampleProcessingBuilder::new()
            .for_high_throughput()
            .build()
    };

    // High-accuracy configuration
    (high_accuracy) => {
        $crate::assembly::components::sample_processing::SampleProcessingBuilder::new()
            .for_high_accuracy()
            .build()
    };

    // Custom configuration
    (confidence: $threshold:expr, rag: $rag_enabled:expr) => {
        $crate::assembly::components::sample_processing::SampleProcessingBuilder::new()
            .with_confidence_threshold($threshold)
            .with_rag($rag_enabled)
            .build()
    };
}
