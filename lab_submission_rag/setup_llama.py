#!/usr/bin/env python3
"""
Setup Script for Llama Integration with Laboratory Submission RAG System

This script helps you set up and configure Ollama to run Llama models locally
for the laboratory submission RAG system.

Usage:
    python setup_llama.py --install    # Install Ollama and pull models
    python setup_llama.py --test       # Test Ollama setup
    python setup_llama.py --configure  # Configure system to use Ollama
"""

import platform
import subprocess
import sys
from pathlib import Path


def print_header():
    print("🦙 Llama Setup for Laboratory Submission RAG System")
    print("=" * 60)


def check_ollama_installed() -> bool:
    """Check if Ollama is installed and accessible"""
    try:
        result = subprocess.run(["ollama", "--version"], capture_output=True, text=True)
        if result.returncode == 0:
            print(f"✅ Ollama is installed: {result.stdout.strip()}")
            return True
        else:
            print("❌ Ollama command not found")
            return False
    except FileNotFoundError:
        print("❌ Ollama is not installed")
        return False


def install_ollama():
    """Install Ollama based on the operating system"""
    print("\n📦 Installing Ollama...")

    system = platform.system().lower()

    if system == "linux":
        print("Installing Ollama on Linux...")
        try:
            # Download and run Ollama install script
            result = subprocess.run(
                ["curl", "-fsSL", "https://ollama.ai/install.sh"], capture_output=True
            )

            if result.returncode == 0:
                # Run the install script
                install_result = subprocess.run(
                    ["sh", "-c", result.stdout.decode()], capture_output=True, text=True
                )

                if install_result.returncode == 0:
                    print("✅ Ollama installed successfully!")
                    return True
                else:
                    print(f"❌ Installation failed: {install_result.stderr}")
                    return False
            else:
                print("❌ Failed to download Ollama installer")
                return False

        except Exception as e:
            print(f"❌ Error installing Ollama: {str(e)}")
            print("\nManual installation:")
            print("curl -fsSL https://ollama.ai/install.sh | sh")
            return False

    elif system == "darwin":  # macOS
        print("For macOS, please install Ollama manually:")
        print("1. Download from https://ollama.ai/download")
        print("2. Or use Homebrew: brew install ollama")
        return False

    elif system == "windows":
        print("For Windows, please install Ollama manually:")
        print("1. Download from https://ollama.ai/download")
        print("2. Run the installer")
        return False

    else:
        print(f"❌ Unsupported operating system: {system}")
        return False


def start_ollama_service():
    """Start the Ollama service"""
    print("\n🚀 Starting Ollama service...")
    try:
        # Start Ollama in the background
        subprocess.Popen(["ollama", "serve"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        print("✅ Ollama service started")
        return True
    except Exception as e:
        print(f"❌ Failed to start Ollama service: {str(e)}")
        return False


def check_ollama_running() -> bool:
    """Check if Ollama service is running"""
    try:
        result = subprocess.run(["ollama", "list"], capture_output=True, text=True)
        return result.returncode == 0
    except:
        return False


def pull_llama_models():
    """Pull recommended Llama models"""
    print("\n📥 Pulling Llama models...")

    # Recommended models for laboratory document processing
    models = [
        ("llama3.1:8b", "Llama 3.1 8B - Good balance of speed and quality"),
        ("llama3.1:13b", "Llama 3.1 13B - Higher quality, slower (optional)"),
    ]

    pulled_models = []

    for model, description in models:
        print(f"\n📦 Pulling {model} - {description}")
        try:
            result = subprocess.run(["ollama", "pull", model], capture_output=True, text=True)
            if result.returncode == 0:
                print(f"✅ Successfully pulled {model}")
                pulled_models.append(model)
            else:
                print(f"❌ Failed to pull {model}: {result.stderr}")
        except Exception as e:
            print(f"❌ Error pulling {model}: {str(e)}")

    return pulled_models


def test_llama_model(model: str = "llama3.1:8b") -> bool:
    """Test a Llama model with a simple laboratory-related query"""
    print(f"\n🧪 Testing {model} with laboratory document processing...")

    test_prompt = """
    Extract the following information from this laboratory submission:
    
    "Dr. John Smith from the University Lab submitted blood samples for WGS analysis. 
    Contact: john.smith@university.edu, Phone: 555-0123. 
    Project: PROJ-2024-LAB-001. Sample volume: 5mL, stored at -80°C."
    
    Return JSON format with: submitter_name, email, project, sample_type, volume.
    """

    try:
        result = subprocess.run(
            ["ollama", "generate", model, test_prompt], capture_output=True, text=True, timeout=60
        )

        if result.returncode == 0:
            print("✅ Model test successful!")
            print("📄 Sample response:")
            print(result.stdout[:300] + "..." if len(result.stdout) > 300 else result.stdout)
            return True
        else:
            print(f"❌ Model test failed: {result.stderr}")
            return False

    except subprocess.TimeoutExpired:
        print("❌ Model test timed out (model might be too slow)")
        return False
    except Exception as e:
        print(f"❌ Error testing model: {str(e)}")
        return False


def configure_rag_system():
    """Configure the RAG system to use Ollama by default"""
    print("\n⚙️  Configuring RAG system for Ollama...")

    env_file = Path(".env")

    # Read existing .env or create new one
    env_content = {}
    if env_file.exists():
        with open(env_file) as f:
            for line in f:
                line = line.strip()
                if line and "=" in line and not line.startswith("#"):
                    key, value = line.split("=", 1)
                    env_content[key] = value

    # Add Ollama configuration
    env_content["USE_OLLAMA"] = "true"
    env_content["OLLAMA_MODEL"] = "llama3.1:8b"
    env_content["OLLAMA_BASE_URL"] = "http://localhost:11434"

    # Write updated .env file
    with open(env_file, "w") as f:
        f.write("# Laboratory Submission RAG System Configuration\n")
        f.write("# Generated by setup_llama.py\n\n")

        f.write("# Ollama Configuration (Local Llama Models)\n")
        f.write(f"USE_OLLAMA={env_content['USE_OLLAMA']}\n")
        f.write(f"OLLAMA_MODEL={env_content['OLLAMA_MODEL']}\n")
        f.write(f"OLLAMA_BASE_URL={env_content['OLLAMA_BASE_URL']}\n\n")

        f.write("# Cloud LLM APIs (optional fallbacks)\n")
        for key, value in env_content.items():
            if key not in ["USE_OLLAMA", "OLLAMA_MODEL", "OLLAMA_BASE_URL"]:
                f.write(f"{key}={value}\n")

    print("✅ Configuration updated in .env file")
    print("🦙 System configured to use Ollama by default")


def main():
    import argparse

    parser = argparse.ArgumentParser(description="Setup Llama for RAG system")
    parser.add_argument("--install", action="store_true", help="Install Ollama and models")
    parser.add_argument("--test", action="store_true", help="Test Ollama setup")
    parser.add_argument("--configure", action="store_true", help="Configure system for Ollama")
    parser.add_argument("--all", action="store_true", help="Run complete setup")

    args = parser.parse_args()

    print_header()

    if args.all or args.install:
        # Full installation process
        if not check_ollama_installed():
            if not install_ollama():
                sys.exit(1)

        if not check_ollama_running():
            start_ollama_service()

        # Wait a moment for service to start
        import time

        time.sleep(2)

        pulled_models = pull_llama_models()
        if not pulled_models:
            print("❌ No models were successfully pulled")
            sys.exit(1)

        # Test the first pulled model
        if pulled_models and not test_llama_model(pulled_models[0]):
            print("❌ Model testing failed")
            sys.exit(1)

        configure_rag_system()

    elif args.test:
        # Test existing setup
        if not check_ollama_installed():
            print("❌ Ollama not installed. Run with --install first.")
            sys.exit(1)

        if not check_ollama_running():
            print("❌ Ollama service not running. Starting...")
            start_ollama_service()

        test_llama_model()

    elif args.configure:
        # Just configure the system
        configure_rag_system()

    else:
        # Show status and instructions
        print("🔍 Checking current setup...")

        if check_ollama_installed():
            if check_ollama_running():
                print("✅ Ollama is running")

                # List available models
                try:
                    result = subprocess.run(["ollama", "list"], capture_output=True, text=True)
                    if result.stdout.strip():
                        print("\n📋 Available models:")
                        print(result.stdout)
                    else:
                        print("📋 No models found. Run with --install to pull models.")
                except:
                    pass
            else:
                print("⚠️  Ollama installed but not running")
                print("Run: ollama serve (in another terminal)")

        print("\n" + "=" * 60)
        print("Usage options:")
        print("  python setup_llama.py --all        # Complete setup")
        print("  python setup_llama.py --install    # Install and pull models")
        print("  python setup_llama.py --test       # Test current setup")
        print("  python setup_llama.py --configure  # Configure .env file")


if __name__ == "__main__":
    main()
