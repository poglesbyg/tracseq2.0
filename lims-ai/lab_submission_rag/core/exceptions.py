"""
Custom exception hierarchy for the Laboratory Submission RAG System

This module defines a comprehensive exception hierarchy that provides better error
handling, debugging capabilities, and allows for specific error recovery strategies.
"""

from datetime import datetime
from typing import Any


class LabSubmissionException(Exception):
    """Base exception for all laboratory submission processing errors"""

    def __init__(
        self,
        message: str,
        error_code: str | None = None,
        context: dict[str, Any] | None = None,
        cause: Exception | None = None,
    ) -> None:
        super().__init__(message)
        self.message = message
        self.error_code = error_code
        self.context = context or {}
        self.cause = cause
        self.timestamp = datetime.utcnow()

    def to_dict(self) -> dict[str, Any]:
        """Convert exception to dictionary for structured logging"""
        return {
            "exception_type": self.__class__.__name__,
            "message": self.message,
            "error_code": self.error_code,
            "context": self.context,
            "timestamp": self.timestamp.isoformat(),
            "cause": str(self.cause) if self.cause else None,
        }


class DocumentProcessingException(LabSubmissionException):
    """Exceptions related to document processing"""

    def __init__(
        self,
        message: str,
        file_path: str | None = None,
        file_type: str | None = None,
        **kwargs,
    ) -> None:
        context = kwargs.get("context", {})
        context.update({"file_path": file_path, "file_type": file_type})
        kwargs["context"] = context
        super().__init__(message, **kwargs)


class UnsupportedFileTypeException(DocumentProcessingException):
    """Exception raised when trying to process unsupported file types"""

    pass


class FileNotFoundError(DocumentProcessingException):
    """Exception raised when document file is not found"""

    pass


class DocumentCorruptedException(DocumentProcessingException):
    """Exception raised when document is corrupted or unreadable"""

    pass


class ExtractionException(LabSubmissionException):
    """Exceptions related to information extraction"""

    def __init__(
        self,
        message: str,
        extraction_stage: str | None = None,
        confidence_score: float | None = None,
        **kwargs,
    ) -> None:
        context = kwargs.get("context", {})
        context.update({"extraction_stage": extraction_stage, "confidence_score": confidence_score})
        kwargs["context"] = context
        super().__init__(message, **kwargs)


class LLMProviderException(ExtractionException):
    """Exception raised when LLM provider fails"""

    def __init__(
        self, message: str, provider: str | None = None, model: str | None = None, **kwargs
    ) -> None:
        context = kwargs.get("context", {})
        context.update({"provider": provider, "model": model})
        kwargs["context"] = context
        super().__init__(message, **kwargs)


class RateLimitException(LLMProviderException):
    """Exception raised when hitting rate limits"""

    def __init__(self, message: str, retry_after: int | None = None, **kwargs) -> None:
        context = kwargs.get("context", {})
        context.update({"retry_after": retry_after})
        kwargs["context"] = context
        super().__init__(message, **kwargs)


class ValidationException(ExtractionException):
    """Exception raised when extracted data fails validation"""

    def __init__(self, message: str, validation_errors: list | None = None, **kwargs) -> None:
        context = kwargs.get("context", {})
        context.update({"validation_errors": validation_errors or []})
        kwargs["context"] = context
        super().__init__(message, **kwargs)


class VectorStoreException(LabSubmissionException):
    """Exceptions related to vector store operations"""

    def __init__(
        self,
        message: str,
        operation: str | None = None,
        vector_store_type: str | None = None,
        **kwargs,
    ) -> None:
        context = kwargs.get("context", {})
        context.update({"operation": operation, "vector_store_type": vector_store_type})
        kwargs["context"] = context
        super().__init__(message, **kwargs)


class EmbeddingException(VectorStoreException):
    """Exception raised during text embedding generation"""

    pass


class SearchException(VectorStoreException):
    """Exception raised during vector similarity search"""

    pass


class DatabaseException(LabSubmissionException):
    """Exceptions related to database operations"""

    def __init__(
        self, message: str, operation: str | None = None, table: str | None = None, **kwargs
    ) -> None:
        context = kwargs.get("context", {})
        context.update({"operation": operation, "table": table})
        kwargs["context"] = context
        super().__init__(message, **kwargs)


class ConnectionException(DatabaseException):
    """Exception raised when database connection fails"""

    pass


class TransactionException(DatabaseException):
    """Exception raised during database transactions"""

    pass


class ConfigurationException(LabSubmissionException):
    """Exception raised for configuration-related errors"""

    def __init__(self, message: str, config_key: str | None = None, **kwargs) -> None:
        context = kwargs.get("context", {})
        context.update({"config_key": config_key})
        kwargs["context"] = context
        super().__init__(message, **kwargs)


class ServiceException(LabSubmissionException):
    """Exception raised by service layer operations"""

    def __init__(
        self,
        message: str,
        service_name: str | None = None,
        operation: str | None = None,
        **kwargs,
    ) -> None:
        context = kwargs.get("context", {})
        context.update({"service_name": service_name, "operation": operation})
        kwargs["context"] = context
        super().__init__(message, **kwargs)


class CircuitBreakerException(ServiceException):
    """Exception raised when circuit breaker is open"""

    def __init__(self, message: str, failure_count: int | None = None, **kwargs) -> None:
        context = kwargs.get("context", {})
        context.update({"failure_count": failure_count})
        kwargs["context"] = context
        super().__init__(message, **kwargs)
