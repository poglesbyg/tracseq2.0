#!/usr/bin/env python3
"""
Test runner script for the RAG system tests

This script provides convenient ways to run different test suites with various options.
"""

import sys
import os
import subprocess
import argparse
from pathlib import Path

# Add the parent directory to the path to import the application modules
sys.path.insert(0, str(Path(__file__).parent.parent))


def run_command(cmd, description):
    """Run a command and display the results"""
    print(f"\n{'='*60}")
    print(f"Running: {description}")
    print(f"Command: {' '.join(cmd)}")
    print(f"{'='*60}")
    
    try:
        result = subprocess.run(cmd, capture_output=False, text=True, check=True)
        print(f"\n✅ {description} completed successfully")
        return True
    except subprocess.CalledProcessError as e:
        print(f"\n❌ {description} failed with exit code {e.returncode}")
        return False
    except Exception as e:
        print(f"\n❌ {description} failed with error: {str(e)}")
        return False


def run_unit_tests():
    """Run unit tests only"""
    cmd = ["python", "-m", "pytest", "tests/unit/", "-v", "--tb=short"]
    return run_command(cmd, "Unit Tests")


def run_integration_tests():
    """Run integration tests only"""
    cmd = ["python", "-m", "pytest", "tests/integration/", "-v", "--tb=short"]
    return run_command(cmd, "Integration Tests")


def run_all_tests():
    """Run all tests"""
    cmd = ["python", "-m", "pytest", "tests/", "-v", "--tb=short"]
    return run_command(cmd, "All Tests")


def run_tests_with_coverage():
    """Run tests with coverage reporting"""
    cmd = [
        "python", "-m", "pytest", 
        "tests/", 
        "-v", 
        "--cov=rag", 
        "--cov=api",
        "--cov-report=html:htmlcov",
        "--cov-report=term-missing",
        "--cov-report=xml"
    ]
    return run_command(cmd, "Tests with Coverage")


def run_specific_test(test_path):
    """Run a specific test file or test function"""
    cmd = ["python", "-m", "pytest", test_path, "-v", "--tb=short"]
    return run_command(cmd, f"Specific Test: {test_path}")


def run_fast_tests():
    """Run tests excluding slow ones"""
    cmd = ["python", "-m", "pytest", "tests/", "-v", "--tb=short", "-m", "not slow"]
    return run_command(cmd, "Fast Tests (excluding slow tests)")


def run_marked_tests(marker):
    """Run tests with a specific marker"""
    cmd = ["python", "-m", "pytest", "tests/", "-v", "--tb=short", "-m", marker]
    return run_command(cmd, f"Tests marked with '{marker}'")


def check_test_environment():
    """Check if the test environment is properly set up"""
    print("🔍 Checking test environment...")
    
    # Check if pytest is installed
    try:
        import pytest
        print(f"✅ pytest is installed (version: {pytest.__version__})")
    except ImportError:
        print("❌ pytest is not installed")
        return False
    
    # Check if required test dependencies are available
    test_deps = [
        "pytest-asyncio",
        "pytest-cov",
        "httpx",  # For FastAPI testing
    ]
    
    missing_deps = []
    for dep in test_deps:
        try:
            __import__(dep.replace("-", "_"))
            print(f"✅ {dep} is available")
        except ImportError:
            missing_deps.append(dep)
            print(f"❌ {dep} is not installed")
    
    if missing_deps:
        print(f"\n❌ Missing dependencies: {', '.join(missing_deps)}")
        print("Install them with: pip install " + " ".join(missing_deps))
        return False
    
    # Check if test directory structure exists
    test_dirs = ["tests", "tests/unit", "tests/integration"]
    for test_dir in test_dirs:
        if not Path(test_dir).exists():
            print(f"❌ Test directory {test_dir} does not exist")
            return False
        else:
            print(f"✅ Test directory {test_dir} exists")
    
    print("\n✅ Test environment is properly set up!")
    return True


def main():
    """Main entry point for the test runner"""
    parser = argparse.ArgumentParser(
        description="Test runner for the RAG system",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python tests/run_tests.py --unit                 # Run unit tests only
  python tests/run_tests.py --integration         # Run integration tests only
  python tests/run_tests.py --all                 # Run all tests
  python tests/run_tests.py --coverage            # Run with coverage
  python tests/run_tests.py --fast                # Run fast tests only
  python tests/run_tests.py --marker api          # Run API tests only
  python tests/run_tests.py --specific tests/unit/test_document_processor.py
  python tests/run_tests.py --check               # Check test environment
        """
    )
    
    parser.add_argument("--unit", action="store_true", help="Run unit tests only")
    parser.add_argument("--integration", action="store_true", help="Run integration tests only")
    parser.add_argument("--all", action="store_true", help="Run all tests")
    parser.add_argument("--coverage", action="store_true", help="Run tests with coverage")
    parser.add_argument("--fast", action="store_true", help="Run fast tests only")
    parser.add_argument("--marker", type=str, help="Run tests with specific marker")
    parser.add_argument("--specific", type=str, help="Run specific test file or function")
    parser.add_argument("--check", action="store_true", help="Check test environment")
    
    args = parser.parse_args()
    
    # If no arguments provided, show help
    if not any(vars(args).values()):
        parser.print_help()
        return
    
    success = True
    
    if args.check:
        success = check_test_environment()
    
    if args.unit:
        success = run_unit_tests() and success
    
    if args.integration:
        success = run_integration_tests() and success
    
    if args.all:
        success = run_all_tests() and success
    
    if args.coverage:
        success = run_tests_with_coverage() and success
    
    if args.fast:
        success = run_fast_tests() and success
    
    if args.marker:
        success = run_marked_tests(args.marker) and success
    
    if args.specific:
        success = run_specific_test(args.specific) and success
    
    if success:
        print("\n🎉 All requested tests completed successfully!")
        sys.exit(0)
    else:
        print("\n💥 Some tests failed!")
        sys.exit(1)


if __name__ == "__main__":
    main() 
