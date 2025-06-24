use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use tracing::{info, error};

use crate::{
    error::{StorageError, StorageResult},
    models::*,
    AppState,
};

/// Predict storage capacity needs
/// GET /analytics/capacity/prediction
pub async fn predict_capacity(
    State(state): State<AppState>,
    Query(query): Query<CapacityPredictionQuery>,
) -> StorageResult<Json<ApiResponse<CapacityPrediction>>> {
    info!("Generating capacity prediction analysis");

    let days_ahead = query.days_ahead.unwrap_or(30);
    let location_id = query.location_id;

    // Get historical storage data
    let historical_data = if let Some(loc_id) = location_id {
        get_location_historical_data(&state, loc_id, 90).await?
    } else {
        get_system_historical_data(&state, 90).await?
    };

    // Perform prediction analysis
    let prediction = calculate_capacity_prediction(&historical_data, days_ahead)?;

    Ok(Json(ApiResponse::success(prediction)))
}

/// Predict maintenance requirements
/// GET /analytics/maintenance/schedule
pub async fn predict_maintenance(
    State(state): State<AppState>,
    Query(query): Query<MaintenancePredictionQuery>,
) -> StorageResult<Json<ApiResponse<MaintenancePrediction>>> {
    info!("Generating maintenance prediction analysis");

    let equipment_type = query.equipment_type.as_deref().unwrap_or("all");
    
    // Get equipment health data
    let equipment_data = get_equipment_health_data(&state, equipment_type).await?;
    
    // Calculate maintenance predictions
    let prediction = calculate_maintenance_prediction(&equipment_data)?;

    Ok(Json(ApiResponse::success(prediction)))
}

/// Optimize energy consumption
/// GET /analytics/energy/optimization
pub async fn optimize_energy(
    State(state): State<AppState>,
    Query(query): Query<EnergyOptimizationQuery>,
) -> StorageResult<Json<ApiResponse<EnergyOptimization>>> {
    info!("Generating energy optimization recommendations");

    let analysis_period = query.analysis_period.unwrap_or(7);

    // Get energy consumption data
    let energy_data = get_energy_consumption_data(&state, analysis_period).await?;
    
    // Generate optimization recommendations
    let optimization = calculate_energy_optimization(&energy_data)?;

    Ok(Json(ApiResponse::success(optimization)))
}

/// Get storage utilization analytics
/// GET /analytics/utilization
pub async fn get_utilization_analytics(
    State(state): State<AppState>,
    Query(query): Query<UtilizationAnalyticsQuery>,
) -> StorageResult<Json<ApiResponse<UtilizationAnalytics>>> {
    info!("Generating utilization analytics");

    let period_days = query.period_days.unwrap_or(30);
    let granularity = query.granularity.as_deref().unwrap_or("daily");

    // Get utilization data
    let utilization_data = get_utilization_data(&state, period_days, granularity).await?;

    Ok(Json(ApiResponse::success(utilization_data)))
}

/// Get sample flow analytics
/// GET /analytics/sample-flow
pub async fn get_sample_flow_analytics(
    State(state): State<AppState>,
    Query(query): Query<SampleFlowQuery>,
) -> StorageResult<Json<ApiResponse<SampleFlowAnalytics>>> {
    info!("Generating sample flow analytics");

    let period_days = query.period_days.unwrap_or(30);

    // Get sample movement data
    let flow_data = get_sample_flow_data(&state, period_days).await?;

    Ok(Json(ApiResponse::success(flow_data)))
}

/// Get cost analysis
/// GET /analytics/cost-analysis
pub async fn get_cost_analysis(
    State(state): State<AppState>,
    Query(query): Query<CostAnalysisQuery>,
) -> StorageResult<Json<ApiResponse<CostAnalysis>>> {
    info!("Generating cost analysis");

    let period_days = query.period_days.unwrap_or(30);
    let include_projections = query.include_projections.unwrap_or(false);

    // Get cost data
    let cost_data = get_cost_data(&state, period_days, include_projections).await?;

    Ok(Json(ApiResponse::success(cost_data)))
}

/// Generate comprehensive analytics report
/// POST /analytics/reports
pub async fn generate_analytics_report(
    State(state): State<AppState>,
    Json(request): Json<AnalyticsReportRequest>,
) -> StorageResult<Json<ApiResponse<AnalyticsReport>>> {
    info!("Generating comprehensive analytics report");

    // Validate request
    if request.report_types.is_empty() {
        return Err(StorageError::Validation("At least one report type must be specified".to_string()));
    }

    let mut report_data = AnalyticsReport {
        id: Uuid::new_v4(),
        report_types: request.report_types.clone(),
        period_start: request.period_start,
        period_end: request.period_end,
        generated_at: Utc::now(),
        data: json!({}),
    };

    // Generate each requested report type
    for report_type in &request.report_types {
        match report_type.as_str() {
            "capacity" => {
                let capacity_data = generate_capacity_report(&state, &request).await?;
                report_data.data["capacity"] = capacity_data;
            }
            "utilization" => {
                let utilization_data = generate_utilization_report(&state, &request).await?;
                report_data.data["utilization"] = utilization_data;
            }
            "energy" => {
                let energy_data = generate_energy_report(&state, &request).await?;
                report_data.data["energy"] = energy_data;
            }
            "cost" => {
                let cost_data = generate_cost_report(&state, &request).await?;
                report_data.data["cost"] = cost_data;
            }
            _ => {
                return Err(StorageError::Validation(format!("Unknown report type: {}", report_type)));
            }
        }
    }

    // Save report to database
    let report_types_json = serde_json::to_value(&report_data.report_types)?;
    sqlx::query(
        r#"
        INSERT INTO analytics_reports (id, report_types, period_start, period_end, data, generated_at, generated_by)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(report_data.id)
    .bind(&report_types_json)
    .bind(request.period_start)
    .bind(request.period_end)
    .bind(&report_data.data)
    .bind(report_data.generated_at)
    .bind(request.generated_by.as_deref())
    .execute(&state.storage_service.db.pool)
    .await?;

    Ok(Json(ApiResponse::success(report_data)))
}

// Helper functions
async fn get_location_historical_data(
    state: &AppState,
    location_id: Uuid,
    days_back: i32,
) -> StorageResult<Vec<HistoricalDataPoint>> {
    let start_date = Utc::now() - Duration::days(days_back as i64);
    
    let data = sqlx::query_as::<_, HistoricalDataPoint>(
        r#"
        SELECT 
            DATE(created_at) as date,
            COUNT(*) as sample_count,
            AVG(CASE WHEN status = 'stored' THEN 1 ELSE 0 END)::float as utilization_rate
        FROM samples 
        WHERE storage_location_id = $1 AND created_at >= $2
        GROUP BY DATE(created_at)
        ORDER BY date
        "#,
    )
    .bind(location_id)
    .bind(start_date)
    .fetch_all(&state.storage_service.db.pool)
    .await?;

    Ok(data)
}

async fn get_system_historical_data(
    state: &AppState,
    days_back: i32,
) -> StorageResult<Vec<HistoricalDataPoint>> {
    let start_date = Utc::now() - Duration::days(days_back as i64);
    
    let data = sqlx::query_as::<_, HistoricalDataPoint>(
        r#"
        SELECT 
            DATE(created_at) as date,
            COUNT(*) as sample_count,
            AVG(CASE WHEN status = 'stored' THEN 1 ELSE 0 END)::float as utilization_rate
        FROM samples 
        WHERE created_at >= $1
        GROUP BY DATE(created_at)
        ORDER BY date
        "#,
    )
    .bind(start_date)
    .fetch_all(&state.storage_service.db.pool)
    .await?;

    Ok(data)
}

fn calculate_capacity_prediction(
    historical_data: &[HistoricalDataPoint],
    days_ahead: i32,
) -> StorageResult<CapacityPrediction> {
    if historical_data.is_empty() {
        return Err(StorageError::Validation("Insufficient historical data for prediction".to_string()));
    }

    // Simple linear regression for prediction
    let n = historical_data.len() as f64;
    let sum_x: f64 = (0..historical_data.len()).map(|i| i as f64).sum();
    let sum_y: f64 = historical_data.iter().map(|d| d.sample_count as f64).sum();
    let sum_xy: f64 = historical_data.iter().enumerate()
        .map(|(i, d)| i as f64 * d.sample_count as f64).sum();
    let sum_x2: f64 = (0..historical_data.len()).map(|i| (i as f64).powi(2)).sum();

    let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
    let intercept = (sum_y - slope * sum_x) / n;

    // Calculate prediction
    let future_x = historical_data.len() as f64 + days_ahead as f64;
    let predicted_samples = (slope * future_x + intercept).max(0.0);

    // Calculate confidence based on data variance
    let mean_samples = sum_y / n;
    let variance: f64 = historical_data.iter()
        .map(|d| (d.sample_count as f64 - mean_samples).powi(2))
        .sum::<f64>() / n;
    let confidence = (1.0 - (variance.sqrt() / mean_samples)).max(0.5).min(1.0);

    Ok(CapacityPrediction {
        predicted_samples: predicted_samples as i32,
        predicted_utilization: predicted_samples / 1000.0, // Assuming 1000 max capacity
        confidence_score: confidence,
        trend_direction: if slope > 0.0 { "increasing".to_string() } else { "decreasing".to_string() },
        prediction_date: Utc::now() + Duration::days(days_ahead as i64),
        recommendations: generate_capacity_recommendations(predicted_samples, confidence),
    })
}

async fn get_equipment_health_data(
    state: &AppState,
    equipment_type: &str,
) -> StorageResult<Vec<EquipmentHealthData>> {
    let equipment_data = sqlx::query_as::<_, EquipmentHealthData>(
        r#"
        SELECT 
            id,
            equipment_type,
            last_maintenance,
            usage_hours,
            performance_score,
            alert_count
        FROM equipment_health 
        WHERE ($1 = 'all' OR equipment_type = $1)
        AND status = 'active'
        "#,
    )
    .bind(equipment_type)
    .fetch_all(&state.storage_service.db.pool)
    .await
    .unwrap_or_default();

    Ok(equipment_data)
}

fn calculate_maintenance_prediction(
    equipment_data: &[EquipmentHealthData],
) -> StorageResult<MaintenancePrediction> {
    let mut maintenance_items = Vec::new();

    for equipment in equipment_data {
        let days_since_maintenance = equipment.last_maintenance
            .map(|last| (Utc::now() - last).num_days())
            .unwrap_or(365);

        let maintenance_score = calculate_maintenance_score(
            days_since_maintenance,
            equipment.usage_hours,
            equipment.performance_score,
            equipment.alert_count,
        );

        if maintenance_score > 0.7 {
            maintenance_items.push(MaintenanceItem {
                equipment_id: equipment.id,
                equipment_type: equipment.equipment_type.clone(),
                priority: if maintenance_score > 0.9 { "critical".to_string() } 
                         else if maintenance_score > 0.8 { "high".to_string() } 
                         else { "medium".to_string() },
                recommended_date: Utc::now() + Duration::days((30.0 * (1.0 - maintenance_score)) as i64),
                estimated_cost: estimate_maintenance_cost(&equipment.equipment_type),
                description: format!("Maintenance required for {} equipment", equipment.equipment_type),
            });
        }
    }

    Ok(MaintenancePrediction {
        total_estimated_cost: maintenance_items.iter().map(|m| m.estimated_cost).sum(),
        maintenance_items,
        generated_at: Utc::now(),
    })
}

fn calculate_maintenance_score(
    days_since_maintenance: i64,
    usage_hours: i32,
    performance_score: f64,
    alert_count: i32,
) -> f64 {
    let age_factor = (days_since_maintenance as f64 / 365.0).min(1.0);
    let usage_factor = (usage_hours as f64 / 8760.0).min(1.0); // Hours in a year
    let performance_factor = 1.0 - performance_score;
    let alert_factor = (alert_count as f64 / 10.0).min(1.0);

    (age_factor * 0.3 + usage_factor * 0.3 + performance_factor * 0.3 + alert_factor * 0.1).min(1.0)
}

fn estimate_maintenance_cost(equipment_type: &str) -> f64 {
    match equipment_type {
        "freezer" => 500.0,
        "sensor" => 100.0,
        "robot" => 2000.0,
        "hvac" => 1500.0,
        _ => 300.0,
    }
}

async fn get_energy_consumption_data(
    state: &AppState,
    analysis_period: i32,
) -> StorageResult<Vec<EnergyDataPoint>> {
    let start_date = Utc::now() - Duration::days(analysis_period as i64);
    
    let data = sqlx::query_as::<_, EnergyDataPoint>(
        r#"
        SELECT 
            DATE(timestamp) as date,
            AVG(power_consumption) as avg_power_kw,
            MAX(power_consumption) as peak_power_kw,
            SUM(power_consumption * 24) as daily_kwh
        FROM energy_readings 
        WHERE timestamp >= $1
        GROUP BY DATE(timestamp)
        ORDER BY date
        "#,
    )
    .bind(start_date)
    .fetch_all(&state.storage_service.db.pool)
    .await
    .unwrap_or_default();

    Ok(data)
}

fn calculate_energy_optimization(
    energy_data: &[EnergyDataPoint],
) -> StorageResult<EnergyOptimization> {
    if energy_data.is_empty() {
        return Ok(EnergyOptimization {
            current_consumption_kwh: 0.0,
            optimized_consumption_kwh: 0.0,
            potential_savings_kwh: 0.0,
            cost_savings_usd: 0.0,
            recommendations: vec!["No data available for optimization".to_string()],
            implementation_priority: vec![],
        });
    }

    let total_consumption: f64 = energy_data.iter().map(|d| d.daily_kwh).sum();
    let avg_daily_consumption = total_consumption / energy_data.len() as f64;
    
    // Calculate optimization potential (simplified)
    let optimization_potential = 0.15; // 15% potential savings
    let optimized_consumption = total_consumption * (1.0 - optimization_potential);
    let potential_savings = total_consumption - optimized_consumption;
    let cost_per_kwh = 0.12; // $0.12 per kWh
    let cost_savings = potential_savings * cost_per_kwh;

    let recommendations = vec![
        "Implement smart temperature controls".to_string(),
        "Optimize freezer defrost cycles".to_string(),
        "Use LED lighting throughout facility".to_string(),
        "Install variable frequency drives on motors".to_string(),
        "Implement power factor correction".to_string(),
    ];

    let implementation_priority = vec![
        OptimizationPriority {
            item: "Smart temperature controls".to_string(),
            savings_kwh: potential_savings * 0.4,
            implementation_cost: 5000.0,
            payback_months: 8,
        },
        OptimizationPriority {
            item: "LED lighting upgrade".to_string(),
            savings_kwh: potential_savings * 0.3,
            implementation_cost: 2000.0,
            payback_months: 4,
        },
        OptimizationPriority {
            item: "VFD installation".to_string(),
            savings_kwh: potential_savings * 0.3,
            implementation_cost: 8000.0,
            payback_months: 12,
        },
    ];

    Ok(EnergyOptimization {
        current_consumption_kwh: total_consumption,
        optimized_consumption_kwh: optimized_consumption,
        potential_savings_kwh: potential_savings,
        cost_savings_usd: cost_savings,
        recommendations,
        implementation_priority,
    })
}

fn generate_capacity_recommendations(predicted_samples: f64, confidence: f64) -> Vec<String> {
    let mut recommendations = Vec::new();

    if predicted_samples > 900.0 {
        recommendations.push("Consider expanding storage capacity".to_string());
        recommendations.push("Implement sample archival policies".to_string());
    }

    if confidence < 0.7 {
        recommendations.push("Collect more historical data for better predictions".to_string());
    }

    if predicted_samples < 100.0 {
        recommendations.push("Current capacity may be oversized".to_string());
    }

    if recommendations.is_empty() {
        recommendations.push("Current capacity planning appears adequate".to_string());
    }

    recommendations
}

// Additional helper functions for comprehensive reports
async fn generate_capacity_report(
    state: &AppState,
    request: &AnalyticsReportRequest,
) -> StorageResult<serde_json::Value> {
    let historical_data = get_system_historical_data(state, 90).await?;
    let prediction = calculate_capacity_prediction(&historical_data, 30)?;
    
    Ok(json!({
        "historical_utilization": historical_data,
        "prediction": prediction,
        "summary": {
            "avg_daily_samples": historical_data.iter().map(|d| d.sample_count).sum::<i32>() / historical_data.len() as i32,
            "trend": prediction.trend_direction,
            "confidence": prediction.confidence_score
        }
    }))
}

async fn generate_utilization_report(
    state: &AppState,
    request: &AnalyticsReportRequest,
) -> StorageResult<serde_json::Value> {
    let period_days = (request.period_end - request.period_start).num_days() as i32;
    let utilization_data = get_utilization_data(state, period_days, "daily").await?;
    
    Ok(json!(utilization_data))
}

async fn generate_energy_report(
    state: &AppState,
    request: &AnalyticsReportRequest,
) -> StorageResult<serde_json::Value> {
    let period_days = (request.period_end - request.period_start).num_days() as i32;
    let energy_data = get_energy_consumption_data(state, period_days).await?;
    let optimization = calculate_energy_optimization(&energy_data)?;
    
    Ok(json!({
        "consumption_data": energy_data,
        "optimization": optimization
    }))
}

async fn generate_cost_report(
    state: &AppState,
    request: &AnalyticsReportRequest,
) -> StorageResult<serde_json::Value> {
    let period_days = (request.period_end - request.period_start).num_days() as i32;
    let cost_data = get_cost_data(state, period_days, true).await?;
    
    Ok(json!(cost_data))
}

async fn get_utilization_data(
    state: &AppState,
    period_days: i32,
    granularity: &str,
) -> StorageResult<UtilizationAnalytics> {
    // This would implement actual utilization data gathering
    // For now, return a structured response
    Ok(UtilizationAnalytics {
        period_days,
        granularity: granularity.to_string(),
        average_utilization: 0.75,
        peak_utilization: 0.95,
        low_utilization: 0.45,
        utilization_trend: "stable".to_string(),
        location_utilization: vec![],
    })
}

async fn get_sample_flow_data(
    state: &AppState,
    period_days: i32,
) -> StorageResult<SampleFlowAnalytics> {
    // This would implement actual sample flow analysis
    Ok(SampleFlowAnalytics {
        period_days,
        total_samples_processed: 1500,
        avg_storage_time_hours: 48.5,
        flow_efficiency: 0.85,
        bottlenecks: vec!["Manual sample handling".to_string()],
        recommendations: vec!["Implement automated sample handling".to_string()],
    })
}

async fn get_cost_data(
    state: &AppState,
    period_days: i32,
    include_projections: bool,
) -> StorageResult<CostAnalysis> {
    // This would implement actual cost analysis
    Ok(CostAnalysis {
        period_days,
        total_cost_usd: 25000.0,
        cost_per_sample_usd: 16.67,
        cost_breakdown: json!({
            "energy": 8000.0,
            "maintenance": 5000.0,
            "labor": 10000.0,
            "supplies": 2000.0
        }),
        projected_monthly_cost: if include_projections { Some(30000.0) } else { None },
        cost_optimization_opportunities: vec![
            "Energy efficiency improvements".to_string(),
            "Preventive maintenance scheduling".to_string(),
        ],
    })
}

// Request/Response structures
#[derive(Debug, Deserialize)]
pub struct CapacityPredictionQuery {
    pub days_ahead: Option<i32>,
    pub location_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct MaintenancePredictionQuery {
    pub equipment_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EnergyOptimizationQuery {
    pub analysis_period: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UtilizationAnalyticsQuery {
    pub period_days: Option<i32>,
    pub granularity: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SampleFlowQuery {
    pub period_days: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CostAnalysisQuery {
    pub period_days: Option<i32>,
    pub include_projections: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyticsReportRequest {
    pub report_types: Vec<String>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub generated_by: Option<String>,
}

// Data structures
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct HistoricalDataPoint {
    pub date: chrono::NaiveDate,
    pub sample_count: i32,
    pub utilization_rate: f64,
}

#[derive(Debug, Serialize)]
pub struct CapacityPrediction {
    pub predicted_samples: i32,
    pub predicted_utilization: f64,
    pub confidence_score: f64,
    pub trend_direction: String,
    pub prediction_date: DateTime<Utc>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct EquipmentHealthData {
    pub id: Uuid,
    pub equipment_type: String,
    pub last_maintenance: Option<DateTime<Utc>>,
    pub usage_hours: i32,
    pub performance_score: f64,
    pub alert_count: i32,
}

#[derive(Debug, Serialize)]
pub struct MaintenancePrediction {
    pub maintenance_items: Vec<MaintenanceItem>,
    pub total_estimated_cost: f64,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MaintenanceItem {
    pub equipment_id: Uuid,
    pub equipment_type: String,
    pub priority: String,
    pub recommended_date: DateTime<Utc>,
    pub estimated_cost: f64,
    pub description: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct EnergyDataPoint {
    pub date: chrono::NaiveDate,
    pub avg_power_kw: f64,
    pub peak_power_kw: f64,
    pub daily_kwh: f64,
}

#[derive(Debug, Serialize)]
pub struct EnergyOptimization {
    pub current_consumption_kwh: f64,
    pub optimized_consumption_kwh: f64,
    pub potential_savings_kwh: f64,
    pub cost_savings_usd: f64,
    pub recommendations: Vec<String>,
    pub implementation_priority: Vec<OptimizationPriority>,
}

#[derive(Debug, Serialize)]
pub struct OptimizationPriority {
    pub item: String,
    pub savings_kwh: f64,
    pub implementation_cost: f64,
    pub payback_months: i32,
}

#[derive(Debug, Serialize)]
pub struct UtilizationAnalytics {
    pub period_days: i32,
    pub granularity: String,
    pub average_utilization: f64,
    pub peak_utilization: f64,
    pub low_utilization: f64,
    pub utilization_trend: String,
    pub location_utilization: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct SampleFlowAnalytics {
    pub period_days: i32,
    pub total_samples_processed: i32,
    pub avg_storage_time_hours: f64,
    pub flow_efficiency: f64,
    pub bottlenecks: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CostAnalysis {
    pub period_days: i32,
    pub total_cost_usd: f64,
    pub cost_per_sample_usd: f64,
    pub cost_breakdown: serde_json::Value,
    pub projected_monthly_cost: Option<f64>,
    pub cost_optimization_opportunities: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AnalyticsReport {
    pub id: Uuid,
    pub report_types: Vec<String>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub generated_at: DateTime<Utc>,
    pub data: serde_json::Value,
}
