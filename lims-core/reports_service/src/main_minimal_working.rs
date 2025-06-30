use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Simple print for logging
    println!("📊 Starting Reports Service - Minimal Version");

    // Get port from environment or use default
    let port = std::env::var("REPORTS_PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .unwrap_or(8000);

    // Create a simple HTTP response
    let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"healthy\",\"service\":\"reports-service\",\"version\":\"0.1.0-minimal\"}\r\n";
    
    println!("🚀 Reports Service (Minimal) listening on 0.0.0.0:{}", port);
    
    // Create a basic TCP listener
    let listener = std::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port))).unwrap();
    
    // Handle connections
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            use std::io::Write;
            let _ = stream.write_all(response.as_bytes());
        }
    }
} 