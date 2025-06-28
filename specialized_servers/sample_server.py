"""
Sample Management Server - FastMCP Implementation
"""

import logging
import time
import uuid
from datetime import datetime
from typing import Any

from fastmcp import Context, FastMCP
from pydantic import BaseModel, Field

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Initialize FastMCP server
mcp = FastMCP("TracSeq Sample Management Server", version="2.0.0")

# Pydantic models
class IntelligentSearchRequest(BaseModel):
    query: str = Field(description="Natural language search query")
    search_type: str = Field(default="comprehensive", description="Type of search")

class SampleCreationRequest(BaseModel):
    sample_data: dict[str, Any] = Field(description="Sample metadata")
    priority: str = Field(default="normal", description="Priority")

# Global state
sample_state = {
    "total_samples": 1247,
    "active_samples": 89,
    "last_activity": None
}

@mcp.tool
async def intelligent_sample_search(
    request: IntelligentSearchRequest,
    ctx: Context
) -> dict[str, Any]:
    """AI-powered intelligent sample search."""
    await ctx.info(f"Processing search: {request.query}")

    start_time = time.time()

    try:
        # AI query analysis
        query_analysis = await ctx.sample(
            messages=[{
                "role": "user",
                "content": f"""
                Analyze this sample search query: "{request.query}"
                
                Extract search parameters for sample types, quality criteria,
                storage locations, and processing stages.
                """
            }],
            model_preferences=["claude-3-sonnet-20240229"]
        )

        # Mock search results
        mock_samples = [{
            "id": "SMPL-DNA-001",
            "name": "DNA Sample 001",
            "type": "DNA",
            "quality_score": 96.2,
            "status": "Ready"
        }] if "dna" in request.query.lower() else []

        processing_time = time.time() - start_time
        sample_state["last_activity"] = datetime.now().isoformat()

        return {
            "success": True,
            "query": request.query,
            "query_analysis": query_analysis.text,
            "samples_found": len(mock_samples),
            "samples": mock_samples,
            "processing_time": processing_time
        }

    except Exception as e:
        await ctx.error(f"Search failed: {str(e)}")
        return {"success": False, "error": str(e)}

@mcp.tool
async def create_samples_with_ai_optimization(
    request: SampleCreationRequest,
    ctx: Context
) -> dict[str, Any]:
    """Create samples with AI optimization."""
    await ctx.info("Creating samples with AI optimization")

    start_time = time.time()

    try:
        # AI optimization
        optimization = await ctx.sample(
            messages=[{
                "role": "user",
                "content": f"""
                Optimize this sample creation:
                Data: {request.sample_data}
                Priority: {request.priority}
                
                Provide recommendations for naming, storage, and workflow.
                """
            }],
            model_preferences=["claude-3-sonnet-20240229"]
        )

        # Create optimized sample
        sample_id = f"SMPL-{datetime.now().strftime('%Y%m%d')}-{str(uuid.uuid4())[:8]}"

        optimized_sample = {
            "id": sample_id,
            "data": request.sample_data,
            "priority": request.priority,
            "ai_optimization": optimization.text,
            "created_at": datetime.now().isoformat(),
            "status": "Created"
        }

        # Update state
        sample_state["total_samples"] += 1
        sample_state["active_samples"] += 1
        sample_state["last_activity"] = datetime.now().isoformat()

        processing_time = time.time() - start_time

        return {
            "success": True,
            "sample_created": optimized_sample,
            "processing_time": processing_time
        }

    except Exception as e:
        await ctx.error(f"Creation failed: {str(e)}")
        return {"success": False, "error": str(e)}

@mcp.resource("samples://status")
async def sample_status(ctx: Context) -> str:
    """Sample Management Server status."""
    return f"""
# Sample Management Server Status

- **Total Samples**: {sample_state['total_samples']:,}
- **Active Samples**: {sample_state['active_samples']:,}
- **Last Activity**: {sample_state['last_activity'] or 'None'}
- **Status**: Operational

*Updated: {datetime.now().isoformat()}*
    """.strip()

# Main execution
if __name__ == "__main__":
    import sys

    logger.info("Starting Sample Management Server")
    sample_state["last_activity"] = datetime.now().isoformat()

    if len(sys.argv) > 1 and sys.argv[1] == "--http":
        mcp.run(transport="http", port=8010)
    else:
        mcp.run(transport="stdio")
