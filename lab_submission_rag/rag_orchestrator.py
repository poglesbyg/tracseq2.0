"""
RAG Orchestrator for Laboratory Submission Information Extraction
"""

import asyncio
import logging
import time
import uuid
from pathlib import Path
from typing import List, Dict, Any, Optional, Union
from datetime import datetime

from rag.document_processor import DocumentProcessor
from rag.vector_store import VectorStore
from rag.llm_interface import LLMInterface
from rag.enhanced_llm_interface import enhanced_llm
from models.submission import LabSubmission, ExtractionResult, BatchExtractionResult
from models.database import LabSubmissionDB, SampleDB, DocumentDB, DocumentChunkDB
from repositories.submission_repository import SubmissionRepository
from database import db_manager
from config import settings

# Configure logging
logging.basicConfig(
    level=getattr(logging, settings.log_level.upper()),
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class LabSubmissionRAG:
    """
    Main RAG system for extracting laboratory submission information from documents.
    
    This system can:
    1. Process laboratory documents (PDF, DOCX, TXT)
    2. Extract information across 7 categories using LLM
    3. Answer questions about submissions
    4. Store and retrieve submission data from PostgreSQL
    """
    
    def __init__(self):
        """Initialize the RAG system components"""
        logger.info("Initializing Lab Submission RAG system...")
        
        # Initialize components
        self.document_processor = DocumentProcessor()
        self.vector_store = VectorStore()
        self.llm_interface = LLMInterface()
        
        # Initialize enhanced LLM for queries
        from rag.enhanced_llm_interface import enhanced_llm
        self.enhanced_llm = enhanced_llm
        
        # Create necessary directories
        self._ensure_directories()
        
        # Initialize database (async, will be called separately)
        self._database_initialized = False
        
        logger.info("Lab Submission RAG system initialized successfully")
    
    async def initialize_database(self):
        """Initialize database connection and create tables"""
        if self._database_initialized:
            return
            
        try:
            await db_manager.initialize()
            await db_manager.create_tables()
            self._database_initialized = True
            logger.info("Database initialized successfully")
        except Exception as e:
            logger.error(f"Failed to initialize database: {e}")
            raise
    
    def _ensure_directories(self):
        """Ensure all required directories exist"""
        directories = [
            settings.vector_store_path,
            settings.upload_dir,
            settings.export_dir,
            settings.log_dir
        ]
        
        for directory in directories:
            Path(directory).mkdir(parents=True, exist_ok=True)
    
    async def process_document(self, file_path: Union[str, Path]) -> ExtractionResult:
        """
        Process a single laboratory document and extract submission information.
        
        Args:
            file_path: Path to the document to process
            
        Returns:
            ExtractionResult containing extracted submission information
        """
        start_time = time.time()
        file_path = Path(file_path)
        
        logger.info(f"Processing document: {file_path}")
        
        try:
            # Step 1: Process document into chunks
            logger.info(f"Starting document processing for {file_path}")
            document_chunks = await self.document_processor.process_document(file_path)
            logger.info(f"Document processor returned {len(document_chunks)} chunks")
            
            if not document_chunks:
                logger.warning(f"No chunks extracted from {file_path}")
                return ExtractionResult(
                    success=False,
                    confidence_score=0.0,
                    missing_fields=[],
                    warnings=["No content could be extracted from document"],
                    processing_time=time.time() - start_time,
                    source_document=str(file_path)
                )
            
            # Log chunk details
            for i, chunk in enumerate(document_chunks):
                logger.debug(f"Chunk {i}: ID={chunk.chunk_id}, content_length={len(chunk.content)}")
            
            # Step 2: Add chunks to vector store
            logger.info(f"Adding {len(document_chunks)} chunks to vector store")
            await self.vector_store.add_chunks(document_chunks)
            
            # Step 3: Search for relevant chunks for each category
            logger.info("Getting relevant chunks for extraction")
            relevant_chunks = await self._get_relevant_chunks_for_extraction(str(file_path))
            logger.info(f"Found {len(relevant_chunks)} relevant chunks")
            
            # Step 4: Extract submission information using LLM
            logger.info("Starting LLM extraction")
            extraction_result = await self.llm_interface.extract_submission_info(
                relevant_chunks, str(file_path)
            )
            logger.info(f"LLM extraction completed. Success: {extraction_result.success}")
            
            # Step 5: Save to database if extraction was successful
            if extraction_result.success and extraction_result.submission:
                try:
                    await self._save_extraction_to_database(
                        extraction_result, 
                        document_chunks, 
                        file_path
                    )
                    logger.info("Successfully saved extraction results to database")
                except Exception as db_error:
                    logger.error(f"Failed to save to database: {db_error}")
                    # Don't fail the entire process if database save fails
                    extraction_result.warnings.append(f"Database save failed: {str(db_error)}")
            
            # Update processing time
            extraction_result.processing_time = time.time() - start_time
            
            logger.info(f"Document processing completed. Success: {extraction_result.success}, "
                       f"Confidence: {extraction_result.confidence_score:.2f}")
            
            return extraction_result
            
        except Exception as e:
            logger.error(f"Error processing document {file_path}: {str(e)}")
            return ExtractionResult(
                success=False,
                confidence_score=0.0,
                missing_fields=[],
                warnings=[f"Processing error: {str(e)}"],
                processing_time=time.time() - start_time,
                source_document=str(file_path)
            )
    
    async def _save_extraction_to_database(
        self, 
        extraction_result: ExtractionResult, 
        document_chunks: List, 
        file_path: Path
    ):
        """Save extraction results to PostgreSQL database"""
        async with db_manager.get_session() as session:
            repo = SubmissionRepository(session)
            
            # Create submission record
            submission = extraction_result.submission
            db_submission = await repo.create_submission(submission)
            
            # Create document record
            document_id = str(uuid.uuid4())
            document_data = {
                "document_id": document_id,
                "submission_id": submission.submission_id,
                "filename": file_path.name,
                "file_path": str(file_path),
                "file_type": file_path.suffix[1:],
                "file_size": file_path.stat().st_size if file_path.exists() else None,
                "processed": True,
                "processing_time": extraction_result.processing_time,
                "chunk_count": len(document_chunks),
                "processed_at": datetime.utcnow()
            }
            db_document = await repo.create_document(document_data)
            
            # Create document chunks
            for chunk in document_chunks:
                chunk_data = {
                    "chunk_id": chunk.chunk_id,
                    "document_id": document_id,
                    "content": chunk.content,
                    "chunk_index": chunk.chunk_index,
                    "page_number": chunk.metadata.get("page_number", 1),
                    "embedding": chunk.embedding,
                    "metadata": chunk.metadata
                }
                await repo.create_document_chunk(chunk_data)
            
            # Create extraction result record
            extraction_data = {
                "extraction_id": str(uuid.uuid4()),
                "submission_id": submission.submission_id,
                "success": extraction_result.success,
                "confidence_score": extraction_result.confidence_score,
                "missing_fields": extraction_result.missing_fields,
                "warnings": extraction_result.warnings,
                "processing_time": extraction_result.processing_time,
                "source_document": extraction_result.source_document,
                "extracted_data": submission.dict() if submission else None
            }
            await repo.create_extraction_result(extraction_data)
    
    async def process_documents_batch(self, file_paths: List[Union[str, Path]]) -> BatchExtractionResult:
        """
        Process multiple laboratory documents in batch.
        
        Args:
            file_paths: List of document paths to process
            
        Returns:
            BatchExtractionResult containing results for all documents
        """
        start_time = time.time()
        logger.info(f"Starting batch processing of {len(file_paths)} documents")
        
        results = []
        successful_extractions = 0
        
        # Process documents in batches to avoid overwhelming the system
        for i in range(0, len(file_paths), settings.batch_size):
            batch = file_paths[i:i + settings.batch_size]
            batch_tasks = [self.process_document(file_path) for file_path in batch]
            batch_results = await asyncio.gather(*batch_tasks, return_exceptions=True)
            
            for result in batch_results:
                if isinstance(result, Exception):
                    logger.error(f"Batch processing error: {str(result)}")
                    results.append(ExtractionResult(
                        success=False,
                        confidence_score=0.0,
                        missing_fields=[],
                        warnings=[f"Batch processing error: {str(result)}"],
                        processing_time=0.0,
                        source_document="unknown"
                    ))
                else:
                    results.append(result)
                    if result.success:
                        successful_extractions += 1
        
        # Calculate overall confidence
        successful_results = [r for r in results if r.success]
        overall_confidence = (
            sum(r.confidence_score for r in successful_results) / len(successful_results)
            if successful_results else 0.0
        )
        
        processing_time = time.time() - start_time
        
        logger.info(f"Batch processing completed. {successful_extractions}/{len(file_paths)} successful")
        
        return BatchExtractionResult(
            total_documents=len(file_paths),
            successful_extractions=successful_extractions,
            failed_extractions=len(file_paths) - successful_extractions,
            results=results,
            overall_confidence=overall_confidence,
            processing_time=processing_time
        )
    
    async def query_submissions(self, query: str, filter_metadata: Optional[Dict[str, Any]] = None, session_id: str = "default") -> str:
        """
        Answer questions about laboratory submissions using enhanced RAG intelligence and database queries.
        
        Args:
            query: Natural language query about submissions
            filter_metadata: Optional metadata filters for search
            session_id: Session ID for conversation context
            
        Returns:
            Natural language answer based on stored submission data and enhanced intelligence
        """
        start_time = time.time()
        logger.info(f"Processing enhanced query: {query}")
        
        try:
            # Check if this is a database-specific query (sample counts, statistics, etc.)
            db_answer = await self._handle_database_query(query)
            if db_answer:
                # Log the query
                await self._log_query(query, db_answer, session_id, time.time() - start_time)
                return db_answer
            
            # Search for relevant chunks in vector store
            relevant_chunks = await self.vector_store.similarity_search(
                query, 
                k=settings.max_search_results if hasattr(settings, 'max_search_results') else 5,
                filter_metadata=filter_metadata
            )
            
            # Convert to format expected by enhanced LLM interface
            chunks_with_scores = [(chunk.content, score) for chunk, score in relevant_chunks]
            
            # Get enhanced answer from improved LLM interface
            answer = await self.enhanced_llm.answer_query(
                query, 
                chunks_with_scores, 
                session_id=session_id,
                submission_data=None  # Could add submission context here
            )
            
            # Log the query
            await self._log_query(query, answer, session_id, time.time() - start_time, len(relevant_chunks))
            
            logger.info("Enhanced query processed successfully")
            return answer
            
        except Exception as e:
            logger.error(f"Error processing enhanced query: {str(e)}")
            error_message = f"I apologize, but I encountered an error while processing your query. Please try rephrasing your question or contact support if the issue persists."
            
            # Log the failed query
            await self._log_query(query, error_message, session_id, time.time() - start_time)
            return error_message
    
    async def _handle_database_query(self, query: str) -> Optional[str]:
        """Handle database-specific queries like sample counts and statistics"""
        query_lower = query.lower()
        
        try:
            async with db_manager.get_session() as session:
                repo = SubmissionRepository(session)
                
                # Sample count queries
                if any(phrase in query_lower for phrase in ["how many samples", "sample count", "number of samples", "total samples"]):
                    
                    # Check for specific filters
                    sample_type = None
                    storage_condition = None
                    
                    # Extract sample type from query
                    for stype in ["dna", "rna", "blood", "saliva", "tissue", "urine"]:
                        if stype in query_lower:
                            sample_type = stype
                            break
                    
                    # Extract storage condition from query
                    for condition in ["frozen", "refrigerated", "room temperature", "cryogenic"]:
                        if condition in query_lower:
                            storage_condition = condition
                            break
                    
                    count = await repo.get_sample_count(
                        sample_type=sample_type,
                        storage_condition=storage_condition
                    )
                    
                    if sample_type and storage_condition:
                        return f"There are **{count}** {sample_type.upper()} samples stored at {storage_condition} conditions in the system."
                    elif sample_type:
                        return f"There are **{count}** {sample_type.upper()} samples in the system."
                    elif storage_condition:
                        return f"There are **{count}** samples stored at {storage_condition} conditions in the system."
                    else:
                        return f"There are **{count}** total samples in the system."
                
                # Sample statistics queries
                elif any(phrase in query_lower for phrase in ["sample statistics", "sample breakdown", "sample summary"]):
                    stats = await repo.get_sample_statistics()
                    
                    response = f"**Sample Statistics:**\n\n"
                    response += f"**Total Samples:** {stats['total_samples']}\n\n"
                    
                    if stats['by_type']:
                        response += "**By Sample Type:**\n"
                        for sample_type, count in stats['by_type'].items():
                            if sample_type:  # Skip None values
                                response += f"• {sample_type.title()}: {count}\n"
                        response += "\n"
                    
                    if stats['by_storage_condition']:
                        response += "**By Storage Condition:**\n"
                        for condition, count in stats['by_storage_condition'].items():
                            if condition:  # Skip None values
                                response += f"• {condition.title()}: {count}\n"
                        response += "\n"
                    
                    if stats['by_priority']:
                        response += "**By Priority:**\n"
                        for priority, count in stats['by_priority'].items():
                            if priority:  # Skip None values
                                response += f"• {priority.title()}: {count}\n"
                    
                    return response
                
                # Submission queries
                elif any(phrase in query_lower for phrase in ["how many submissions", "submission count", "total submissions"]):
                    submissions = await repo.get_submissions(limit=1000)  # Get all to count
                    count = len(submissions)
                    return f"There are **{count}** total submissions in the system."
                
                # Search queries
                elif any(phrase in query_lower for phrase in ["search for", "find samples", "look for"]):
                    # Extract search term (simple implementation)
                    search_terms = query_lower.replace("search for", "").replace("find samples", "").replace("look for", "").strip()
                    if search_terms:
                        samples = await repo.search_samples(search_terms, limit=10)
                        if samples:
                            response = f"Found **{len(samples)}** samples matching '{search_terms}':\n\n"
                            for sample in samples[:5]:  # Show first 5
                                response += f"• **{sample.sample_id}** - {sample.sample_name or 'No name'}\n"
                            if len(samples) > 5:
                                response += f"\n... and {len(samples) - 5} more samples."
                            return response
                        else:
                            return f"No samples found matching '{search_terms}'."
                
        except Exception as e:
            logger.error(f"Error handling database query: {e}")
            return None
        
        return None
    
    async def _log_query(
        self, 
        query: str, 
        response: str, 
        session_id: str, 
        processing_time: float,
        chunks_retrieved: int = 0
    ):
        """Log query for analytics"""
        try:
            async with db_manager.get_session() as session:
                repo = SubmissionRepository(session)
                
                query_data = {
                    "query_id": str(uuid.uuid4()),
                    "query_text": query,
                    "session_id": session_id,
                    "response_text": response,
                    "processing_time": processing_time,
                    "chunks_retrieved": chunks_retrieved
                }
                
                await repo.log_query(query_data)
                
        except Exception as e:
            logger.error(f"Failed to log query: {e}")
            # Don't fail the main operation if logging fails
    
    async def _get_relevant_chunks_for_extraction(self, source_document: str) -> List[tuple]:
        """Get relevant chunks for information extraction from a specific document"""
        # Define search queries for each category
        category_queries = [
            "submitter name email phone contact administrative information",
            "source material DNA RNA genomic biological sample type",
            "pooling multiplexing barcode index sequences",
            "sequencing platform read length coverage library preparation",
            "container tube volume concentration diluent storage",
            "informatics analysis pipeline reference genome computational",
            "sample details quality metrics priority patient identifier"
        ]
        
        all_relevant_chunks = []
        
        for query in category_queries:
            chunks = await self.vector_store.similarity_search(
                query,
                k=3,
                filter_metadata={"source_document": source_document}
            )
            
            # Add chunks with their similarity scores
            for chunk, score in chunks:
                if score >= settings.similarity_threshold:
                    all_relevant_chunks.append((chunk.content, score))
        
        # Remove duplicates and sort by relevance
        unique_chunks = {}
        for content, score in all_relevant_chunks:
            if content not in unique_chunks or score > unique_chunks[content]:
                unique_chunks[content] = score
        
        # Return top chunks sorted by score
        sorted_chunks = sorted(unique_chunks.items(), key=lambda x: x[1], reverse=True)
        return sorted_chunks[:10]  # Limit to top 10 most relevant chunks
    
    async def get_system_status(self) -> Dict[str, Any]:
        """Get status information about the RAG system"""
        try:
            vector_store_info = self.vector_store.get_store_info()
            
            return {
                "status": "operational",
                "vector_store": {
                    "total_documents": vector_store_info.total_documents,
                    "total_chunks": vector_store_info.total_chunks,
                    "embedding_model": vector_store_info.embedding_model,
                    "last_updated": vector_store_info.last_updated.isoformat()
                },
                "configuration": {
                    "chunk_size": settings.chunk_size,
                    "chunk_overlap": settings.chunk_overlap,
                    "similarity_threshold": settings.similarity_threshold,
                    "max_search_results": settings.max_search_results
                },
                "supported_categories": [
                    "Administrative Information",
                    "Source and Submitting Material", 
                    "Pooling (Multiplexing)",
                    "Sequence Generation",
                    "Container and Diluent",
                    "Informatics",
                    "Sample Details"
                ]
            }
            
        except Exception as e:
            logger.error(f"Error getting system status: {str(e)}")
            return {
                "status": "error",
                "error": str(e)
            }
    
    async def export_submission_data(
        self, 
        submission: LabSubmission, 
        format: str = "json"
    ) -> str:
        """
        Export extracted submission data to file.
        
        Args:
            submission: The LabSubmission object to export
            format: Export format ('json', 'csv', 'xlsx')
            
        Returns:
            Path to the exported file
        """
        try:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            export_dir = Path(settings.export_directory)
            
            if format == "json":
                export_path = export_dir / f"submission_{timestamp}.json"
                with open(export_path, 'w') as f:
                    f.write(submission.json(indent=2))
                    
            elif format == "csv":
                import pandas as pd
                export_path = export_dir / f"submission_{timestamp}.csv"
                
                # Flatten the submission data for CSV export
                flat_data = {}
                for category, data in submission.dict().items():
                    if isinstance(data, dict):
                        for key, value in data.items():
                            flat_data[f"{category}_{key}"] = value
                    else:
                        flat_data[category] = data
                
                df = pd.DataFrame([flat_data])
                df.to_csv(export_path, index=False)
                
            else:
                raise ValueError(f"Unsupported export format: {format}")
            
            logger.info(f"Submission data exported to {export_path}")
            return str(export_path)
            
        except Exception as e:
            logger.error(f"Error exporting submission data: {str(e)}")
            raise

# Create global RAG instance
rag_system = LabSubmissionRAG() 
 