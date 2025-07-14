use serde_json::json;
use std::io::Write;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn main() {
    println!("🚀 Starting ELK Integration Test for TracSeq 2.0 Laboratory Management System");
    println!("═══════════════════════════════════════════════════════════════════════════════");

    // Test direct TCP connection to Logstash
    let logstash_host = "localhost";
    let logstash_port = 5000;

    println!("📡 Testing connection to Logstash at {}:{}", logstash_host, logstash_port);

    // Test connection first
    match TcpStream::connect(format!("{}:{}", logstash_host, logstash_port)) {
        Ok(_) => println!("✅ Successfully connected to Logstash"),
        Err(e) => {
            eprintln!("❌ Failed to connect to Logstash: {}", e);
            eprintln!("💡 Make sure the ELK stack is running with: cd infrastructure/logging && ./deploy-simple.sh");
            return;
        }
    }

    println!("\n📊 Sending laboratory workflow test logs...");

    // Test 1: Sample processing logs
    send_laboratory_logs();
    
    // Test 2: Sequencing workflow logs
    send_sequencing_logs();
    
    // Test 3: Quality control logs
    send_qc_logs();
    
    // Test 4: Error handling logs
    send_error_logs();

    // Final completion log
    let completion_log = json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "level": "info",
        "service": "lab_manager_elk_test",
        "message": "ELK integration test completed successfully",
        "metadata": {
            "event": "test_completion",
            "total_test_categories": 4,
            "test_status": "passed"
        },
        "source": "rust-standalone-test"
    });

    send_log_entry(&completion_log);

    println!("\n🎉 ELK integration test completed successfully!");
    println!("🔍 View logs in Kibana:");
    println!("   • URL: http://localhost:5601");
    println!("   • Index: tracseq-logs-*");
    println!("   • Filter: service:lab_manager_elk_test");
    println!("═══════════════════════════════════════════════════════════════════════════════");
}

fn send_laboratory_logs() {
    println!("\n🧪 Testing laboratory sample processing logs...");
    
    let samples = ["SAMPLE-001", "SAMPLE-002", "SAMPLE-003"];
    
    for (i, sample_id) in samples.iter().enumerate() {
        let log = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "level": "info",
            "service": "lab_manager_elk_test",
            "message": format!("Processing sample {}", sample_id),
            "metadata": {
                "category": "laboratory",
                "operation": "sample_processing",
                "sample_id": sample_id,
                "job_id": format!("JOB-{:03}", i + 1),
                "status": "in_progress",
                "processing_stage": "validation",
                "operator": "lab_tech_001"
            },
            "source": "rust-standalone-test"
        });
        
        send_log_entry(&log);
        thread::sleep(Duration::from_millis(100));
    }
}

fn send_sequencing_logs() {
    println!("🧬 Testing sequencing workflow logs...");
    
    let runs = ["RUN-001", "RUN-002"];
    
    for (i, run_id) in runs.iter().enumerate() {
        let log = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "level": "info",
            "service": "lab_manager_elk_test",
            "message": format!("Sequencing run {} started", run_id),
            "metadata": {
                "category": "sequencing",
                "sequencing_run_id": run_id,
                "platform": "NextSeq",
                "read_length": 150,
                "status": "running",
                "estimated_completion": chrono::Utc::now().checked_add_signed(chrono::Duration::hours(8)).unwrap().to_rfc3339(),
                "samples_count": 3 + i
            },
            "source": "rust-standalone-test"
        });
        
        send_log_entry(&log);
        thread::sleep(Duration::from_millis(100));
    }
}

fn send_qc_logs() {
    println!("✅ Testing quality control logs...");
    
    let qc_checks = [
        ("concentration_check", "passed"),
        ("purity_check", "passed"),
        ("fragment_size_check", "warning")
    ];
    
    for (check_type, status) in qc_checks.iter() {
        let log = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "level": if *status == "warning" { "warn" } else { "info" },
            "service": "lab_manager_elk_test",
            "message": format!("QC check {} completed with status: {}", check_type, status),
            "metadata": {
                "category": "quality_control",
                "qc_type": check_type,
                "status": status,
                "sample_id": "SAMPLE-001",
                "threshold_met": *status != "warning",
                "reviewer": "qc_analyst_001"
            },
            "source": "rust-standalone-test"
        });
        
        send_log_entry(&log);
        thread::sleep(Duration::from_millis(100));
    }
}

fn send_error_logs() {
    println!("⚠️  Testing error handling logs...");
    
    let errors = [
        ("VALIDATION_ERROR", "Sample concentration below minimum threshold"),
        ("STORAGE_ERROR", "Storage location not available"),
        ("NETWORK_ERROR", "Failed to connect to sequencing platform")
    ];
    
    for (error_code, error_message) in errors.iter() {
        let log = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "level": "error",
            "service": "lab_manager_elk_test",
            "message": format!("Error occurred: {}", error_message),
            "metadata": {
                "category": "error",
                "error_code": error_code,
                "error_message": error_message,
                "context": {
                    "sample_id": "SAMPLE-ERROR-001",
                    "operation": "sample_validation",
                    "retry_count": 2
                }
            },
            "source": "rust-standalone-test"
        });
        
        send_log_entry(&log);
        thread::sleep(Duration::from_millis(100));
    }
}

fn send_log_entry(log_entry: &serde_json::Value) {
    match TcpStream::connect("localhost:5000") {
        Ok(mut stream) => {
            let log_line = format!("{}\n", log_entry.to_string());
            match stream.write_all(log_line.as_bytes()) {
                Ok(_) => print!("✓ "),
                Err(e) => eprintln!("✗ Failed to send log: {}", e),
            }
        }
        Err(e) => eprintln!("✗ Failed to connect: {}", e),
    }
} 