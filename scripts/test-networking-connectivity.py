#!/usr/bin/env python3
"""
TracSeq 2.0 Network Connectivity Test
Tests connectivity between all microservices after Docker networking fixes.

Requirements: pip install aiohttp
"""

import asyncio
import sys
import time
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
from datetime import datetime

try:
    import aiohttp
except ImportError:
    print("❌ Error: aiohttp is required. Install with: pip install aiohttp")
    sys.exit(1)

@dataclass
class ServiceTest:
    name: str
    container_name: str
    port: int
    health_endpoint: Optional[str]
    expected_response: int = 200

# Define all services with their correct container names and ports
SERVICES = [
    ServiceTest("PostgreSQL", "lims-postgres", 5432, None),  # No HTTP health check
    ServiceTest("Redis", "lims-redis", 6379, None),  # No HTTP health check
    ServiceTest("API Gateway", "lims-gateway", 8000, "/health"),
    ServiceTest("Auth Service", "lims-auth", 8080, "/health"),
    ServiceTest("Sample Service", "lims-samples", 8081, "/health"),
    ServiceTest("Storage Service", "lims-storage", 8082, "/health"),
    ServiceTest("Template Service", "lims-templates", 8083, "/health"),
    ServiceTest("Sequencing Service", "lims-sequencing", 8084, "/health"),
    ServiceTest("Notification Service", "lims-notification", 8085, "/health"),
    ServiceTest("RAG Service", "lims-rag", 8086, "/health"),
    ServiceTest("Event Service", "lims-events", 8087, "/health"),
    ServiceTest("Transaction Service", "lims-transactions", 8088, "/health"),
    ServiceTest("Project Service", "lims-projects", 8101, "/health"),
    ServiceTest("Library Prep Service", "lims-library-prep", 8102, "/health"),
    ServiceTest("QA/QC Service", "lims-qaqc", 8103, "/health"),
    ServiceTest("Flow Cell Service", "lims-flow-cells", 8104, "/health"),
    ServiceTest("Dashboard Service", "lims-dashboard", 8015, "/health"),
    ServiceTest("Spreadsheet Service", "lims-spreadsheet", 8088, "/health"),
    ServiceTest("Ollama Service", "lims-ollama", 11434, "/api/version"),
]

# Service discovery test routes through API Gateway
GATEWAY_ROUTES = [
    ("/api/auth/health", "Auth Service via Gateway"),
    ("/api/samples/health", "Sample Service via Gateway"),
    ("/api/storage/health", "Storage Service via Gateway"),
    ("/api/templates/health", "Template Service via Gateway"),
    ("/api/sequencing/health", "Sequencing Service via Gateway"),
    ("/api/notifications/health", "Notification Service via Gateway"),
    ("/api/rag/health", "RAG Service via Gateway"),
    ("/api/events/health", "Event Service via Gateway"),
    ("/api/transactions/health", "Transaction Service via Gateway"),
    ("/api/projects/health", "Project Service via Gateway"),
    ("/api/qaqc/health", "QA/QC Service via Gateway"),
    ("/api/library-prep/health", "Library Prep Service via Gateway"),
    ("/api/flow-cells/health", "Flow Cell Service via Gateway"),
    ("/api/dashboard/health", "Dashboard Service via Gateway"),
    ("/api/spreadsheets/health", "Spreadsheet Service via Gateway"),
]

class NetworkTester:
    def __init__(self):
        self.results: Dict[str, Dict] = {}
        self.gateway_url = "http://localhost:8000"
        
    async def test_service_health(self, service: ServiceTest) -> Tuple[bool, str, float]:
        """Test individual service health endpoint."""
        if not service.health_endpoint:
            return True, "No HTTP health check", 0.0
            
        url = f"http://localhost:{service.port}{service.health_endpoint}"
        start_time = time.time()
        
        try:
            async with aiohttp.ClientSession(timeout=aiohttp.ClientTimeout(total=10)) as session:
                async with session.get(url) as response:
                    response_time = time.time() - start_time
                    
                    if response.status == service.expected_response:
                        return True, f"HTTP {response.status}", response_time
                    else:
                        return False, f"HTTP {response.status} (expected {service.expected_response})", response_time
                        
        except aiohttp.ClientConnectorError:
            response_time = time.time() - start_time
            return False, "Connection refused", response_time
        except asyncio.TimeoutError:
            response_time = time.time() - start_time
            return False, "Timeout", response_time
        except Exception as e:
            response_time = time.time() - start_time
            return False, f"Error: {str(e)}", response_time

    async def test_gateway_routing(self, route: str, description: str) -> Tuple[bool, str, float]:
        """Test API Gateway routing to services."""
        url = f"{self.gateway_url}{route}"
        start_time = time.time()
        
        try:
            async with aiohttp.ClientSession(timeout=aiohttp.ClientTimeout(total=15)) as session:
                async with session.get(url) as response:
                    response_time = time.time() - start_time
                    
                    # Accept both 200 (healthy) and 503 (service unavailable but gateway working)
                    if response.status in [200, 503]:
                        return True, f"HTTP {response.status}", response_time
                    else:
                        return False, f"HTTP {response.status}", response_time
                        
        except aiohttp.ClientConnectorError:
            response_time = time.time() - start_time
            return False, "Gateway connection refused", response_time
        except asyncio.TimeoutError:
            response_time = time.time() - start_time
            return False, "Gateway timeout", response_time
        except Exception as e:
            response_time = time.time() - start_time
            return False, f"Error: {str(e)}", response_time

    async def test_database_connectivity(self) -> Tuple[bool, str]:
        """Test database connectivity through a service."""
        try:
            # Test database connectivity through API Gateway health endpoint
            async with aiohttp.ClientSession(timeout=aiohttp.ClientTimeout(total=10)) as session:
                async with session.get(f"{self.gateway_url}/api/health") as response:
                    if response.status == 200:
                        data = await response.json()
                        db_healthy = data.get("database", {}).get("healthy", False)
                        if db_healthy:
                            return True, "Database accessible via gateway"
                        else:
                            return False, "Database reported as unhealthy"
                    else:
                        return False, f"Gateway health check failed: HTTP {response.status}"
        except Exception as e:
            return False, f"Database connectivity test failed: {str(e)}"

    def print_header(self, title: str):
        """Print a formatted header."""
        print(f"\n{'='*80}")
        print(f" {title}")
        print(f"{'='*80}")

    def print_result(self, name: str, success: bool, message: str, response_time: float = 0.0):
        """Print a formatted test result."""
        status = "✅ PASS" if success else "❌ FAIL"
        time_str = f"({response_time:.3f}s)" if response_time > 0 else ""
        print(f"{status} {name:<40} {message:<30} {time_str}")

    async def run_all_tests(self):
        """Run all connectivity tests."""
        print(f"\n🚀 TracSeq 2.0 Network Connectivity Test")
        print(f"Started at: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        
        # Test 1: Individual service health checks
        self.print_header("1. Individual Service Health Checks")
        
        service_results = []
        for service in SERVICES:
            success, message, response_time = await self.test_service_health(service)
            self.print_result(service.name, success, message, response_time)
            service_results.append((service.name, success))
            
        # Test 2: API Gateway routing
        self.print_header("2. API Gateway Service Discovery & Routing")
        
        gateway_results = []
        for route, description in GATEWAY_ROUTES:
            success, message, response_time = await self.test_gateway_routing(route, description)
            self.print_result(description, success, message, response_time)
            gateway_results.append((description, success))

        # Test 3: Database connectivity
        self.print_header("3. Database Connectivity Test")
        
        db_success, db_message = await self.test_database_connectivity()
        self.print_result("Database Connectivity", db_success, db_message)

        # Summary
        self.print_header("Test Summary")
        
        service_passed = sum(1 for _, success in service_results if success)
        service_total = len(service_results)
        
        gateway_passed = sum(1 for _, success in gateway_results if success)
        gateway_total = len(gateway_results)
        
        print(f"✅ Service Health Checks:     {service_passed}/{service_total} passed")
        print(f"✅ Gateway Routing Tests:     {gateway_passed}/{gateway_total} passed")
        print(f"✅ Database Connectivity:     {'1/1' if db_success else '0/1'} passed")
        
        total_passed = service_passed + gateway_passed + (1 if db_success else 0)
        total_tests = service_total + gateway_total + 1
        
        print(f"\n🎯 Overall Results: {total_passed}/{total_tests} tests passed")
        
        if total_passed == total_tests:
            print("🎉 All networking tests PASSED! Services are properly connected.")
            return 0
        else:
            print("⚠️  Some tests FAILED. Check service configurations and Docker networking.")
            
            # Print failed services for debugging
            failed_services = [name for name, success in service_results if not success]
            failed_gateway = [name for name, success in gateway_results if not success]
            
            if failed_services:
                print(f"\n❌ Failed Service Health Checks: {', '.join(failed_services)}")
            if failed_gateway:
                print(f"❌ Failed Gateway Routes: {', '.join(failed_gateway)}")
            if not db_success:
                print(f"❌ Database Connectivity: {db_message}")
                
            return 1

    async def test_specific_service(self, service_name: str):
        """Test connectivity for a specific service."""
        service = next((s for s in SERVICES if s.name.lower() == service_name.lower()), None)
        if not service:
            print(f"❌ Service '{service_name}' not found.")
            print(f"Available services: {', '.join(s.name for s in SERVICES)}")
            return 1
            
        print(f"🔍 Testing connectivity for: {service.name}")
        success, message, response_time = await self.test_service_health(service)
        self.print_result(service.name, success, message, response_time)
        
        return 0 if success else 1

async def main():
    """Main function."""
    tester = NetworkTester()
    
    if len(sys.argv) > 1:
        # Test specific service
        service_name = sys.argv[1]
        exit_code = await tester.test_specific_service(service_name)
    else:
        # Run all tests
        exit_code = await tester.run_all_tests()
    
    sys.exit(exit_code)

if __name__ == "__main__":
    asyncio.run(main()) 