#!/usr/bin/env python3
"""
CORS Middleware for TracSeq 2.0 API Gateway
Centralized CORS configuration and handling
"""

from typing import List, Union
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from ..core.config import get_config
from ..core.logging import main_logger


def setup_cors(app: FastAPI) -> None:
    """Setup CORS middleware for the FastAPI application"""
    
    config = get_config()
    
    # Get CORS configuration
    cors_config = {
        "allow_origins": config.gateway.cors_origins,
        "allow_credentials": config.gateway.cors_credentials,
        "allow_methods": config.gateway.cors_methods,
        "allow_headers": config.gateway.cors_headers,
    }
    
    # Add CORS middleware
    app.add_middleware(
        CORSMiddleware,
        **cors_config
    )
    
    main_logger.info(
        "CORS middleware configured",
        extra={
            "cors_config": cors_config,
            "environment": "development" if config.is_development else "production"
        }
    )


def get_cors_config():
    """Get current CORS configuration"""
    config = get_config()
    
    return {
        "allow_origins": config.gateway.cors_origins,
        "allow_credentials": config.gateway.cors_credentials,
        "allow_methods": config.gateway.cors_methods,
        "allow_headers": config.gateway.cors_headers,
    }