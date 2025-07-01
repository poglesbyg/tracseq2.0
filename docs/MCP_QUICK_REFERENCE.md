# MCP Integration Quick Reference

## What is MCP?

Model Context Protocol (MCP) is a standardized protocol for AI model interactions that provides:
- Unified interface for multiple AI models
- Built-in context management
- Service coordination capabilities
- Progress tracking and monitoring

## Current MCP Status in TracSeq 2.0

### ✅ Already Implemented:
- **Enhanced RAG Service** (`fastmcp_enhanced_rag_server.py`)
- **Laboratory RAG Server** (`fastmcp_rag_server.py`)
- **FastMCP configuration** (`fastmcp_config.py`)
- **Dependencies installed** (`requirements-fastmcp.txt`)

### ❌ Not Yet Integrated:
- Cognitive Assistant (uses basic HTTP)
- Rust microservices
- ML Platform services
- Unified service coordination

## Quick Integration Guide

### 1. Install MCP Dependencies
```bash
pip install fastmcp anthropic openai
```

### 2. Convert a Service to MCP

**Before (HTTP approach):**
```python
response = requests.post("http://ollama:11434/api/generate", 
                       json={"model": "llama3.2", "prompt": query})
```

**After (MCP approach):**
```python
from fastmcp import FastMCP, Context

mcp = FastMCP("Service Name")

@mcp.tool
async def process_query(query: str, ctx: Context) -> str:
    response = await ctx.sample(
        messages=query,
        model_preferences=["claude-3", "gpt-4", "llama3.2"]
    )
    return str(response)
```

### 3. Key MCP Features to Use

#### Progress Reporting
```python
await ctx.report_progress(0.0, "Starting")
# ... do work ...
await ctx.report_progress(0.5, "Halfway done")
# ... more work ...
await ctx.report_progress(1.0, "Complete")
```

#### Service Coordination
```python
# Call another MCP service
result = await ctx.call_tool("other_service.tool_name", params)
```

#### Dynamic Resources
```python
@mcp.resource("service://status")
async def get_status(ctx: Context) -> str:
    return f"Current status: {get_current_metrics()}"
```

#### Error Handling
```python
await ctx.info("Processing started")
await ctx.error("Something went wrong")
await ctx.warning("Potential issue detected")
```

## Benefits for TracSeq 2.0

| Feature | Without MCP | With MCP |
|---------|------------|----------|
| AI Models | Fixed model per service | Automatic model selection |
| Error Handling | Manual try/catch | Built-in with retry |
| Service Calls | Custom HTTP/REST | Standardized MCP tools |
| Testing | Must run all services | In-memory testing |
| Progress | Custom implementation | Native progress tracking |
| Context | Manual management | Automatic preservation |

## Next Steps

1. **Immediate**: Test existing MCP services
   ```bash
   python lims-ai/enhanced_rag_service/fastmcp_enhanced_rag_server.py --http
   ```

2. **This Week**: Migrate Cognitive Assistant
   - Replace HTTP calls with MCP
   - Add conversation context
   - Implement model preferences

3. **Next Month**: Full integration
   - Create MCP proxy server
   - Add Rust MCP bridge
   - Implement service coordination

## Testing MCP Integration

Run the example to see benefits:
```bash
python scripts/mcp-integration-example.py
```

Run integration tests:
```bash
python scripts/test-ai-integration.sh
```

## Resources

- **Full Strategy**: `/docs/MCP_INTEGRATION_STRATEGY.md`
- **FastMCP Docs**: https://github.com/jlowin/fastmcp
- **MCP Spec**: https://modelcontextprotocol.io

---

*MCP integration will make TracSeq 2.0 more intelligent, coordinated, and maintainable!* 