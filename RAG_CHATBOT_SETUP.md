# RAG Chatbot Setup Guide 🤖

This guide will help you set up and use the RAG-powered chatbot that assists users with the Lab Manager system.

## Overview

The RAG chatbot is an intelligent assistant that helps users navigate and use the lab_manager system. It leverages the existing RAG (Retrieval-Augmented Generation) system to answer questions about:

- Laboratory submission processes
- Sample management procedures
- Storage requirements and protocols
- Sequencing workflows
- System navigation and help

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   React Frontend│    │   Rust Backend  │    │ Python RAG API  │
│                 │    │                 │    │                 │
│  ChatBot.tsx    │ => │ /api/samples/   │ => │ /query          │
│  ChatBotFloat   │    │ rag/query       │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Prerequisites

1. **Lab Manager System**: Rust backend + React frontend
2. **RAG System**: Python-based RAG system running on port 8000
3. **LLM Provider**: Either local Ollama or cloud API (OpenAI/Anthropic)

## Setup Instructions

### 1. Start RAG System

You can run the RAG system either with Docker (recommended) or directly with Python.

#### Option A: Docker Deployment (Recommended)

Navigate to the RAG system directory and start with Docker:

```bash
cd lab_submission_rag/app

# Create .env file for configuration (optional)
cat > .env << EOF
# LLM Provider Configuration (choose one)
USE_OLLAMA=true
OLLAMA_MODEL=llama3.1:8b
OLLAMA_BASE_URL=http://localhost:11434

# OR use cloud APIs
# OPENAI_API_KEY=your_openai_api_key_here
# ANTHROPIC_API_KEY=your_anthropic_api_key_here

# Optional: Custom settings
CHUNK_SIZE=1000
VECTOR_DB_PATH=./data/vector_store
EOF

# Start the RAG service with Docker Compose
docker-compose up -d

# View logs to ensure it's running
docker-compose logs -f rag-service
```

#### Option B: Direct Python Deployment

```bash
cd lab_submission_rag/app
source .venv/bin/activate
python -m uvicorn api.main:app --host 0.0.0.0 --port 8000
```

### 2. Verify RAG System Health

Check that the RAG system is operational:

```bash
curl http://localhost:8000/health
curl http://localhost:8000/system-info
```

### 3. Start Lab Manager

The chatbot is already integrated into the Lab Manager frontend. Simply start the system:

```bash
cd lab_manager
./scripts/run.sh
```

Or for development:

```bash
docker-compose up frontend-dev dev db
```

### 4. Docker Management Commands

#### Useful Docker Commands for RAG System

```bash
# View running containers
docker-compose ps

# Stop the RAG service
docker-compose down

# Restart the RAG service
docker-compose restart rag-service

# View logs
docker-compose logs rag-service

# Follow logs in real-time
docker-compose logs -f rag-service

# Rebuild and restart (after code changes)
docker-compose up --build -d

# Access container shell for debugging
docker-compose exec rag-service bash

# Remove containers and volumes (full reset)
docker-compose down -v
```

#### Volume Management

The Docker setup creates persistent volumes for:
- `./uploads` - Uploaded documents
- `./exports` - Exported data
- `./logs` - Application logs
- `./data` - Vector store and embeddings

These directories persist between container restarts, preserving your data and documents.

### 5. Access the Chatbot

Once the system is running:

1. Visit the Lab Manager frontend: `http://localhost:5173` (dev) or `http://localhost` (prod)
2. Look for the floating blue chat button in the bottom-right corner
3. Click it to open the chatbot interface

## Features

### 🎯 **Intelligent Assistance**
- Natural language queries about lab procedures
- Context-aware responses based on processed documents
- Suggested questions to help users get started

### 💬 **User-Friendly Interface**
- Clean, modern chat interface
- Typing indicators and smooth animations
- Minimizable window that doesn't interfere with workflow
- Mobile-responsive design

### 🔍 **Lab-Specific Knowledge**
- Information about sample submission processes
- Storage requirements and conditions
- Sequencing platform details
- Barcode and tracking procedures

## Usage Examples

Here are some questions you can ask the chatbot:

### Sample Management
- "How do I submit a new sample?"
- "What are the storage requirements for DNA samples?"
- "How do I generate barcodes for my samples?"
- "What file formats are supported for submissions?"

### Workflow Help
- "How do I track my submission status?"
- "What sequencing platforms are available?"
- "How do I set up a new project?"
- "Where can I find my sample results?"

### Technical Questions
- "What are the quality requirements for samples?"
- "How do I handle failed extractions?"
- "What analysis pipelines are supported?"
- "How do I export my data?"

## Troubleshooting

### Chatbot Won't Open
1. Check browser console for JavaScript errors
2. Verify the Lab Manager frontend is running
3. Ensure you're on a supported page

### "Connection Issues" Error
1. **Check RAG System**: Ensure RAG service is running
2. **Check Backend Proxy**: Verify lab_manager backend is proxying to RAG system
3. **Network Issues**: Check firewall/network configuration

#### Docker Deployment Troubleshooting

```bash
# Check if RAG container is running
docker-compose ps

# Check container logs for errors
docker-compose logs rag-service

# Test RAG system directly
curl http://localhost:8000/health

# Test through lab_manager proxy
curl http://localhost:3000/api/samples/rag/status

# Restart RAG service if needed
docker-compose restart rag-service
```

#### Python Deployment Troubleshooting

```bash
# Test RAG system directly
curl http://localhost:8000/health

# Check if Python process is running
ps aux | grep uvicorn

# Check Python logs
tail -f lab_submission_rag/app/logs/app.log
```

### Slow Response Times
1. **Local LLM**: If using Ollama, ensure sufficient system resources
2. **Cloud API**: Check API key limits and quotas
3. **Document Index**: Large document collections may slow responses

### Poor Answer Quality
1. **Upload Documents**: The RAG system learns from uploaded lab documents
2. **Check Confidence**: Low confidence scores indicate unclear documents
3. **Improve Queries**: Be specific in your questions

## Configuration

### Docker Environment Configuration

The RAG system can be configured using environment variables in a `.env` file:

```bash
# Create .env file in lab_submission_rag/app/
cd lab_submission_rag/app

# Example .env configuration
cat > .env << EOF
# === LLM Provider Configuration ===
# Choose ONE of the following options:

# Option 1: Local Ollama (Recommended for privacy)
USE_OLLAMA=true
OLLAMA_MODEL=llama3.1:8b
OLLAMA_BASE_URL=http://localhost:11434

# Option 2: OpenAI
# OPENAI_API_KEY=your_openai_api_key_here
# LLM_PROVIDER=openai
# MODEL_NAME=gpt-3.5-turbo

# Option 3: Anthropic Claude
# ANTHROPIC_API_KEY=your_anthropic_api_key_here
# LLM_PROVIDER=anthropic
# MODEL_NAME=claude-3-sonnet-20240229

# === Document Processing ===
CHUNK_SIZE=1000
CHUNK_OVERLAP=200

# === Storage Paths ===
VECTOR_STORE_PATH=./data/vector_store
UPLOAD_DIR=./uploads
EXPORT_DIR=./exports
LOG_DIR=./logs

# === Logging ===
LOG_LEVEL=INFO
EOF
```

### Setting up Local Ollama (Recommended)

For privacy and cost-effectiveness, use local Ollama:

```bash
# Install Ollama (if not already installed)
curl -fsSL https://ollama.ai/install.sh | sh

# Pull a suitable model
ollama pull llama3.1:8b

# Verify Ollama is running
curl http://localhost:11434/api/tags

# Update .env to use Ollama
echo "USE_OLLAMA=true" >> lab_submission_rag/app/.env
echo "OLLAMA_MODEL=llama3.1:8b" >> lab_submission_rag/app/.env
```

### Customizing Suggested Questions

Edit the `suggestedQuestions` array in `ChatBot.tsx`:

```typescript
const suggestedQuestions = [
  "How do I submit a new sample?",
  "What are the storage requirements for DNA samples?",
  // Add your custom questions here
];
```

### Styling

The chatbot uses Tailwind CSS classes. Key styling can be modified in:
- `ChatBot.tsx`: Main chat interface
- `ChatBotFloat.tsx`: Floating action button

### API Configuration

The chatbot connects through the existing lab_manager API at:
- Endpoint: `/api/samples/rag/query`
- Method: `POST`
- Payload: `{ query: string }`

## Advanced Features

### Context Awareness
The chatbot can reference previously processed documents and submissions, making it context-aware for your specific lab environment.

### Integration with Workflows
The chatbot can guide users through complex workflows like:
- Multi-step sample submission processes
- Quality control procedures
- Report generation workflows

### Knowledge Base Growth
As you upload more laboratory documents to the RAG system, the chatbot becomes more intelligent and can answer more specific questions about your lab's procedures.

## Development

### Adding New Features

1. **Frontend Changes**: Modify `ChatBot.tsx` or `ChatBotFloat.tsx`
2. **Backend Integration**: Extend the RAG handlers in `rag_enhanced_handlers.rs`
3. **RAG System**: Add new capabilities to the Python RAG API

### Testing

Test the chatbot integration:

```bash
# Test RAG endpoint directly
curl -X POST http://localhost:3000/api/samples/rag/query \
  -H "Content-Type: application/json" \
  -d '{"query": "How do I submit a sample?"}'
```

## Security Considerations

1. **API Access**: The chatbot uses existing authentication through the lab_manager
2. **Data Privacy**: Queries are processed through your local RAG system
3. **Access Control**: Users need lab_manager access to use the chatbot

## Support

If you encounter issues:

1. Check the browser console for errors
2. Verify all services are running (Lab Manager + RAG system)
3. Test the RAG system endpoints directly
4. Review the setup steps above

## Future Enhancements

Potential improvements for the chatbot:
- File upload capability for direct document analysis
- Integration with sample tracking workflows
- Automated assistance for common tasks
- Multi-language support
- Voice input/output capabilities

---

*Context improved by Giga AI* 
