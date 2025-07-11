"""
Service routing configuration for the TracSeq API Gateway.

This module defines service endpoints and provides routing logic
for mapping incoming requests to appropriate microservices.
"""

from dataclasses import dataclass
from typing import List, Optional
from api_gateway.core.config import get_config


@dataclass
class ServiceEndpoint:
    """Represents a service endpoint configuration."""
    name: str
    base_url: str
    path_prefix: str
    health_url: str
    timeout: int = 30


def get_service_endpoints() -> List[ServiceEndpoint]:
    """Get all configured service endpoints."""
    config = get_config()
    
    return [
        ServiceEndpoint(
            name="auth",
            base_url=config.services.auth_service_url,
            path_prefix="/api/auth",
            health_url=f"{config.services.auth_service_url}/health",
            timeout=config.services.service_timeout
        ),
        ServiceEndpoint(
            name="samples",
            base_url=config.services.sample_service_url,
            path_prefix="/api/samples",
            health_url=f"{config.services.sample_service_url}/health",
            timeout=config.services.service_timeout
        ),
        ServiceEndpoint(
            name="storage",
            base_url=config.services.storage_service_url,
            path_prefix="/api/storage",
            health_url=f"{config.services.storage_service_url}/health",
            timeout=config.services.service_timeout
        ),
        ServiceEndpoint(
            name="templates",
            base_url=config.services.template_service_url,
            path_prefix="/api/templates",
            health_url=f"{config.services.template_service_url}/health",
            timeout=config.services.service_timeout
        ),
        ServiceEndpoint(
            name="sequencing",
            base_url=config.services.sequencing_service_url,
            path_prefix="/api/sequencing",
            health_url=f"{config.services.sequencing_service_url}/health",
            timeout=config.services.service_timeout
        ),
        ServiceEndpoint(
            name="rag",
            base_url=config.services.rag_service_url,
            path_prefix="/api/rag",
            health_url=f"{config.services.rag_service_url}/health",
            timeout=config.services.service_timeout
        ),
        ServiceEndpoint(
            name="notifications",
            base_url=config.services.notification_service_url,
            path_prefix="/api/notifications",
            health_url=f"{config.services.notification_service_url}/health",
            timeout=config.services.service_timeout
        ),
    ]


def get_service_by_path(path: str) -> Optional[ServiceEndpoint]:
    """
    Find the appropriate service endpoint for a given path.
    
    Args:
        path: The request path to match
        
    Returns:
        ServiceEndpoint if found, None otherwise
    """
    endpoints = get_service_endpoints()
    
    # Sort by path prefix length (longest first) to match most specific routes first
    sorted_endpoints = sorted(endpoints, key=lambda e: len(e.path_prefix), reverse=True)
    
    for endpoint in sorted_endpoints:
        if path.startswith(endpoint.path_prefix):
            return endpoint
    
    return None


def get_service_by_name(name: str) -> Optional[ServiceEndpoint]:
    """
    Find a service endpoint by name.
    
    Args:
        name: The service name to find
        
    Returns:
        ServiceEndpoint if found, None otherwise
    """
    endpoints = get_service_endpoints()
    
    for endpoint in endpoints:
        if endpoint.name == name:
            return endpoint
    
    return None 