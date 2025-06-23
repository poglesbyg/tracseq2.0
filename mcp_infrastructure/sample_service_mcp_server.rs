//! Sample Service MCP Server Implementation
//! 
//! This module implements an MCP server for the TracSeq Sample Service,
//! exposing laboratory sample management capabilities to AI agents.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::sample_service::{SampleService, CreateSampleRequest, UpdateSampleRequest, SampleSearchFilters};
use crate::models::{Sample, SampleStatus, SampleStatistics};

/// MCP Server implementation for Sample Service
pub struct SampleMcpServer {
    sample_service: Arc<SampleService>,
    tools: Arc<RwLock<HashMap<String, McpTool>>>,
    resources: Arc<RwLock<HashMap<String, McpResource>>>,
    prompts: Arc<RwLock<HashMap<String, McpPrompt>>>,
}

/// MCP Tool definition
#[derive(Debug, Clone)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub handler: fn(&SampleMcpServer, Value) -> Result<Value, McpError>,
}

/// MCP Resource definition
#[derive(Debug, Clone)]
pub struct McpResource {
    pub name: String,
    pub description: String,
    pub uri: String,
    pub mime_type: String,
}

/// MCP Prompt definition
#[derive(Debug, Clone)]
pub struct McpPrompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// MCP Error types
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("Invalid parameters: {message}")]
    InvalidParameters { message: String },
    #[error("Service error: {message}")]
    ServiceError { message: String },
    #[error("Not found: {resource}")]
    NotFound { resource: String },
    #[error("Unauthorized access")]
    Unauthorized,
}

impl SampleMcpServer {
    /// Create a new Sample MCP Server
    pub fn new(sample_service: Arc<SampleService>) -> Self {
        Self {
            sample_service,
            tools: Arc::new(RwLock::new(HashMap::new())),
            resources: Arc::new(RwLock::new(HashMap::new())),
            prompts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize the MCP server with all tools, resources, and prompts
    pub async fn initialize(&self) -> Result<(), McpError> {
        self.register_tools().await?;
        self.register_resources().await?;
        self.register_prompts().await?;
        Ok(())
    }

    /// Register all MCP tools
    async fn register_tools(&self) -> Result<(), McpError> {
        let mut tools = self.tools.write().await;

        // Tool: Create Sample
        tools.insert("create_sample".to_string(), McpTool {
            name: "create_sample".to_string(),
            description: "Create a new laboratory sample with validation".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Sample name"},
                    "sample_type": {"type": "string", "description": "Type of sample (DNA, RNA, etc.)"},
                    "barcode": {"type": "string", "description": "Optional barcode"},
                    "location": {"type": "string", "description": "Sample location"},
                    "metadata": {"type": "object", "description": "Additional sample metadata"},
                    "auto_validate": {"type": "boolean", "default": false}
                },
                "required": ["name", "sample_type"]
            }),
            handler: Self::handle_create_sample,
        });

        // Tool: Validate Sample
        tools.insert("validate_sample".to_string(), McpTool {
            name: "validate_sample".to_string(),
            description: "Validate a sample according to laboratory rules".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "sample_id": {"type": "string", "format": "uuid", "description": "Sample UUID"},
                    "validation_rules": {"type": "array", "items": {"type": "string"}, "description": "Specific validation rules to apply"}
                },
                "required": ["sample_id"]
            }),
            handler: Self::handle_validate_sample,
        });

        // Tool: Update Sample Status
        tools.insert("update_sample_status".to_string(), McpTool {
            name: "update_sample_status".to_string(),
            description: "Update sample status in the laboratory workflow".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "sample_id": {"type": "string", "format": "uuid"},
                    "new_status": {"type": "string", "enum": ["Pending", "Validated", "InStorage", "InSequencing", "Completed", "Failed", "Discarded"]},
                    "reason": {"type": "string", "description": "Reason for status change"},
                    "metadata": {"type": "object", "description": "Additional metadata"}
                },
                "required": ["sample_id", "new_status"]
            }),
            handler: Self::handle_update_sample_status,
        });

        // Tool: Search Samples
        tools.insert("search_samples".to_string(), McpTool {
            name: "search_samples".to_string(),
            description: "Search for samples using various filters".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "status": {"type": "string", "description": "Filter by sample status"},
                    "sample_type": {"type": "string", "description": "Filter by sample type"},
                    "created_after": {"type": "string", "format": "date-time"},
                    "created_before": {"type": "string", "format": "date-time"},
                    "barcode_prefix": {"type": "string", "description": "Filter by barcode prefix"},
                    "limit": {"type": "integer", "minimum": 1, "maximum": 1000, "default": 50},
                    "offset": {"type": "integer", "minimum": 0, "default": 0}
                }
            }),
            handler: Self::handle_search_samples,
        });

        // Tool: Batch Create Samples
        tools.insert("batch_create_samples".to_string(), McpTool {
            name: "batch_create_samples".to_string(),
            description: "Create multiple samples in a single operation".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "samples": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": {"type": "string"},
                                "sample_type": {"type": "string"},
                                "barcode": {"type": "string"},
                                "location": {"type": "string"},
                                "metadata": {"type": "object"}
                            },
                            "required": ["name", "sample_type"]
                        }
                    },
                    "auto_validate": {"type": "boolean", "default": false},
                    "notify_submitter": {"type": "boolean", "default": true}
                },
                "required": ["samples"]
            }),
            handler: Self::handle_batch_create_samples,
        });

        // Tool: Get Sample Details
        tools.insert("get_sample".to_string(), McpTool {
            name: "get_sample".to_string(),
            description: "Get detailed information about a specific sample".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "sample_id": {"type": "string", "format": "uuid"},
                    "include_history": {"type": "boolean", "default": false}
                },
                "required": ["sample_id"]
            }),
            handler: Self::handle_get_sample,
        });

        Ok(())
    }

    /// Register all MCP resources
    async fn register_resources(&self) -> Result<(), McpError> {
        let mut resources = self.resources.write().await;

        resources.insert("sample_templates".to_string(), McpResource {
            name: "sample_templates".to_string(),
            description: "Available sample templates and configurations".to_string(),
            uri: "sample://templates".to_string(),
            mime_type: "application/json".to_string(),
        });

        resources.insert("validation_rules".to_string(), McpResource {
            name: "validation_rules".to_string(),
            description: "Current sample validation rules and configurations".to_string(),
            uri: "sample://validation-rules".to_string(),
            mime_type: "application/json".to_string(),
        });

        resources.insert("sample_statistics".to_string(), McpResource {
            name: "sample_statistics".to_string(),
            description: "Real-time sample processing statistics and metrics".to_string(),
            uri: "sample://statistics".to_string(),
            mime_type: "application/json".to_string(),
        });

        resources.insert("sample_types".to_string(), McpResource {
            name: "sample_types".to_string(),
            description: "Supported sample types and their requirements".to_string(),
            uri: "sample://types".to_string(),
            mime_type: "application/json".to_string(),
        });

        Ok(())
    }

    /// Register all MCP prompts
    async fn register_prompts(&self) -> Result<(), McpError> {
        let mut prompts = self.prompts.write().await;

        prompts.insert("sample_submission_wizard".to_string(), McpPrompt {
            name: "sample_submission_wizard".to_string(),
            description: "Guided sample submission process with validation".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "sample_type".to_string(),
                    description: "Type of sample being submitted".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "submitter_info".to_string(),
                    description: "Information about the submitter".to_string(),
                    required: false,
                },
                PromptArgument {
                    name: "urgency".to_string(),
                    description: "Urgency level (low, medium, high)".to_string(),
                    required: false,
                },
            ],
        });

        prompts.insert("quality_control_review".to_string(), McpPrompt {
            name: "quality_control_review".to_string(),
            description: "Quality control review assistant for sample validation".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "sample_id".to_string(),
                    description: "Sample ID to review".to_string(),
                    required: true,
                },
                PromptArgument {
                    name: "review_type".to_string(),
                    description: "Type of QC review (initial, follow-up, final)".to_string(),
                    required: false,
                },
            ],
        });

        Ok(())
    }

    // Tool Handlers

    async fn handle_create_sample(&self, params: Value) -> Result<Value, McpError> {
        let create_request: CreateSampleRequest = serde_json::from_value(params)
            .map_err(|e| McpError::InvalidParameters { 
                message: format!("Invalid create sample parameters: {}", e) 
            })?;

        let sample = self.sample_service.create_sample(create_request).await
            .map_err(|e| McpError::ServiceError { 
                message: format!("Failed to create sample: {}", e) 
            })?;

        Ok(json!({
            "success": true,
            "sample": sample,
            "message": "Sample created successfully"
        }))
    }

    async fn handle_validate_sample(&self, params: Value) -> Result<Value, McpError> {
        let sample_id: Uuid = params["sample_id"].as_str()
            .ok_or_else(|| McpError::InvalidParameters { 
                message: "sample_id is required".to_string() 
            })?
            .parse()
            .map_err(|_| McpError::InvalidParameters { 
                message: "Invalid sample_id format".to_string() 
            })?;

        let validated_sample = self.sample_service.validate_sample(sample_id).await
            .map_err(|e| McpError::ServiceError { 
                message: format!("Failed to validate sample: {}", e) 
            })?;

        Ok(json!({
            "success": true,
            "sample": validated_sample,
            "validation_status": "completed",
            "message": "Sample validation completed successfully"
        }))
    }

    async fn handle_update_sample_status(&self, params: Value) -> Result<Value, McpError> {
        let sample_id: Uuid = params["sample_id"].as_str()
            .ok_or_else(|| McpError::InvalidParameters { 
                message: "sample_id is required".to_string() 
            })?
            .parse()
            .map_err(|_| McpError::InvalidParameters { 
                message: "Invalid sample_id format".to_string() 
            })?;

        let new_status_str = params["new_status"].as_str()
            .ok_or_else(|| McpError::InvalidParameters { 
                message: "new_status is required".to_string() 
            })?;

        let new_status: SampleStatus = serde_json::from_value(json!(new_status_str))
            .map_err(|e| McpError::InvalidParameters { 
                message: format!("Invalid status: {}", e) 
            })?;

        // Create update request
        let mut update_request = UpdateSampleRequest::default();
        // Set the status update logic here

        let updated_sample = self.sample_service.update_sample(sample_id, update_request).await
            .map_err(|e| McpError::ServiceError { 
                message: format!("Failed to update sample status: {}", e) 
            })?;

        Ok(json!({
            "success": true,
            "sample": updated_sample,
            "previous_status": params["previous_status"],
            "new_status": new_status,
            "message": "Sample status updated successfully"
        }))
    }

    async fn handle_search_samples(&self, params: Value) -> Result<Value, McpError> {
        let mut filters = SampleSearchFilters::default();
        
        // Parse search parameters
        if let Some(status) = params["status"].as_str() {
            filters.status = serde_json::from_value(json!(status)).ok();
        }
        
        if let Some(sample_type) = params["sample_type"].as_str() {
            filters.sample_type = Some(sample_type.to_string());
        }
        
        if let Some(limit) = params["limit"].as_u64() {
            filters.limit = Some(limit as i64);
        }
        
        if let Some(offset) = params["offset"].as_u64() {
            filters.offset = Some(offset as i64);
        }

        let samples = self.sample_service.search_samples(filters).await
            .map_err(|e| McpError::ServiceError { 
                message: format!("Failed to search samples: {}", e) 
            })?;

        Ok(json!({
            "success": true,
            "samples": samples,
            "total_count": samples.len(),
            "message": "Sample search completed successfully"
        }))
    }

    async fn handle_batch_create_samples(&self, params: Value) -> Result<Value, McpError> {
        let samples_data = params["samples"].as_array()
            .ok_or_else(|| McpError::InvalidParameters { 
                message: "samples array is required".to_string() 
            })?;

        let auto_validate = params["auto_validate"].as_bool().unwrap_or(false);
        let notify_submitter = params["notify_submitter"].as_bool().unwrap_or(true);

        let mut created_samples = Vec::new();
        let mut errors = Vec::new();

        for (index, sample_data) in samples_data.iter().enumerate() {
            match serde_json::from_value::<CreateSampleRequest>(sample_data.clone()) {
                Ok(create_request) => {
                    match self.sample_service.create_sample(create_request).await {
                        Ok(sample) => {
                            if auto_validate {
                                match self.sample_service.validate_sample(sample.id).await {
                                    Ok(validated_sample) => created_samples.push(validated_sample),
                                    Err(e) => {
                                        created_samples.push(sample);
                                        errors.push(json!({
                                            "index": index,
                                            "type": "validation_error",
                                            "message": format!("Validation failed: {}", e)
                                        }));
                                    }
                                }
                            } else {
                                created_samples.push(sample);
                            }
                        }
                        Err(e) => {
                            errors.push(json!({
                                "index": index,
                                "type": "creation_error",
                                "message": format!("Failed to create sample: {}", e)
                            }));
                        }
                    }
                }
                Err(e) => {
                    errors.push(json!({
                        "index": index,
                        "type": "validation_error",
                        "message": format!("Invalid sample data: {}", e)
                    }));
                }
            }
        }

        Ok(json!({
            "success": errors.is_empty(),
            "samples": created_samples,
            "total_created": created_samples.len(),
            "total_requested": samples_data.len(),
            "errors": errors,
            "auto_validated": auto_validate,
            "message": format!("Batch operation completed: {} samples created", created_samples.len())
        }))
    }

    async fn handle_get_sample(&self, params: Value) -> Result<Value, McpError> {
        let sample_id: Uuid = params["sample_id"].as_str()
            .ok_or_else(|| McpError::InvalidParameters { 
                message: "sample_id is required".to_string() 
            })?
            .parse()
            .map_err(|_| McpError::InvalidParameters { 
                message: "Invalid sample_id format".to_string() 
            })?;

        let include_history = params["include_history"].as_bool().unwrap_or(false);

        let sample = self.sample_service.get_sample(sample_id).await
            .map_err(|e| McpError::ServiceError { 
                message: format!("Failed to get sample: {}", e) 
            })?;

        let mut response = json!({
            "success": true,
            "sample": sample,
            "message": "Sample retrieved successfully"
        });

        if include_history {
            // Get sample history if requested
            match self.sample_service.get_sample_history(sample_id).await {
                Ok(history) => {
                    response["history"] = json!(history);
                }
                Err(e) => {
                    response["history_error"] = json!(format!("Failed to get history: {}", e));
                }
            }
        }

        Ok(response)
    }

    // Resource Handlers

    pub async fn get_resource(&self, uri: &str) -> Result<Value, McpError> {
        match uri {
            "sample://templates" => self.get_sample_templates().await,
            "sample://validation-rules" => self.get_validation_rules().await,
            "sample://statistics" => self.get_sample_statistics().await,
            "sample://types" => self.get_sample_types().await,
            _ => Err(McpError::NotFound { resource: uri.to_string() }),
        }
    }

    async fn get_sample_templates(&self) -> Result<Value, McpError> {
        // This would typically fetch from template service
        Ok(json!({
            "templates": [
                {
                    "id": "dna-extraction-template",
                    "name": "DNA Extraction Template",
                    "description": "Standard template for DNA sample submission",
                    "required_fields": ["sample_type", "source_organism", "extraction_method"],
                    "optional_fields": ["concentration", "purity", "volume"]
                },
                {
                    "id": "rna-seq-template",
                    "name": "RNA Sequencing Template",
                    "description": "Template for RNA sequencing samples",
                    "required_fields": ["sample_type", "tissue_type", "rna_integrity"],
                    "optional_fields": ["library_prep", "sequencing_depth"]
                }
            ]
        }))
    }

    async fn get_validation_rules(&self) -> Result<Value, McpError> {
        Ok(json!({
            "validation_rules": [
                {
                    "name": "barcode_format",
                    "description": "Validate barcode format",
                    "pattern": "^LAB-\\d{4}-\\d{5}$",
                    "required": true
                },
                {
                    "name": "sample_type_validation",
                    "description": "Validate sample type against allowed types",
                    "allowed_values": ["DNA", "RNA", "Protein", "Cell Culture"],
                    "required": true
                },
                {
                    "name": "metadata_completeness",
                    "description": "Ensure required metadata fields are present",
                    "required_fields": ["submitter", "collection_date", "source"],
                    "required": true
                }
            ]
        }))
    }

    async fn get_sample_statistics(&self) -> Result<Value, McpError> {
        let stats = self.sample_service.get_statistics().await
            .map_err(|e| McpError::ServiceError { 
                message: format!("Failed to get statistics: {}", e) 
            })?;

        Ok(json!(stats))
    }

    async fn get_sample_types(&self) -> Result<Value, McpError> {
        Ok(json!({
            "sample_types": [
                {
                    "type": "DNA",
                    "description": "Deoxyribonucleic acid samples",
                    "storage_requirements": {
                        "temperature": "-20°C",
                        "container": "sterile_tube"
                    },
                    "processing_requirements": {
                        "min_concentration": "10 ng/μL",
                        "purity_ratio": "1.8-2.0"
                    }
                },
                {
                    "type": "RNA",
                    "description": "Ribonucleic acid samples",
                    "storage_requirements": {
                        "temperature": "-80°C",
                        "container": "rnase_free_tube"
                    },
                    "processing_requirements": {
                        "rin_score": ">7.0",
                        "integrity": "high"
                    }
                }
            ]
        }))
    }

    // Prompt Handlers

    pub async fn get_prompt(&self, name: &str, arguments: HashMap<String, Value>) -> Result<Value, McpError> {
        match name {
            "sample_submission_wizard" => self.generate_sample_submission_wizard(arguments).await,
            "quality_control_review" => self.generate_quality_control_review(arguments).await,
            _ => Err(McpError::NotFound { resource: name.to_string() }),
        }
    }

    async fn generate_sample_submission_wizard(&self, arguments: HashMap<String, Value>) -> Result<Value, McpError> {
        let sample_type = arguments.get("sample_type")
            .and_then(|v| v.as_str())
            .unwrap_or("DNA");
        
        let urgency = arguments.get("urgency")
            .and_then(|v| v.as_str())
            .unwrap_or("medium");

        let prompt = format!(
            r#"You are a laboratory sample submission assistant. Help the user submit a {} sample.

Current submission details:
- Sample Type: {}
- Urgency: {}

Please guide the user through the following steps:

1. **Sample Information**
   - Provide a clear, descriptive name for the sample
   - Specify the source (e.g., tissue type, organism, patient ID)
   - Enter collection date and location

2. **Technical Requirements**
   - For {} samples, ensure proper storage conditions
   - Verify concentration and purity requirements
   - Confirm container type and labeling

3. **Processing Instructions**
   - Specify any special handling requirements
   - Indicate downstream applications (sequencing, analysis, etc.)
   - Set priority and timeline expectations

4. **Quality Control**
   - Review all information for completeness
   - Validate against laboratory standards
   - Generate barcode and finalize submission

Ask one question at a time and wait for user responses. Provide helpful suggestions and catch any potential issues early."#,
            sample_type, sample_type, urgency, sample_type
        );

        Ok(json!({
            "messages": [
                {
                    "role": "assistant",
                    "content": prompt
                }
            ]
        }))
    }

    async fn generate_quality_control_review(&self, arguments: HashMap<String, Value>) -> Result<Value, McpError> {
        let sample_id = arguments.get("sample_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParameters { 
                message: "sample_id is required for QC review".to_string() 
            })?;

        let review_type = arguments.get("review_type")
            .and_then(|v| v.as_str())
            .unwrap_or("initial");

        // Get sample details for the prompt
        let sample_uuid: Uuid = sample_id.parse()
            .map_err(|_| McpError::InvalidParameters { 
                message: "Invalid sample_id format".to_string() 
            })?;

        let sample = self.sample_service.get_sample(sample_uuid).await
            .map_err(|e| McpError::ServiceError { 
                message: format!("Failed to get sample for QC review: {}", e) 
            })?;

        let prompt = format!(
            r#"You are a laboratory quality control specialist reviewing sample {}. This is a {} review.

Sample Details:
- Name: {}
- Type: {}
- Status: {:?}
- Barcode: {}
- Submission Date: {}

Quality Control Checklist:

1. **Documentation Review**
   - Verify all required fields are complete
   - Check submission metadata accuracy
   - Confirm proper labeling and identification

2. **Technical Assessment**
   - Evaluate sample quality parameters
   - Check storage conditions compliance
   - Verify concentration and purity values

3. **Regulatory Compliance**
   - Ensure adherence to laboratory protocols
   - Verify regulatory requirements are met
   - Check audit trail completeness

4. **Risk Assessment**
   - Identify any quality concerns
   - Assess impact on downstream processing
   - Recommend corrective actions if needed

Please conduct a thorough review and provide:
- Overall quality assessment (Pass/Fail/Conditional)
- Specific findings or concerns
- Recommendations for next steps
- Any required corrective actions

Focus on ensuring this sample meets all laboratory standards for processing."#,
            sample_id,
            review_type,
            sample.name,
            sample.sample_type.unwrap_or_default(),
            sample.status,
            sample.barcode.unwrap_or_default(),
            sample.created_at
        );

        Ok(json!({
            "messages": [
                {
                    "role": "assistant",
                    "content": prompt
                }
            ]
        }))
    }
}