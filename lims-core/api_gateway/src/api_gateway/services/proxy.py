"""
Proxy Service for TracSeq API Gateway

Handles request forwarding, response transformation, and error handling.
"""

import json
import time
from typing import Any, Dict, Optional

import httpx
import structlog
from fastapi import HTTPException, Request, Response

from api_gateway.core.config import ServiceEndpoint, TracSeqAPIGatewayConfig

logger = structlog.get_logger(__name__)


class ProxyService:
    """Service for proxying requests to microservices."""

    def __init__(self, http_client: httpx.AsyncClient, config: TracSeqAPIGatewayConfig):
        self.http_client = http_client
        self.config = config
        self.request_count = 0
        self.error_count = 0

    async def proxy_request(
        self,
        request: Request,
        target_service: ServiceEndpoint,
        path: str,
        response: Response
    ) -> Response:
        """Proxy a request to the target service."""

        start_time = time.time()
        self.request_count += 1

        try:
            # Build upstream URL
            upstream_path = path[len(target_service.path_prefix):] if path.startswith(target_service.path_prefix) else path
            upstream_url = f"{target_service.base_url}/api/v1{upstream_path}"

            # Prepare headers (remove hop-by-hop headers)
            headers = self._prepare_headers(dict(request.headers))

            # Get request body
            body = await request.body()

            # Log request
            logger.info("Proxying request",
                       method=request.method,
                       path=path,
                       upstream_url=upstream_url,
                       service=target_service.name)

            # Make upstream request
            upstream_response = await self.http_client.request(
                method=request.method,
                url=upstream_url,
                headers=headers,
                content=body,
                params=dict(request.query_params),
                timeout=target_service.timeout
            )

            # Prepare response headers
            response_headers = self._prepare_response_headers(dict(upstream_response.headers))

            # Calculate request duration
            duration = time.time() - start_time

            # Log response
            logger.info("Request completed",
                       status_code=upstream_response.status_code,
                       duration=duration,
                       service=target_service.name)

            # Return response
            return Response(
                content=upstream_response.content,
                status_code=upstream_response.status_code,
                headers=response_headers
            )

        except httpx.TimeoutException:
            self.error_count += 1
            duration = time.time() - start_time

            logger.error("Request timeout",
                        service=target_service.name,
                        path=path,
                        timeout=target_service.timeout,
                        duration=duration)

            raise HTTPException(
                status_code=504,
                detail=f"Gateway timeout - {target_service.name} did not respond within {target_service.timeout}s"
            )

        except httpx.ConnectError as e:
            self.error_count += 1
            duration = time.time() - start_time

            logger.error("Service connection error",
                        service=target_service.name,
                        path=path,
                        error=str(e),
                        duration=duration)

            raise HTTPException(
                status_code=503,
                detail=f"Service {target_service.name} is temporarily unavailable"
            )

        except httpx.HTTPStatusError as e:
            duration = time.time() - start_time

            logger.warning("HTTP error from upstream",
                          service=target_service.name,
                          status_code=e.response.status_code,
                          path=path,
                          duration=duration)

            # Forward the error response
            return Response(
                content=e.response.content,
                status_code=e.response.status_code,
                headers=self._prepare_response_headers(dict(e.response.headers))
            )

        except Exception as e:
            self.error_count += 1
            duration = time.time() - start_time

            logger.error("Proxy error",
                        service=target_service.name,
                        path=path,
                        error=str(e),
                        error_type=type(e).__name__,
                        duration=duration)

            raise HTTPException(
                status_code=502,
                detail=f"Bad gateway - error communicating with {target_service.name}"
            )

    def _prepare_headers(self, headers: Dict[str, str]) -> Dict[str, str]:
        """Prepare request headers for upstream service."""

        # Remove hop-by-hop headers
        hop_by_hop_headers = {
            "connection", "keep-alive", "proxy-authenticate",
            "proxy-authorization", "te", "trailers", "transfer-encoding",
            "upgrade", "proxy-connection"
        }

        cleaned_headers = {
            k: v for k, v in headers.items()
            if k.lower() not in hop_by_hop_headers
        }

        # Add gateway headers
        cleaned_headers["X-Gateway"] = "TracSeq-API-Gateway"
        cleaned_headers["X-Gateway-Version"] = self.config.version
        cleaned_headers["X-Forwarded-By"] = "TracSeq-Gateway"

        return cleaned_headers

    def _prepare_response_headers(self, headers: Dict[str, str]) -> Dict[str, str]:
        """Prepare response headers from upstream service."""

        # Remove hop-by-hop headers
        hop_by_hop_headers = {
            "connection", "keep-alive", "proxy-authenticate",
            "proxy-authorization", "te", "trailers", "transfer-encoding",
            "upgrade", "proxy-connection"
        }

        cleaned_headers = {
            k: v for k, v in headers.items()
            if k.lower() not in hop_by_hop_headers
        }

        # Add gateway response headers
        cleaned_headers["X-Gateway"] = "TracSeq-API-Gateway"
        cleaned_headers["X-Gateway-Request-ID"] = str(self.request_count)

        return cleaned_headers

    async def health_check_service(self, service: ServiceEndpoint) -> Dict[str, Any]:
        """Check health of a specific service."""

        try:
            start_time = time.time()

            response = await self.http_client.get(
                service.health_url,
                timeout=service.timeout
            )

            duration = time.time() - start_time

            if response.status_code == 200:
                return {
                    "name": service.name,
                    "status": "healthy",
                    "response_time_ms": round(duration * 1000, 2),
                    "url": service.health_url
                }
            else:
                return {
                    "name": service.name,
                    "status": "unhealthy",
                    "response_time_ms": round(duration * 1000, 2),
                    "status_code": response.status_code,
                    "url": service.health_url
                }

        except httpx.TimeoutException:
            return {
                "name": service.name,
                "status": "timeout",
                "error": f"Health check timed out after {service.timeout}s",
                "url": service.health_url
            }

        except httpx.ConnectError:
            return {
                "name": service.name,
                "status": "unreachable",
                "error": "Cannot connect to service",
                "url": service.health_url
            }

        except Exception as e:
            return {
                "name": service.name,
                "status": "error",
                "error": str(e),
                "url": service.health_url
            }

    def get_stats(self) -> Dict[str, Any]:
        """Get proxy service statistics."""

        error_rate = (self.error_count / self.request_count * 100) if self.request_count > 0 else 0

        return {
            "total_requests": self.request_count,
            "total_errors": self.error_count,
            "error_rate_percent": round(error_rate, 2),
            "success_rate_percent": round(100 - error_rate, 2)
        }
