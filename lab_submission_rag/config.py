"""
Configuration settings for the RAG system
"""

import os
from pathlib import Path
from typing import Optional
from pydantic_settings import BaseSettings
from pydantic import Field, ConfigDict

class Settings(BaseSettings):
    """Application settings"""
    
    # API Keys (optional when using Ollama)
    openai_api_key: Optional[str] = Field(None, description="OpenAI API key")
    anthropic_api_key: Optional[str] = Field(None, description="Anthropic API key")
    
    # LLM Settings
    llm_provider: str = Field(
        default="openai",
        description="LLM provider to use (openai, anthropic, ollama)"
    )
    model_name: str = Field(
        default="gpt-3.5-turbo",
        description="Name of the LLM model to use"
    )
    
    # Ollama Settings
    use_ollama: bool = Field(
        default=False,
        description="Whether to use Ollama for local LLM inference"
    )
    ollama_model: str = Field(
        default="llama2:7b",
        description="Ollama model to use"
    )
    ollama_base_url: str = Field(
        default="http://localhost:11434",
        description="Base URL for Ollama API"
    )
    
    # Embedding Settings
    embedding_model: str = Field(
        default="sentence-transformers/all-MiniLM-L6-v2",
        description="Model to use for embeddings"
    )
    
    # Document Processing
    chunk_size: int = Field(
        default=1000,
        description="Size of document chunks in characters"
    )
    chunk_overlap: int = Field(
        default=200,
        description="Overlap between chunks in characters"
    )
    
    # Vector Store
    vector_store_path: Path = Field(
        default=Path("data/vector_store"),
        description="Path to vector store files"
    )
    
    # File Storage
    upload_dir: Path = Field(
        default=Path("uploads"),
        description="Directory for uploaded files"
    )
    export_dir: Path = Field(
        default=Path("exports"),
        description="Directory for exported files"
    )
    
    # Logging
    log_level: str = Field(
        default="INFO",
        description="Logging level"
    )
    log_dir: Path = Field(
        default=Path("logs"),
        description="Directory for log files"
    )
    
    # Enhanced RAG Settings
    max_search_results: int = Field(
        default=5,
        description="Maximum number of search results to retrieve"
    )
    llm_temperature: float = Field(
        default=0.3,
        description="Temperature for LLM responses (0.0-1.0)"
    )
    max_tokens: int = Field(
        default=2048,
        description="Maximum tokens for LLM responses"
    )
    similarity_threshold: float = Field(
        default=0.5,
        description="Minimum similarity score for chunk inclusion"
    )
    batch_size: int = Field(
        default=5,
        description="Batch size for processing multiple documents"
    )
    export_directory: str = Field(
        default="exports",
        description="Directory for exporting processed data"
    )
    
    # Database Settings
    database_url: str = Field(
        default="postgresql+asyncpg://user:password@host.docker.internal:5432/lab_manager",
        description="PostgreSQL database URL"
    )
    database_host: str = Field(
        default="host.docker.internal",
        description="Database host"
    )
    database_port: int = Field(
        default=5432,
        description="Database port"
    )
    database_name: str = Field(
        default="lab_manager",
        description="Database name"
    )
    database_user: str = Field(
        default="user",
        description="Database user"
    )
    database_password: str = Field(
        default="password",
        description="Database password"
    )
    database_pool_size: int = Field(
        default=10,
        description="Database connection pool size"
    )
    database_max_overflow: int = Field(
        default=20,
        description="Database connection pool max overflow"
    )
    
    # RAG-specific table prefix to avoid conflicts with lab_manager tables
    table_prefix: str = Field(
        default="rag_",
        description="Prefix for RAG system tables to avoid conflicts"
    )
    
    # Schema name for RAG tables (optional)
    database_schema: Optional[str] = Field(
        default=None,
        description="Database schema name for RAG tables (optional)"
    )
    
    # Memory Optimization Settings
    MEMORY_OPTIMIZED: bool = True
    EMBEDDING_MODEL_CACHE: bool = False  # Don't cache models in memory
    VECTOR_STORE_MEMORY_LIMIT: int = 100  # MB limit for vector storage
    CHROMADB_PERSIST_DIRECTORY: str = "/app/data/chromadb"  # Persist to disk
    EMBEDDING_BATCH_SIZE: int = 8  # Smaller batches to reduce memory spikes
    
    def validate_api_keys(self) -> None:
        """Validate that required API keys are present based on provider"""
        if self.llm_provider == "openai" and not self.openai_api_key:
            raise ValueError("OpenAI API key is required when using OpenAI provider")
        if self.llm_provider == "anthropic" and not self.anthropic_api_key:
            raise ValueError("Anthropic API key is required when using Anthropic provider")
        if self.llm_provider == "ollama" and not self.use_ollama:
            raise ValueError("use_ollama must be True when using Ollama provider")
    
    model_config = ConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        case_sensitive=False,  # Allow case-insensitive environment variables
        extra="ignore"  # Ignore extra environment variables
    )

# Create settings instance
settings = Settings()

# Validate API keys
try:
    settings.validate_api_keys()
except ValueError as e:
    # Log warning but don't fail - allow service to start with default config
    print(f"Warning: {e}")

# Create necessary directories
for directory in [settings.upload_dir, settings.export_dir, settings.log_dir, settings.vector_store_path]:
    directory.mkdir(parents=True, exist_ok=True) 
