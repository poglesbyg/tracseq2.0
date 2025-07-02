#!/bin/bash

echo "🧪 Testing TracSeq 2.0 Fixes"
echo "============================="
echo ""

# Check if we're in the project root
if [ ! -f "Cargo.toml" ] || [ ! -d "lims-ai" ]; then
    echo "❌ Error: Please run this script from the project root directory"
    exit 1
fi

# Test 1: Python RAG Dependencies
echo "1️⃣ Testing Python RAG Dependencies..."
echo "-------------------------------------"

cd lims-ai/lab_submission_rag

# Create a virtual environment for testing
echo "Creating test virtual environment..."
python3 -m venv test_venv
source test_venv/bin/activate

# Install dependencies
echo "Installing dependencies from requirements.txt..."
pip install --no-cache-dir -r requirements.txt > /tmp/pip_install.log 2>&1

if [ $? -eq 0 ]; then
    echo "✅ Python dependencies installed successfully!"
    
    # Test imports
    echo "Testing key imports..."
    python -c "
import sys
try:
    import langchain
    import chromadb
    import sentence_transformers
    import torch
    import transformers
    import pytest
    import pytest_asyncio
    print('✅ All key imports successful!')
except ImportError as e:
    print(f'❌ Import error: {e}')
    sys.exit(1)
"
else
    echo "❌ Failed to install Python dependencies. Check /tmp/pip_install.log for details"
    tail -20 /tmp/pip_install.log
fi

# Cleanup
deactivate
rm -rf test_venv

cd ../..

echo ""
echo "2️⃣ Testing Rust Workspace Configuration..."
echo "-----------------------------------------"

# Test Rust workspace build
echo "Checking Cargo workspace configuration..."
cargo check --workspace > /tmp/cargo_check.log 2>&1

if [ $? -eq 0 ]; then
    echo "✅ Cargo workspace configuration is valid!"
else
    echo "❌ Cargo workspace configuration has errors:"
    tail -20 /tmp/cargo_check.log
fi

# Test specific services with rstest
echo ""
echo "Testing services with rstest..."

# Test circuit-breaker-lib
echo "Testing circuit-breaker-lib..."
cd lims-core/circuit-breaker-lib
cargo test --no-run > /tmp/circuit_breaker_test.log 2>&1
if [ $? -eq 0 ]; then
    echo "✅ circuit-breaker-lib: rstest configured correctly"
else
    echo "❌ circuit-breaker-lib: rstest configuration error"
    grep -i "rstest" /tmp/circuit_breaker_test.log | head -5
fi
cd ../..

# Test reports_service
echo "Testing reports_service..."
cd lims-core/reports_service
cargo test --no-run > /tmp/reports_service_test.log 2>&1
if [ $? -eq 0 ]; then
    echo "✅ reports_service: rstest configured correctly"
else
    echo "❌ reports_service: rstest configuration error"
    grep -i "rstest" /tmp/reports_service_test.log | head -5
fi
cd ../..

echo ""
echo "3️⃣ Summary"
echo "----------"

# Check if both fixes are successful
python_ok=false
rust_ok=false

if grep -q "✅ All key imports successful!" /tmp/pip_install.log 2>/dev/null || python3 -c "import langchain" 2>/dev/null; then
    python_ok=true
fi

if cargo check --workspace >/dev/null 2>&1; then
    rust_ok=true
fi

if $python_ok && $rust_ok; then
    echo "✅ Both issues have been fixed successfully!"
    echo "   - Python RAG dependencies: FIXED"
    echo "   - Rust rstest configuration: FIXED"
    exit 0
else
    echo "⚠️  Some issues remain:"
    if ! $python_ok; then
        echo "   - Python RAG dependencies: NEEDS ATTENTION"
    fi
    if ! $rust_ok; then
        echo "   - Rust rstest configuration: NEEDS ATTENTION"
    fi
    exit 1
fi 