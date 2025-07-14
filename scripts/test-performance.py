#!/usr/bin/env python3
"""
TracSeq 2.0 - Advanced Performance Testing Framework
Tests performance, load, and stress scenarios across all microservices
"""

import asyncio
import aiohttp
import json
import time
import statistics
import argparse
import sys
from datetime import datetime, timedelta
from typing import Dict, List, Tuple, Optional, Any
from dataclasses import dataclass, asdict
from concurrent.futures import ThreadPoolExecutor, as_completed
import threading
import psutil
import os

# Test configuration
TEST_CONFIG = {
    'api_gateway_url': 'http://localhost:8089',
    'frontend_proxy_url': 'http://localhost:8085',
    'services': {
        'dashboard': 'http://localhost:8080',
        'samples': 'http://localhost:8081',
        'sequencing': 'http://localhost:8082',
        'spreadsheet': 'http://localhost:8083'
    },
    'test_duration': 60,  # seconds
    'concurrent_users': [1, 5, 10, 20, 50],
    'ramp_up_time': 10,  # seconds
    'think_time': 1,  # seconds between requests
}

@dataclass
class TestResult:
    """Test result data structure"""
    test_name: str
    start_time: datetime
    end_time: datetime
    duration: float
    total_requests: int
    successful_requests: int
    failed_requests: int
    avg_response_time: float
    min_response_time: float
    max_response_time: float
    p95_response_time: float
    p99_response_time: float
    requests_per_second: float
    errors: List[str]
    resource_usage: Dict[str, Any]

@dataclass
class LoadTestResult:
    """Load test result data structure"""
    concurrent_users: int
    test_duration: float
    total_requests: int
    successful_requests: int
    failed_requests: int
    avg_response_time: float
    requests_per_second: float
    cpu_usage: float
    memory_usage: float
    errors: List[str]

class PerformanceMonitor:
    """Monitor system performance during tests"""
    
    def __init__(self):
        self.monitoring = False
        self.metrics = []
        self.monitor_thread = None
        
    def start_monitoring(self):
        """Start performance monitoring"""
        self.monitoring = True
        self.metrics = []
        self.monitor_thread = threading.Thread(target=self._monitor_loop)
        self.monitor_thread.start()
        
    def stop_monitoring(self):
        """Stop performance monitoring"""
        self.monitoring = False
        if self.monitor_thread:
            self.monitor_thread.join()
            
    def _monitor_loop(self):
        """Monitor system resources"""
        while self.monitoring:
            try:
                cpu_percent = psutil.cpu_percent(interval=1)
                memory = psutil.virtual_memory()
                disk = psutil.disk_usage('/')
                
                self.metrics.append({
                    'timestamp': datetime.utcnow().isoformat(),
                    'cpu_percent': cpu_percent,
                    'memory_percent': memory.percent,
                    'memory_available': memory.available,
                    'disk_percent': disk.percent,
                    'disk_free': disk.free
                })
            except Exception as e:
                print(f"Monitoring error: {e}")
                
            time.sleep(1)
            
    def get_average_metrics(self) -> Dict[str, float]:
        """Get average performance metrics"""
        if not self.metrics:
            return {}
            
        return {
            'avg_cpu_percent': statistics.mean(m['cpu_percent'] for m in self.metrics),
            'avg_memory_percent': statistics.mean(m['memory_percent'] for m in self.metrics),
            'max_cpu_percent': max(m['cpu_percent'] for m in self.metrics),
            'max_memory_percent': max(m['memory_percent'] for m in self.metrics),
        }

class PerformanceTester:
    """Advanced performance testing framework"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.monitor = PerformanceMonitor()
        self.results = []
        
    async def single_request_test(self, session, url: str, method: str = 'GET', 
                                data: Optional[Dict] = None) -> Tuple[float, int, str]:
        """Execute a single HTTP request and measure performance"""
        start_time = time.time()
        
        try:
            if method.upper() == 'GET':
                async with session.get(url) as response:
                    await response.text()
                    return time.time() - start_time, response.status, ""
            elif method.upper() == 'POST':
                async with session.post(url, json=data) as response:
                    await response.text()
                    return time.time() - start_time, response.status, ""
            else:
                return time.time() - start_time, 0, f"Unsupported method: {method}"
        except Exception as e:
            return time.time() - start_time, 0, str(e)
            
    async def health_check_performance_test(self) -> TestResult:
        """Test health check endpoints performance"""
        print("🔍 Running Health Check Performance Test...")
        
        start_time = datetime.utcnow()
        response_times = []
        errors = []
        successful_requests = 0
        failed_requests = 0
        
        # Test endpoints
        endpoints = [
            f"{self.config['api_gateway_url']}/api/dashboard/health",
            f"{self.config['api_gateway_url']}/api/samples/health",
            f"{self.config['api_gateway_url']}/api/sequencing/health",
            f"{self.config['api_gateway_url']}/api/spreadsheet/health",
        ]
        
        self.monitor.start_monitoring()
        
        async with aiohttp.ClientSession() as session:
            # Run 100 requests to each endpoint
            for _ in range(100):
                for endpoint in endpoints:
                    response_time, status_code, error = await self.single_request_test(session, endpoint)
                    response_times.append(response_time)
                    
                    if status_code == 200:
                        successful_requests += 1
                    else:
                        failed_requests += 1
                        if error:
                            errors.append(f"{endpoint}: {error}")
                            
        self.monitor.stop_monitoring()
        end_time = datetime.utcnow()
        
        # Calculate statistics
        duration = (end_time - start_time).total_seconds()
        total_requests = len(response_times)
        
        return TestResult(
            test_name="Health Check Performance",
            start_time=start_time,
            end_time=end_time,
            duration=duration,
            total_requests=total_requests,
            successful_requests=successful_requests,
            failed_requests=failed_requests,
            avg_response_time=statistics.mean(response_times) if response_times else 0,
            min_response_time=min(response_times) if response_times else 0,
            max_response_time=max(response_times) if response_times else 0,
            p95_response_time=statistics.quantiles(response_times, n=20)[18] if len(response_times) >= 20 else 0,
            p99_response_time=statistics.quantiles(response_times, n=100)[98] if len(response_times) >= 100 else 0,
            requests_per_second=total_requests / duration if duration > 0 else 0,
            errors=errors,
            resource_usage=self.monitor.get_average_metrics()
        )
        
    async def api_endpoint_performance_test(self) -> TestResult:
        """Test API endpoint performance"""
        print("🔍 Running API Endpoint Performance Test...")
        
        start_time = datetime.utcnow()
        response_times = []
        errors = []
        successful_requests = 0
        failed_requests = 0
        
        # Test endpoints
        endpoints = [
            f"{self.config['api_gateway_url']}/api/samples/v1/samples",
            f"{self.config['api_gateway_url']}/api/dashboard/v1/users",
            f"{self.config['api_gateway_url']}/api/dashboard/v1/storage/locations",
            f"{self.config['api_gateway_url']}/api/sequencing/v1/jobs",
            f"{self.config['api_gateway_url']}/api/spreadsheet/v1/templates",
        ]
        
        self.monitor.start_monitoring()
        
        async with aiohttp.ClientSession() as session:
            # Run 50 requests to each endpoint
            for _ in range(50):
                for endpoint in endpoints:
                    response_time, status_code, error = await self.single_request_test(session, endpoint)
                    response_times.append(response_time)
                    
                    if status_code == 200:
                        successful_requests += 1
                    else:
                        failed_requests += 1
                        if error:
                            errors.append(f"{endpoint}: {error}")
                            
        self.monitor.stop_monitoring()
        end_time = datetime.utcnow()
        
        # Calculate statistics
        duration = (end_time - start_time).total_seconds()
        total_requests = len(response_times)
        
        return TestResult(
            test_name="API Endpoint Performance",
            start_time=start_time,
            end_time=end_time,
            duration=duration,
            total_requests=total_requests,
            successful_requests=successful_requests,
            failed_requests=failed_requests,
            avg_response_time=statistics.mean(response_times) if response_times else 0,
            min_response_time=min(response_times) if response_times else 0,
            max_response_time=max(response_times) if response_times else 0,
            p95_response_time=statistics.quantiles(response_times, n=20)[18] if len(response_times) >= 20 else 0,
            p99_response_time=statistics.quantiles(response_times, n=100)[98] if len(response_times) >= 100 else 0,
            requests_per_second=total_requests / duration if duration > 0 else 0,
            errors=errors,
            resource_usage=self.monitor.get_average_metrics()
        )
        
    async def load_test_scenario(self, concurrent_users: int, duration: int) -> LoadTestResult:
        """Run load test with specified concurrent users"""
        print(f"🔍 Running Load Test with {concurrent_users} concurrent users for {duration}s...")
        
        start_time = time.time()
        response_times = []
        errors = []
        successful_requests = 0
        failed_requests = 0
        
        # Test endpoints to simulate real usage
        endpoints = [
            f"{self.config['api_gateway_url']}/api/samples/v1/samples",
            f"{self.config['api_gateway_url']}/api/dashboard/v1/users",
            f"{self.config['api_gateway_url']}/api/sequencing/v1/jobs",
        ]
        
        self.monitor.start_monitoring()
        
        async def user_session():
            """Simulate a user session"""
            session_start = time.time()
            session_requests = 0
            
            async with aiohttp.ClientSession() as session:
                while time.time() - session_start < duration:
                    # Simulate user behavior - random endpoint selection
                    import random
                    endpoint = random.choice(endpoints)
                    
                    response_time, status_code, error = await self.single_request_test(session, endpoint)
                    response_times.append(response_time)
                    session_requests += 1
                    
                    if status_code == 200:
                        nonlocal successful_requests
                        successful_requests += 1
                    else:
                        nonlocal failed_requests
                        failed_requests += 1
                        if error:
                            errors.append(f"{endpoint}: {error}")
                    
                    # Think time between requests
                    await asyncio.sleep(self.config['think_time'])
                    
            return session_requests
            
        # Run concurrent user sessions
        tasks = [user_session() for _ in range(concurrent_users)]
        await asyncio.gather(*tasks)
        
        self.monitor.stop_monitoring()
        
        # Calculate results
        actual_duration = time.time() - start_time
        total_requests = len(response_times)
        avg_response_time = statistics.mean(response_times) if response_times else 0
        requests_per_second = total_requests / actual_duration if actual_duration > 0 else 0
        
        resource_metrics = self.monitor.get_average_metrics()
        
        return LoadTestResult(
            concurrent_users=concurrent_users,
            test_duration=actual_duration,
            total_requests=total_requests,
            successful_requests=successful_requests,
            failed_requests=failed_requests,
            avg_response_time=avg_response_time,
            requests_per_second=requests_per_second,
            cpu_usage=resource_metrics.get('avg_cpu_percent', 0),
            memory_usage=resource_metrics.get('avg_memory_percent', 0),
            errors=errors[:10]  # Limit error list
        )
        
    async def stress_test(self) -> TestResult:
        """Run stress test to find breaking point"""
        print("🔍 Running Stress Test...")
        
        start_time = datetime.utcnow()
        
        # Gradually increase load until failure
        stress_results = []
        max_users = 100
        step = 10
        
        for users in range(step, max_users + 1, step):
            print(f"  Testing with {users} concurrent users...")
            
            result = await self.load_test_scenario(users, 30)  # 30 second tests
            stress_results.append(result)
            
            # Stop if error rate exceeds 10%
            error_rate = result.failed_requests / result.total_requests if result.total_requests > 0 else 0
            if error_rate > 0.1:
                print(f"  Breaking point reached at {users} users (error rate: {error_rate:.2%})")
                break
                
        end_time = datetime.utcnow()
        
        # Aggregate results
        total_requests = sum(r.total_requests for r in stress_results)
        successful_requests = sum(r.successful_requests for r in stress_results)
        failed_requests = sum(r.failed_requests for r in stress_results)
        avg_response_time = statistics.mean(r.avg_response_time for r in stress_results)
        max_rps = max(r.requests_per_second for r in stress_results)
        
        all_errors = []
        for r in stress_results:
            all_errors.extend(r.errors)
            
        return TestResult(
            test_name="Stress Test",
            start_time=start_time,
            end_time=end_time,
            duration=(end_time - start_time).total_seconds(),
            total_requests=total_requests,
            successful_requests=successful_requests,
            failed_requests=failed_requests,
            avg_response_time=avg_response_time,
            min_response_time=0,
            max_response_time=0,
            p95_response_time=0,
            p99_response_time=0,
            requests_per_second=max_rps,
            errors=all_errors[:20],  # Limit error list
            resource_usage={}
        )
        
    async def integration_test(self) -> TestResult:
        """Test complex integration workflows"""
        print("🔍 Running Integration Test...")
        
        start_time = datetime.utcnow()
        response_times = []
        errors = []
        successful_requests = 0
        failed_requests = 0
        
        self.monitor.start_monitoring()
        
        async with aiohttp.ClientSession() as session:
            # Test complex workflow: Get samples -> Get users -> Get storage -> Create sample
            for i in range(20):  # 20 complete workflows
                workflow_start = time.time()
                
                # Step 1: Get samples
                response_time, status_code, error = await self.single_request_test(
                    session, f"{self.config['api_gateway_url']}/api/samples/v1/samples"
                )
                response_times.append(response_time)
                
                if status_code == 200:
                    successful_requests += 1
                else:
                    failed_requests += 1
                    errors.append(f"Get samples: {error}")
                    
                # Step 2: Get users
                response_time, status_code, error = await self.single_request_test(
                    session, f"{self.config['api_gateway_url']}/api/dashboard/v1/users"
                )
                response_times.append(response_time)
                
                if status_code == 200:
                    successful_requests += 1
                else:
                    failed_requests += 1
                    errors.append(f"Get users: {error}")
                    
                # Step 3: Get storage locations
                response_time, status_code, error = await self.single_request_test(
                    session, f"{self.config['api_gateway_url']}/api/dashboard/v1/storage/locations"
                )
                response_times.append(response_time)
                
                if status_code == 200:
                    successful_requests += 1
                else:
                    failed_requests += 1
                    errors.append(f"Get storage: {error}")
                    
                # Step 4: Create sample
                sample_data = {
                    "name": f"Integration Test Sample {i}",
                    "sample_type": "DNA",
                    "volume": 100.0,
                    "concentration": 50.0,
                    "storage_location": "Freezer A1",
                    "submitter_id": "test-user-1"
                }
                
                response_time, status_code, error = await self.single_request_test(
                    session, f"{self.config['api_gateway_url']}/api/samples/v1/samples", 
                    method='POST', data=sample_data
                )
                response_times.append(response_time)
                
                if status_code == 200:
                    successful_requests += 1
                else:
                    failed_requests += 1
                    errors.append(f"Create sample: {error}")
                    
                # Small delay between workflows
                await asyncio.sleep(0.1)
                
        self.monitor.stop_monitoring()
        end_time = datetime.utcnow()
        
        # Calculate statistics
        duration = (end_time - start_time).total_seconds()
        total_requests = len(response_times)
        
        return TestResult(
            test_name="Integration Test",
            start_time=start_time,
            end_time=end_time,
            duration=duration,
            total_requests=total_requests,
            successful_requests=successful_requests,
            failed_requests=failed_requests,
            avg_response_time=statistics.mean(response_times) if response_times else 0,
            min_response_time=min(response_times) if response_times else 0,
            max_response_time=max(response_times) if response_times else 0,
            p95_response_time=statistics.quantiles(response_times, n=20)[18] if len(response_times) >= 20 else 0,
            p99_response_time=statistics.quantiles(response_times, n=100)[98] if len(response_times) >= 100 else 0,
            requests_per_second=total_requests / duration if duration > 0 else 0,
            errors=errors,
            resource_usage=self.monitor.get_average_metrics()
        )
        
    def print_test_result(self, result: TestResult):
        """Print formatted test result"""
        print(f"\n{'='*60}")
        print(f"TEST RESULT: {result.test_name}")
        print(f"{'='*60}")
        print(f"Duration: {result.duration:.2f}s")
        print(f"Total Requests: {result.total_requests}")
        print(f"Successful: {result.successful_requests}")
        print(f"Failed: {result.failed_requests}")
        print(f"Success Rate: {(result.successful_requests/result.total_requests)*100:.1f}%")
        print(f"")
        print(f"Response Times:")
        print(f"  Average: {result.avg_response_time*1000:.2f}ms")
        print(f"  Minimum: {result.min_response_time*1000:.2f}ms")
        print(f"  Maximum: {result.max_response_time*1000:.2f}ms")
        print(f"  95th percentile: {result.p95_response_time*1000:.2f}ms")
        print(f"  99th percentile: {result.p99_response_time*1000:.2f}ms")
        print(f"")
        print(f"Throughput: {result.requests_per_second:.2f} requests/second")
        
        if result.resource_usage:
            print(f"")
            print(f"Resource Usage:")
            print(f"  Average CPU: {result.resource_usage.get('avg_cpu_percent', 0):.1f}%")
            print(f"  Average Memory: {result.resource_usage.get('avg_memory_percent', 0):.1f}%")
            print(f"  Max CPU: {result.resource_usage.get('max_cpu_percent', 0):.1f}%")
            print(f"  Max Memory: {result.resource_usage.get('max_memory_percent', 0):.1f}%")
            
        if result.errors:
            print(f"")
            print(f"Errors (first 10):")
            for error in result.errors[:10]:
                print(f"  - {error}")
                
    def print_load_test_result(self, result: LoadTestResult):
        """Print formatted load test result"""
        print(f"\n{'='*60}")
        print(f"LOAD TEST RESULT: {result.concurrent_users} Concurrent Users")
        print(f"{'='*60}")
        print(f"Duration: {result.test_duration:.2f}s")
        print(f"Total Requests: {result.total_requests}")
        print(f"Successful: {result.successful_requests}")
        print(f"Failed: {result.failed_requests}")
        print(f"Success Rate: {(result.successful_requests/result.total_requests)*100:.1f}%")
        print(f"Average Response Time: {result.avg_response_time*1000:.2f}ms")
        print(f"Requests/Second: {result.requests_per_second:.2f}")
        print(f"CPU Usage: {result.cpu_usage:.1f}%")
        print(f"Memory Usage: {result.memory_usage:.1f}%")
        
        if result.errors:
            print(f"Errors (first 5):")
            for error in result.errors[:5]:
                print(f"  - {error}")
                
    def save_results_to_file(self, results: List[TestResult], filename: str):
        """Save test results to JSON file"""
        results_data = []
        for result in results:
            results_data.append(asdict(result))
            
        with open(filename, 'w') as f:
            json.dump(results_data, f, indent=2, default=str)
            
        print(f"\n📊 Results saved to {filename}")
        
    async def run_all_tests(self):
        """Run all performance tests"""
        print("🚀 Starting TracSeq 2.0 Advanced Performance Testing")
        print("=" * 60)
        
        results = []
        
        # 1. Health Check Performance Test
        result = await self.health_check_performance_test()
        self.print_test_result(result)
        results.append(result)
        
        # 2. API Endpoint Performance Test
        result = await self.api_endpoint_performance_test()
        self.print_test_result(result)
        results.append(result)
        
        # 3. Load Tests
        print(f"\n🔍 Running Load Tests...")
        load_results = []
        for users in [1, 5, 10, 20]:
            result = await self.load_test_scenario(users, 30)
            self.print_load_test_result(result)
            load_results.append(result)
            
        # 4. Integration Test
        result = await self.integration_test()
        self.print_test_result(result)
        results.append(result)
        
        # 5. Stress Test
        result = await self.stress_test()
        self.print_test_result(result)
        results.append(result)
        
        # Save results
        timestamp = datetime.utcnow().strftime("%Y%m%d_%H%M%S")
        self.save_results_to_file(results, f"test_results_{timestamp}.json")
        
        # Print summary
        print(f"\n🎉 Performance Testing Complete!")
        print(f"Total Tests: {len(results)}")
        print(f"All results saved with timestamp: {timestamp}")
        
        return results

async def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description='TracSeq 2.0 Advanced Performance Testing')
    parser.add_argument('--test', choices=['health', 'api', 'load', 'integration', 'stress', 'all'], 
                       default='all', help='Test type to run')
    parser.add_argument('--users', type=int, default=10, help='Number of concurrent users for load test')
    parser.add_argument('--duration', type=int, default=30, help='Test duration in seconds')
    
    args = parser.parse_args()
    
    tester = PerformanceTester(TEST_CONFIG)
    
    if args.test == 'all':
        await tester.run_all_tests()
    elif args.test == 'health':
        result = await tester.health_check_performance_test()
        tester.print_test_result(result)
    elif args.test == 'api':
        result = await tester.api_endpoint_performance_test()
        tester.print_test_result(result)
    elif args.test == 'load':
        result = await tester.load_test_scenario(args.users, args.duration)
        tester.print_load_test_result(result)
    elif args.test == 'integration':
        result = await tester.integration_test()
        tester.print_test_result(result)
    elif args.test == 'stress':
        result = await tester.stress_test()
        tester.print_test_result(result)

if __name__ == "__main__":
    asyncio.run(main()) 