use serde_json::json;
use std::io::Write;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn main() {
    println!("Starting simple ELK integration test...");

    // Test direct TCP connection to Logstash
    let logstash_host = "localhost";
    let logstash_port = 5000;

    // Send test logs
    for i in 1..=10 {
        let log_entry = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "level": "info",
            "service": "lab_manager_simple_test",
            "message": format!("Test log entry {}", i),
            "metadata": {
                "test_sequence": i,
                "category": "simple_test",
                "sample_id": format!("SAMPLE-{:03}", i),
                "job_id": format!("JOB-{:03}", i * 10),
                "status": if i % 2 == 0 { "completed" } else { "in_progress" }
            },
            "source": "rust-simple-test"
        });

        match TcpStream::connect(format!("{}:{}", logstash_host, logstash_port)) {
            Ok(mut stream) => {
                let log_line = format!("{}\n", log_entry.to_string());
                match stream.write_all(log_line.as_bytes()) {
                    Ok(_) => println!("✓ Sent log entry {}", i),
                    Err(e) => eprintln!("✗ Failed to send log entry {}: {}", i, e),
                }
            }
            Err(e) => {
                eprintln!("✗ Failed to connect to Logstash: {}", e);
                break;
            }
        }
        
        thread::sleep(Duration::from_millis(200));
    }

    // Send a final completion log
    let completion_log = json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "level": "info",
        "service": "lab_manager_simple_test",
        "message": "Simple ELK integration test completed",
        "metadata": {
            "event": "test_completion",
            "total_logs_sent": 10,
            "test_duration_ms": 2000
        },
        "source": "rust-simple-test"
    });

    if let Ok(mut stream) = TcpStream::connect(format!("{}:{}", logstash_host, logstash_port)) {
        let log_line = format!("{}\n", completion_log.to_string());
        if let Err(e) = stream.write_all(log_line.as_bytes()) {
            eprintln!("Failed to send completion log: {}", e);
        } else {
            println!("✓ Test completed successfully!");
        }
    }

    println!("\nELK integration test finished.");
    println!("Check Kibana at http://localhost:5601 to view the logs.");
    println!("Look for logs with service='lab_manager_simple_test'");
} 