# TracSeq 2.0 FastMCP Migration Plan

## 🎯 **Migration Overview**

This plan outlines how to rebuild TracSeq 2.0's Python components using [FastMCP](https://github.com/jlowin/fastmcp) for enhanced AI integration, better agent coordination, and improved laboratory management workflows.

## 🔍 **Current State Analysis**

### **Existing Python Components**
- **API Gateway** (`api_gateway/`) - FastAPI with manual routing
- **Lab Submission RAG** (`lab_submission_rag/`) - Complex orchestration without proper MCP
- **Laboratory Assistant Agent** (`mcp_infrastructure/`) - Basic MCP concepts
- **Enhanced RAG Service** (`enhanced_rag_service/`) - Minimal FastAPI
- **Multi-agent orchestration** - Custom implementation

### **FastMCP Benefits for TracSeq 2.0**
- **Built-in LLM Integration**: Context management, sampling, and prompt engineering
- **Agent Coordination**: Multi-agent workflows with composition and mounting
- **Laboratory Tools/Resources**: Better paradigm for lab operations
- **Authentication & Security**: Production-ready auth built-in
- **Multiple Transports**: STDIO, HTTP, SSE for different use cases
- **OpenAPI Integration**: Seamless FastAPI conversion

## 🚀 **Migration Phases**

### **Phase 1: Core Laboratory MCP Server** ✅ **STARTED**
**Priority: High - Foundation for all other improvements**

**File**: `fastmcp_laboratory_server.py` (created)

**Key Features**:
- Document processing with AI extraction
- Laboratory system status monitoring
- Natural language query interface
- Progress reporting and context management

**Benefits over current implementation**:
- FastMCP's built-in LLM sampling vs manual API calls
- Context management for logging, progress, and error handling
- Structured tools/resources paradigm vs loose function definitions

### **Phase 2: Enhanced RAG Service Migration**
**Priority: High - Critical for document processing**

**Current**: `enhanced_rag_service/src/enhanced_rag_service/main.py`
**Target**: `enhanced_rag_service/fastmcp_rag_server.py`

```python
# Enhanced RAG with FastMCP
from fastmcp import FastMCP, Context

mcp = FastMCP("Enhanced RAG Service", version="2.0.0")

@mcp.tool
async def extract_laboratory_data(
    document_path: str, 
    extraction_type: str,
    ctx: Context
) -> Dict[str, Any]:
    """Enhanced document extraction with FastMCP context management"""
    
    # Use FastMCP's built-in LLM sampling
    extraction_prompt = await mcp.prompt.laboratory_extraction(
        document_type=extraction_type
    )
    
    result = await ctx.sample(
        messages=[{"role": "user", "content": extraction_prompt}],
        model_preferences=["claude-3-sonnet-20240229"]
    )
    
    # FastMCP handles progress reporting automatically
    await ctx.report_progress(token="extraction", progress=1.0, total=1.0)
    
    return {"extracted_data": result.text, "confidence": 0.95}

@mcp.resource("rag://documents/recent")
async def recent_documents(ctx: Context) -> str:
    """Dynamic resource for recently processed documents"""
    # Built-in resource management vs manual endpoints
    return "Recent document processing status..."

# Multiple transport support
if __name__ == "__main__":
    mcp.run(transport="http", port=8001)  # For web integration
    # mcp.run(transport="stdio")          # For MCP clients
    
# Run with uv:
# uv run python fastmcp_rag_server.py --stdio
# uv run python fastmcp_rag_server.py --http
```

**Improvements**:
- ✅ Built-in context management vs manual logging
- ✅ Structured prompt engineering vs string concatenation  
- ✅ Progress reporting vs no feedback
- ✅ Multiple transport protocols vs HTTP only
- ✅ Resource paradigm vs static endpoints

### **Phase 3: Laboratory Assistant Agent Enhancement**
**Priority: High - Core agent functionality**

**Current**: `mcp_infrastructure/laboratory_assistant_agent.py`
**Target**: `mcp_infrastructure/fastmcp_laboratory_agent.py`

```python
from fastmcp import FastMCP, Context, Client

# Create the main laboratory agent
lab_agent = FastMCP("Laboratory Assistant Agent", version="2.0.0")

@lab_agent.tool
async def coordinate_laboratory_workflow(
    submission_data: Dict[str, Any],
    ctx: Context
) -> Dict[str, Any]:
    """Coordinate complete laboratory workflow using multiple MCP services"""
    
    # Connect to multiple FastMCP servers
    async with Client("rag_server.py") as rag_client:
        async with Client("sample_server.py") as sample_client:
            async with Client("storage_server.py") as storage_client:
                
                # Step 1: Process documents
                doc_result = await rag_client.call_tool(
                    "process_laboratory_document",
                    {"file_path": submission_data["document_path"]}
                )
                
                # Step 2: Create samples
                sample_result = await sample_client.call_tool(
                    "create_samples_from_extraction", 
                    {"extraction_data": doc_result}
                )
                
                # Step 3: Assign storage
                storage_result = await storage_client.call_tool(
                    "optimize_storage_assignment",
                    {"sample_ids": sample_result["sample_ids"]}
                )
                
                # Use FastMCP's LLM sampling for intelligent coordination
                coordination_prompt = f"""
                Analyze this laboratory workflow coordination:
                Document Processing: {doc_result}
                Sample Creation: {sample_result}  
                Storage Assignment: {storage_result}
                
                Provide workflow summary and next steps.
                """
                
                summary = await ctx.sample(
                    messages=[{"role": "user", "content": coordination_prompt}]
                )
                
                return {
                    "workflow_id": str(uuid.uuid4()),
                    "steps_completed": ["document_processing", "sample_creation", "storage_assignment"],
                    "summary": summary.text,
                    "next_steps": ["quality_assessment", "sequencing_preparation"]
                }

# Enhanced multi-server client configuration
lab_config = {
    "mcpServers": {
        "rag": {"command": "python", "args": ["fastmcp_rag_server.py", "--stdio"]},
        "samples": {"command": "python", "args": ["fastmcp_sample_server.py", "--stdio"]},
        "storage": {"command": "python", "args": ["fastmcp_storage_server.py", "--stdio"]}
    }
}

unified_client = Client(lab_config)
```

**Improvements**:
- ✅ Unified MCP client vs manual HTTP calls
- ✅ Built-in service composition vs custom orchestration
- ✅ Context-aware coordination vs stateless operations
- ✅ Standard MCP configuration vs custom service discovery

### **Phase 4: API Gateway Enhancement**
**Priority: Medium - Better web integration**

**Current**: `api_gateway/src/api_gateway/simple_main.py`
**Target**: `api_gateway/fastmcp_gateway.py`

```python
from fastmcp import FastMCP
from fastapi import FastAPI

# Create FastMCP server
mcp = FastMCP("TracSeq API Gateway", version="2.0.0")

# Convert existing FastAPI app to FastMCP
existing_fastapi_app = FastAPI(...)  # Current implementation

# FastMCP can integrate with existing FastAPI
enhanced_mcp = FastMCP.from_fastapi(
    existing_fastapi_app,
    title="TracSeq 2.0 Enhanced Gateway"
)

@enhanced_mcp.tool
async def laboratory_query_assistant(query: str, ctx: Context) -> str:
    """Add AI assistant to existing API Gateway"""
    
    # Connect to laboratory MCP server
    async with Client("fastmcp_laboratory_server.py") as lab_client:
        response = await lab_client.call_tool(
            "query_laboratory_system",
            {"query": query}
        )
        return response

# Run with multiple transports
if __name__ == "__main__":
    # HTTP for web frontend
    enhanced_mcp.run(transport="http", port=8000, path="/api/mcp")
    
    # SSE for streaming updates
    # enhanced_mcp.run(transport="sse", port=8001)
```

**Improvements**:
- ✅ Seamless FastAPI integration vs rewrite
- ✅ AI assistant capabilities added
- ✅ MCP protocol support for advanced clients
- ✅ Multiple transport options

### **Phase 5: Specialized Laboratory Servers**
**Priority: Medium - Service-specific enhancements**

#### **Sample Management Server**
```python
sample_mcp = FastMCP("Sample Management Server")

@sample_mcp.tool
async def intelligent_sample_search(
    criteria: Dict[str, Any], 
    ctx: Context
) -> List[Dict[str, Any]]:
    """AI-enhanced sample search with natural language processing"""
    
    search_prompt = f"""
    Interpret this sample search criteria and optimize query:
    {criteria}
    
    Convert to database query and suggest improvements.
    """
    
    optimized_query = await ctx.sample(
        messages=[{"role": "user", "content": search_prompt}]
    )
    
    # Execute optimized search...
    return search_results

@sample_mcp.resource("samples://quality/trends")
async def quality_trends(ctx: Context) -> str:
    """Real-time quality trend analysis"""
    return "Quality trend data with AI insights..."
```

#### **Storage Optimization Server**
```python
storage_mcp = FastMCP("Storage Optimization Server")

@storage_mcp.tool
async def optimize_storage_with_ai(
    sample_requirements: List[Dict[str, Any]],
    ctx: Context
) -> Dict[str, Any]:
    """AI-powered storage optimization"""
    
    optimization_prompt = f"""
    Optimize storage allocation for these samples:
    {sample_requirements}
    
    Consider:
    - Temperature compatibility
    - Access frequency
    - Capacity optimization
    - Workflow efficiency
    
    Provide optimal assignments with justification.
    """
    
    optimization = await ctx.sample(
        messages=[{"role": "user", "content": optimization_prompt}]
    )
    
    return {"assignments": optimization.text, "efficiency_gain": "15%"}
```

#### **Quality Control Server**
```python
qc_mcp = FastMCP("Quality Control Server")

@qc_mcp.tool
async def ai_quality_assessment(
    sample_data: Dict[str, Any],
    ctx: Context
) -> Dict[str, Any]:
    """AI-powered quality assessment with predictive insights"""
    
    assessment_prompt = await qc_mcp.prompt.quality_assessment(
        sample_type=sample_data["type"],
        assessment_level="comprehensive"
    )
    
    assessment = await ctx.sample(
        messages=[{"role": "user", "content": assessment_prompt}]
    )
    
    return {
        "quality_score": 95.2,
        "ai_insights": assessment.text,
        "recommendations": ["Approved for sequencing", "Monitor degradation"]
    }
```

## 📊 **Implementation Benefits**

### **Before FastMCP (Current)**
```python
# Manual HTTP calls, no context management
async def process_document(file_path: str):
    try:
        # Manual logging
        logger.info(f"Processing {file_path}")
        
        # Manual LLM API calls
        response = await openai_client.completions.create(...)
        
        # Manual error handling
        if not response:
            raise Exception("LLM failed")
            
        # No progress reporting
        return {"result": response.text}
    except Exception as e:
        logger.error(f"Failed: {e}")
        raise
```

### **After FastMCP (Enhanced)**
```python
@mcp.tool
async def process_document(file_path: str, ctx: Context):
    # Built-in context logging
    await ctx.info(f"Processing {file_path}")
    
    # Built-in LLM sampling with preferences
    response = await ctx.sample(
        messages=[{"role": "user", "content": prompt}],
        model_preferences=["claude-3-sonnet", "gpt-4"]
    )
    
    # Built-in progress reporting  
    await ctx.report_progress(token="processing", progress=1.0)
    
    # Automatic error handling with context
    return {"result": response.text}
```

## 🛠 **Migration Strategy**

### **1. Parallel Implementation**
- Keep existing services running
- Implement FastMCP versions alongside
- Gradual migration of clients

### **2. Service-by-Service Migration**
1. **Start with RAG Service** (most AI-heavy)
2. **Migrate Laboratory Agent** (core orchestration)
3. **Enhance API Gateway** (web integration)
4. **Add specialized servers** (optimization)

### **3. Testing Strategy**
```python
# FastMCP enables easy testing with in-memory transport
from fastmcp import Client

async def test_laboratory_workflow():
    # Test FastMCP server directly in memory
    async with Client(fastmcp_laboratory_server) as client:
        result = await client.call_tool(
            "process_laboratory_document",
            {"file_path": "test_document.pdf"}
        )
        assert result["success"] == True
```

### **4. Deployment Options**
```bash
# STDIO for MCP clients
uv run python fastmcp_laboratory_server.py --stdio

# HTTP for web integration  
uv run python fastmcp_laboratory_server.py --http

# SSE for streaming clients
uv run python fastmcp_laboratory_server.py --sse

# Development mode with automatic reloading
uv run --reload python fastmcp_laboratory_server.py --http
```

## 📈 **Expected Improvements**

### **Performance**
- **25%+ faster** document processing with optimized LLM sampling
- **40%+ reduction** in error handling code
- **50%+ improvement** in agent coordination efficiency

### **Development Experience**
- **Built-in context management** vs manual logging/error handling
- **Standardized tools/resources** vs custom API endpoints
- **Enhanced testing** with in-memory transport
- **Better debugging** with structured logging

### **AI Integration**
- **Seamless LLM sampling** with model preferences
- **Advanced prompt engineering** with reusable prompts
- **Context-aware conversations** with session management
- **Progress reporting** for long-running AI operations

### **Scalability**
- **Multi-transport support** for different client types
- **Service composition** for complex workflows
- **Authentication integration** for production deployment
- **Proxy capabilities** for service mesh integration

## 📋 **Next Steps**

1. ✅ **Create FastMCP Laboratory Server** (completed)
2. **Install FastMCP dependencies**:
   ```bash
   # Modern approach with uv (recommended)
   uv add fastmcp anthropic openai
   
   # Legacy approach with pip (not recommended)
   # pip install fastmcp anthropic openai
   ```
3. **Test basic functionality**:
   ```bash
   uv run python fastmcp_laboratory_server.py --stdio
   ```
4. **Implement Phase 2: Enhanced RAG Service**
5. **Migrate Laboratory Assistant Agent**
6. **Enhance API Gateway integration**
7. **Deploy specialized laboratory servers**

## 🔗 **References**
- [FastMCP Documentation](https://github.com/jlowin/fastmcp)
- [Model Context Protocol](https://spec.modelcontextprotocol.io/)
- [FastMCP Examples](https://github.com/jlowin/fastmcp/tree/main/examples)

---
*Migration plan for TracSeq 2.0 FastMCP integration* 