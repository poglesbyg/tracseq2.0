#!/usr/bin/env python3
"""
Modular Main Entry Point for TracSeq 2.0 API Gateway
Demonstrates the refactored, modular architecture
"""

import os
import sys
import uvicorn
from pathlib import Path

# Add the src directory to the path
src_path = Path(__file__).parent.parent
sys.path.insert(0, str(src_path))

from api_gateway.app import get_app
from api_gateway.core.config import get_config
from api_gateway.core.logging import main_logger


def main():
    """Main entry point for the modular API Gateway"""
    
    # Get configuration
    config = get_config()
    
    # Log startup information
    main_logger.info(
        f"🚀 Starting TracSeq 2.0 API Gateway (Modular)",
        extra={
            "version": config.gateway.version,
            "host": config.gateway.host,
            "port": config.gateway.port,
            "debug": config.gateway.debug,
            "environment": "development" if config.is_development else "production"
        }
    )
    
    # Create the application
    app = get_app()
    
    # Run the application
    uvicorn.run(
        app,
        host=config.gateway.host,
        port=config.gateway.port,
        log_level=config.logging.log_level.lower(),
        access_log=config.logging.enable_access_log,
        reload=config.is_development,
        workers=1 if config.is_development else 4
    )


if __name__ == "__main__":
    main()