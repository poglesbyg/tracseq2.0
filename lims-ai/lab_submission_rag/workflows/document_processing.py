"""
Document Processing Workflow using LlamaIndex Workflows

This workflow handles the multi-step process of:
1. Document ingestion and validation
2. Text extraction and chunking
3. Vector embedding and storage
4. Information extraction using LLM
5. Database persistence
"""

import asyncio
import logging
import time
import uuid
from datetime import datetime
from pathlib import Path
from typing import Any, Optional

from llama_index.workflows import (
    Context,
    Event,
    StartEvent,
    StopEvent,
    Workflow,
    step,
)
from pydantic import BaseModel, Field

from ..config import settings
from ..database import db_manager
from ..models.submission import ExtractionResult, LabSubmission
from ..rag.document_processor import DocumentProcessor
from ..rag.llm_interface import LLMInterface
from ..rag.vector_store import VectorStore
from ..repositories.submission_repository import SubmissionRepository

logger = logging.getLogger(__name__)


# Define workflow events
class DocumentValidatedEvent(Event):
    """Emitted when document is validated and ready for processing"""
    file_path: Path
    file_type: str
    file_size: int
    

class ChunksCreatedEvent(Event):
    """Emitted when document is chunked"""
    chunks: list
    source_document: str
    chunk_count: int


class ChunksStoredEvent(Event):
    """Emitted when chunks are stored in vector database"""
    chunk_ids: list[str]
    source_document: str


class ExtractionCompletedEvent(Event):
    """Emitted when information extraction is complete"""
    extraction_result: ExtractionResult
    relevant_chunks: list
    

class DatabaseSavedEvent(Event):
    """Emitted when results are saved to database"""
    submission_id: str
    document_id: str
    success: bool
    extraction_result: Optional[ExtractionResult] = None


class ValidationErrorEvent(Event):
    """Emitted when validation fails"""
    error: str
    file_path: str
    retry_count: int = 0


class DocumentProcessingWorkflow(Workflow):
    """
    Event-driven workflow for processing laboratory documents.
    
    This workflow orchestrates the entire document processing pipeline
    using an event-driven architecture for better control and observability.
    """
    
    def __init__(self, timeout: int = 300, verbose: bool = True):
        super().__init__(timeout=timeout, verbose=verbose)
        
        # Initialize components
        self.document_processor = DocumentProcessor()
        self.vector_store = VectorStore()
        self.llm_interface = LLMInterface()
        
    @step(pass_context=True)
    async def validate_document(
        self, ctx: Context, ev: StartEvent
    ) -> DocumentValidatedEvent | ValidationErrorEvent:
        """Validate the input document"""
        start_time = time.time()
        
        file_path = Path(getattr(ev, "file_path", ""))
        ctx.data["file_path"] = file_path
        ctx.data["start_time"] = start_time
        
        logger.info(f"Validating document: {file_path}")
        
        # Check if file exists
        if not file_path.exists():
            return ValidationErrorEvent(
                error=f"File not found: {file_path}",
                file_path=str(file_path)
            )
        
        # Check file type
        if file_path.suffix.lower() not in ['.pdf', '.docx', '.txt']:
            return ValidationErrorEvent(
                error=f"Unsupported file type: {file_path.suffix}",
                file_path=str(file_path)
            )
        
        # Check file size
        file_size = file_path.stat().st_size
        if file_size > settings.max_file_size:
            return ValidationErrorEvent(
                error=f"File too large: {file_size} bytes",
                file_path=str(file_path)
            )
        
        return DocumentValidatedEvent(
            file_path=file_path,
            file_type=file_path.suffix[1:],
            file_size=file_size
        )
    
    @step(pass_context=True)
    async def process_chunks(
        self, ctx: Context, ev: DocumentValidatedEvent
    ) -> ChunksCreatedEvent | StopEvent:
        """Process document into chunks"""
        try:
            logger.info(f"Processing document into chunks: {ev.file_path}")
            
            # Process document
            document_chunks = await self.document_processor.process_document(ev.file_path)
            
            if not document_chunks:
                return StopEvent(
                    result=ExtractionResult(
                        success=False,
                        confidence_score=0.0,
                        missing_fields=[],
                        warnings=["No content could be extracted from document"],
                        processing_time=time.time() - ctx.data["start_time"],
                        source_document=str(ev.file_path)
                    )
                )
            
            # Store chunks in context for later use
            ctx.data["document_chunks"] = document_chunks
            
            return ChunksCreatedEvent(
                chunks=document_chunks,
                source_document=str(ev.file_path),
                chunk_count=len(document_chunks)
            )
            
        except Exception as e:
            logger.error(f"Error processing chunks: {e}")
            return StopEvent(
                result=ExtractionResult(
                    success=False,
                    confidence_score=0.0,
                    missing_fields=[],
                    warnings=[f"Chunk processing error: {str(e)}"],
                    processing_time=time.time() - ctx.data["start_time"],
                    source_document=str(ev.file_path)
                )
            )
    
    @step(pass_context=True)
    async def store_chunks(
        self, ctx: Context, ev: ChunksCreatedEvent
    ) -> ChunksStoredEvent:
        """Store chunks in vector database"""
        logger.info(f"Storing {ev.chunk_count} chunks in vector store")
        
        # Add chunks to vector store
        await self.vector_store.add_chunks(ev.chunks)
        
        # Extract chunk IDs
        chunk_ids = [chunk.chunk_id for chunk in ev.chunks]
        
        return ChunksStoredEvent(
            chunk_ids=chunk_ids,
            source_document=ev.source_document
        )
    
    @step(pass_context=True)
    async def extract_information(
        self, ctx: Context, ev: ChunksStoredEvent
    ) -> ExtractionCompletedEvent:
        """Extract submission information using LLM"""
        logger.info("Starting LLM extraction")
        
        # Get relevant chunks for extraction
        relevant_chunks = await self._get_relevant_chunks_for_extraction(
            ev.source_document,
            ctx.data.get("document_chunks", [])
        )
        
        # Extract submission info
        extraction_result = await self.llm_interface.extract_submission_info(
            relevant_chunks, 
            ev.source_document
        )
        
        # Store extraction result in context for later use
        ctx.data["extraction_result"] = extraction_result
        
        logger.info(
            f"Extraction completed. Success: {extraction_result.success}, "
            f"Confidence: {extraction_result.confidence_score:.2f}"
        )
        
        return ExtractionCompletedEvent(
            extraction_result=extraction_result,
            relevant_chunks=relevant_chunks
        )
    
    @step(pass_context=True)
    async def save_to_database(
        self, ctx: Context, ev: ExtractionCompletedEvent
    ) -> DatabaseSavedEvent | StopEvent:
        """Save extraction results to database"""
        if not ev.extraction_result.success or not ev.extraction_result.submission:
            # Skip database save if extraction failed
            return StopEvent(
                result=ev.extraction_result
            )
        
        try:
            async with db_manager.get_session() as session:
                repo = SubmissionRepository(session)
                
                # Create submission record
                submission = ev.extraction_result.submission
                db_submission = await repo.create_submission(submission)
                
                # Create document record
                document_id = str(uuid.uuid4())
                file_path = Path(ctx.data["file_path"])
                
                document_data = {
                    "document_id": document_id,
                    "submission_id": submission.submission_id,
                    "filename": file_path.name,
                    "file_path": str(file_path),
                    "file_type": file_path.suffix[1:],
                    "file_size": file_path.stat().st_size,
                    "processed": True,
                    "processing_time": ev.extraction_result.processing_time,
                    "chunk_count": len(ctx.data.get("document_chunks", [])),
                    "processed_at": datetime.utcnow(),
                }
                await repo.create_document(document_data)
                
                # Create extraction result record
                extraction_data = {
                    "extraction_id": str(uuid.uuid4()),
                    "submission_id": submission.submission_id,
                    "success": ev.extraction_result.success,
                    "confidence_score": ev.extraction_result.confidence_score,
                    "missing_fields": ev.extraction_result.missing_fields,
                    "warnings": ev.extraction_result.warnings,
                    "processing_time": ev.extraction_result.processing_time,
                    "source_document": ev.extraction_result.source_document,
                    "extracted_data": submission.dict(),
                }
                await repo.create_extraction_result(extraction_data)
                
                logger.info("Successfully saved extraction results to database")
                
                return DatabaseSavedEvent(
                    submission_id=submission.submission_id,
                    document_id=document_id,
                    success=True,
                    extraction_result=ev.extraction_result
                )
                
        except Exception as e:
            logger.error(f"Failed to save to database: {e}")
            # Add warning but don't fail the entire process
            ev.extraction_result.warnings.append(f"Database save failed: {str(e)}")
            return StopEvent(
                result=ev.extraction_result
            )
    
    @step(pass_context=True)
    async def finalize_processing(
        self, ctx: Context, ev: DatabaseSavedEvent
    ) -> StopEvent:
        """Finalize the workflow and return results"""
        # Get the extraction result from event first, then fallback to context
        extraction_result = ev.extraction_result or ctx.data.get("extraction_result")
        
        # Update processing time
        if extraction_result:
            extraction_result.processing_time = time.time() - ctx.data.get("start_time", 0)
            
            logger.info(
                f"Document processing completed for submission {ev.submission_id}. "
                f"Total time: {extraction_result.processing_time:.2f}s"
            )
        else:
            logger.warning(
                f"No extraction result found for submission {ev.submission_id}"
            )
        
        return StopEvent(result=extraction_result)
    
    @step(pass_context=True)
    async def handle_validation_error(
        self, ctx: Context, ev: ValidationErrorEvent
    ) -> StopEvent:
        """Handle validation errors"""
        logger.error(f"Validation error: {ev.error}")
        
        return StopEvent(
            result=ExtractionResult(
                success=False,
                confidence_score=0.0,
                missing_fields=[],
                warnings=[ev.error],
                processing_time=time.time() - ctx.data.get("start_time", 0),
                source_document=ev.file_path
            )
        )
    
    async def _get_relevant_chunks_for_extraction(
        self, source_document: str, chunks: list
    ) -> list[tuple]:
        """Get relevant chunks for each extraction category"""
        # This is a simplified version - in production, you'd use
        # semantic search to find the most relevant chunks
        return [(chunk.content, 1.0) for chunk in chunks[:10]]


# Convenience function for backwards compatibility
async def process_document_with_workflow(file_path: str | Path) -> ExtractionResult:
    """Process a document using the workflow"""
    workflow = DocumentProcessingWorkflow()
    
    # Ensure database is initialized
    await db_manager.initialize()
    await db_manager.create_tables()
    
    # Run the workflow
    result = await workflow.run(file_path=str(file_path))
    return result