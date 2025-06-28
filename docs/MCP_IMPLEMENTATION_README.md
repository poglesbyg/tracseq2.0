# MCP Integration Implementation Guide for TracSeq 2.0

## Overview

This guide provides step-by-step instructions for implementing the Model Context Protocol (MCP) integration into your TracSeq 2.0 laboratory management system. The implementation transforms your existing microservices architecture into an AI-driven platform with standardized communication between AI agents and laboratory services.

## What You'll Get

- **Standardized AI Integration**: MCP servers for all major services (Sample, RAG, Storage, Transaction, QA/QC)
- **Intelligent Laboratory Assistant**: AI agent that can process documents, create samples, assign storage, and run QC
- **Multi-Agent Orchestration**: Framework for coordinating multiple specialized AI agents
- **Enterprise-Grade Security**: OAuth 2.1, role-based access control, and audit logging
- **Monitoring & Observability**: Comprehensive dashboards and metrics

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    AI Agent Orchestrator                       │
│                 (Claude, GPT-4, Custom Agents)                │
└─────────────────────┬───────────────────────────────────────────┘
                      │ MCP Protocol (JSON-RPC 2.0)
┌─────────────────────┴───────────────────────────────────────────┐
│                    MCP Registry & Gateway                      │
│         Discovery: localhost:9000 | Gateway: localhost:9001    │
└─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┘
      │     │     │     │     │     │     │     │     │     │
   :8081  :8000  :8082  :8088  :8085  :8084  :8087  :8083  :8086  :8090
   Sample  RAG   Stor   Txn   QAQC  Auth  Temp  Seq   Noti  Agent
   MCP    MCP    MCP    MCP    MCP   MCP   MCP   MCP   MCP   API
```

## Prerequisites

- Docker and Docker Compose
- Rust (for MCP server development)
- Python 3.9+ (for the AI agent)
- API keys for Anthropic Claude and/or OpenAI
- TracSeq 2.0 existing services running

## Quick Start

### 1. Clone and Setup

```bash
# Clone your TracSeq repository
cd /path/to/tracseq-2.0

# Create MCP infrastructure directory
mkdir -p mcp_infrastructure
cd mcp_infrastructure

# Copy the provided implementation files
cp /path/to/provided/files/* .
```

### 2. Environment Configuration

Create `.env` file:

```bash
# AI API Keys
ANTHROPIC_API_KEY=your_anthropic_key_here
OPENAI_API_KEY=your_openai_key_here

# Security
JWT_SECRET=your_jwt_secret_here
OAUTH_ISSUER=http://localhost:8080

# Database
DATABASE_URL=postgresql://tracseq:tracseq@localhost:5432

# MCP Configuration
MCP_REGISTRY_URL=http://localhost:9000
MCP_GATEWAY_URL=http://localhost:9001
CONFIDENCE_THRESHOLD=0.7
```

### 3. Deploy MCP Infrastructure

```bash
# Start the MCP infrastructure
docker-compose -f docker-compose.mcp.yml up -d

# Check health status
curl http://localhost:9000/health  # Registry
curl http://localhost:9001/health  # Gateway
curl http://localhost:8081/mcp/health  # Sample MCP Server
```

### 4. Test the Laboratory Assistant Agent

```python
import asyncio
from mcp_infrastructure.laboratory_assistant_agent import LaboratoryAssistantAgent, AgentConfig

async def test_agent():
    config = AgentConfig(
        anthropic_api_key="your-key",
        mcp_endpoints={
            'sample_service': 'http://localhost:8081/mcp',
            'rag_service': 'http://localhost:8000/mcp',
            'storage_service': 'http://localhost:8082/mcp',
            'transaction_service': 'http://localhost:8088/mcp',
            'qaqc_service': 'http://localhost:8085/mcp'
        }
    )
    
    agent = LaboratoryAssistantAgent(config)
    
    # Test document processing
    result = await agent.process_laboratory_submission(
        document_path="test_documents/sample_submission.pdf"
    )
    
    print(f"Success: {result.success}")
    print(f"Samples created: {result.data.get('samples_created', {}).get('total_created', 0)}")

asyncio.run(test_agent())
```

## Implementation Phases

### Phase 1: Core MCP Infrastructure (Week 1-2)

#### 1.1 MCP Registry Service

```bash
cd mcp_infrastructure
mkdir mcp-registry
cd mcp-registry

# Create Dockerfile
cat > Dockerfile << 'EOF'
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl
COPY --from=builder /app/target/release/mcp-registry /usr/local/bin/
EXPOSE 9000
CMD ["mcp-registry"]
EOF

# Create Cargo.toml
cat > Cargo.toml << 'EOF'
[package]
name = "mcp-registry"
version = "1.0.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls"] }
redis = "0.24"
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"
EOF
```

#### 1.2 Sample Service MCP Server

```bash
cd ../sample-service-mcp

# Use the provided sample_service_mcp_server.rs
cp ../sample_service_mcp_server.rs src/lib.rs

# Create main.rs
cat > src/main.rs << 'EOF'
use axum::{Router, routing::post, extract::State, Json};
use mcp_infrastructure::sample_service_mcp_server::SampleMcpServer;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Initialize MCP server
    let sample_service = Arc::new(/* initialize your sample service */);
    let mcp_server = SampleMcpServer::new(sample_service);
    mcp_server.initialize().await.unwrap();
    
    // Start HTTP server
    let app = Router::new()
        .route("/mcp/tools/:tool_name", post(handle_tool_call))
        .route("/mcp/resources/:resource_name", post(handle_resource_request))
        .route("/mcp/prompts/:prompt_name", post(handle_prompt_request))
        .with_state(Arc::new(mcp_server));
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_tool_call(/* implementation */) { /* ... */ }
async fn handle_resource_request(/* implementation */) { /* ... */ }
async fn handle_prompt_request(/* implementation */) { /* ... */ }
EOF
```

### Phase 2: Service Integration (Week 3-4)

#### 2.1 RAG Service MCP Integration

```python
# rag-service-mcp/mcp_server.py
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import json
from typing import Dict, Any
import sys
import os

# Add the existing RAG service to the path
sys.path.append('/app/rag_service')
from rag_orchestrator import LabSubmissionRAG

app = FastAPI()
rag_system = LabSubmissionRAG()

class McpRequest(BaseModel):
    method: str
    params: Dict[str, Any]

@app.post("/mcp/tools/process_document")
async def process_document(request: McpRequest):
    try:
        file_path = request.params.get('file_path')
        confidence_threshold = request.params.get('confidence_threshold', 0.7)
        
        result = await rag_system.process_document(file_path)
        
        return {
            "success": result.success,
            "extracted_samples": result.submission.samples if result.success else [],
            "confidence_score": result.confidence_score,
            "storage_requirements": result.submission.storage_requirements if result.success else {},
            "warnings": result.warnings or []
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/mcp/tools/semantic_search")
async def semantic_search(request: McpRequest):
    query = request.params.get('query')
    limit = request.params.get('limit', 10)
    
    results = await rag_system.query_submissions(query)
    
    return {
        "success": True,
        "results": results,
        "query": query
    }

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
```

#### 2.2 Storage Service MCP Integration

```rust
// storage-service-mcp/src/main.rs
use axum::{Router, routing::post, extract::State, Json};
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Clone)]
struct StorageMcpServer {
    storage_service: Arc<dyn StorageService>,
}

impl StorageMcpServer {
    async fn optimize_storage_assignment(&self, params: Value) -> Result<Value, String> {
        let sample_ids: Vec<String> = serde_json::from_value(params["sample_ids"].clone())
            .map_err(|e| format!("Invalid sample_ids: {}", e))?;
        
        let requirements = params.get("requirements").cloned().unwrap_or(json!({}));
        let priority = params.get("priority").and_then(|v| v.as_str()).unwrap_or("efficiency");
        
        // Call your existing storage optimization logic
        let assignments = self.storage_service.optimize_assignments(
            sample_ids,
            requirements,
            priority
        ).await?;
        
        Ok(json!({
            "success": true,
            "assignments": assignments,
            "optimization_strategy": priority
        }))
    }
}

#[tokio::main]
async fn main() {
    let storage_service = Arc::new(/* your storage service implementation */);
    let mcp_server = StorageMcpServer { storage_service };
    
    let app = Router::new()
        .route("/mcp/tools/optimize_storage_assignment", post(handle_optimize_storage))
        .with_state(Arc::new(mcp_server));
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8082").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### Phase 3: AI Agent Development (Week 5-6)

#### 3.1 Deploy Laboratory Assistant Agent

```bash
cd lab-assistant-agent

# Use the provided laboratory_assistant_agent.py
cp ../laboratory_assistant_agent.py app.py

# Create requirements.txt
cat > requirements.txt << 'EOF'
anthropic>=0.7.0
httpx>=0.24.0
pydantic>=2.0.0
fastapi>=0.100.0
uvicorn>=0.22.0
redis>=4.5.0
asyncio-mqtt>=0.13.0
python-multipart>=0.0.6
EOF

# Create Dockerfile
cat > Dockerfile << 'EOF'
FROM python:3.11-slim

WORKDIR /app
COPY requirements.txt .
RUN pip install -r requirements.txt

COPY . .

EXPOSE 8090
CMD ["python", "app.py"]
EOF
```

#### 3.2 Agent API Interface

```python
# lab-assistant-agent/api.py
from fastapi import FastAPI, HTTPException, UploadFile, File
from laboratory_assistant_agent import LaboratoryAssistantAgent, AgentConfig
import os

app = FastAPI(title="Laboratory Assistant Agent API")

# Initialize agent
config = AgentConfig(
    anthropic_api_key=os.getenv("ANTHROPIC_API_KEY"),
    mcp_endpoints={
        'sample_service': 'http://sample-service-mcp:8081/mcp',
        'rag_service': 'http://rag-service-mcp:8000/mcp',
        'storage_service': 'http://storage-service-mcp:8082/mcp',
        'transaction_service': 'http://transaction-service-mcp:8088/mcp',
        'qaqc_service': 'http://qaqc-service-mcp:8085/mcp'
    }
)

agent = LaboratoryAssistantAgent(config)

@app.post("/api/process-submission")
async def process_submission(file: UploadFile = File(...)):
    """Process a laboratory submission document"""
    # Save uploaded file
    file_path = f"/tmp/{file.filename}"
    with open(file_path, "wb") as f:
        f.write(await file.read())
    
    # Process with agent
    result = await agent.process_laboratory_submission(file_path)
    
    return {
        "success": result.success,
        "data": result.data,
        "errors": result.errors,
        "warnings": result.warnings,
        "processing_time": result.processing_time
    }

@app.post("/api/quality-control")
async def run_quality_control(sample_ids: list[str]):
    """Run automated quality control on samples"""
    result = await agent.automated_quality_control(sample_ids)
    return result.__dict__

@app.get("/api/search")
async def search_samples(query: str):
    """Intelligent sample search"""
    result = await agent.intelligent_sample_search(query)
    return result.__dict__

@app.get("/api/status")
async def get_agent_status():
    """Get agent health and status"""
    return await agent.get_agent_status()

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8090)
```

## Usage Examples

### Example 1: Automated Laboratory Submission Processing

```python
import requests
import json

# Upload a laboratory submission document
with open("lab_submission.pdf", "rb") as f:
    response = requests.post(
        "http://localhost:8090/api/process-submission",
        files={"file": f}
    )

result = response.json()
print(f"Processing successful: {result['success']}")
print(f"Samples created: {result['data']['samples_created']['total_created']}")
print(f"Processing time: {result['processing_time']:.2f}s")
```

### Example 2: Intelligent Quality Control

```python
# Run QC on specific samples
sample_ids = ["sample-123", "sample-124", "sample-125"]

response = requests.post(
    "http://localhost:8090/api/quality-control",
    json=sample_ids
)

qc_result = response.json()
print(f"QC completed for {qc_result['data']['samples_assessed']} samples")
print(f"Overall quality score: {qc_result['data']['overall_quality_score']:.2f}")
print(f"AI recommendations: {len(qc_result['data']['recommendations'])} items")
```

### Example 3: Natural Language Sample Search

```python
# Search using natural language
response = requests.get(
    "http://localhost:8090/api/search",
    params={"query": "Find all high-quality DNA samples from last week"}
)

search_result = response.json()
print(f"Found {len(search_result['data']['samples'])} samples")
print(f"AI insights: {search_result['data']['ai_insights']['insights']}")
```

## Advanced Configuration

### Security Configuration

```yaml
# config/security.yml
oauth:
  enabled: true
  issuer: "http://auth-service:8080"
  audience: "tracseq-mcp"
  
rate_limiting:
  requests_per_minute: 1000
  burst_size: 100
  
audit_logging:
  enabled: true
  log_level: "INFO"
  retention_days: 90
```

### Performance Tuning

```yaml
# config/performance.yml
mcp_servers:
  concurrency_limit: 100
  timeout_seconds: 30
  retry_attempts: 3
  
caching:
  redis_url: "redis://redis:6379/0"
  cache_ttl_seconds: 300
  
monitoring:
  prometheus_enabled: true
  tracing_enabled: true
  metrics_interval_seconds: 30
```

## Monitoring and Observability

### Health Checks

```bash
# Check all services
curl http://localhost:9000/health    # Registry
curl http://localhost:9001/health    # Gateway
curl http://localhost:8081/mcp/health # Sample MCP
curl http://localhost:8000/mcp/health # RAG MCP
curl http://localhost:8090/health    # Agent API
```

### Metrics Dashboard

Access the MCP dashboard at http://localhost:3000 to monitor:

- Request rates and latencies
- Agent operation success rates
- MCP server health status
- Resource utilization
- Error rates and patterns

## Troubleshooting

### Common Issues

1. **MCP Server Connection Failed**
   ```bash
   # Check if services are running
   docker-compose -f docker-compose.mcp.yml ps
   
   # Check logs
   docker-compose -f docker-compose.mcp.yml logs sample-service-mcp
   ```

2. **Agent Processing Timeout**
   ```bash
   # Increase timeout in config
   export OPERATION_TIMEOUT=600  # 10 minutes
   ```

3. **Low Confidence Scores**
   ```bash
   # Adjust confidence threshold
   export CONFIDENCE_THRESHOLD=0.5
   ```

### Debug Mode

```bash
# Enable debug logging
export LOG_LEVEL=DEBUG
export RUST_LOG=debug

# Restart services
docker-compose -f docker-compose.mcp.yml restart
```

## Production Deployment

### Security Checklist

- [ ] Configure OAuth 2.1 authentication
- [ ] Enable HTTPS/TLS for all endpoints
- [ ] Set up proper firewall rules
- [ ] Configure audit logging
- [ ] Enable rate limiting
- [ ] Set up monitoring alerts

### Scalability Configuration

```yaml
# docker-compose.prod.yml
services:
  sample-service-mcp:
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '1.0'
          memory: 1G
    environment:
      - MAX_CONNECTIONS=100
      - POOL_SIZE=20
```

### Backup and Recovery

```bash
# Backup MCP registry data
docker exec mcp-registry pg_dump -U tracseq mcp_registry > mcp_backup.sql

# Backup agent session data
docker exec redis redis-cli --rdb /data/agent_sessions.rdb
```

## Next Steps

1. **Extend MCP Servers**: Add remaining services (Auth, Template, Sequencing, Notification)
2. **Develop Specialized Agents**: Create domain-specific agents for different laboratory workflows
3. **Implement Multi-Agent Orchestration**: Set up agent-to-agent communication using A2A protocol
4. **Add Advanced Analytics**: Implement predictive models and optimization algorithms
5. **Mobile Integration**: Create mobile apps that use the MCP-enabled agent API

## Support and Resources

- **Documentation**: See `docs/mcp/` directory for detailed API documentation
- **Examples**: Check `examples/mcp/` for additional implementation examples
- **Community**: Join the TracSeq MCP discussion forum
- **Issues**: Report bugs at [TracSeq GitHub Issues]

---

## Implementation Checklist

### Phase 1: Foundation
- [ ] Deploy MCP Registry and Gateway
- [ ] Implement Sample Service MCP Server
- [ ] Test basic MCP communication

### Phase 2: Core Services
- [ ] Implement RAG Service MCP Server
- [ ] Implement Storage Service MCP Server
- [ ] Implement Transaction Service MCP Server
- [ ] Test service-to-service communication

### Phase 3: AI Integration
- [ ] Deploy Laboratory Assistant Agent
- [ ] Test document processing workflow
- [ ] Implement quality control automation
- [ ] Test intelligent search capabilities

### Phase 4: Production Ready
- [ ] Configure security and authentication
- [ ] Set up monitoring and alerting
- [ ] Implement backup and recovery
- [ ] Conduct load testing
- [ ] Train laboratory staff

---

*This implementation guide provides a complete roadmap for integrating MCP into TracSeq 2.0. Follow the phases sequentially for best results, and customize the configuration based on your specific laboratory requirements.*

*Context improved by Giga AI*