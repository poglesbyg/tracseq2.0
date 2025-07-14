use std::io::Write;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn main() {
    println!("🚀 Testing ELK Integration for TracSeq 2.0");
    println!("═══════════════════════════════════════════");

    // Test direct TCP connection to Logstash
    let logstash_host = "localhost";
    let logstash_port = 5000;

    println!("📡 Testing connection to Logstash at {}:{}", logstash_host, logstash_port);

    // Test connection first
    match TcpStream::connect(format!("{}:{}", logstash_host, logstash_port)) {
        Ok(_) => println!("✅ Successfully connected to Logstash"),
        Err(e) => {
            eprintln!("❌ Failed to connect to Logstash: {}", e);
            eprintln!("💡 Make sure the ELK stack is running");
            return;
        }
    }

    println!("\n📊 Sending test logs...");

    // Send test logs
    for i in 1..=10 {
        let log_entry = format!(
            r#"{{"timestamp":"2024-07-14T{}:00:00Z","level":"info","service":"lab_manager_test","message":"Test log entry {}","metadata":{{"test_sequence":{},"category":"test","sample_id":"SAMPLE-{:03}","job_id":"JOB-{:03}","status":"{}","source":"rust-simple-test"}}}}"#,
            format!("{:02}", 10 + i),
            i,
            i,
            i,
            i * 10,
            if i % 2 == 0 { "completed" } else { "in_progress" }
        );

        match TcpStream::connect(format!("{}:{}", logstash_host, logstash_port)) {
            Ok(mut stream) => {
                let log_line = format!("{}\n", log_entry);
                match stream.write_all(log_line.as_bytes()) {
                    Ok(_) => print!("✓ "),
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

    // Send laboratory-specific test logs
    println!("\n🧪 Sending laboratory workflow logs...");
    
    let lab_logs = [
        r#"{"timestamp":"2024-07-14T12:00:00Z","level":"info","service":"lab_manager","message":"Sample processing started","metadata":{"category":"laboratory","operation":"sample_processing","sample_id":"SAMPLE-001","job_id":"JOB-001","status":"started","processing_stage":"validation","operator":"lab_tech_001"}}"#,
        r#"{"timestamp":"2024-07-14T12:01:00Z","level":"info","service":"lab_manager","message":"Sequencing run RUN-001 started","metadata":{"category":"sequencing","sequencing_run_id":"RUN-001","platform":"NextSeq","read_length":150,"status":"running","samples_count":3}}"#,
        r#"{"timestamp":"2024-07-14T12:02:00Z","level":"warn","service":"lab_manager","message":"QC check fragment_size_check completed with warning","metadata":{"category":"quality_control","qc_type":"fragment_size_check","status":"warning","sample_id":"SAMPLE-001","threshold_met":false,"reviewer":"qc_analyst_001"}}"#,
        r#"{"timestamp":"2024-07-14T12:03:00Z","level":"error","service":"lab_manager","message":"Storage location not available","metadata":{"category":"error","error_code":"STORAGE_ERROR","error_message":"Storage location not available","context":{"sample_id":"SAMPLE-ERROR-001","operation":"sample_validation","retry_count":2}}}"#,
    ];

    for (i, log) in lab_logs.iter().enumerate() {
        match TcpStream::connect(format!("{}:{}", logstash_host, logstash_port)) {
            Ok(mut stream) => {
                let log_line = format!("{}\n", log);
                match stream.write_all(log_line.as_bytes()) {
                    Ok(_) => print!("✓ "),
                    Err(e) => eprintln!("✗ Failed to send lab log {}: {}", i + 1, e),
                }
            }
            Err(e) => eprintln!("✗ Failed to connect: {}", e),
        }
        thread::sleep(Duration::from_millis(100));
    }

    // Send completion log
    let completion_log = r#"{"timestamp":"2024-07-14T12:04:00Z","level":"info","service":"lab_manager_test","message":"ELK integration test completed successfully","metadata":{"event":"test_completion","total_logs_sent":14,"test_status":"passed"}}"#;

    match TcpStream::connect(format!("{}:{}", logstash_host, logstash_port)) {
        Ok(mut stream) => {
            let log_line = format!("{}\n", completion_log);
            match stream.write_all(log_line.as_bytes()) {
                Ok(_) => println!("✓ Test completed!"),
                Err(e) => eprintln!("✗ Failed to send completion log: {}", e),
            }
        }
        Err(e) => eprintln!("✗ Failed to connect: {}", e),
    }

    println!("\n🎉 ELK integration test completed!");
    println!("🔍 View logs in Kibana:");
    println!("   • URL: http://localhost:5601");
    println!("   • Index: tracseq-logs-*");
    println!("   • Filter: service:lab_manager_test OR service:lab_manager");
    println!("═══════════════════════════════════════════");
} 