# MCP Development Guide for TracSeq 2.0

This guide provides practical examples for building MCP-enabled services, implementing real-time communication, orchestrating workflows, and integrating AI capabilities.

## 📁 Example Structure

```
examples/
├── mcp-service-template/
│   └── sample_analyzer_service.py    # MCP-enabled service template
├── mcp-websocket/
│   └── realtime_monitor.py          # WebSocket real-time client
├── mcp-workflows/
│   └── laboratory_workflow_coordinator.py  # Multi-service workflows
├── mcp-ai-integration/
│   └── ollama_laboratory_assistant.py     # Advanced AI integration
└── MCP_DEVELOPMENT_GUIDE.md         # This guide
```

## 1️⃣ Building MCP-Enabled Services

### Quick Start
```python
# Create a new MCP service
from examples.mcp_service_template.sample_analyzer_service import SampleAnalyzerService

# Initialize and run
service = SampleAnalyzerService(service_name="my_analyzer", port=8025)
await service.start()
```

### Key Features
- **Auto-registration**: Services register with MCP proxy on startup
- **Health checks**: Built-in health endpoint at `/health`
- **Tool endpoints**: Expose capabilities as MCP tools
- **AI integration**: Optional Ollama integration for enhanced analysis

### Example Service Structure
```python
class MyMCPService:
    def __init__(self):
        self.mcp_proxy_url = "http://mcp-proxy:9500"
        self.setup_routes()
        
    async def register_with_proxy(self):
        # Auto-register on startup
        
    async def my_tool(self, request):
        # Implement MCP tool logic
```

## 2️⃣ WebSocket Real-Time Communication

### Quick Start
```python
from examples.mcp_websocket.realtime_monitor import MCPRealtimeMonitor

# Connect and monitor
monitor = MCPRealtimeMonitor()
await monitor.connect()
await monitor.subscribe_to_events(["sample_processed", "analysis_complete"])
await monitor.receive_messages()
```

### Real-Time Features
- **Event subscription**: Subscribe to specific event types
- **Bidirectional communication**: Send and receive messages
- **Streaming responses**: Handle streaming data from services
- **Custom handlers**: Register handlers for specific message types

### Common Use Cases
- Monitor sample processing in real-time
- Receive alerts for critical results
- Stream analysis progress
- Interactive dashboards

## 3️⃣ Multi-Service Workflow Orchestration

### Quick Start
```python
from examples.mcp_workflows.laboratory_workflow_coordinator import SampleSubmissionWorkflow

# Execute a complete workflow
workflow = SampleSubmissionWorkflow(document_path="submission.pdf")
result = await workflow.execute()
```

### Workflow Types

#### Sequential Workflow
- Extract → Validate → Process → Store → Notify
- Transaction support with rollback
- Error handling at each step

#### Parallel Workflow
```python
# Process multiple samples simultaneously
workflow = ParallelAnalysisWorkflow(sample_ids=["S001", "S002", "S003"])
results = await workflow.execute()
```

### Key Patterns
- **Transaction logging**: Track all steps for rollback
- **Parallel execution**: Use `asyncio.gather()` for efficiency
- **Error recovery**: Graceful handling with partial results
- **Progress tracking**: Real-time status updates

## 4️⃣ AI Integration with Ollama

### Quick Start
```python
from examples.mcp_ai_integration.ollama_laboratory_assistant import OllamaLaboratoryAssistant

# Initialize AI assistant
assistant = OllamaLaboratoryAssistant()

# Analyze results with context
analysis = await assistant.analyze_results_with_context(
    results=lab_results,
    context=AnalysisContext(sample_type="blood", urgency="stat")
)
```

### AI Capabilities

#### Clinical Analysis
- Interpret test results
- Identify critical values
- Suggest follow-up tests
- Generate clinical reports

#### Workflow Optimization
```python
optimization = await assistant.optimize_workflow(
    pending_samples=samples,
    available_resources=resources
)
```

#### Anomaly Detection
```python
anomalies = await assistant.detect_anomalies(
    current_results=current,
    historical_data=history,
    sensitivity="high"
)
```

#### Interactive Consultation
```python
response = await assistant.interactive_consultation(
    "What does this pattern indicate?",
    context={"test_results": results}
)
```

## 🚀 Putting It All Together

### Complete Example: AI-Enhanced Sample Processing

```python
import asyncio
from datetime import datetime

async def process_laboratory_submission():
    # 1. Start real-time monitoring
    monitor = MCPRealtimeMonitor()
    await monitor.connect()
    await monitor.subscribe_to_events(["workflow_*", "analysis_*"])
    
    # 2. Initialize AI assistant
    ai_assistant = OllamaLaboratoryAssistant()
    
    # 3. Create and execute workflow
    workflow = SampleSubmissionWorkflow("submission.pdf")
    
    # Add AI enhancement to workflow
    workflow.ai_assistant = ai_assistant
    
    # 4. Execute with real-time updates
    asyncio.create_task(monitor.receive_messages())
    result = await workflow.execute()
    
    # 5. Generate AI report
    report = await ai_assistant.generate_clinical_report(
        patient_id="P12345",
        test_results=result["processed_results"]
    )
    
    return report

# Run the complete process
report = await process_laboratory_submission()
```

## 🛠️ Development Tips

### 1. Service Development
- Use environment variables for configuration
- Implement comprehensive error handling
- Add detailed logging for debugging
- Include health checks and metrics

### 2. WebSocket Best Practices
- Implement reconnection logic
- Handle connection timeouts
- Use message queuing for reliability
- Implement heartbeat/ping-pong

### 3. Workflow Design
- Keep workflows modular and reusable
- Implement proper transaction boundaries
- Use parallel execution where possible
- Include comprehensive logging

### 4. AI Integration
- Cache AI responses when appropriate
- Use temperature settings for consistency
- Implement fallback for AI failures
- Validate AI outputs before use

## 📊 Monitoring and Debugging

### Service Metrics
```bash
# Check service health
curl http://localhost:9500/health

# View service registry
curl http://localhost:9500/services

# Monitor WebSocket connections
docker logs mcp-proxy -f | grep WebSocket
```

### Workflow Tracking
```python
# Add workflow tracking
workflow.on_step_complete = lambda step: logger.info(f"Completed: {step}")
workflow.on_error = lambda error: logger.error(f"Error: {error}")
```

### AI Performance
```python
# Monitor AI response times
start = datetime.now()
response = await ai_assistant.analyze_results(...)
duration = (datetime.now() - start).total_seconds()
logger.info(f"AI response time: {duration}s")
```

## 🔗 Next Steps

1. **Deploy Your Service**: Package as Docker container
2. **Add to docker-compose**: Include in `docker-compose.with-mcp.yml`
3. **Register with Proxy**: Auto-registration on startup
4. **Monitor Performance**: Use Consul and metrics endpoints
5. **Iterate and Improve**: Based on real-world usage

## 📚 Additional Resources

- [MCP Integration Strategy](../docs/MCP_INTEGRATION_STRATEGY.md)
- [MCP Quick Reference](../docs/MCP_QUICK_REFERENCE.md)
- [TracSeq Architecture](../docs/ARCHITECTURE.md)
- [Ollama Documentation](https://ollama.ai/docs)

Happy coding with MCP! 🚀 