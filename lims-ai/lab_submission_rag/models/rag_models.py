"""
Data models for RAG system components
"""

from datetime import datetime
from typing import Any

from pydantic import BaseModel, Field


class DocumentChunk(BaseModel):
    """Represents a chunk of text from a document"""

    content: str = Field(..., description="The text content of the chunk")
    metadata: dict[str, Any] = Field(
        default_factory=dict, description="Metadata associated with the chunk"
    )
    chunk_id: str = Field(..., description="Unique identifier for the chunk")
    source_document: str = Field(..., description="Source document identifier")
    chunk_index: int | None = Field(None, description="Index of the chunk in the document")
    embedding: list[float] | None = Field(None, description="Embedding vector for the chunk")
    created_at: datetime = Field(
        default_factory=datetime.utcnow, description="Timestamp when the chunk was created"
    )


class ExtractionResult(BaseModel):
    """Result of extracting information from a document"""

    success: bool = Field(..., description="Whether extraction was successful")
    confidence_score: float = Field(
        ..., description="Confidence score of the extraction (0-1)", ge=0.0, le=1.0
    )
    missing_fields: list[str] = Field(
        default_factory=list, description="List of required fields that were not found"
    )
    warnings: list[str] = Field(
        default_factory=list, description="List of warnings generated during extraction"
    )
    processing_time: float = Field(
        ..., description="Time taken to process the document in seconds", ge=0.0
    )
    source_document: str = Field(..., description="Source document identifier")
    submission_id: str | None = Field(None, description="Unique identifier for the submission")
    extracted_data: dict[str, Any] = Field(
        default_factory=dict, description="Extracted data from the document"
    )


class VectorStoreInfo(BaseModel):
    """Information about the vector store state"""

    total_chunks: int = Field(..., description="Total number of chunks in store")
    total_documents: int = Field(..., description="Total number of source documents")
    embedding_model: str = Field(..., description="Name of the embedding model used")
    last_updated: datetime = Field(..., description="Last update timestamp")
    storage_size: int = Field(..., description="Storage size in bytes")


class QueryResult(BaseModel):
    """Result of a RAG query"""

    query: str
    relevant_chunks: list[DocumentChunk]
    confidence_scores: list[float]
    generated_response: str
    processing_time: float


class DocumentMetadata(BaseModel):
    """Metadata for processed documents"""

    document_id: str
    filename: str
    file_type: str
    file_size: int
    upload_time: datetime
    processing_time: float
    total_chunks: int
    extraction_status: str
