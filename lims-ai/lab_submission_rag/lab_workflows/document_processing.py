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
from typing import Any, Dict, List, Optional, Tuple

from llama_index.core.workflow import (
    Context,
    Event,
    StartEvent,
    StopEvent,
    Workflow,
    step,
)
from pydantic import BaseModel, Field
from sqlalchemy.exc import SQLAlchemyError

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


class ProcessingErrorEvent(Event):
    """Emitted when processing encounters a recoverable error"""
    error: str
    step: str
    retry_count: int = 0
    max_retries: int = 3
    context_data: Optional[Dict[str, Any]] = None


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
        
        # Configuration
        self.max_file_size = getattr(settings, "max_file_size", 50 * 1024 * 1024)  # 50MB
        self.supported_file_types = ['.pdf', '.docx', '.txt', '.doc', '.rtf']
        self.max_retries = getattr(settings, "max_processing_retries", 3)
        
    @step(pass_context=True)
    async def validate_document(
        self, ctx: Context, ev: StartEvent
    ) -> DocumentValidatedEvent | ValidationErrorEvent:
        """Validate the input document"""
        start_time = time.time()
        
        file_path = Path(getattr(ev, "file_path", ""))
        ctx.data["file_path"] = file_path
        ctx.data["start_time"] = start_time
        ctx.data["processing_id"] = str(uuid.uuid4())
        
        logger.info(f"Validating document: {file_path} (ID: {ctx.data['processing_id']})")
        
        # Check if file exists
        if not file_path.exists():
            return ValidationErrorEvent(
                error=f"File not found: {file_path}",
                file_path=str(file_path)
            )
        
        # Check file type
        if file_path.suffix.lower() not in self.supported_file_types:
            return ValidationErrorEvent(
                error=f"Unsupported file type: {file_path.suffix}. Supported types: {', '.join(self.supported_file_types)}",
                file_path=str(file_path)
            )
        
        # Check file size
        file_size = file_path.stat().st_size
        if file_size > self.max_file_size:
            return ValidationErrorEvent(
                error=f"File too large: {file_size} bytes (max: {self.max_file_size} bytes)",
                file_path=str(file_path)
            )
        
        # Check if file is readable
        try:
            with open(file_path, 'rb') as f:
                # Try to read first few bytes
                f.read(1024)
        except Exception as e:
            return ValidationErrorEvent(
                error=f"File is not readable: {str(e)}",
                file_path=str(file_path)
            )
        
        logger.info(f"Document validated successfully: {file_path.name} ({file_size} bytes)")
        
        return DocumentValidatedEvent(
            file_path=file_path,
            file_type=file_path.suffix[1:],
            file_size=file_size
        )
    
    @step(pass_context=True)
    async def process_chunks(
        self, ctx: Context, ev: DocumentValidatedEvent | ProcessingErrorEvent
    ) -> ChunksCreatedEvent | ProcessingErrorEvent | StopEvent:
        """Process document into chunks with retry logic"""
        
        # Handle retry event
        if isinstance(ev, ProcessingErrorEvent):
            if ev.retry_count >= ev.max_retries:
                logger.error(f"Max retries reached for chunk processing")
                return StopEvent(
                    result=ExtractionResult(
                        success=False,
                        confidence_score=0.0,
                        missing_fields=[],
                        warnings=[f"Chunk processing failed after {ev.retry_count} attempts: {ev.error}"],
                        processing_time=time.time() - ctx.data["start_time"],
                        source_document=str(ctx.data["file_path"])
                    )
                )
            # Extract file path from context
            file_path = Path(ev.context_data["file_path"]) if ev.context_data else ctx.data["file_path"]
        else:
            file_path = ev.file_path
        
        try:
            logger.info(f"Processing document into chunks: {file_path}")
            
            # Process document with timeout
            process_task = asyncio.create_task(
                self.document_processor.process_document(file_path)
            )
            
            document_chunks = await asyncio.wait_for(
                process_task,
                timeout=getattr(settings, "chunk_processing_timeout", 120)
            )
            
            if not document_chunks:
                # Retry on empty chunks
                return ProcessingErrorEvent(
                    error="No content could be extracted from document",
                    step="process_chunks",
                    retry_count=getattr(ev, "retry_count", 0) + 1 if isinstance(ev, ProcessingErrorEvent) else 1,
                    max_retries=self.max_retries,
                    context_data={"file_path": str(file_path)}
                )
            
            # Validate chunks
            valid_chunks = []
            for chunk in document_chunks:
                if hasattr(chunk, 'content') and chunk.content.strip():
                    valid_chunks.append(chunk)
            
            if not valid_chunks:
                return ProcessingErrorEvent(
                    error="All extracted chunks were empty",
                    step="process_chunks",
                    retry_count=getattr(ev, "retry_count", 0) + 1 if isinstance(ev, ProcessingErrorEvent) else 1,
                    max_retries=self.max_retries,
                    context_data={"file_path": str(file_path)}
                )
            
            # Store chunks in context for later use
            ctx.data["document_chunks"] = valid_chunks
            
            logger.info(f"Successfully created {len(valid_chunks)} chunks")
            
            return ChunksCreatedEvent(
                chunks=valid_chunks,
                source_document=str(file_path),
                chunk_count=len(valid_chunks)
            )
            
        except asyncio.TimeoutError:
            logger.error(f"Chunk processing timed out")
            return ProcessingErrorEvent(
                error="Chunk processing timed out",
                step="process_chunks",
                retry_count=getattr(ev, "retry_count", 0) + 1 if isinstance(ev, ProcessingErrorEvent) else 1,
                max_retries=self.max_retries,
                context_data={"file_path": str(file_path)}
            )
        except Exception as e:
            logger.error(f"Error processing chunks: {e}")
            return ProcessingErrorEvent(
                error=str(e),
                step="process_chunks",
                retry_count=getattr(ev, "retry_count", 0) + 1 if isinstance(ev, ProcessingErrorEvent) else 1,
                max_retries=self.max_retries,
                context_data={"file_path": str(file_path)}
            )
    
    @step(pass_context=True)
    async def store_chunks(
        self, ctx: Context, ev: ChunksCreatedEvent
    ) -> ChunksStoredEvent | ProcessingErrorEvent:
        """Store chunks in vector database with error handling"""
        logger.info(f"Storing {ev.chunk_count} chunks in vector store")
        
        try:
            # Add chunks to vector store with batch processing
            batch_size = getattr(settings, "vector_store_batch_size", 100)
            chunk_ids = []
            
            for i in range(0, len(ev.chunks), batch_size):
                batch = ev.chunks[i:i + batch_size]
                batch_ids = await self.vector_store.add_chunks(batch)
                chunk_ids.extend(batch_ids)
                
                # Log progress for large documents
                if ev.chunk_count > batch_size:
                    progress = min(i + batch_size, ev.chunk_count)
                    logger.info(f"Stored {progress}/{ev.chunk_count} chunks")
            
            # Verify all chunks were stored
            if len(chunk_ids) != ev.chunk_count:
                logger.warning(
                    f"Chunk count mismatch: expected {ev.chunk_count}, stored {len(chunk_ids)}"
                )
            
            return ChunksStoredEvent(
                chunk_ids=chunk_ids,
                source_document=ev.source_document
            )
            
        except Exception as e:
            logger.error(f"Failed to store chunks: {e}")
            return ProcessingErrorEvent(
                error=f"Vector store error: {str(e)}",
                step="store_chunks",
                retry_count=1,
                max_retries=2,
                context_data={"chunk_count": ev.chunk_count}
            )
    
    @step(pass_context=True)
    async def extract_information(
        self, ctx: Context, ev: ChunksStoredEvent
    ) -> ExtractionCompletedEvent | ProcessingErrorEvent:
        """Extract submission information using LLM with improved relevance scoring"""
        logger.info("Starting LLM extraction")
        
        try:
            # Get relevant chunks for extraction
            relevant_chunks = await self._get_relevant_chunks_for_extraction(
                ev.source_document,
                ctx.data.get("document_chunks", [])
            )
            
            if not relevant_chunks:
                logger.warning("No relevant chunks found for extraction")
                relevant_chunks = [(chunk.content, 0.5) for chunk in ctx.data.get("document_chunks", [])[:5]]
            
            # Extract submission info
            extraction_result = await self.llm_interface.extract_submission_info(
                relevant_chunks, 
                ev.source_document
            )
            
            # Enhance extraction result with additional metadata
            extraction_result.metadata = {
                "chunk_ids": ev.chunk_ids,
                "total_chunks": len(ctx.data.get("document_chunks", [])),
                "relevant_chunks_used": len(relevant_chunks),
                "processing_id": ctx.data.get("processing_id")
            }
            
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
            
        except Exception as e:
            logger.error(f"Extraction failed: {e}")
            return ProcessingErrorEvent(
                error=f"LLM extraction error: {str(e)}",
                step="extract_information",
                retry_count=1,
                max_retries=2
            )
    
    @step(pass_context=True)
    async def save_to_database(
        self, ctx: Context, ev: ExtractionCompletedEvent
    ) -> DatabaseSavedEvent | StopEvent:
        """Save extraction results to database with proper transaction handling"""
        
        if not ev.extraction_result.success:
            # Skip database save if extraction failed
            logger.info("Skipping database save due to extraction failure")
            return StopEvent(
                result=ev.extraction_result
            )
        
        retry_count = 0
        max_retries = 3
        
        while retry_count < max_retries:
            try:
                async with db_manager.get_session() as session:
                    async with session.begin():  # Use transaction
                        repo = SubmissionRepository(session)
                        
                        # Generate IDs
                        submission_id = str(uuid.uuid4())
                        document_id = str(uuid.uuid4())
                        file_path = Path(ctx.data["file_path"])
                        
                        # Create submission record if we have submission data
                        if ev.extraction_result.submission:
                            submission = ev.extraction_result.submission
                            submission.submission_id = submission_id
                            db_submission = await repo.create_submission(submission)
                        else:
                            # Create minimal submission record from extracted data
                            extracted_data = getattr(ev.extraction_result, 'extracted_data', {})
                            submission_data = {
                                "submission_id": submission_id,
                                "metadata": extracted_data,
                                "status": "extracted",
                                "confidence_score": ev.extraction_result.confidence_score
                            }
                            await repo.create_submission_from_dict(submission_data)
                        
                        # Create document record
                        document_data = {
                            "document_id": document_id,
                            "submission_id": submission_id,
                            "filename": file_path.name,
                            "file_path": str(file_path),
                            "file_type": file_path.suffix[1:],
                            "file_size": file_path.stat().st_size,
                            "processed": True,
                            "processing_time": ev.extraction_result.processing_time,
                            "chunk_count": len(ctx.data.get("document_chunks", [])),
                            "processed_at": datetime.utcnow(),
                            "processing_id": ctx.data.get("processing_id")
                        }
                        await repo.create_document(document_data)
                        
                        # Create extraction result record
                        extraction_data = {
                            "extraction_id": str(uuid.uuid4()),
                            "submission_id": submission_id,
                            "document_id": document_id,
                            "success": ev.extraction_result.success,
                            "confidence_score": ev.extraction_result.confidence_score,
                            "missing_fields": ev.extraction_result.missing_fields,
                            "warnings": ev.extraction_result.warnings,
                            "processing_time": ev.extraction_result.processing_time,
                            "source_document": ev.extraction_result.source_document,
                            "extracted_data": getattr(ev.extraction_result, 'extracted_data', {}),
                            "metadata": getattr(ev.extraction_result, 'metadata', {})
                        }
                        await repo.create_extraction_result(extraction_data)
                    
                    # Commit is automatic when exiting the transaction block
                    logger.info(f"Successfully saved extraction results to database (submission_id: {submission_id})")
                    
                    return DatabaseSavedEvent(
                        submission_id=submission_id,
                        document_id=document_id,
                        success=True,
                        extraction_result=ev.extraction_result
                    )
                
            except SQLAlchemyError as e:
                retry_count += 1
                logger.error(f"Database error (attempt {retry_count}/{max_retries}): {e}")
                if retry_count < max_retries:
                    await asyncio.sleep(2 ** retry_count)  # Exponential backoff
                else:
                    # Add warning but don't fail the entire process
                    ev.extraction_result.warnings.append(f"Database save failed after {max_retries} attempts: {str(e)}")
                    return StopEvent(
                        result=ev.extraction_result
                    )
            except Exception as e:
                logger.error(f"Unexpected error saving to database: {e}")
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
            
            # Add final metadata
            if not hasattr(extraction_result, 'metadata'):
                extraction_result.metadata = {}
            
            extraction_result.metadata.update({
                "database_saved": ev.success,
                "submission_id": ev.submission_id,
                "document_id": ev.document_id,
                "processing_completed_at": datetime.utcnow().isoformat()
            })
            
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
    
    @step(pass_context=True)
    async def handle_processing_error(
        self, ctx: Context, ev: ProcessingErrorEvent
    ) -> StopEvent:
        """Handle processing errors that exceeded retry limits"""
        logger.error(f"Processing error in step '{ev.step}': {ev.error}")
        
        return StopEvent(
            result=ExtractionResult(
                success=False,
                confidence_score=0.0,
                missing_fields=[],
                warnings=[f"Processing failed at step '{ev.step}': {ev.error}"],
                processing_time=time.time() - ctx.data.get("start_time", 0),
                source_document=str(ctx.data.get("file_path", "unknown"))
            )
        )
    
    async def _get_relevant_chunks_for_extraction(
        self, source_document: str, chunks: list
    ) -> List[Tuple[str, float]]:
        """Get relevant chunks for each extraction category using semantic search"""
        
        # Define search queries for different extraction categories
        search_queries = [
            "submitter name email contact information administrative",
            "sample type material DNA RNA tissue collection",
            "sequencing platform illumina pacbio nanopore coverage",
            "storage temperature conditions freezer refrigerated",
            "analysis pipeline bioinformatics reference genome",
            "quality control metrics concentration volume",
            "project assignment study protocol"
        ]
        
        relevant_chunks = []
        seen_content = set()
        
        try:
            # Perform semantic search for each query
            for query in search_queries:
                search_results = await self.vector_store.search(
                    query=query,
                    k=3,  # Top 3 chunks per query
                    filter_dict={"source_document": source_document}
                )
                
                for result in search_results:
                    content = result.get("content", "")
                    score = result.get("score", 0.0)
                    
                    # Avoid duplicate chunks
                    content_hash = hash(content[:100])  # Use first 100 chars as hash
                    if content_hash not in seen_content and score > 0.5:
                        seen_content.add(content_hash)
                        relevant_chunks.append((content, score))
            
            # Sort by relevance score
            relevant_chunks.sort(key=lambda x: x[1], reverse=True)
            
            # Limit to top chunks based on configuration
            max_chunks = getattr(settings, "max_extraction_chunks", 15)
            relevant_chunks = relevant_chunks[:max_chunks]
            
            logger.info(f"Found {len(relevant_chunks)} relevant chunks for extraction")
            
        except Exception as e:
            logger.error(f"Error in semantic search: {e}")
            # Fallback to first N chunks with default scores
            relevant_chunks = [(chunk.content, 0.7) for chunk in chunks[:10]]
        
        return relevant_chunks


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