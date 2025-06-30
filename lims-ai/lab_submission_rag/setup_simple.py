#!/usr/bin/env python3
"""
Ultra-Lightweight Setup Script for Laboratory Submission RAG System

This script helps you get started quickly with Ollama (local LLM) or OpenAI fallback.
Supports automatic Ollama installation and model downloading.
"""

import os
import platform
import shutil
import subprocess
import sys
from pathlib import Path


def check_python_version():
    """Check if Python version is compatible"""
    if sys.version_info < (3, 9):
        print("❌ Python 3.9 or higher is required")
        print(f"   Current version: {sys.version}")
        return False
    print(f"✅ Python version: {sys.version.split()[0]}")
    return True


def check_ollama_installed():
    """Check if Ollama is installed"""
    return shutil.which("ollama") is not None


def install_ollama():
    """Install Ollama based on the operating system"""
    print("\n🦙 Installing Ollama...")

    system = platform.system().lower()

    try:
        if system == "windows":
            print("Please install Ollama manually:")
            print("1. Download from: https://ollama.ai/download")
            print("2. Run the installer")
            print("3. Restart this script")
            return False

        elif system == "darwin":  # macOS
            # Try brew first, then curl
            if shutil.which("brew"):
                subprocess.run(["brew", "install", "ollama"], check=True)
            else:
                subprocess.run(
                    ["curl", "-fsSL", "https://ollama.ai/install.sh"], shell=True, check=True
                )

        elif system == "linux":
            # Use the official install script
            subprocess.run(
                ["curl", "-fsSL", "https://ollama.ai/install.sh"], shell=True, check=True
            )

        else:
            print(f"❌ Unsupported system: {system}")
            return False

        print("✅ Ollama installed successfully")
        return True

    except subprocess.CalledProcessError as e:
        print(f"❌ Ollama installation failed: {e}")
        print("Please install manually from: https://ollama.ai/download")
        return False


def check_ollama_running():
    """Check if Ollama service is running"""
    try:
        result = subprocess.run(["ollama", "list"], capture_output=True, text=True, timeout=10)
        return result.returncode == 0
    except (subprocess.TimeoutExpired, FileNotFoundError):
        return False


def start_ollama():
    """Start Ollama service"""
    print("\n🦙 Starting Ollama service...")

    system = platform.system().lower()

    try:
        if system == "windows":
            # On Windows, Ollama usually runs as a service
            subprocess.Popen(["ollama", "serve"], creationflags=subprocess.CREATE_NEW_CONSOLE)
        else:
            # On Unix systems, start in background
            subprocess.Popen(
                ["ollama", "serve"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
            )

        # Wait a moment for service to start
        import time

        time.sleep(3)

        if check_ollama_running():
            print("✅ Ollama service started")
            return True
        else:
            print("⚠️  Ollama service may need manual start")
            return False

    except Exception as e:
        print(f"❌ Failed to start Ollama: {e}")
        return False


def download_ollama_model(model="llama3.2:3b"):
    """Download the specified Ollama model"""
    print(f"\n🦙 Downloading model: {model}")
    print("This may take a few minutes depending on your connection...")

    try:
        result = subprocess.run(
            ["ollama", "pull", model],
            capture_output=True,
            text=True,
            timeout=600,  # 10 minutes timeout
        )

        if result.returncode == 0:
            print(f"✅ Model {model} downloaded successfully")
            return True
        else:
            print(f"❌ Failed to download model: {result.stderr}")
            return False

    except subprocess.TimeoutExpired:
        print("❌ Model download timed out")
        return False
    except Exception as e:
        print(f"❌ Model download failed: {e}")
        return False


def setup_ollama():
    """Complete Ollama setup process"""
    print("\n🦙 Setting up Ollama (Local LLM)...")

    # Check if Ollama is installed
    if not check_ollama_installed():
        print("Ollama not found. Installing...")
        if not install_ollama():
            return False
    else:
        print("✅ Ollama is installed")

    # Check if Ollama is running
    if not check_ollama_running():
        print("Ollama service not running. Starting...")
        if not start_ollama():
            print("⚠️  Please start Ollama manually: ollama serve")
            return False
    else:
        print("✅ Ollama service is running")

    # Download the model
    if not download_ollama_model("llama3.2:3b"):
        print("⚠️  Model download failed. You can try manually:")
        print("   ollama pull llama3.2:3b")
        return False

    print("✅ Ollama setup complete!")
    return True


def install_dependencies():
    """Install required dependencies"""
    print("\n📦 Installing dependencies...")

    try:
        # Install from requirements-lite.txt
        subprocess.check_call(
            [sys.executable, "-m", "pip", "install", "-r", "requirements-lite.txt"]
        )
        print("✅ Dependencies installed successfully")
        return True
    except subprocess.CalledProcessError as e:
        print(f"❌ Failed to install dependencies: {e}")
        return False
    except FileNotFoundError:
        print("❌ requirements-lite.txt not found")
        return False


def check_llm_setup():
    """Check if LLM setup is configured (Ollama or OpenAI)"""
    ollama_available = check_ollama_installed() and check_ollama_running()
    openai_available = bool(os.getenv("OPENAI_API_KEY"))

    if ollama_available:
        print("✅ Ollama is available (local LLM)")
        return True, "ollama"
    elif openai_available:
        print("✅ OpenAI API key found (fallback mode)")
        return True, "openai"
    else:
        print("⚠️  No LLM configured")
        print("\nOptions:")
        print("1. Install Ollama (recommended - free, private):")
        print("   Run: python setup_simple.py --ollama")
        print("2. Or configure OpenAI API key:")
        print("   Get key from: https://platform.openai.com/api-keys")
        print("   Create .env file with: OPENAI_API_KEY=your_key")
        return False, "none"


def create_directories():
    """Create necessary directories"""
    print("\n📁 Creating directories...")

    directories = ["data", "demo", "uploads"]

    for dir_name in directories:
        Path(dir_name).mkdir(exist_ok=True)
        print(f"✅ Created directory: {dir_name}")


def test_imports() -> None:
    """Test if all required packages can be imported"""
    print("\n🔍 Testing imports...")

    required_packages = [
        ("ollama", "Ollama API client"),
        ("chromadb", "ChromaDB vector database"),
        ("sentence_transformers", "Sentence Transformers"),
        ("pypdf", "PDF processing"),
        ("docx", "Word document processing"),
        ("pydantic", "Data validation"),
        ("dotenv", "Environment variables"),
    ]

    optional_packages = [("openai", "OpenAI API client (fallback)")]

    all_good = True

    for package, description in required_packages:
        try:
            __import__(package)
            print(f"✅ {description}")
        except ImportError:
            print(f"❌ {description} - not installed")
            all_good = False

    # Test optional packages
    for package, description in optional_packages:
        try:
            __import__(package)
            print(f"✅ {description}")
        except ImportError:
            print(f"⚠️  {description} - optional")

    return all_good


def run_demo():
    """Ask if user wants to run the demo"""
    if input("\n🧬 Would you like to run the demo? (y/n): ").lower().startswith("y"):
        try:
            print("\nRunning demo...")
            subprocess.run([sys.executable, "simple_lab_rag.py"])
        except KeyboardInterrupt:
            print("\nDemo interrupted by user")
        except Exception as e:
            print(f"Demo failed: {e}")


def main():
    """Main setup function"""
    # Check for command line arguments
    args = sys.argv[1:]
    setup_ollama_only = "--ollama" in args

    print("🧬 Ultra-Lightweight Laboratory Submission RAG System - Setup")
    print("🦙 Powered by Ollama (Local LLM)")
    print("=" * 70)

    # Check Python version
    if not check_python_version():
        return 1

    # Install dependencies
    if not install_dependencies():
        return 1

    # Test imports
    if not test_imports():
        print("\n❌ Some packages failed to import. Try:")
        print("   pip install -r requirements-lite.txt")
        return 1

    # Create directories
    create_directories()

    # Setup LLM (Ollama preferred)
    if setup_ollama_only:
        print("\n🦙 Setting up Ollama only...")
        ollama_setup = setup_ollama()
        llm_configured = ollama_setup
        llm_type = "ollama" if ollama_setup else "none"
    else:
        llm_configured, llm_type = check_llm_setup()

        if not llm_configured:
            # Try to setup Ollama automatically
            print("\n🦙 Attempting to setup Ollama automatically...")
            if setup_ollama():
                llm_configured, llm_type = True, "ollama"

    print("\n" + "=" * 70)

    if llm_configured:
        print("🎉 Setup Complete!")

        if llm_type == "ollama":
            print("\n✅ Using Ollama (Local LLM):")
            print("   🔒 Private - your data stays local")
            print("   💰 Free - no API costs")
            print("   🚀 Fast - no network latency")
        elif llm_type == "openai":
            print("\n✅ Using OpenAI (Fallback mode):")
            print("   ☁️  Cloud-based processing")
            print("   💳 API costs apply")

        print("\n🚀 Ready to run:")
        print("   python simple_lab_rag.py")

        run_demo()

    else:
        print("⚠️  Setup incomplete - no LLM configured")
        print("\nNext steps:")
        print("1. For Ollama (recommended): python setup_simple.py --ollama")
        print("2. For OpenAI: set OPENAI_API_KEY environment variable")
        print("3. Then run: python simple_lab_rag.py")

    print("\n📝 Usage:")
    print("   1. Put your lab documents in the 'uploads' folder")
    print("   2. Run: python simple_lab_rag.py")
    print("   3. The system will process documents and allow queries")
    print("\n🔧 Commands:")
    print("   python setup_simple.py --ollama    # Setup Ollama only")
    print("   python test_simple.py              # Test installation")

    return 0 if llm_configured else 1


if __name__ == "__main__":
    exit(main())
