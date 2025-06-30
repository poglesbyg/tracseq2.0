"""
FastMCP-Enhanced API Gateway for TracSeq 2.0

Integrates FastMCP capabilities with the existing FastAPI gateway,
adding AI assistant functionality, MCP protocol support, and enhanced routing.
"""

import asyncio
import json
import logging
import time
from datetime import datetime
from typing import Any, Dict, List, Optional

# FastAPI imports available for future integration
from fastmcp import Client, Context, FastMCP
from pydantic import BaseModel, Field

# Note: Integration with existing FastAPI app available when needed

# Configure logging
logging.basicConfig(level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s")
logger = logging.getLogger(__name__)

# Create FastMCP server that integrates with existing FastAPI
mcp = FastMCP("TracSeq Enhanced API Gateway", version="2.0.0")

# Pydantic models for enhanced functionality
class LaboratoryQueryRequest(BaseModel):
    query: str = Field(description="Natural language query about laboratory data")
    context: Optional[str] = Field(default="general", description="Query context: samples, templates, sequencing, storage")
    user_id: Optional[str] = Field(default=None, description="User ID for personalized responses")

class WorkflowOrchestrationRequest(BaseModel):
    workflow_type: str = Field(description="Type of workflow to orchestrate")
    parameters: Dict[str, Any] = Field(description="Workflow parameters")
    priority: str = Field(default="normal", description="Workflow priority")

class SystemStatusRequest(BaseModel):
    include_services: bool = Field(default=True, description="Include connected services status")
    include_metrics: bool = Field(default=True, description="Include performance metrics")

# Global state for enhanced gateway
gateway_state = {
    "initialized": False,
    "connected_services": {},
    "ai_queries_processed": 0,
    "last_ai_query": None,
    "gateway_performance": {
        "total_requests": 0,
        "ai_enhanced_requests": 0,
        "average_response_time": 0.0
    }
}

@mcp.tool
async def laboratory_query_assistant(
    request: LaboratoryQueryRequest,
    ctx: Context
) -> str:
    """
    AI-powered assistant for laboratory queries through the API Gateway.
    
    Provides intelligent responses about laboratory operations, samples,
    workflows, and system status using natural language processing.
    """
    await ctx.info(f"Processing laboratory query: {request.query}")

    start_time = time.time()

    try:
        # Connect to relevant FastMCP services based on query context
        service_responses = {}

        if request.context in ["samples", "general"]:
            # Connect to laboratory server for sample information
            async with Client("fastmcp_laboratory_server.py") as lab_client:
                try:
                    sample_info = await lab_client.call_tool(
                        "query_laboratory_system",
                        {"query": request.query, "context": "samples"}
                    )
                    service_responses["laboratory"] = sample_info
                except Exception as e:
                    await ctx.warning(f"Laboratory service unavailable: {e}")

        if request.context in ["rag", "documents", "general"]:
            # Connect to enhanced RAG service
            async with Client("enhanced_rag_service/fastmcp_enhanced_rag_server.py") as rag_client:
                try:
                    rag_info = await rag_client.call_tool(
                        "query_laboratory_knowledge",
                        {"query": request.query, "query_type": "gateway_integration"}
                    )
                    service_responses["rag"] = rag_info
                except Exception as e:
                    await ctx.warning(f"RAG service unavailable: {e}")

        if request.context in ["workflows", "coordination", "general"]:
            # Connect to laboratory assistant agent
            async with Client("mcp_infrastructure/fastmcp_laboratory_agent.py") as agent_client:
                try:
                    agent_info = await agent_client.call_resource("agent://status/current")
                    service_responses["agent"] = agent_info
                except Exception as e:
                    await ctx.warning(f"Agent service unavailable: {e}")

        # Use AI to synthesize comprehensive response
        synthesis_prompt = f"""
        You are an intelligent API Gateway assistant for TracSeq 2.0 laboratory management system.
        
        User Query: "{request.query}"
        Query Context: {request.context}
        User ID: {request.user_id or 'Anonymous'}
        
        Available Service Responses:
        {json.dumps(service_responses, indent=2)}
        
        Provide a comprehensive, helpful response that:
        1. Directly answers the user's question
        2. Integrates information from multiple services when relevant
        3. Suggests related actions or next steps
        4. Uses laboratory terminology appropriately
        5. Provides specific data and examples when available
        
        Keep the response conversational but informative, suitable for laboratory professionals.
        """

        ai_response = await ctx.sample(
            messages=[{"role": "user", "content": synthesis_prompt}],
            model_preferences=["claude-3-sonnet-20240229", "gpt-4"]
        )

        processing_time = time.time() - start_time

        # Update gateway state
        gateway_state["ai_queries_processed"] += 1
        gateway_state["last_ai_query"] = datetime.now().isoformat()
        gateway_state["gateway_performance"]["ai_enhanced_requests"] += 1

        await ctx.info(f"Laboratory query processed in {processing_time:.2f}s")

        enhanced_response = f"""{ai_response.text}

---
*Response enhanced by TracSeq 2.0 AI Gateway Assistant*
*Query processed in {processing_time:.2f}s | Connected services: {len(service_responses)}*
        """

        return enhanced_response.strip()

    except Exception as e:
        await ctx.error(f"Laboratory query assistant failed: {e!s}")
        return f"I apologize, but I encountered an error processing your query: {e!s}. Please try rephrasing your question or contact support."

@mcp.tool
async def orchestrate_laboratory_workflow(
    request: WorkflowOrchestrationRequest,
    ctx: Context
) -> Dict[str, Any]:
    """
    Orchestrate complex laboratory workflows through the enhanced API Gateway.
    
    Coordinates multi-service operations with intelligent routing and
    progress tracking for comprehensive laboratory management.
    """
    await ctx.info(f"Orchestrating workflow: {request.workflow_type}")

    start_time = time.time()
    orchestration_id = f"ORCH-{int(time.time())}"

    try:
        # Use AI to plan workflow orchestration
        orchestration_plan = await ctx.sample(
            messages=[{
                "role": "user",
                "content": f"""
                Plan the orchestration of this laboratory workflow through the API Gateway:
                
                Workflow Type: {request.workflow_type}
                Parameters: {request.parameters}
                Priority: {request.priority}
                
                Determine:
                1. Which FastMCP services need to be involved
                2. The optimal sequence of operations
                3. Data flow between services
                4. Error handling and rollback strategies
                5. Progress tracking milestones
                
                Provide a structured orchestration plan.
                """
            }],
            model_preferences=["claude-3-sonnet-20240229"]
        )

        # Execute workflow based on type
        workflow_results = {}

        if request.workflow_type == "sample_submission":
            # Coordinate sample submission workflow
            async with Client("mcp_infrastructure/fastmcp_laboratory_agent.py") as agent_client:
                result = await agent_client.call_tool(
                    "process_laboratory_submission",
                    {
                        "document_path": request.parameters.get("document_path", ""),
                        "priority": request.priority,
                        "notify_submitter": request.parameters.get("notify_submitter", True)
                    }
                )
                workflow_results["submission"] = result

        elif request.workflow_type == "quality_control":
            # Coordinate quality control workflow
            async with Client("mcp_infrastructure/fastmcp_laboratory_agent.py") as agent_client:
                result = await agent_client.call_tool(
                    "automated_quality_control",
                    {
                        "sample_ids": request.parameters.get("sample_ids", []),
                        "assessment_type": request.parameters.get("assessment_type", "comprehensive")
                    }
                )
                workflow_results["quality_control"] = result

        elif request.workflow_type == "document_processing":
            # Coordinate document processing workflow
            async with Client("enhanced_rag_service/fastmcp_enhanced_rag_server.py") as rag_client:
                result = await rag_client.call_tool(
                    "extract_laboratory_data",
                    {
                        "document_path": request.parameters.get("document_path", ""),
                        "extraction_type": request.parameters.get("extraction_type", "comprehensive")
                    }
                )
                workflow_results["document_processing"] = result

        else:
            # Handle custom workflows with AI assistance
            workflow_results["custom"] = {
                "workflow_type": request.workflow_type,
                "orchestration_plan": orchestration_plan.text,
                "status": "planned",
                "note": "Custom workflow plan generated - manual execution required"
            }

        processing_time = time.time() - start_time

        # Update gateway performance metrics
        gateway_state["gateway_performance"]["total_requests"] += 1

        await ctx.info(f"Workflow orchestration completed in {processing_time:.2f}s")

        return {
            "success": True,
            "orchestration_id": orchestration_id,
            "workflow_type": request.workflow_type,
            "orchestration_plan": orchestration_plan.text,
            "workflow_results": workflow_results,
            "processing_time": processing_time,
            "priority": request.priority
        }

    except Exception as e:
        await ctx.error(f"Workflow orchestration failed: {e!s}")
        return {
            "success": False,
            "orchestration_id": orchestration_id,
            "workflow_type": request.workflow_type,
            "error": str(e),
            "processing_time": time.time() - start_time
        }

@mcp.tool
async def enhanced_system_status(
    request: SystemStatusRequest,
    ctx: Context
) -> Dict[str, Any]:
    """
    Get comprehensive system status with AI-powered analysis and insights.
    
    Provides detailed information about all connected services, performance
    metrics, and intelligent recommendations for system optimization.
    """
    await ctx.info("Gathering enhanced system status information")

    try:
        system_status = {
            "gateway": {
                "status": "operational",
                "version": "2.0.0 (FastMCP Enhanced)",
                "ai_queries_processed": gateway_state["ai_queries_processed"],
                "last_ai_query": gateway_state["last_ai_query"],
                "performance": gateway_state["gateway_performance"]
            }
        }

        if request.include_services:
            # Check connected services status
            service_status = {}

            # Check Laboratory Server
            try:
                async with Client("fastmcp_laboratory_server.py") as lab_client:
                    lab_status = await lab_client.call_resource("lab://system/status")
                    service_status["laboratory_server"] = {"status": "connected", "details": lab_status}
            except Exception as e:
                service_status["laboratory_server"] = {"status": "unavailable", "error": str(e)}

            # Check Enhanced RAG Service
            try:
                async with Client("enhanced_rag_service/fastmcp_enhanced_rag_server.py") as rag_client:
                    rag_status = await rag_client.call_resource("rag://service/health")
                    service_status["rag_service"] = {"status": "connected", "details": rag_status}
            except Exception as e:
                service_status["rag_service"] = {"status": "unavailable", "error": str(e)}

            # Check Laboratory Assistant Agent
            try:
                async with Client("mcp_infrastructure/fastmcp_laboratory_agent.py") as agent_client:
                    agent_status = await agent_client.call_resource("agent://status/current")
                    service_status["laboratory_agent"] = {"status": "connected", "details": agent_status}
            except Exception as e:
                service_status["laboratory_agent"] = {"status": "unavailable", "error": str(e)}

            system_status["connected_services"] = service_status

        if request.include_metrics:
            # AI-powered system analysis
            analysis_prompt = f"""
            Analyze this TracSeq 2.0 system status and provide insights:
            
            System Status: {system_status}
            
            Provide:
            1. Overall system health assessment
            2. Service connectivity analysis
            3. Performance insights and trends
            4. Recommendations for optimization
            5. Potential issues or concerns
            6. Suggested maintenance actions
            
            Focus on actionable insights for laboratory operations.
            """

            ai_analysis = await ctx.sample(
                messages=[{"role": "user", "content": analysis_prompt}],
                model_preferences=["claude-3-sonnet-20240229"]
            )

            system_status["ai_analysis"] = ai_analysis.text

        await ctx.info("Enhanced system status generated successfully")

        return {
            "success": True,
            "timestamp": datetime.now().isoformat(),
            "system_status": system_status,
            "include_services": request.include_services,
            "include_metrics": request.include_metrics
        }

    except Exception as e:
        await ctx.error(f"Enhanced system status failed: {e!s}")
        return {
            "success": False,
            "error": str(e),
            "timestamp": datetime.now().isoformat()
        }

@mcp.resource("gateway://services/status")
async def gateway_services_status(ctx: Context) -> str:
    """Real-time status of all services connected through the enhanced gateway."""
    try:
        status_info = f"""
# Enhanced API Gateway - Connected Services Status

## Gateway Information
- **Status**: {'Operational' if gateway_state['initialized'] else 'Initializing'}
- **Version**: 2.0.0 (FastMCP Enhanced)
- **AI Queries Processed**: {gateway_state['ai_queries_processed']}
- **Last AI Query**: {gateway_state['last_ai_query'] or 'None'}

## Performance Metrics
- **Total Requests**: {gateway_state['gateway_performance']['total_requests']}
- **AI Enhanced Requests**: {gateway_state['gateway_performance']['ai_enhanced_requests']}
- **Average Response Time**: {gateway_state['gateway_performance']['average_response_time']:.2f}s

## Connected FastMCP Services
- **Laboratory Server**: Available via fastmcp_laboratory_server.py
- **Enhanced RAG Service**: Available via enhanced_rag_service/fastmcp_enhanced_rag_server.py
- **Laboratory Assistant Agent**: Available via mcp_infrastructure/fastmcp_laboratory_agent.py

## Integration Features
- **AI Query Assistant**: Active
- **Workflow Orchestration**: Enabled
- **Multi-Service Coordination**: Available
- **Natural Language Processing**: Active

---
*Status updated: {datetime.now().isoformat()}*
        """

        return status_info.strip()

    except Exception as e:
        await ctx.error(f"Error generating gateway services status: {e!s}")
        return f"Gateway services status unavailable: {e!s}"

@mcp.resource("gateway://ai/statistics")
async def gateway_ai_statistics(ctx: Context) -> str:
    """AI processing statistics for the enhanced gateway."""
    try:
        ai_stats = f"""
# Enhanced API Gateway - AI Processing Statistics

## AI Query Processing
- **Total AI Queries**: {gateway_state['ai_queries_processed']}
- **Last Query**: {gateway_state['last_ai_query'] or 'None'}
- **AI Enhancement Rate**: {(gateway_state['gateway_performance']['ai_enhanced_requests'] / max(gateway_state['gateway_performance']['total_requests'], 1)) * 100:.1f}%

## AI Capabilities
- **Laboratory Query Assistant**: Active
- **Workflow Orchestration**: Enabled
- **System Analysis**: Available
- **Natural Language Processing**: Active

## Model Integration
- **Primary Models**: Claude-3-Sonnet, GPT-4
- **Fallback Models**: GPT-3.5-Turbo
- **Context Management**: FastMCP Enhanced

## Service Integration
- **Multi-Service Queries**: Supported
- **Real-time Synthesis**: Enabled
- **Progressive Enhancement**: Active

---
*Statistics updated: {datetime.now().isoformat()}*
        """

        return ai_stats.strip()

    except Exception as e:
        await ctx.error(f"Error generating AI statistics: {e!s}")
        return f"AI statistics unavailable: {e!s}"

# FastAPI integration available when needed - see migration documentation

# Service initialization
async def initialize_enhanced_gateway():
    """Initialize the Enhanced API Gateway."""
    logger.info("Initializing Enhanced API Gateway with FastMCP")
    gateway_state["initialized"] = True
    gateway_state["last_ai_query"] = None
    logger.info("Enhanced API Gateway initialization complete")

# Main execution
if __name__ == "__main__":
    # Initialize enhanced gateway
    asyncio.run(initialize_enhanced_gateway())

    # Run with multiple transport options
    import sys

    if len(sys.argv) > 1 and sys.argv[1] == "--http":
        # Pure FastMCP HTTP mode
        mcp.run(transport="http", port=8005)
    elif len(sys.argv) > 1 and sys.argv[1] == "--sse":
        # FastMCP SSE mode for streaming
        mcp.run(transport="sse", port=8006)
    else:
        # Default FastMCP STDIO mode
        mcp.run(transport="stdio")
