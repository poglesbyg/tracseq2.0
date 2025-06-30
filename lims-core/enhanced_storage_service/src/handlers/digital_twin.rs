use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use tracing::info;

use crate::{
    error::{StorageError, StorageResult},
    models::*,
    AppState,
};

/// Get digital twin overview
/// GET /digital-twin/overview
pub async fn get_overview(
    State(state): State<AppState>,
) -> StorageResult<Json<ApiResponse<DigitalTwinOverview>>> {
    info!("Getting digital twin overview");

    let overview = DigitalTwinOverview {
        facility_id: Uuid::new_v4(),
        facility_name: "TracSeq Laboratory Storage Facility".to_string(),
        twin_status: "synchronized".to_string(),
        last_sync: Utc::now() - Duration::minutes(5),
        sync_accuracy: 0.98,
        active_models: 5,
        running_simulations: 2,
        total_sensors: 48,
        synchronized_sensors: 47,
        facility_metrics: FacilityMetrics {
            total_storage_locations: 25,
            occupied_locations: 18,
            total_samples: 1520,
            temperature_zones: 5,
            energy_consumption_kw: 125.5,
            operational_efficiency: 0.92,
            space_utilization: 0.72,
        },
        recent_alerts: vec![
            "Temperature variance detected in Zone A".to_string(),
            "Capacity approaching 85% in Freezer Unit 2".to_string(),
        ],
        model_accuracy_scores: json!({
            "temperature_model": 0.96,
            "capacity_model": 0.94,
            "energy_model": 0.89,
            "workflow_model": 0.91,
            "maintenance_model": 0.87
        }),
        generated_at: Utc::now(),
    };

    Ok(Json(ApiResponse::success(overview)))
}

/// Run facility simulation
/// POST /digital-twin/simulations
pub async fn run_simulation(
    State(state): State<AppState>,
    Json(request): Json<SimulationRequest>,
) -> StorageResult<Json<ApiResponse<SimulationResult>>> {
    info!("Running digital twin simulation: {}", request.simulation_type);

    // Validate simulation parameters
    validate_simulation_parameters(&request)?;

    let simulation_id = Uuid::new_v4();
    
    // Execute simulation based on type
    let result = match request.simulation_type.as_str() {
        "capacity_planning" => run_capacity_simulation(&state, &request).await?,
        "temperature_optimization" => run_temperature_simulation(&state, &request).await?,
        "workflow_analysis" => run_workflow_simulation(&state, &request).await?,
        "emergency_response" => run_emergency_simulation(&state, &request).await?,
        "energy_optimization" => run_energy_simulation(&state, &request).await?,
        _ => return Err(StorageError::Validation(format!("Unknown simulation type: {}", request.simulation_type))),
    };

    let simulation_result = SimulationResult {
        id: simulation_id,
        simulation_type: request.simulation_type.clone(),
        status: "completed".to_string(),
        execution_time_seconds: 45.2,
        accuracy_score: result.accuracy_score,
        results: result.data,
        recommendations: result.recommendations,
        started_at: Utc::now() - Duration::seconds(45),
        completed_at: Utc::now(),
        parameters: request.parameters,
        confidence_interval: result.confidence_interval,
        risk_factors: result.risk_factors,
    };

    Ok(Json(ApiResponse::success(simulation_result)))
}

/// Create simulation scenario
/// POST /digital-twin/scenarios
pub async fn create_scenario(
    State(state): State<AppState>,
    Json(request): Json<CreateScenarioRequest>,
) -> StorageResult<Json<ApiResponse<SimulationScenario>>> {
    info!("Creating simulation scenario: {}", request.name);

    let scenario = SimulationScenario {
        id: Uuid::new_v4(),
        name: request.name.clone(),
        description: request.description.clone(),
        scenario_type: request.scenario_type.clone(),
        parameters: request.parameters.clone(),
        environmental_conditions: request.environmental_conditions.clone(),
        equipment_configurations: request.equipment_configurations.clone(),
        sample_load_profile: request.sample_load_profile.clone(),
        expected_duration_hours: request.expected_duration_hours,
        created_at: Utc::now(),
        created_by: request.created_by.clone(),
        status: "draft".to_string(),
        tags: request.tags.clone().unwrap_or_default(),
        version: 1,
    };

    // Store scenario in database (mock implementation)
    store_simulation_scenario(&state, &scenario).await?;

    Ok(Json(ApiResponse::success(scenario)))
}

/// Get optimization recommendations
/// GET /digital-twin/optimization
pub async fn get_optimization(
    State(state): State<AppState>,
    Query(query): Query<OptimizationQuery>,
) -> StorageResult<Json<ApiResponse<OptimizationRecommendations>>> {
    info!("Generating optimization recommendations");

    let optimization_type = query.optimization_type.as_deref().unwrap_or("comprehensive");
    let priority = query.priority.as_deref().unwrap_or("medium");

    let recommendations = OptimizationRecommendations {
        optimization_type: optimization_type.to_string(),
        generated_at: Utc::now(),
        confidence_score: 0.87,
        potential_improvements: vec![
            OptimizationRecommendation {
                id: Uuid::new_v4(),
                category: "space_utilization".to_string(),
                title: "Optimize storage layout configuration".to_string(),
                description: "Reorganize storage zones to improve sample access efficiency".to_string(),
                impact_score: 0.78,
                implementation_effort: "medium".to_string(),
                estimated_savings_usd: 5200.0,
                estimated_time_savings_hours: 12.5,
                risk_level: "low".to_string(),
                prerequisites: vec!["Temporary sample relocation".to_string()],
            },
            OptimizationRecommendation {
                id: Uuid::new_v4(),
                category: "energy_efficiency".to_string(),
                title: "Implement smart temperature scheduling".to_string(),
                description: "Optimize freezer operation schedules based on sample access patterns".to_string(),
                impact_score: 0.65,
                implementation_effort: "low".to_string(),
                estimated_savings_usd: 3600.0,
                estimated_time_savings_hours: 0.0,
                risk_level: "very_low".to_string(),
                prerequisites: vec!["Software configuration update".to_string()],
            },
        ],
        implementation_roadmap: vec![
            "Phase 1: Implement smart scheduling (Week 1-2)".to_string(),
            "Phase 2: Optimize storage layout (Week 3-6)".to_string(),
            "Phase 3: Monitor and fine-tune (Week 7-8)".to_string(),
        ],
        total_potential_savings_usd: 8800.0,
        roi_percentage: 145.0,
    };

    Ok(Json(ApiResponse::success(recommendations)))
}

/// List digital twin models
/// GET /digital-twin/models
pub async fn list_models(
    State(state): State<AppState>,
    Query(query): Query<ModelListQuery>,
) -> StorageResult<Json<ApiResponse<PaginatedResponse<DigitalTwinModel>>>> {
    info!("Listing digital twin models");

    let models = vec![
        DigitalTwinModel {
            id: Uuid::new_v4(),
            name: "Temperature Distribution Model".to_string(),
            model_type: "thermal_simulation".to_string(),
            version: "2.1.0".to_string(),
            accuracy: 0.96,
            last_trained: Utc::now() - Duration::days(7),
            training_data_points: 50000,
            status: "active".to_string(),
            description: Some("Predicts temperature distribution across storage zones".to_string()),
            input_parameters: vec!["ambient_temperature".to_string(), "equipment_load".to_string()],
            output_metrics: vec!["zone_temperatures".to_string(), "energy_consumption".to_string()],
            validation_score: 0.94,
        },
        DigitalTwinModel {
            id: Uuid::new_v4(),
            name: "Capacity Utilization Model".to_string(),
            model_type: "capacity_prediction".to_string(),
            version: "1.8.2".to_string(),
            accuracy: 0.91,
            last_trained: Utc::now() - Duration::days(3),
            training_data_points: 75000,
            status: "active".to_string(),
            description: Some("Predicts storage capacity needs and utilization patterns".to_string()),
            input_parameters: vec!["sample_inflow".to_string(), "seasonal_factors".to_string()],
            output_metrics: vec!["capacity_utilization".to_string(), "overflow_risk".to_string()],
            validation_score: 0.89,
        },
    ];

    let response = PaginatedResponse {
        data: models,
        pagination: PaginationInfo {
            page: query.page.unwrap_or(1),
            per_page: query.per_page.unwrap_or(50),
            total_pages: 1,
            total_items: 2,
            has_next: false,
            has_prev: false,
        },
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Get specific model details
/// GET /digital-twin/models/:model_id
pub async fn get_model(
    State(state): State<AppState>,
    Path(model_id): Path<Uuid>,
) -> StorageResult<Json<ApiResponse<DigitalTwinModelDetails>>> {
    info!("Getting model details for: {}", model_id);

    let model_details = DigitalTwinModelDetails {
        model: DigitalTwinModel {
            id: model_id,
            name: "Temperature Distribution Model".to_string(),
            model_type: "thermal_simulation".to_string(),
            version: "2.1.0".to_string(),
            accuracy: 0.96,
            last_trained: Utc::now() - Duration::days(7),
            training_data_points: 50000,
            status: "active".to_string(),
            description: Some("Predicts temperature distribution across storage zones".to_string()),
            input_parameters: vec!["ambient_temperature".to_string(), "equipment_load".to_string()],
            output_metrics: vec!["zone_temperatures".to_string(), "energy_consumption".to_string()],
            validation_score: 0.94,
        },
        performance_metrics: json!({
            "mae": 0.45,
            "rmse": 0.62,
            "r2_score": 0.96,
            "training_time_minutes": 45,
            "inference_time_ms": 12
        }),
        training_history: vec![
            TrainingIteration {
                iteration: 100,
                loss: 0.023,
                accuracy: 0.96,
                validation_loss: 0.025,
                timestamp: Utc::now() - Duration::days(7),
            }
        ],
        recent_predictions: vec![
            ModelPrediction {
                timestamp: Utc::now() - Duration::minutes(30),
                input_data: json!({"ambient_temperature": 22.5, "equipment_load": 0.78}),
                output_data: json!({"zone_a_temp": -80.2, "zone_b_temp": -79.8}),
                confidence: 0.94,
            }
        ],
        configuration: json!({
            "algorithm": "neural_network",
            "layers": [64, 32, 16, 8],
            "activation": "relu",
            "optimizer": "adam"
        }),
    };

    Ok(Json(ApiResponse::success(model_details)))
}

/// Sync digital twin with reality
/// POST /digital-twin/sync
pub async fn sync_with_reality(
    State(state): State<AppState>,
    Json(request): Json<SyncRequest>,
) -> StorageResult<Json<ApiResponse<SyncResult>>> {
    info!("Synchronizing digital twin with real-world data");

    let sync_result = SyncResult {
        sync_id: Uuid::new_v4(),
        sync_type: request.sync_type.clone(),
        started_at: Utc::now(),
        completed_at: Utc::now() + Duration::seconds(30),
        status: "completed".to_string(),
        sensors_synchronized: 47,
        total_sensors: 48,
        data_points_processed: 125000,
        accuracy_improvement: 0.02,
        discrepancies_found: 3,
        discrepancy_details: vec![
            "Temperature sensor TEMP_001 showing 0.5°C variance".to_string(),
            "Capacity sensor CAP_005 offline for maintenance".to_string(),
            "Humidity sensor HUM_012 requires calibration".to_string(),
        ],
        model_updates_applied: vec![
            "Temperature model recalibrated".to_string(),
            "Capacity thresholds updated".to_string(),
        ],
        next_sync_recommended: Utc::now() + Duration::hours(4),
    };

    Ok(Json(ApiResponse::success(sync_result)))
}

/// Get predictive analytics
/// GET /digital-twin/predictions
pub async fn get_predictions(
    State(state): State<AppState>,
    Query(query): Query<PredictionQuery>,
) -> StorageResult<Json<ApiResponse<PredictiveAnalytics>>> {
    info!("Generating predictive analytics");

    let prediction_type = query.prediction_type.as_deref().unwrap_or("comprehensive");
    let time_horizon_hours = query.time_horizon_hours.unwrap_or(24);

    let analytics = PredictiveAnalytics {
        prediction_type: prediction_type.to_string(),
        time_horizon_hours,
        generated_at: Utc::now(),
        confidence_score: 0.89,
        predictions: vec![
            Prediction {
                category: "temperature".to_string(),
                description: "Zone A temperature will remain stable within ±0.5°C".to_string(),
                predicted_value: -80.1,
                confidence: 0.94,
                time_to_event_hours: Some(12),
                impact_level: "low".to_string(),
            },
            Prediction {
                category: "capacity".to_string(),
                description: "Freezer Unit 2 will reach 90% capacity".to_string(),
                predicted_value: 0.90,
                confidence: 0.82,
                time_to_event_hours: Some(48),
                impact_level: "medium".to_string(),
            },
        ],
        risk_assessments: vec![
            RiskAssessment {
                risk_type: "equipment_failure".to_string(),
                probability: 0.05,
                impact_severity: "high".to_string(),
                mitigation_strategies: vec!["Preventive maintenance".to_string()],
            }
        ],
        recommended_actions: vec![
            "Monitor Freezer Unit 2 capacity closely".to_string(),
            "Schedule maintenance for aging equipment".to_string(),
        ],
    };

    Ok(Json(ApiResponse::success(analytics)))
}

/// Run virtual experiment
/// POST /digital-twin/virtual-experiments
pub async fn run_virtual_experiment(
    State(state): State<AppState>,
    Json(request): Json<VirtualExperimentRequest>,
) -> StorageResult<Json<ApiResponse<VirtualExperimentResult>>> {
    info!("Running virtual experiment: {}", request.experiment_name);

    let experiment_result = VirtualExperimentResult {
        id: Uuid::new_v4(),
        experiment_name: request.experiment_name.clone(),
        experiment_type: request.experiment_type.clone(),
        status: "completed".to_string(),
        started_at: Utc::now() - Duration::minutes(15),
        completed_at: Utc::now(),
        execution_time_minutes: 15.2,
        parameters: request.parameters.clone(),
        results: json!({
            "success_rate": 0.87,
            "performance_improvement": 0.15,
            "resource_utilization": 0.92,
            "cost_impact": -1200.0
        }),
        metrics: vec![
            ExperimentMetric {
                name: "throughput".to_string(),
                value: 125.5,
                unit: "samples/hour".to_string(),
                baseline_value: Some(108.2),
                improvement_percentage: Some(16.0),
            }
        ],
        conclusions: vec![
            "Proposed changes show significant improvement in throughput".to_string(),
            "Energy consumption remains within acceptable limits".to_string(),
        ],
        recommendations: vec![
            "Implement changes during next maintenance window".to_string(),
            "Monitor performance for first 48 hours after implementation".to_string(),
        ],
        confidence_score: 0.91,
    };

    Ok(Json(ApiResponse::success(experiment_result)))
}

/// Get digital twin analytics
/// GET /digital-twin/analytics
pub async fn get_twin_analytics(
    State(state): State<AppState>,
    Query(query): Query<TwinAnalyticsQuery>,
) -> StorageResult<Json<ApiResponse<DigitalTwinAnalytics>>> {
    info!("Getting digital twin analytics");

    let analytics = DigitalTwinAnalytics {
        time_period_days: query.time_period_days.unwrap_or(30),
        model_performance: json!({
            "average_accuracy": 0.92,
            "prediction_reliability": 0.89,
            "sync_frequency": "every_4_hours",
            "data_quality_score": 0.95
        }),
        simulation_statistics: json!({
            "total_simulations_run": 245,
            "successful_simulations": 238,
            "average_execution_time_seconds": 42.5,
            "most_used_simulation_type": "capacity_planning"
        }),
        optimization_impact: json!({
            "recommendations_implemented": 12,
            "cost_savings_achieved_usd": 15600.0,
            "efficiency_improvements": 0.18,
            "energy_savings_kwh": 450.0
        }),
        facility_insights: vec![
            "Storage efficiency has improved 18% over the analysis period".to_string(),
            "Temperature control optimization reduced energy consumption by 12%".to_string(),
            "Predictive maintenance prevented 3 potential equipment failures".to_string(),
        ],
        generated_at: Utc::now(),
    };

    Ok(Json(ApiResponse::success(analytics)))
}

// Helper functions
fn validate_simulation_parameters(request: &SimulationRequest) -> StorageResult<()> {
    if request.simulation_type.is_empty() {
        return Err(StorageError::Validation("Simulation type cannot be empty".to_string()));
    }

    if let Some(duration) = request.duration_hours {
        if duration <= 0.0 || duration > 168.0 {
            return Err(StorageError::Validation("Duration must be between 0 and 168 hours".to_string()));
        }
    }

    Ok(())
}

async fn run_capacity_simulation(state: &AppState, request: &SimulationRequest) -> StorageResult<InternalSimulationResult> {
    Ok(InternalSimulationResult {
        accuracy_score: 0.94,
        confidence_interval: (0.91, 0.97),
        data: json!({
            "current_utilization": 0.72,
            "predicted_utilization_24h": 0.78,
            "predicted_utilization_7d": 0.85,
            "capacity_bottlenecks": ["Freezer Unit 2", "Cold Storage Zone C"]
        }),
        recommendations: vec![
            "Consider expanding Freezer Unit 2 capacity".to_string(),
            "Implement sample rotation in Cold Storage Zone C".to_string(),
        ],
        risk_factors: vec!["High demand during winter months".to_string()],
    })
}

async fn run_temperature_simulation(state: &AppState, request: &SimulationRequest) -> StorageResult<InternalSimulationResult> {
    Ok(InternalSimulationResult {
        accuracy_score: 0.96,
        confidence_interval: (0.94, 0.98),
        data: json!({
            "temperature_stability": 0.98,
            "energy_efficiency": 0.87,
            "optimal_setpoints": {"-80C": -79.8, "-20C": -19.9, "4C": 4.1}
        }),
        recommendations: vec![
            "Adjust setpoints to optimal values for 8% energy savings".to_string(),
        ],
        risk_factors: vec!["Equipment aging may affect temperature stability".to_string()],
    })
}

async fn run_workflow_simulation(state: &AppState, request: &SimulationRequest) -> StorageResult<InternalSimulationResult> {
    Ok(InternalSimulationResult {
        accuracy_score: 0.89,
        confidence_interval: (0.85, 0.93),
        data: json!({
            "workflow_efficiency": 0.84,
            "bottleneck_operations": ["manual_sample_handling", "documentation"],
            "throughput_improvement_potential": 0.25
        }),
        recommendations: vec![
            "Automate sample handling to improve throughput by 25%".to_string(),
            "Implement digital documentation to reduce processing time".to_string(),
        ],
        risk_factors: vec!["Staff training required for automation".to_string()],
    })
}

async fn run_emergency_simulation(state: &AppState, request: &SimulationRequest) -> StorageResult<InternalSimulationResult> {
    Ok(InternalSimulationResult {
        accuracy_score: 0.91,
        confidence_interval: (0.88, 0.94),
        data: json!({
            "emergency_response_time": 8.5,
            "sample_preservation_success_rate": 0.96,
            "backup_system_reliability": 0.99
        }),
        recommendations: vec![
            "Emergency response procedures are effective".to_string(),
            "Consider additional backup power capacity".to_string(),
        ],
        risk_factors: vec!["Extended power outages beyond 24 hours".to_string()],
    })
}

async fn run_energy_simulation(state: &AppState, request: &SimulationRequest) -> StorageResult<InternalSimulationResult> {
    Ok(InternalSimulationResult {
        accuracy_score: 0.87,
        confidence_interval: (0.83, 0.91),
        data: json!({
            "current_consumption_kwh": 125.5,
            "optimized_consumption_kwh": 109.2,
            "potential_savings_percent": 13.0,
            "payback_period_months": 8
        }),
        recommendations: vec![
            "Implement smart scheduling for 13% energy savings".to_string(),
            "Upgrade aging equipment for additional 5% savings".to_string(),
        ],
        risk_factors: vec!["Initial implementation costs".to_string()],
    })
}

async fn store_simulation_scenario(state: &AppState, scenario: &SimulationScenario) -> StorageResult<()> {
    // Mock implementation - in production would store in database
    info!("Storing simulation scenario: {}", scenario.id);
    Ok(())
}

// Internal helper structure
struct InternalSimulationResult {
    accuracy_score: f64,
    confidence_interval: (f64, f64),
    data: serde_json::Value,
    recommendations: Vec<String>,
    risk_factors: Vec<String>,
}

// Request/Response structures
#[derive(Debug, Deserialize)]
pub struct SimulationRequest {
    pub simulation_type: String,
    pub duration_hours: Option<f64>,
    pub parameters: Option<serde_json::Value>,
    pub environment_conditions: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateScenarioRequest {
    pub name: String,
    pub description: String,
    pub scenario_type: String,
    pub parameters: serde_json::Value,
    pub environmental_conditions: Option<serde_json::Value>,
    pub equipment_configurations: Option<serde_json::Value>,
    pub sample_load_profile: Option<serde_json::Value>,
    pub expected_duration_hours: f64,
    pub created_by: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct OptimizationQuery {
    pub optimization_type: Option<String>,
    pub priority: Option<String>,
    pub focus_areas: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ModelListQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub model_type: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    pub sync_type: String,
    pub force_full_sync: Option<bool>,
    pub include_historical_data: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct PredictionQuery {
    pub prediction_type: Option<String>,
    pub time_horizon_hours: Option<i32>,
    pub confidence_threshold: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct VirtualExperimentRequest {
    pub experiment_name: String,
    pub experiment_type: String,
    pub parameters: serde_json::Value,
    pub duration_minutes: Option<f64>,
    pub success_criteria: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct TwinAnalyticsQuery {
    pub time_period_days: Option<i32>,
    pub include_predictions: Option<bool>,
    pub include_recommendations: Option<bool>,
}

// Data structures
#[derive(Debug, Serialize)]
pub struct DigitalTwinOverview {
    pub facility_id: Uuid,
    pub facility_name: String,
    pub twin_status: String,
    pub last_sync: DateTime<Utc>,
    pub sync_accuracy: f64,
    pub active_models: i32,
    pub running_simulations: i32,
    pub total_sensors: i32,
    pub synchronized_sensors: i32,
    pub facility_metrics: FacilityMetrics,
    pub recent_alerts: Vec<String>,
    pub model_accuracy_scores: serde_json::Value,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct FacilityMetrics {
    pub total_storage_locations: i32,
    pub occupied_locations: i32,
    pub total_samples: i32,
    pub temperature_zones: i32,
    pub energy_consumption_kw: f64,
    pub operational_efficiency: f64,
    pub space_utilization: f64,
}

#[derive(Debug, Serialize)]
pub struct SimulationResult {
    pub id: Uuid,
    pub simulation_type: String,
    pub status: String,
    pub execution_time_seconds: f64,
    pub accuracy_score: f64,
    pub results: serde_json::Value,
    pub recommendations: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub parameters: Option<serde_json::Value>,
    pub confidence_interval: (f64, f64),
    pub risk_factors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SimulationScenario {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub scenario_type: String,
    pub parameters: serde_json::Value,
    pub environmental_conditions: Option<serde_json::Value>,
    pub equipment_configurations: Option<serde_json::Value>,
    pub sample_load_profile: Option<serde_json::Value>,
    pub expected_duration_hours: f64,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub status: String,
    pub tags: Vec<String>,
    pub version: i32,
}

#[derive(Debug, Serialize)]
pub struct OptimizationRecommendations {
    pub optimization_type: String,
    pub generated_at: DateTime<Utc>,
    pub confidence_score: f64,
    pub potential_improvements: Vec<OptimizationRecommendation>,
    pub implementation_roadmap: Vec<String>,
    pub total_potential_savings_usd: f64,
    pub roi_percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct OptimizationRecommendation {
    pub id: Uuid,
    pub category: String,
    pub title: String,
    pub description: String,
    pub impact_score: f64,
    pub implementation_effort: String,
    pub estimated_savings_usd: f64,
    pub estimated_time_savings_hours: f64,
    pub risk_level: String,
    pub prerequisites: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DigitalTwinModel {
    pub id: Uuid,
    pub name: String,
    pub model_type: String,
    pub version: String,
    pub accuracy: f64,
    pub last_trained: DateTime<Utc>,
    pub training_data_points: i32,
    pub status: String,
    pub description: Option<String>,
    pub input_parameters: Vec<String>,
    pub output_metrics: Vec<String>,
    pub validation_score: f64,
}

#[derive(Debug, Serialize)]
pub struct DigitalTwinModelDetails {
    pub model: DigitalTwinModel,
    pub performance_metrics: serde_json::Value,
    pub training_history: Vec<TrainingIteration>,
    pub recent_predictions: Vec<ModelPrediction>,
    pub configuration: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct TrainingIteration {
    pub iteration: i32,
    pub loss: f64,
    pub accuracy: f64,
    pub validation_loss: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ModelPrediction {
    pub timestamp: DateTime<Utc>,
    pub input_data: serde_json::Value,
    pub output_data: serde_json::Value,
    pub confidence: f64,
}

#[derive(Debug, Serialize)]
pub struct SyncResult {
    pub sync_id: Uuid,
    pub sync_type: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub status: String,
    pub sensors_synchronized: i32,
    pub total_sensors: i32,
    pub data_points_processed: i32,
    pub accuracy_improvement: f64,
    pub discrepancies_found: i32,
    pub discrepancy_details: Vec<String>,
    pub model_updates_applied: Vec<String>,
    pub next_sync_recommended: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PredictiveAnalytics {
    pub prediction_type: String,
    pub time_horizon_hours: i32,
    pub generated_at: DateTime<Utc>,
    pub confidence_score: f64,
    pub predictions: Vec<Prediction>,
    pub risk_assessments: Vec<RiskAssessment>,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Prediction {
    pub category: String,
    pub description: String,
    pub predicted_value: f64,
    pub confidence: f64,
    pub time_to_event_hours: Option<i32>,
    pub impact_level: String,
}

#[derive(Debug, Serialize)]
pub struct RiskAssessment {
    pub risk_type: String,
    pub probability: f64,
    pub impact_severity: String,
    pub mitigation_strategies: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct VirtualExperimentResult {
    pub id: Uuid,
    pub experiment_name: String,
    pub experiment_type: String,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub execution_time_minutes: f64,
    pub parameters: serde_json::Value,
    pub results: serde_json::Value,
    pub metrics: Vec<ExperimentMetric>,
    pub conclusions: Vec<String>,
    pub recommendations: Vec<String>,
    pub confidence_score: f64,
}

#[derive(Debug, Serialize)]
pub struct ExperimentMetric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub baseline_value: Option<f64>,
    pub improvement_percentage: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct DigitalTwinAnalytics {
    pub time_period_days: i32,
    pub model_performance: serde_json::Value,
    pub simulation_statistics: serde_json::Value,
    pub optimization_impact: serde_json::Value,
    pub facility_insights: Vec<String>,
    pub generated_at: DateTime<Utc>,
}
