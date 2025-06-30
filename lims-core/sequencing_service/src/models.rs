use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ================================
// Core Sequencing Models
// ================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SequencingJob {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: JobStatus,
    pub priority: Priority,
    pub platform_id: String,
    pub workflow_id: String,
    pub sample_sheet_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub created_by: Uuid,
    pub assigned_to: Option<Uuid>,
    pub estimated_start: Option<DateTime<Utc>>,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub actual_start: Option<DateTime<Utc>>,
    pub actual_completion: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Alias fields for compatibility
    pub platform: Option<String>,
    pub job_name: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "job_status", rename_all = "snake_case")]
pub enum JobStatus {
    Draft,
    Submitted,
    Validated,
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
    OnHold,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Draft => write!(f, "draft"),
            JobStatus::Submitted => write!(f, "submitted"),
            JobStatus::Validated => write!(f, "validated"),
            JobStatus::Queued => write!(f, "queued"),
            JobStatus::Running => write!(f, "running"),
            JobStatus::Completed => write!(f, "completed"),
            JobStatus::Failed => write!(f, "failed"),
            JobStatus::Cancelled => write!(f, "cancelled"),
            JobStatus::OnHold => write!(f, "on_hold"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "priority", rename_all = "snake_case")]
pub enum Priority {
    Low,
    Normal,
    High,
    Urgent,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SequencingRun {
    pub id: Uuid,
    pub name: String,
    pub platform_id: String,
    pub chemistry: String,
    pub flowcell_id: String,
    pub status: RunStatus,
    pub cluster_generation_kit: Option<String>,
    pub sequencing_kit: Option<String>,
    pub read_structure: String,
    pub sample_count: i32,
    pub estimated_yield_gb: Option<f64>,
    pub actual_yield_gb: Option<f64>,
    pub quality_score_mean: Option<f64>,
    pub percent_pf: Option<f64>,
    pub percent_q30: Option<f64>,
    pub error_rate: Option<f64>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub data_path: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "run_status", rename_all = "snake_case")]
pub enum RunStatus {
    Preparing,
    Ready,
    Running,
    Analyzing,
    Completed,
    Failed,
    Aborted,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SampleSheet {
    pub id: Uuid,
    pub name: String,
    pub platform_id: String,
    pub version: String,
    pub status: SampleSheetStatus,
    pub sample_count: i32,
    pub file_path: Option<String>,
    pub validation_errors: Option<serde_json::Value>,
    pub metadata: serde_json::Value,
    pub samples_data: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "sample_sheet_status", rename_all = "snake_case")]
pub enum SampleSheetStatus {
    Draft,
    Validating,
    Validated,
    Valid,
    Invalid,
    InUse,
    Generated,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SampleSheetEntry {
    pub id: Uuid,
    pub sample_sheet_id: Uuid,
    pub sample_id: String,
    pub sample_name: String,
    pub sample_plate: Option<String>,
    pub sample_well: Option<String>,
    pub index1: String,
    pub index2: Option<String>,
    pub project: String,
    pub description: Option<String>,
}

// ================================
// Workflow Models
// ================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub platform_ids: Vec<String>,
    pub workflow_type: WorkflowType,
    pub steps: serde_json::Value,
    pub default_parameters: serde_json::Value,
    pub estimated_duration_hours: Option<f64>,
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "workflow_type", rename_all = "snake_case")]
pub enum WorkflowType {
    WholeGenome,
    Exome,
    TargetedSequencing,
    RnaSeq,
    ChipSeq,
    Bisulfite,
    SingleCell,
    Metagenomics,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkflowExecution {
    pub id: Uuid,
    pub job_id: Uuid,
    pub workflow_id: String,
    pub status: ExecutionStatus,
    pub current_step: Option<String>,
    pub parameters: serde_json::Value,
    pub outputs: serde_json::Value,
    pub logs: serde_json::Value,
    pub error_message: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "execution_status", rename_all = "snake_case")]
pub enum ExecutionStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Aborted,
    Retrying,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "step_status", rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    Retrying,
}

// ================================
// Analysis Models
// ================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnalysisPipeline {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub pipeline_id: PipelineType,
    pub container_image: Option<String>,
    pub command_template: String,
    pub input_types: Vec<String>,
    pub output_types: Vec<String>,
    pub resource_requirements: serde_json::Value,
    pub parameters_schema: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "pipeline_id", rename_all = "snake_case")]
pub enum PipelineType {
    QualityControl,
    Preprocessing,
    Alignment,
    VariantCalling,
    Expression,
    Annotation,
    Reporting,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnalysisJob {
    pub id: Uuid,
    pub sequencing_job_id: Uuid,
    pub pipeline_id: String,
    pub status: JobStatus,
    pub parameters: serde_json::Value,
    pub input_files: Vec<String>,
    pub output_files: Vec<String>,
    pub log_file: Option<String>,
    pub compute_resources_used: serde_json::Value,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Compatibility field
    pub pipeline_type: Option<String>,
}

/// Analysis status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "analysis_status", rename_all = "snake_case")]
pub enum AnalysisStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

// ================================
// Quality Control Models
// ================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QualityMetrics {
    pub id: Uuid,
    pub entity_type: QualityEntityType,
    pub entity_id: Uuid,
    pub metric_type: String,
    pub value: f64,
    pub threshold_min: Option<f64>,
    pub threshold_max: Option<f64>,
    pub status: QualityStatus,
    pub measured_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "quality_entity_type", rename_all = "snake_case")]
pub enum QualityEntityType {
    Job,
    Run,
    Sample,
    Lane,
    Analysis,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "quality_status", rename_all = "snake_case")]
pub enum QualityStatus {
    Pass,
    Warning,
    Fail,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QualityReport {
    pub id: Uuid,
    pub entity_type: QualityEntityType,
    pub entity_id: Uuid,
    pub report_type: String,
    pub overall_status: QualityStatus,
    pub metrics_summary: serde_json::Value,
    pub recommendations: Vec<String>,
    pub report_data: serde_json::Value,
    pub generated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// ================================
// Scheduling Models
// ================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScheduledJob {
    pub id: Uuid,
    pub sequencing_job_id: Uuid,
    pub platform_id: String,
    pub priority: Priority,
    pub estimated_duration_hours: f64,
    pub earliest_start: DateTime<Utc>,
    pub latest_start: Option<DateTime<Utc>>,
    pub scheduled_start: Option<DateTime<Utc>>,
    pub actual_start: Option<DateTime<Utc>>,
    pub status: ScheduleStatus,
    pub constraints: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "schedule_status", rename_all = "snake_case")]
pub enum ScheduleStatus {
    Pending,
    Scheduled,
    Running,
    Completed,
    Cancelled,
    Rescheduled,
}

// ================================
// Request/Response Models
// ================================

#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    pub name: String,
    pub description: Option<String>,
    pub priority: Priority,
    pub platform_id: String,
    pub workflow_id: String,
    pub sample_sheet_id: Option<Uuid>,
    pub estimated_start: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateJobRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub priority: Option<Priority>,
    pub assigned_to: Option<Uuid>,
    pub estimated_start: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRunRequest {
    pub name: String,
    pub platform_id: String,
    pub chemistry: String,
    pub flowcell_id: String,
    pub cluster_generation_kit: Option<String>,
    pub sequencing_kit: Option<String>,
    pub read_structure: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateSampleSheetRequest {
    pub name: String,
    pub platform_id: String,
    pub samples: Vec<SampleSheetSample>,
}

#[derive(Debug, Deserialize)]
pub struct SampleSheetSample {
    pub sample_id: String,
    pub sample_name: String,
    pub sample_plate: Option<String>,
    pub sample_well: Option<String>,
    pub index1: String,
    pub index2: Option<String>,
    pub project: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct JobResponse {
    pub id: Uuid,
    pub name: String,
    pub status: JobStatus,
    pub priority: Priority,
    pub platform_id: String,
    pub workflow_id: String,
    pub progress_percentage: Option<f64>,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct RunMetrics {
    pub run_id: Uuid,
    pub total_reads: Option<i64>,
    pub reads_pf: Option<i64>,
    pub yield_gb: Option<f64>,
    pub quality_score_mean: Option<f64>,
    pub percent_q30: Option<f64>,
    pub error_rate: Option<f64>,
    pub cluster_density: Option<f64>,
    pub percent_occupied: Option<f64>,
}

// ================================
// Validation and Helper Methods
// ================================

impl CreateJobRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Job name cannot be empty".to_string());
        }

        if self.platform_id.trim().is_empty() {
            return Err("Platform ID cannot be empty".to_string());
        }

        if self.workflow_id.trim().is_empty() {
            return Err("Workflow ID cannot be empty".to_string());
        }

        Ok(())
    }
}

impl CreateSampleSheetRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Sample sheet name cannot be empty".to_string());
        }

        if self.samples.is_empty() {
            return Err("Sample sheet must contain at least one sample".to_string());
        }

        // Validate each sample
        for (index, sample) in self.samples.iter().enumerate() {
            if sample.sample_id.trim().is_empty() {
                return Err(format!("Sample {} ID cannot be empty", index + 1));
            }
            if sample.index1.trim().is_empty() {
                return Err(format!("Sample {} must have index1", index + 1));
            }
        }

        // Check for duplicate sample IDs
        let mut sample_ids = std::collections::HashSet::new();
        for sample in &self.samples {
            if !sample_ids.insert(&sample.sample_id) {
                return Err(format!("Duplicate sample ID: {}", sample.sample_id));
            }
        }

        Ok(())
    }
}

impl JobStatus {
    pub fn can_transition_to(&self, new_status: &JobStatus) -> bool {
        use JobStatus::*;
        match (self, new_status) {
            (Draft, Submitted) => true,
            (Submitted, Validated | Cancelled) => true,
            (Validated, Queued | OnHold | Cancelled) => true,
            (Queued, Running | OnHold | Cancelled) => true,
            (Running, Completed | Failed | Cancelled) => true,
            (OnHold, Queued | Cancelled) => true,
            (Failed, Queued | Cancelled) => true,
            (_, _) => false,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
        )
    }

    pub fn is_active(&self) -> bool {
        matches!(self, JobStatus::Queued | JobStatus::Running)
    }
}

impl Priority {
    pub fn to_numeric(&self) -> u8 {
        match self {
            Priority::Low => 1,
            Priority::Normal => 2,
            Priority::High => 3,
            Priority::Urgent => 4,
            Priority::Critical => 5,
        }
    }
}

// ================================
// Export and Reporting Models
// ================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnalysisResult {
    pub id: Uuid,
    pub analysis_id: Uuid,
    pub result_type: String,
    pub result_data: serde_json::Value,
    pub file_path: Option<String>,
    pub status: String,
    pub quality_score: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BatchExport {
    pub id: Uuid,
    pub export_type: String,
    pub job_ids: serde_json::Value,
    pub format: String,
    pub status: String,
    pub file_size: Option<i64>,
    pub successful_count: Option<i32>,
    pub failed_count: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub export_options: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ComprehensiveReport {
    pub id: Uuid,
    pub report_type: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub platform_filter: Option<String>,
    pub status: String,
    pub file_size: Option<i64>,
    pub content: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExportLog {
    pub id: Uuid,
    pub export_type: String,
    pub job_id: Option<Uuid>,
    pub analysis_id: Option<Uuid>,
    pub format: String,
    pub file_size: i64,
    pub exported_at: DateTime<Utc>,
    pub exported_by: Option<String>,
    pub export_options: serde_json::Value,
}

// ================================
// Integration Models
// ================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IntegrationWebhook {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub event_types: serde_json::Value,
    pub is_active: bool,
    pub secret_key: Option<String>,
    pub timeout_seconds: i32,
    pub retry_attempts: i32,
    pub retry_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub response_code: Option<i32>,
    pub response_body: Option<String>,
    pub retry_count: i32,
    pub delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IntegrationLog {
    pub id: Uuid,
    pub integration_type: String,
    pub operation: String,
    pub status: String,
    pub request_data: serde_json::Value,
    pub response_data: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub duration_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LIMSSync {
    pub id: Uuid,
    pub sync_type: String,
    pub sync_direction: String,
    pub job_ids: serde_json::Value,
    pub lims_system: String,
    pub status: String,
    pub records_processed: Option<i32>,
    pub records_failed: Option<i32>,
    pub error_details: Option<serde_json::Value>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub initiated_by: Option<String>,
}

// ================================
// Quality Control Extended Models
// ================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QCThreshold {
    pub id: Uuid,
    pub metric_type: String,
    pub platform_id: String,
    pub entity_type: QualityEntityType,
    pub threshold_min: Option<f64>,
    pub threshold_max: Option<f64>,
    pub warning_min: Option<f64>,
    pub warning_max: Option<f64>,
    pub threshold_type: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QCEvaluation {
    pub id: Uuid,
    pub entity_type: QualityEntityType,
    pub entity_id: Uuid,
    pub evaluation_type: String,
    pub overall_status: QualityStatus,
    pub metric_results: serde_json::Value,
    pub recommendations: serde_json::Value,
    pub evaluated_at: DateTime<Utc>,
    pub evaluated_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QCReport {
    pub id: Uuid,
    pub entity_type: QualityEntityType,
    pub entity_id: Uuid,
    pub report_type: String,
    pub status: QualityStatus,
    pub summary: serde_json::Value,
    pub detailed_metrics: serde_json::Value,
    pub generated_at: DateTime<Utc>,
    pub file_path: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QCAlert {
    pub id: Uuid,
    pub entity_type: QualityEntityType,
    pub entity_id: Uuid,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold_value: f64,
    pub is_acknowledged: bool,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ================================
// Sample Sheet Extended Models
// ================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SampleSheetValidation {
    pub id: Uuid,
    pub sample_sheet_id: Uuid,
    pub validation_type: String,
    pub status: String,
    pub errors: serde_json::Value,
    pub warnings: serde_json::Value,
    pub validated_at: DateTime<Utc>,
    pub validated_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ================================
// Enhanced Workflow Models
// ================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SequencingWorkflow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub version: String,
    pub workflow_type: WorkflowType,
    pub platform_compatibility: serde_json::Value,
    pub steps: serde_json::Value,
    pub parameters: serde_json::Value,
    pub estimated_duration_minutes: Option<i32>,
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkflowStepExecution {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub step_name: String,
    pub step_order: i32,
    pub status: StepStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub outputs: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkflowTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: String,
    pub workflow_type: WorkflowType,
    pub platform_ids: serde_json::Value,
    pub template_data: serde_json::Value,
    pub default_parameters: serde_json::Value,
    pub is_system_template: bool,
    pub usage_count: i32,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ================================
// Type Aliases for Compatibility
// ================================

// Alias for JobPriority -> Priority
pub type JobPriority = Priority;
