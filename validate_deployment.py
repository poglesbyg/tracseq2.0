#!/usr/bin/env python3
"""
TracSeq 2.0 Deployment Validation Script
Comprehensive testing of deployed microservices and infrastructure
"""

import os
import sys
import time
import json
import requests
import subprocess
from datetime import datetime
from typing import Dict, List, Tuple, Any
from dataclasses import dataclass
import concurrent.futures
import threading

@dataclass
class ServiceConfig:
    name: str
    port: int
    health_endpoint: str = "/health"
    critical: bool = True
    timeout: int = 10

@dataclass
class TestResult:
    service: str
    test_name: str
    status: str  # PASS, FAIL, WARN, SKIP
    response_time: float
    details: str
    error: str = ""

class DeploymentValidator:
    def __init__(self):
        self.services = [
            ServiceConfig("auth-service", 8080, critical=True),
            ServiceConfig("sample-service", 8081, critical=True),
            ServiceConfig("template-service", 8083, critical=True),
            ServiceConfig("sequencing-service", 8084, critical=True),
            ServiceConfig("notification-service", 8085, critical=True),
            ServiceConfig("transaction-service", 8088, critical=True),
            ServiceConfig("api-gateway", 8089, critical=False),  # Phase 2
            ServiceConfig("rag-service", 8086, critical=False),  # Phase 2
        ]
        
        self.infrastructure = [
            ServiceConfig("postgres", 5432, health_endpoint="", critical=True),
            ServiceConfig("redis", 6379, health_endpoint="", critical=True),
            ServiceConfig("prometheus", 9090, health_endpoint="/api/v1/status/config", critical=False),
            ServiceConfig("grafana", 3001, health_endpoint="/api/health", critical=False),
        ]
        
        self.results: List[TestResult] = []
        self.start_time = time.time()
        self.session = requests.Session()
        self.session.timeout = 10
        
    def log(self, message: str, level: str = "INFO"):
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        color_codes = {
            "INFO": "\033[94m",    # Blue
            "SUCCESS": "\033[92m", # Green
            "WARNING": "\033[93m", # Yellow
            "ERROR": "\033[91m",   # Red
            "RESET": "\033[0m"     # Reset
        }
        
        color = color_codes.get(level, color_codes["INFO"])
        reset = color_codes["RESET"]
        print(f"{color}[{timestamp}] {level}: {message}{reset}")

    def add_result(self, service: str, test_name: str, status: str, 
                   response_time: float, details: str, error: str = ""):
        result = TestResult(service, test_name, status, response_time, details, error)
        self.results.append(result)
        
        status_symbol = {
            "PASS": "✅",
            "FAIL": "❌", 
            "WARN": "⚠️",
            "SKIP": "⏭️"
        }.get(status, "❓")
        
        log_level = {
            "PASS": "SUCCESS",
            "FAIL": "ERROR",
            "WARN": "WARNING", 
            "SKIP": "INFO"
        }.get(status, "INFO")
        
        self.log(f"{status_symbol} {service} {test_name}: {details} ({response_time:.2f}s)", log_level)

    def test_container_status(self) -> Dict[str, bool]:
        """Check if Docker containers are running"""
        try:
            result = subprocess.run(
                ["docker", "ps", "--format", "{{.Names}}"], 
                capture_output=True, text=True, timeout=10
            )
            running_containers = set(result.stdout.strip().split('\n'))
            return {
                container: container in running_containers 
                for container in [
                    "tracseq-postgres-primary",
                    "tracseq-redis-primary", 
                    "tracseq-auth-service",
                    "tracseq-sample-service",
                    "tracseq-template-service",
                    "tracseq-notification-service",
                    "tracseq-sequencing-service",
                    "tracseq-transaction-service",
                ]
            }
        except subprocess.TimeoutExpired:
            return {}
        except Exception as e:
            self.log(f"Error checking container status: {e}", "ERROR")
            return {}

    def test_port_connectivity(self, host: str = "localhost", port: int = 80, timeout: int = 5) -> bool:
        """Test if port is accessible"""
        import socket
        try:
            with socket.create_connection((host, port), timeout=timeout):
                return True
        except (socket.timeout, socket.error):
            return False

    def test_http_endpoint(self, url: str, timeout: int = 10) -> Tuple[bool, float, str]:
        """Test HTTP endpoint accessibility"""
        start_time = time.time()
        try:
            response = self.session.get(url, timeout=timeout)
            response_time = time.time() - start_time
            
            if response.status_code == 200:
                return True, response_time, f"HTTP 200 OK"
            else:
                return False, response_time, f"HTTP {response.status_code}"
                
        except requests.exceptions.ConnectTimeout:
            response_time = time.time() - start_time
            return False, response_time, "Connection timeout"
        except requests.exceptions.ConnectionError:
            response_time = time.time() - start_time
            return False, response_time, "Connection refused"
        except Exception as e:
            response_time = time.time() - start_time
            return False, response_time, f"Error: {str(e)}"

    def test_database_connectivity(self) -> None:
        """Test PostgreSQL database connectivity"""
        try:
            result = subprocess.run([
                "docker", "exec", "tracseq-postgres-primary", 
                "pg_isready", "-U", "tracseq_admin", "-d", "tracseq_prod"
            ], capture_output=True, text=True, timeout=10)
            
            if result.returncode == 0:
                self.add_result("postgres", "connectivity", "PASS", 0.1, 
                              "Database accepting connections")
                
                # Test query execution
                query_result = subprocess.run([
                    "docker", "exec", "tracseq-postgres-primary",
                    "psql", "-U", "tracseq_admin", "-d", "tracseq_prod", 
                    "-c", "SELECT version();"
                ], capture_output=True, text=True, timeout=10)
                
                if query_result.returncode == 0:
                    self.add_result("postgres", "query_execution", "PASS", 0.2,
                                  "Query execution successful")
                else:
                    self.add_result("postgres", "query_execution", "FAIL", 0.2,
                                  "Query execution failed", query_result.stderr)
            else:
                self.add_result("postgres", "connectivity", "FAIL", 0.1,
                              "Database not accepting connections", result.stderr)
                
        except subprocess.TimeoutExpired:
            self.add_result("postgres", "connectivity", "FAIL", 10.0,
                          "Database connectivity test timed out")
        except Exception as e:
            self.add_result("postgres", "connectivity", "FAIL", 0.1,
                          "Database connectivity test error", str(e))

    def test_redis_connectivity(self) -> None:
        """Test Redis connectivity"""
        try:
            result = subprocess.run([
                "docker", "exec", "tracseq-redis-primary", "redis-cli", "ping"
            ], capture_output=True, text=True, timeout=10)
            
            if result.returncode == 0 and "PONG" in result.stdout:
                self.add_result("redis", "connectivity", "PASS", 0.1,
                              "Redis responding to ping")
                
                # Test basic operations
                set_result = subprocess.run([
                    "docker", "exec", "tracseq-redis-primary", 
                    "redis-cli", "set", "test_key", "test_value"
                ], capture_output=True, text=True, timeout=5)
                
                get_result = subprocess.run([
                    "docker", "exec", "tracseq-redis-primary",
                    "redis-cli", "get", "test_key"
                ], capture_output=True, text=True, timeout=5)
                
                if (set_result.returncode == 0 and get_result.returncode == 0 
                    and "test_value" in get_result.stdout):
                    self.add_result("redis", "operations", "PASS", 0.1,
                                  "Basic Redis operations working")
                else:
                    self.add_result("redis", "operations", "WARN", 0.1,
                                  "Redis basic operations may have issues")
            else:
                self.add_result("redis", "connectivity", "FAIL", 0.1,
                              "Redis not responding to ping", result.stderr)
                
        except subprocess.TimeoutExpired:
            self.add_result("redis", "connectivity", "FAIL", 10.0,
                          "Redis connectivity test timed out")
        except Exception as e:
            self.add_result("redis", "connectivity", "FAIL", 0.1,
                          "Redis connectivity test error", str(e))

    def test_service_health(self, service: ServiceConfig) -> None:
        """Test individual service health"""
        # Port connectivity test
        port_open = self.test_port_connectivity("localhost", service.port, 5)
        if not port_open:
            self.add_result(service.name, "port_connectivity", "FAIL", 5.0,
                          f"Port {service.port} not accessible")
            return
        
        self.add_result(service.name, "port_connectivity", "PASS", 0.1,
                      f"Port {service.port} accessible")
        
        # Health endpoint test
        if service.health_endpoint:
            url = f"http://localhost:{service.port}{service.health_endpoint}"
            success, response_time, details = self.test_http_endpoint(url)
            
            if success:
                self.add_result(service.name, "health_endpoint", "PASS", 
                              response_time, details)
            else:
                status = "FAIL" if service.critical else "WARN"
                self.add_result(service.name, "health_endpoint", status,
                              response_time, details)

    def test_authentication_flow(self) -> None:
        """Test complete authentication flow"""
        auth_url = "http://localhost:8080/api/auth/login"
        test_credentials = {
            "email": "admin@lab.local",
            "password": "admin123"
        }
        
        try:
            start_time = time.time()
            response = self.session.post(auth_url, json=test_credentials, timeout=10)
            response_time = time.time() - start_time
            
            if response.status_code == 200:
                try:
                    data = response.json()
                    if "token" in data or "access_token" in data:
                        token = data.get("token") or data.get("access_token")
                        self.add_result("auth-flow", "login", "PASS", response_time,
                                      "Login successful, token received")
                        
                        # Test token validation
                        self.test_token_validation(token)
                    else:
                        self.add_result("auth-flow", "login", "WARN", response_time,
                                      "Login successful but no token in response")
                except json.JSONDecodeError:
                    self.add_result("auth-flow", "login", "WARN", response_time,
                                  "Login successful but invalid JSON response")
            else:
                self.add_result("auth-flow", "login", "FAIL", response_time,
                              f"Login failed with status {response.status_code}")
                
        except Exception as e:
            self.add_result("auth-flow", "login", "FAIL", 10.0,
                          "Login test failed", str(e))

    def test_token_validation(self, token: str) -> None:
        """Test JWT token validation"""
        me_url = "http://localhost:8080/api/users/me"
        headers = {"Authorization": f"Bearer {token}"}
        
        try:
            start_time = time.time()
            response = self.session.get(me_url, headers=headers, timeout=10)
            response_time = time.time() - start_time
            
            if response.status_code == 200:
                self.add_result("auth-flow", "token_validation", "PASS", 
                              response_time, "Token validation successful")
            else:
                self.add_result("auth-flow", "token_validation", "FAIL",
                              response_time, f"Token validation failed: {response.status_code}")
                
        except Exception as e:
            self.add_result("auth-flow", "token_validation", "FAIL", 10.0,
                          "Token validation test failed", str(e))

    def test_api_gateway_routing(self) -> None:
        """Test API Gateway routing to services"""
        test_routes = [
            ("/api/auth/health", "auth-service"),
            ("/api/samples/health", "sample-service"),
            ("/api/templates/health", "template-service"),
            ("/api/sequencing/health", "sequencing-service"), 
            ("/api/notifications/health", "notification-service"),
        ]
        
        for route, service in test_routes:
            url = f"http://localhost:8089{route}"
            success, response_time, details = self.test_http_endpoint(url)
            
            if success:
                self.add_result("api-gateway", f"routing_{service}", "PASS",
                              response_time, f"Routing to {service} successful")
            else:
                self.add_result("api-gateway", f"routing_{service}", "WARN",
                              response_time, f"Routing to {service} failed: {details}")

    def test_inter_service_communication(self) -> None:
        """Test communication between services"""
        # This would require more complex setup, so we'll test basic connectivity
        service_pairs = [
            ("sample-service", "auth-service"),
            ("template-service", "auth-service"),
            ("notification-service", "auth-service"),
            ("sequencing-service", "sample-service"),
        ]
        
        for service1, service2 in service_pairs:
            # Test if services can resolve each other (basic Docker networking)
            try:
                result = subprocess.run([
                    "docker", "exec", f"tracseq-{service1.replace('-', '-')}", 
                    "nslookup", service2
                ], capture_output=True, text=True, timeout=5)
                
                if result.returncode == 0:
                    self.add_result("inter-service", f"{service1}_to_{service2}", 
                                  "PASS", 0.1, "DNS resolution successful")
                else:
                    self.add_result("inter-service", f"{service1}_to_{service2}",
                                  "WARN", 0.1, "DNS resolution failed")
                                  
            except Exception as e:
                self.add_result("inter-service", f"{service1}_to_{service2}",
                              "SKIP", 0.1, "Inter-service test skipped", str(e))

    def run_container_tests(self) -> None:
        """Test Docker container status"""
        self.log("🐳 Testing Docker container status...")
        container_status = self.test_container_status()
        
        for container, is_running in container_status.items():
            if is_running:
                self.add_result("docker", f"container_{container}", "PASS", 0.1,
                              "Container running")
            else:
                self.add_result("docker", f"container_{container}", "FAIL", 0.1,
                              "Container not running")

    def run_infrastructure_tests(self) -> None:
        """Test infrastructure services"""
        self.log("🏗️ Testing infrastructure services...")
        
        # Database tests
        self.test_database_connectivity()
        
        # Redis tests
        self.test_redis_connectivity()
        
        # Other infrastructure services
        for service in self.infrastructure[2:]:  # Skip postgres and redis (tested above)
            self.test_service_health(service)

    def run_service_tests(self) -> None:
        """Test microservices"""
        self.log("🔧 Testing microservices...")
        
        # Test services in parallel for faster execution
        with concurrent.futures.ThreadPoolExecutor(max_workers=6) as executor:
            future_to_service = {
                executor.submit(self.test_service_health, service): service 
                for service in self.services
            }
            
            for future in concurrent.futures.as_completed(future_to_service):
                service = future_to_service[future]
                try:
                    future.result()
                except Exception as e:
                    self.add_result(service.name, "service_test", "FAIL", 0.1,
                                  "Service test failed", str(e))

    def run_integration_tests(self) -> None:
        """Test service integration"""
        self.log("🔗 Testing service integration...")
        
        # Authentication flow
        self.test_authentication_flow()
        
        # API Gateway routing (if gateway is running)
        if self.test_port_connectivity("localhost", 8089, 2):
            self.test_api_gateway_routing()
        else:
            self.add_result("api-gateway", "routing_test", "SKIP", 0.1,
                          "API Gateway not accessible, skipping routing tests")
        
        # Inter-service communication
        self.test_inter_service_communication()

    def generate_report(self) -> None:
        """Generate comprehensive deployment report"""
        total_time = time.time() - self.start_time
        
        # Categorize results
        results_by_status = {"PASS": [], "FAIL": [], "WARN": [], "SKIP": []}
        results_by_service = {}
        
        for result in self.results:
            results_by_status[result.status].append(result)
            if result.service not in results_by_service:
                results_by_service[result.service] = []
            results_by_service[result.service].append(result)
        
        # Calculate metrics
        total_tests = len(self.results)
        pass_count = len(results_by_status["PASS"])
        fail_count = len(results_by_status["FAIL"]) 
        warn_count = len(results_by_status["WARN"])
        skip_count = len(results_by_status["SKIP"])
        
        pass_rate = (pass_count / total_tests * 100) if total_tests > 0 else 0
        
        # Critical service status
        critical_services = [s.name for s in self.services if s.critical]
        critical_failures = [
            r for r in results_by_status["FAIL"] 
            if r.service in critical_services
        ]
        
        # Generate report
        print("\n" + "="*80)
        print("📊 TRACSEQ 2.0 DEPLOYMENT VALIDATION REPORT")
        print("="*80)
        print(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        print(f"Total Execution Time: {total_time:.2f} seconds")
        print(f"Total Tests: {total_tests}")
        
        print(f"\n📈 TEST RESULTS:")
        print(f"✅ PASSED: {pass_count} ({pass_rate:.1f}%)")
        print(f"❌ FAILED: {fail_count}")
        print(f"⚠️  WARNINGS: {warn_count}")
        print(f"⏭️  SKIPPED: {skip_count}")
        
        # Overall status
        if fail_count == 0:
            print(f"\n🎉 DEPLOYMENT STATUS: SUCCESS")
        elif len(critical_failures) == 0:
            print(f"\n⚠️  DEPLOYMENT STATUS: SUCCESS WITH WARNINGS")
        else:
            print(f"\n🚨 DEPLOYMENT STATUS: CRITICAL ISSUES DETECTED")
        
        # Service summary
        print(f"\n🔧 SERVICE STATUS:")
        for service_name in sorted(results_by_service.keys()):
            service_results = results_by_service[service_name]
            service_passes = sum(1 for r in service_results if r.status == "PASS")
            service_total = len(service_results)
            service_rate = (service_passes / service_total * 100) if service_total > 0 else 0
            
            status_symbol = "✅" if service_rate >= 90 else "⚠️" if service_rate >= 70 else "❌"
            print(f"{status_symbol} {service_name}: {service_passes}/{service_total} tests passed ({service_rate:.1f}%)")
        
        # Critical failures
        if critical_failures:
            print(f"\n🚨 CRITICAL ISSUES:")
            for failure in critical_failures:
                print(f"❌ {failure.service} - {failure.test_name}: {failure.details}")
                if failure.error:
                    print(f"   Error: {failure.error}")
        
        # Warnings
        if results_by_status["WARN"]:
            print(f"\n⚠️  WARNINGS:")
            for warning in results_by_status["WARN"][:5]:  # Show first 5 warnings
                print(f"⚠️  {warning.service} - {warning.test_name}: {warning.details}")
            
            if len(results_by_status["WARN"]) > 5:
                print(f"   ... and {len(results_by_status['WARN']) - 5} more warnings")
        
        # Recommendations
        print(f"\n💡 RECOMMENDATIONS:")
        
        if critical_failures:
            print("1. 🚨 Fix critical service failures before proceeding to production")
            print("2. 🔧 Check service logs: docker logs <service-name>")
            print("3. 🔄 Restart failed services: docker-compose restart <service>")
        elif results_by_status["WARN"]:
            print("1. ⚠️  Review warnings for potential issues")
            print("2. 📊 Monitor service performance in production")
            print("3. 🔧 Consider deploying Phase 2 services")
        else:
            print("1. 🎉 Deployment is ready for production!")
            print("2. 📊 Set up monitoring dashboards in Grafana")
            print("3. 🔄 Configure automated backups")
            print("4. 📝 Train users on the new system")
        
        # Next steps
        print(f"\n🎯 NEXT STEPS:")
        if len(critical_failures) == 0:
            print("✅ Core services are operational")
            print("✅ Configure external integrations (SMTP, SMS, Slack)")
            print("✅ Set up SSL certificates for production domain")
            print("✅ Deploy frontend application")
            print("✅ Run user acceptance testing")
        else:
            print("🔧 Fix critical service issues")
            print("🔄 Rerun validation: python validate_deployment.py")
            print("📝 Check troubleshooting guide in DEPLOYMENT_GUIDE.md")
        
        print(f"\n📚 RESOURCES:")
        print("• Deployment Guide: DEPLOYMENT_GUIDE.md")
        print("• Health Check: ./scripts/comprehensive-health-check.sh")
        print("• Service Logs: docker logs <service-name>")
        print("• Grafana Monitoring: http://localhost:3001")
        print("• Prometheus Metrics: http://localhost:9090")
        
        print("="*80)
        
        # Exit with appropriate code
        if len(critical_failures) > 0:
            sys.exit(1)
        elif fail_count > 0 or warn_count > 3:
            sys.exit(2)
        else:
            sys.exit(0)

    def run_validation(self) -> None:
        """Run complete deployment validation"""
        self.log("🚀 Starting TracSeq 2.0 Deployment Validation", "INFO")
        self.log(f"Validation started at {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        
        try:
            # Test sequence
            self.run_container_tests()
            self.run_infrastructure_tests()
            self.run_service_tests()
            self.run_integration_tests()
            
            # Generate final report
            self.generate_report()
            
        except KeyboardInterrupt:
            self.log("Validation interrupted by user", "WARNING")
            sys.exit(130)
        except Exception as e:
            self.log(f"Validation failed with error: {e}", "ERROR")
            sys.exit(1)

def main():
    """Main entry point"""
    print("🧪 TracSeq 2.0 Deployment Validation")
    print("=====================================")
    
    validator = DeploymentValidator()
    validator.run_validation()

if __name__ == "__main__":
    main() 
