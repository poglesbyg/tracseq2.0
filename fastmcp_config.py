"""
Configuration module for TracSeq 2.0 FastMCP services.
Handles environment variables and service configuration.
"""

import os
from typing import Optional
from pydantic import Field
from pydantic_settings import BaseSettings


class FastMCPConfig(BaseSettings):
    """Configuration for FastMCP services with environment variable support."""
    
    # AI Model Configuration
    anthropic_api_key: Optional[str] = Field(default=None, env="ANTHROPIC_API_KEY")
    openai_api_key: Optional[str] = Field(default=None, env="OPENAI_API_KEY")
    fastmcp_default_model: str = Field(default="claude-3-sonnet-20240229", env="FASTMCP_DEFAULT_MODEL")
    
    # Service Ports
    lab_service_port: int = Field(default=8000, env="LAB_SERVICE_PORT")
    rag_service_port: int = Field(default=8001, env="RAG_SERVICE_PORT")
    agent_service_port: int = Field(default=8003, env="AGENT_SERVICE_PORT")
    gateway_service_port: int = Field(default=8005, env="GATEWAY_SERVICE_PORT")
    
    # Laboratory Configuration
    lab_confidence_threshold: float = Field(default=0.85, env="LAB_CONFIDENCE_THRESHOLD")
    lab_processing_timeout: int = Field(default=300, env="LAB_PROCESSING_TIMEOUT")
    lab_enable_notifications: bool = Field(default=True, env="LAB_ENABLE_NOTIFICATIONS")
    
    # Database Configuration
    database_url: Optional[str] = Field(default=None, env="DATABASE_URL")
    redis_url: Optional[str] = Field(default=None, env="REDIS_URL")
    
    # Performance Settings
    max_concurrent_processing: int = Field(default=5, env="MAX_CONCURRENT_PROCESSING")
    batch_size_limit: int = Field(default=10, env="BATCH_SIZE_LIMIT")
    
    # Logging
    log_level: str = Field(default="INFO", env="FASTMCP_LOG_LEVEL")
    
    class Config:
        env_file = ".env"
        env_file_encoding = "utf-8"
        extra = "ignore"  # Ignore extra environment variables


# Global configuration instance
config = FastMCPConfig()


def get_model_preferences() -> list[str]:
    """Get model preferences based on available API keys."""
    preferences = []
    
    if config.anthropic_api_key:
        preferences.extend([
            "claude-3-sonnet-20240229",
            "claude-3-haiku-20240307"
        ])
    
    if config.openai_api_key:
        preferences.extend([
            "gpt-4",
            "gpt-3.5-turbo"
        ])
    
    # Fallback if no API keys configured
    if not preferences:
        preferences = ["claude-3-sonnet-20240229", "gpt-4", "gpt-3.5-turbo"]
    
    return preferences


def validate_api_keys() -> dict[str, bool]:
    """Validate which API keys are configured."""
    return {
        "anthropic": bool(config.anthropic_api_key and config.anthropic_api_key != "your_anthropic_api_key_here"),
        "openai": bool(config.openai_api_key and config.openai_api_key != "your_openai_api_key_here"),
    }


def get_service_info() -> dict:
    """Get service configuration information."""
    api_status = validate_api_keys()
    
    return {
        "configuration": {
            "anthropic_configured": api_status["anthropic"],
            "openai_configured": api_status["openai"],
            "default_model": config.fastmcp_default_model,
            "confidence_threshold": config.lab_confidence_threshold,
            "processing_timeout": config.lab_processing_timeout,
        },
        "ports": {
            "laboratory_server": config.lab_service_port,
            "rag_service": config.rag_service_port,
            "agent_service": config.agent_service_port,
            "gateway_service": config.gateway_service_port,
        },
        "performance": {
            "max_concurrent": config.max_concurrent_processing,
            "batch_size_limit": config.batch_size_limit,
        }
    } 