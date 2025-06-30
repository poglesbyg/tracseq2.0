use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

/// Quality Control Workflow Status
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "qc_workflow_status", rename_all = "lowercase")]
pub enum QcWorkflowStatus {
    Draft,
    Active,
    Executing,
    Completed,
    Failed,
    Cancelled,
    Suspended,
}

/// Quality Control Workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QcWorkflow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub workflow_type: QcWorkflowType,
    pub status: QcWorkflowStatus,
    pub steps: Vec<QcWorkflowStep>,
    pub triggers: Vec<QcTrigger>,
    pub quality_thresholds: HashMap<String, QualityThreshold>,
    pub compliance_requirements: Vec<ComplianceRequirement>,
    pub version: i32,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_executed: Option<DateTime<Utc>>,
}

/// Quality Control Workflow Type
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "qc_workflow_type", rename_all = "lowercase")]
pub enum QcWorkflowType {
    SampleValidation,
    SequencingQc,
    DataQuality,
    ComplianceCheck,
    PerformanceAnalysis,
    LibraryQc,
    SpreadsheetValidation,
    Custom,
}

/// Quality Control Workflow Step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QcWorkflowStep {
    pub step_id: String,
    pub name: String,
    pub description: Option<String>,
    pub step_type: QcStepType,
    pub order: i32,
    pub required: bool,
    pub parameters: HashMap<String, serde_json::Value>,
    pub expected_duration_minutes: Option<i32>,
    pub dependencies: Vec<String>,
    pub quality_checks: Vec<QualityCheck>,
    pub automation_config: Option<AutomationConfig>,
}

/// Quality Control Step Type
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "qc_step_type", rename_all = "lowercase")]
pub enum QcStepType {
    Validation,
    Measurement,
    Analysis,
    Comparison,
    Approval,
    Documentation,
    Notification,
    Integration,
    CustomScript,
}

/// Quality Control Trigger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QcTrigger {
    pub trigger_id: String,
    pub name: String,
    pub trigger_type: QcTriggerType,
    pub conditions: Vec<TriggerCondition>,
    pub actions: Vec<TriggerAction>,
    pub enabled: bool,
}

/// Quality Control Trigger Type
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "qc_trigger_type", rename_all = "lowercase")]
pub enum QcTriggerType {
    Scheduled,
    EventBased,
    ThresholdBased,
    StatusChange,
    DataAvailable,
    Manual,
}

/// Quality Threshold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThreshold {
    pub metric_name: String,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub target_value: Option<f64>,
    pub tolerance: Option<f64>,
    pub severity: ThresholdSeverity,
    pub unit: Option<String>,
    pub description: Option<String>,
}

/// Threshold Severity
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "threshold_severity", rename_all = "lowercase")]
pub enum ThresholdSeverity {
    Critical,
    Warning,
    Info,
    Advisory,
}

/// Quality Check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityCheck {
    pub check_id: String,
    pub name: String,
    pub check_type: QualityCheckType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub pass_criteria: Vec<PassCriteria>,
    pub automated: bool,
    pub required_role: Option<String>,
}

/// Quality Check Type
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "quality_check_type", rename_all = "lowercase")]
pub enum QualityCheckType {
    NumericRange,
    StringMatch,
    RegexMatch,
    FileExists,
    DatabaseQuery,
    ServiceCall,
    Statistical,
    Visual,
    Custom,
}

/// Pass Criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassCriteria {
    pub field: String,
    pub operator: ComparisonOperator,
    pub value: serde_json::Value,
    pub logic: LogicOperator,
}

/// Comparison Operator
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "comparison_operator", rename_all = "lowercase")]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Contains,
    StartsWith,
    EndsWith,
    Regex,
    In,
    NotIn,
}

/// Logic Operator
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "logic_operator", rename_all = "lowercase")]
pub enum LogicOperator {
    And,
    Or,
    Not,
}

/// Quality Metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetric {
    pub id: Uuid,
    pub name: String,
    pub metric_type: QualityMetricType,
    pub value: f64,
    pub unit: Option<String>,
    pub sample_id: Option<Uuid>,
    pub workflow_id: Option<Uuid>,
    pub step_id: Option<String>,
    pub threshold_id: Option<Uuid>,
    pub status: MetricStatus,
    pub measured_at: DateTime<Utc>,
    pub measured_by: Option<Uuid>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Quality Metric Type
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "quality_metric_type", rename_all = "lowercase")]
pub enum QualityMetricType {
    Concentration,
    Purity,
    Integrity,
    Yield,
    Coverage,
    Quality,
    Error,
    Performance,
    Compliance,
    Custom,
}

/// Metric Status
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "metric_status", rename_all = "lowercase")]
pub enum MetricStatus {
    Pass,
    Fail,
    Warning,
    Pending,
    NotApplicable,
}

/// Compliance Rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRule {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub rule_type: ComplianceRuleType,
    pub standard: String,
    pub section: Option<String>,
    pub severity: ComplianceSeverity,
    pub conditions: Vec<ComplianceCondition>,
    pub actions: Vec<ComplianceAction>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compliance Rule Type
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "compliance_rule_type", rename_all = "lowercase")]
pub enum ComplianceRuleType {
    Validation,
    Documentation,
    Approval,
    Retention,
    Access,
    Audit,
    Reporting,
    Custom,
}

/// Compliance Severity
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "compliance_severity", rename_all = "lowercase")]
pub enum ComplianceSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Trigger Condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCondition {
    pub field: String,
    pub operator: ComparisonOperator,
    pub value: serde_json::Value,
    pub logic: LogicOperator,
}

/// Trigger Action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerAction {
    pub action_type: TriggerActionType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub delay_seconds: Option<u64>,
}

/// Trigger Action Type
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "trigger_action_type", rename_all = "lowercase")]
pub enum TriggerActionType {
    StartWorkflow,
    SendNotification,
    UpdateStatus,
    CreateTask,
    LogEvent,
    CallService,
    Custom,
}

/// Compliance Condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCondition {
    pub field: String,
    pub operator: ComparisonOperator,
    pub value: serde_json::Value,
    pub required: bool,
}

/// Compliance Action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAction {
    pub action_type: ComplianceActionType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub required: bool,
}

/// Compliance Action Type
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "compliance_action_type", rename_all = "lowercase")]
pub enum ComplianceActionType {
    Require,
    Validate,
    Document,
    Approve,
    Archive,
    Notify,
    Audit,
    Custom,
}

/// Compliance Requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirement {
    pub requirement_id: String,
    pub name: String,
    pub standard: String,
    pub section: Option<String>,
    pub description: Option<String>,
    pub required: bool,
    pub automated: bool,
}

/// Automation Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationConfig {
    pub enabled: bool,
    pub script_path: Option<String>,
    pub api_endpoint: Option<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub timeout_seconds: Option<u64>,
    pub retry_count: Option<u32>,
}

/// Quality Analysis Report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAnalysisReport {
    pub id: Uuid,
    pub report_type: ReportType,
    pub title: String,
    pub description: Option<String>,
    pub time_period: TimePeriod,
    pub metrics: Vec<MetricSummary>,
    pub trends: Vec<TrendAnalysis>,
    pub recommendations: Vec<String>,
    pub generated_at: DateTime<Utc>,
    pub generated_by: Uuid,
}

/// Report Type
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "report_type", rename_all = "lowercase")]
pub enum ReportType {
    Quality,
    Compliance,
    Performance,
    Trend,
    Summary,
    Custom,
}

/// Time Period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePeriod {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub period_type: PeriodType,
}

/// Period Type
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "period_type", rename_all = "lowercase")]
pub enum PeriodType {
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
    Custom,
}

/// Metric Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSummary {
    pub metric_name: String,
    pub count: i64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub pass_rate: f64,
}

/// Trend Analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub metric_name: String,
    pub trend_direction: TrendDirection,
    pub trend_strength: f64,
    pub confidence: f64,
    pub forecast: Option<Vec<f64>>,
}

/// Trend Direction
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "trend_direction", rename_all = "lowercase")]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
    Unknown,
}

// Request/Response DTOs

/// Create QC Workflow Request
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateQcWorkflowRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub description: Option<String>,
    pub workflow_type: QcWorkflowType,
    pub steps: Vec<QcWorkflowStep>,
    pub triggers: Vec<QcTrigger>,
    pub quality_thresholds: HashMap<String, QualityThreshold>,
    pub compliance_requirements: Vec<ComplianceRequirement>,
}

/// Update QC Workflow Request
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateQcWorkflowRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<QcWorkflowStatus>,
    pub steps: Option<Vec<QcWorkflowStep>>,
    pub triggers: Option<Vec<QcTrigger>>,
    pub quality_thresholds: Option<HashMap<String, QualityThreshold>>,
    pub compliance_requirements: Option<Vec<ComplianceRequirement>>,
}

/// Create Quality Metric Request
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateQualityMetricRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub metric_type: QualityMetricType,
    pub value: f64,
    pub unit: Option<String>,
    pub sample_id: Option<Uuid>,
    pub workflow_id: Option<Uuid>,
    pub step_id: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Create Compliance Rule Request
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateComplianceRuleRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub description: Option<String>,
    pub rule_type: ComplianceRuleType,
    #[validate(length(min = 1, max = 100))]
    pub standard: String,
    pub section: Option<String>,
    pub severity: ComplianceSeverity,
    pub conditions: Vec<ComplianceCondition>,
    pub actions: Vec<ComplianceAction>,
}

/// Quality Analysis Request
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct QualityAnalysisRequest {
    pub report_type: ReportType,
    pub time_period: TimePeriod,
    pub metric_types: Option<Vec<QualityMetricType>>,
    pub sample_ids: Option<Vec<Uuid>>,
    pub workflow_ids: Option<Vec<Uuid>>,
    pub include_trends: bool,
    pub include_recommendations: bool,
}

/// Workflow Execution Request
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct WorkflowExecutionRequest {
    pub workflow_id: Uuid,
    pub target_id: Uuid, // Sample ID, Sequencing Job ID, etc.
    pub target_type: String,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
    pub priority: Option<ExecutionPriority>,
}

/// Execution Priority
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "execution_priority", rename_all = "lowercase")]
pub enum ExecutionPriority {
    Critical,
    High,
    Normal,
    Low,
}
