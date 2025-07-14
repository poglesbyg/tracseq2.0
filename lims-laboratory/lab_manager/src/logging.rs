use serde_json::json;
use std::io::Write;
use std::net::TcpStream;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// ELK Stack integration configuration
pub struct ElkLogger {
    logstash_host: String,
    logstash_port: u16,
    service_name: String,
}

impl ElkLogger {
    pub fn new(logstash_host: &str, logstash_port: u16, service_name: &str) -> Self {
        Self {
            logstash_host: logstash_host.to_string(),
            logstash_port,
            service_name: service_name.to_string(),
        }
    }

    /// Send structured log to Logstash via TCP
    pub fn send_log(&self, level: &str, message: &str, metadata: Option<serde_json::Value>) {
        let log_entry = json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "level": level,
            "service": self.service_name,
            "message": message,
            "metadata": metadata.unwrap_or(json!({})),
            "source": "rust-service"
        });

        if let Ok(mut stream) = TcpStream::connect(format!("{}:{}", self.logstash_host, self.logstash_port)) {
            let log_line = format!("{}\n", log_entry.to_string());
            if let Err(e) = stream.write_all(log_line.as_bytes()) {
                eprintln!("Failed to send log to Logstash: {}", e);
            }
        } else {
            eprintln!("Failed to connect to Logstash at {}:{}", self.logstash_host, self.logstash_port);
        }
    }

    /// Log a laboratory operation
    pub fn log_lab_operation(&self, operation: &str, sample_id: Option<&str>, job_id: Option<&str>, status: &str) {
        let metadata = json!({
            "operation": operation,
            "sample_id": sample_id,
            "job_id": job_id,
            "status": status,
            "category": "laboratory"
        });

        self.send_log("info", &format!("Laboratory operation: {}", operation), Some(metadata));
    }

    /// Log a sequencing run event
    pub fn log_sequencing_run(&self, run_id: &str, status: &str, details: Option<serde_json::Value>) {
        let metadata = json!({
            "sequencing_run_id": run_id,
            "status": status,
            "details": details.unwrap_or(json!({})),
            "category": "sequencing"
        });

        self.send_log("info", &format!("Sequencing run {} status: {}", run_id, status), Some(metadata));
    }

    /// Log an error with context
    pub fn log_error(&self, error: &str, context: Option<serde_json::Value>) {
        let metadata = json!({
            "error": error,
            "context": context.unwrap_or(json!({})),
            "category": "error"
        });

        self.send_log("error", error, Some(metadata));
    }
}

/// Initialize tracing with ELK integration
pub fn init_tracing_with_elk(service_name: &str) -> ElkLogger {
    // Initialize standard tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create ELK logger
    let elk_logger = ElkLogger::new("localhost", 5000, service_name);

    // Send startup log
    elk_logger.send_log("info", &format!("{} service started", service_name), Some(json!({
        "event": "service_startup",
        "service": service_name
    })));

    info!("Tracing initialized with ELK integration for service: {}", service_name);

    elk_logger
} 