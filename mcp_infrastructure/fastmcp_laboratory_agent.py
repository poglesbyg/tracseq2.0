"""
Enhanced Laboratory Assistant Agent - FastMCP Implementation

An intelligent laboratory assistant agent using FastMCP for superior
multi-service coordination, context management, and AI-powered workflows.
"""

import asyncio
import logging
import time
import uuid
from datetime import datetime
from typing import Any, Dict, List, Optional

from fastmcp import FastMCP, Context, Client
from pydantic import BaseModel, Field

# Configure logging
logging.basicConfig(level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s")
logger = logging.getLogger(__name__)

# Initialize FastMCP server for Laboratory Assistant Agent
mcp = FastMCP("TracSeq Laboratory Assistant Agent", version="2.0.0")

# Pydantic models for tool inputs
class SubmissionProcessingRequest(BaseModel):
    document_path: str = Field(description="Path to the laboratory submission document")
    priority: str = Field(default="normal", description="Processing priority")
    notify_submitter: bool = Field(default=True, description="Whether to notify the submitter")

class QualityControlRequest(BaseModel):
    sample_ids: List[str] = Field(description="List of sample IDs for quality control")
    assessment_type: str = Field(default="comprehensive", description="Type of QC assessment")
    automated: bool = Field(default=True, description="Whether to run automated QC")

class WorkflowCoordinationRequest(BaseModel):
    workflow_type: str = Field(description="Type of workflow to coordinate")
    workflow_data: Dict[str, Any] = Field(description="Workflow-specific data")

class SampleSearchRequest(BaseModel):
    query: str = Field(description="Natural language search query for samples")
    use_ai_interpretation: bool = Field(default=True, description="Use AI to interpret the query")
    max_results: int = Field(default=50, description="Maximum number of results to return")

# Global state for agent management
agent_state = {
    "session_id": str(uuid.uuid4()),
    "operations_completed": 0,
    "last_activity": None,
    "active_workflows": {},
    "service_connections": {},
    "agent_performance": {
        "average_processing_time": 0.0,
        "success_rate": 0.0,
        "total_operations": 0
    }
}

@mcp.tool
async def coordinate_laboratory_workflow(
    request: WorkflowCoordinationRequest,
    ctx: Context
) -> Dict[str, Any]:
    """
    Coordinate complete laboratory workflow using multiple FastMCP services.
    
    This is the core orchestration tool that manages complex multi-service
    laboratory operations with intelligent coordination and error handling.
    """
    await ctx.info(f"Starting workflow coordination: {request.workflow_type}")
    
    start_time = time.time()
    workflow_id = str(uuid.uuid4())
    
    try:
        # Step 1: Initialize workflow tracking
        agent_state["active_workflows"][workflow_id] = {
            "type": request.workflow_type,
            "started_at": datetime.now().isoformat(),
            "status": "initializing"
        }
        
        if request.workflow_type == "sample_submission":
            result = await _coordinate_sample_submission_workflow(request.workflow_data, ctx)
        elif request.workflow_type == "quality_control":
            result = await _coordinate_quality_control_workflow(request.workflow_data, ctx)
        elif request.workflow_type == "storage_optimization":
            result = await _coordinate_storage_optimization_workflow(request.workflow_data, ctx)
        else:
            # Use AI to understand custom workflow types
            result = await _coordinate_custom_workflow(request, ctx)
        
        processing_time = time.time() - start_time
        
        # Update workflow status
        agent_state["active_workflows"][workflow_id]["status"] = "completed" if result["success"] else "failed"
        agent_state["active_workflows"][workflow_id]["completed_at"] = datetime.now().isoformat()
        agent_state["active_workflows"][workflow_id]["processing_time"] = processing_time
        
        # Update agent performance metrics
        await _update_agent_performance(result["success"], processing_time)
        
        await ctx.info(f"Workflow coordination completed in {processing_time:.2f}s")
        
        return {
            "success": result["success"],
            "workflow_id": workflow_id,
            "workflow_type": request.workflow_type,
            "coordination_results": result,
            "processing_time": processing_time,
            "agent_session": agent_state["session_id"]
        }
        
    except Exception as e:
        await ctx.error(f"Workflow coordination failed: {str(e)}")
        agent_state["active_workflows"][workflow_id]["status"] = "error"
        return {
            "success": False,
            "workflow_id": workflow_id,
            "error": str(e),
            "processing_time": time.time() - start_time
        }

@mcp.tool
async def process_laboratory_submission(
    request: SubmissionProcessingRequest,
    ctx: Context
) -> Dict[str, Any]:
    """
    Complete laboratory submission processing with multi-service coordination.
    
    Orchestrates document processing, sample creation, storage assignment,
    and workflow initiation using FastMCP's enhanced service communication.
    """
    await ctx.info(f"Processing laboratory submission: {request.document_path}")
    
    start_time = time.time()
    operation_id = str(uuid.uuid4())
    
    try:
        # Connect to multiple FastMCP servers for coordinated processing
        async with Client("fastmcp_laboratory_server.py") as lab_client:
            async with Client("enhanced_rag_service/fastmcp_enhanced_rag_server.py") as rag_client:
                
                # Step 1: Process document using Enhanced RAG Service
                await ctx.info("Step 1: Processing document with Enhanced RAG Service")
                await ctx.report_progress(token="submission", progress=0.1, total=1.0)
                
                doc_result = await rag_client.call_tool(
                    "extract_laboratory_data",
                    {
                        "document_path": request.document_path,
                        "extraction_type": "comprehensive",
                        "confidence_threshold": 0.85
                    }
                )
                
                if not doc_result.get("success", False):
                    await ctx.error("Document processing failed")
                    return {
                        "success": False,
                        "operation_id": operation_id,
                        "error": "Document processing failed",
                        "processing_time": time.time() - start_time
                    }
                
                # Step 2: Use AI to analyze and plan next steps
                await ctx.info("Step 2: AI-powered workflow planning")
                await ctx.report_progress(token="submission", progress=0.3, total=1.0)
                
                workflow_plan = await ctx.sample(
                    messages=[{
                        "role": "user", 
                        "content": f"""
                        Analyze this laboratory submission extraction and create an optimal workflow plan:
                        
                        Document: {request.document_path}
                        Extracted Data: {doc_result.get('extracted_data', {})}
                        Priority: {request.priority}
                        
                        Create a step-by-step plan for:
                        1. Sample creation and validation
                        2. Storage optimization
                        3. Quality control requirements
                        4. Sequencing preparation if needed
                        5. Notification and tracking setup
                        
                        Consider laboratory best practices and efficiency.
                        """
                    }],
                    model_preferences=["claude-3-sonnet-20240229", "gpt-4"]
                )
                
                # Step 3: Execute sample creation
                await ctx.info("Step 3: Creating samples from extracted data")
                await ctx.report_progress(token="submission", progress=0.5, total=1.0)
                
                sample_result = await lab_client.call_tool(
                    "create_laboratory_samples",
                    {
                        "extraction_data": doc_result.get("extracted_data", {}),
                        "workflow_plan": workflow_plan.text,
                        "priority": request.priority
                    }
                )
                
                # Step 4: Assign storage with AI optimization
                await ctx.info("Step 4: Optimizing storage assignments")
                await ctx.report_progress(token="submission", progress=0.7, total=1.0)
                
                storage_result = await lab_client.call_tool(
                    "optimize_sample_storage",
                    {
                        "samples": sample_result.get("samples", []),
                        "storage_requirements": doc_result.get("extracted_data", {}).get("storage_requirements", {}),
                        "optimization_strategy": "ai_enhanced"
                    }
                )
                
                # Step 5: Generate comprehensive summary
                await ctx.info("Step 5: Generating submission summary")
                await ctx.report_progress(token="submission", progress=0.9, total=1.0)
                
                summary = await ctx.sample(
                    messages=[{
                        "role": "user",
                        "content": f"""
                        Generate a comprehensive laboratory submission summary:
                        
                        Operation ID: {operation_id}
                        Document Processed: {request.document_path}
                        Samples Created: {len(sample_result.get('samples', []))}
                        Storage Assigned: {storage_result.get('success', False)}
                        
                        Workflow Results:
                        - Document Processing: {doc_result}
                        - Sample Creation: {sample_result}
                        - Storage Assignment: {storage_result}
                        
                        Provide:
                        1. Executive summary of submission processing
                        2. Key metrics and quality scores
                        3. Next steps and recommendations
                        4. Any issues or concerns identified
                        5. Estimated timeline for completion
                        """
                    }],
                    model_preferences=["claude-3-sonnet-20240229"]
                )
                
                processing_time = time.time() - start_time
                
                # Complete progress reporting
                await ctx.report_progress(token="submission", progress=1.0, total=1.0)
                
                # Update global state
                agent_state["operations_completed"] += 1
                agent_state["last_activity"] = datetime.now().isoformat()
                
                await ctx.info(f"Laboratory submission processing completed in {processing_time:.2f}s")
                
                return {
                    "success": True,
                    "operation_id": operation_id,
                    "document_path": request.document_path,
                    "processing_results": {
                        "document_extraction": doc_result,
                        "sample_creation": sample_result,
                        "storage_assignment": storage_result,
                        "workflow_plan": workflow_plan.text
                    },
                    "ai_summary": summary.text,
                    "processing_time": processing_time,
                    "priority": request.priority,
                    "notify_submitter": request.notify_submitter
                }
        
    except Exception as e:
        await ctx.error(f"Laboratory submission processing failed: {str(e)}")
        return {
            "success": False,
            "operation_id": operation_id,
            "error": str(e),
            "processing_time": time.time() - start_time
        }

@mcp.tool
async def automated_quality_control(
    request: QualityControlRequest,
    ctx: Context
) -> Dict[str, Any]:
    """
    Run comprehensive automated quality control with AI analysis and recommendations.
    
    Coordinates QC operations across multiple services with intelligent
    analysis and actionable recommendations for laboratory staff.
    """
    await ctx.info(f"Starting automated QC for {len(request.sample_ids)} samples")
    
    start_time = time.time()
    qc_session_id = str(uuid.uuid4())
    
    try:
        # Connect to laboratory and QC services
        async with Client("fastmcp_laboratory_server.py") as lab_client:
            
            # Step 1: Retrieve sample details
            await ctx.info("Retrieving sample details for QC analysis")
            
            sample_details = await lab_client.call_tool(
                "get_sample_details_batch",
                {"sample_ids": request.sample_ids}
            )
            
            if not sample_details.get("success", False):
                return {
                    "success": False,
                    "qc_session_id": qc_session_id,
                    "error": "Failed to retrieve sample details",
                    "processing_time": time.time() - start_time
                }
            
            # Step 2: AI-powered QC strategy planning
            await ctx.info("Planning QC strategy with AI analysis")
            
            qc_strategy = await ctx.sample(
                messages=[{
                    "role": "user",
                    "content": f"""
                    Design an optimal quality control strategy for these laboratory samples:
                    
                    Assessment Type: {request.assessment_type}
                    Sample Count: {len(request.sample_ids)}
                    Sample Details: {sample_details.get('samples', [])}
                    
                    Create a comprehensive QC plan that includes:
                    1. Sample-specific QC protocols based on type and characteristics
                    2. Critical quality parameters to assess
                    3. Pass/fail criteria for each sample type
                    4. Risk assessment for downstream processing
                    5. Recommended validation steps
                    
                    Focus on laboratory best practices and regulatory compliance.
                    """
                }],
                model_preferences=["claude-3-sonnet-20240229", "gpt-4"]
            )
            
            # Step 3: Execute QC assessments
            await ctx.info("Executing quality control assessments")
            
            qc_results = []
            overall_scores = []
            
            for i, sample_id in enumerate(request.sample_ids):
                # Report progress for each sample
                progress = (i + 1) / len(request.sample_ids)
                await ctx.report_progress(token="qc_assessment", progress=progress, total=1.0)
                
                # Run QC for individual sample
                sample_qc = await lab_client.call_tool(
                    "assess_sample_quality",
                    {
                        "sample_id": sample_id,
                        "assessment_type": request.assessment_type,
                        "qc_strategy": qc_strategy.text,
                        "automated": request.automated
                    }
                )
                
                qc_results.append(sample_qc)
                if sample_qc.get("success", False):
                    overall_scores.append(sample_qc.get("quality_score", 0.0))
            
            # Step 4: AI-powered analysis of QC results
            await ctx.info("Analyzing QC results with AI")
            
            qc_analysis = await ctx.sample(
                messages=[{
                    "role": "user",
                    "content": f"""
                    Analyze these laboratory quality control results and provide expert insights:
                    
                    QC Session: {qc_session_id}
                    Assessment Type: {request.assessment_type}
                    Samples Assessed: {len(request.sample_ids)}
                    
                    QC Results: {qc_results}
                    Sample Details: {sample_details.get('samples', [])}
                    
                    Provide a comprehensive analysis including:
                    1. Overall quality assessment summary
                    2. Individual sample quality status and concerns
                    3. Patterns or trends identified across samples
                    4. Risk assessment for laboratory workflows
                    5. Specific recommendations for failed or borderline samples
                    6. Suggested follow-up actions and timeline
                    
                    Use your laboratory expertise to provide actionable guidance.
                    """
                }],
                model_preferences=["claude-3-sonnet-20240229"]
            )
            
            processing_time = time.time() - start_time
            average_quality = sum(overall_scores) / len(overall_scores) if overall_scores else 0.0
            
            # Update agent state
            agent_state["operations_completed"] += 1
            agent_state["last_activity"] = datetime.now().isoformat()
            
            await ctx.info(f"Automated QC completed in {processing_time:.2f}s")
            
            return {
                "success": True,
                "qc_session_id": qc_session_id,
                "assessment_type": request.assessment_type,
                "samples_assessed": len(request.sample_ids),
                "qc_results": qc_results,
                "average_quality_score": average_quality,
                "qc_strategy": qc_strategy.text,
                "ai_analysis": qc_analysis.text,
                "processing_time": processing_time,
                "automated": request.automated
            }
        
    except Exception as e:
        await ctx.error(f"Automated QC failed: {str(e)}")
        return {
            "success": False,
            "qc_session_id": qc_session_id,
            "error": str(e),
            "processing_time": time.time() - start_time
        }

@mcp.tool
async def intelligent_sample_search(
    request: SampleSearchRequest,
    ctx: Context
) -> Dict[str, Any]:
    """
    Intelligent sample search with AI-powered query interpretation and result enhancement.
    
    Uses natural language processing to understand complex search queries
    and provides enhanced results with AI-generated insights.
    """
    await ctx.info(f"Processing intelligent sample search: {request.query}")
    
    try:
        # Step 1: AI-powered query interpretation
        if request.use_ai_interpretation:
            await ctx.info("Interpreting search query with AI")
            
            query_analysis = await ctx.sample(
                messages=[{
                    "role": "user",
                    "content": f"""
                    Interpret this laboratory sample search query and convert it to structured search criteria:
                    
                    User Query: "{request.query}"
                    
                    Analyze the query for:
                    1. Sample types or categories mentioned
                    2. Date/time ranges if specified
                    3. Quality or status requirements
                    4. Storage conditions or locations
                    5. Processing stage or workflow status
                    6. Submitter or project information
                    7. Any specific identifiers or properties
                    
                    Convert to structured search parameters that can be used for database queries.
                    Include confidence scores for each interpreted parameter.
                    """
                }],
                model_preferences=["claude-3-sonnet-20240229", "gpt-4"]
            )
            
            search_criteria = query_analysis.text
        else:
            search_criteria = request.query
        
        # Step 2: Execute search using laboratory service
        async with Client("fastmcp_laboratory_server.py") as lab_client:
            search_results = await lab_client.call_tool(
                "search_laboratory_samples",
                {
                    "search_criteria": search_criteria,
                    "max_results": request.max_results,
                    "include_metadata": True
                }
            )
        
        # Step 3: AI-enhanced result analysis
        if search_results.get("success", False) and search_results.get("samples", []):
            await ctx.info("Enhancing search results with AI analysis")
            
            result_enhancement = await ctx.sample(
                messages=[{
                    "role": "user",
                    "content": f"""
                    Enhance these laboratory sample search results with intelligent analysis:
                    
                    Original Query: "{request.query}"
                    Search Criteria: {search_criteria}
                    Results Found: {len(search_results.get('samples', []))}
                    
                    Sample Results: {search_results.get('samples', [])}
                    
                    Provide:
                    1. Summary of search results and relevance to query
                    2. Key patterns or groupings in the results
                    3. Notable samples or outliers
                    4. Suggestions for refining the search if needed
                    5. Related samples or categories that might be of interest
                    
                    Make the results more actionable and insightful for laboratory staff.
                    """
                }],
                model_preferences=["claude-3-sonnet-20240229"]
            )
            
            enhanced_analysis = result_enhancement.text
        else:
            enhanced_analysis = "No samples found matching the search criteria."
        
        await ctx.info("Intelligent sample search completed")
        
        return {
            "success": True,
            "original_query": request.query,
            "interpreted_criteria": search_criteria if request.use_ai_interpretation else None,
            "search_results": search_results,
            "ai_enhanced_analysis": enhanced_analysis,
            "use_ai_interpretation": request.use_ai_interpretation
        }
        
    except Exception as e:
        await ctx.error(f"Intelligent sample search failed: {str(e)}")
        return {
            "success": False,
            "original_query": request.query,
            "error": str(e)
        }

@mcp.resource("agent://status/current")
async def agent_status(ctx: Context) -> str:
    """Real-time status of the Laboratory Assistant Agent."""
    try:
        status_info = f"""
# Laboratory Assistant Agent Status

## Agent Information
- **Session ID**: {agent_state['session_id']}
- **Status**: Operational
- **Version**: 2.0.0 (FastMCP Enhanced)

## Performance Metrics
- **Operations Completed**: {agent_state['operations_completed']}
- **Average Processing Time**: {agent_state['agent_performance']['average_processing_time']:.2f}s
- **Success Rate**: {agent_state['agent_performance']['success_rate']:.1f}%
- **Last Activity**: {agent_state['last_activity'] or 'None'}

## Active Workflows
- **Currently Active**: {len(agent_state['active_workflows'])}
- **Active Workflows**: {list(agent_state['active_workflows'].keys())}

## Service Connections
- **Connected Services**: {len(agent_state['service_connections'])}
- **FastMCP Integration**: Enabled
- **AI Analysis**: Active

---
*Status updated: {datetime.now().isoformat()}*
        """
        
        return status_info.strip()
        
    except Exception as e:
        await ctx.error(f"Error generating agent status: {str(e)}")
        return f"Agent status unavailable: {str(e)}"

@mcp.resource("agent://workflows/active")
async def active_workflows(ctx: Context) -> str:
    """Information about currently active laboratory workflows."""
    try:
        if not agent_state['active_workflows']:
            return "No active workflows currently running."
        
        workflow_info = "# Active Laboratory Workflows\n\n"
        
        for workflow_id, workflow_data in agent_state['active_workflows'].items():
            workflow_info += f"""
## Workflow: {workflow_id}
- **Type**: {workflow_data.get('type', 'Unknown')}
- **Status**: {workflow_data.get('status', 'Unknown')}
- **Started**: {workflow_data.get('started_at', 'Unknown')}
- **Processing Time**: {workflow_data.get('processing_time', 'In progress')}

"""
        
        workflow_info += f"\n---\n*Last updated: {datetime.now().isoformat()}*"
        
        return workflow_info.strip()
        
    except Exception as e:
        await ctx.error(f"Error generating active workflows info: {str(e)}")
        return f"Active workflows information unavailable: {str(e)}"

# Helper functions for workflow coordination
async def _coordinate_sample_submission_workflow(workflow_data: Dict[str, Any], ctx: Context) -> Dict[str, Any]:
    """Coordinate sample submission workflow."""
    await ctx.info("Coordinating sample submission workflow")
    
    # Mock implementation - would coordinate actual services
    return {
        "success": True,
        "workflow_steps": ["document_processing", "sample_creation", "storage_assignment"],
        "completed_steps": 3,
        "total_steps": 3,
        "samples_created": workflow_data.get("sample_count", 1),
        "storage_assigned": True
    }

async def _coordinate_quality_control_workflow(workflow_data: Dict[str, Any], ctx: Context) -> Dict[str, Any]:
    """Coordinate quality control workflow."""
    await ctx.info("Coordinating quality control workflow")
    
    # Mock implementation
    return {
        "success": True,
        "workflow_steps": ["sample_retrieval", "qc_analysis", "result_validation"],
        "completed_steps": 3,
        "total_steps": 3,
        "samples_assessed": len(workflow_data.get("sample_ids", [])),
        "average_quality": 94.2
    }

async def _coordinate_storage_optimization_workflow(workflow_data: Dict[str, Any], ctx: Context) -> Dict[str, Any]:
    """Coordinate storage optimization workflow."""
    await ctx.info("Coordinating storage optimization workflow")
    
    # Mock implementation
    return {
        "success": True,
        "workflow_steps": ["capacity_analysis", "optimization_planning", "assignment_execution"],
        "completed_steps": 3,
        "total_steps": 3,
        "efficiency_improvement": "15%",
        "samples_relocated": workflow_data.get("sample_count", 0)
    }

async def _coordinate_custom_workflow(request: WorkflowCoordinationRequest, ctx: Context) -> Dict[str, Any]:
    """Use AI to coordinate custom workflow types."""
    await ctx.info(f"Coordinating custom workflow: {request.workflow_type}")
    
    # Use AI to understand and coordinate custom workflows
    coordination_plan = await ctx.sample(
        messages=[{
            "role": "user",
            "content": f"""
            Coordinate this custom laboratory workflow:
            
            Workflow Type: {request.workflow_type}
            Workflow Data: {request.workflow_data}
            Coordination Level: {request.coordination_level}
            
            Determine the appropriate steps and coordination strategy.
            Provide a structured workflow execution plan.
            """
        }],
        model_preferences=["claude-3-sonnet-20240229"]
    )
    
    return {
        "success": True,
        "workflow_type": request.workflow_type,
        "coordination_plan": coordination_plan.text,
        "custom_workflow": True
    }

async def _update_agent_performance(success: bool, processing_time: float):
    """Update agent performance metrics."""
    perf = agent_state["agent_performance"]
    
    # Update total operations
    perf["total_operations"] += 1
    
    # Update success rate
    if perf["total_operations"] == 1:
        perf["success_rate"] = 100.0 if success else 0.0
    else:
        current_successes = (perf["success_rate"] / 100.0) * (perf["total_operations"] - 1)
        if success:
            current_successes += 1
        perf["success_rate"] = (current_successes / perf["total_operations"]) * 100.0
    
    # Update average processing time
    if perf["total_operations"] == 1:
        perf["average_processing_time"] = processing_time
    else:
        current_avg = perf["average_processing_time"]
        total_ops = perf["total_operations"]
        perf["average_processing_time"] = (current_avg * (total_ops - 1) + processing_time) / total_ops

# Main execution
if __name__ == "__main__":
    # Initialize agent
    logger.info("Initializing Laboratory Assistant Agent with FastMCP")
    agent_state["last_activity"] = datetime.now().isoformat()
    
    # Run FastMCP server with multiple transport options
    import sys
    
    if len(sys.argv) > 1 and sys.argv[1] == "--http":
        # HTTP mode for web integration
        mcp.run(transport="http", port=8003)
    elif len(sys.argv) > 1 and sys.argv[1] == "--sse":
        # SSE mode for streaming clients
        mcp.run(transport="sse", port=8004)
    else:
        # Default STDIO mode for MCP clients
        mcp.run(transport="stdio") 