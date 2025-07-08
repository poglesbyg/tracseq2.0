#!/usr/bin/env python3
"""
Test script for TracSeq API Gateway
Tests connectivity between frontend, gateway, and microservices
"""

import asyncio
import json
import sys
from typing import Dict, Any, List

import httpx
import structlog

# Configure logging
structlog.configure(
    processors=[
        structlog.stdlib.filter_by_level,
        structlog.stdlib.add_logger_name,
        structlog.stdlib.add_log_level,
        structlog.stdlib.PositionalArgumentsFormatter(),
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.StackInfoRenderer(),
        structlog.processors.format_exc_info,
        structlog.processors.UnicodeDecoder(),
        structlog.processors.JSONRenderer()
    ],
    context_class=dict,
    logger_factory=structlog.stdlib.LoggerFactory(),
    wrapper_class=structlog.stdlib.BoundLogger,
    cache_logger_on_first_use=True,
)

logger = structlog.get_logger(__name__)


class GatewayTester:
    """Test suite for API Gateway functionality."""
    
    def __init__(self, gateway_url: str = "http://localhost:8089"):
        self.gateway_url = gateway_url
        self.client = httpx.AsyncClient(timeout=30.0)
        self.results: List[Dict[str, Any]] = []
        
    async def __aenter__(self):
        return self
        
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self.client.aclose()
    
    async def test_endpoint(self, name: str, method: str, path: str, expected_status: int = 200, **kwargs) -> Dict[str, Any]:
        """Test a single endpoint."""
        url = f"{self.gateway_url}{path}"
        
        try:
            response = await self.client.request(method, url, **kwargs)
            
            result = {
                "name": name,
                "method": method,
                "path": path,
                "url": url,
                "status_code": response.status_code,
                "expected_status": expected_status,
                "success": response.status_code == expected_status,
                "response_time": response.elapsed.total_seconds() if response.elapsed else 0,
                "headers": dict(response.headers),
                "content_type": response.headers.get("content-type", "")
            }
            
            # Try to parse JSON response
            try:
                result["response"] = response.json()
            except Exception:
                result["response"] = response.text[:200] if response.text else ""
                
            logger.info("Test completed", **result)
            
        except Exception as e:
            result = {
                "name": name,
                "method": method,
                "path": path,
                "url": url,
                "status_code": 0,
                "expected_status": expected_status,
                "success": False,
                "error": str(e),
                "response_time": 0
            }
            logger.error("Test failed", **result)
        
        self.results.append(result)
        return result
    
    async def test_gateway_health(self):
        """Test gateway health endpoints."""
        logger.info("Testing gateway health endpoints...")
        
        await self.test_endpoint("Gateway Root", "GET", "/")
        await self.test_endpoint("Gateway Health", "GET", "/health")
        await self.test_endpoint("Gateway Services", "GET", "/services")
        await self.test_endpoint("Gateway Stats", "GET", "/gateway/stats")
        await self.test_endpoint("Gateway Docs", "GET", "/docs", expected_status=200)
    
    async def test_auth_endpoints(self):
        """Test authentication endpoints."""
        logger.info("Testing authentication endpoints...")
        
        # Test auth endpoints (should fail without proper auth service)
        await self.test_endpoint("Auth Login", "POST", "/api/auth/login", expected_status=503)
        await self.test_endpoint("Auth Users", "GET", "/api/users", expected_status=503)
        await self.test_endpoint("Auth Current User", "GET", "/api/users/me", expected_status=503)
    
    async def test_sample_endpoints(self):
        """Test sample management endpoints."""
        logger.info("Testing sample endpoints...")
        
        await self.test_endpoint("Sample List", "GET", "/api/samples", expected_status=503)
        await self.test_endpoint("Sample Create", "POST", "/api/samples", expected_status=503)
        await self.test_endpoint("Sample Batch", "POST", "/api/samples/batch", expected_status=503)
    
    async def test_storage_endpoints(self):
        """Test storage endpoints."""
        logger.info("Testing storage endpoints...")
        
        await self.test_endpoint("Storage Locations", "GET", "/api/storage/locations", expected_status=503)
        await self.test_endpoint("Storage Samples", "GET", "/api/storage/samples", expected_status=503)
        await self.test_endpoint("Storage Analytics", "GET", "/api/storage/analytics/utilization", expected_status=503)
    
    async def test_template_endpoints(self):
        """Test template endpoints."""
        logger.info("Testing template endpoints...")
        
        await self.test_endpoint("Template List", "GET", "/api/templates", expected_status=503)
        await self.test_endpoint("Template Upload", "POST", "/api/templates/upload", expected_status=503)
    
    async def test_laboratory_endpoints(self):
        """Test laboratory service endpoints."""
        logger.info("Testing laboratory endpoints...")
        
        await self.test_endpoint("Sequencing Jobs", "GET", "/api/sequencing/jobs", expected_status=503)
        await self.test_endpoint("QC Dashboard", "GET", "/api/qc/dashboard/stats", expected_status=503)
        await self.test_endpoint("Library Prep", "GET", "/api/library-prep/preparations", expected_status=503)
        await self.test_endpoint("Flow Cells", "GET", "/api/flow-cells/types", expected_status=503)
        await self.test_endpoint("Projects", "GET", "/api/projects", expected_status=503)
    
    async def test_enhanced_endpoints(self):
        """Test enhanced service endpoints."""
        logger.info("Testing enhanced service endpoints...")
        
        await self.test_endpoint("Notifications", "GET", "/api/notifications/list", expected_status=503)
        await self.test_endpoint("Events", "GET", "/api/events/recent", expected_status=503)
        await self.test_endpoint("Spreadsheets", "GET", "/api/spreadsheets/datasets", expected_status=503)
        await self.test_endpoint("Reports", "GET", "/api/reports/templates", expected_status=503)
        await self.test_endpoint("Dashboard", "GET", "/api/dashboard/stats", expected_status=503)
    
    async def test_ai_endpoints(self):
        """Test AI service endpoints."""
        logger.info("Testing AI service endpoints...")
        
        await self.test_endpoint("RAG Submissions", "GET", "/api/rag/submissions", expected_status=503)
        await self.test_endpoint("Chat Stream", "POST", "/api/chat/stream", expected_status=503)
    
    async def test_error_handling(self):
        """Test error handling."""
        logger.info("Testing error handling...")
        
        await self.test_endpoint("Not Found", "GET", "/api/nonexistent", expected_status=404)
        await self.test_endpoint("Invalid Method", "PATCH", "/api/invalid", expected_status=404)
        await self.test_endpoint("Deep Path", "GET", "/api/very/deep/path/that/should/not/exist", expected_status=404)
    
    async def run_all_tests(self):
        """Run all tests."""
        logger.info("Starting comprehensive gateway tests...")
        
        await self.test_gateway_health()
        await self.test_auth_endpoints()
        await self.test_sample_endpoints()
        await self.test_storage_endpoints()
        await self.test_template_endpoints()
        await self.test_laboratory_endpoints()
        await self.test_enhanced_endpoints()
        await self.test_ai_endpoints()
        await self.test_error_handling()
        
        logger.info("All tests completed")
    
    def print_summary(self):
        """Print test summary."""
        if not self.results:
            print("No tests were run.")
            return
        
        total_tests = len(self.results)
        successful_tests = sum(1 for r in self.results if r["success"])
        failed_tests = total_tests - successful_tests
        
        print(f"\n{'='*60}")
        print(f"TracSeq API Gateway Test Results")
        print(f"{'='*60}")
        print(f"Total Tests: {total_tests}")
        print(f"Successful: {successful_tests}")
        print(f"Failed: {failed_tests}")
        print(f"Success Rate: {(successful_tests/total_tests)*100:.1f}%")
        
        if failed_tests > 0:
            print(f"\nFailed Tests:")
            for result in self.results:
                if not result["success"]:
                    error = result.get("error", f"Status {result['status_code']}")
                    print(f"  - {result['name']}: {error}")
        
        print(f"\nSuccessful Tests:")
        for result in self.results:
            if result["success"]:
                print(f"  ✓ {result['name']} ({result['response_time']:.3f}s)")
        
        print(f"\n{'='*60}")
        
        # Return non-zero exit code if any tests failed
        return 0 if failed_tests == 0 else 1


async def main():
    """Main test runner."""
    gateway_url = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:8089"
    
    print(f"Testing TracSeq API Gateway at: {gateway_url}")
    print("Note: Many tests will show 503 errors - this is expected when microservices are not running")
    print("The important thing is that the gateway is routing requests properly\n")
    
    async with GatewayTester(gateway_url) as tester:
        await tester.run_all_tests()
        exit_code = tester.print_summary()
        
        # Save detailed results
        with open("gateway_test_results.json", "w") as f:
            json.dump(tester.results, f, indent=2)
        
        print(f"\nDetailed results saved to gateway_test_results.json")
        sys.exit(exit_code)


if __name__ == "__main__":
    asyncio.run(main())