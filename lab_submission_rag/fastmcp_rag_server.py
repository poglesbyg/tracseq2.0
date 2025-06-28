#!/usr/bin/env python3
"""
FastMCP-based Laboratory RAG Server for TracSeq 2.0

Replaces the traditional rag_orchestrator.py with a modern MCP server
that provides tools, resources, and prompts for laboratory document processing.
"""

import asyncio
import logging
import time
import uuid
from datetime import datetime
from pathlib import Path
from typing import Any

from fastmcp import Context, FastMCP
from pydantic import BaseModel, Field

# Import existing components to reuse business logic
from lab_submission_rag.config import settings
from lab_submission_rag.database import db_manager
from lab_submission_rag.rag.document_processor import DocumentProcessor
from lab_submission_rag.rag.vector_store import VectorStore
from lab_submission_rag.repositories.submission_repository import SubmissionRepository

# Configure logging
logging.basicConfig(
    level=getattr(logging, settings.log_level.upper()),
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)

# Initialize FastMCP server
mcp = FastMCP("TracSeq Laboratory RAG Server", version="2.0.0")

# Pydantic models for tool inputs
class DocumentProcessingRequest(BaseModel):
    file_path: str = Field(description="Path to the laboratory document to process")
    confidence_threshold: float | None = Field(default=0.7, description="Minimum confidence threshold for extraction")
    additional_context: dict[str, Any] | None = Field(default_factory=dict, description="Additional processing context")

class BatchProcessingRequest(BaseModel):
    file_paths: list[str] = Field(description="List of document paths to process in batch")
    batch_size: int | None = Field(default=5, description="Number of documents to process simultaneously")

class QueryRequest(BaseModel):
    query: str = Field(description="Natural language query about laboratory submissions")
    session_id: str | None = Field(default="default", description="Session ID for conversation context")
    filter_metadata: dict[str, Any] | None = Field(default_factory=dict, description="Optional metadata filters")

class SampleSearchRequest(BaseModel):
    search_criteria: dict[str, Any] = Field(description="Search criteria for samples")
    include_rag_processed: bool | None = Field(default=True, description="Include RAG-processed samples")

# Global components - initialized once
document_processor = None
vector_store = None
database_initialized = False

@mcp.tool
async def process_laboratory_document(
    request: DocumentProcessingRequest,
    ctx: Context
) -> dict[str, Any]:
    """
    Process a laboratory document using advanced RAG techniques.
    
    Extracts structured information from PDFs, DOCX, and other documents
    containing laboratory submission data, sample information, and protocols.
    """
    await ctx.info(f"Starting laboratory document processing: {request.file_path}")

    try:
        # Initialize components if needed
        await _ensure_components_initialized(ctx)

        start_time = time.time()
        file_path = Path(request.file_path)

        # Step 1: Process document into chunks
        await ctx.info(f"Processing document into chunks: {file_path.name}")
        document_chunks = await document_processor.process_document(file_path)

        if not document_chunks:
            await ctx.error(f"No content extracted from {file_path}")
            return {
                "success": False,
                "confidence_score": 0.0,
                "error": "No content could be extracted from document",
                "processing_time": time.time() - start_time
            }

        await ctx.info(f"Extracted {len(document_chunks)} chunks from document")

        # Step 2: Add to vector store with progress reporting
        await ctx.info("Adding chunks to vector store")
        await vector_store.add_chunks(document_chunks)

        # Report progress
        await ctx.report_progress(
            token="processing",
            progress=0.4,
            total=1.0
        )

        # Step 3: Extract submission information using enhanced LLM
        await ctx.info("Extracting laboratory submission information")

        # Get relevant chunks for extraction
        relevant_chunks = await _get_relevant_chunks_for_extraction(str(file_path), ctx)

        # Use context's LLM sampling for extraction
        extraction_prompt = f"""
        Extract laboratory submission information from the following document chunks.
        Apply confidence threshold of {request.confidence_threshold}.
        
        Document: {file_path.name}
        Chunks: {len(relevant_chunks)} relevant sections
        
        Extract information for:
        1. Administrative Information (submitter, institution, project)
        2. Sample Details (type, concentration, volume, quality)
        3. Storage Requirements (temperature, container, conditions)
        4. Sequencing Parameters (platform, coverage, analysis)
        5. Quality Control Metrics
        6. Timeline and Priority Information
        7. Special Instructions or Notes
        
        Return structured JSON with confidence scores for each field.
        """

        # Sample the LLM via MCP context
        llm_response = await ctx.sample(
            messages=[{"role": "user", "content": extraction_prompt}],
            model_preferences=["claude-3-sonnet-20240229", "gpt-4"]
        )

        # Process LLM response into extraction result
        extraction_result = await _process_llm_extraction_response(
            llm_response.text,
            relevant_chunks,
            str(file_path),
            ctx
        )

        # Report progress
        await ctx.report_progress(
            token="processing",
            progress=0.8,
            total=1.0
        )

        # Step 4: Save to database if successful
        if extraction_result.get("success", False):
            await _save_extraction_to_database(extraction_result, document_chunks, file_path, ctx)
            await ctx.info("Successfully saved extraction results to database")

        processing_time = time.time() - start_time
        extraction_result["processing_time"] = processing_time

        # Complete progress
        await ctx.report_progress(
            token="processing",
            progress=1.0,
            total=1.0
        )

        await ctx.info(f"Document processing completed in {processing_time:.2f}s")
        return extraction_result

    except Exception as e:
        await ctx.error(f"Error processing document: {str(e)}")
        return {
            "success": False,
            "confidence_score": 0.0,
            "error": str(e),
            "processing_time": time.time() - start_time if 'start_time' in locals() else 0
        }

@mcp.tool
async def process_documents_batch(
    request: BatchProcessingRequest,
    ctx: Context
) -> dict[str, Any]:
    """
    Process multiple laboratory documents in an optimized batch operation.
    
    Efficiently handles bulk document processing with progress tracking
    and intelligent error handling for large-scale laboratory operations.
    """
    await ctx.info(f"Starting batch processing of {len(request.file_paths)} documents")

    try:
        await _ensure_components_initialized(ctx)

        start_time = time.time()
        results = []
        successful_extractions = 0

        # Process in smaller batches to avoid overwhelming the system
        total_documents = len(request.file_paths)

        for i in range(0, total_documents, request.batch_size):
            batch = request.file_paths[i:i + request.batch_size]
            batch_num = (i // request.batch_size) + 1
            total_batches = (total_documents + request.batch_size - 1) // request.batch_size

            await ctx.info(f"Processing batch {batch_num}/{total_batches} ({len(batch)} documents)")

            # Process batch documents in parallel
            batch_tasks = []
            for file_path in batch:
                doc_request = DocumentProcessingRequest(
                    file_path=file_path,
                    confidence_threshold=0.7
                )
                batch_tasks.append(process_laboratory_document(doc_request, ctx))

            batch_results = await asyncio.gather(*batch_tasks, return_exceptions=True)

            # Process batch results
            for j, result in enumerate(batch_results):
                if isinstance(result, Exception):
                    await ctx.error(f"Batch processing error for {batch[j]}: {str(result)}")
                    results.append({
                        "success": False,
                        "file_path": batch[j],
                        "error": str(result)
                    })
                else:
                    results.append(result)
                    if result.get("success", False):
                        successful_extractions += 1

            # Report batch progress
            progress = (i + len(batch)) / total_documents
            await ctx.report_progress(
                token="batch_processing",
                progress=progress,
                total=1.0
            )

        # Calculate overall statistics
        processing_time = time.time() - start_time
        success_rate = successful_extractions / total_documents if total_documents > 0 else 0

        await ctx.info(f"Batch processing completed: {successful_extractions}/{total_documents} successful")

        return {
            "success": True,
            "total_documents": total_documents,
            "successful_extractions": successful_extractions,
            "failed_extractions": total_documents - successful_extractions,
            "success_rate": success_rate,
            "results": results,
            "processing_time": processing_time
        }

    except Exception as e:
        await ctx.error(f"Batch processing error: {str(e)}")
        return {
            "success": False,
            "error": str(e),
            "processing_time": time.time() - start_time if 'start_time' in locals() else 0
        }

@mcp.tool
async def query_laboratory_submissions(
    request: QueryRequest,
    ctx: Context
) -> str:
    """
    Answer questions about laboratory submissions using advanced RAG and database queries.
    
    Provides intelligent responses about samples, storage, processing status,
    quality metrics, and other laboratory management information.
    """
    await ctx.info(f"Processing laboratory query: {request.query}")

    try:
        await _ensure_components_initialized(ctx)

        # Check for database-specific queries first
        db_answer = await _handle_database_query(request.query, ctx)
        if db_answer:
            return db_answer

        # Search vector store for relevant information
        relevant_chunks = await vector_store.similarity_search(
            request.query,
            k=settings.max_search_results if hasattr(settings, "max_search_results") else 5,
            filter_metadata=request.filter_metadata
        )

        # Prepare context for LLM
        context_chunks = [(chunk.content, score) for chunk, score in relevant_chunks]

        # Use MCP context to sample LLM for enhanced answer
        enhanced_prompt = f"""
        You are an expert laboratory management assistant for TracSeq 2.0.
        Answer the following query based on the provided laboratory data and your knowledge:
        
        Query: {request.query}
        
        Relevant Laboratory Data:
        {chr(10).join([f"Source {i+1} (relevance: {score:.2f}): {content[:500]}..."
                      for i, (content, score) in enumerate(context_chunks)])}
        
        Provide a comprehensive, accurate response that:
        1. Directly answers the question
        2. Includes specific data from the sources when relevant
        3. Uses laboratory terminology appropriately
        4. Suggests next steps or related actions when helpful
        5. Indicates confidence level in the response
        """

        llm_response = await ctx.sample(
            messages=[{"role": "user", "content": enhanced_prompt}],
            model_preferences=["claude-3-sonnet-20240229", "gpt-4"]
        )

        answer = llm_response.text

        # Log the query for analytics
        await _log_query_interaction(request.query, answer, request.session_id, ctx)

        await ctx.info("Laboratory query processed successfully")
        return answer

    except Exception as e:
        await ctx.error(f"Error processing query: {str(e)}")
        return "I apologize, but I encountered an error while processing your query. Please try rephrasing your question or contact support if the issue persists."

@mcp.tool
async def search_laboratory_samples(
    request: SampleSearchRequest,
    ctx: Context
) -> dict[str, Any]:
    """
    Search laboratory samples with advanced filtering and AI-enhanced results.
    
    Supports complex search criteria including sample type, storage conditions,
    processing status, quality metrics, and RAG-extracted metadata.
    """
    await ctx.info(f"Searching laboratory samples with criteria: {request.search_criteria}")

    try:
        await _ensure_components_initialized(ctx)

        async with db_manager.get_session() as session:
            repo = SubmissionRepository(session)

            # Perform database search
            samples = await repo.search_samples_advanced(
                criteria=request.search_criteria,
                include_rag_data=request.include_rag_processed,
                limit=100
            )

            # Enhance results with AI analysis if requested
            if samples and len(samples) > 0:
                enhanced_results = await _enhance_search_results_with_ai(
                    request.search_criteria,
                    samples,
                    ctx
                )

                return {
                    "success": True,
                    "samples_found": len(samples),
                    "samples": [sample.dict() for sample in samples],
                    "enhanced_analysis": enhanced_results,
                    "search_criteria": request.search_criteria
                }
            else:
                return {
                    "success": True,
                    "samples_found": 0,
                    "samples": [],
                    "message": "No samples found matching the specified criteria",
                    "search_criteria": request.search_criteria
                }

    except Exception as e:
        await ctx.error(f"Error searching samples: {str(e)}")
        return {
            "success": False,
            "error": str(e),
            "search_criteria": request.search_criteria
        }

@mcp.resource("lab://submissions/status")
async def laboratory_submissions_status(ctx: Context) -> str:
    """
    Get real-time status of all laboratory submissions in the system.
    
    Provides comprehensive overview of document processing status,
    sample creation progress, and system health metrics.
    """
    try:
        await _ensure_components_initialized(ctx)

        async with db_manager.get_session() as session:
            repo = SubmissionRepository(session)

            # Get comprehensive status information
            stats = await repo.get_comprehensive_statistics()

            status_report = f"""
# TracSeq 2.0 Laboratory Submissions Status

## Overall Statistics
- **Total Submissions:** {stats.get('total_submissions', 0)}
- **Processed Documents:** {stats.get('processed_documents', 0)}
- **Active Samples:** {stats.get('active_samples', 0)}
- **Completed Workflows:** {stats.get('completed_workflows', 0)}

## Processing Status
- **Documents Pending:** {stats.get('pending_documents', 0)}
- **Currently Processing:** {stats.get('processing_documents', 0)}
- **Failed Extractions:** {stats.get('failed_extractions', 0)}
- **Average Confidence:** {stats.get('average_confidence', 0):.2f}

## Sample Distribution
{chr(10).join([f"- **{stype.title()}:** {count}" for stype, count in stats.get('samples_by_type', {}).items()])}

## Storage Status
{chr(10).join([f"- **{condition.title()}:** {count} samples" for condition, count in stats.get('storage_distribution', {}).items()])}

## System Health
- **Database Status:** {'Connected' if stats.get('db_healthy', False) else 'Issues Detected'}
- **Vector Store:** {'Operational' if stats.get('vector_store_healthy', False) else 'Issues Detected'}
- **Last Updated:** {datetime.now().isoformat()}

---
*Generated by TracSeq 2.0 Laboratory RAG Server*
            """

            return status_report.strip()

    except Exception as e:
        await ctx.error(f"Error generating status report: {str(e)}")
        return f"Error generating laboratory status report: {str(e)}"

@mcp.resource("lab://samples/recent")
async def recent_laboratory_samples(ctx: Context) -> str:
    """
    Get information about recently processed laboratory samples.
    
    Returns details about the most recent samples including
    processing status, quality metrics, and storage assignments.
    """
    try:
        await _ensure_components_initialized(ctx)

        async with db_manager.get_session() as session:
            repo = SubmissionRepository(session)

            # Get recent samples (last 24 hours)
            recent_samples = await repo.get_recent_samples(hours=24, limit=20)

            if not recent_samples:
                return "No samples processed in the last 24 hours."

            samples_info = []
            for sample in recent_samples:
                info = f"""
**{sample.sample_id}** ({sample.sample_type or 'Unknown Type'})
- Submitted: {sample.created_at.strftime('%Y-%m-%d %H:%M')}
- Status: {sample.status or 'Unknown'}
- Quality Score: {sample.quality_score or 'Not assessed'}
- Storage: {sample.storage_location or 'Not assigned'}
"""
                samples_info.append(info.strip())

            report = f"""
# Recent Laboratory Samples (Last 24 Hours)

Found **{len(recent_samples)}** recently processed samples:

{chr(10).join(samples_info)}

---
*Last updated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}*
            """

            return report.strip()

    except Exception as e:
        await ctx.error(f"Error fetching recent samples: {str(e)}")
        return f"Error fetching recent laboratory samples: {str(e)}"

@mcp.prompt
async def laboratory_document_analysis_prompt(
    document_type: str,
    analysis_focus: str = "comprehensive"
) -> str:
    """
    Generate optimized prompts for laboratory document analysis.
    
    Creates specialized prompts for different types of laboratory documents
    and analysis requirements, ensuring consistent high-quality extractions.
    """
    base_prompts = {
        "sample_manifest": f"""
        Analyze this laboratory sample manifest document with focus on {analysis_focus}.
        
        Extract the following information with high precision:
        1. **Sample Identifiers**: All sample IDs, barcodes, and tracking numbers
        2. **Sample Types**: DNA, RNA, Blood, Tissue, etc. with collection methods
        3. **Quantities**: Volume, concentration, purity measurements
        4. **Storage Requirements**: Temperature, container type, special conditions
        5. **Chain of Custody**: Collection dates, personnel, transport conditions
        6. **Quality Metrics**: Initial assessments, degradation indicators
        7. **Project Information**: Study codes, protocol references, priorities
        
        Provide confidence scores (0-1) for each extracted field.
        Flag any inconsistencies or missing critical information.
        """,

        "quality_report": f"""
        Analyze this laboratory quality control report with focus on {analysis_focus}.
        
        Extract key quality metrics and assessments:
        1. **Sample Quality Scores**: Numerical and categorical assessments
        2. **Instrumentation Data**: Equipment used, calibration status
        3. **Test Results**: Pass/fail status, numerical results, thresholds
        4. **Anomalies**: Any deviations, contamination, or quality issues
        5. **Recommendations**: Next steps, reprocessing needs, approvals
        6. **Compliance**: Regulatory requirements, standard adherence
        
        Identify critical quality failures that require immediate attention.
        """,

        "protocol": f"""
        Analyze this laboratory protocol document with focus on {analysis_focus}.
        
        Extract procedural and technical information:
        1. **Protocol Steps**: Detailed procedures, timing, conditions
        2. **Equipment Requirements**: Instruments, reagents, consumables
        3. **Safety Considerations**: Hazards, protective measures, emergency procedures
        4. **Quality Controls**: Checkpoints, validation steps, acceptance criteria
        5. **Expected Outcomes**: Results format, success criteria, troubleshooting
        6. **Version Control**: Document version, approval status, effective dates
        
        Highlight any protocol deviations or special instructions.
        """
    }

    return base_prompts.get(document_type, f"""
    Analyze this laboratory document with focus on {analysis_focus}.
    
    Extract all relevant laboratory information including:
    - Sample details and identifiers
    - Procedural information and protocols
    - Quality metrics and assessments
    - Storage and handling requirements
    - Personnel and chain of custody
    - Dates, times, and scheduling information
    
    Provide structured output with confidence scores for each field.
    """)

# Helper functions
async def _ensure_components_initialized(ctx: Context):
    """Initialize core components if not already done"""
    global document_processor, vector_store, database_initialized

    if not database_initialized:
        await ctx.info("Initializing database connection")
        await db_manager.initialize()
        await db_manager.create_tables()
        database_initialized = True

    if document_processor is None:
        await ctx.info("Initializing document processor")
        document_processor = DocumentProcessor()

    if vector_store is None:
        await ctx.info("Initializing vector store")
        vector_store = VectorStore()

async def _get_relevant_chunks_for_extraction(source_document: str, ctx: Context) -> list[tuple]:
    """Get relevant document chunks for information extraction"""
    # Implementation similar to original but with context logging
    extraction_queries = [
        "administrative information submitter institution project",
        "sample details type concentration volume quality",
        "storage requirements temperature container conditions",
        "sequencing parameters platform coverage analysis",
        "quality control metrics assessment validation",
        "timeline priority schedule dates",
        "special instructions notes protocols"
    ]

    all_chunks = []
    for query in extraction_queries:
        chunks = await vector_store.similarity_search(query, k=3)
        all_chunks.extend(chunks)

    # Remove duplicates and return top chunks
    unique_chunks = list({chunk.chunk_id: (chunk.content, score)
                         for chunk, score in all_chunks}.values())

    await ctx.info(f"Retrieved {len(unique_chunks)} unique chunks for extraction")
    return unique_chunks[:15]  # Limit to top 15

async def _process_llm_extraction_response(
    llm_text: str,
    chunks: list[tuple],
    source_document: str,
    ctx: Context
) -> dict[str, Any]:
    """Process LLM response into structured extraction result"""
    try:
        # Try to parse JSON response from LLM
        import json

        # Simple implementation - in production, would have more sophisticated parsing
        if llm_text.strip().startswith('{'):
            extracted_data = json.loads(llm_text)
        else:
            # Fallback: use LLM to structure the response
            structure_prompt = f"""
            Convert the following extraction response into structured JSON:
            {llm_text}
            
            Return JSON with fields: success, confidence_score, extracted_data, warnings
            """

            structure_response = await ctx.sample(
                messages=[{"role": "user", "content": structure_prompt}]
            )
            extracted_data = json.loads(structure_response.text)

        return {
            "success": extracted_data.get("success", True),
            "confidence_score": extracted_data.get("confidence_score", 0.8),
            "extracted_data": extracted_data.get("extracted_data", {}),
            "warnings": extracted_data.get("warnings", []),
            "source_document": source_document,
            "chunks_used": len(chunks)
        }

    except Exception as e:
        await ctx.error(f"Error processing LLM extraction: {str(e)}")
        return {
            "success": False,
            "confidence_score": 0.0,
            "error": str(e),
            "source_document": source_document
        }

async def _save_extraction_to_database(
    extraction_result: dict[str, Any],
    document_chunks: list,
    file_path: Path,
    ctx: Context
):
    """Save extraction results to database"""
    try:
        async with db_manager.get_session() as session:
            repo = SubmissionRepository(session)

            # Create extraction record
            extraction_data = {
                "extraction_id": str(uuid.uuid4()),
                "success": extraction_result["success"],
                "confidence_score": extraction_result["confidence_score"],
                "source_document": str(file_path),
                "extracted_data": extraction_result.get("extracted_data", {}),
                "warnings": extraction_result.get("warnings", []),
                "processing_time": extraction_result.get("processing_time", 0),
                "chunks_processed": len(document_chunks)
            }

            await repo.create_extraction_result(extraction_data)
            await ctx.info("Extraction results saved to database")

    except Exception as e:
        await ctx.error(f"Failed to save to database: {str(e)}")
        raise

async def _handle_database_query(query: str, ctx: Context) -> str | None:
    """Handle database-specific queries"""
    query_lower = query.lower()

    try:
        async with db_manager.get_session() as session:
            repo = SubmissionRepository(session)

            if "how many samples" in query_lower or "sample count" in query_lower:
                count = await repo.get_total_sample_count()
                return f"There are **{count}** total samples in the TracSeq 2.0 system."

            elif "submission" in query_lower and "count" in query_lower:
                submissions = await repo.get_submissions(limit=1000)
                return f"There are **{len(submissions)}** laboratory submissions in the system."

            # Add more database query patterns as needed

    except Exception as e:
        await ctx.error(f"Database query error: {str(e)}")

    return None

async def _log_query_interaction(query: str, response: str, session_id: str, ctx: Context):
    """Log query interaction for analytics"""
    try:
        async with db_manager.get_session() as session:
            repo = SubmissionRepository(session)

            query_data = {
                "query_id": str(uuid.uuid4()),
                "query_text": query,
                "session_id": session_id,
                "response_text": response,
                "timestamp": datetime.now()
            }

            await repo.log_query(query_data)

    except Exception as e:
        await ctx.error(f"Failed to log query: {str(e)}")

async def _enhance_search_results_with_ai(
    search_criteria: dict[str, Any],
    samples: list,
    ctx: Context
) -> dict[str, Any]:
    """Enhance search results with AI analysis"""
    try:
        # Use AI to analyze search results and provide insights
        analysis_prompt = f"""
        Analyze these laboratory sample search results:
        
        Search Criteria: {search_criteria}
        Results Found: {len(samples)} samples
        
        Sample Summary:
        {chr(10).join([f"- {sample.sample_id}: {sample.sample_type}, Status: {sample.status}"
                      for sample in samples[:10]])}
        
        Provide:
        1. Summary of results pattern
        2. Key insights about the samples found
        3. Recommendations for further analysis
        4. Any notable patterns or anomalies
        """

        ai_analysis = await ctx.sample(
            messages=[{"role": "user", "content": analysis_prompt}]
        )

        return {
            "ai_insights": ai_analysis.text,
            "result_summary": f"Found {len(samples)} samples matching criteria",
            "patterns_detected": [],  # Could be enhanced with more analysis
            "recommendations": []  # Could be enhanced with more analysis
        }

    except Exception as e:
        await ctx.error(f"Error enhancing search results: {str(e)}")
        return {"error": str(e)}

if __name__ == "__main__":
    # Run the FastMCP server
    import sys

    if len(sys.argv) > 1 and sys.argv[1] == "--stdio":
        # Run with STDIO transport for MCP clients
        mcp.run(transport="stdio")
    elif len(sys.argv) > 1 and sys.argv[1] == "--http":
        # Run with HTTP transport for web integration
        mcp.run(transport="http", host="0.0.0.0", port=8001, path="/mcp")
    else:
        # Default: run with STDIO transport
        print("🧬 TracSeq 2.0 Laboratory RAG Server")
        print("Usage:")
        print("  python fastmcp_rag_server.py --stdio   # For MCP clients")
        print("  python fastmcp_rag_server.py --http    # For web integration")
        print("")
        print("Starting with STDIO transport...")
        mcp.run()
