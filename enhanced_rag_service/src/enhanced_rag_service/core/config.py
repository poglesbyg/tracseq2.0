"""Configuration management for Enhanced RAG Service."""

import os
from functools import lru_cache
from typing import Any, Dict, List, Optional

from pydantic import BaseModel, Field
from pydantic_settings import BaseSettings


class DatabaseSettings(BaseModel):
    """Database configuration."""
    url: str = Field(
        default="postgresql://rag_user:password@localhost:5432/enhanced_rag_db",
        description="PostgreSQL database URL"
    )
    pool_size: int = Field(default=10, description="Connection pool size")
    echo: bool = Field(default=False, description="Enable SQL logging")


class VectorStoreSettings(BaseModel):
    """Vector store configuration."""
    provider: str = Field(default="chromadb", description="Vector store provider")
    collection_name: str = Field(default="lab_documents", description="Collection name")
    embedding_model: str = Field(default="all-MiniLM-L6-v2", description="Embedding model")
    chunk_size: int = Field(default=512, description="Text chunk size")
    similarity_threshold: float = Field(default=0.7, description="Similarity threshold")
    chroma_persist_directory: str = Field(default="./chroma_db", description="ChromaDB directory")


class LLMSettings(BaseModel):
    """Large Language Model configuration."""
    provider: str = Field(default="openai", description="LLM provider")
    model_name: str = Field(default="gpt-3.5-turbo", description="Model name")
    temperature: float = Field(default=0.1, description="Model temperature")
    max_tokens: int = Field(default=2048, description="Maximum tokens")
    openai_api_key: Optional[str] = Field(default=None, description="OpenAI API key")


class DocumentProcessingSettings(BaseModel):
    """Document processing configuration."""
    upload_dir: str = Field(default="./uploads", description="Upload directory")
    max_file_size: int = Field(default=50 * 1024 * 1024, description="Max file size (50MB)")
    allowed_extensions: List[str] = Field(
        default=["pdf", "docx", "txt", "md", "csv", "xlsx", "png", "jpg"],
        description="Allowed file extensions"
    )
    ocr_enabled: bool = Field(default=True, description="Enable OCR")
    ocr_language: str = Field(default="eng", description="OCR language")


class EnhancedRAGSettings(BaseSettings):
    """Main configuration class for Enhanced RAG Service."""
    
    # Service metadata
    service_name: str = Field(default="Enhanced RAG Service", description="Service name")
    version: str = Field(default="0.1.0", description="Service version")
    environment: str = Field(default="development", description="Environment")
    host: str = Field(default="0.0.0.0", description="Server host")
    port: int = Field(default=8086, description="Server port")
    
    # Component configurations
    database: DatabaseSettings = Field(default_factory=DatabaseSettings)
    vector_store: VectorStoreSettings = Field(default_factory=VectorStoreSettings)
    llm: LLMSettings = Field(default_factory=LLMSettings)
    document_processing: DocumentProcessingSettings = Field(
        default_factory=DocumentProcessingSettings
    )
    
    # Integration settings
    auth_service_url: str = Field(
        default="http://auth-service:8080",
        description="Authentication service URL"
    )
    storage_service_url: str = Field(
        default="http://enhanced-storage-service:8082",
        description="Storage service URL"
    )
    
    # Security
    secret_key: str = Field(
        default="your-secret-key-change-in-production",
        description="Secret key for JWT tokens"
    )
    cors_origins: List[str] = Field(
        default=["http://localhost:3000", "http://localhost:8080"],
        description="CORS allowed origins"
    )
    
    # Feature flags
    enable_template_matching: bool = Field(default=True, description="Enable template matching")
    enable_form_validation: bool = Field(default=True, description="Enable form validation")
    enable_intelligent_extraction: bool = Field(default=True, description="Enable intelligent extraction")
    enable_multi_modal: bool = Field(default=True, description="Enable multi-modal processing")
    enable_real_time_processing: bool = Field(default=True, description="Enable real-time processing")
    
    class Config:
        """Pydantic configuration."""
        env_file = ".env"
        env_file_encoding = "utf-8"
        env_nested_delimiter = "__"
        case_sensitive = False


@lru_cache()
def get_settings() -> EnhancedRAGSettings:
    """Get cached application settings."""
    return EnhancedRAGSettings()


def is_production() -> bool:
    """Check if running in production environment."""
    return get_settings().environment == "production"
