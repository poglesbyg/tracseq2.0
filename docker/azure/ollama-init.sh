#!/bin/bash

# 🤖 Ollama Model Initialization Script for TracSeq 2.0
# This script downloads and initializes the required model for local deployment

set -e

MODEL_NAME=${OLLAMA_MODEL:-"llama3.2:3b"}
OLLAMA_URL=${OLLAMA_BASE_URL:-"http://localhost:11434"}

echo "🤖 Initializing Ollama for TracSeq 2.0"
echo "======================================"
echo ""

# Function to check if Ollama is ready
wait_for_ollama() {
    echo "⏳ Waiting for Ollama service to be ready..."
    local max_attempts=30
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s "$OLLAMA_URL/api/version" > /dev/null 2>&1; then
            echo "✅ Ollama service is ready!"
            break
        fi
        
        echo "   Attempt $attempt/$max_attempts - Ollama not ready yet..."
        sleep 10
        ((attempt++))
    done
    
    if [ $attempt -gt $max_attempts ]; then
        echo "❌ Ollama service failed to start after $max_attempts attempts"
        echo "Please check if the Ollama container is running:"
        echo "   docker-compose ps ollama"
        echo "   docker-compose logs ollama"
        exit 1
    fi
}

# Function to download model
download_model() {
    echo "📥 Downloading model: $MODEL_NAME"
    echo "   This may take a few minutes depending on your internet connection..."
    echo ""
    
    # Pull the model
    if curl -X POST "$OLLAMA_URL/api/pull" \
        -H "Content-Type: application/json" \
        -d "{\"name\": \"$MODEL_NAME\"}" \
        --max-time 1800; then  # 30 minute timeout
        echo ""
        echo "✅ Model $MODEL_NAME downloaded successfully!"
    else
        echo ""
        echo "❌ Failed to download model $MODEL_NAME"
        echo "Please check your internet connection and try again."
        exit 1
    fi
}

# Function to test model
test_model() {
    echo ""
    echo "🧪 Testing model functionality..."
    
    local test_prompt="Hello, this is a test. Please respond with 'Model is working correctly.'"
    
    local response=$(curl -s -X POST "$OLLAMA_URL/api/generate" \
        -H "Content-Type: application/json" \
        -d "{
            \"model\": \"$MODEL_NAME\",
            \"prompt\": \"$test_prompt\",
            \"stream\": false
        }" | grep -o '"response":"[^"]*"' | cut -d'"' -f4 | head -1)
    
    if [ -n "$response" ]; then
        echo "✅ Model test successful!"
        echo "   Response: $response"
        echo ""
        echo "🎉 Ollama is ready for TracSeq 2.0!"
        echo ""
        echo "📊 Model Information:"
        echo "   Model: $MODEL_NAME"
        echo "   Endpoint: $OLLAMA_URL"
        echo "   Status: Ready for RAG processing"
    else
        echo "⚠️  Model downloaded but test failed"
        echo "   The model may still be initializing"
        echo "   RAG service will automatically retry when ready"
    fi
}

# Function to show usage information
show_usage() {
    echo ""
    echo "🔧 Usage Information:"
    echo "==================="
    echo ""
    echo "Environment Variables:"
    echo "   OLLAMA_MODEL      - Model to download (default: llama3.2:3b)"
    echo "   OLLAMA_BASE_URL   - Ollama service URL (default: http://localhost:11434)"
    echo ""
    echo "Docker Compose:"
    echo "   Start services:   docker-compose up -d"
    echo "   View logs:        docker-compose logs ollama"
    echo "   Stop services:    docker-compose down"
    echo ""
    echo "Manual Model Management:"
    echo "   List models:      ollama list"
    echo "   Pull model:       ollama pull $MODEL_NAME"
    echo "   Remove model:     ollama rm $MODEL_NAME"
    echo ""
}

# Main execution
main() {
    echo "Configuration:"
    echo "   Model: $MODEL_NAME"
    echo "   Ollama URL: $OLLAMA_URL"
    echo ""
    
    wait_for_ollama
    download_model
    test_model
    show_usage
    
    echo "✨ Ollama initialization complete!"
    echo "   Your TracSeq 2.0 system is ready for AI-powered document processing!"
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "Ollama Model Initialization Script for TracSeq 2.0"
        echo ""
        echo "This script downloads and tests the required LLM model for local inference."
        echo ""
        show_usage
        exit 0
        ;;
    --test-only)
        echo "🧪 Testing existing model only..."
        wait_for_ollama
        test_model
        exit 0
        ;;
    *)
        main
        ;;
esac 
