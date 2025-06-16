"""
Interface definitions for the Laboratory Submission RAG System

This module defines the contracts (interfaces) for all major services in the system,
enabling dependency injection, better testability, and loose coupling between components.
"""

from abc import ABC, abstractmethod
from typing import List, Dict, Any, Optional, Union, Tuple
from pathlib import Path

from models.rag_models import DocumentChunk
from models.submission import LabSubmission, ExtractionResult, BatchExtractionResult


class IDocumentProcessor(ABC):
    """Interface for document processing operations"""
    
    @abstractmethod
    async def process_document(self, file_path: Union[str, Path]) -> List[DocumentChunk]:
        """Process a document and return chunks"""
        pass
    
    @abstractmethod
    async def validate_document(self, file_path: Union[str, Path]) -> bool:
        """Validate if document can be processed"""
        pass
    
    @abstractmethod
    def get_supported_formats(self) -> List[str]:
        """Get list of supported file formats"""
        pass


class IVectorStore(ABC):
    """Interface for vector store operations"""
    
    @abstractmethod
    async def add_chunks(self, chunks: List[DocumentChunk]) -> None:
        """Add document chunks to vector store"""
        pass
    
    @abstractmethod
    async def similarity_search(
        self, 
        query: str, 
        k: int = 5,
        filter_metadata: Optional[Dict[str, Any]] = None
    ) -> List[Tuple[DocumentChunk, float]]:
        """Search for similar chunks"""
        pass
    
    @abstractmethod
    async def delete_by_source(self, source_document: str) -> None:
        """Delete chunks by source document"""
        pass
    
    @abstractmethod
    async def get_collection_stats(self) -> Dict[str, Any]:
        """Get statistics about the vector store collection"""
        pass


class ILLMInterface(ABC):
    """Interface for LLM operations"""
    
    @abstractmethod
    async def extract_submission_info(
        self, 
        document_chunks: List[Tuple[str, float]], 
        source_document: str
    ) -> ExtractionResult:
        """Extract submission information from document chunks"""
        pass
    
    @abstractmethod
    async def answer_query(
        self, 
        query: str, 
        relevant_chunks: List[Tuple[str, float]],
        submission_data: Optional[LabSubmission] = None
    ) -> str:
        """Answer questions using RAG"""
        pass
    
    @abstractmethod
    def get_provider_info(self) -> Dict[str, Any]:
        """Get information about the LLM provider"""
        pass
    
    @abstractmethod
    async def health_check(self) -> bool:
        """Check if LLM provider is healthy"""
        pass


class ISubmissionRepository(ABC):
    """Interface for submission data access"""
    
    @abstractmethod
    async def create_submission(self, submission: LabSubmission) -> str:
        """Create a new submission"""
        pass
    
    @abstractmethod
    async def get_submission(self, submission_id: str) -> Optional[LabSubmission]:
        """Get submission by ID"""
        pass
    
    @abstractmethod
    async def update_submission(self, submission: LabSubmission) -> bool:
        """Update existing submission"""
        pass
    
    @abstractmethod
    async def delete_submission(self, submission_id: str) -> bool:
        """Delete submission by ID"""
        pass
    
    @abstractmethod
    async def search_submissions(
        self, 
        criteria: Dict[str, Any],
        limit: int = 100,
        offset: int = 0
    ) -> List[LabSubmission]:
        """Search submissions by criteria"""
        pass
    
    @abstractmethod
    async def get_submission_statistics(self) -> Dict[str, Any]:
        """Get submission statistics"""
        pass


class ISubmissionService(ABC):
    """Interface for high-level submission processing operations"""
    
    @abstractmethod
    async def process_document(self, file_path: Union[str, Path]) -> ExtractionResult:
        """Process a single document"""
        pass
    
    @abstractmethod
    async def process_documents_batch(
        self, 
        file_paths: List[Union[str, Path]]
    ) -> BatchExtractionResult:
        """Process multiple documents in batch"""
        pass
    
    @abstractmethod
    async def query_submissions(
        self, 
        query: str, 
        filter_metadata: Optional[Dict[str, Any]] = None,
        session_id: str = "default"
    ) -> str:
        """Query submissions using natural language"""
        pass
    
    @abstractmethod
    async def get_submission(self, submission_id: str) -> Optional[LabSubmission]:
        """Get submission by ID"""
        pass
    
    @abstractmethod
    async def search_submissions(
        self, 
        criteria: Dict[str, Any]
    ) -> List[LabSubmission]:
        """Search submissions by criteria"""
        pass


class ICircuitBreaker(ABC):
    """Interface for circuit breaker pattern"""
    
    @abstractmethod
    async def call(self, func, *args, **kwargs):
        """Execute function with circuit breaker protection"""
        pass
    
    @abstractmethod
    def get_state(self) -> str:
        """Get current circuit breaker state"""
        pass
    
    @abstractmethod
    def reset(self) -> None:
        """Reset circuit breaker"""
        pass


class IRetryPolicy(ABC):
    """Interface for retry policies"""
    
    @abstractmethod
    async def execute(self, func, *args, **kwargs):
        """Execute function with retry policy"""
        pass
    
    @abstractmethod
    def configure(
        self, 
        max_retries: int = 3,
        base_delay: float = 1.0,
        max_delay: float = 60.0,
        exponential_base: float = 2.0
    ) -> None:
        """Configure retry parameters"""
        pass


class IMetricsCollector(ABC):
    """Interface for metrics collection"""
    
    @abstractmethod
    def increment_counter(self, name: str, tags: Optional[Dict[str, str]] = None) -> None:
        """Increment a counter metric"""
        pass
    
    @abstractmethod
    def record_gauge(self, name: str, value: float, tags: Optional[Dict[str, str]] = None) -> None:
        """Record a gauge metric"""
        pass
    
    @abstractmethod
    def record_timing(self, name: str, duration: float, tags: Optional[Dict[str, str]] = None) -> None:
        """Record a timing metric"""
        pass
    
    @abstractmethod
    def record_histogram(self, name: str, value: float, tags: Optional[Dict[str, str]] = None) -> None:
        """Record a histogram metric"""
        pass


class IHealthChecker(ABC):
    """Interface for health checking components"""
    
    @abstractmethod
    async def check_health(self) -> Dict[str, Any]:
        """Check health of the component"""
        pass
    
    @abstractmethod
    def get_component_name(self) -> str:
        """Get name of the component being checked"""
        pass


class IEventPublisher(ABC):
    """Interface for publishing domain events"""
    
    @abstractmethod
    async def publish(self, event_type: str, data: Dict[str, Any]) -> None:
        """Publish an event"""
        pass
    
    @abstractmethod
    def subscribe(self, event_type: str, handler) -> None:
        """Subscribe to events of a specific type"""
        pass


class ICacheProvider(ABC):
    """Interface for caching operations"""
    
    @abstractmethod
    async def get(self, key: str) -> Optional[Any]:
        """Get value from cache"""
        pass
    
    @abstractmethod
    async def set(self, key: str, value: Any, ttl: Optional[int] = None) -> None:
        """Set value in cache"""
        pass
    
    @abstractmethod
    async def delete(self, key: str) -> None:
        """Delete value from cache"""
        pass
    
    @abstractmethod
    async def clear(self) -> None:
        """Clear all cache entries"""
        pass


class IConfigurationProvider(ABC):
    """Interface for configuration management"""
    
    @abstractmethod
    def get(self, key: str, default: Any = None) -> Any:
        """Get configuration value"""
        pass
    
    @abstractmethod
    def set(self, key: str, value: Any) -> None:
        """Set configuration value"""
        pass
    
    @abstractmethod
    def reload(self) -> None:
        """Reload configuration"""
        pass
    
    @abstractmethod
    def validate(self) -> List[str]:
        """Validate configuration and return errors"""
        pass 
