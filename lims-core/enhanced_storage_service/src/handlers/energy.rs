use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use tracing::info;

use crate::{
    error::{StorageError, StorageResult},
    models::*,
    AppState,
};

/// Get energy consumption overview
/// GET /energy/overview
pub async fn get_consumption_overview(
    State(state): State<AppState>,
) -> StorageResult<Json<ApiResponse<EnergyOverview>>> {
    info!("Getting energy consumption overview");

    let overview = EnergyOverview {
        facility_id: Uuid::new_v4(),
        facility_name: "TracSeq Laboratory Storage Facility".to_string(),
        current_consumption_kw: 125.8,
        daily_consumption_kwh: 3019.2,
        monthly_consumption_kwh: 89456.0,
        baseline_consumption_kw: 142.5,
        efficiency_percentage: 88.7,
        cost_usd_daily: 453.88,
        cost_usd_monthly: 13456.40,
        carbon_footprint_kg_co2: 44.72,
        energy_breakdown: EnergyBreakdown {
            cooling_systems: 62.5,
            lighting: 15.3,
            equipment: 28.7,
            hvac: 18.9,
            other: 5.4,
        },
        efficiency_metrics: EfficiencyMetrics {
            power_usage_effectiveness: 1.42,
            cooling_efficiency_cop: 3.8,
            equipment_utilization: 0.74,
            standby_power_percentage: 12.5,
        },
        alerts: vec![
            EnergyAlert {
                id: Uuid::new_v4(),
                alert_type: "high_consumption".to_string(),
                equipment: "Freezer Unit B".to_string(),
                message: "Consumption 15% above baseline".to_string(),
                severity: "medium".to_string(),
                timestamp: Utc::now() - Duration::minutes(30),
            }
        ],
        generated_at: Utc::now(),
    };

    Ok(Json(ApiResponse::success(overview)))
}

/// Get real-time energy metrics
/// GET /energy/metrics/realtime
pub async fn get_realtime_metrics(
    State(state): State<AppState>,
    Query(query): Query<RealtimeQuery>,
) -> StorageResult<Json<ApiResponse<RealtimeEnergyMetrics>>> {
    info!("Getting real-time energy metrics");

    let metrics = RealtimeEnergyMetrics {
        timestamp: Utc::now(),
        total_power_kw: 125.8,
        voltage_v: 240.2,
        current_a: 524.1,
        power_factor: 0.89,
        frequency_hz: 60.02,
        equipment_metrics: vec![
            EquipmentMetric {
                equipment_id: Uuid::new_v4(),
                equipment_name: "Freezer Unit A".to_string(),
                equipment_type: "ultra_low_freezer".to_string(),
                power_consumption_kw: 2.8,
                efficiency_percentage: 87.5,
                temperature_setpoint: -80.0,
                actual_temperature: -79.8,
                status: "operational".to_string(),
                runtime_hours: 168.5,
                maintenance_due_hours: Some(504),
            },
            EquipmentMetric {
                equipment_id: Uuid::new_v4(),
                equipment_name: "HVAC System 1".to_string(),
                equipment_type: "hvac".to_string(),
                power_consumption_kw: 12.4,
                efficiency_percentage: 82.3,
                temperature_setpoint: 22.0,
                actual_temperature: 21.8,
                status: "operational".to_string(),
                runtime_hours: 8760.0,
                maintenance_due_hours: Some(240),
            },
        ],
        zone_consumption: vec![
            ZoneEnergyData {
                zone_id: "zone_a".to_string(),
                zone_name: "Ultra-Low Storage".to_string(),
                power_consumption_kw: 45.2,
                target_temperature: -80.0,
                actual_temperature: -79.8,
                occupancy_percentage: 85.0,
                efficiency_score: 0.91,
            },
            ZoneEnergyData {
                zone_id: "zone_b".to_string(),
                zone_name: "Standard Refrigeration".to_string(),
                power_consumption_kw: 18.7,
                target_temperature: 4.0,
                actual_temperature: 4.2,
                occupancy_percentage: 72.0,
                efficiency_score: 0.88,
            },
        ],
        grid_data: GridData {
            grid_frequency_hz: 60.02,
            voltage_stability: 0.98,
            power_quality_score: 0.92,
            renewable_percentage: 15.8,
            peak_demand_kw: 156.7,
            off_peak_rate_usd_kwh: 0.08,
            peak_rate_usd_kwh: 0.15,
        },
    };

    Ok(Json(ApiResponse::success(metrics)))
}

/// Get energy consumption history
/// GET /energy/history
pub async fn get_consumption_history(
    State(state): State<AppState>,
    Query(query): Query<HistoryQuery>,
) -> StorageResult<Json<ApiResponse<EnergyHistory>>> {
    info!("Getting energy consumption history");

    let time_period = query.time_period.as_deref().unwrap_or("7_days");
    let granularity = query.granularity.as_deref().unwrap_or("hourly");

    let history = EnergyHistory {
        time_period: time_period.to_string(),
        granularity: granularity.to_string(),
        start_time: Utc::now() - Duration::days(7),
        end_time: Utc::now(),
        total_consumption_kwh: 21134.4,
        average_power_kw: 125.5,
        peak_power_kw: 156.7,
        minimum_power_kw: 98.2,
        cost_breakdown: CostBreakdown {
            energy_cost_usd: 2536.13,
            demand_charges_usd: 234.56,
            taxes_fees_usd: 127.91,
            total_cost_usd: 2898.60,
        },
        consumption_data: generate_consumption_data(time_period, granularity),
        efficiency_trends: EfficiencyTrends {
            average_pue: 1.43,
            best_pue: 1.38,
            worst_pue: 1.52,
            trend_direction: "improving".to_string(),
            efficiency_improvement_percentage: 3.2,
        },
        peak_demand_events: vec![
            PeakDemandEvent {
                timestamp: Utc::now() - Duration::days(2),
                peak_power_kw: 156.7,
                duration_minutes: 45,
                cause: "Simultaneous compressor startup".to_string(),
                cost_impact_usd: 89.45,
            }
        ],
        carbon_footprint: CarbonFootprint {
            total_co2_kg: 10567.2,
            co2_per_kwh: 0.5,
            renewable_offset_kg: 1670.6,
            net_carbon_footprint_kg: 8896.6,
        },
    };

    Ok(Json(ApiResponse::success(history)))
}

/// Create energy optimization plan
/// POST /energy/optimization/plan
pub async fn create_optimization_plan(
    State(state): State<AppState>,
    Json(request): Json<OptimizationPlanRequest>,
) -> StorageResult<Json<ApiResponse<OptimizationPlan>>> {
    info!("Creating energy optimization plan: {}", request.plan_name);

    let plan = OptimizationPlan {
        id: Uuid::new_v4(),
        plan_name: request.plan_name.clone(),
        description: request.description.clone(),
        optimization_targets: request.optimization_targets.clone(),
        current_baseline: BaselineMetrics {
            total_consumption_kwh: 89456.0,
            monthly_cost_usd: 13456.40,
            efficiency_score: 0.74,
            carbon_footprint_kg: 44720.0,
        },
        projected_improvements: ProjectedImprovements {
            consumption_reduction_percentage: 12.5,
            cost_savings_usd_monthly: 1682.05,
            efficiency_improvement_percentage: 8.3,
            carbon_reduction_kg_monthly: 5590.0,
            payback_period_months: 14.2,
        },
        optimization_actions: vec![
            OptimizationAction {
                id: Uuid::new_v4(),
                action_type: "equipment_upgrade".to_string(),
                title: "Replace aging freezer compressors".to_string(),
                description: "Upgrade to high-efficiency variable-speed compressors".to_string(),
                priority: "high".to_string(),
                estimated_savings_kwh_monthly: 2340.0,
                estimated_cost_usd: 15600.0, 
                implementation_effort: "medium".to_string(),
                payback_months: 8.5,
                prerequisites: vec!["Schedule maintenance window".to_string()],
            },
            OptimizationAction {
                id: Uuid::new_v4(),
                action_type: "control_optimization".to_string(),
                title: "Implement smart scheduling".to_string(),
                description: "Optimize equipment operation based on demand patterns".to_string(),
                priority: "medium".to_string(),
                estimated_savings_kwh_monthly: 1560.0,
                estimated_cost_usd: 2800.0,
                implementation_effort: "low".to_string(),
                payback_months: 2.1,
                prerequisites: vec!["Software configuration".to_string()],
            },
        ],
        implementation_timeline: vec![
            "Week 1-2: Smart scheduling implementation".to_string(),
            "Week 3-4: Equipment procurement".to_string(),
            "Week 5-8: Compressor upgrades".to_string(),
            "Week 9-12: Performance validation".to_string(),
        ],
        risk_assessment: PlanRiskAssessment {
            technical_risk: "low".to_string(),
            financial_risk: "medium".to_string(),
            operational_risk: "low".to_string(),
            mitigation_strategies: vec![
                "Phased implementation to minimize disruption".to_string(),
                "Backup equipment during maintenance".to_string(),
            ],
        },
        created_at: Utc::now(),
        created_by: request.created_by.clone(),
        status: "draft".to_string(),
    };

    Ok(Json(ApiResponse::success(plan)))
}

/// Get equipment energy profiles
/// GET /energy/equipment
pub async fn get_equipment_profiles(
    State(state): State<AppState>,
    Query(query): Query<EquipmentQuery>,
) -> StorageResult<Json<ApiResponse<PaginatedResponse<EquipmentEnergyProfile>>>> {
    info!("Getting equipment energy profiles");

    let profiles = vec![
        EquipmentEnergyProfile {
            id: Uuid::new_v4(),
            equipment_name: "Ultra-Low Freezer ULF-001".to_string(),
            equipment_type: "ultra_low_freezer".to_string(),
            model: "Thermo Fisher TSX -86°C".to_string(),
            location: "Zone A - Position 1".to_string(),
            rated_power_kw: 3.2,
            actual_power_kw: 2.8,
            efficiency_percentage: 87.5,
            annual_consumption_kwh: 24528.0,
            annual_cost_usd: 3679.20,
            operating_hours: 8760.0,
            temperature_range: (-86.0, -70.0),
            current_temperature: -79.8,
            load_factor: 0.85,
            maintenance_status: "up_to_date".to_string(),
            age_years: 3.5,
            performance_trend: "stable".to_string(),
            optimization_potential: OptimizationPotential {
                energy_savings_kwh_annual: 2450.0,
                cost_savings_usd_annual: 367.50,
                improvement_actions: vec![
                    "Regular defrost cycle optimization".to_string(),
                    "Insulation integrity check".to_string(),
                ],
            },
        },
        EquipmentEnergyProfile {
            id: Uuid::new_v4(),
            equipment_name: "HVAC System Main".to_string(),
            equipment_type: "hvac".to_string(),
            model: "Carrier 30GX Chiller".to_string(),
            location: "Mechanical Room".to_string(),
            rated_power_kw: 15.0,
            actual_power_kw: 12.4,
            efficiency_percentage: 82.3,
            annual_consumption_kwh: 108576.0,
            annual_cost_usd: 16286.40,
            operating_hours: 8760.0,
            temperature_range: (18.0, 26.0),
            current_temperature: 21.8,
            load_factor: 0.83,
            maintenance_status: "due_soon".to_string(),
            age_years: 7.2,
            performance_trend: "declining".to_string(),
            optimization_potential: OptimizationPotential {
                energy_savings_kwh_annual: 15000.0,
                cost_savings_usd_annual: 2250.0,
                improvement_actions: vec![
                    "Variable frequency drive installation".to_string(),
                    "Scheduled maintenance completion".to_string(),
                ],
            },
        },
    ];

    let response = PaginatedResponse {
        data: profiles,
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

/// Control equipment power settings
/// POST /energy/equipment/:equipment_id/control
pub async fn control_equipment_power(
    State(state): State<AppState>,
    Path(equipment_id): Path<Uuid>,
    Json(request): Json<PowerControlRequest>,
) -> StorageResult<Json<ApiResponse<PowerControlResult>>> {
    info!("Controlling equipment power for: {}", equipment_id);

    // Validate control request
    validate_power_control_request(&request)?;

    let result = PowerControlResult {
        equipment_id,
        control_action: request.action.clone(),
        previous_state: EquipmentState {
            power_level_percentage: 85.0,
            temperature_setpoint: -80.0,
            operating_mode: "normal".to_string(),
            status: "operational".to_string(),
        },
        new_state: EquipmentState {
            power_level_percentage: request.power_level_percentage.unwrap_or(85.0),
            temperature_setpoint: request.temperature_setpoint.unwrap_or(-80.0),
            operating_mode: request.operating_mode.clone().unwrap_or("normal".to_string()),
            status: match request.action.as_str() {
                "power_on" => "starting",
                "power_off" => "shutting_down",
                "adjust_power" => "adjusting",
                _ => "operational",
            }.to_string(),
        },
        executed_at: Utc::now(),
        estimated_completion: Utc::now() + Duration::minutes(5),
        safety_checks_passed: true,
        power_impact_kw: calculate_power_impact(&request),
        estimated_savings_kwh_daily: request.estimated_savings_kwh_daily.unwrap_or(0.0),
    };

    Ok(Json(ApiResponse::success(result)))
}

/// Get energy analytics and insights
/// GET /energy/analytics
pub async fn get_energy_analytics(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsQuery>,
) -> StorageResult<Json<ApiResponse<EnergyAnalytics>>> {
    info!("Getting energy analytics");

    let analytics = EnergyAnalytics {
        analysis_period_days: query.period_days.unwrap_or(30),
        total_consumption_kwh: 89456.0,
        average_daily_consumption_kwh: 2982.0,
        peak_demand_kw: 156.7,
        load_factor: 0.80,
        consumption_trends: ConsumptionTrends {
            daily_pattern: generate_daily_pattern(),
            weekly_pattern: generate_weekly_pattern(),
            seasonal_variation_percentage: 8.5,
            trend_direction: "stable".to_string(),
        },
        efficiency_analysis: EfficiencyAnalysis {
            current_pue: 1.42,
            industry_benchmark_pue: 1.6,
            efficiency_ranking: "above_average".to_string(),
            improvement_opportunities: vec![
                "Cooling system optimization".to_string(),
                "Equipment right-sizing".to_string(),
            ],
        },
        cost_analysis: CostAnalysis {
            total_cost_usd: 13456.40,
            cost_per_kwh_usd: 0.12,
            demand_charges_percentage: 15.5,
            peak_vs_offpeak_ratio: 1.88,
            cost_optimization_potential_usd: 1682.05,
        },
        environmental_impact: EnvironmentalImpact {
            carbon_footprint_kg_co2: 44720.0,
            renewable_energy_percentage: 15.8,
            carbon_intensity_kg_co2_per_kwh: 0.5,
            sustainability_score: 0.68,
        },
        benchmarking: EnergyBenchmarking {
            facility_type: "laboratory_storage".to_string(),
            industry_average_kwh_per_sqft: 25.6,
            facility_kwh_per_sqft: 22.3,
            performance_percentile: 78,
            comparison_facilities: 156,
        },
        recommendations: vec![
            EnergyRecommendation {
                id: Uuid::new_v4(),
                category: "equipment_optimization".to_string(),
                title: "Implement variable speed drives".to_string(),
                impact: "high".to_string(),
                savings_kwh_annual: 15000.0,
                cost_savings_usd_annual: 2250.0,
                implementation_cost_usd: 12000.0,
                payback_months: 6.4,
            }
        ],
        generated_at: Utc::now(),
    };

    Ok(Json(ApiResponse::success(analytics)))
}

/// Configure energy management settings
/// POST /energy/settings
pub async fn configure_energy_settings(
    State(state): State<AppState>,
    Json(request): Json<EnergySettingsRequest>,
) -> StorageResult<Json<ApiResponse<EnergySettings>>> {
    info!("Configuring energy management settings");

    let settings = EnergySettings {
        id: Uuid::new_v4(),
        facility_id: request.facility_id,
        monitoring_enabled: request.monitoring_enabled.unwrap_or(true),
        alert_thresholds: AlertThresholds {
            high_consumption_percentage: request.high_consumption_threshold.unwrap_or(115.0),
            peak_demand_kw: request.peak_demand_threshold.unwrap_or(150.0),
            efficiency_drop_percentage: request.efficiency_threshold.unwrap_or(10.0),
            cost_spike_percentage: request.cost_spike_threshold.unwrap_or(125.0),
        },
        optimization_preferences: OptimizationPreferences {
            priority: request.optimization_priority.unwrap_or("balanced".to_string()),
            auto_optimization_enabled: request.auto_optimization.unwrap_or(false),
            maintenance_window_hours: request.maintenance_windows.unwrap_or_default(),
            risk_tolerance: request.risk_tolerance.unwrap_or("medium".to_string()),
        },
        reporting_settings: ReportingSettings {
            frequency: request.report_frequency.unwrap_or("weekly".to_string()),
            recipients: request.report_recipients.unwrap_or_default(),
            metrics_to_include: request.metrics_to_include.unwrap_or_else(|| vec![
                "consumption".to_string(),
                "cost".to_string(), 
                "efficiency".to_string(),
            ]),
        },
        carbon_tracking: CarbonTrackingSettings {
            enabled: request.carbon_tracking_enabled.unwrap_or(true),
            emission_factor_kg_co2_per_kwh: request.emission_factor.unwrap_or(0.5),
            renewable_percentage: request.renewable_percentage.unwrap_or(15.8),
            carbon_offset_programs: request.carbon_offset_programs.unwrap_or_default(),
        },
        updated_at: Utc::now(),
        updated_by: request.updated_by.clone(),
    };

    Ok(Json(ApiResponse::success(settings)))
}

/// Get carbon footprint report
/// GET /energy/carbon-footprint
pub async fn get_carbon_footprint(
    State(state): State<AppState>,
    Query(query): Query<CarbonFootprintQuery>,
) -> StorageResult<Json<ApiResponse<CarbonFootprintReport>>> {
    info!("Getting carbon footprint report");

    let report = CarbonFootprintReport {
        reporting_period: query.period.as_deref().unwrap_or("monthly").to_string(),
        start_date: Utc::now() - Duration::days(30),
        end_date: Utc::now(),
        total_emissions_kg_co2: 44720.0,
        energy_consumption_kwh: 89456.0,
        emission_factor_kg_co2_per_kwh: 0.5,
        renewable_energy_kwh: 14135.0,
        renewable_percentage: 15.8,
        emissions_breakdown: EmissionsBreakdown {
            electricity_grid: 37862.4,
            backup_generators: 1245.6,
            other_sources: 612.0,
        },
        scope_emissions: ScopeEmissions {
            scope_1_kg_co2: 1857.6,    // Direct emissions
            scope_2_kg_co2: 42250.4,   // Indirect electricity
            scope_3_kg_co2: 612.0,     // Other indirect
        },
        comparison_data: EmissionsComparison {
            previous_period_kg_co2: 46200.0,
            change_percentage: -3.2,
            trend: "improving".to_string(),
            benchmark_kg_co2: 50000.0,
            performance_vs_benchmark: "better".to_string(),
        },
        reduction_initiatives: vec![
            CarbonReductionInitiative {
                name: "LED Lighting Upgrade".to_string(),
                reduction_kg_co2_annual: 2400.0,
                implementation_date: Utc::now() - Duration::days(60),
                status: "completed".to_string(),
            },
            CarbonReductionInitiative {
                name: "Solar Panel Installation".to_string(),
                reduction_kg_co2_annual: 15000.0,
                implementation_date: Utc::now() + Duration::days(90),
                status: "planned".to_string(),
            },
        ],
        sustainability_metrics: SustainabilityMetrics {
            sustainability_score: 0.68,
            green_energy_percentage: 15.8,
            carbon_intensity_improvement: 3.2,
            renewable_energy_certificates: 50,
        },
        generated_at: Utc::now(),
    };

    Ok(Json(ApiResponse::success(report)))
}

// Helper functions
fn validate_power_control_request(request: &PowerControlRequest) -> StorageResult<()> {
    match request.action.as_str() {
        "power_on" | "power_off" | "adjust_power" | "emergency_shutdown" => Ok(()),
        _ => Err(StorageError::Validation(format!("Invalid control action: {}", request.action))),
    }
}

fn calculate_power_impact(request: &PowerControlRequest) -> f64 {
    match request.action.as_str() {
        "power_off" => -2.8,
        "power_on" => 2.8,
        "adjust_power" => {
            if let Some(level) = request.power_level_percentage {
                (level - 85.0) / 100.0 * 2.8
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

fn generate_consumption_data(time_period: &str, granularity: &str) -> Vec<ConsumptionDataPoint> {
    // Mock data generation - in production would query actual data
    vec![
        ConsumptionDataPoint {
            timestamp: Utc::now() - Duration::hours(1),
            consumption_kwh: 125.8,
            cost_usd: 18.87,
            efficiency_score: 0.89,
        }
    ]
}

fn generate_daily_pattern() -> Vec<HourlyConsumption> {
    (0..24).map(|hour| HourlyConsumption {
        hour,
        average_consumption_kw: 115.0 + (hour as f64 - 12.0).abs() * 2.5,
        peak_consumption_kw: 125.0 + (hour as f64 - 12.0).abs() * 3.0,
    }).collect()
}

fn generate_weekly_pattern() -> Vec<DailyConsumption> {
    ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"]
        .iter()
        .enumerate()
        .map(|(i, day)| DailyConsumption {
            day_of_week: day.to_string(),
            average_consumption_kwh: 2850.0 + (i as f64 - 3.0).abs() * 50.0,
            peak_consumption_kw: 140.0 + (i as f64 - 3.0).abs() * 5.0,
        })
        .collect()
}

// Request/Response structures
#[derive(Debug, Deserialize)]
pub struct RealtimeQuery {
    pub include_equipment: Option<bool>,
    pub include_zones: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub time_period: Option<String>,
    pub granularity: Option<String>,
    pub equipment_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct OptimizationPlanRequest {
    pub plan_name: String,
    pub description: String,
    pub optimization_targets: Vec<String>,
    pub created_by: String,
}

#[derive(Debug, Deserialize)]
pub struct EquipmentQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub equipment_type: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PowerControlRequest {
    pub action: String,
    pub power_level_percentage: Option<f64>,
    pub temperature_setpoint: Option<f64>,
    pub operating_mode: Option<String>,
    pub estimated_savings_kwh_daily: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub period_days: Option<i32>,
    pub include_benchmarking: Option<bool>,
    pub include_recommendations: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct EnergySettingsRequest {
    pub facility_id: Uuid,
    pub monitoring_enabled: Option<bool>,
    pub high_consumption_threshold: Option<f64>,
    pub peak_demand_threshold: Option<f64>,
    pub efficiency_threshold: Option<f64>,
    pub cost_spike_threshold: Option<f64>,
    pub optimization_priority: Option<String>,
    pub auto_optimization: Option<bool>,
    pub maintenance_windows: Option<Vec<String>>,
    pub risk_tolerance: Option<String>,
    pub report_frequency: Option<String>,
    pub report_recipients: Option<Vec<String>>,
    pub metrics_to_include: Option<Vec<String>>,
    pub carbon_tracking_enabled: Option<bool>,
    pub emission_factor: Option<f64>,
    pub renewable_percentage: Option<f64>,
    pub carbon_offset_programs: Option<Vec<String>>,
    pub updated_by: String,
}

#[derive(Debug, Deserialize)]
pub struct CarbonFootprintQuery {
    pub period: Option<String>,
    pub include_breakdown: Option<bool>,
    pub include_initiatives: Option<bool>,
}

// Data structures
#[derive(Debug, Serialize)]
pub struct EnergyOverview {
    pub facility_id: Uuid,
    pub facility_name: String,
    pub current_consumption_kw: f64,
    pub daily_consumption_kwh: f64,
    pub monthly_consumption_kwh: f64,
    pub baseline_consumption_kw: f64,
    pub efficiency_percentage: f64,
    pub cost_usd_daily: f64,
    pub cost_usd_monthly: f64,
    pub carbon_footprint_kg_co2: f64,
    pub energy_breakdown: EnergyBreakdown,
    pub efficiency_metrics: EfficiencyMetrics,
    pub alerts: Vec<EnergyAlert>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct EnergyBreakdown {
    pub cooling_systems: f64,
    pub lighting: f64,
    pub equipment: f64,
    pub hvac: f64,
    pub other: f64,
}

#[derive(Debug, Serialize)]
pub struct EfficiencyMetrics {
    pub power_usage_effectiveness: f64,
    pub cooling_efficiency_cop: f64,
    pub equipment_utilization: f64,
    pub standby_power_percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct EnergyAlert {
    pub id: Uuid,
    pub alert_type: String,
    pub equipment: String,
    pub message: String,
    pub severity: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct RealtimeEnergyMetrics {
    pub timestamp: DateTime<Utc>,
    pub total_power_kw: f64,
    pub voltage_v: f64,
    pub current_a: f64,
    pub power_factor: f64,
    pub frequency_hz: f64,
    pub equipment_metrics: Vec<EquipmentMetric>,
    pub zone_consumption: Vec<ZoneEnergyData>,
    pub grid_data: GridData,
}

#[derive(Debug, Serialize)]
pub struct EquipmentMetric {
    pub equipment_id: Uuid,
    pub equipment_name: String,
    pub equipment_type: String,
    pub power_consumption_kw: f64,
    pub efficiency_percentage: f64,
    pub temperature_setpoint: f64,
    pub actual_temperature: f64,
    pub status: String,
    pub runtime_hours: f64,
    pub maintenance_due_hours: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ZoneEnergyData {
    pub zone_id: String,
    pub zone_name: String,
    pub power_consumption_kw: f64,
    pub target_temperature: f64,
    pub actual_temperature: f64,
    pub occupancy_percentage: f64,
    pub efficiency_score: f64,
}

#[derive(Debug, Serialize)]
pub struct GridData {
    pub grid_frequency_hz: f64,
    pub voltage_stability: f64,
    pub power_quality_score: f64,
    pub renewable_percentage: f64,
    pub peak_demand_kw: f64,
    pub off_peak_rate_usd_kwh: f64,
    pub peak_rate_usd_kwh: f64,
}

#[derive(Debug, Serialize)]
pub struct EnergyHistory {
    pub time_period: String,
    pub granularity: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub total_consumption_kwh: f64,
    pub average_power_kw: f64,
    pub peak_power_kw: f64,
    pub minimum_power_kw: f64,
    pub cost_breakdown: CostBreakdown,
    pub consumption_data: Vec<ConsumptionDataPoint>,
    pub efficiency_trends: EfficiencyTrends,
    pub peak_demand_events: Vec<PeakDemandEvent>,
    pub carbon_footprint: CarbonFootprint,
}

#[derive(Debug, Serialize)]
pub struct CostBreakdown {
    pub energy_cost_usd: f64,
    pub demand_charges_usd: f64,
    pub taxes_fees_usd: f64,
    pub total_cost_usd: f64,
}

#[derive(Debug, Serialize)]
pub struct ConsumptionDataPoint {
    pub timestamp: DateTime<Utc>,
    pub consumption_kwh: f64,
    pub cost_usd: f64,
    pub efficiency_score: f64,
}

#[derive(Debug, Serialize)]
pub struct EfficiencyTrends {
    pub average_pue: f64,
    pub best_pue: f64,
    pub worst_pue: f64,
    pub trend_direction: String,
    pub efficiency_improvement_percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct PeakDemandEvent {
    pub timestamp: DateTime<Utc>,
    pub peak_power_kw: f64,
    pub duration_minutes: i32,
    pub cause: String,
    pub cost_impact_usd: f64,
}

#[derive(Debug, Serialize)]
pub struct CarbonFootprint {
    pub total_co2_kg: f64,
    pub co2_per_kwh: f64,
    pub renewable_offset_kg: f64,
    pub net_carbon_footprint_kg: f64,
}

#[derive(Debug, Serialize)]
pub struct OptimizationPlan {
    pub id: Uuid,
    pub plan_name: String,
    pub description: String,
    pub optimization_targets: Vec<String>,
    pub current_baseline: BaselineMetrics,
    pub projected_improvements: ProjectedImprovements,
    pub optimization_actions: Vec<OptimizationAction>,
    pub implementation_timeline: Vec<String>,
    pub risk_assessment: PlanRiskAssessment,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct BaselineMetrics {
    pub total_consumption_kwh: f64,
    pub monthly_cost_usd: f64,
    pub efficiency_score: f64,
    pub carbon_footprint_kg: f64,
}

#[derive(Debug, Serialize)]
pub struct ProjectedImprovements {
    pub consumption_reduction_percentage: f64,
    pub cost_savings_usd_monthly: f64,
    pub efficiency_improvement_percentage: f64,
    pub carbon_reduction_kg_monthly: f64,
    pub payback_period_months: f64,
}

#[derive(Debug, Serialize)]
pub struct OptimizationAction {
    pub id: Uuid,
    pub action_type: String,
    pub title: String,
    pub description: String,
    pub priority: String,
    pub estimated_savings_kwh_monthly: f64,
    pub estimated_cost_usd: f64,
    pub implementation_effort: String,
    pub payback_months: f64,
    pub prerequisites: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PlanRiskAssessment {
    pub technical_risk: String,
    pub financial_risk: String,
    pub operational_risk: String,
    pub mitigation_strategies: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct EquipmentEnergyProfile {
    pub id: Uuid,
    pub equipment_name: String,
    pub equipment_type: String,
    pub model: String,
    pub location: String,
    pub rated_power_kw: f64,
    pub actual_power_kw: f64,
    pub efficiency_percentage: f64,
    pub annual_consumption_kwh: f64,
    pub annual_cost_usd: f64,
    pub operating_hours: f64,
    pub temperature_range: (f64, f64),
    pub current_temperature: f64,
    pub load_factor: f64,
    pub maintenance_status: String,
    pub age_years: f64,
    pub performance_trend: String,
    pub optimization_potential: OptimizationPotential,
}

#[derive(Debug, Serialize)]
pub struct OptimizationPotential {
    pub energy_savings_kwh_annual: f64,
    pub cost_savings_usd_annual: f64,
    pub improvement_actions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PowerControlResult {
    pub equipment_id: Uuid,
    pub control_action: String,
    pub previous_state: EquipmentState,
    pub new_state: EquipmentState,
    pub executed_at: DateTime<Utc>,
    pub estimated_completion: DateTime<Utc>,
    pub safety_checks_passed: bool,
    pub power_impact_kw: f64,
    pub estimated_savings_kwh_daily: f64,
}

#[derive(Debug, Serialize)]
pub struct EquipmentState {
    pub power_level_percentage: f64,
    pub temperature_setpoint: f64,
    pub operating_mode: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct EnergyAnalytics {
    pub analysis_period_days: i32,
    pub total_consumption_kwh: f64,
    pub average_daily_consumption_kwh: f64,
    pub peak_demand_kw: f64,
    pub load_factor: f64,
    pub consumption_trends: ConsumptionTrends,
    pub efficiency_analysis: EfficiencyAnalysis,
    pub cost_analysis: CostAnalysis,
    pub environmental_impact: EnvironmentalImpact,
    pub benchmarking: EnergyBenchmarking,
    pub recommendations: Vec<EnergyRecommendation>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ConsumptionTrends {
    pub daily_pattern: Vec<HourlyConsumption>,
    pub weekly_pattern: Vec<DailyConsumption>,
    pub seasonal_variation_percentage: f64,
    pub trend_direction: String,
}

#[derive(Debug, Serialize)]
pub struct HourlyConsumption {
    pub hour: i32,
    pub average_consumption_kw: f64,
    pub peak_consumption_kw: f64,
}

#[derive(Debug, Serialize)]
pub struct DailyConsumption {
    pub day_of_week: String,
    pub average_consumption_kwh: f64,
    pub peak_consumption_kw: f64,
}

#[derive(Debug, Serialize)]
pub struct EfficiencyAnalysis {
    pub current_pue: f64,
    pub industry_benchmark_pue: f64,
    pub efficiency_ranking: String,
    pub improvement_opportunities: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CostAnalysis {
    pub total_cost_usd: f64,
    pub cost_per_kwh_usd: f64,
    pub demand_charges_percentage: f64,
    pub peak_vs_offpeak_ratio: f64,
    pub cost_optimization_potential_usd: f64,
}

#[derive(Debug, Serialize)]
pub struct EnvironmentalImpact {
    pub carbon_footprint_kg_co2: f64,
    pub renewable_energy_percentage: f64,
    pub carbon_intensity_kg_co2_per_kwh: f64,
    pub sustainability_score: f64,
}

#[derive(Debug, Serialize)]
pub struct EnergyBenchmarking {
    pub facility_type: String,
    pub industry_average_kwh_per_sqft: f64,
    pub facility_kwh_per_sqft: f64,
    pub performance_percentile: i32,
    pub comparison_facilities: i32,
}

#[derive(Debug, Serialize)]
pub struct EnergyRecommendation {
    pub id: Uuid,
    pub category: String,
    pub title: String,
    pub impact: String,
    pub savings_kwh_annual: f64,
    pub cost_savings_usd_annual: f64,
    pub implementation_cost_usd: f64,
    pub payback_months: f64,
}

#[derive(Debug, Serialize)]
pub struct EnergySettings {
    pub id: Uuid,
    pub facility_id: Uuid,
    pub monitoring_enabled: bool,
    pub alert_thresholds: AlertThresholds,
    pub optimization_preferences: OptimizationPreferences,
    pub reporting_settings: ReportingSettings,
    pub carbon_tracking: CarbonTrackingSettings,
    pub updated_at: DateTime<Utc>,
    pub updated_by: String,
}

#[derive(Debug, Serialize)]
pub struct AlertThresholds {
    pub high_consumption_percentage: f64,
    pub peak_demand_kw: f64,
    pub efficiency_drop_percentage: f64,
    pub cost_spike_percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct OptimizationPreferences {
    pub priority: String,
    pub auto_optimization_enabled: bool,
    pub maintenance_window_hours: Vec<String>,
    pub risk_tolerance: String,
}

#[derive(Debug, Serialize)]
pub struct ReportingSettings {
    pub frequency: String,
    pub recipients: Vec<String>,
    pub metrics_to_include: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CarbonTrackingSettings {
    pub enabled: bool,
    pub emission_factor_kg_co2_per_kwh: f64,
    pub renewable_percentage: f64,
    pub carbon_offset_programs: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CarbonFootprintReport {
    pub reporting_period: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub total_emissions_kg_co2: f64,
    pub energy_consumption_kwh: f64,
    pub emission_factor_kg_co2_per_kwh: f64,
    pub renewable_energy_kwh: f64,
    pub renewable_percentage: f64,
    pub emissions_breakdown: EmissionsBreakdown,
    pub scope_emissions: ScopeEmissions,
    pub comparison_data: EmissionsComparison,
    pub reduction_initiatives: Vec<CarbonReductionInitiative>,
    pub sustainability_metrics: SustainabilityMetrics,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct EmissionsBreakdown {
    pub electricity_grid: f64,
    pub backup_generators: f64,
    pub other_sources: f64,
}

#[derive(Debug, Serialize)]
pub struct ScopeEmissions {
    pub scope_1_kg_co2: f64,
    pub scope_2_kg_co2: f64,
    pub scope_3_kg_co2: f64,
}

#[derive(Debug, Serialize)]
pub struct EmissionsComparison {
    pub previous_period_kg_co2: f64,
    pub change_percentage: f64,
    pub trend: String,
    pub benchmark_kg_co2: f64,
    pub performance_vs_benchmark: String,
}

#[derive(Debug, Serialize)]
pub struct CarbonReductionInitiative {
    pub name: String,
    pub reduction_kg_co2_annual: f64,
    pub implementation_date: DateTime<Utc>,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct SustainabilityMetrics {
    pub sustainability_score: f64,
    pub green_energy_percentage: f64,
    pub carbon_intensity_improvement: f64,
    pub renewable_energy_certificates: i32,
} 
