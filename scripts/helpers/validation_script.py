#!/usr/bin/env python3
"""
TracSeq 2.0 Microservices Validation Script
Comprehensive validation of all microservices implementation
"""

import sys
from pathlib import Path
from typing import Any


class MicroserviceValidator:
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
        self.results = {}

    def validate_all_services(self) -> dict[str, Any]:
        """Validate all microservices"""
        print("🔍 Starting comprehensive microservices validation...")
        print("=" * 80)

        for service in self.services:
            print(f"\n📋 Validating {service}...")
            self.results[service] = self.validate_service(service)

        self.generate_summary_report()
        return self.results

    def validate_service(self, service_name: str) -> dict[str, Any]:
        """Validate a single microservice"""
        service_path = self.root_dir / service_name

        result = {
            "exists": service_path.exists(),
            "structure": {},
            "config": {},
            "documentation": {},
            "docker": {},
            "database": {},
            "errors": [],
            "warnings": [],
            "score": 0
        }

        if not result["exists"]:
            result["errors"].append(f"Service directory {service_name} does not exist")
            return result

        # Check directory structure
        result["structure"] = self.check_structure(service_path, service_name)

        # Check configuration files
        result["config"] = self.check_config_files(service_path, service_name)

        # Check documentation
        result["documentation"] = self.check_documentation(service_path, service_name)

        # Check Docker configuration
        result["docker"] = self.check_docker_config(service_path, service_name)

        # Check database configuration
        result["database"] = self.check_database_config(service_path, service_name)

        # Calculate score
        result["score"] = self.calculate_score(result)

        return result

    def check_structure(self, service_path: Path, service_name: str) -> dict[str, Any]:
        """Check service directory structure"""
        structure = {
            "src_exists": False,
            "cargo_toml": False,
            "dockerfile": False,
            "readme": False,
            "source_files": 0,
            "handler_files": 0,
            "test_files": 0
        }

        # Check for src directory or main files
        if service_name in ["api_gateway", "enhanced_rag_service"]:
            # Python services
            src_path = service_path / "src"
            main_py = service_path / "src" / "main.py"
            requirements = service_path / "requirements.txt"

            structure["src_exists"] = src_path.exists()
            structure["cargo_toml"] = requirements.exists()  # Using this field for requirements.txt
            structure["main_file"] = main_py.exists()
        else:
            # Rust services
            src_path = service_path / "src"
            cargo_toml = service_path / "Cargo.toml"

            structure["src_exists"] = src_path.exists()
            structure["cargo_toml"] = cargo_toml.exists()

        # Check for common files
        structure["dockerfile"] = (service_path / "Dockerfile").exists()
        structure["readme"] = (service_path / "README.md").exists()

        # Count source files
        if src_path.exists():
            structure["source_files"] = len(list(src_path.rglob("*.rs" if service_name not in ["api_gateway", "enhanced_rag_service"] else "*.py")))

            handlers_path = src_path / "handlers"
            if handlers_path.exists():
                structure["handler_files"] = len(list(handlers_path.glob("*.rs" if service_name not in ["api_gateway", "enhanced_rag_service"] else "*.py")))

            tests_path = src_path / "tests"
            if tests_path.exists():
                structure["test_files"] = len(list(tests_path.glob("*.rs" if service_name not in ["api_gateway", "enhanced_rag_service"] else "*.py")))

        return structure

    def check_config_files(self, service_path: Path, service_name: str) -> dict[str, Any]:
        """Check configuration files"""
        config = {
            "cargo_toml_valid": False,
            "has_dependencies": False,
            "docker_compose": False,
            "config_files": []
        }

        # Check Cargo.toml for Rust services
        if service_name not in ["api_gateway", "enhanced_rag_service"]:
            cargo_toml = service_path / "Cargo.toml"
            if cargo_toml.exists():
                try:
                    content = cargo_toml.read_text()
                    config["cargo_toml_valid"] = "[package]" in content and "[dependencies]" in content
                    config["has_dependencies"] = "axum" in content or "tokio" in content
                except Exception:
                    config["cargo_toml_valid"] = False
        else:
            # Check requirements.txt for Python services
            requirements = service_path / "requirements.txt"
            if requirements.exists():
                try:
                    content = requirements.read_text()
                    config["cargo_toml_valid"] = len(content.strip()) > 0
                    config["has_dependencies"] = "fastapi" in content.lower() or "flask" in content.lower()
                except Exception:
                    config["cargo_toml_valid"] = False

        # Check for docker-compose.yml
        docker_compose = service_path / "docker-compose.yml"
        config["docker_compose"] = docker_compose.exists()

        # Check for config files
        config_dir = service_path / "config"
        if config_dir.exists():
            config["config_files"] = [f.name for f in config_dir.glob("*")]

        return config

    def check_documentation(self, service_path: Path, service_name: str) -> dict[str, Any]:
        """Check documentation completeness"""
        docs = {
            "readme_exists": False,
            "readme_size": 0,
            "has_api_docs": False,
            "has_examples": False,
            "docs_folder": False
        }

        readme = service_path / "README.md"
        if readme.exists():
            docs["readme_exists"] = True
            docs["readme_size"] = len(readme.read_text())

            content = readme.read_text().lower()
            docs["has_api_docs"] = "api" in content and ("endpoint" in content or "route" in content)
            docs["has_examples"] = "example" in content or "usage" in content

        docs_folder = service_path / "docs"
        docs["docs_folder"] = docs_folder.exists()

        return docs

    def check_docker_config(self, service_path: Path, service_name: str) -> dict[str, Any]:
        """Check Docker configuration"""
        docker = {
            "dockerfile_exists": False,
            "dockerfile_valid": False,
            "compose_exists": False,
            "compose_valid": False,
            "multi_stage": False
        }

        dockerfile = service_path / "Dockerfile"
        if dockerfile.exists():
            docker["dockerfile_exists"] = True
            try:
                content = dockerfile.read_text()
                docker["dockerfile_valid"] = "FROM" in content and ("COPY" in content or "ADD" in content)
                docker["multi_stage"] = content.count("FROM") > 1
            except Exception:
                docker["dockerfile_valid"] = False

        compose_file = service_path / "docker-compose.yml"
        if compose_file.exists():
            docker["compose_exists"] = True
            try:
                content = compose_file.read_text()
                docker["compose_valid"] = "version:" in content and "services:" in content
            except Exception:
                docker["compose_valid"] = False

        return docker

    def check_database_config(self, service_path: Path, service_name: str) -> dict[str, Any]:
        """Check database configuration"""
        database = {
            "migrations_exist": False,
            "migration_count": 0,
            "schema_files": [],
            "has_models": False
        }

        # Check for migrations
        migrations_dir = service_path / "migrations"
        if migrations_dir.exists():
            database["migrations_exist"] = True
            migration_files = list(migrations_dir.glob("*.sql"))
            database["migration_count"] = len(migration_files)
            database["schema_files"] = [f.name for f in migration_files]

        # Check for models
        src_path = service_path / "src"
        if src_path.exists():
            models_file = src_path / "models.rs"
            models_py = src_path / "models.py"
            database["has_models"] = models_file.exists() or models_py.exists()

            # Also check for models directory
            models_dir = src_path / "models"
            if models_dir.exists():
                database["has_models"] = True

        return database

    def calculate_score(self, result: dict[str, Any]) -> int:
        """Calculate service completeness score (0-100)"""
        score = 0

        # Structure checks (30 points)
        if result["structure"]["src_exists"]:
            score += 5
        if result["structure"]["cargo_toml"]:
            score += 5
        if result["structure"]["dockerfile"]:
            score += 5
        if result["structure"]["readme"]:
            score += 5
        if result["structure"]["source_files"] > 5:
            score += 5
        if result["structure"]["handler_files"] > 0:
            score += 5

        # Configuration checks (25 points)
        if result["config"]["cargo_toml_valid"]:
            score += 10
        if result["config"]["has_dependencies"]:
            score += 5
        if result["config"]["docker_compose"]:
            score += 5
        if result["config"]["config_files"]:
            score += 5

        # Documentation checks (20 points)
        if result["documentation"]["readme_exists"]:
            score += 5
        if result["documentation"]["readme_size"] > 1000:
            score += 5
        if result["documentation"]["has_api_docs"]:
            score += 5
        if result["documentation"]["has_examples"]:
            score += 5

        # Docker checks (15 points)
        if result["docker"]["dockerfile_exists"]:
            score += 5
        if result["docker"]["dockerfile_valid"]:
            score += 5
        if result["docker"]["multi_stage"]:
            score += 5

        # Database checks (10 points)
        if result["database"]["migrations_exist"]:
            score += 5
        if result["database"]["has_models"]:
            score += 5

        return min(score, 100)

    def generate_summary_report(self):
        """Generate summary validation report"""
        print("\n" + "=" * 80)
        print("📊 MICROSERVICES VALIDATION SUMMARY")
        print("=" * 80)

        total_services = len(self.services)
        total_score = 0

        print(f"\n{'Service':<25} {'Score':<8} {'Status':<12} {'Issues'}")
        print("-" * 80)

        for service, result in self.results.items():
            score = result.get("score", 0)
            total_score += score

            status = "✅ EXCELLENT" if score >= 90 else \
                    "✅ GOOD" if score >= 80 else \
                    "⚠️  NEEDS WORK" if score >= 60 else \
                    "❌ CRITICAL"

            issues = len(result.get("errors", [])) + len(result.get("warnings", []))

            print(f"{service:<25} {score:<8} {status:<12} {issues}")

        avg_score = total_score / total_services

        print("\n" + "=" * 80)
        print("📈 OVERALL STATISTICS")
        print(f"Total Services: {total_services}")
        print(f"Average Score: {avg_score:.1f}/100")
        print(f"Services Ready: {sum(1 for r in self.results.values() if r.get('score', 0) >= 80)}/{total_services}")

        # Detailed issues
        print("\n🔍 DETAILED ISSUES")
        for service, result in self.results.items():
            if result.get("errors") or result.get("warnings"):
                print(f"\n{service}:")
                for error in result.get("errors", []):
                    print(f"  ❌ {error}")
                for warning in result.get("warnings", []):
                    print(f"  ⚠️  {warning}")

        print("\n" + "=" * 80)

    def run_python_syntax_check(self, service_path: Path) -> list[str]:
        """Run Python syntax check"""
        errors = []
        try:
            import ast
            for py_file in service_path.rglob("*.py"):
                try:
                    with open(py_file, encoding='utf-8') as f:
                        ast.parse(f.read())
                except SyntaxError as e:
                    errors.append(f"Syntax error in {py_file}: {e}")
                except Exception as e:
                    errors.append(f"Error parsing {py_file}: {e}")
        except Exception as e:
            errors.append(f"Failed to run Python syntax check: {e}")
        return errors

if __name__ == "__main__":
    validator = MicroserviceValidator()
    results = validator.validate_all_services()

    # Exit with error code if any critical issues found
    critical_issues = sum(1 for r in results.values() if r.get("score", 0) < 60)
    sys.exit(1 if critical_issues > 0 else 0)
