#!/usr/bin/env python3
"""
Test runner for the API Gateway modular architecture.

This script runs the test suite for the modular API Gateway components
without requiring pytest to be installed system-wide.
"""

import os
import sys
import subprocess
import unittest
from pathlib import Path

# Add the source directory to the Python path
current_dir = Path(__file__).parent
src_dir = current_dir / "src"
sys.path.insert(0, str(src_dir))

def run_basic_tests():
    """Run basic tests without pytest."""
    print("Running basic API Gateway tests...")
    
    # Test 1: Import all core modules
    print("\n1. Testing module imports...")
    try:
        from api_gateway.core.config import AppConfig, get_config
        from api_gateway.core.logging import setup_logging
        from api_gateway.core.exceptions import GatewayException
        from api_gateway.services.proxy import EnhancedProxyService, CircuitBreaker
        from api_gateway.middleware.cors import setup_cors
        from api_gateway.routes.finder import router as finder_router
        print("✓ All core modules imported successfully")
    except ImportError as e:
        print(f"✗ Import error: {e}")
        return False
    
    # Test 2: Configuration system
    print("\n2. Testing configuration system...")
    try:
        config = get_config()
        assert config.app_name == "TracSeq API Gateway"
        assert config.app_version == "2.0.0"
        assert config.environment in ["development", "staging", "production", "testing"]
        print("✓ Configuration system working")
    except Exception as e:
        print(f"✗ Configuration error: {e}")
        return False
    
    # Test 3: Circuit breaker functionality
    print("\n3. Testing circuit breaker...")
    try:
        from api_gateway.services.proxy import CircuitBreaker, CircuitState
        cb = CircuitBreaker("test-service", failure_threshold=3)
        assert cb.service_name == "test-service"
        assert cb.failure_threshold == 3
        assert cb.state == CircuitState.CLOSED
        print("✓ Circuit breaker initialization working")
    except Exception as e:
        print(f"✗ Circuit breaker error: {e}")
        return False
    
    # Test 4: Exception handling
    print("\n4. Testing exception handling...")
    try:
        from api_gateway.core.exceptions import (
            GatewayException, 
            ConfigurationException,
            DatabaseException,
            CircuitBreakerException
        )
        
        # Test custom exception creation
        exc = GatewayException("Test error", error_code="TEST_001")
        assert exc.message == "Test error"
        assert exc.error_code == "TEST_001"
        print("✓ Exception handling working")
    except Exception as e:
        print(f"✗ Exception handling error: {e}")
        return False
    
    # Test 5: Logging system
    print("\n5. Testing logging system...")
    try:
        from api_gateway.core.logging import setup_logging, get_logger
        
        # Setup logging
        setup_logging()
        
        # Get logger
        logger = get_logger("test")
        logger.info("Test log message")
        print("✓ Logging system working")
    except Exception as e:
        print(f"✗ Logging error: {e}")
        return False
    
    print("\n✓ All basic tests passed!")
    return True

def run_integration_tests():
    """Run integration tests."""
    print("\nRunning integration tests...")
    
    # Test 1: Application factory
    print("\n1. Testing application factory...")
    try:
        from api_gateway.main import create_app
        
        # Create app instance
        app = create_app()
        assert app is not None
        print("✓ Application factory working")
    except Exception as e:
        print(f"✗ Application factory error: {e}")
        return False
    
    # Test 2: Route registration
    print("\n2. Testing route registration...")
    try:
        from api_gateway.routes import register_routes
        from fastapi import FastAPI
        
        app = FastAPI()
        register_routes(app)
        
        # Check that routes are registered
        routes = [route.path for route in app.routes]
        assert any("/api/finder" in route for route in routes)
        print("✓ Route registration working")
    except Exception as e:
        print(f"✗ Route registration error: {e}")
        return False
    
    print("\n✓ All integration tests passed!")
    return True

def run_performance_tests():
    """Run basic performance tests."""
    print("\nRunning performance tests...")
    
    # Test 1: Configuration loading performance
    print("\n1. Testing configuration loading performance...")
    try:
        import time
        from api_gateway.core.config import reload_config
        
        start_time = time.time()
        for _ in range(100):
            reload_config()
        end_time = time.time()
        
        avg_time = (end_time - start_time) / 100
        assert avg_time < 0.01  # Should be under 10ms
        print(f"✓ Configuration loading: {avg_time:.4f}s average")
    except Exception as e:
        print(f"✗ Configuration performance error: {e}")
        return False
    
    # Test 2: Circuit breaker performance
    print("\n2. Testing circuit breaker performance...")
    try:
        import time
        import asyncio
        from api_gateway.services.proxy import CircuitBreaker
        
        async def test_circuit_breaker_performance():
            cb = CircuitBreaker("test-service", failure_threshold=5)
            
            async def success_func():
                return "success"
            
            start_time = time.time()
            for _ in range(1000):
                await cb.call(success_func)
            end_time = time.time()
            
            avg_time = (end_time - start_time) / 1000
            assert avg_time < 0.001  # Should be under 1ms
            return avg_time
        
        avg_time = asyncio.run(test_circuit_breaker_performance())
        print(f"✓ Circuit breaker calls: {avg_time:.6f}s average")
    except Exception as e:
        print(f"✗ Circuit breaker performance error: {e}")
        return False
    
    print("\n✓ All performance tests passed!")
    return True

def check_dependencies():
    """Check if all required dependencies are available."""
    print("Checking dependencies...")
    
    required_packages = [
        "fastapi",
        "uvicorn", 
        "pydantic",
        "httpx",
        "asyncpg",
        "python-multipart",
        "python-jose"
    ]
    
    missing_packages = []
    
    for package in required_packages:
        try:
            __import__(package.replace("-", "_"))
            print(f"✓ {package}")
        except ImportError:
            missing_packages.append(package)
            print(f"✗ {package} - Missing")
    
    if missing_packages:
        print(f"\nMissing packages: {', '.join(missing_packages)}")
        print("Install with: pip install " + " ".join(missing_packages))
        return False
    
    print("\n✓ All dependencies available!")
    return True

def main():
    """Main test runner."""
    print("=" * 60)
    print("API Gateway Modular Architecture Test Suite")
    print("=" * 60)
    
    # Check dependencies first
    if not check_dependencies():
        print("\n❌ Dependency check failed!")
        return 1
    
    # Run test suites
    test_results = []
    
    # Basic tests
    test_results.append(("Basic Tests", run_basic_tests()))
    
    # Integration tests
    test_results.append(("Integration Tests", run_integration_tests()))
    
    # Performance tests
    test_results.append(("Performance Tests", run_performance_tests()))
    
    # Summary
    print("\n" + "=" * 60)
    print("TEST SUMMARY")
    print("=" * 60)
    
    passed = 0
    failed = 0
    
    for test_name, result in test_results:
        if result:
            print(f"✓ {test_name}: PASSED")
            passed += 1
        else:
            print(f"✗ {test_name}: FAILED")
            failed += 1
    
    print(f"\nTotal: {passed + failed} test suites")
    print(f"Passed: {passed}")
    print(f"Failed: {failed}")
    
    if failed == 0:
        print("\n🎉 All tests passed!")
        return 0
    else:
        print(f"\n❌ {failed} test suite(s) failed!")
        return 1

if __name__ == "__main__":
    sys.exit(main())