use reqwest::Client;
use std::error::Error;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 Testing RAG HTTP Client Connectivity");
    println!("======================================");

    let base_url = std::env::var("RAG_SERVICE_URL")
        .unwrap_or_else(|_| "http://host.docker.internal:8000".to_string());

    println!("🎯 Target URL: {}", base_url);
    println!("📊 Environment Variables:");
    println!("   RAG_SERVICE_URL: {:?}", std::env::var("RAG_SERVICE_URL"));
    println!();

    // Test 1: Basic client creation
    println!("📦 Test 1: Creating HTTP client...");
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()?;
    println!("✅ HTTP client created successfully");
    println!();

    // Test 2: Health endpoint
    println!("🏥 Test 2: Testing /health endpoint...");
    let health_url = format!("{}/health", base_url);
    println!("   Making request to: {}", health_url);

    match client.get(&health_url).send().await {
        Ok(response) => {
            println!("✅ Response received!");
            println!("   Status: {}", response.status());
            println!("   Headers: {:?}", response.headers());

            match response.text().await {
                Ok(body) => println!("   Body: {}", body),
                Err(e) => println!("❌ Failed to read response body: {}", e),
            }
        }
        Err(e) => {
            println!("❌ Request failed: {}", e);
            println!("   Error type: {:?}", e);
            if let Some(source) = e.source() {
                println!("   Source: {}", source);
            }
        }
    }
    println!();

    // Test 3: Query endpoint
    println!("🔍 Test 3: Testing /query endpoint...");
    let query_url = format!("{}/query", base_url);
    let request_body = serde_json::json!({
        "query": "test connectivity"
    });

    println!("   Making POST request to: {}", query_url);
    println!("   Request body: {}", request_body);

    match client.post(&query_url).json(&request_body).send().await {
        Ok(response) => {
            println!("✅ Response received!");
            println!("   Status: {}", response.status());
            println!("   Headers: {:?}", response.headers());

            match response.text().await {
                Ok(body) => {
                    println!("   Body length: {} chars", body.len());
                    if body.len() < 500 {
                        println!("   Body: {}", body);
                    } else {
                        println!("   Body (truncated): {}...", &body[..500]);
                    }
                }
                Err(e) => println!("❌ Failed to read response body: {}", e),
            }
        }
        Err(e) => {
            println!("❌ Request failed: {}", e);
            println!("   Error type: {:?}", e);
            if let Some(source) = e.source() {
                println!("   Source: {}", source);
            }
        }
    }
    println!();

    // Test 4: Network connectivity test
    println!("🌐 Test 4: Network connectivity diagnostics...");

    // Test with different URLs
    let test_urls = vec![
        "http://host.docker.internal:8000/health",
        "http://localhost:8000/health",
        "http://127.0.0.1:8000/health",
    ];

    for url in test_urls {
        println!("   Testing: {}", url);
        match client.get(url).timeout(Duration::from_secs(5)).send().await {
            Ok(response) => println!("   ✅ Connected - Status: {}", response.status()),
            Err(e) => println!("   ❌ Failed: {}", e),
        }
    }

    println!();
    println!("🏁 Test completed!");
    Ok(())
}
