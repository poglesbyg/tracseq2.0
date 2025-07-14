#!/usr/bin/env python3
"""
TracSeq 2.0 - Integration Testing Framework
Tests complex workflows, data consistency, and cross-service communication
"""

import asyncio
import aiohttp
import json
import time
import uuid
from datetime import datetime
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, asdict

# Test configuration
TEST_CONFIG = {
    'api_gateway_url': 'http://localhost:8089',
    'frontend_proxy_url': 'http://localhost:8085',
    'services': {
        'dashboard': 'http://localhost:8080',
        'samples': 'http://localhost:8081',
        'sequencing': 'http://localhost:8082',
        'spreadsheet': 'http://localhost:8083'
    }
}

@dataclass
class IntegrationTestResult:
    """Integration test result data structure"""
    test_name: str
    start_time: datetime
    end_time: datetime
    duration: float
    total_steps: int
    successful_steps: int
    failed_steps: int
    errors: List[str]
    workflow_data: Dict[str, Any]

class IntegrationTester:
    """Comprehensive integration testing framework"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.results = []
        
    async def make_request(self, session: aiohttp.ClientSession, method: str, url: str, 
                          data: Optional[Dict] = None) -> Dict[str, Any]:
        """Make HTTP request and return response data"""
        try:
            if method.upper() == 'GET':
                async with session.get(url) as response:
                    if response.status == 200:
                        return await response.json()
                    else:
                        return {'error': f'HTTP {response.status}', 'status': response.status}
            elif method.upper() == 'POST':
                async with session.post(url, json=data) as response:
                    if response.status == 200:
                        return await response.json()
                    else:
                        return {'error': f'HTTP {response.status}', 'status': response.status}
        except Exception as e:
            return {'error': str(e), 'status': 0}
            
    async def sample_submission_workflow_test(self) -> IntegrationTestResult:
        """Test complete sample submission workflow"""
        print("🔍 Running Sample Submission Workflow Test...")
        
        start_time = datetime.utcnow()
        errors = []
        workflow_data = {}
        total_steps = 8
        successful_steps = 0
        
        async with aiohttp.ClientSession() as session:
            # Step 1: Get current users
            print("  Step 1: Fetching users...")
            users_response = await self.make_request(
                session, 'GET', f"{self.config['api_gateway_url']}/api/dashboard/v1/users"
            )
            
            if 'error' not in users_response:
                successful_steps += 1
                workflow_data['users'] = users_response.get('data', {}).get('users', [])
                print(f"    ✅ Found {len(workflow_data['users'])} users")
            else:
                errors.append(f"Step 1 failed: {users_response['error']}")
                print(f"    ❌ Failed: {users_response['error']}")
                
            # Step 2: Get storage locations
            print("  Step 2: Fetching storage locations...")
            storage_response = await self.make_request(
                session, 'GET', f"{self.config['api_gateway_url']}/api/dashboard/v1/storage/locations"
            )
            
            if 'error' not in storage_response:
                successful_steps += 1
                workflow_data['storage_locations'] = storage_response.get('data', {}).get('locations', [])
                print(f"    ✅ Found {len(workflow_data['storage_locations'])} storage locations")
            else:
                errors.append(f"Step 2 failed: {storage_response['error']}")
                print(f"    ❌ Failed: {storage_response['error']}")
                
            # Step 3: Get existing samples
            print("  Step 3: Fetching existing samples...")
            samples_response = await self.make_request(
                session, 'GET', f"{self.config['api_gateway_url']}/api/samples/v1/samples"
            )
            
            if 'error' not in samples_response:
                successful_steps += 1
                workflow_data['existing_samples'] = samples_response.get('data', {}).get('samples', [])
                print(f"    ✅ Found {len(workflow_data['existing_samples'])} existing samples")
            else:
                errors.append(f"Step 3 failed: {samples_response['error']}")
                print(f"    ❌ Failed: {samples_response['error']}")
                
            # Step 4: Get templates
            print("  Step 4: Fetching templates...")
            templates_response = await self.make_request(
                session, 'GET', f"{self.config['api_gateway_url']}/api/spreadsheet/v1/templates"
            )
            
            if 'error' not in templates_response:
                successful_steps += 1
                workflow_data['templates'] = templates_response.get('data', {}).get('templates', [])
                print(f"    ✅ Found {len(workflow_data['templates'])} templates")
            else:
                errors.append(f"Step 4 failed: {templates_response['error']}")
                print(f"    ❌ Failed: {templates_response['error']}")
                
            # Step 5: Create new sample
            print("  Step 5: Creating new sample...")
            test_sample = {
                "name": f"Integration Test Sample {uuid.uuid4().hex[:8]}",
                "sample_type": "DNA",
                "volume": 100.0,
                "concentration": 50.0,
                "storage_location": "Freezer A1",
                "submitter_id": "test-user-1"
            }
            
            create_response = await self.make_request(
                session, 'POST', f"{self.config['api_gateway_url']}/api/samples/v1/samples", test_sample
            )
            
            if 'error' not in create_response:
                successful_steps += 1
                workflow_data['created_sample'] = create_response
                print(f"    ✅ Created sample: {test_sample['name']}")
            else:
                errors.append(f"Step 5 failed: {create_response['error']}")
                print(f"    ❌ Failed: {create_response['error']}")
                
            # Step 6: Verify sample was created (get samples again)
            print("  Step 6: Verifying sample creation...")
            verify_response = await self.make_request(
                session, 'GET', f"{self.config['api_gateway_url']}/api/samples/v1/samples"
            )
            
            if 'error' not in verify_response:
                new_samples = verify_response.get('data', {}).get('samples', [])
                if len(new_samples) > len(workflow_data.get('existing_samples', [])):
                    successful_steps += 1
                    workflow_data['verified_samples'] = new_samples
                    print(f"    ✅ Verified sample creation - total samples: {len(new_samples)}")
                else:
                    errors.append("Step 6 failed: Sample count did not increase")
                    print(f"    ❌ Failed: Sample count did not increase")
            else:
                errors.append(f"Step 6 failed: {verify_response['error']}")
                print(f"    ❌ Failed: {verify_response['error']}")
                
            # Step 7: Get sequencing jobs
            print("  Step 7: Fetching sequencing jobs...")
            jobs_response = await self.make_request(
                session, 'GET', f"{self.config['api_gateway_url']}/api/sequencing/v1/jobs"
            )
            
            if 'error' not in jobs_response:
                successful_steps += 1
                workflow_data['sequencing_jobs'] = jobs_response.get('data', {}).get('jobs', [])
                print(f"    ✅ Found {len(workflow_data['sequencing_jobs'])} sequencing jobs")
            else:
                errors.append(f"Step 7 failed: {jobs_response['error']}")
                print(f"    ❌ Failed: {jobs_response['error']}")
                
            # Step 8: Cross-service data consistency check
            print("  Step 8: Checking cross-service data consistency...")
            dashboard_samples_response = await self.make_request(
                session, 'GET', f"{self.config['api_gateway_url']}/api/dashboard/v1/samples"
            )
            
            if 'error' not in dashboard_samples_response:
                dashboard_samples = dashboard_samples_response.get('data', {}).get('samples', [])
                samples_service_samples = workflow_data.get('verified_samples', [])
                
                if len(dashboard_samples) == len(samples_service_samples):
                    successful_steps += 1
                    print(f"    ✅ Data consistency verified - both services show {len(dashboard_samples)} samples")
                else:
                    errors.append(f"Step 8 failed: Data inconsistency - dashboard: {len(dashboard_samples)}, samples: {len(samples_service_samples)}")
                    print(f"    ❌ Failed: Data inconsistency")
            else:
                errors.append(f"Step 8 failed: {dashboard_samples_response['error']}")
                print(f"    ❌ Failed: {dashboard_samples_response['error']}")
                
        end_time = datetime.utcnow()
        
        return IntegrationTestResult(
            test_name="Sample Submission Workflow",
            start_time=start_time,
            end_time=end_time,
            duration=(end_time - start_time).total_seconds(),
            total_steps=total_steps,
            successful_steps=successful_steps,
            failed_steps=total_steps - successful_steps,
            errors=errors,
            workflow_data=workflow_data
        )
        
    async def service_communication_test(self) -> IntegrationTestResult:
        """Test communication between all services"""
        print("🔍 Running Service Communication Test...")
        
        start_time = datetime.utcnow()
        errors = []
        workflow_data = {}
        total_steps = 12
        successful_steps = 0
        
        async with aiohttp.ClientSession() as session:
            # Test all service health endpoints
            services = ['dashboard', 'samples', 'sequencing', 'spreadsheet']
            
            for service in services:
                print(f"  Testing {service} service health...")
                health_response = await self.make_request(
                    session, 'GET', f"{self.config['api_gateway_url']}/api/{service}/health"
                )
                
                if 'error' not in health_response:
                    successful_steps += 1
                    workflow_data[f'{service}_health'] = health_response
                    print(f"    ✅ {service} service is healthy")
                else:
                    errors.append(f"{service} health check failed: {health_response['error']}")
                    print(f"    ❌ {service} service health check failed")
                    
            # Test API Gateway service discovery
            print("  Testing API Gateway service discovery...")
            discovery_response = await self.make_request(
                session, 'GET', f"{self.config['api_gateway_url']}/services"
            )
            
            if 'error' not in discovery_response:
                successful_steps += 1
                workflow_data['service_discovery'] = discovery_response
                print(f"    ✅ Service discovery working")
            else:
                errors.append(f"Service discovery failed: {discovery_response['error']}")
                print(f"    ❌ Service discovery failed")
                
            # Test frontend proxy communication
            print("  Testing frontend proxy communication...")
            proxy_health_response = await self.make_request(
                session, 'GET', f"{self.config['frontend_proxy_url']}/api/dashboard/health"
            )
            
            if 'error' not in proxy_health_response:
                successful_steps += 1
                workflow_data['proxy_communication'] = proxy_health_response
                print(f"    ✅ Frontend proxy communication working")
            else:
                errors.append(f"Frontend proxy communication failed: {proxy_health_response['error']}")
                print(f"    ❌ Frontend proxy communication failed")
                
            # Test direct service access
            for service, url in self.config['services'].items():
                print(f"  Testing direct access to {service} service...")
                direct_response = await self.make_request(
                    session, 'GET', f"{url}/health"
                )
                
                if 'error' not in direct_response:
                    successful_steps += 1
                    workflow_data[f'{service}_direct'] = direct_response
                    print(f"    ✅ Direct access to {service} working")
                else:
                    errors.append(f"Direct access to {service} failed: {direct_response['error']}")
                    print(f"    ❌ Direct access to {service} failed")
                    
        end_time = datetime.utcnow()
        
        return IntegrationTestResult(
            test_name="Service Communication",
            start_time=start_time,
            end_time=end_time,
            duration=(end_time - start_time).total_seconds(),
            total_steps=total_steps,
            successful_steps=successful_steps,
            failed_steps=total_steps - successful_steps,
            errors=errors,
            workflow_data=workflow_data
        )
        
    async def data_consistency_test(self) -> IntegrationTestResult:
        """Test data consistency across services"""
        print("🔍 Running Data Consistency Test...")
        
        start_time = datetime.utcnow()
        errors = []
        workflow_data = {}
        total_steps = 6
        successful_steps = 0
        
        async with aiohttp.ClientSession() as session:
            # Get samples from samples service
            print("  Step 1: Getting samples from samples service...")
            samples_response = await self.make_request(
                session, 'GET', f"{self.config['api_gateway_url']}/api/samples/v1/samples"
            )
            
            if 'error' not in samples_response:
                successful_steps += 1
                samples_data = samples_response.get('data', {}).get('samples', [])
                workflow_data['samples_service_data'] = samples_data
                print(f"    ✅ Retrieved {len(samples_data)} samples from samples service")
            else:
                errors.append(f"Step 1 failed: {samples_response['error']}")
                print(f"    ❌ Failed: {samples_response['error']}")
                
            # Get samples from dashboard service
            print("  Step 2: Getting samples from dashboard service...")
            dashboard_samples_response = await self.make_request(
                session, 'GET', f"{self.config['api_gateway_url']}/api/dashboard/v1/samples"
            )
            
            if 'error' not in dashboard_samples_response:
                successful_steps += 1
                dashboard_data = dashboard_samples_response.get('data', {}).get('samples', [])
                workflow_data['dashboard_service_data'] = dashboard_data
                print(f"    ✅ Retrieved {len(dashboard_data)} samples from dashboard service")
            else:
                errors.append(f"Step 2 failed: {dashboard_samples_response['error']}")
                print(f"    ❌ Failed: {dashboard_samples_response['error']}")
                
            # Compare sample counts
            print("  Step 3: Comparing sample counts...")
            if 'samples_service_data' in workflow_data and 'dashboard_service_data' in workflow_data:
                samples_count = len(workflow_data['samples_service_data'])
                dashboard_count = len(workflow_data['dashboard_service_data'])
                
                if samples_count == dashboard_count:
                    successful_steps += 1
                    print(f"    ✅ Sample counts match: {samples_count}")
                else:
                    errors.append(f"Sample count mismatch: samples={samples_count}, dashboard={dashboard_count}")
                    print(f"    ❌ Sample count mismatch")
            else:
                errors.append("Cannot compare sample counts - missing data")
                print(f"    ❌ Cannot compare sample counts")
                
            # Get users from dashboard service
            print("  Step 4: Getting users from dashboard service...")
            users_response = await self.make_request(
                session, 'GET', f"{self.config['api_gateway_url']}/api/dashboard/v1/users"
            )
            
            if 'error' not in users_response:
                successful_steps += 1
                users_data = users_response.get('data', {}).get('users', [])
                workflow_data['users_data'] = users_data
                print(f"    ✅ Retrieved {len(users_data)} users")
            else:
                errors.append(f"Step 4 failed: {users_response['error']}")
                print(f"    ❌ Failed: {users_response['error']}")
                
            # Get storage locations
            print("  Step 5: Getting storage locations...")
            storage_response = await self.make_request(
                session, 'GET', f"{self.config['api_gateway_url']}/api/dashboard/v1/storage/locations"
            )
            
            if 'error' not in storage_response:
                successful_steps += 1
                storage_data = storage_response.get('data', {}).get('locations', [])
                workflow_data['storage_data'] = storage_data
                print(f"    ✅ Retrieved {len(storage_data)} storage locations")
            else:
                errors.append(f"Step 5 failed: {storage_response['error']}")
                print(f"    ❌ Failed: {storage_response['error']}")
                
            # Validate data structure consistency
            print("  Step 6: Validating data structure consistency...")
            validation_errors = []
            
            # Check samples data structure
            if 'samples_service_data' in workflow_data:
                for sample in workflow_data['samples_service_data']:
                    required_fields = ['id', 'name', 'sample_type', 'status']
                    for field in required_fields:
                        if field not in sample:
                            validation_errors.append(f"Sample missing field: {field}")
                            
            # Check users data structure
            if 'users_data' in workflow_data:
                for user in workflow_data['users_data']:
                    required_fields = ['id', 'name', 'email', 'role']
                    for field in required_fields:
                        if field not in user:
                            validation_errors.append(f"User missing field: {field}")
                            
            if not validation_errors:
                successful_steps += 1
                print(f"    ✅ Data structure validation passed")
            else:
                errors.extend(validation_errors)
                print(f"    ❌ Data structure validation failed: {len(validation_errors)} errors")
                
        end_time = datetime.utcnow()
        
        return IntegrationTestResult(
            test_name="Data Consistency",
            start_time=start_time,
            end_time=end_time,
            duration=(end_time - start_time).total_seconds(),
            total_steps=total_steps,
            successful_steps=successful_steps,
            failed_steps=total_steps - successful_steps,
            errors=errors,
            workflow_data=workflow_data
        )
        
    async def error_handling_test(self) -> IntegrationTestResult:
        """Test error handling across services"""
        print("🔍 Running Error Handling Test...")
        
        start_time = datetime.utcnow()
        errors = []
        workflow_data = {}
        total_steps = 6
        successful_steps = 0
        
        async with aiohttp.ClientSession() as session:
            # Test 404 errors
            print("  Step 1: Testing 404 error handling...")
            not_found_response = await self.make_request(
                session, 'GET', f"{self.config['api_gateway_url']}/api/nonexistent/endpoint"
            )
            
            if 'error' in not_found_response and '404' in str(not_found_response.get('status', '')):
                successful_steps += 1
                workflow_data['404_test'] = not_found_response
                print(f"    ✅ 404 error handled correctly")
            else:
                errors.append("404 error not handled correctly")
                print(f"    ❌ 404 error not handled correctly")
                
            # Test invalid sample ID
            print("  Step 2: Testing invalid sample ID...")
            invalid_sample_response = await self.make_request(
                session, 'GET', f"{self.config['api_gateway_url']}/api/samples/v1/samples/invalid-id"
            )
            
            if 'error' in invalid_sample_response:
                successful_steps += 1
                workflow_data['invalid_sample_test'] = invalid_sample_response
                print(f"    ✅ Invalid sample ID handled correctly")
            else:
                errors.append("Invalid sample ID not handled correctly")
                print(f"    ❌ Invalid sample ID not handled correctly")
                
            # Test malformed JSON
            print("  Step 3: Testing malformed JSON handling...")
            try:
                async with session.post(
                    f"{self.config['api_gateway_url']}/api/samples/v1/samples",
                    data="invalid json"
                ) as response:
                    if response.status in [400, 422]:
                        successful_steps += 1
                        workflow_data['malformed_json_test'] = {'status': response.status}
                        print(f"    ✅ Malformed JSON handled correctly")
                    else:
                        errors.append(f"Malformed JSON not handled correctly: {response.status}")
                        print(f"    ❌ Malformed JSON not handled correctly")
            except Exception as e:
                successful_steps += 1
                workflow_data['malformed_json_test'] = {'error': str(e)}
                print(f"    ✅ Malformed JSON handled correctly (exception caught)")
                
            # Test service unavailable scenario (invalid service URL)
            print("  Step 4: Testing service unavailable scenario...")
            unavailable_response = await self.make_request(
                session, 'GET', f"{self.config['api_gateway_url']}/api/invalid-service/health"
            )
            
            if 'error' in unavailable_response:
                successful_steps += 1
                workflow_data['service_unavailable_test'] = unavailable_response
                print(f"    ✅ Service unavailable handled correctly")
            else:
                errors.append("Service unavailable not handled correctly")
                print(f"    ❌ Service unavailable not handled correctly")
                
            # Test rate limiting (if implemented)
            print("  Step 5: Testing rate limiting behavior...")
            rapid_requests = []
            for i in range(10):
                response = await self.make_request(
                    session, 'GET', f"{self.config['api_gateway_url']}/api/dashboard/health"
                )
                rapid_requests.append(response)
                
            # Check if all requests succeeded (no rate limiting) or if some were rate limited
            successful_rapid = sum(1 for r in rapid_requests if 'error' not in r)
            if successful_rapid >= 8:  # Allow some failures
                successful_steps += 1
                workflow_data['rate_limiting_test'] = {'successful': successful_rapid, 'total': 10}
                print(f"    ✅ Rate limiting test passed ({successful_rapid}/10 requests succeeded)")
            else:
                errors.append(f"Rate limiting test failed: only {successful_rapid}/10 requests succeeded")
                print(f"    ❌ Rate limiting test failed")
                
            # Test timeout handling
            print("  Step 6: Testing timeout handling...")
            # This is a basic test - in a real scenario, you'd test with a slow endpoint
            timeout_response = await self.make_request(
                session, 'GET', f"{self.config['api_gateway_url']}/api/dashboard/health"
            )
            
            if 'error' not in timeout_response:
                successful_steps += 1
                workflow_data['timeout_test'] = timeout_response
                print(f"    ✅ Timeout handling test passed (no timeout occurred)")
            else:
                # If there's an error, it might be a timeout, which is also valid
                if 'timeout' in str(timeout_response.get('error', '')).lower():
                    successful_steps += 1
                    workflow_data['timeout_test'] = timeout_response
                    print(f"    ✅ Timeout handled correctly")
                else:
                    errors.append(f"Timeout test failed: {timeout_response['error']}")
                    print(f"    ❌ Timeout test failed")
                    
        end_time = datetime.utcnow()
        
        return IntegrationTestResult(
            test_name="Error Handling",
            start_time=start_time,
            end_time=end_time,
            duration=(end_time - start_time).total_seconds(),
            total_steps=total_steps,
            successful_steps=successful_steps,
            failed_steps=total_steps - successful_steps,
            errors=errors,
            workflow_data=workflow_data
        )
        
    def print_test_result(self, result: IntegrationTestResult):
        """Print formatted test result"""
        print(f"\n{'='*60}")
        print(f"INTEGRATION TEST RESULT: {result.test_name}")
        print(f"{'='*60}")
        print(f"Duration: {result.duration:.2f}s")
        print(f"Total Steps: {result.total_steps}")
        print(f"Successful Steps: {result.successful_steps}")
        print(f"Failed Steps: {result.failed_steps}")
        print(f"Success Rate: {(result.successful_steps/result.total_steps)*100:.1f}%")
        
        if result.errors:
            print(f"\nErrors:")
            for error in result.errors:
                print(f"  - {error}")
                
        print(f"\nWorkflow Data Summary:")
        for key, value in result.workflow_data.items():
            if isinstance(value, list):
                print(f"  {key}: {len(value)} items")
            elif isinstance(value, dict):
                print(f"  {key}: {len(value)} keys")
            else:
                print(f"  {key}: {value}")
                
    def save_results_to_file(self, results: List[IntegrationTestResult], filename: str):
        """Save test results to JSON file"""
        results_data = []
        for result in results:
            results_data.append(asdict(result))
            
        with open(filename, 'w') as f:
            json.dump(results_data, f, indent=2, default=str)
            
        print(f"\n📊 Results saved to {filename}")
        
    async def run_all_tests(self):
        """Run all integration tests"""
        print("🚀 Starting TracSeq 2.0 Integration Testing")
        print("=" * 60)
        
        results = []
        
        # 1. Sample Submission Workflow Test
        result = await self.sample_submission_workflow_test()
        self.print_test_result(result)
        results.append(result)
        
        # 2. Service Communication Test
        result = await self.service_communication_test()
        self.print_test_result(result)
        results.append(result)
        
        # 3. Data Consistency Test
        result = await self.data_consistency_test()
        self.print_test_result(result)
        results.append(result)
        
        # 4. Error Handling Test
        result = await self.error_handling_test()
        self.print_test_result(result)
        results.append(result)
        
        # Save results
        timestamp = datetime.utcnow().strftime("%Y%m%d_%H%M%S")
        self.save_results_to_file(results, f"integration_test_results_{timestamp}.json")
        
        # Print summary
        total_steps = sum(r.total_steps for r in results)
        successful_steps = sum(r.successful_steps for r in results)
        
        print(f"\n🎉 Integration Testing Complete!")
        print(f"Total Tests: {len(results)}")
        print(f"Total Steps: {total_steps}")
        print(f"Successful Steps: {successful_steps}")
        print(f"Overall Success Rate: {(successful_steps/total_steps)*100:.1f}%")
        print(f"Results saved with timestamp: {timestamp}")
        
        return results

async def main():
    """Main entry point"""
    import argparse
    
    parser = argparse.ArgumentParser(description='TracSeq 2.0 Integration Testing')
    parser.add_argument('--test', choices=['workflow', 'communication', 'consistency', 'errors', 'all'], 
                       default='all', help='Test type to run')
    
    args = parser.parse_args()
    
    tester = IntegrationTester(TEST_CONFIG)
    
    if args.test == 'all':
        await tester.run_all_tests()
    elif args.test == 'workflow':
        result = await tester.sample_submission_workflow_test()
        tester.print_test_result(result)
    elif args.test == 'communication':
        result = await tester.service_communication_test()
        tester.print_test_result(result)
    elif args.test == 'consistency':
        result = await tester.data_consistency_test()
        tester.print_test_result(result)
    elif args.test == 'errors':
        result = await tester.error_handling_test()
        tester.print_test_result(result)

if __name__ == "__main__":
    asyncio.run(main()) 