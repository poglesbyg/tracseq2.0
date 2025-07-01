//! Unit tests for dashboard handlers

use dashboard_service::{
    AppState, DashboardData,
    handlers,
};
use crate::test_utils::*;
use axum::extract::State;
use std::sync::Arc;

test_with_mock_services!(
    test_main_dashboard_handler,
    |app_state: &AppState, mock_server: &MockServer| async move {
        // Set up mock responses
        MockServerSetup::setup_healthy_services(mock_server).await;
        MockServerSetup::setup_sample_service_mocks(mock_server).await;
        MockServerSetup::setup_storage_service_mocks(mock_server).await;
        
        let state = Arc::new(app_state.clone());
        let response = handlers::dashboard::get_main_dashboard(State(state)).await;
        
        let json = response.0;
        assert_eq!(json["dashboard"], "main");
        assert!(json["sections"].is_object());
        assert!(json["sections"]["overview"].is_object());
        
        // Verify overview metrics
        let overview = &json["sections"]["overview"];
        assert!(overview["total_samples"].is_number());
        assert!(overview["active_sequencing"].is_number());
        assert!(overview["storage_utilization"].is_number());
        assert_eq!(overview["system_health"], "operational");
    }
);

test_with_mock_services!(
    test_system_metrics_handler,
    |app_state: &AppState, _mock_server: &MockServer| async move {
        let state = Arc::new(app_state.clone());
        let response = handlers::metrics::get_system_metrics(State(state)).await;
        
        let json = response.0;
        assert!(json["metrics"].is_object());
        
        let metrics = &json["metrics"];
        assert!(metrics["cpu_usage"].is_number());
        assert!(metrics["memory_usage"].is_number());
        assert!(metrics["disk_usage"].is_number());
        assert!(metrics["network_io"].is_object());
        assert!(metrics["network_io"]["incoming"].is_number());
        assert!(metrics["network_io"]["outgoing"].is_number());
    }
);

test_with_mock_services!(
    test_kpis_handler,
    |app_state: &AppState, _mock_server: &MockServer| async move {
        let state = Arc::new(app_state.clone());
        let response = handlers::kpis::get_kpis(State(state)).await;
        
        let json = response.0;
        assert!(json["kpis"].is_object());
        
        let kpis = &json["kpis"];
        
        // Test sample throughput
        assert!(kpis["sample_throughput"].is_object());
        assert!(kpis["sample_throughput"]["daily"].is_number());
        assert!(kpis["sample_throughput"]["weekly"].is_number());
        assert!(kpis["sample_throughput"]["monthly"].is_number());
        
        // Test turnaround time
        assert!(kpis["turnaround_time"].is_object());
        assert!(kpis["turnaround_time"]["average_hours"].is_number());
        assert!(kpis["turnaround_time"]["median_hours"].is_number());
        
        // Test other KPIs
        assert!(kpis["success_rate"].is_number());
        assert!(kpis["cost_per_sample"].is_number());
    }
);

test_with_mock_services!(
    test_service_status_handler,
    |app_state: &AppState, mock_server: &MockServer| async move {
        // Set up healthy service mocks
        MockServerSetup::setup_healthy_services(mock_server).await;
        
        let state = Arc::new(app_state.clone());
        let response = handlers::services::get_service_status(State(state)).await;
        
        let json = response.0;
        assert!(json["services"].is_object());
        
        let services = &json["services"];
        let expected_services = vec!["auth", "sample", "storage", "sequencing", "notification", "rag"];
        
        for service in expected_services {
            assert_eq!(services[service], "healthy");
        }
    }
);

test_with_mock_services!(
    test_lab_sample_metrics,
    |app_state: &AppState, mock_server: &MockServer| async move {
        // Set up sample service mocks
        MockServerSetup::setup_sample_service_mocks(mock_server).await;
        
        let state = Arc::new(app_state.clone());
        let response = handlers::lab::get_sample_metrics(State(state)).await;
        
        let json = response.0;
        assert!(json["sample_metrics"].is_object());
    }
);

test_with_mock_services!(
    test_lab_sequencing_metrics,
    |app_state: &AppState, mock_server: &MockServer| async move {
        // Set up sequencing service mocks
        MockServerSetup::setup_sequencing_service_mocks(mock_server).await;
        
        let state = Arc::new(app_state.clone());
        let response = handlers::lab::get_sequencing_metrics(State(state)).await;
        
        let json = response.0;
        assert!(json["sequencing_metrics"].is_object());
    }
);

test_with_mock_services!(
    test_lab_storage_metrics,
    |app_state: &AppState, mock_server: &MockServer| async move {
        // Set up storage service mocks
        MockServerSetup::setup_storage_service_mocks(mock_server).await;
        
        let state = Arc::new(app_state.clone());
        let response = handlers::lab::get_storage_metrics(State(state)).await;
        
        let json = response.0;
        assert!(json["storage_metrics"].is_object());
    }
);

test_with_mock_services!(
    test_analytics_handler,
    |app_state: &AppState, _mock_server: &MockServer| async move {
        let state = Arc::new(app_state.clone());
        let response = handlers::analytics::get_analytics(State(state)).await;
        
        let json = response.0;
        assert!(json["analytics"].is_object());
    }
);

test_with_mock_services!(
    test_performance_metrics_handler,
    |app_state: &AppState, _mock_server: &MockServer| async move {
        let state = Arc::new(app_state.clone());
        let response = handlers::performance::get_performance_metrics(State(state)).await;
        
        let json = response.0;
        assert!(json["performance"].is_object());
    }
);

test_with_mock_services!(
    test_active_alerts_handler,
    |app_state: &AppState, _mock_server: &MockServer| async move {
        let state = Arc::new(app_state.clone());
        let response = handlers::alerts::get_active_alerts(State(state)).await;
        
        let json = response.0;
        assert!(json["alerts"].is_array());
    }
);

test_with_mock_services!(
    test_usage_stats_handler,
    |app_state: &AppState, _mock_server: &MockServer| async move {
        let state = Arc::new(app_state.clone());
        let response = handlers::usage::get_usage_stats(State(state)).await;
        
        let json = response.0;
        assert!(json["usage"].is_object());
    }
);

test_with_mock_services!(
    test_create_custom_dashboard,
    |app_state: &AppState, _mock_server: &MockServer| async move {
        let state = Arc::new(app_state.clone());
        let dashboard_config = TestDataGenerator::custom_dashboard_config();
        
        let response = handlers::custom::create_custom_dashboard(
            State(state),
            axum::Json(dashboard_config),
        ).await;
        
        let json = response.0;
        assert!(json["id"].is_string());
        assert_eq!(json["created"], true);
    }
);

test_with_mock_services!(
    test_get_custom_dashboard,
    |app_state: &AppState, _mock_server: &MockServer| async move {
        let state = Arc::new(app_state.clone());
        let dashboard_id = "test-dashboard-123";
        
        let response = handlers::custom::get_custom_dashboard(
            State(state),
            axum::extract::Path(dashboard_id.to_string()),
        ).await;
        
        let json = response.0;
        assert_eq!(json["id"], dashboard_id);
        assert!(json["dashboard"].is_object());
    }
);

test_with_mock_services!(
    test_list_available_widgets,
    |app_state: &AppState, _mock_server: &MockServer| async move {
        let state = Arc::new(app_state.clone());
        let response = handlers::widgets::list_available_widgets(State(state)).await;
        
        let json = response.0;
        assert!(json["widgets"].is_array());
    }
);

// Test error handling
test_with_mock_services!(
    test_handler_with_service_failure,
    |app_state: &AppState, mock_server: &MockServer| async move {
        // Don't set up any mocks, causing service calls to fail
        
        let state = Arc::new(app_state.clone());
        // This should handle the error gracefully
        let response = handlers::services::get_service_status(State(state)).await;
        
        let json = response.0;
        assert!(json["services"].is_object());
        // Services should show as unhealthy or unknown when they can't be reached
    }
);

// Test response formats
#[tokio::test]
async fn test_dashboard_response_format() {
    let mock_server = MockServer::start().await;
    let app_state = create_test_app_state(&mock_server.uri()).await;
    
    let state = Arc::new(app_state);
    let response = handlers::dashboard::get_main_dashboard(State(state)).await;
    
    DashboardAssertions::assert_valid_dashboard_response(&response.0);
}

// Test performance
test_with_mock_services!(
    test_handler_performance,
    |app_state: &AppState, mock_server: &MockServer| async move {
        MockServerSetup::setup_healthy_services(mock_server).await;
        
        let state = Arc::new(app_state.clone());
        
        let duration = PerformanceTestUtils::measure_aggregation_time(|| async {
            handlers::dashboard::get_main_dashboard(State(state.clone())).await
        }).await;
        
        // Dashboard should respond within reasonable time
        assert!(
            duration.as_millis() < 1000,
            "Dashboard handler took {:?}, which is too slow",
            duration
        );
    }
);