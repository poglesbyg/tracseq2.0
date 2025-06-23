"""
Factory classes for creating components in the Laboratory Submission RAG System

This module provides factory classes that encapsulate the creation logic for various
components, allowing for easy configuration and testing.
"""

import logging
from pathlib import Path
from typing import Any, Dict, Optional, Type

from config import settings

from .exceptions import ConfigurationException, ServiceException
from .interfaces import (
    ICircuitBreaker,
    IDocumentProcessor,
    ILLMInterface,
    IRetryPolicy,
    IVectorStore,
)

logger = logging.getLogger(__name__)


class ComponentFactory:
    """Base factory class for creating components"""

    def __init__(self, config: Optional[Dict[str, Any]] = None):
        self.config = config or {}
        self._validate_config()

    def _validate_config(self) -> None:
        """Validate factory configuration"""
        pass

    def _get_config_value(self, key: str, default: Any = None) -> Any:
        """Get configuration value with fallback to settings"""
        return self.config.get(key, getattr(settings, key, default))


class DocumentProcessorFactory(ComponentFactory):
    """Factory for creating document processors"""

    def _validate_config(self) -> None:
        """Validate document processor configuration"""
        required_keys = ["chunk_size", "chunk_overlap"]
        for key in required_keys:
            if not hasattr(settings, key) and key not in self.config:
                raise ConfigurationException(
                    f"Missing required configuration: {key}", config_key=key
                )

    def create(self) -> IDocumentProcessor:
        """Create a document processor instance"""
        try:
            # Import here to avoid circular imports

            # Create with enhanced error handling
            processor = EnhancedDocumentProcessor(
                chunk_size=self._get_config_value("chunk_size", 1000),
                chunk_overlap=self._get_config_value("chunk_overlap", 200),
            )

            logger.info("Created DocumentProcessor instance")
            return processor

        except Exception as e:
            logger.error(f"Failed to create DocumentProcessor: {str(e)}")
            raise ServiceException(
                f"DocumentProcessor creation failed: {str(e)}",
                service_name="DocumentProcessorFactory",
                cause=e,
            )


class VectorStoreFactory(ComponentFactory):
    """Factory for creating vector stores"""

    def _validate_config(self) -> None:
        """Validate vector store configuration"""
        required_keys = ["vector_store_path"]
        for key in required_keys:
            if not hasattr(settings, key) and key not in self.config:
                raise ConfigurationException(
                    f"Missing required configuration: {key}", config_key=key
                )

    def create(self) -> IVectorStore:
        """Create a vector store instance"""
        try:
            # Import here to avoid circular imports

            vector_store = EnhancedVectorStore(
                store_path=self._get_config_value("vector_store_path")
            )

            logger.info("Created VectorStore instance")
            return vector_store

        except Exception as e:
            logger.error(f"Failed to create VectorStore: {str(e)}")
            raise ServiceException(
                f"VectorStore creation failed: {str(e)}", service_name="VectorStoreFactory", cause=e
            )


class LLMInterfaceFactory(ComponentFactory):
    """Factory for creating LLM interfaces"""

    def _validate_config(self) -> None:
        """Validate LLM interface configuration"""
        # Check if at least one provider is configured
        has_provider = any(
            [
                hasattr(settings, "openai_api_key") and settings.openai_api_key,
                hasattr(settings, "anthropic_api_key") and settings.anthropic_api_key,
                hasattr(settings, "use_ollama") and settings.use_ollama,
            ]
        )

        if not has_provider:
            logger.warning("No LLM providers configured, will use mock responses")

    def create(self) -> ILLMInterface:
        """Create an LLM interface instance"""
        try:
            # Import here to avoid circular imports

            llm_interface = EnhancedLLMInterface()

            logger.info("Created LLMInterface instance")
            return llm_interface

        except Exception as e:
            logger.error(f"Failed to create LLMInterface: {str(e)}")
            raise ServiceException(
                f"LLMInterface creation failed: {str(e)}",
                service_name="LLMInterfaceFactory",
                cause=e,
            )


class RetryPolicyFactory(ComponentFactory):
    """Factory for creating retry policies"""

    def create(
        self,
        max_retries: int = 3,
        base_delay: float = 1.0,
        max_delay: float = 60.0,
        exponential_base: float = 2.0,
    ) -> IRetryPolicy:
        """Create a retry policy instance"""
        try:
            retry_policy = ExponentialBackoffRetryPolicy(
                max_retries=max_retries,
                base_delay=base_delay,
                max_delay=max_delay,
                exponential_base=exponential_base,
            )

            logger.info("Created RetryPolicy instance")
            return retry_policy

        except Exception as e:
            logger.error(f"Failed to create RetryPolicy: {str(e)}")
            raise ServiceException(
                f"RetryPolicy creation failed: {str(e)}", service_name="RetryPolicyFactory", cause=e
            )


class CircuitBreakerFactory(ComponentFactory):
    """Factory for creating circuit breakers"""

    def create(
        self,
        failure_threshold: int = 5,
        timeout: float = 60.0,
        expected_exception: Type[Exception] = Exception,
    ) -> ICircuitBreaker:
        """Create a circuit breaker instance"""
        try:
            circuit_breaker = SimpleCircuitBreaker(
                failure_threshold=failure_threshold,
                timeout=timeout,
                expected_exception=expected_exception,
            )

            logger.info("Created CircuitBreaker instance")
            return circuit_breaker

        except Exception as e:
            logger.error(f"Failed to create CircuitBreaker: {str(e)}")
            raise ServiceException(
                f"CircuitBreaker creation failed: {str(e)}",
                service_name="CircuitBreakerFactory",
                cause=e,
            )


# Enhanced implementations with better error handling and resilience


class EnhancedDocumentProcessor(IDocumentProcessor):
    """Enhanced document processor with better error handling"""

    def __init__(self, chunk_size: int = 1000, chunk_overlap: int = 200):
        from rag.document_processor import DocumentProcessor

        self._processor = DocumentProcessor()
        self.chunk_size = chunk_size
        self.chunk_overlap = chunk_overlap

    async def process_document(self, file_path):
        """Process document with enhanced error handling"""
        try:
            return await self._processor.process_document(file_path)
        except Exception as e:
            logger.error(f"Error in enhanced document processing: {str(e)}")
            raise

    async def validate_document(self, file_path):
        """Validate if document can be processed"""
        try:
            file_path = Path(file_path)

            # Check if file exists
            if not file_path.exists():
                return False

            # Check file size (avoid processing very large files)
            max_size = 100 * 1024 * 1024  # 100MB
            if file_path.stat().st_size > max_size:
                logger.warning(f"File {file_path} is too large: {file_path.stat().st_size} bytes")
                return False

            # Check supported formats
            supported_formats = self.get_supported_formats()
            if file_path.suffix.lower() not in supported_formats:
                return False

            return True

        except Exception as e:
            logger.error(f"Error validating document {file_path}: {str(e)}")
            return False

    def get_supported_formats(self):
        """Get list of supported file formats"""
        return [".pdf", ".docx", ".txt"]


class EnhancedVectorStore(IVectorStore):
    """Enhanced vector store with better error handling"""

    def __init__(self, store_path: str):
        from rag.vector_store import VectorStore

        self._vector_store = VectorStore()
        self.store_path = store_path

    async def add_chunks(self, chunks):
        """Add chunks with enhanced error handling"""
        try:
            return await self._vector_store.add_chunks(chunks)
        except Exception as e:
            logger.error(f"Error adding chunks to vector store: {str(e)}")
            raise

    async def similarity_search(self, query, k=5, filter_metadata=None):
        """Search with enhanced error handling"""
        try:
            return await self._vector_store.similarity_search(query, k, filter_metadata)
        except Exception as e:
            logger.error(f"Error in vector store search: {str(e)}")
            raise

    async def delete_by_source(self, source_document):
        """Delete by source with enhanced error handling"""
        try:
            # Implementation would depend on the specific vector store
            logger.info(f"Deleting chunks from source: {source_document}")
        except Exception as e:
            logger.error(f"Error deleting chunks: {str(e)}")
            raise

    async def get_collection_stats(self):
        """Get collection statistics"""
        try:
            # Implementation would depend on the specific vector store
            return {"total_chunks": 0, "total_documents": 0}
        except Exception as e:
            logger.error(f"Error getting collection stats: {str(e)}")
            raise


class EnhancedLLMInterface(ILLMInterface):
    """Enhanced LLM interface with better error handling"""

    def __init__(self):
        from rag.llm_interface import LLMInterface

        self._llm_interface = LLMInterface()

    async def extract_submission_info(self, document_chunks, source_document):
        """Extract submission info with enhanced error handling"""
        try:
            return await self._llm_interface.extract_submission_info(
                document_chunks, source_document
            )
        except Exception as e:
            logger.error(f"Error in LLM extraction: {str(e)}")
            raise

    async def answer_query(self, query, relevant_chunks, submission_data=None):
        """Answer query with enhanced error handling"""
        try:
            return await self._llm_interface.answer_query(query, relevant_chunks, submission_data)
        except Exception as e:
            logger.error(f"Error in LLM query: {str(e)}")
            raise

    def get_provider_info(self):
        """Get provider information"""
        try:
            return {
                "provider": getattr(self._llm_interface, "client_type", "unknown"),
                "status": "active",
            }
        except Exception as e:
            logger.error(f"Error getting provider info: {str(e)}")
            return {"provider": "unknown", "status": "error"}

    async def health_check(self):
        """Check LLM provider health"""
        try:
            # Simple health check - could be enhanced based on provider
            return hasattr(self._llm_interface, "client_type")
        except Exception as e:
            logger.error(f"LLM health check failed: {str(e)}")
            return False


# Resilience implementations

import asyncio
import random
from datetime import datetime, timedelta


class ExponentialBackoffRetryPolicy(IRetryPolicy):
    """Exponential backoff retry policy implementation"""

    def __init__(
        self,
        max_retries: int = 3,
        base_delay: float = 1.0,
        max_delay: float = 60.0,
        exponential_base: float = 2.0,
    ):
        self.max_retries = max_retries
        self.base_delay = base_delay
        self.max_delay = max_delay
        self.exponential_base = exponential_base

    def configure(self, max_retries=3, base_delay=1.0, max_delay=60.0, exponential_base=2.0):
        """Configure retry parameters"""
        self.max_retries = max_retries
        self.base_delay = base_delay
        self.max_delay = max_delay
        self.exponential_base = exponential_base

    async def execute(self, func, *args, **kwargs):
        """Execute function with retry policy"""
        last_exception = None

        for attempt in range(self.max_retries + 1):
            try:
                return await func(*args, **kwargs)
            except Exception as e:
                last_exception = e

                if attempt < self.max_retries:
                    delay = min(self.base_delay * (self.exponential_base**attempt), self.max_delay)
                    # Add jitter to prevent thundering herd
                    jitter = random.uniform(0, 0.1) * delay
                    total_delay = delay + jitter

                    logger.warning(
                        f"Attempt {attempt + 1} failed, retrying in {total_delay:.2f}s: {str(e)}"
                    )
                    await asyncio.sleep(total_delay)
                else:
                    logger.error(f"All retry attempts failed: {str(e)}")

        raise last_exception


class SimpleCircuitBreaker(ICircuitBreaker):
    """Simple circuit breaker implementation"""

    def __init__(
        self,
        failure_threshold: int = 5,
        timeout: float = 60.0,
        expected_exception: Type[Exception] = Exception,
    ):
        self.failure_threshold = failure_threshold
        self.timeout = timeout
        self.expected_exception = expected_exception

        self.failure_count = 0
        self.last_failure_time = None
        self.state = "CLOSED"  # CLOSED, OPEN, HALF_OPEN

    async def call(self, func, *args, **kwargs):
        """Execute function with circuit breaker protection"""
        if self.state == "OPEN":
            if self._should_attempt_reset():
                self.state = "HALF_OPEN"
            else:
                from .exceptions import CircuitBreakerException

                raise CircuitBreakerException(
                    "Circuit breaker is OPEN", failure_count=self.failure_count
                )

        try:
            result = await func(*args, **kwargs)
            self._on_success()
            return result

        except self.expected_exception:
            self._on_failure()
            raise

    def _should_attempt_reset(self) -> bool:
        """Check if circuit breaker should attempt reset"""
        if self.last_failure_time is None:
            return True

        return datetime.utcnow() - self.last_failure_time > timedelta(seconds=self.timeout)

    def _on_success(self):
        """Handle successful execution"""
        self.failure_count = 0
        self.state = "CLOSED"

    def _on_failure(self):
        """Handle failed execution"""
        self.failure_count += 1
        self.last_failure_time = datetime.utcnow()

        if self.failure_count >= self.failure_threshold:
            self.state = "OPEN"

    def get_state(self) -> str:
        """Get current circuit breaker state"""
        return self.state

    def reset(self):
        """Reset circuit breaker"""
        self.failure_count = 0
        self.last_failure_time = None
        self.state = "CLOSED"
