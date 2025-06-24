//! Laboratory Workflow Templates
//!
//! This module provides pre-defined workflow templates for common laboratory operations,
//! including AI-generated workflows and standard protocols.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::workflows::laboratory::WorkflowStepDefinition;
use crate::workflows::{RiskLevel, WorkflowPriority};

/// A laboratory workflow template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaboratoryWorkflowTemplate {
    pub template_id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStepDefinition>,
    pub estimated_duration_minutes: u32,
    pub required_equipment: Vec<String>,
    pub quality_checkpoints: Vec<String>,
    pub ai_generated: bool,
    pub confidence_score: f64,
}

/// Template metadata for workflow management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    pub template_id: String,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub approval_date: Option<DateTime<Utc>>,
    pub usage_count: u64,
    pub success_rate: f64,
    pub average_duration_minutes: Option<u32>,
    pub tags: Vec<String>,
    pub compliance_requirements: Vec<String>,
}

/// Workflow template category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateCategory {
    SampleProcessing,
    QualityControl,
    DataAnalysis,
    Compliance,
    Maintenance,
    Calibration,
    Research,
    CustomProtocol,
}

/// Template validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub estimated_duration: Option<u32>,
    pub resource_requirements: HashMap<String, String>,
}

/// Laboratory workflow template manager
pub struct TemplateManager {
    templates: HashMap<String, LaboratoryWorkflowTemplate>,
    metadata: HashMap<String, TemplateMetadata>,
}

impl TemplateManager {
    /// Create a new template manager with default templates
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
            metadata: HashMap::new(),
        };

        manager.load_default_templates();
        manager
    }

    /// Load default laboratory workflow templates
    fn load_default_templates(&mut self) {
        // Standard DNA Extraction Template
        let dna_extraction = LaboratoryWorkflowTemplate {
            template_id: "dna_extraction_standard".to_string(),
            name: "Standard DNA Extraction".to_string(),
            description: "Standardized DNA extraction workflow for various sample types"
                .to_string(),
            steps: vec![
                WorkflowStepDefinition {
                    step_name: "sample_preparation".to_string(),
                    description: "Prepare samples for DNA extraction".to_string(),
                    step_type: crate::workflows::laboratory::WorkflowStepType::SamplePreparation,
                    required_inputs: vec!["sample_id".to_string(), "sample_type".to_string()],
                    expected_outputs: vec!["prepared_sample".to_string()],
                    estimated_duration_minutes: 30,
                    required_equipment: vec!["pipettes".to_string(), "tubes".to_string()],
                    quality_checks: vec!["volume_check".to_string()],
                    ai_validation_enabled: true,
                    human_approval_required: false,
                    risk_level: RiskLevel::Low,
                },
                WorkflowStepDefinition {
                    step_name: "dna_extraction".to_string(),
                    description: "Extract DNA using kit protocol".to_string(),
                    step_type: crate::workflows::laboratory::WorkflowStepType::Processing,
                    required_inputs: vec!["prepared_sample".to_string()],
                    expected_outputs: vec!["extracted_dna".to_string()],
                    estimated_duration_minutes: 90,
                    required_equipment: vec![
                        "extraction_kit".to_string(),
                        "centrifuge".to_string(),
                    ],
                    quality_checks: vec!["purity_check".to_string(), "yield_check".to_string()],
                    ai_validation_enabled: true,
                    human_approval_required: false,
                    risk_level: RiskLevel::Medium,
                },
                WorkflowStepDefinition {
                    step_name: "quality_control".to_string(),
                    description: "Quality control analysis of extracted DNA".to_string(),
                    step_type: crate::workflows::laboratory::WorkflowStepType::QualityControl,
                    required_inputs: vec!["extracted_dna".to_string()],
                    expected_outputs: vec!["qc_report".to_string()],
                    estimated_duration_minutes: 45,
                    required_equipment: vec!["spectrophotometer".to_string()],
                    quality_checks: vec![
                        "concentration_check".to_string(),
                        "purity_check".to_string(),
                    ],
                    ai_validation_enabled: true,
                    human_approval_required: true,
                    risk_level: RiskLevel::Low,
                },
            ],
            estimated_duration_minutes: 165,
            required_equipment: vec![
                "pipettes".to_string(),
                "tubes".to_string(),
                "extraction_kit".to_string(),
                "centrifuge".to_string(),
                "spectrophotometer".to_string(),
            ],
            quality_checkpoints: vec![
                "volume_check".to_string(),
                "purity_check".to_string(),
                "yield_check".to_string(),
                "concentration_check".to_string(),
            ],
            ai_generated: false,
            confidence_score: 1.0,
        };

        self.add_template(dna_extraction);

        // RNA Extraction Template
        let rna_extraction = LaboratoryWorkflowTemplate {
            template_id: "rna_extraction_standard".to_string(),
            name: "Standard RNA Extraction".to_string(),
            description: "Standardized RNA extraction workflow with RNase protection".to_string(),
            steps: vec![
                WorkflowStepDefinition {
                    step_name: "rnase_treatment".to_string(),
                    description: "RNase decontamination of work area".to_string(),
                    step_type: crate::workflows::laboratory::WorkflowStepType::EquipmentSetup,
                    required_inputs: vec!["workspace".to_string()],
                    expected_outputs: vec!["clean_workspace".to_string()],
                    estimated_duration_minutes: 15,
                    required_equipment: vec!["rnase_away".to_string()],
                    quality_checks: vec!["surface_check".to_string()],
                    ai_validation_enabled: false,
                    human_approval_required: true,
                    risk_level: RiskLevel::Medium,
                },
                WorkflowStepDefinition {
                    step_name: "sample_preparation".to_string(),
                    description: "Prepare samples for RNA extraction".to_string(),
                    step_type: crate::workflows::laboratory::WorkflowStepType::SamplePreparation,
                    required_inputs: vec!["sample_id".to_string(), "clean_workspace".to_string()],
                    expected_outputs: vec!["prepared_sample".to_string()],
                    estimated_duration_minutes: 30,
                    required_equipment: vec![
                        "pipettes".to_string(),
                        "rnase_free_tubes".to_string(),
                    ],
                    quality_checks: vec!["volume_check".to_string()],
                    ai_validation_enabled: true,
                    human_approval_required: false,
                    risk_level: RiskLevel::Medium,
                },
                WorkflowStepDefinition {
                    step_name: "rna_extraction".to_string(),
                    description: "Extract RNA using specialized protocol".to_string(),
                    step_type: crate::workflows::laboratory::WorkflowStepType::Processing,
                    required_inputs: vec!["prepared_sample".to_string()],
                    expected_outputs: vec!["extracted_rna".to_string()],
                    estimated_duration_minutes: 120,
                    required_equipment: vec![
                        "rna_extraction_kit".to_string(),
                        "centrifuge".to_string(),
                    ],
                    quality_checks: vec!["integrity_check".to_string(), "yield_check".to_string()],
                    ai_validation_enabled: true,
                    human_approval_required: false,
                    risk_level: RiskLevel::High,
                },
            ],
            estimated_duration_minutes: 165,
            required_equipment: vec![
                "rnase_away".to_string(),
                "pipettes".to_string(),
                "rnase_free_tubes".to_string(),
                "rna_extraction_kit".to_string(),
                "centrifuge".to_string(),
            ],
            quality_checkpoints: vec![
                "surface_check".to_string(),
                "volume_check".to_string(),
                "integrity_check".to_string(),
                "yield_check".to_string(),
            ],
            ai_generated: false,
            confidence_score: 1.0,
        };

        self.add_template(rna_extraction);

        // Sample Quality Control Template
        let sample_qc = LaboratoryWorkflowTemplate {
            template_id: "sample_qc_comprehensive".to_string(),
            name: "Comprehensive Sample QC".to_string(),
            description: "Complete quality control workflow for incoming samples".to_string(),
            steps: vec![
                WorkflowStepDefinition {
                    step_name: "visual_inspection".to_string(),
                    description: "Visual inspection of sample integrity".to_string(),
                    step_type: crate::workflows::laboratory::WorkflowStepType::QualityControl,
                    required_inputs: vec!["sample_id".to_string()],
                    expected_outputs: vec!["visual_assessment".to_string()],
                    estimated_duration_minutes: 10,
                    required_equipment: vec!["microscope".to_string()],
                    quality_checks: vec!["integrity_check".to_string()],
                    ai_validation_enabled: true,
                    human_approval_required: true,
                    risk_level: RiskLevel::Low,
                },
                WorkflowStepDefinition {
                    step_name: "contamination_screening".to_string(),
                    description: "Screen for potential contamination".to_string(),
                    step_type: crate::workflows::laboratory::WorkflowStepType::QualityControl,
                    required_inputs: vec!["sample_id".to_string()],
                    expected_outputs: vec!["contamination_report".to_string()],
                    estimated_duration_minutes: 60,
                    required_equipment: vec!["pcr_machine".to_string(), "reagents".to_string()],
                    quality_checks: vec!["contamination_check".to_string()],
                    ai_validation_enabled: true,
                    human_approval_required: false,
                    risk_level: RiskLevel::Medium,
                },
                WorkflowStepDefinition {
                    step_name: "documentation_review".to_string(),
                    description: "Review and validate sample documentation".to_string(),
                    step_type: crate::workflows::laboratory::WorkflowStepType::Documentation,
                    required_inputs: vec!["sample_metadata".to_string()],
                    expected_outputs: vec!["documentation_report".to_string()],
                    estimated_duration_minutes: 15,
                    required_equipment: vec![],
                    quality_checks: vec!["completeness_check".to_string()],
                    ai_validation_enabled: true,
                    human_approval_required: true,
                    risk_level: RiskLevel::Low,
                },
            ],
            estimated_duration_minutes: 85,
            required_equipment: vec![
                "microscope".to_string(),
                "pcr_machine".to_string(),
                "reagents".to_string(),
            ],
            quality_checkpoints: vec![
                "integrity_check".to_string(),
                "contamination_check".to_string(),
                "completeness_check".to_string(),
            ],
            ai_generated: false,
            confidence_score: 1.0,
        };

        self.add_template(sample_qc);
    }

    /// Add a new template
    pub fn add_template(&mut self, template: LaboratoryWorkflowTemplate) {
        let metadata = TemplateMetadata {
            template_id: template.template_id.clone(),
            version: "1.0".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: None,
            approved_by: None,
            approval_date: None,
            usage_count: 0,
            success_rate: 1.0,
            average_duration_minutes: Some(template.estimated_duration_minutes),
            tags: vec![],
            compliance_requirements: vec![],
        };

        self.metadata.insert(template.template_id.clone(), metadata);
        self.templates
            .insert(template.template_id.clone(), template);
    }

    /// Get all available templates
    pub fn get_all_templates(&self) -> Vec<&LaboratoryWorkflowTemplate> {
        self.templates.values().collect()
    }

    /// Get a specific template by ID
    pub fn get_template(&self, template_id: &str) -> Option<&LaboratoryWorkflowTemplate> {
        self.templates.get(template_id)
    }

    /// Get template metadata
    pub fn get_template_metadata(&self, template_id: &str) -> Option<&TemplateMetadata> {
        self.metadata.get(template_id)
    }

    /// Validate a template
    pub fn validate_template(
        &self,
        template: &LaboratoryWorkflowTemplate,
    ) -> TemplateValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate template structure
        if template.name.is_empty() {
            errors.push("Template name cannot be empty".to_string());
        }

        if template.steps.is_empty() {
            errors.push("Template must contain at least one step".to_string());
        }

        // Validate step dependencies
        let mut available_outputs = Vec::new();
        for step in &template.steps {
            for required_input in &step.required_inputs {
                if !available_outputs.contains(required_input)
                    && !["sample_id", "sample_type", "workspace", "sample_metadata"]
                        .contains(&required_input.as_str())
                {
                    warnings.push(format!(
                        "Step '{}' requires input '{}' that may not be available",
                        step.step_name, required_input
                    ));
                }
            }
            available_outputs.extend(step.expected_outputs.clone());
        }

        // Calculate estimated duration
        let estimated_duration = template
            .steps
            .iter()
            .map(|step| step.estimated_duration_minutes)
            .sum();

        if estimated_duration != template.estimated_duration_minutes {
            warnings.push(format!(
                "Template duration ({} min) doesn't match sum of step durations ({} min)",
                template.estimated_duration_minutes, estimated_duration
            ));
        }

        // Collect resource requirements
        let mut resource_requirements = HashMap::new();
        let mut all_equipment = Vec::new();
        for step in &template.steps {
            all_equipment.extend(step.required_equipment.clone());
        }
        all_equipment.sort();
        all_equipment.dedup();
        resource_requirements.insert("equipment".to_string(), all_equipment.join(", "));

        TemplateValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            estimated_duration: Some(estimated_duration),
            resource_requirements,
        }
    }

    /// Update template usage statistics
    pub fn update_usage_stats(&mut self, template_id: &str, success: bool, duration_minutes: u32) {
        if let Some(metadata) = self.metadata.get_mut(template_id) {
            metadata.usage_count += 1;

            // Update success rate (simple exponential moving average)
            let weight = 0.1;
            if success {
                metadata.success_rate = metadata.success_rate * (1.0 - weight) + weight;
            } else {
                metadata.success_rate = metadata.success_rate * (1.0 - weight);
            }

            // Update average duration
            if let Some(avg_duration) = metadata.average_duration_minutes {
                metadata.average_duration_minutes = Some(
                    (((avg_duration as f64 * (metadata.usage_count - 1) as f64)
                        + duration_minutes as f64)
                        / metadata.usage_count as f64) as u32
                );
            } else {
                metadata.average_duration_minutes = Some(duration_minutes);
            }

            metadata.updated_at = chrono::Utc::now();
        }
    }

    /// Get templates by category
    pub fn get_templates_by_category(
        &self,
        category: TemplateCategory,
    ) -> Vec<&LaboratoryWorkflowTemplate> {
        // This is a simple implementation - in a real system you'd store category with template
        match category {
            TemplateCategory::SampleProcessing => self
                .templates
                .values()
                .filter(|t| {
                    t.template_id.contains("extraction") || t.template_id.contains("processing")
                })
                .collect(),
            TemplateCategory::QualityControl => self
                .templates
                .values()
                .filter(|t| t.template_id.contains("qc") || t.name.contains("QC"))
                .collect(),
            _ => Vec::new(),
        }
    }
}

impl Default for TemplateManager {
    fn default() -> Self {
        Self::new()
    }
}
