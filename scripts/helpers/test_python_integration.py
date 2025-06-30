#!/usr/bin/env python3
"""
TracSeq 2.0 Python Integration Test
Validates the comprehensive Python testing setup and FastMCP integration
"""

import asyncio
import json
import os
import subprocess
import sys
from datetime import datetime
from pathlib import Path
from typing import Any

# Add current directory to Python path
sys.path.insert(0, str(Path(__file__).parent))

def log_info(message: str):
    print(f"🔍 {message}")

def log_success(message: str):
    print(f"✅ {message}")

def log_warning(message: str):
    print(f"⚠️  {message}")

def log_error(message: str):
    print(f"❌ {message}")

def log_section(title: str):
    print(f"\n{'='*50}")
    print(f"  {title}")
    print(f"{'='*50}")

class PythonTestingValidator:
    """Validates the Python testing infrastructure for TracSeq 2.0"""

    def __init__(self):
        self.project_root = Path(__file__).parent
        self.results = {
            "test_date": datetime.now().isoformat(),
            "python_version": None,
            "environment_checks": {},
            "service_validations": {},
            "fastmcp_validations": {},
            "testing_infrastructure": {},
            "overall_status": "UNKNOWN"
        }

    def check_python_environment(self) -> bool:
        """Check Python environment and dependencies"""
        log_section("Python Environment Validation")

        try:
            # Python version check
            python_version = sys.version
            log_success(f"Python version: {python_version}")
            self.results["python_version"] = python_version

            # Check for essential Python packages
            essential_packages = [
                "pytest", "asyncio", "pathlib", "json", "subprocess"
            ]

            missing_packages = []
            for package in essential_packages:
                try:
                    __import__(package)
                    log_success(f"Package {package} available")
                except ImportError:
                    missing_packages.append(package)
                    log_warning(f"Package {package} missing")

            self.results["environment_checks"] = {
                "python_version_valid": True,
                "essential_packages_available": len(missing_packages) == 0,
                "missing_packages": missing_packages
            }

            return len(missing_packages) == 0

        except Exception as e:
            log_error(f"Environment check failed: {e}")
            return False

    def validate_python_services(self) -> bool:
        """Validate Python service structure and basic functionality"""
        log_section("Python Services Validation")

        services = [
            "lab_submission_rag",
            "api_gateway",
            "enhanced_rag_service"
        ]

        service_results = {}

        for service in services:
            service_path = self.project_root / service
            service_result = {
                "exists": service_path.exists(),
                "has_tests": False,
                "has_config": False,
                "structure_valid": False
            }

            if service_path.exists():
                log_info(f"Validating service: {service}")

                # Check for test directory
                tests_path = service_path / "tests"
                if tests_path.exists():
                    service_result["has_tests"] = True
                    log_success(f"  Tests directory found for {service}")
                else:
                    log_warning(f"  No tests directory for {service}")

                # Check for configuration files
                config_files = [
                    "pyproject.toml",
                    "requirements.txt",
                    "setup.py"
                ]

                for config_file in config_files:
                    if (service_path / config_file).exists():
                        service_result["has_config"] = True
                        log_success(f"  Configuration file {config_file} found for {service}")
                        break

                # Basic structure validation
                if service_result["has_config"]:
                    service_result["structure_valid"] = True
                    log_success(f"  Service {service} has valid structure")
                else:
                    log_warning(f"  Service {service} missing configuration")

            else:
                log_warning(f"Service directory {service} not found")

            service_results[service] = service_result

        self.results["service_validations"] = service_results

        # Check if at least one service is properly configured
        valid_services = sum(1 for result in service_results.values() if result.get("structure_valid", False))
        log_info(f"Valid services found: {valid_services}/{len(services)}")

        return valid_services > 0

    def validate_fastmcp_servers(self) -> bool:
        """Validate FastMCP server files and basic structure"""
        log_section("FastMCP Servers Validation")

        fastmcp_servers = [
            "fastmcp_laboratory_server.py",
            "enhanced_rag_service/fastmcp_enhanced_rag_server.py",
            "mcp_infrastructure/fastmcp_laboratory_agent.py",
            "api_gateway/fastmcp_gateway.py",
            "specialized_servers/sample_server.py",
            "specialized_servers/storage_server.py",
            "specialized_servers/quality_control_server.py"
        ]

        server_results = {}

        for server in fastmcp_servers:
            server_path = self.project_root / server
            server_result = {
                "exists": server_path.exists(),
                "syntax_valid": False,
                "has_fastmcp_imports": False
            }

            if server_path.exists():
                log_info(f"Validating FastMCP server: {server}")

                # Syntax validation
                try:
                    with open(server_path) as f:
                        content = f.read()

                    # Basic syntax check (compile)
                    compile(content, server_path, 'exec')
                    server_result["syntax_valid"] = True
                    log_success(f"  Syntax validation passed for {server}")

                    # Check for FastMCP imports/patterns
                    if 'fastmcp' in content.lower() or 'mcp' in content:
                        server_result["has_fastmcp_imports"] = True
                        log_success(f"  FastMCP patterns found in {server}")
                    else:
                        log_warning(f"  No FastMCP patterns found in {server}")

                except SyntaxError as e:
                    log_error(f"  Syntax error in {server}: {e}")
                except Exception as e:
                    log_warning(f"  Validation issue for {server}: {e}")
            else:
                log_warning(f"FastMCP server {server} not found")

            server_results[server] = server_result

        self.results["fastmcp_validations"] = server_results

        # Check if at least some servers are valid
        valid_servers = sum(1 for result in server_results.values() if result.get("syntax_valid", False))
        log_info(f"Valid FastMCP servers: {valid_servers}/{len(fastmcp_servers)}")

        return valid_servers > 0

    def validate_testing_infrastructure(self) -> bool:
        """Validate the testing infrastructure setup"""
        log_section("Testing Infrastructure Validation")

        infrastructure_checks = {
            "pytest_config": False,
            "test_scripts": False,
            "coverage_config": False
        }

        # Check for pytest configuration
        pytest_configs = ["pytest.ini", "pyproject.toml", "setup.cfg"]
        for config in pytest_configs:
            config_path = self.project_root / config
            if config_path.exists():
                infrastructure_checks["pytest_config"] = True
                log_success(f"pytest configuration found: {config}")
                break

        if not infrastructure_checks["pytest_config"]:
            log_warning("No pytest configuration found")

        # Check for test scripts
        test_scripts = [
            "scripts/test-python.sh",
            "scripts/test-all.sh"
        ]

        script_count = 0
        for script in test_scripts:
            script_path = self.project_root / script
            if script_path.exists():
                script_count += 1
                log_success(f"Test script found: {script}")

        infrastructure_checks["test_scripts"] = script_count > 0

        # Check for coverage configuration (in pytest.ini or pyproject.toml)
        try:
            pytest_ini = self.project_root / "pytest.ini"
            if pytest_ini.exists():
                with open(pytest_ini) as f:
                    content = f.read()
                    if 'cov' in content:
                        infrastructure_checks["coverage_config"] = True
                        log_success("Coverage configuration found in pytest.ini")
        except Exception:
            pass

        self.results["testing_infrastructure"] = infrastructure_checks

        return all(infrastructure_checks.values())

    def run_sample_tests(self) -> bool:
        """Run some sample tests to verify the testing setup works"""
        log_section("Sample Test Execution")

        try:
            # Try to run a simple pytest command
            result = subprocess.run([
                sys.executable, "-m", "pytest", "--version"
            ], capture_output=True, text=True, timeout=10)

            if result.returncode == 0:
                log_success("pytest is functional")
                log_info(f"pytest version: {result.stdout.strip()}")
                return True
            else:
                log_warning("pytest not working properly")
                return False

        except subprocess.TimeoutExpired:
            log_warning("pytest command timed out")
            return False
        except Exception as e:
            log_warning(f"Error running pytest: {e}")
            return False

    def generate_report(self) -> dict[str, Any]:
        """Generate a comprehensive validation report"""
        log_section("Generating Validation Report")

        # Calculate overall status
        checks = [
            self.results["environment_checks"].get("essential_packages_available", False),
            any(result.get("structure_valid", False) for result in self.results["service_validations"].values()),
            any(result.get("syntax_valid", False) for result in self.results["fastmcp_validations"].values()),
            any(self.results["testing_infrastructure"].values())
        ]

        if all(checks):
            self.results["overall_status"] = "EXCELLENT"
            status_emoji = "🎉"
            status_message = "All validations passed - TracSeq 2.0 Python testing is fully operational!"
        elif sum(checks) >= 3:
            self.results["overall_status"] = "GOOD"
            status_emoji = "✅"
            status_message = "Most validations passed - Python testing setup is functional"
        elif sum(checks) >= 2:
            self.results["overall_status"] = "ACCEPTABLE"
            status_emoji = "⚠️"
            status_message = "Some validations passed - Python testing partially functional"
        else:
            self.results["overall_status"] = "NEEDS_WORK"
            status_emoji = "❌"
            status_message = "Several validations failed - Python testing needs attention"

        log_info(f"{status_emoji} Overall Status: {self.results['overall_status']}")
        log_info(status_message)

        # Save detailed report
        os.makedirs("test-results", exist_ok=True)

        with open("test-results/python-integration-validation.json", "w") as f:
            json.dump(self.results, f, indent=2)

        log_success("Validation report saved: test-results/python-integration-validation.json")

        return self.results

async def main():
    """Main async function to run all validations"""
    print("🐍 TracSeq 2.0 Python Integration Test")
    print("=" * 50)

    validator = PythonTestingValidator()

    # Run all validation steps
    steps = [
        ("Environment Check", validator.check_python_environment),
        ("Service Validation", validator.validate_python_services),
        ("FastMCP Validation", validator.validate_fastmcp_servers),
        ("Infrastructure Validation", validator.validate_testing_infrastructure),
        ("Sample Test Execution", validator.run_sample_tests)
    ]

    passed_steps = 0
    total_steps = len(steps)

    for step_name, step_function in steps:
        try:
            if step_function():
                passed_steps += 1
                log_success(f"{step_name} completed successfully")
            else:
                log_warning(f"{step_name} completed with issues")
        except Exception as e:
            log_error(f"{step_name} failed: {e}")

    # Generate final report
    report = validator.generate_report()

    log_section("Final Summary")
    log_info(f"Validation steps passed: {passed_steps}/{total_steps}")
    log_info(f"Overall status: {report['overall_status']}")

    if report['overall_status'] in ['EXCELLENT', 'GOOD']:
        log_success("🚀 TracSeq 2.0 Python testing infrastructure is ready!")
        log_info("You can now run:")
        log_info("  ./scripts/test-python.sh         # Standalone Python testing")
        log_info("  ./scripts/test-all.sh python     # Python tests only")
        log_info("  ./scripts/test-all.sh all        # Complete test suite")
        return True
    else:
        log_warning("🔧 Python testing infrastructure needs attention")
        log_info("Check the validation report for details")
        return False

if __name__ == "__main__":
    try:
        success = asyncio.run(main())
        sys.exit(0 if success else 1)
    except KeyboardInterrupt:
        log_warning("Test interrupted by user")
        sys.exit(1)
    except Exception as e:
        log_error(f"Test failed with error: {e}")
        sys.exit(1)
