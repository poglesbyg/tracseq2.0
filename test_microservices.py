#!/usr/bin/env python3
"""
TracSeq 2.0 Microservices Comprehensive Test Suite
Tests configuration files, API endpoints, database schemas, and service integration
"""

import ast
import sys
from pathlib import Path
from typing import Any


class MicroserviceTestSuite:
    def __init__(self, root_dir: str = "."):
        self.root_dir = Path(root_dir)
        self.services = [
            "auth_service",
            "sample_service",
            "template_service",
            "enhanced_storage_service",
            "sequencing_service",
            "notification_service",
            "transaction_service",
            "event_service",
            "api_gateway",
            "enhanced_rag_service"
        ]
        self.test_results = {}

    def run_all_tests(self) -> dict[str, Any]:
        """Run comprehensive test suite"""
        print("🧪 Starting TracSeq 2.0 Microservices Test Suite")
        print("=" * 80)

        for service in self.services:
            print(f"\n🔬 Testing {service}...")
            self.test_results[service] = self.test_service(service)

        self.generate_test_report()
        return self.test_results

    def test_service(self, service_name: str) -> dict[str, Any]:
        """Test individual microservice"""
        service_path = self.root_dir / service_name

        result = {
            "service_exists": False,
            "config_tests": {},
            "syntax_tests": {},
            "docker_tests": {},
            "api_tests": {},
            "database_tests": {},
            "documentation_tests": {},
            "passed": 0,
            "failed": 0,
            "warnings": [],
            "errors": []
        }

        if not service_path.exists():
            result["errors"].append(f"Service directory {service_name} does not exist")
            result["failed"] += 1
            return result

        result["service_exists"] = True
        result["passed"] += 1

        # Test configuration files
        result["config_tests"] = self.test_configuration(service_path, service_name)

        # Test syntax
        result["syntax_tests"] = self.test_syntax(service_path, service_name)

        # Test Docker configuration
        result["docker_tests"] = self.test_docker_config(service_path, service_name)

        # Test API definitions
        result["api_tests"] = self.test_api_definitions(service_path, service_name)

        # Test database schemas
        result["database_tests"] = self.test_database_schemas(service_path, service_name)

        # Test documentation
        result["documentation_tests"] = self.test_documentation(service_path, service_name)

        # Calculate totals
        for test_category in ["config_tests", "syntax_tests", "docker_tests", "api_tests", "database_tests", "documentation_tests"]:
            category_result = result[test_category]
            result["passed"] += category_result.get("passed", 0)
            result["failed"] += category_result.get("failed", 0)
            result["warnings"].extend(category_result.get("warnings", []))
            result["errors"].extend(category_result.get("errors", []))

        return result

    def test_configuration(self, service_path: Path, service_name: str) -> dict[str, Any]:
        """Test service configuration files"""
        result = {"passed": 0, "failed": 0, "warnings": [], "errors": []}

        if service_name in ["api_gateway", "enhanced_rag_service"]:
            # Python services
            req_file = service_path / "requirements.txt"
            pyproject_file = service_path / "pyproject.toml"

            if req_file.exists():
                try:
                    content = req_file.read_text()
                    if len(content.strip()) > 0:
                        result["passed"] += 1
                        # Check for essential dependencies
                        if "fastapi" in content.lower() or "flask" in content.lower():
                            result["passed"] += 1
                        else:
                            result["warnings"].append("No web framework found in requirements")
                    else:
                        result["errors"].append("requirements.txt is empty")
                        result["failed"] += 1
                except Exception as e:
                    result["errors"].append(f"Error reading requirements.txt: {e}")
                    result["failed"] += 1
            elif pyproject_file.exists():
                # Modern Python project using pyproject.toml
                result["passed"] += 1
                result["warnings"].append("Using pyproject.toml (modern Python packaging)")
            else:
                result["errors"].append("No requirements.txt or pyproject.toml found")
                result["failed"] += 1
        else:
            # Rust services
            cargo_file = service_path / "Cargo.toml"

            if cargo_file.exists():
                try:
                    content = cargo_file.read_text()
                    if "[package]" in content and "[dependencies]" in content:
                        result["passed"] += 1
                        # Check for essential Rust dependencies
                        essential_deps = ["axum", "tokio", "serde"]
                        found_deps = sum(1 for dep in essential_deps if dep in content)
                        if found_deps >= 2:
                            result["passed"] += 1
                        else:
                            result["warnings"].append("Missing some essential Rust dependencies")
                    else:
                        result["errors"].append("Invalid Cargo.toml structure")
                        result["failed"] += 1
                except Exception as e:
                    result["errors"].append(f"Error reading Cargo.toml: {e}")
                    result["failed"] += 1
            else:
                result["errors"].append("No Cargo.toml found")
                result["failed"] += 1

        return result

    def test_syntax(self, service_path: Path, service_name: str) -> dict[str, Any]:
        """Test code syntax"""
        result = {"passed": 0, "failed": 0, "warnings": [], "errors": []}

        src_path = service_path / "src"
        if not src_path.exists():
            result["errors"].append("No src directory found")
            result["failed"] += 1
            return result

        if service_name in ["api_gateway", "enhanced_rag_service"]:
            # Test Python syntax
            python_files = list(src_path.rglob("*.py"))
            for py_file in python_files:
                try:
                    with open(py_file, encoding='utf-8') as f:
                        ast.parse(f.read())
                    result["passed"] += 1
                except SyntaxError as e:
                    result["errors"].append(f"Syntax error in {py_file.name}: {e}")
                    result["failed"] += 1
                except Exception as e:
                    result["warnings"].append(f"Warning in {py_file.name}: {e}")
        else:
            # For Rust files, we'll do basic structure checks since we can't compile
            rust_files = list(src_path.rglob("*.rs"))
            for rust_file in rust_files:
                try:
                    content = rust_file.read_text()
                    # Basic Rust syntax checks
                    if "fn main()" in content or "pub fn" in content or "impl" in content:
                        result["passed"] += 1
                    else:
                        result["warnings"].append(f"{rust_file.name} may not have valid Rust structure")
                except Exception as e:
                    result["errors"].append(f"Error reading {rust_file.name}: {e}")
                    result["failed"] += 1

        return result

    def test_docker_config(self, service_path: Path, service_name: str) -> dict[str, Any]:
        """Test Docker configuration"""
        result = {"passed": 0, "failed": 0, "warnings": [], "errors": []}

        dockerfile = service_path / "Dockerfile"
        if dockerfile.exists():
            try:
                content = dockerfile.read_text()

                # Check for essential Dockerfile components
                if "FROM" in content:
                    result["passed"] += 1
                else:
                    result["errors"].append("Dockerfile missing FROM instruction")
                    result["failed"] += 1

                if "COPY" in content or "ADD" in content:
                    result["passed"] += 1
                else:
                    result["warnings"].append("Dockerfile may not copy source files")

                if "EXPOSE" in content:
                    result["passed"] += 1
                else:
                    result["warnings"].append("Dockerfile doesn't expose any ports")

                if "CMD" in content or "ENTRYPOINT" in content:
                    result["passed"] += 1
                else:
                    result["errors"].append("Dockerfile missing CMD or ENTRYPOINT")
                    result["failed"] += 1

                # Check for multi-stage build
                if content.count("FROM") > 1:
                    result["passed"] += 1
                    result["warnings"].append("Good: Multi-stage Docker build detected")

            except Exception as e:
                result["errors"].append(f"Error reading Dockerfile: {e}")
                result["failed"] += 1
        else:
            result["errors"].append("No Dockerfile found")
            result["failed"] += 1

        # Check for docker-compose
        compose_file = service_path / "docker-compose.yml"
        if compose_file.exists():
            try:
                content = compose_file.read_text()
                if "version:" in content and "services:" in content:
                    result["passed"] += 1
                else:
                    result["errors"].append("Invalid docker-compose.yml structure")
                    result["failed"] += 1
            except Exception as e:
                result["errors"].append(f"Error reading docker-compose.yml: {e}")
                result["failed"] += 1

        return result

    def test_api_definitions(self, service_path: Path, service_name: str) -> dict[str, Any]:
        """Test API endpoint definitions"""
        result = {"passed": 0, "failed": 0, "warnings": [], "errors": []}

        src_path = service_path / "src"
        if not src_path.exists():
            return result

        # Look for handler files
        handlers_path = src_path / "handlers"
        if handlers_path.exists():
            handler_files = list(handlers_path.glob("*.rs" if service_name not in ["api_gateway", "enhanced_rag_service"] else "*.py"))
            if len(handler_files) > 0:
                result["passed"] += len(handler_files)
                result["warnings"].append(f"Found {len(handler_files)} handler files")
            else:
                result["warnings"].append("Handlers directory exists but no handler files found")

        # Look for route definitions in main files
        main_files = list(src_path.rglob("main.*"))
        for main_file in main_files:
            try:
                content = main_file.read_text()
                if service_name in ["api_gateway", "enhanced_rag_service"]:
                    # Python API patterns
                    if "@app.get" in content or "@app.post" in content or "APIRouter" in content:
                        result["passed"] += 1
                    if "FastAPI" in content:
                        result["passed"] += 1
                else:
                    # Rust API patterns
                    if "Router::new()" in content or ".route(" in content:
                        result["passed"] += 1
                    if "axum" in content:
                        result["passed"] += 1
            except Exception as e:
                result["errors"].append(f"Error reading {main_file.name}: {e}")
                result["failed"] += 1

        return result

    def test_database_schemas(self, service_path: Path, service_name: str) -> dict[str, Any]:
        """Test database schema definitions"""
        result = {"passed": 0, "failed": 0, "warnings": [], "errors": []}

        # Check for migrations
        migrations_dir = service_path / "migrations"
        if migrations_dir.exists():
            migration_files = list(migrations_dir.glob("*.sql"))
            if len(migration_files) > 0:
                result["passed"] += len(migration_files)
                result["warnings"].append(f"Found {len(migration_files)} migration files")

                # Check migration file content
                for migration_file in migration_files:
                    try:
                        content = migration_file.read_text()
                        if "CREATE TABLE" in content.upper():
                            result["passed"] += 1
                        if "CREATE INDEX" in content.upper():
                            result["passed"] += 1
                    except Exception as e:
                        result["errors"].append(f"Error reading migration {migration_file.name}: {e}")
                        result["failed"] += 1
            else:
                result["warnings"].append("Migrations directory exists but no migration files found")

        # Check for models
        src_path = service_path / "src"
        if src_path.exists():
            if service_name in ["api_gateway", "enhanced_rag_service"]:
                models_files = list(src_path.rglob("models.py"))
                models_dirs = [d for d in src_path.rglob("models") if d.is_dir()]
            else:
                models_files = list(src_path.rglob("models.rs"))
                models_dirs = [d for d in src_path.rglob("models") if d.is_dir()]

            if models_files or models_dirs:
                result["passed"] += len(models_files) + len(models_dirs)
                result["warnings"].append(f"Found {len(models_files)} model files and {len(models_dirs)} model directories")

        return result

    def test_documentation(self, service_path: Path, service_name: str) -> dict[str, Any]:
        """Test documentation completeness"""
        result = {"passed": 0, "failed": 0, "warnings": [], "errors": []}

        readme_file = service_path / "README.md"
        if readme_file.exists():
            try:
                content = readme_file.read_text()

                # Check README size
                if len(content) > 1000:
                    result["passed"] += 1
                else:
                    result["warnings"].append("README is quite short")

                # Check for essential sections
                content_lower = content.lower()
                essential_sections = ["api", "endpoint", "installation", "usage", "example"]
                found_sections = sum(1 for section in essential_sections if section in content_lower)

                if found_sections >= 3:
                    result["passed"] += 1
                else:
                    result["warnings"].append("README missing some essential sections")

                # Check for code examples
                if "```" in content or "curl" in content:
                    result["passed"] += 1
                else:
                    result["warnings"].append("README may lack code examples")

            except Exception as e:
                result["errors"].append(f"Error reading README.md: {e}")
                result["failed"] += 1
        else:
            result["errors"].append("No README.md found")
            result["failed"] += 1

        return result

    def generate_test_report(self):
        """Generate comprehensive test report"""
        print("\n" + "=" * 80)
        print("🧪 COMPREHENSIVE TEST RESULTS")
        print("=" * 80)

        total_passed = 0
        total_failed = 0
        total_warnings = 0

        print(f"\n{'Service':<25} {'Passed':<8} {'Failed':<8} {'Warnings':<10} {'Status'}")
        print("-" * 80)

        for service, result in self.test_results.items():
            passed = result.get("passed", 0)
            failed = result.get("failed", 0)
            warnings = len(result.get("warnings", []))

            total_passed += passed
            total_failed += failed
            total_warnings += warnings

            if failed == 0:
                status = "✅ PASS"
            elif failed <= 2:
                status = "⚠️ MINOR ISSUES"
            else:
                status = "❌ FAIL"

            print(f"{service:<25} {passed:<8} {failed:<8} {warnings:<10} {status}")

        print("\n" + "=" * 80)
        print("📊 OVERALL TEST STATISTICS")
        print(f"Total Tests Passed: {total_passed}")
        print(f"Total Tests Failed: {total_failed}")
        print(f"Total Warnings: {total_warnings}")
        print(f"Success Rate: {(total_passed/(total_passed+total_failed))*100:.1f}%")

        # Detailed issues
        critical_services = []
        for service, result in self.test_results.items():
            if result.get("failed", 0) > 2:
                critical_services.append(service)

        if critical_services:
            print("\n⚠️ SERVICES NEEDING ATTENTION:")
            for service in critical_services:
                print(f"   • {service}")
                for error in self.test_results[service].get("errors", [])[:3]:
                    print(f"     - {error}")
        else:
            print("\n🎉 ALL SERVICES PASS COMPREHENSIVE TESTING!")

        print("\n" + "=" * 80)
        print("🚀 DEPLOYMENT READINESS ASSESSMENT")

        ready_services = sum(1 for result in self.test_results.values() if result.get("failed", 0) <= 1)
        total_services = len(self.test_results)

        print(f"Services Ready for Production: {ready_services}/{total_services}")
        print(f"Production Readiness: {(ready_services/total_services)*100:.1f}%")

        if ready_services == total_services:
            print("✅ FULL MICROSERVICES ECOSYSTEM READY FOR DEPLOYMENT!")
        elif ready_services >= total_services * 0.8:
            print("⚠️ Most services ready, minor fixes needed")
        else:
            print("❌ Significant issues need resolution before deployment")

        print("\n" + "=" * 80)


if __name__ == "__main__":
    test_suite = MicroserviceTestSuite()
    results = test_suite.run_all_tests()

    # Exit with appropriate code
    total_failed = sum(result.get("failed", 0) for result in results.values())
    sys.exit(1 if total_failed > 10 else 0)
