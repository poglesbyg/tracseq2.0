use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    
    // Read the request
    if let Ok(size) = stream.read(&mut buffer) {
        let request = String::from_utf8_lossy(&buffer[..size]);
        
        // Check if it's a health check request
        let response = if request.contains("GET /health") {
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"healthy\",\"service\":\"reports-service\",\"version\":\"0.1.0-minimal\"}\r\n"
        } else if request.contains("GET /api/reports") {
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"reports\":[],\"message\":\"Reports service is running but minimal\"}\r\n"
        } else {
            "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\n\r\nNot Found\r\n"
        };
        
        let _ = stream.write_all(response.as_bytes());
    }
}

fn main() {
    println!("📊 Reports Service - Starting with health check");
    
    // Get port from environment or use default
    let port = std::env::var("REPORTS_PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .unwrap_or(8000);
    
    println!("🚀 Reports Service listening on 0.0.0.0:{}", port);
    
    // Create TCP listener
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .expect("Failed to bind to port");
    
    // Handle incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream);
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
} 