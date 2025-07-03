use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LabQuery {
    pub query: String,
    pub context: LabContext,
    pub user_role: UserRole,
    pub query_type: QueryType,
    pub session_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LabContext {
    pub current_samples: Vec<String>,
    pub active_workflows: Vec<String>,
    pub user_department: String,
    pub lab_location: String,
    pub current_projects: Vec<String>,
    pub recent_activities: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum UserRole {
    LabAdministrator,
    PrincipalInvestigator,
    LabTechnician,
    ResearchScientist,
    DataAnalyst,
    Guest,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum QueryType {
    Question,           // General laboratory questions
    ActionRequest,      // Request to perform actions
    DataAnalysis,       // Analysis of laboratory data
    Troubleshooting,    // Help with problems
    Planning,           // Workflow and resource planning
    Compliance,         // Regulatory and compliance queries
}

#[derive(Debug, Serialize, Clone)]
pub struct IntelligentResponse {
    pub response: String,
    pub confidence: f64,
    pub reasoning: String,
    pub suggested_actions: Vec<SuggestedAction>,
    pub relevant_data: Vec<DataPoint>,
    pub predictions: Vec<Prediction>,
    pub follow_up_questions: Vec<String>,
    pub response_time_ms: u64,
    pub sources: Vec<DataSource>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SuggestedAction {
    pub action_type: ActionType,
    pub description: String,
    pub priority: Priority,
    pub estimated_time: String,
    pub required_resources: Vec<String>,
    pub endpoint: Option<String>,
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Clone)]
pub enum ActionType {
    CreateSample,
    UpdateSample,
    AssignStorage,
    RunAnalysis,
    GenerateReport,
    ScheduleTask,
    NotifyUser,
    OptimizeWorkflow,
}

#[derive(Debug, Serialize, Clone)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize, Clone)]
pub struct DataPoint {
    pub data_type: String,
    pub value: serde_json::Value,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub relevance_score: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct Prediction {
    pub prediction_type: PredictionType,
    pub description: String,
    pub confidence: f64,
    pub timeframe: String,
    pub impact: Impact,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub enum PredictionType {
    EquipmentFailure,
    CapacityAlert,
    QualityIssue,
    WorkflowBottleneck,
    ResourceShortage,
    ComplianceRisk,
}

#[derive(Debug, Serialize, Clone)]
pub enum Impact {
    Minimal,
    Moderate,
    Significant,
    Critical,
}

#[derive(Debug, Serialize, Clone)]
pub struct DataSource {
    pub source_type: SourceType,
    pub source_id: String,
    pub confidence: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize, Clone)]
pub enum SourceType {
    Database,
    SensorData,
    UserInput,
    HistoricalData,
    ExternalAPI,
    AIInference,
}

#[derive(Debug, Deserialize)]
pub struct ChatMessage {
    pub message: String,
    pub user_id: Uuid,
    pub session_id: Option<Uuid>,
    pub context: Option<LabContext>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub session_id: Uuid,
    pub suggestions: Vec<String>,
    pub clarification_needed: bool,
    pub clarification_questions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AnalysisRequest {
    pub data_type: String,
    pub data: serde_json::Value,
    pub analysis_type: AnalysisType,
    pub context: LabContext,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum AnalysisType {
    TrendAnalysis,
    AnomalyDetection,
    PatternRecognition,
    PredictiveModeling,
    QualityAssessment,
    PerformanceAnalysis,
}

#[derive(Debug, Serialize)]
pub struct AnalysisResult {
    pub insights: Vec<Insight>,
    pub visualizations: Vec<Visualization>,
    pub recommendations: Vec<Recommendation>,
    pub confidence_score: f64,
    pub processing_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct Insight {
    pub title: String,
    pub description: String,
    pub significance: f64,
    pub supporting_data: Vec<DataPoint>,
}

#[derive(Debug, Serialize)]
pub struct Visualization {
    pub chart_type: String,
    pub data: serde_json::Value,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct Recommendation {
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub impact: Impact,
    pub implementation_effort: ImplementationEffort,
    pub actions: Vec<SuggestedAction>,
}

#[derive(Debug, Serialize)]
pub enum ImplementationEffort {
    Minimal,
    Low,
    Medium,
    High,
    Extensive,
}

// Ollama-specific models
#[derive(Debug, Serialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub system: Option<String>,
    pub stream: bool,
    pub options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
pub struct OllamaOptions {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<u32>,
    pub repeat_penalty: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct OllamaResponse {
    pub response: String,
    pub done: bool,
    pub context: Option<Vec<u64>>,
}

#[derive(Debug, Serialize)]
pub struct ProactiveSuggestionRequest {
    pub user_context: LabContext,
    pub current_time: DateTime<Utc>,
    pub user_role: UserRole,
}

#[derive(Debug, Serialize)]
pub struct ProactiveSuggestion {
    pub suggestion_type: SuggestionType,
    pub title: String,
    pub description: String,
    pub urgency: Priority,
    pub potential_benefit: String,
    pub action: Option<SuggestedAction>,
}

#[derive(Debug, Serialize)]
pub enum SuggestionType {
    Optimization,
    MaintenanceReminder,
    QualityAlert,
    ResourceManagement,
    WorkflowImprovement,
    ComplianceCheck,
    CapacityPlanning,
}

// Database models for persistence
#[derive(Debug, sqlx::FromRow)]
pub struct QueryHistory {
    pub id: Uuid,
    pub user_id: Uuid,
    pub query: String,
    pub response: String,
    pub confidence: f64,
    pub response_time_ms: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub context: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub is_active: bool,
} 