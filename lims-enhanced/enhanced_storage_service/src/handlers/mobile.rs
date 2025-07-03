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
    error::StorageResult,
    models::*,
    AppState,
};

/// Get mobile dashboard overview (optimized for mobile)
/// GET /mobile/dashboard
pub async fn get_mobile_dashboard(
    State(state): State<AppState>,
) -> StorageResult<Json<ApiResponse<MobileDashboard>>> {
    info!("Getting mobile dashboard overview");

    let dashboard = MobileDashboard {
        facility_name: "TracSeq Lab".to_string(),
        last_updated: Utc::now(),
        summary: DashboardSummary {
            total_samples: 1520,
            active_storage_units: 18,
            alerts_count: 2,
            temperature_zones_ok: 4,
            temperature_zones_alert: 1,
            energy_status: "normal".to_string(),
            system_health: "operational".to_string(),
        },
        quick_stats: vec![
            QuickStat {
                label: "Storage Utilization".to_string(),
                value: "72%".to_string(),
                status: "normal".to_string(),
                icon: "storage".to_string(),
            },
            QuickStat {
                label: "Energy Efficiency".to_string(),
                value: "88.7%".to_string(),
                status: "good".to_string(),
                icon: "energy".to_string(),
            },
            QuickStat {
                label: "Temperature Zones".to_string(),
                value: "4/5 OK".to_string(),
                status: "warning".to_string(),
                icon: "temperature".to_string(),
            },
        ],
        recent_alerts: vec![
            MobileAlert {
                id: Uuid::new_v4(),
                title: "Temperature Variance".to_string(),
                message: "Zone A showing +0.3°C variance".to_string(),
                severity: "medium".to_string(),
                timestamp: Utc::now() - Duration::minutes(15),
                acknowledged: false,
                category: "temperature".to_string(),
            },
            MobileAlert {
                id: Uuid::new_v4(),
                title: "High Energy Usage".to_string(),
                message: "Freezer Unit B 15% above baseline".to_string(),
                severity: "low".to_string(),
                timestamp: Utc::now() - Duration::hours(2),
                acknowledged: true,
                category: "energy".to_string(),
            },
        ],
        quick_actions: vec![
            QuickAction {
                id: "view_samples".to_string(),
                title: "View Samples".to_string(),
                description: "Browse and search samples".to_string(),
                icon: "samples".to_string(),
                action_type: "navigate".to_string(),
                params: json!({"route": "/samples"}),
            },
            QuickAction {
                id: "scan_barcode".to_string(),
                title: "Scan Barcode".to_string(),
                description: "Scan sample or equipment barcode".to_string(),
                icon: "scan".to_string(),
                action_type: "camera".to_string(),
                params: json!({"type": "barcode_scanner"}),
            },
        ],
        sync_status: SyncStatus {
            last_sync: Utc::now() - Duration::minutes(2),
            sync_status: "up_to_date".to_string(),
            pending_uploads: 0,
            pending_downloads: 0,
            offline_mode: false,
        },
    };

    Ok(Json(ApiResponse::success(dashboard)))
}

/// Get mobile-optimized sample list
/// GET /mobile/samples
pub async fn get_mobile_samples(
    State(state): State<AppState>,
    Query(query): Query<MobileSampleQuery>,
) -> StorageResult<Json<ApiResponse<MobileSampleList>>> {
    info!("Getting mobile sample list");

    let samples = vec![
        MobileSample {
            id: Uuid::new_v4(),
            barcode: "SAM-2024-001".to_string(),
            sample_type: "DNA".to_string(),
            location: "Zone A, Rack 1, Position 5".to_string(),
            temperature: -80.0,
            status: "stored".to_string(),
            priority: "normal".to_string(),
            submitter: "Dr. Smith".to_string(),
            submitted_date: Utc::now() - Duration::days(3),
            thumbnail_image: None,
            qr_code_data: "SAM-2024-001|DNA|A1-5".to_string(),
        },
        MobileSample {
            id: Uuid::new_v4(),
            barcode: "SAM-2024-002".to_string(),
            sample_type: "RNA".to_string(),
            location: "Zone B, Rack 3, Position 12".to_string(),
            temperature: -80.0,
            status: "processing".to_string(),
            priority: "high".to_string(),
            submitter: "Dr. Johnson".to_string(),
            submitted_date: Utc::now() - Duration::days(1),
            thumbnail_image: None,
            qr_code_data: "SAM-2024-002|RNA|B3-12".to_string(),
        },
    ];

    let sample_list = MobileSampleList {
        samples,
        total_count: 2,
        page: query.page.unwrap_or(1),
        per_page: query.per_page.unwrap_or(20),
        has_more: false,
        filters_applied: query.status.is_some() || query.sample_type.is_some(),
        last_updated: Utc::now(),
    };

    Ok(Json(ApiResponse::success(sample_list)))
}

/// Get sample details by barcode (mobile optimized)
/// GET /mobile/samples/barcode/:barcode
pub async fn get_sample_by_barcode(
    State(state): State<AppState>,
    Path(barcode): Path<String>,
) -> StorageResult<Json<ApiResponse<MobileSampleDetails>>> {
    info!("Getting sample details for barcode: {}", barcode);

    let sample_details = MobileSampleDetails {
        sample: DetailedMobileSample {
            id: Uuid::new_v4(),
            barcode: barcode.clone(),
            sample_type: "DNA".to_string(),
            location: "Zone A, Rack 1, Position 5".to_string(),
            temperature: -80.0,
            status: "stored".to_string(),
            priority: "normal".to_string(),
            submitter: "Dr. Smith".to_string(),
            submitter_contact: "smith@lab.edu".to_string(),
            submitted_date: Utc::now() - Duration::days(3),
            volume_ml: Some(1.5),
            concentration_ng_ul: Some(125.5),
            quality_score: Some(8.7),
            notes: Some("High quality sample for sequencing".to_string()),
            storage_conditions: "Ultra-low freezer, -80°C".to_string(),
            expiry_date: Some(Utc::now() + Duration::days(365)),
            chain_of_custody: vec![
                CustodyEvent {
                    event_type: "submitted".to_string(),
                    timestamp: Utc::now() - Duration::days(3),
                    user: "Dr. Smith".to_string(),
                    location: "Submission Portal".to_string(),
                },
                CustodyEvent {
                    event_type: "received".to_string(),
                    timestamp: Utc::now() - Duration::days(3) + Duration::hours(2),
                    user: "Lab Tech A".to_string(),
                    location: "Receiving Station".to_string(),
                },
                CustodyEvent {
                    event_type: "stored".to_string(),
                    timestamp: Utc::now() - Duration::days(3) + Duration::hours(4),
                    user: "Lab Tech A".to_string(),
                    location: "Zone A, Rack 1, Position 5".to_string(),
                },
            ],
        },
        related_samples: vec![],
        recent_activities: vec![
            ActivityEvent {
                timestamp: Utc::now() - Duration::hours(6),
                activity_type: "temperature_check".to_string(),
                description: "Temperature verified at -79.8°C".to_string(),
                user: "System".to_string(),
            }
        ],
        qr_code_svg: generate_qr_code_svg(&barcode),
    };

    Ok(Json(ApiResponse::success(sample_details)))
}

/// Get mobile alerts and notifications
/// GET /mobile/alerts
pub async fn get_mobile_alerts(
    State(state): State<AppState>,
    Query(query): Query<MobileAlertQuery>,
) -> StorageResult<Json<ApiResponse<MobileAlertList>>> {
    info!("Getting mobile alerts");

    let alerts = vec![
        MobileAlert {
            id: Uuid::new_v4(),
            title: "Temperature Variance Detected".to_string(),
            message: "Zone A showing +0.3°C variance from setpoint. Current: -79.7°C, Target: -80.0°C".to_string(),
            severity: "medium".to_string(),
            timestamp: Utc::now() - Duration::minutes(15),
            acknowledged: false,
            category: "temperature".to_string(),
        },
        MobileAlert {
            id: Uuid::new_v4(),
            title: "Capacity Warning".to_string(),
            message: "Freezer Unit 2 approaching 85% capacity. Consider sample relocation.".to_string(),
            severity: "low".to_string(),
            timestamp: Utc::now() - Duration::hours(1),
            acknowledged: false,
            category: "capacity".to_string(),
        },
        MobileAlert {
            id: Uuid::new_v4(),
            title: "Maintenance Reminder".to_string(),
            message: "HVAC System 1 maintenance due in 3 days".to_string(),
            severity: "info".to_string(),
            timestamp: Utc::now() - Duration::hours(8),
            acknowledged: true,
            category: "maintenance".to_string(),
        },
    ];

    let alert_list = MobileAlertList {
        alerts,
        unread_count: 2,
        total_count: 3,
        categories: vec![
            AlertCategory {
                name: "temperature".to_string(),
                display_name: "Temperature".to_string(),
                count: 1,
                icon: "thermometer".to_string(),
            },
            AlertCategory {
                name: "capacity".to_string(),
                display_name: "Capacity".to_string(),
                count: 1,
                icon: "storage".to_string(),
            },
        ],
        last_updated: Utc::now(),
    };

    Ok(Json(ApiResponse::success(alert_list)))
}

/// Acknowledge mobile alert
/// POST /mobile/alerts/:alert_id/acknowledge
pub async fn acknowledge_alert(
    State(state): State<AppState>,
    Path(alert_id): Path<Uuid>,
    Json(request): Json<AcknowledgeAlertRequest>,
) -> StorageResult<Json<ApiResponse<AlertAcknowledgment>>> {
    info!("Acknowledging alert: {}", alert_id);

    let acknowledgment = AlertAcknowledgment {
        alert_id,
        acknowledged_by: request.user_id,
        acknowledged_at: Utc::now(),
        notes: request.notes,
        status: "acknowledged".to_string(),
    };

    Ok(Json(ApiResponse::success(acknowledgment)))
}

/// Get mobile equipment status
/// GET /mobile/equipment
pub async fn get_mobile_equipment(
    State(state): State<AppState>,
    Query(query): Query<MobileEquipmentQuery>,
) -> StorageResult<Json<ApiResponse<MobileEquipmentList>>> {
    info!("Getting mobile equipment status");

    let equipment = vec![
        MobileEquipment {
            id: Uuid::new_v4(),
            name: "Ultra-Low Freezer A".to_string(),
            equipment_type: "freezer".to_string(),
            location: "Zone A".to_string(),
            status: "operational".to_string(),
            health_score: 95,
            current_temperature: -79.8,
            target_temperature: -80.0,
            capacity_used_percentage: 72,
            energy_consumption_kw: 2.8,
            last_maintenance: Utc::now() - Duration::days(30),
            next_maintenance: Utc::now() + Duration::days(60),
            alerts_count: 0,
            icon: "freezer".to_string(),
        },
        MobileEquipment {
            id: Uuid::new_v4(),
            name: "HVAC System 1".to_string(),
            equipment_type: "hvac".to_string(),
            location: "Mechanical Room".to_string(),
            status: "operational".to_string(),
            health_score: 78,
            current_temperature: 21.8,
            target_temperature: 22.0,
            capacity_used_percentage: 83,
            energy_consumption_kw: 12.4,
            last_maintenance: Utc::now() - Duration::days(85),
            next_maintenance: Utc::now() + Duration::days(5),
            alerts_count: 1,
            icon: "hvac".to_string(),
        },
    ];

    let equipment_list = MobileEquipmentList {
        equipment,
        summary: EquipmentSummary {
            total_equipment: 2,
            operational: 2,
            warning: 0,
            critical: 0,
            offline: 0,
            maintenance_due_soon: 1,
        },
        last_updated: Utc::now(),
    };

    Ok(Json(ApiResponse::success(equipment_list)))
}

/// Get mobile-optimized analytics
/// GET /mobile/analytics
pub async fn get_mobile_analytics(
    State(state): State<AppState>,
    Query(query): Query<MobileAnalyticsQuery>,
) -> StorageResult<Json<ApiResponse<MobileAnalytics>>> {
    info!("Getting mobile analytics");

    let analytics = MobileAnalytics {
        time_period: query.period.as_deref().unwrap_or("24h").to_string(),
        charts: vec![
            MobileChart {
                id: "temperature_trend".to_string(),
                title: "Temperature Trend".to_string(),
                chart_type: "line".to_string(),
                data: json!({
                    "labels": ["12:00", "13:00", "14:00", "15:00", "16:00"],
                    "datasets": [{
                        "label": "Zone A",
                        "data": [-79.8, -79.9, -79.7, -79.8, -79.8]
                    }]
                }),
                unit: "°C".to_string(),
                color: "#2196F3".to_string(),
            },
            MobileChart {
                id: "energy_usage".to_string(),
                title: "Energy Usage".to_string(),
                chart_type: "bar".to_string(),
                data: json!({
                    "labels": ["Freezers", "HVAC", "Lighting", "Other"],
                    "datasets": [{
                        "label": "kW",
                        "data": [45.2, 18.9, 15.3, 8.1]
                    }]
                }),
                unit: "kW".to_string(),
                color: "#4CAF50".to_string(),
            },
        ],
        key_metrics: vec![
            KeyMetric {
                label: "Total Samples".to_string(),
                value: "1,520".to_string(),
                change: "+12".to_string(),
                change_type: "positive".to_string(),
                period: "today".to_string(),
            },
            KeyMetric {
                label: "Energy Efficiency".to_string(),
                value: "88.7%".to_string(),
                change: "+2.1%".to_string(),
                change_type: "positive".to_string(),
                period: "this week".to_string(),
            },
        ],
        insights: vec![
            "Temperature control is stable across all zones".to_string(),
            "Energy consumption is 5% below monthly average".to_string(),
        ],
        generated_at: Utc::now(),
    };

    Ok(Json(ApiResponse::success(analytics)))
}

/// Update user device preferences
/// POST /mobile/preferences
pub async fn update_mobile_preferences(
    State(state): State<AppState>,
    Json(request): Json<MobilePreferencesRequest>,
) -> StorageResult<Json<ApiResponse<MobilePreferences>>> {
    info!("Updating mobile preferences for user: {}", request.user_id);

    let preferences = MobilePreferences {
        user_id: request.user_id,
        device_id: request.device_id.clone(),
        notification_settings: NotificationSettings {
            push_enabled: request.push_notifications.unwrap_or(true),
            alert_types: request.alert_types.unwrap_or_else(|| vec![
                "critical".to_string(),
                "high".to_string(),
                "medium".to_string(),
            ]),
            quiet_hours_start: request.quiet_hours_start,
            quiet_hours_end: request.quiet_hours_end,
            sound_enabled: request.sound_enabled.unwrap_or(true),
            vibration_enabled: request.vibration_enabled.unwrap_or(true),
        },
        display_settings: DisplaySettings {
            theme: request.theme.unwrap_or("system".to_string()),
            dashboard_layout: request.dashboard_layout.unwrap_or("default".to_string()),
            show_thumbnails: request.show_thumbnails.unwrap_or(true),
            compact_mode: request.compact_mode.unwrap_or(false),
            auto_refresh_interval: request.auto_refresh_interval.unwrap_or(300),
        },
        data_settings: DataSettings {
            offline_sync_enabled: request.offline_sync.unwrap_or(true),
            image_quality: request.image_quality.unwrap_or("medium".to_string()),
            auto_download_updates: request.auto_download.unwrap_or(true),
            cache_size_mb: request.cache_size_mb.unwrap_or(100),
        },
        updated_at: Utc::now(),
    };

    Ok(Json(ApiResponse::success(preferences)))
}

/// Get mobile app configuration
/// GET /mobile/config
pub async fn get_mobile_config(
    State(state): State<AppState>,
) -> StorageResult<Json<ApiResponse<MobileConfig>>> {
    info!("Getting mobile app configuration");

    let config = MobileConfig {
        app_version: "2.1.0".to_string(),
        api_version: "v1".to_string(),
        features: MobileFeatures {
            barcode_scanner: true,
            offline_mode: true,
            push_notifications: true,
            camera_integration: true,
            biometric_auth: true,
            real_time_updates: true,
            export_data: true,
            multi_language: false,
        },
        server_endpoints: ServerEndpoints {
            api_base_url: "https://api.tracseq.lab".to_string(),
            websocket_url: "wss://api.tracseq.lab/ws".to_string(),
            file_upload_url: "https://api.tracseq.lab/upload".to_string(),
        },
        limits: MobileLimits {
            max_offline_samples: 1000,
            max_image_size_mb: 10,
            max_cache_size_mb: 500,
            sync_interval_seconds: 300,
            request_timeout_seconds: 30,
        },
        branding: MobileBranding {
            app_name: "TracSeq Mobile".to_string(),
            primary_color: "#2196F3".to_string(),
            secondary_color: "#4CAF50".to_string(),
            logo_url: "https://api.tracseq.lab/logo.png".to_string(),
        },
        last_updated: Utc::now(),
    };

    Ok(Json(ApiResponse::success(config)))
}

/// Sync offline data
/// POST /mobile/sync
pub async fn sync_offline_data(
    State(state): State<AppState>,
    Json(request): Json<SyncRequest>,
) -> StorageResult<Json<ApiResponse<SyncResult>>> {
    info!("Syncing offline data for device: {}", request.device_id);

    let sync_result = SyncResult {
        sync_id: Uuid::new_v4(),
        device_id: request.device_id.clone(),
        sync_type: "full".to_string(),
        started_at: Utc::now(),
        completed_at: Utc::now() + Duration::seconds(5),
        status: "completed".to_string(),
        items_uploaded: request.pending_uploads.len() as i32,
        items_downloaded: 25,
        conflicts_resolved: 0,
        errors: vec![],
        next_sync_recommended: Utc::now() + Duration::minutes(15),
        data_usage_mb: 2.5,
    };

    Ok(Json(ApiResponse::success(sync_result)))
}

// Helper functions
fn generate_qr_code_svg(data: &str) -> String {
    // Mock QR code SVG generation - in production would use actual QR library
    format!(
        r#"<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
            <rect width="100" height="100" fill="white"/>
            <text x="50" y="50" text-anchor="middle" font-size="8">{}</text>
        </svg>"#,
        data
    )
}

// Request/Response structures
#[derive(Debug, Deserialize)]
pub struct MobileSampleQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub status: Option<String>,
    pub sample_type: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MobileAlertQuery {
    pub severity: Option<String>,
    pub category: Option<String>,
    pub acknowledged: Option<bool>,
    pub limit: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct AcknowledgeAlertRequest {
    pub user_id: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MobileEquipmentQuery {
    pub equipment_type: Option<String>,
    pub status: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MobileAnalyticsQuery {
    pub period: Option<String>,
    pub metrics: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct MobilePreferencesRequest {
    pub user_id: String,
    pub device_id: String,
    pub push_notifications: Option<bool>,
    pub alert_types: Option<Vec<String>>,
    pub quiet_hours_start: Option<String>,
    pub quiet_hours_end: Option<String>,
    pub sound_enabled: Option<bool>,
    pub vibration_enabled: Option<bool>,
    pub theme: Option<String>,
    pub dashboard_layout: Option<String>,
    pub show_thumbnails: Option<bool>,
    pub compact_mode: Option<bool>,
    pub auto_refresh_interval: Option<i32>,
    pub offline_sync: Option<bool>,
    pub image_quality: Option<String>,
    pub auto_download: Option<bool>,
    pub cache_size_mb: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    pub device_id: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub pending_uploads: Vec<serde_json::Value>,
    pub sync_type: Option<String>,
}

// Data structures
#[derive(Debug, Serialize)]
pub struct MobileDashboard {
    pub facility_name: String,
    pub last_updated: DateTime<Utc>,
    pub summary: DashboardSummary,
    pub quick_stats: Vec<QuickStat>,
    pub recent_alerts: Vec<MobileAlert>,
    pub quick_actions: Vec<QuickAction>,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Serialize)]
pub struct DashboardSummary {
    pub total_samples: i32,
    pub active_storage_units: i32,
    pub alerts_count: i32,
    pub temperature_zones_ok: i32,
    pub temperature_zones_alert: i32,
    pub energy_status: String,
    pub system_health: String,
}

#[derive(Debug, Serialize)]
pub struct QuickStat {
    pub label: String,
    pub value: String,
    pub status: String,
    pub icon: String,
}

#[derive(Debug, Serialize)]
pub struct MobileAlert {
    pub id: Uuid,
    pub title: String,
    pub message: String,
    pub severity: String,
    pub timestamp: DateTime<Utc>,
    pub acknowledged: bool,
    pub category: String,
}

#[derive(Debug, Serialize)]
pub struct QuickAction {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub action_type: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct SyncStatus {
    pub last_sync: DateTime<Utc>,
    pub sync_status: String,
    pub pending_uploads: i32,
    pub pending_downloads: i32,
    pub offline_mode: bool,
}

#[derive(Debug, Serialize)]
pub struct MobileSampleList {
    pub samples: Vec<MobileSample>,
    pub total_count: i32,
    pub page: i32,
    pub per_page: i32,
    pub has_more: bool,
    pub filters_applied: bool,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MobileSample {
    pub id: Uuid,
    pub barcode: String,
    pub sample_type: String,
    pub location: String,
    pub temperature: f64,
    pub status: String,
    pub priority: String,
    pub submitter: String,
    pub submitted_date: DateTime<Utc>,
    pub thumbnail_image: Option<String>,
    pub qr_code_data: String,
}

#[derive(Debug, Serialize)]
pub struct MobileSampleDetails {
    pub sample: DetailedMobileSample,
    pub related_samples: Vec<MobileSample>,
    pub recent_activities: Vec<ActivityEvent>,
    pub qr_code_svg: String,
}

#[derive(Debug, Serialize)]
pub struct DetailedMobileSample {
    pub id: Uuid,
    pub barcode: String,
    pub sample_type: String,
    pub location: String,
    pub temperature: f64,
    pub status: String,
    pub priority: String,
    pub submitter: String,
    pub submitter_contact: String,
    pub submitted_date: DateTime<Utc>,
    pub volume_ml: Option<f64>,
    pub concentration_ng_ul: Option<f64>,
    pub quality_score: Option<f64>,
    pub notes: Option<String>,
    pub storage_conditions: String,
    pub expiry_date: Option<DateTime<Utc>>,
    pub chain_of_custody: Vec<CustodyEvent>,
}

#[derive(Debug, Serialize)]
pub struct CustodyEvent {
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub user: String,
    pub location: String,
}

#[derive(Debug, Serialize)]
pub struct ActivityEvent {
    pub timestamp: DateTime<Utc>,
    pub activity_type: String,
    pub description: String,
    pub user: String,
}

#[derive(Debug, Serialize)]
pub struct MobileAlertList {
    pub alerts: Vec<MobileAlert>,
    pub unread_count: i32,
    pub total_count: i32,
    pub categories: Vec<AlertCategory>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AlertCategory {
    pub name: String,
    pub display_name: String,
    pub count: i32,
    pub icon: String,
}

#[derive(Debug, Serialize)]
pub struct AlertAcknowledgment {
    pub alert_id: Uuid,
    pub acknowledged_by: String,
    pub acknowledged_at: DateTime<Utc>,
    pub notes: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct MobileEquipmentList {
    pub equipment: Vec<MobileEquipment>,
    pub summary: EquipmentSummary,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MobileEquipment {
    pub id: Uuid,
    pub name: String,
    pub equipment_type: String,
    pub location: String,
    pub status: String,
    pub health_score: i32,
    pub current_temperature: f64,
    pub target_temperature: f64,
    pub capacity_used_percentage: i32,
    pub energy_consumption_kw: f64,
    pub last_maintenance: DateTime<Utc>,
    pub next_maintenance: DateTime<Utc>,
    pub alerts_count: i32,
    pub icon: String,
}

#[derive(Debug, Serialize)]
pub struct EquipmentSummary {
    pub total_equipment: i32,
    pub operational: i32,
    pub warning: i32,
    pub critical: i32,
    pub offline: i32,
    pub maintenance_due_soon: i32,
}

#[derive(Debug, Serialize)]
pub struct MobileAnalytics {
    pub time_period: String,
    pub charts: Vec<MobileChart>,
    pub key_metrics: Vec<KeyMetric>,
    pub insights: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MobileChart {
    pub id: String,
    pub title: String,
    pub chart_type: String,
    pub data: serde_json::Value,
    pub unit: String,
    pub color: String,
}

#[derive(Debug, Serialize)]
pub struct KeyMetric {
    pub label: String,
    pub value: String,
    pub change: String,
    pub change_type: String,
    pub period: String,
}

#[derive(Debug, Serialize)]
pub struct MobilePreferences {
    pub user_id: String,
    pub device_id: String,
    pub notification_settings: NotificationSettings,
    pub display_settings: DisplaySettings,
    pub data_settings: DataSettings,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct NotificationSettings {
    pub push_enabled: bool,
    pub alert_types: Vec<String>,
    pub quiet_hours_start: Option<String>,
    pub quiet_hours_end: Option<String>,
    pub sound_enabled: bool,
    pub vibration_enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct DisplaySettings {
    pub theme: String,
    pub dashboard_layout: String,
    pub show_thumbnails: bool,
    pub compact_mode: bool,
    pub auto_refresh_interval: i32,
}

#[derive(Debug, Serialize)]
pub struct DataSettings {
    pub offline_sync_enabled: bool,
    pub image_quality: String,
    pub auto_download_updates: bool,
    pub cache_size_mb: i32,
}

#[derive(Debug, Serialize)]
pub struct MobileConfig {
    pub app_version: String,
    pub api_version: String,
    pub features: MobileFeatures,
    pub server_endpoints: ServerEndpoints,
    pub limits: MobileLimits,
    pub branding: MobileBranding,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MobileFeatures {
    pub barcode_scanner: bool,
    pub offline_mode: bool,
    pub push_notifications: bool,
    pub camera_integration: bool,
    pub biometric_auth: bool,
    pub real_time_updates: bool,
    pub export_data: bool,
    pub multi_language: bool,
}

#[derive(Debug, Serialize)]
pub struct ServerEndpoints {
    pub api_base_url: String,
    pub websocket_url: String,
    pub file_upload_url: String,
}

#[derive(Debug, Serialize)]
pub struct MobileLimits {
    pub max_offline_samples: i32,
    pub max_image_size_mb: i32,
    pub max_cache_size_mb: i32,
    pub sync_interval_seconds: i32,
    pub request_timeout_seconds: i32,
}

#[derive(Debug, Serialize)]
pub struct MobileBranding {
    pub app_name: String,
    pub primary_color: String,
    pub secondary_color: String,
    pub logo_url: String,
}

#[derive(Debug, Serialize)]
pub struct SyncResult {
    pub sync_id: Uuid,
    pub device_id: String,
    pub sync_type: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub status: String,
    pub items_uploaded: i32,
    pub items_downloaded: i32,
    pub conflicts_resolved: i32,
    pub errors: Vec<String>,
    pub next_sync_recommended: DateTime<Utc>,
    pub data_usage_mb: f64,
}
