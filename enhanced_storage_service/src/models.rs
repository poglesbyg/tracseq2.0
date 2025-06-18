use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

// ============================================================================
// Storage Location Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StorageLocation {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub location_type: String,
    pub temperature_zone: String,
    pub max_capacity: i32,
    pub current_capacity: i32,
    pub coordinates: Option<serde_json::Value>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateStorageLocationRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub description: Option<String>,
    #[validate(length(min = 1))]
    pub location_type: String,
    #[validate(length(min = 1))]
    pub temperature_zone: String,
    #[validate(range(min = 1))]
    pub max_capacity: i32,
    pub coordinates: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateStorageLocationRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub location_type: Option<String>,
    pub temperature_zone: Option<String>,
    pub max_capacity: Option<i32>,
    pub coordinates: Option<serde_json::Value>,
    pub status: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// ============================================================================
// Sample Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Sample {
    pub id: Uuid,
    pub barcode: String,
    pub sample_type: String,
    pub storage_location_id: Option<Uuid>,
    pub position: Option<serde_json::Value>,
    pub temperature_requirements: Option<String>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub chain_of_custody: serde_json::Value,
    pub stored_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct StoreSampleRequest {
    #[validate(length(min = 1))]
    pub barcode: String,
    #[validate(length(min = 1))]
    pub sample_type: String,
    pub storage_location_id: Uuid,
    pub position: Option<serde_json::Value>,
    pub temperature_requirements: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoveSampleRequest {
    pub new_location_id: Uuid,
    pub new_position: Option<serde_json::Value>,
    pub reason: String,
}

// ============================================================================
// IoT Sensor Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IoTSensor {
    pub id: Uuid,
    pub sensor_id: String,
    pub sensor_type: String,
    pub location_id: Option<Uuid>,
    pub status: String,
    pub last_reading: Option<serde_json::Value>,
    pub calibration_data: serde_json::Value,
    pub maintenance_schedule: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SensorData {
    pub id: Uuid,
    pub sensor_id: Uuid,
    pub reading_type: String,
    pub value: f64,
    pub unit: String,
    pub quality_score: f64,
    pub metadata: serde_json::Value,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorReading {
    pub sensor_id: String,
    pub readings: Vec<SensorReadingValue>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorReadingValue {
    pub reading_type: String,
    pub value: f64,
    pub unit: String,
    pub quality_score: Option<f64>,
}

// ============================================================================
// Alert Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Alert {
    pub id: Uuid,
    pub alert_type: String,
    pub severity: String,
    pub title: String,
    pub message: String,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub status: String,
    pub acknowledged_by: Option<Uuid>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAlertRequest {
    pub alert_type: String,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

// ============================================================================
// Analytics Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnalyticsModel {
    pub id: Uuid,
    pub model_name: String,
    pub model_type: String,
    pub version: String,
    pub model_data: serde_json::Value,
    pub performance_metrics: serde_json::Value,
    pub training_metadata: serde_json::Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Prediction {
    pub id: Uuid,
    pub model_id: Uuid,
    pub prediction_type: String,
    pub input_data: serde_json::Value,
    pub prediction_result: serde_json::Value,
    pub confidence_score: Option<f64>,
    pub prediction_horizon: Option<i32>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PredictionRequest {
    pub model_type: String,
    pub input_data: serde_json::Value,
    pub prediction_horizon: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CapacityPrediction {
    pub location_id: Uuid,
    pub predicted_capacity: f64,
    pub prediction_date: DateTime<Utc>,
    pub confidence: f64,
    pub factors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaintenancePrediction {
    pub equipment_id: String,
    pub predicted_failure_date: DateTime<Utc>,
    pub confidence: f64,
    pub recommended_action: String,
    pub priority: String,
}

// ============================================================================
// Blockchain Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BlockchainTransaction {
    pub id: Uuid,
    pub transaction_hash: String,
    pub block_number: Option<i64>,
    pub transaction_type: String,
    pub data_hash: String,
    pub previous_hash: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub signature: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockchainRecord {
    pub transaction_type: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub previous_hash: Option<String>,
}

// ============================================================================
// Automation Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AutomationTask {
    pub id: Uuid,
    pub task_type: String,
    pub priority: i32,
    pub status: String,
    pub input_parameters: serde_json::Value,
    pub output_results: Option<serde_json::Value>,
    pub assigned_robot_id: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAutomationTaskRequest {
    pub task_type: String,
    pub priority: Option<i32>,
    pub input_parameters: serde_json::Value,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RobotStatus {
    pub robot_id: String,
    pub status: String,
    pub current_task: Option<Uuid>,
    pub position: Option<serde_json::Value>,
    pub battery_level: Option<f64>,
    pub last_communication: DateTime<Utc>,
}

// ============================================================================
// Energy Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EnergyConsumption {
    pub id: Uuid,
    pub location_id: Option<Uuid>,
    pub equipment_type: String,
    pub consumption_kwh: f64,
    pub cost_usd: Option<f64>,
    pub efficiency_ratio: Option<f64>,
    pub optimization_suggestions: serde_json::Value,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub metadata: serde_json::Value,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnergyOptimizationSuggestion {
    pub suggestion_type: String,
    pub description: String,
    pub potential_savings_kwh: f64,
    pub potential_savings_usd: f64,
    pub implementation_effort: String,
    pub priority: String,
}

// ============================================================================
// Compliance Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ComplianceEvent {
    pub id: Uuid,
    pub event_type: String,
    pub regulatory_standard: String,
    pub compliance_status: String,
    pub description: String,
    pub affected_entity_type: String,
    pub affected_entity_id: Uuid,
    pub remediation_required: bool,
    pub remediation_actions: serde_json::Value,
    pub auditor_notes: Option<String>,
    pub metadata: serde_json::Value,
    pub occurred_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceStatus {
    pub overall_status: String,
    pub regulatory_standards: Vec<RegulatoryStandardStatus>,
    pub violations: Vec<ComplianceViolation>,
    pub last_audit_date: Option<DateTime<Utc>>,
    pub next_audit_due: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegulatoryStandardStatus {
    pub standard: String,
    pub status: String,
    pub compliance_percentage: f64,
    pub last_check: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceViolation {
    pub id: Uuid,
    pub violation_type: String,
    pub severity: String,
    pub description: String,
    pub remediation_required: bool,
    pub due_date: Option<DateTime<Utc>>,
}

// ============================================================================
// Digital Twin Models
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct DigitalTwinState {
    pub twin_id: Uuid,
    pub physical_entity_id: Uuid,
    pub entity_type: String,
    pub current_state: serde_json::Value,
    pub predicted_state: Option<serde_json::Value>,
    pub simulation_parameters: serde_json::Value,
    pub last_sync: DateTime<Utc>,
    pub sync_status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationScenario {
    pub scenario_name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub expected_outcomes: Option<serde_json::Value>,
    pub simulation_duration: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationResult {
    pub scenario_id: Uuid,
    pub execution_time: i32,
    pub results: serde_json::Value,
    pub performance_metrics: serde_json::Value,
    pub recommendations: Vec<String>,
    pub completed_at: DateTime<Utc>,
}

// ============================================================================
// Mobile Integration Models
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct MobileTaskAssignment {
    pub task_id: Uuid,
    pub user_id: Uuid,
    pub task_type: String,
    pub description: String,
    pub location: Option<serde_json::Value>,
    pub priority: String,
    pub estimated_duration: Option<i32>,
    pub assigned_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BarcodeScanning {
    pub barcode: String,
    pub scan_type: String,
    pub location: Option<serde_json::Value>,
    pub user_id: Uuid,
    pub metadata: Option<serde_json::Value>,
    pub scanned_at: DateTime<Utc>,
}

// ============================================================================
// API Response Models
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
    pub total_items: i64,
    pub has_next: bool,
    pub has_prev: bool,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message),
            timestamp: Utc::now(),
        }
    }
}
