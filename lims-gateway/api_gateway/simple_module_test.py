#!/usr/bin/env python3
"""
Simple module test for the API Gateway modular architecture.

This script validates the modular architecture without requiring external dependencies.
"""

import os
import sys
import importlib.util
from pathlib import Path

# Add the source directory to the Python path
current_dir = Path(__file__).parent
src_dir = current_dir / "src"
sys.path.insert(0, str(src_dir))

def test_module_structure():
    """Test that all expected modules exist and can be imported."""
    print("Testing modular architecture structure...")
    
    # Expected modules
    expected_modules = [
        "api_gateway.core.config",
        "api_gateway.core.logging", 
        "api_gateway.core.exceptions",
        "api_gateway.core.database",
        "api_gateway.core.health_checks",
        "api_gateway.core.monitoring",
        "api_gateway.core.rate_limiter",
        "api_gateway.core.circuit_breaker",
        "api_gateway.services.proxy",
        "api_gateway.middleware.cors",
        "api_gateway.routes.finder",
        "api_gateway.routes",
        "api_gateway.utils.helpers",
        "api_gateway.models"
    ]
    
    success_count = 0
    total_count = len(expected_modules)
    
    for module_name in expected_modules:
        try:
            module = importlib.import_module(module_name)
            print(f"✓ {module_name}")
            success_count += 1
        except ImportError as e:
            print(f"✗ {module_name}: {e}")
        except Exception as e:
            print(f"? {module_name}: {e}")
    
    print(f"\nModule Import Results: {success_count}/{total_count} modules imported successfully")
    return success_count, total_count

def test_core_classes():
    """Test that core classes can be instantiated."""
    print("\nTesting core class instantiation...")
    
    tests = []
    
    # Test 1: Configuration classes
    try:
        from api_gateway.core.config import AppConfig, DatabaseConfig, SecurityConfig
        
        # Test basic config creation
        db_config = DatabaseConfig()
        assert hasattr(db_config, 'url')
        assert hasattr(db_config, 'pool_min_size')
        
        security_config = SecurityConfig()
        assert hasattr(security_config, 'jwt_secret_key')
        assert hasattr(security_config, 'jwt_algorithm')
        
        app_config = AppConfig()
        assert hasattr(app_config, 'app_name')
        assert hasattr(app_config, 'environment')
        
        print("✓ Configuration classes working")
        tests.append(("Configuration", True))
    except Exception as e:
        print(f"✗ Configuration classes: {e}")
        tests.append(("Configuration", False))
    
    # Test 2: Exception classes
    try:
        from api_gateway.core.exceptions import (
            GatewayException, 
            ConfigurationException,
            DatabaseException,
            CircuitBreakerException
        )
        
        # Test exception creation
        exc = GatewayException("Test error", error_code="TEST_001")
        assert hasattr(exc, 'message')
        assert hasattr(exc, 'error_code')
        assert exc.message == "Test error"
        assert exc.error_code == "TEST_001"
        
        print("✓ Exception classes working")
        tests.append(("Exceptions", True))
    except Exception as e:
        print(f"✗ Exception classes: {e}")
        tests.append(("Exceptions", False))
    
    # Test 3: Circuit breaker
    try:
        from api_gateway.core.circuit_breaker import CircuitBreaker, CircuitState
        
        cb = CircuitBreaker("test-service", failure_threshold=3)
        assert hasattr(cb, 'service_name')
        assert hasattr(cb, 'failure_threshold')
        assert hasattr(cb, 'state')
        assert cb.service_name == "test-service"
        assert cb.failure_threshold == 3
        
        print("✓ Circuit breaker working")
        tests.append(("Circuit Breaker", True))
    except Exception as e:
        print(f"✗ Circuit breaker: {e}")
        tests.append(("Circuit Breaker", False))
    
    # Test 4: Rate limiter
    try:
        from api_gateway.core.rate_limiter import RateLimiter, RateLimitConfig
        
        config = RateLimitConfig(requests_per_minute=100)
        rate_limiter = RateLimiter(config)
        assert hasattr(rate_limiter, 'config')
        assert hasattr(rate_limiter, 'requests')
        
        print("✓ Rate limiter working")
        tests.append(("Rate Limiter", True))
    except Exception as e:
        print(f"✗ Rate limiter: {e}")
        tests.append(("Rate Limiter", False))
    
    # Test 5: Health checks
    try:
        from api_gateway.core.health_checks import HealthChecker, HealthStatus
        
        health_checker = HealthChecker()
        assert hasattr(health_checker, 'checks')
        
        print("✓ Health checks working")
        tests.append(("Health Checks", True))
    except Exception as e:
        print(f"✗ Health checks: {e}")
        tests.append(("Health Checks", False))
    
    # Test 6: Monitoring
    try:
        from api_gateway.core.monitoring import MetricsCollector, Metric
        
        metrics = MetricsCollector()
        assert hasattr(metrics, 'metrics')
        
        print("✓ Monitoring working")
        tests.append(("Monitoring", True))
    except Exception as e:
        print(f"✗ Monitoring: {e}")
        tests.append(("Monitoring", False))
    
    success_count = sum(1 for _, success in tests if success)
    total_count = len(tests)
    
    print(f"\nCore Class Results: {success_count}/{total_count} classes working")
    return success_count, total_count

def test_file_structure():
    """Test that all expected files exist."""
    print("\nTesting file structure...")
    
    src_dir = Path(__file__).parent / "src" / "api_gateway"
    
    expected_files = [
        "core/__init__.py",
        "core/config.py",
        "core/logging.py",
        "core/exceptions.py",
        "core/database.py",
        "core/health_checks.py",
        "core/monitoring.py",
        "core/rate_limiter.py",
        "core/circuit_breaker.py",
        "services/__init__.py",
        "services/proxy.py",
        "middleware/__init__.py",
        "middleware/cors.py",
        "routes/__init__.py",
        "routes/finder.py",
        "utils/__init__.py",
        "utils/helpers.py",
        "models.py",
        "main.py",
        "__init__.py"
    ]
    
    success_count = 0
    total_count = len(expected_files)
    
    for file_path in expected_files:
        full_path = src_dir / file_path
        if full_path.exists():
            print(f"✓ {file_path}")
            success_count += 1
        else:
            print(f"✗ {file_path} - Missing")
    
    print(f"\nFile Structure Results: {success_count}/{total_count} files exist")
    return success_count, total_count

def test_code_quality():
    """Test basic code quality metrics."""
    print("\nTesting code quality...")
    
    src_dir = Path(__file__).parent / "src" / "api_gateway"
    
    tests = []
    
    # Test 1: No empty Python files (except __init__.py)
    try:
        empty_files = []
        for py_file in src_dir.rglob("*.py"):
            if py_file.name != "__init__.py" and py_file.stat().st_size == 0:
                empty_files.append(str(py_file.relative_to(src_dir)))
        
        if empty_files:
            print(f"✗ Empty files found: {', '.join(empty_files)}")
            tests.append(("No Empty Files", False))
        else:
            print("✓ No empty files")
            tests.append(("No Empty Files", True))
    except Exception as e:
        print(f"✗ Empty file check: {e}")
        tests.append(("No Empty Files", False))
    
    # Test 2: All directories have __init__.py
    try:
        missing_init = []
        for dir_path in src_dir.rglob("*"):
            if dir_path.is_dir() and dir_path.name != "__pycache__":
                init_file = dir_path / "__init__.py"
                if not init_file.exists():
                    missing_init.append(str(dir_path.relative_to(src_dir)))
        
        if missing_init:
            print(f"✗ Missing __init__.py: {', '.join(missing_init)}")
            tests.append(("Init Files", False))
        else:
            print("✓ All directories have __init__.py")
            tests.append(("Init Files", True))
    except Exception as e:
        print(f"✗ Init file check: {e}")
        tests.append(("Init Files", False))
    
    # Test 3: Reasonable file sizes
    try:
        large_files = []
        for py_file in src_dir.rglob("*.py"):
            size_kb = py_file.stat().st_size / 1024
            if size_kb > 100:  # Files larger than 100KB
                large_files.append(f"{py_file.relative_to(src_dir)} ({size_kb:.1f}KB)")
        
        if large_files:
            print(f"⚠ Large files (>100KB): {', '.join(large_files)}")
            tests.append(("File Sizes", True))  # Warning, not failure
        else:
            print("✓ All files are reasonable size")
            tests.append(("File Sizes", True))
    except Exception as e:
        print(f"✗ File size check: {e}")
        tests.append(("File Sizes", False))
    
    success_count = sum(1 for _, success in tests if success)
    total_count = len(tests)
    
    print(f"\nCode Quality Results: {success_count}/{total_count} checks passed")
    return success_count, total_count

def main():
    """Main test runner."""
    print("=" * 60)
    print("API Gateway Modular Architecture Validation")
    print("=" * 60)
    
    # Run all tests
    test_results = []
    
    # File structure test
    file_success, file_total = test_file_structure()
    test_results.append(("File Structure", file_success, file_total))
    
    # Module import test
    module_success, module_total = test_module_structure()
    test_results.append(("Module Imports", module_success, module_total))
    
    # Core class test
    class_success, class_total = test_core_classes()
    test_results.append(("Core Classes", class_success, class_total))
    
    # Code quality test
    quality_success, quality_total = test_code_quality()
    test_results.append(("Code Quality", quality_success, quality_total))
    
    # Summary
    print("\n" + "=" * 60)
    print("ARCHITECTURE VALIDATION SUMMARY")
    print("=" * 60)
    
    overall_success = 0
    overall_total = 0
    
    for test_name, success, total in test_results:
        percentage = (success / total * 100) if total > 0 else 0
        status = "✓" if percentage >= 80 else "⚠" if percentage >= 60 else "✗"
        print(f"{status} {test_name}: {success}/{total} ({percentage:.1f}%)")
        overall_success += success
        overall_total += total
    
    overall_percentage = (overall_success / overall_total * 100) if overall_total > 0 else 0
    
    print(f"\nOverall: {overall_success}/{overall_total} ({overall_percentage:.1f}%)")
    
    if overall_percentage >= 80:
        print("\n🎉 Modular architecture is well-structured!")
        return 0
    elif overall_percentage >= 60:
        print("\n⚠ Modular architecture needs some improvements.")
        return 1
    else:
        print("\n❌ Modular architecture has significant issues.")
        return 2

if __name__ == "__main__":
    sys.exit(main())