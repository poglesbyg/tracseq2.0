use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

use crate::{
    errors::api::ApiError,
    sample_submission::CreateSample,
    services::{HealthCheck, HealthStatus, Service, ServiceConfig, ServiceHealth},
};

/// Configuration for RAG system integration
#[derive(Debug, Clone)]
pub struct RagConfig {
    pub base_url: String,
    pub timeout_seconds: u64,
    pub max_file_size_mb: u64,
    pub supported_formats: Vec<String>,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:8000".to_string(),
            timeout_seconds: 30,
            max_file_size_mb: 50,
            supported_formats: vec!["pdf".to_string(), "docx".to_string(), "txt".to_string()],
        }
    }
}

/// RAG extraction result models (matching the Python side)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RagExtractionResult {
    pub success: bool,
    pub submission: Option<RagSubmission>,
    pub confidence_score: f64,
    pub missing_fields: Vec<String>,
    pub warnings: Vec<String>,
    pub processing_time: f64,
    pub source_document: String,
}

/// Temporary adapter for Python API response format
#[derive(Debug, Serialize, Deserialize, Clone)]
struct PythonApiResponse {
    submission_id: String,
    status: String,
    message: String,
    success: Option<bool>,
    confidence_score: Option<f64>,
}

/// Make fields more flexible to handle actual RAG API responses
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlexibleRagSubmission {
    pub administrative_info: AdministrativeInfo,
    pub source_material: FlexibleSourceMaterial,
    pub pooling_info: PoolingInfo,
    pub sequence_generation: FlexibleSequenceGeneration,
    pub container_info: FlexibleContainerInfo,
    pub informatics_info: InformaticsInfo,
    pub sample_details: SampleDetails,
    pub submission_id: Option<String>,
    pub status: String,
    pub extracted_confidence: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlexibleSourceMaterial {
    #[serde(alias = "source_type")]
    pub material_type: String,
    pub extraction_method: Option<String>,
    #[serde(alias = "storage_conditions")]
    pub storage_temperature: Option<String>,
    pub collection_date: Option<String>,
    pub collection_method: Option<String>,
    pub source_organism: Option<String>,
    pub tissue_type: Option<String>,
    pub preservation_method: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlexibleSequenceGeneration {
    pub sequencing_platform: Option<String>,
    #[serde(alias = "read_length")]
    pub read_length: Option<String>, // Changed to String to handle "150bp paired-end"
    pub read_type: Option<String>,
    pub target_coverage: Option<String>, // Changed to String to handle "30x"
    pub library_prep_kit: Option<String>,
    pub index_sequences: Option<Vec<String>>,
    pub quality_metrics: Option<HashMap<String, f64>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlexibleContainerInfo {
    pub container_type: Option<String>,
    pub container_id: Option<String>,
    #[serde(alias = "volume_ul")]
    pub volume: Option<f64>,
    #[serde(alias = "concentration_ng_ul")]
    pub concentration: Option<f64>,
    pub diluent_used: Option<String>,
    pub storage_temperature: Option<String>,
    pub container_barcode: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RagSubmission {
    pub administrative_info: AdministrativeInfo,
    pub source_material: SourceMaterial,
    pub pooling_info: PoolingInfo,
    pub sequence_generation: SequenceGeneration,
    pub container_info: ContainerInfo,
    pub informatics_info: InformaticsInfo,
    pub sample_details: SampleDetails,
    pub submission_id: Option<String>,
    pub status: String,
    pub extracted_confidence: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdministrativeInfo {
    pub submitter_first_name: String,
    pub submitter_last_name: String,
    pub submitter_email: String,
    pub submitter_phone: String,
    pub assigned_project: String,
    pub department: Option<String>,
    pub institution: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SourceMaterial {
    pub source_type: String,
    pub collection_date: Option<String>,
    pub collection_method: Option<String>,
    pub source_organism: Option<String>,
    pub tissue_type: Option<String>,
    pub preservation_method: Option<String>,
    pub storage_conditions: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoolingInfo {
    pub is_pooled: bool,
    pub pool_id: Option<String>,
    pub samples_in_pool: Vec<String>,
    #[serde(default)]
    pub pooling_ratio: HashMap<String, f64>,
    #[serde(default)]
    pub barcode_sequences: HashMap<String, String>,
    #[serde(alias = "pooling_strategy")]
    pub multiplex_strategy: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SequenceGeneration {
    pub sequencing_platform: Option<String>,
    pub read_length: Option<i32>,
    pub read_type: Option<String>,
    pub target_coverage: Option<f64>,
    pub library_prep_kit: Option<String>,
    #[serde(default)]
    pub index_sequences: Vec<String>,
    #[serde(default)]
    pub quality_metrics: HashMap<String, f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContainerInfo {
    pub container_type: Option<String>,
    pub container_id: Option<String>,
    pub volume: Option<f64>,
    pub concentration: Option<f64>,
    pub diluent_used: Option<String>,
    pub storage_temperature: Option<String>,
    pub container_barcode: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InformaticsInfo {
    pub analysis_type: String,
    pub reference_genome: Option<String>,
    pub analysis_pipeline: Option<String>,
    #[serde(default)]
    pub custom_parameters: HashMap<String, Value>,
    pub data_delivery_format: Option<String>,
    pub computational_requirements: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SampleDetails {
    pub sample_id: String,
    pub patient_id: Option<String>,
    pub sample_name: Option<String>,
    pub priority: String,
    pub quality_score: Option<f64>,
    pub purity_ratio: Option<f64>,
    pub integrity_number: Option<f64>,
    pub notes: Option<String>,
    pub special_instructions: Option<String>,
}

/// Enhanced sample creation request with RAG integration
#[derive(Debug, Serialize, Deserialize)]
pub struct RagEnhancedSampleRequest {
    pub document_path: Option<String>,
    pub manual_data: Option<CreateSample>,
    pub use_rag_extraction: bool,
    pub confidence_threshold: Option<f64>,
}

/// Result of RAG-enhanced sample creation
#[derive(Debug, Serialize, Deserialize)]
pub struct RagEnhancedSampleResult {
    pub samples: Vec<CreateSample>,
    pub extraction_result: Option<RagExtractionResult>,
    pub confidence_score: f64,
    pub validation_warnings: Vec<String>,
    pub processing_time: f64,
}

/// Service for integrating with the RAG LLM system
pub struct RagIntegrationService {
    client: Client,
    config: RagConfig,
}

impl RagIntegrationService {
    pub fn new(config: RagConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    /// Process a document using the RAG system and extract sample information
    pub async fn process_document(
        &self,
        document_path: &str,
    ) -> Result<RagExtractionResult, ApiError> {
        // Check if file exists and is supported
        let path = Path::new(document_path);
        if !path.exists() {
            return Err(ApiError::BadRequest(
                "Document file does not exist".to_string(),
            ));
        }

        let file_extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !self.config.supported_formats.contains(&file_extension) {
            return Err(ApiError::BadRequest(format!(
                "Unsupported file format: {}. Supported formats: {:?}",
                file_extension, self.config.supported_formats
            )));
        }

        // Check file size
        let metadata = fs::metadata(path).await.map_err(|e| {
            ApiError::InternalServerError(format!("Failed to read file metadata: {}", e))
        })?;

        let file_size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
        if file_size_mb > self.config.max_file_size_mb as f64 {
            return Err(ApiError::BadRequest(format!(
                "File size ({:.2} MB) exceeds maximum allowed size ({} MB)",
                file_size_mb, self.config.max_file_size_mb
            )));
        }

        // Make request to RAG system
        let url = format!("{}/process", self.config.base_url);

        // Read file and create form
        let file_bytes = fs::read(document_path)
            .await
            .map_err(|e| ApiError::InternalServerError(format!("Failed to read file: {}", e)))?;

        let file_part = reqwest::multipart::Part::bytes(file_bytes).file_name(
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        );

        let form = reqwest::multipart::Form::new().part("file", file_part);

        let response = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                ApiError::ServiceUnavailable(format!("RAG system is unavailable: {}", e))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApiError::InternalServerError(format!(
                "RAG system error: {}",
                error_text
            )));
        }

        // Parse the response
        let response_text = response.text().await.map_err(|e| {
            ApiError::InternalServerError(format!("Failed to read RAG response: {}", e))
        })?;

        // Parse the actual RAG response format with flexible structures
        #[derive(Debug, Serialize, Deserialize)]
        struct FlexibleRagResponse {
            success: bool,
            submission: Option<FlexibleRagSubmission>,
            confidence_score: f64,
            missing_fields: Vec<String>,
            warnings: Vec<String>,
            processing_time: f64,
            source_document: String,
        }

        let flexible_response: FlexibleRagResponse =
            serde_json::from_str(&response_text).map_err(|e| {
                ApiError::InternalServerError(format!(
                    "Failed to parse RAG response: {}. Response: {}",
                    e, response_text
                ))
            })?;

        // Convert flexible response to expected format
        let submission = if let Some(flex_submission) = flexible_response.submission {
            Some(RagSubmission {
                administrative_info: flex_submission.administrative_info,
                source_material: SourceMaterial {
                    source_type: flex_submission.source_material.material_type,
                    collection_date: flex_submission.source_material.collection_date,
                    collection_method: flex_submission.source_material.collection_method,
                    source_organism: flex_submission.source_material.source_organism,
                    tissue_type: flex_submission.source_material.tissue_type,
                    preservation_method: flex_submission.source_material.preservation_method,
                    storage_conditions: flex_submission.source_material.storage_temperature,
                },
                pooling_info: flex_submission.pooling_info,
                sequence_generation: SequenceGeneration {
                    sequencing_platform: flex_submission.sequence_generation.sequencing_platform,
                    read_length: None, // Parse from string if needed
                    read_type: flex_submission.sequence_generation.read_type,
                    target_coverage: None, // Parse from string if needed
                    library_prep_kit: flex_submission.sequence_generation.library_prep_kit,
                    index_sequences: flex_submission
                        .sequence_generation
                        .index_sequences
                        .unwrap_or_default(),
                    quality_metrics: flex_submission
                        .sequence_generation
                        .quality_metrics
                        .unwrap_or_default(),
                },
                container_info: ContainerInfo {
                    container_type: flex_submission.container_info.container_type,
                    container_id: flex_submission.container_info.container_id,
                    volume: flex_submission.container_info.volume,
                    concentration: flex_submission.container_info.concentration,
                    diluent_used: flex_submission.container_info.diluent_used,
                    storage_temperature: flex_submission.container_info.storage_temperature,
                    container_barcode: flex_submission.container_info.container_barcode,
                },
                informatics_info: flex_submission.informatics_info,
                sample_details: flex_submission.sample_details,
                submission_id: flex_submission.submission_id,
                status: flex_submission.status,
                extracted_confidence: flex_submission.extracted_confidence,
            })
        } else {
            None
        };

        let extraction_result = RagExtractionResult {
            success: flexible_response.success,
            submission,
            confidence_score: flexible_response.confidence_score,
            missing_fields: flexible_response.missing_fields,
            warnings: flexible_response.warnings,
            processing_time: flexible_response.processing_time,
            source_document: flexible_response.source_document,
        };

        Ok(extraction_result)
    }

    /// Query the RAG system for specific information
    pub async fn query_submissions(&self, query: &str) -> Result<String, ApiError> {
        let url = format!("{}/query", self.config.base_url);
        let request_body = serde_json::json!({
            "query": query
        });

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                ApiError::ServiceUnavailable(format!("RAG system is unavailable: {}", e))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ApiError::InternalServerError(format!(
                "RAG query error: {}",
                error_text
            )));
        }

        let answer: Value = response.json().await.map_err(|e| {
            ApiError::InternalServerError(format!("Failed to parse RAG response: {}", e))
        })?;

        Ok(answer
            .get("answer")
            .and_then(|a| a.as_str())
            .unwrap_or("No answer available")
            .to_string())
    }

    /// Convert RAG extraction result to lab manager sample format
    pub fn convert_to_samples(
        &self,
        extraction_result: &RagExtractionResult,
    ) -> Result<Vec<CreateSample>, ApiError> {
        if !extraction_result.success {
            return Err(ApiError::BadRequest(
                "RAG extraction was not successful".to_string(),
            ));
        }

        let submission = extraction_result.submission.as_ref().ok_or_else(|| {
            ApiError::BadRequest("No submission data in extraction result".to_string())
        })?;

        let mut samples = Vec::new();

        // Handle pooled vs individual samples
        if submission.pooling_info.is_pooled && !submission.pooling_info.samples_in_pool.is_empty()
        {
            // Create samples for each sample in the pool
            for (index, sample_id) in submission.pooling_info.samples_in_pool.iter().enumerate() {
                let sample =
                    self.create_sample_from_submission(submission, Some(sample_id), index)?;
                samples.push(sample);
            }
        } else {
            // Create a single sample
            let sample = self.create_sample_from_submission(submission, None, 0)?;
            samples.push(sample);
        }

        Ok(samples)
    }

    /// Create a sample from RAG submission data
    fn create_sample_from_submission(
        &self,
        submission: &RagSubmission,
        sample_id_override: Option<&String>,
        index: usize,
    ) -> Result<CreateSample, ApiError> {
        // Generate sample name
        let sample_name = sample_id_override
            .cloned()
            .or_else(|| submission.sample_details.sample_name.clone())
            .unwrap_or_else(|| {
                format!(
                    "{}-{}-{}",
                    submission.administrative_info.assigned_project,
                    submission.source_material.source_type,
                    index + 1
                )
            });

        // Generate barcode (use container barcode if available, otherwise generate)
        let barcode = submission
            .container_info
            .container_barcode
            .clone()
            .or_else(|| {
                submission
                    .pooling_info
                    .barcode_sequences
                    .values()
                    .next()
                    .cloned()
            })
            .unwrap_or_else(|| self.generate_barcode(&submission.source_material.source_type));

        // Determine storage location
        let location = submission
            .container_info
            .storage_temperature
            .as_ref()
            .map(|temp| format!("Storage-{}", temp))
            .unwrap_or_else(|| "Unknown-Location".to_string());

        // Build comprehensive metadata
        let metadata = serde_json::json!({
            "rag_extraction": {
                "confidence_score": submission.extracted_confidence,
                "source_document": submission.submission_id,
                "administrative_info": submission.administrative_info,
                "source_material": submission.source_material,
                "sequence_generation": submission.sequence_generation,
                "container_info": submission.container_info,
                "informatics_info": submission.informatics_info,
                "sample_details": submission.sample_details
            },
            "processing": {
                "extracted_at": chrono::Utc::now().to_rfc3339(),
                "priority": submission.sample_details.priority,
                "quality_score": submission.sample_details.quality_score,
                "special_instructions": submission.sample_details.special_instructions
            }
        });

        Ok(CreateSample {
            name: sample_name,
            barcode,
            location,
            metadata: Some(metadata),
        })
    }

    /// Generate a unique barcode for the sample
    fn generate_barcode(&self, source_type: &str) -> String {
        let prefix = match source_type.to_lowercase().as_str() {
            "blood" => "BLD",
            "saliva" => "SAL",
            "tissue" => "TSU",
            "dna" => "DNA",
            "rna" => "RNA",
            _ => "UNK",
        };

        let timestamp = chrono::Utc::now().format("%y%m%d%H%M");
        let random_suffix: String = (0..3)
            .map(|_| fastrand::alphanumeric())
            .collect::<String>()
            .to_uppercase();

        format!("{}-{}-{}", prefix, timestamp, random_suffix)
    }

    /// Check if the RAG system is available and healthy
    pub async fn check_health(&self) -> Result<Value, ApiError> {
        let url = format!("{}/health", self.config.base_url);

        let response =
            self.client.get(&url).send().await.map_err(|_| {
                ApiError::ServiceUnavailable("RAG system is unavailable".to_string())
            })?;

        if !response.status().is_success() {
            return Err(ApiError::ServiceUnavailable(
                "RAG system health check failed".to_string(),
            ));
        }

        let health_data: Value = response.json().await.map_err(|e| {
            ApiError::InternalServerError(format!("Failed to parse health response: {}", e))
        })?;

        Ok(health_data)
    }
}

#[async_trait]
impl Service for RagIntegrationService {
    fn name(&self) -> &'static str {
        "rag_integration_service"
    }

    async fn health_check(&self) -> ServiceHealth {
        let mut checks = HashMap::new();

        // Check RAG system connectivity
        let start = std::time::Instant::now();
        let rag_check = match self.check_health().await {
            Ok(_) => HealthCheck {
                status: HealthStatus::Healthy,
                duration_ms: start.elapsed().as_millis() as u64,
                details: Some("RAG system is operational".to_string()),
            },
            Err(e) => HealthCheck {
                status: HealthStatus::Unhealthy,
                duration_ms: start.elapsed().as_millis() as u64,
                details: Some(format!("RAG system error: {}", e)),
            },
        };

        checks.insert("rag_system".to_string(), rag_check.clone());

        ServiceHealth {
            status: rag_check.status,
            message: Some("RAG Integration service health check".to_string()),
            checks,
        }
    }

    fn config(&self) -> ServiceConfig {
        ServiceConfig {
            name: "rag_integration_service".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["rag_system".to_string()],
            settings: [
                ("base_url".to_string(), self.config.base_url.clone()),
                (
                    "timeout_seconds".to_string(),
                    self.config.timeout_seconds.to_string(),
                ),
                (
                    "max_file_size_mb".to_string(),
                    self.config.max_file_size_mb.to_string(),
                ),
            ]
            .into_iter()
            .collect(),
        }
    }
}
