# Docker Quick Start Guide for RAG Chatbot 🐳

This is a streamlined guide to get the RAG chatbot running with Docker in just a few commands.

## Prerequisites

- Docker and Docker Compose installed
- Lab Manager system ready to run

## Quick Setup (5 minutes)

### 1. Start RAG System with Docker

```bash
# Navigate to RAG system directory
cd lab_submission_rag/app

# Option A: Use with local Ollama (recommended)
# First install Ollama if not already installed
curl -fsSL https://ollama.ai/install.sh | sh
ollama pull llama3.1:8b

# Create environment file for Ollama
cat > .env << EOF
USE_OLLAMA=true
OLLAMA_MODEL=llama3.1:8b
OLLAMA_BASE_URL=http://localhost:11434
EOF

# Option B: Use with OpenAI/Anthropic (requires API keys)
cat > .env << EOF
OPENAI_API_KEY=your_api_key_here
# OR
# ANTHROPIC_API_KEY=your_api_key_here
EOF

# Start the RAG service
docker-compose up -d

# Verify it's running
docker-compose ps
curl http://localhost:8000/health
```

### 2. Start Lab Manager

```bash
# Navigate to lab manager directory
cd ../../lab_manager

# Start lab manager (choose one)
./scripts/run.sh                    # Production mode
# OR
docker-compose up frontend-dev dev db  # Development mode
```

### 3. Test the Chatbot

1. Open browser to `http://localhost:5173` (dev) or `http://localhost` (prod)
2. Look for blue floating chat button in bottom-right corner
3. Click and ask: "How do I submit a new sample?"

## Common Commands

```bash
# View RAG service logs
docker-compose logs -f rag-service

# Restart RAG service
docker-compose restart rag-service

# Stop RAG service
docker-compose down

# Update RAG service (after code changes)
docker-compose up --build -d
```

## Troubleshooting

### RAG Service Won't Start
```bash
# Check logs for errors
docker-compose logs rag-service

# Common issues:
# 1. Port 8000 already in use - stop other services
# 2. Missing .env file - create one as shown above
# 3. Invalid API keys - check your .env configuration
```

### Chatbot Shows Connection Error
```bash
# Verify RAG service is responding
curl http://localhost:8000/health

# Check if lab manager can reach RAG service
curl http://localhost:3000/api/samples/rag/status

# If both fail, restart RAG service
docker-compose restart rag-service
```

### Performance Issues
```bash
# Check resource usage
docker stats --no-stream

# For local Ollama: ensure sufficient RAM (8GB+ recommended)
# For cloud APIs: check API rate limits
```

## Next Steps

1. **Upload Lab Documents**: Add your laboratory documents to improve chatbot responses
2. **Customize Questions**: Edit suggested questions in the chatbot interface
3. **Configure Models**: Try different Ollama models for better performance
4. **Monitor Usage**: Use `docker-compose logs` to monitor chatbot usage

## File Structure

```
lab_submission_rag/app/
├── docker-compose.yml     # Docker service configuration
├── Dockerfile            # RAG service container definition
├── .env                  # Environment configuration (you create this)
├── uploads/              # Document uploads (persistent)
├── exports/              # Exported data (persistent)
├── logs/                 # Application logs (persistent)
└── data/                 # Vector store data (persistent)
```

## Support

- Check the main [RAG_CHATBOT_SETUP.md](RAG_CHATBOT_SETUP.md) for detailed configuration
- Use [test_chatbot.md](test_chatbot.md) for comprehensive testing
- Monitor logs with `docker-compose logs -f rag-service`

*Context improved by Giga AI* 
