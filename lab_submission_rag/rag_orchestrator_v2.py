"""
Enhanced RAG Orchestrator for Laboratory Submission Information Extraction

This is the improved version of the RAG orchestrator that demonstrates the
enhanced modularity, robustness, and maintainability achieved through:
- Dependency injection
- Service layer abstraction
- Proper error handling
- Circuit breaker and retry patterns
- Health monitoring
- Structured logging
"""

import asyncio
import logging
import time
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional, Union

from config import settings
from core import ServiceContainer
from core.exceptions import (
    DocumentProcessingException,
    ExtractionException,
    LabSubmissionException,
    ServiceException,
)
from models.submission import BatchExtractionResult, ExtractionResult, LabSubmission

# Configure structured logging
logging.basicConfig(
    level=getattr(logging, settings.log_level.upper(), logging.INFO),
    format="%(asctime)s - %(name)s - %(levelname)s - %(funcName)s:%(lineno)d - %(message)s",
    handlers=[
        logging.StreamHandler(),
        (
            logging.FileHandler("logs/orchestrator_v2.log")
            if hasattr(settings, "log_dir")
            else logging.StreamHandler()
        ),
    ],
)
logger = logging.getLogger(__name__)


class EnhancedLabSubmissionRAG:
    """
    Enhanced RAG system for extracting laboratory submission information.

    This enhanced version provides:
    - Dependency injection for better testability
    - Service layer abstraction for cleaner separation of concerns
    - Comprehensive error handling with custom exception hierarchy
    - Circuit breaker and retry patterns for resilience
    - Health monitoring and metrics collection
    - Structured logging for better observability
    """

    def __init__(self, config: Optional[Dict[str, Any]] = None):
        """Initialize the enhanced RAG system"""
        logger.info("Initializing Enhanced Lab Submission RAG system...")

        # Initialize service container
        self.container = ServiceContainer(config)
        self._initialized = False

        logger.info("Enhanced Lab Submission RAG system created successfully")

    async def initialize(self) -> None:
        """Initialize the RAG system and all dependencies"""
        if self._initialized:
            logger.warning("System already initialized")
            return

        try:
            logger.info("Starting system initialization...")
            start_time = time.time()

            # Initialize service container
            await self.container.initialize()

            # Verify system health
            health_status = await self.health_check()
            if health_status["status"] != "healthy":
                logger.warning(f"System initialized with health issues: {health_status}")

            self._initialized = True
            initialization_time = time.time() - start_time

            logger.info(f"System initialization completed in {initialization_time:.2f}s")

        except Exception as e:
            logger.error(f"System initialization failed: {str(e)}")
            raise ServiceException(
                f"Failed to initialize Enhanced RAG system: {str(e)}",
                service_name="EnhancedLabSubmissionRAG",
                operation="initialize",
                cause=e,
            )

    async def process_document(self, file_path: Union[str, Path]) -> ExtractionResult:
        """
        Process a single laboratory document with enhanced error handling and monitoring.

        Args:
            file_path: Path to the document to process

        Returns:
            ExtractionResult containing extracted submission information

        Raises:
            DocumentProcessingException: When document processing fails
            ExtractionException: When information extraction fails
            ServiceException: When service layer operations fail
        """
        self._ensure_initialized()

        file_path = Path(file_path)
        logger.info(f"Processing document: {file_path}")

        # Validate input
        if not file_path.exists():
            raise DocumentProcessingException(
                f"Document not found: {file_path}",
                file_path=str(file_path),
                error_code="FILE_NOT_FOUND",
            )

        try:
            # Use the service layer
            submission_service = self.container.get_submission_service()
            result = await submission_service.process_document(file_path)

            logger.info(
                f"Document processing completed. Success: {result.success}, "
                f"Confidence: {result.confidence_score:.2f}, "
                f"Time: {result.processing_time:.2f}s"
            )

            return result

        except (DocumentProcessingException, ExtractionException) as e:
            # Re-raise domain exceptions as-is
            logger.error(f"Domain error processing document: {e.to_dict()}")
            raise

        except Exception as e:
            logger.error(f"Unexpected error processing document: {str(e)}")
            raise ServiceException(
                f"Unexpected error processing document: {str(e)}",
                service_name="EnhancedLabSubmissionRAG",
                operation="process_document",
                context={"file_path": str(file_path)},
                cause=e,
            )

    async def process_documents_batch(
        self, file_paths: List[Union[str, Path]], max_concurrent: int = 3
    ) -> BatchExtractionResult:
        """
        Process multiple laboratory documents with enhanced batch processing.

        Args:
            file_paths: List of document paths to process
            max_concurrent: Maximum number of concurrent document processing operations

        Returns:
            BatchExtractionResult containing results for all documents
        """
        self._ensure_initialized()

        logger.info(f"Starting batch processing of {len(file_paths)} documents")

        if not file_paths:
            logger.warning("No file paths provided for batch processing")
            return BatchExtractionResult(
                total_documents=0,
                successful_extractions=0,
                failed_extractions=0,
                results=[],
                overall_confidence=0.0,
                processing_time=0.0,
            )

        try:
            # Use the service layer with controlled concurrency
            submission_service = self.container.get_submission_service()

            # Limit concurrency to avoid overwhelming the system
            semaphore = asyncio.Semaphore(max_concurrent)

            async def process_with_semaphore(file_path):
                async with semaphore:
                    return await submission_service.process_document(file_path)

            # Process documents with controlled concurrency
            start_time = time.time()
            tasks = [process_with_semaphore(file_path) for file_path in file_paths]
            results = await asyncio.gather(*tasks, return_exceptions=True)

            # Process results
            extraction_results = []
            successful_extractions = 0

            for i, result in enumerate(results):
                if isinstance(result, Exception):
                    logger.error(f"Batch processing error for {file_paths[i]}: {str(result)}")
                    extraction_results.append(
                        ExtractionResult(
                            success=False,
                            confidence_score=0.0,
                            missing_fields=[],
                            warnings=[f"Processing error: {str(result)}"],
                            processing_time=0.0,
                            source_document=str(file_paths[i]),
                        )
                    )
                else:
                    extraction_results.append(result)
                    if result.success:
                        successful_extractions += 1

            # Calculate overall confidence
            successful_results = [r for r in extraction_results if r.success]
            overall_confidence = (
                sum(r.confidence_score for r in successful_results) / len(successful_results)
                if successful_results
                else 0.0
            )

            processing_time = time.time() - start_time

            batch_result = BatchExtractionResult(
                total_documents=len(file_paths),
                successful_extractions=successful_extractions,
                failed_extractions=len(file_paths) - successful_extractions,
                results=extraction_results,
                overall_confidence=overall_confidence,
                processing_time=processing_time,
            )

            logger.info(
                f"Batch processing completed. {successful_extractions}/{len(file_paths)} successful, "
                f"Overall confidence: {overall_confidence:.2f}, "
                f"Time: {processing_time:.2f}s"
            )

            return batch_result

        except Exception as e:
            logger.error(f"Batch processing failed: {str(e)}")
            raise ServiceException(
                f"Batch processing failed: {str(e)}",
                service_name="EnhancedLabSubmissionRAG",
                operation="process_documents_batch",
                context={"file_count": len(file_paths)},
                cause=e,
            )

    async def query_submissions(
        self,
        query: str,
        filter_metadata: Optional[Dict[str, Any]] = None,
        session_id: str = "default",
    ) -> str:
        """
        Answer questions about laboratory submissions with enhanced query processing.

        Args:
            query: Natural language query about submissions
            filter_metadata: Optional metadata filters for search
            session_id: Session ID for conversation context

        Returns:
            Natural language answer based on stored submission data
        """
        self._ensure_initialized()

        logger.info(f"Processing query: '{query}' (session: {session_id})")

        if not query or not query.strip():
            raise ServiceException(
                "Query cannot be empty",
                service_name="EnhancedLabSubmissionRAG",
                operation="query_submissions",
                error_code="EMPTY_QUERY",
            )

        try:
            # Use the service layer
            submission_service = self.container.get_submission_service()
            answer = await submission_service.query_submissions(query, filter_metadata, session_id)

            logger.info(f"Query processed successfully (session: {session_id})")
            return answer

        except Exception as e:
            logger.error(f"Query processing failed: {str(e)}")
            raise ServiceException(
                f"Query processing failed: {str(e)}",
                service_name="EnhancedLabSubmissionRAG",
                operation="query_submissions",
                context={"query": query, "session_id": session_id},
                cause=e,
            )

    async def get_submission(self, submission_id: str) -> Optional[LabSubmission]:
        """Get submission by ID with enhanced error handling"""
        self._ensure_initialized()

        logger.debug(f"Retrieving submission: {submission_id}")

        try:
            submission_service = self.container.get_submission_service()
            return await submission_service.get_submission(submission_id)

        except Exception as e:
            logger.error(f"Failed to retrieve submission {submission_id}: {str(e)}")
            raise ServiceException(
                f"Failed to retrieve submission: {str(e)}",
                service_name="EnhancedLabSubmissionRAG",
                operation="get_submission",
                context={"submission_id": submission_id},
                cause=e,
            )

    async def search_submissions(self, criteria: Dict[str, Any]) -> List[LabSubmission]:
        """Search submissions by criteria with enhanced error handling"""
        self._ensure_initialized()

        logger.debug(f"Searching submissions with criteria: {criteria}")

        try:
            submission_service = self.container.get_submission_service()
            return await submission_service.search_submissions(criteria)

        except Exception as e:
            logger.error(f"Search failed: {str(e)}")
            raise ServiceException(
                f"Search failed: {str(e)}",
                service_name="EnhancedLabSubmissionRAG",
                operation="search_submissions",
                context={"criteria": criteria},
                cause=e,
            )

    async def health_check(self) -> Dict[str, Any]:
        """
        Perform comprehensive health check of all system components.

        Returns:
            Dictionary containing health status of all components
        """
        logger.debug("Performing system health check")

        try:
            if not self._initialized:
                return {
                    "status": "unhealthy",
                    "error": "System not initialized",
                    "timestamp": datetime.utcnow().isoformat(),
                }

            # Use container health check
            health_status = await self.container.health_check()

            # Add system-level information
            health_status.update(
                {
                    "system": "Enhanced Lab Submission RAG",
                    "version": "2.0",
                    "initialized": self._initialized,
                    "timestamp": datetime.utcnow().isoformat(),
                }
            )

            logger.debug(f"Health check completed: {health_status['status']}")
            return health_status

        except Exception as e:
            logger.error(f"Health check failed: {str(e)}")
            return {
                "status": "unhealthy",
                "error": str(e),
                "timestamp": datetime.utcnow().isoformat(),
            }

    async def get_system_statistics(self) -> Dict[str, Any]:
        """Get comprehensive system statistics"""
        self._ensure_initialized()

        try:
            # Get health status
            health = await self.health_check()

            # Get submission statistics
            submission_service = self.container.get_submission_service()

            stats = {
                "system_status": health["status"],
                "components": len(health.get("services", {})),
                "uptime": "N/A",  # Would track from initialization
                "timestamp": datetime.utcnow().isoformat(),
            }

            return stats

        except Exception as e:
            logger.error(f"Failed to get system statistics: {str(e)}")
            raise ServiceException(
                f"Failed to get system statistics: {str(e)}",
                service_name="EnhancedLabSubmissionRAG",
                operation="get_system_statistics",
                cause=e,
            )

    async def shutdown(self) -> None:
        """Gracefully shutdown the RAG system"""
        logger.info("Shutting down Enhanced Lab Submission RAG system...")

        try:
            if self.container:
                await self.container.shutdown()

            self._initialized = False
            logger.info("System shutdown completed successfully")

        except Exception as e:
            logger.error(f"Error during system shutdown: {str(e)}")
            raise ServiceException(
                f"System shutdown failed: {str(e)}",
                service_name="EnhancedLabSubmissionRAG",
                operation="shutdown",
                cause=e,
            )

    def _ensure_initialized(self) -> None:
        """Ensure the system is initialized before operations"""
        if not self._initialized:
            raise ServiceException(
                "System not initialized. Call initialize() first.",
                service_name="EnhancedLabSubmissionRAG",
                operation="_ensure_initialized",
                error_code="NOT_INITIALIZED",
            )

    # Context manager support for automatic lifecycle management
    async def __aenter__(self):
        """Async context manager entry"""
        await self.initialize()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit"""
        await self.shutdown()


# Convenience function for easy usage
async def create_enhanced_rag_system(
    config: Optional[Dict[str, Any]] = None,
) -> EnhancedLabSubmissionRAG:
    """
    Create and initialize an enhanced RAG system.

    Args:
        config: Optional configuration dictionary

    Returns:
        Initialized EnhancedLabSubmissionRAG instance
    """
    rag_system = EnhancedLabSubmissionRAG(config)
    await rag_system.initialize()
    return rag_system


# Example usage demonstration
async def main():
    """Demonstrate the enhanced RAG system usage"""
    logger.info("Starting Enhanced RAG System demonstration")

    try:
        # Using context manager for automatic lifecycle management
        async with EnhancedLabSubmissionRAG() as rag_system:

            # Health check
            health = await rag_system.health_check()
            logger.info(f"System health: {health['status']}")

            # Example document processing (if you have test files)
            # result = await rag_system.process_document("test_document.pdf")

            # Example query
            answer = await rag_system.query_submissions("How many submissions are in the system?")
            logger.info(f"Query answer: {answer}")

            # System statistics
            stats = await rag_system.get_system_statistics()
            logger.info(f"System stats: {stats}")

    except LabSubmissionException as e:
        logger.error(f"Domain error: {e.to_dict()}")
    except Exception as e:
        logger.error(f"Unexpected error: {str(e)}")


if __name__ == "__main__":
    asyncio.run(main())
