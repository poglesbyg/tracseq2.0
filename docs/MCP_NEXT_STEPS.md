# TracSeq 2.0 MCP - Next Steps

Now that your MCP infrastructure is running, here's what you can do next:

## 🎯 Current Status

✅ **Running:**
- MCP Proxy (port 9500) - Central routing hub
- Consul (port 8500) - Service discovery
- PostgreSQL (port 5432) - Database
- Redis (port 6379) - Cache/sessions
- Ollama (port 11434) - Local LLM
- ChromaDB (port 8000) - Vector DB

❌ **Not Yet Running:**
- Cognitive Assistant MCP Service
- Enhanced RAG Service (MCP-enabled)
- MCP Dashboard
- Other MCP-enabled microservices

## 🚀 Next Steps

### 1. Start the MCP-Enabled Services
```bash
# Start cognitive assistant and dashboard
docker-compose -f docker/docker-compose.with-mcp.yml up -d cognitive-assistant-mcp mcp-dashboard enhanced-rag-service

# Check their status
docker ps | grep -E "(cognitive|dashboard|rag)"
```

### 2. Access the MCP Dashboard
Once started, open: http://localhost:7890

The dashboard provides:
- Real-time service monitoring
- Request/response tracking
- Performance metrics
- Service health visualization

### 3. Test Laboratory Workflows
```bash
# Example: Document extraction workflow
curl -X POST http://localhost:9500/mcp/invoke \
  -H "Content-Type: application/json" \
  -d '{
    "service": "rag_service",
    "tool": "extract_laboratory_data",
    "params": {
      "document_path": "/path/to/lab_submission.pdf"
    }
  }'
```

### 4. Develop MCP-Enabled Services
Create new services that register with the MCP proxy:

```python
# Example MCP service registration
import httpx

async def register_my_service():
    async with httpx.AsyncClient() as client:
        await client.post(
            "http://localhost:9500/register",
            json={
                "name": "my_service",
                "endpoint": "http://my-service:8000",
                "transport": "http",
                "capabilities": ["process_samples", "generate_reports"]
            }
        )
```

### 5. Monitor with Consul
View service discovery and health: http://localhost:8500

## 🛠️ Useful Development Commands

```bash
# View all logs
docker-compose -f docker/docker-compose.with-mcp.yml logs -f

# Restart a service
docker-compose -f docker/docker-compose.with-mcp.yml restart mcp-proxy

# Stop everything
docker-compose -f docker/docker-compose.with-mcp.yml down

# Remove volumes (clean slate)
docker-compose -f docker/docker-compose.with-mcp.yml down -v
```

## 📚 Documentation

- MCP Integration Strategy: `docs/MCP_INTEGRATION_STRATEGY.md`
- Quick Reference: `docs/MCP_QUICK_REFERENCE.md`
- API Documentation: Coming soon...

## 🔧 Troubleshooting

### Service Can't Connect
```bash
# Check Docker network
docker network ls | grep tracseq

# Verify service DNS
docker exec mcp-proxy nslookup enhanced-rag-service
```

### Port Conflicts
```bash
# Find what's using a port
lsof -i :9500

# Stop conflicting container
docker stop <container-name>
```

### Health Check Issues
```bash
# Check container health
docker inspect mcp-proxy | jq '.[0].State.Health'

# View health check logs
docker logs mcp-proxy | grep health
```

## 🎨 Example Use Cases

1. **Laboratory Document Processing**
   - Upload PDF submission → Extract data with AI → Store in database
   
2. **Sample Storage Optimization**
   - Predict storage needs → Find optimal locations → Track movement

3. **Intelligent Query System**
   - Ask questions about lab data → Get AI-powered responses

4. **Workflow Automation**
   - Chain multiple services → Transaction support → Automatic rollback

Happy coding with MCP! 🚀 