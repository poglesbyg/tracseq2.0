"""
Service layer implementations for the Laboratory Submission RAG System

This module contains the business logic services that coordinate between different
components and provide high-level operations for the application.
"""

import asyncio
import logging
import time
from pathlib import Path
from typing import Any

from models.submission import BatchExtractionResult, ExtractionResult, LabSubmission

from .exceptions import (
    DatabaseException,
    DocumentProcessingException,
    ExtractionException,
    ServiceException,
    ValidationException,
)
from .interfaces import (
    ICircuitBreaker,
    IDocumentProcessor,
    ILLMInterface,
    IMetricsCollector,
    IRetryPolicy,
    ISubmissionRepository,
    ISubmissionService,
    IVectorStore,
)

logger = logging.getLogger(__name__)


class SubmissionService(ISubmissionService):
    """
    High-level service for processing laboratory submissions

    This service coordinates document processing, information extraction,
    and data persistence operations.
    """

    def __init__(
        self,
        document_processor: IDocumentProcessor,
        vector_store: IVectorStore,
        llm_interface: ILLMInterface,
        submission_repository: ISubmissionRepository,
        metrics_collector: IMetricsCollector | None = None,
        circuit_breaker: ICircuitBreaker | None = None,
        retry_policy: IRetryPolicy | None = None,
        batch_size: int = 5,
    ) -> None:
        self.document_processor = document_processor
        self.vector_store = vector_store
        self.llm_interface = llm_interface
        self.submission_repository = submission_repository
        self.metrics_collector = metrics_collector
        self.circuit_breaker = circuit_breaker
        self.retry_policy = retry_policy
        self.batch_size = batch_size

        logger.info("SubmissionService initialized")

    async def process_document(self, file_path: str | Path) -> ExtractionResult:
        """Process a single laboratory document and extract submission information"""
        start_time = time.time()
        file_path = Path(file_path)

        # Record metrics
        if self.metrics_collector:
            self.metrics_collector.increment_counter(
                "submission.document.processing.started", {"file_type": file_path.suffix}
            )

        logger.info(f"Processing document: {file_path}")

        try:
            # Step 1: Validate and process document
            if not await self.document_processor.validate_document(file_path):
                raise DocumentProcessingException(
                    f"Document validation failed: {file_path}",
                    file_path=str(file_path),
                    file_type=file_path.suffix,
                )

            document_chunks = await self._execute_with_resilience(
                self.document_processor.process_document, file_path, operation="document_processing"
            )

            if not document_chunks:
                raise DocumentProcessingException(
                    f"No content extracted from document: {file_path}", file_path=str(file_path)
                )

            logger.info(f"Extracted {len(document_chunks)} chunks from document")

            # Step 2: Add chunks to vector store
            await self._execute_with_resilience(
                self.vector_store.add_chunks, document_chunks, operation="vector_store_add"
            )

            # Step 3: Get relevant chunks for extraction
            relevant_chunks = await self._get_relevant_chunks_for_extraction(str(file_path))

            # Step 4: Extract submission information
            extraction_result = await self._execute_with_resilience(
                self.llm_interface.extract_submission_info,
                relevant_chunks,
                str(file_path),
                operation="llm_extraction",
            )

            # Step 5: Save to database if successful
            if extraction_result.success and extraction_result.submission:
                try:
                    await self._save_extraction_to_database(
                        extraction_result, document_chunks, file_path
                    )
                    logger.info("Successfully saved extraction results to database")
                except Exception as db_error:
                    logger.error(f"Failed to save to database: {db_error}")
                    extraction_result.warnings.append(f"Database save failed: {str(db_error)}")

            # Update processing time and record metrics
            processing_time = time.time() - start_time
            extraction_result.processing_time = processing_time

            if self.metrics_collector:
                self.metrics_collector.record_timing(
                    "submission.document.processing.duration",
                    processing_time,
                    {"success": str(extraction_result.success)},
                )
                self.metrics_collector.record_gauge(
                    "submission.document.confidence_score", extraction_result.confidence_score
                )

            logger.info(f"Document processing completed. Success: {extraction_result.success}")
            return extraction_result

        except Exception as e:
            processing_time = time.time() - start_time

            if self.metrics_collector:
                self.metrics_collector.increment_counter(
                    "submission.document.processing.failed", {"error_type": type(e).__name__}
                )

            logger.error(f"Error processing document {file_path}: {str(e)}")

            if isinstance(e, (DocumentProcessingException, ExtractionException)):
                raise
            else:
                raise ServiceException(
                    f"Unexpected error processing document: {str(e)}",
                    service_name="SubmissionService",
                    operation="process_document",
                    cause=e,
                )

    async def process_documents_batch(
        self, file_paths: list[str | Path]
    ) -> BatchExtractionResult:
        """Process multiple laboratory documents in batch"""
        start_time = time.time()
        logger.info(f"Starting batch processing of {len(file_paths)} documents")

        if self.metrics_collector:
            self.metrics_collector.increment_counter(
                "submission.batch.processing.started", {"document_count": str(len(file_paths))}
            )

        results = []
        successful_extractions = 0

        try:
            # Process documents in batches to avoid overwhelming the system
            for i in range(0, len(file_paths), self.batch_size):
                batch = file_paths[i : i + self.batch_size]
                batch_tasks = [self.process_document(file_path) for file_path in batch]
                batch_results = await asyncio.gather(*batch_tasks, return_exceptions=True)

                for result in batch_results:
                    if isinstance(result, Exception):
                        logger.error(f"Batch processing error: {str(result)}")
                        results.append(
                            ExtractionResult(
                                success=False,
                                confidence_score=0.0,
                                missing_fields=[],
                                warnings=[f"Batch processing error: {str(result)}"],
                                processing_time=0.0,
                                source_document="unknown",
                            )
                        )
                    else:
                        results.append(result)
                        if result.success:
                            successful_extractions += 1

            # Calculate overall confidence
            successful_results = [r for r in results if r.success]
            overall_confidence = (
                sum(r.confidence_score for r in successful_results) / len(successful_results)
                if successful_results
                else 0.0
            )

            processing_time = time.time() - start_time

            if self.metrics_collector:
                self.metrics_collector.record_timing(
                    "submission.batch.processing.duration", processing_time
                )
                self.metrics_collector.record_gauge(
                    "submission.batch.success_rate", successful_extractions / len(file_paths)
                )

            logger.info(
                f"Batch processing completed. {successful_extractions}/{len(file_paths)} successful"
            )

            return BatchExtractionResult(
                total_documents=len(file_paths),
                successful_extractions=successful_extractions,
                failed_extractions=len(file_paths) - successful_extractions,
                results=results,
                overall_confidence=overall_confidence,
                processing_time=processing_time,
            )

        except Exception as e:
            logger.error(f"Error in batch processing: {str(e)}")
            raise ServiceException(
                f"Batch processing failed: {str(e)}",
                service_name="SubmissionService",
                operation="process_documents_batch",
                cause=e,
            )

    async def query_submissions(
        self,
        query: str,
        filter_metadata: dict[str, Any] | None = None,
        session_id: str = "default",
    ) -> str:
        """Query submissions using natural language"""
        start_time = time.time()
        logger.info(f"Processing query: {query}")

        if self.metrics_collector:
            self.metrics_collector.increment_counter("submission.query.started")

        try:
            # Check if this is a database-specific query
            db_answer = await self._handle_database_query(query)
            if db_answer:
                if self.metrics_collector:
                    self.metrics_collector.record_timing(
                        "submission.query.duration", time.time() - start_time, {"type": "database"}
                    )
                return db_answer

            # Search for relevant chunks in vector store
            relevant_chunks = await self._execute_with_resilience(
                self.vector_store.similarity_search,
                query,
                5,  # k
                filter_metadata,
                operation="vector_search",
            )

            # Convert to format expected by LLM interface
            chunks_with_scores = [(chunk.content, score) for chunk, score in relevant_chunks]

            # Get answer from LLM
            answer = await self._execute_with_resilience(
                self.llm_interface.answer_query,
                query,
                chunks_with_scores,
                None,  # submission_data
                operation="llm_query",
            )

            if self.metrics_collector:
                self.metrics_collector.record_timing(
                    "submission.query.duration", time.time() - start_time, {"type": "rag"}
                )

            logger.info("Query processed successfully")
            return answer

        except Exception as e:
            logger.error(f"Error processing query: {str(e)}")

            if self.metrics_collector:
                self.metrics_collector.increment_counter(
                    "submission.query.failed", {"error_type": type(e).__name__}
                )

            raise ServiceException(
                f"Query processing failed: {str(e)}",
                service_name="SubmissionService",
                operation="query_submissions",
                cause=e,
            )

    async def get_submission(self, submission_id: str) -> LabSubmission | None:
        """Get submission by ID"""
        try:
            return await self._execute_with_resilience(
                self.submission_repository.get_submission, submission_id, operation="database_get"
            )
        except Exception as e:
            logger.error(f"Error getting submission {submission_id}: {str(e)}")
            raise ServiceException(
                f"Failed to get submission: {str(e)}",
                service_name="SubmissionService",
                operation="get_submission",
                cause=e,
            )

    async def search_submissions(self, criteria: dict[str, Any]) -> list[LabSubmission]:
        """Search submissions by criteria"""
        try:
            return await self._execute_with_resilience(
                self.submission_repository.search_submissions, criteria, operation="database_search"
            )
        except Exception as e:
            logger.error(f"Error searching submissions: {str(e)}")
            raise ServiceException(
                f"Failed to search submissions: {str(e)}",
                service_name="SubmissionService",
                operation="search_submissions",
                cause=e,
            )

    async def _execute_with_resilience(self, func, *args, operation: str = "unknown", **kwargs) -> None:
        """Execute function with circuit breaker and retry policies"""
        try:
            # Apply circuit breaker if available
            if self.circuit_breaker:
                return await self.circuit_breaker.call(func, *args, **kwargs)

            # Apply retry policy if available
            if self.retry_policy:
                return await self.retry_policy.execute(func, *args, **kwargs)

            # Execute normally
            return await func(*args, **kwargs)

        except Exception as e:
            logger.error(f"Error in {operation}: {str(e)}")
            raise

    async def _get_relevant_chunks_for_extraction(self, source_document: str) -> list[tuple]:
        """Get relevant chunks for information extraction"""
        category_queries = [
            "submitter name email phone contact administrative information",
            "source material DNA RNA genomic biological sample type",
            "pooling multiplexing barcode index sequences",
            "sequencing platform read length coverage library preparation",
            "container tube volume concentration diluent storage",
            "informatics analysis pipeline reference genome computational",
            "sample details quality metrics priority patient identifier",
        ]

        all_relevant_chunks = []

        for query in category_queries:
            chunks = await self.vector_store.similarity_search(
                query, k=3, filter_metadata={"source_document": source_document}
            )

            for chunk, score in chunks:
                if score >= 0.7:  # Similarity threshold
                    all_relevant_chunks.append((chunk.content, score))

        # Remove duplicates and sort by relevance
        unique_chunks = {}
        for content, score in all_relevant_chunks:
            if content not in unique_chunks or score > unique_chunks[content]:
                unique_chunks[content] = score

        return [(content, score) for content, score in unique_chunks.items()]

    async def _handle_database_query(self, query: str) -> str | None:
        """Handle database-specific queries"""
        query_lower = query.lower()

        try:
            # Sample count queries
            if any(
                phrase in query_lower
                for phrase in ["how many samples", "sample count", "number of samples"]
            ):
                stats = await self.submission_repository.get_submission_statistics()
                return f"There are **{stats.get('total_samples', 0)}** total samples in the system."

            # Submission count queries
            elif any(
                phrase in query_lower for phrase in ["how many submissions", "submission count"]
            ):
                stats = await self.submission_repository.get_submission_statistics()
                return f"There are **{stats.get('total_submissions', 0)}** total submissions in the system."

        except Exception as e:
            logger.error(f"Error handling database query: {e}")
            return None

        return None

    async def _save_extraction_to_database(
        self, extraction_result: ExtractionResult, document_chunks: list, file_path: Path
    ):
        """Save extraction results to database"""
        try:
            if not extraction_result.submission:
                raise ValidationException("No submission data to save")

            submission_id = await self.submission_repository.create_submission(
                extraction_result.submission
            )

            logger.info(f"Saved submission to database with ID: {submission_id}")

        except Exception as e:
            logger.error(f"Error saving to database: {str(e)}")
            raise DatabaseException(
                f"Failed to save extraction results: {str(e)}", operation="save_extraction", cause=e
            )
