"""
Tests for the enhanced proxy service.

This module tests the circuit breaker patterns, service health monitoring,
and proxy request handling functionality.
"""

import asyncio
import pytest
from unittest.mock import Mock, AsyncMock, patch
from datetime import datetime, timedelta
import httpx

from api_gateway.services.proxy import (
    CircuitBreaker,
    CircuitState,
    EnhancedProxyService,
    ServiceHealth
)
from api_gateway.core.exceptions import (
    CircuitBreakerException,
    TimeoutException,
    ServiceUnavailableException,
    ExternalServiceException
)


class TestCircuitBreaker:
    """Test circuit breaker functionality."""
    
    def test_circuit_breaker_initialization(self):
        """Test circuit breaker initialization."""
        cb = CircuitBreaker(
            service_name="test-service",
            failure_threshold=3,
            recovery_timeout=30,
            half_open_max_calls=2
        )
        
        assert cb.service_name == "test-service"
        assert cb.failure_threshold == 3
        assert cb.recovery_timeout == 30
        assert cb.half_open_max_calls == 2
        assert cb.state == CircuitState.CLOSED
        assert cb.failure_count == 0
        assert cb.last_failure_time is None
        assert cb.half_open_calls == 0
    
    @pytest.mark.asyncio
    async def test_circuit_breaker_closed_state(self):
        """Test circuit breaker in closed state."""
        cb = CircuitBreaker("test-service", failure_threshold=3)
        
        # Mock function that succeeds
        async def success_func():
            return "success"
        
        # Should allow calls in closed state
        result = await cb.call(success_func)
        assert result == "success"
        assert cb.state == CircuitState.CLOSED
        assert cb.failure_count == 0
    
    @pytest.mark.asyncio
    async def test_circuit_breaker_failure_tracking(self):
        """Test circuit breaker failure tracking."""
        cb = CircuitBreaker("test-service", failure_threshold=3)
        
        # Mock function that fails
        async def fail_func():
            raise Exception("Service failure")
        
        # Test failures up to threshold
        for i in range(2):
            with pytest.raises(Exception):
                await cb.call(fail_func)
            assert cb.state == CircuitState.CLOSED
            assert cb.failure_count == i + 1
        
        # Third failure should open circuit
        with pytest.raises(Exception):
            await cb.call(fail_func)
        assert cb.state == CircuitState.OPEN
        assert cb.failure_count == 3
    
    @pytest.mark.asyncio
    async def test_circuit_breaker_open_state(self):
        """Test circuit breaker in open state."""
        cb = CircuitBreaker("test-service", failure_threshold=1)
        
        # Mock function that fails
        async def fail_func():
            raise Exception("Service failure")
        
        # Cause circuit to open
        with pytest.raises(Exception):
            await cb.call(fail_func)
        assert cb.state == CircuitState.OPEN
        
        # Should reject calls in open state
        async def success_func():
            return "success"
        
        with pytest.raises(CircuitBreakerException):
            await cb.call(success_func)
    
    @pytest.mark.asyncio
    async def test_circuit_breaker_half_open_state(self):
        """Test circuit breaker in half-open state."""
        cb = CircuitBreaker("test-service", failure_threshold=1, recovery_timeout=0.1)
        
        # Mock function that fails
        async def fail_func():
            raise Exception("Service failure")
        
        # Cause circuit to open
        with pytest.raises(Exception):
            await cb.call(fail_func)
        assert cb.state == CircuitState.OPEN
        
        # Wait for recovery timeout
        await asyncio.sleep(0.2)
        
        # Mock function that succeeds
        async def success_func():
            return "success"
        
        # Should transition to half-open and allow call
        result = await cb.call(success_func)
        assert result == "success"
        assert cb.state == CircuitState.CLOSED
        assert cb.failure_count == 0
    
    @pytest.mark.asyncio
    async def test_circuit_breaker_half_open_failure(self):
        """Test circuit breaker failure in half-open state."""
        cb = CircuitBreaker("test-service", failure_threshold=1, recovery_timeout=0.1)
        
        # Mock function that fails
        async def fail_func():
            raise Exception("Service failure")
        
        # Cause circuit to open
        with pytest.raises(Exception):
            await cb.call(fail_func)
        assert cb.state == CircuitState.OPEN
        
        # Wait for recovery timeout
        await asyncio.sleep(0.2)
        
        # Failure in half-open should return to open
        with pytest.raises(Exception):
            await cb.call(fail_func)
        assert cb.state == CircuitState.OPEN
    
    @pytest.mark.asyncio
    async def test_circuit_breaker_half_open_max_calls(self):
        """Test circuit breaker half-open max calls limit."""
        cb = CircuitBreaker("test-service", failure_threshold=1, recovery_timeout=0.1, half_open_max_calls=2)
        
        # Mock function that fails
        async def fail_func():
            raise Exception("Service failure")
        
        # Cause circuit to open
        with pytest.raises(Exception):
            await cb.call(fail_func)
        assert cb.state == CircuitState.OPEN
        
        # Wait for recovery timeout
        await asyncio.sleep(0.2)
        
        # Mock function that succeeds
        async def success_func():
            return "success"
        
        # First call should work
        result = await cb.call(success_func)
        assert result == "success"
        assert cb.state == CircuitState.CLOSED


class TestServiceHealth:
    """Test service health data structure."""
    
    def test_service_health_creation(self):
        """Test service health creation."""
        health = ServiceHealth(
            name="test-service",
            status="healthy",
            response_time_ms=150.5,
            last_check=datetime.now(),
            error_count=0,
            success_count=10,
            url="http://test-service:8000/health"
        )
        
        assert health.name == "test-service"
        assert health.status == "healthy"
        assert health.response_time_ms == 150.5
        assert health.error_count == 0
        assert health.success_count == 10
        assert health.url == "http://test-service:8000/health"


class TestEnhancedProxyService:
    """Test enhanced proxy service."""
    
    def setup_method(self):
        """Setup test fixtures."""
        self.mock_http_client = Mock(spec=httpx.AsyncClient)
        self.proxy_service = EnhancedProxyService(self.mock_http_client)
    
    def test_proxy_service_initialization(self):
        """Test proxy service initialization."""
        assert self.proxy_service.http_client == self.mock_http_client
        assert self.proxy_service.request_count == 0
        assert self.proxy_service.error_count == 0
        assert isinstance(self.proxy_service.circuit_breakers, dict)
        assert isinstance(self.proxy_service.service_health, dict)
        assert isinstance(self.proxy_service.service_stats, dict)
    
    def test_circuit_breaker_initialization(self):
        """Test circuit breaker initialization for services."""
        # Check that circuit breakers are created for all services
        expected_services = [
            "auth", "sample", "storage", "template", 
            "sequencing", "rag", "notification"
        ]
        
        for service_name in expected_services:
            assert service_name in self.proxy_service.circuit_breakers
            cb = self.proxy_service.circuit_breakers[service_name]
            assert isinstance(cb, CircuitBreaker)
            assert cb.service_name == service_name
    
    @pytest.mark.asyncio
    async def test_health_check_service_success(self):
        """Test successful service health check."""
        # Mock successful HTTP response
        mock_response = Mock()
        mock_response.status_code = 200
        self.mock_http_client.get = AsyncMock(return_value=mock_response)
        
        # Perform health check
        health = await self.proxy_service.health_check_service("test-service", "http://test-service:8000")
        
        assert health.name == "test-service"
        assert health.status == "healthy"
        assert health.response_time_ms > 0
        assert health.url == "http://test-service:8000/health"
        
        # Check that HTTP client was called correctly
        self.mock_http_client.get.assert_called_once()
    
    @pytest.mark.asyncio
    async def test_health_check_service_unhealthy(self):
        """Test unhealthy service health check."""
        # Mock unhealthy HTTP response
        mock_response = Mock()
        mock_response.status_code = 503
        self.mock_http_client.get = AsyncMock(return_value=mock_response)
        
        # Perform health check
        health = await self.proxy_service.health_check_service("test-service", "http://test-service:8000")
        
        assert health.name == "test-service"
        assert health.status == "unhealthy"
        assert health.response_time_ms > 0
        assert health.url == "http://test-service:8000/health"
    
    @pytest.mark.asyncio
    async def test_health_check_service_timeout(self):
        """Test service health check timeout."""
        # Mock timeout exception
        self.mock_http_client.get = AsyncMock(side_effect=httpx.TimeoutException("Timeout"))
        
        # Perform health check
        health = await self.proxy_service.health_check_service("test-service", "http://test-service:8000")
        
        assert health.name == "test-service"
        assert health.status == "timeout"
        assert health.url == "http://test-service:8000/health"
    
    @pytest.mark.asyncio
    async def test_health_check_service_unreachable(self):
        """Test unreachable service health check."""
        # Mock connection error
        self.mock_http_client.get = AsyncMock(side_effect=httpx.ConnectError("Connection failed"))
        
        # Perform health check
        health = await self.proxy_service.health_check_service("test-service", "http://test-service:8000")
        
        assert health.name == "test-service"
        assert health.status == "unreachable"
        assert health.url == "http://test-service:8000/health"
    
    @pytest.mark.asyncio
    async def test_health_check_all_services(self):
        """Test health check for all services."""
        # Mock successful HTTP responses
        mock_response = Mock()
        mock_response.status_code = 200
        self.mock_http_client.get = AsyncMock(return_value=mock_response)
        
        # Perform health check for all services
        health_status = await self.proxy_service.health_check_all_services()
        
        # Check that all expected services are included
        expected_services = [
            "auth", "sample", "storage", "template", 
            "sequencing", "rag", "notification"
        ]
        
        for service_name in expected_services:
            assert service_name in health_status
            health = health_status[service_name]
            assert isinstance(health, ServiceHealth)
            assert health.name == service_name
            assert health.status == "healthy"
    
    @pytest.mark.asyncio
    async def test_make_http_request_success(self):
        """Test successful HTTP request with retry logic."""
        # Mock successful HTTP response
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.content = b"Success"
        self.mock_http_client.request = AsyncMock(return_value=mock_response)
        
        # Make HTTP request
        response = await self.proxy_service._make_http_request(
            method="GET",
            url="http://test-service:8000/api/test"
        )
        
        assert response == mock_response
        assert response.status_code == 200
        assert response.content == b"Success"
        
        # Should be called only once for successful request
        self.mock_http_client.request.assert_called_once()
    
    @pytest.mark.asyncio
    async def test_make_http_request_retry_on_failure(self):
        """Test HTTP request retry logic on failure."""
        # Mock connection error followed by success
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.content = b"Success"
        
        self.mock_http_client.request = AsyncMock(side_effect=[
            httpx.ConnectError("Connection failed"),
            httpx.ConnectError("Connection failed"),
            mock_response
        ])
        
        # Make HTTP request
        response = await self.proxy_service._make_http_request(
            method="GET",
            url="http://test-service:8000/api/test"
        )
        
        assert response == mock_response
        assert response.status_code == 200
        
        # Should be called 3 times (2 failures + 1 success)
        assert self.mock_http_client.request.call_count == 3
    
    @pytest.mark.asyncio
    async def test_make_http_request_all_retries_fail(self):
        """Test HTTP request when all retries fail."""
        # Mock all requests failing
        self.mock_http_client.request = AsyncMock(side_effect=httpx.ConnectError("Connection failed"))
        
        # Make HTTP request
        with pytest.raises(httpx.ConnectError):
            await self.proxy_service._make_http_request(
                method="GET",
                url="http://test-service:8000/api/test"
            )
        
        # Should be called 4 times (1 initial + 3 retries)
        assert self.mock_http_client.request.call_count == 4
    
    @pytest.mark.asyncio
    async def test_make_http_request_no_retry_on_http_error(self):
        """Test HTTP request doesn't retry on HTTP errors."""
        # Mock HTTP error
        http_error = httpx.HTTPStatusError("HTTP Error", request=Mock(), response=Mock())
        self.mock_http_client.request = AsyncMock(side_effect=http_error)
        
        # Make HTTP request
        with pytest.raises(httpx.HTTPStatusError):
            await self.proxy_service._make_http_request(
                method="GET",
                url="http://test-service:8000/api/test"
            )
        
        # Should be called only once (no retries for HTTP errors)
        self.mock_http_client.request.assert_called_once()
    
    def test_update_service_stats(self):
        """Test service statistics updating."""
        # Update stats for successful request
        self.proxy_service._update_service_stats("test-service", True, 0.5)
        
        stats = self.proxy_service.service_stats["test-service"]
        assert stats["total_requests"] == 1
        assert stats["successful_requests"] == 1
        assert stats["failed_requests"] == 0
        assert stats["total_duration"] == 0.5
        assert stats["avg_duration"] == 0.5
        
        # Update stats for failed request
        self.proxy_service._update_service_stats("test-service", False, 1.0)
        
        stats = self.proxy_service.service_stats["test-service"]
        assert stats["total_requests"] == 2
        assert stats["successful_requests"] == 1
        assert stats["failed_requests"] == 1
        assert stats["total_duration"] == 1.5
        assert stats["avg_duration"] == 0.75
    
    def test_get_stats(self):
        """Test getting comprehensive proxy service statistics."""
        # Add some test data
        self.proxy_service.request_count = 100
        self.proxy_service.error_count = 10
        self.proxy_service._update_service_stats("test-service", True, 0.5)
        self.proxy_service._update_service_stats("test-service", False, 1.0)
        
        # Add service health data
        health = ServiceHealth(
            name="test-service",
            status="healthy",
            response_time_ms=150.0,
            last_check=datetime.now(),
            error_count=1,
            success_count=1,
            url="http://test-service:8000/health"
        )
        self.proxy_service.service_health["test-service"] = health
        
        # Get stats
        stats = self.proxy_service.get_stats()
        
        assert stats["total_requests"] == 100
        assert stats["total_errors"] == 10
        assert stats["error_rate_percent"] == 10.0
        assert stats["success_rate_percent"] == 90.0
        assert "service_stats" in stats
        assert "circuit_breaker_states" in stats
        assert "service_health" in stats
        
        # Check service-specific stats
        assert "test-service" in stats["service_stats"]
        service_stats = stats["service_stats"]["test-service"]
        assert service_stats["total_requests"] == 2
        assert service_stats["successful_requests"] == 1
        assert service_stats["failed_requests"] == 1
        
        # Check circuit breaker states
        assert "auth" in stats["circuit_breaker_states"]
        assert stats["circuit_breaker_states"]["auth"] == "closed"
        
        # Check service health
        assert "test-service" in stats["service_health"]
        health_stats = stats["service_health"]["test-service"]
        assert health_stats["status"] == "healthy"
        assert health_stats["response_time_ms"] == 150.0
        assert health_stats["error_count"] == 1
        assert health_stats["success_count"] == 1
    
    def test_prepare_headers(self):
        """Test request header preparation."""
        input_headers = {
            "Content-Type": "application/json",
            "Authorization": "Bearer token",
            "Connection": "keep-alive",  # Should be removed
            "Proxy-Authorization": "Basic auth",  # Should be removed
            "Custom-Header": "value"
        }
        
        cleaned_headers = self.proxy_service._prepare_headers(input_headers)
        
        # Check that valid headers are preserved
        assert cleaned_headers["Content-Type"] == "application/json"
        assert cleaned_headers["Authorization"] == "Bearer token"
        assert cleaned_headers["Custom-Header"] == "value"
        
        # Check that hop-by-hop headers are removed
        assert "Connection" not in cleaned_headers
        assert "Proxy-Authorization" not in cleaned_headers
        
        # Check that gateway headers are added
        assert cleaned_headers["X-Gateway"] == "TracSeq-API-Gateway"
        assert cleaned_headers["X-Gateway-Version"] == "2.0.0"
        assert cleaned_headers["X-Forwarded-By"] == "TracSeq-Gateway"
    
    def test_prepare_response_headers(self):
        """Test response header preparation."""
        input_headers = {
            "Content-Type": "application/json",
            "Content-Length": "100",
            "Connection": "keep-alive",  # Should be removed
            "Transfer-Encoding": "chunked",  # Should be removed
            "Custom-Header": "value"
        }
        
        cleaned_headers = self.proxy_service._prepare_response_headers(input_headers)
        
        # Check that valid headers are preserved
        assert cleaned_headers["Content-Type"] == "application/json"
        assert cleaned_headers["Content-Length"] == "100"
        assert cleaned_headers["Custom-Header"] == "value"
        
        # Check that hop-by-hop headers are removed
        assert "Connection" not in cleaned_headers
        assert "Transfer-Encoding" not in cleaned_headers
        
        # Check that gateway headers are added
        assert cleaned_headers["X-Gateway"] == "TracSeq-API-Gateway"
        assert "X-Gateway-Request-ID" in cleaned_headers