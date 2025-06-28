#!/usr/bin/env python3
"""
Setup Verification Script for Laboratory Submission RAG System

This script verifies that your uv environment is properly configured
and all dependencies are working correctly.

Run this after setting up your environment with:
    uv venv
    source .venv/bin/activate
    uv pip install -e .
    uv pip install pydantic[email]
"""

import importlib
import subprocess
import sys
from pathlib import Path


def check_python_version():
    """Check if Python version meets requirements"""
    print("🐍 Checking Python version...")
    version = sys.version_info
    if version.major == 3 and version.minor >= 9:
        print(f"   ✅ Python {version.major}.{version.minor}.{version.micro} (compatible)")
        return True
    else:
        print(f"   ❌ Python {version.major}.{version.minor}.{version.micro} (requires >= 3.9)")
        return False


def check_uv_available():
    """Check if uv is available and virtual environment is active"""
    print("\n📦 Checking uv and virtual environment...")

    # Check if we're in a virtual environment
    venv_active = hasattr(sys, "real_prefix") or (
        hasattr(sys, "base_prefix") and sys.base_prefix != sys.prefix
    )
    if venv_active:
        print("   ✅ Virtual environment is active")
    else:
        print("   ⚠️  Virtual environment not detected (recommended to activate .venv)")

    # Check if uv is available
    try:
        result = subprocess.run(["uv", "--version"], capture_output=True, text=True)
        if result.returncode == 0:
            print(f"   ✅ uv is available: {result.stdout.strip()}")
            return True
        else:
            print("   ❌ uv command failed")
            return False
    except FileNotFoundError:
        print("   ❌ uv not found in PATH")
        return False


def check_core_dependencies():
    """Check if core RAG dependencies are properly installed"""
    print("\n🔧 Checking core dependencies...")

    dependencies = [
        ("langchain", "LangChain for document processing"),
        ("chromadb", "ChromaDB for vector storage"),
        ("sentence_transformers", "Sentence Transformers for embeddings"),
        ("pydantic", "Pydantic for data validation"),
        ("numpy", "NumPy for numerical operations"),
        ("pandas", "Pandas for data manipulation"),
    ]

    all_good = True

    for package, description in dependencies:
        try:
            module = importlib.import_module(package)
            version = getattr(module, "__version__", "unknown")
            print(f"   ✅ {package} {version} - {description}")
        except ImportError:
            print(f"   ❌ {package} - {description} (not installed)")
            all_good = False

    return all_good


def check_optional_dependencies():
    """Check optional dependencies"""
    print("\n🔌 Checking optional dependencies...")

    optional_deps = [
        ("openai", "OpenAI API client"),
        ("anthropic", "Anthropic API client"),
        ("ollama", "Ollama client for local Llama models"),
        ("email_validator", "Email validation for Pydantic"),
        ("PyPDF2", "PDF document processing"),
        ("docx", "DOCX document processing (python-docx)"),
        ("aiofiles", "Async file operations"),
    ]

    for package, description in optional_deps:
        try:
            if package == "docx":
                # python-docx imports as 'docx'
                importlib.import_module("docx")
            else:
                importlib.import_module(package)
            print(f"   ✅ {package} - {description}")
        except ImportError:
            print(f"   ⚠️  {package} - {description} (optional, not installed)")


def check_llm_providers():
    """Check available LLM providers"""
    print("\n🤖 Checking LLM providers...")

    # Check for Ollama
    try:
        import subprocess

        result = subprocess.run(["ollama", "--version"], capture_output=True, text=True)
        if result.returncode == 0:
            print(f"   ✅ Ollama available: {result.stdout.strip()}")

            # Check if Ollama is running
            try:
                list_result = subprocess.run(["ollama", "list"], capture_output=True, text=True)
                if list_result.returncode == 0:
                    print("   ✅ Ollama service is running")
                    if list_result.stdout.strip():
                        models = [
                            line.split()[0] for line in list_result.stdout.strip().split("\n")[1:]
                        ]
                        print(f"   📋 Available models: {', '.join(models)}")
                    else:
                        print("   ⚠️  No models installed. Run: python setup_llama.py --install")
                else:
                    print("   ⚠️  Ollama service not running. Run: ollama serve")
            except:
                print("   ⚠️  Ollama service not running")
        else:
            print("   ❌ Ollama command failed")
    except FileNotFoundError:
        print("   ⚠️  Ollama not installed. Run: python setup_llama.py --install")
    except Exception as e:
        print(f"   ❌ Ollama check failed: {str(e)}")

    # Check environment configuration
    env_file = Path(".env")
    if env_file.exists():
        env_content = env_file.read_text()
        if "USE_OLLAMA=true" in env_content:
            print("   ✅ System configured to use Ollama")
        elif "OPENAI_API_KEY" in env_content:
            print("   ✅ OpenAI API key configured")
        elif "ANTHROPIC_API_KEY" in env_content:
            print("   ✅ Anthropic API key configured")
        else:
            print("   ⚠️  No LLM provider configured in .env")


def check_project_structure():
    """Check if project structure is correct"""
    print("\n📁 Checking project structure...")

    required_files = [
        "pyproject.toml",
        "config.py",
        "rag_orchestrator.py",
        "example_usage.py",
        "models/submission.py",
        "rag/__init__.py",
        "rag/document_processor.py",
        "rag/vector_store.py",
        "rag/llm_interface.py",
    ]

    all_good = True

    for file_path in required_files:
        if Path(file_path).exists():
            print(f"   ✅ {file_path}")
        else:
            print(f"   ❌ {file_path} (missing)")
            all_good = False

    return all_good


def check_environment_config():
    """Check environment configuration"""
    print("\n⚙️  Checking environment configuration...")

    env_file = Path(".env")
    if env_file.exists():
        print("   ✅ .env file found")
        # Read and check for API keys (don't print actual values)
        env_content = env_file.read_text()
        if "OPENAI_API_KEY" in env_content:
            print("   ✅ OpenAI API key configured")
        elif "ANTHROPIC_API_KEY" in env_content:
            print("   ✅ Anthropic API key configured")
        else:
            print("   ⚠️  No API keys found in .env file")
    else:
        print("   ⚠️  .env file not found (create one with your API keys)")


def test_basic_imports() -> None:
    """Test basic imports from the project"""
    print("\n🧪 Testing basic imports...")

    try:
        from models.submission import AdministrativeInfo, LabSubmission

        print("   ✅ Models import successfully")
    except ImportError as e:
        print(f"   ❌ Models import failed: {e}")
        return False

    try:
        from rag.document_processor import DocumentProcessor
        from rag.llm_interface import LLMInterface
        from rag.vector_store import VectorStore

        print("   ✅ RAG components import successfully")
    except ImportError as e:
        print(f"   ❌ RAG components import failed: {e}")
        return False

    try:
        from config import settings

        print("   ✅ Configuration imports successfully")
    except ImportError as e:
        print(f"   ❌ Configuration import failed: {e}")
        return False

    return True


def main():
    """Run all checks"""
    print("🧬 Laboratory Submission RAG - Setup Verification")
    print("=" * 50)

    checks = [
        check_python_version(),
        check_uv_available(),
        check_core_dependencies(),
        check_project_structure(),
        test_basic_imports(),
    ]

    # Run optional checks
    check_optional_dependencies()
    check_llm_providers()
    check_environment_config()

    print("\n" + "=" * 50)

    if all(checks):
        print("🎉 All essential checks passed! Your setup is ready.")
        print("\nNext steps:")
        print("1. Add API keys to .env file if not done already")
        print("2. Run: python example_usage.py")
        print("3. Start processing laboratory documents!")
        return 0
    else:
        print("❌ Some checks failed. Please review the issues above.")
        print("\nCommon fixes:")
        print("- Make sure virtual environment is activated: source .venv/bin/activate")
        print("- Reinstall dependencies: uv pip install -e .")
        print("- Install email validation: uv pip install pydantic[email]")
        return 1


if __name__ == "__main__":
    sys.exit(main())
