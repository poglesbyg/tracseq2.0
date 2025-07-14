use lab_manager::logging::init_tracing_with_elk;
use serde_json::json;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("Starting ELK integration test...");

    // Initialize logging with ELK integration
    let elk_logger = init_tracing_with_elk("lab_manager_test");

    // Test basic logging
    elk_logger.send_log("info", "ELK integration test started", Some(json!({
        "test_id": "elk_integration_001",
        "component": "test_suite"
    })));

    // Test laboratory operation logging
    elk_logger.log_lab_operation(
        "sample_processing",
        Some("SAMPLE-001"),
        Some("JOB-123"),
        "started"
    );

    // Test sequencing run logging
    elk_logger.log_sequencing_run(
        "RUN-456",
        "in_progress",
        Some(json!({
            "platform": "NextSeq",
            "read_length": 150,
            "samples": ["SAMPLE-001", "SAMPLE-002"]
        }))
    );

    // Test error logging
    elk_logger.log_error(
        "Sample validation failed",
        Some(json!({
            "sample_id": "SAMPLE-003",
            "error_code": "VALIDATION_ERROR",
            "details": "Invalid concentration value"
        }))
    );

    // Send multiple test logs
    for i in 1..=5 {
        elk_logger.send_log("info", &format!("Test log entry {}", i), Some(json!({
            "sequence": i,
            "category": "test",
            "timestamp": chrono::Utc::now().to_rfc3339()
        })));
        
        thread::sleep(Duration::from_millis(100));
    }

    // Test completion log
    elk_logger.send_log("info", "ELK integration test completed successfully", Some(json!({
        "test_id": "elk_integration_001",
        "status": "completed",
        "total_logs_sent": 10
    })));

    println!("ELK integration test completed. Check Kibana at http://localhost:5601 for logs.");
} 