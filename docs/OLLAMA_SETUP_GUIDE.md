# 🤖 Ollama Setup Guide for TracSeq 2.0

This guide explains how to set up and use Ollama for local LLM inference with TracSeq 2.0, eliminating the need for external API keys and providing cost-effective AI processing.

## 🎯 What is Ollama?

Ollama is a tool that allows you to run large language models (LLMs) locally on your machine. For TracSeq 2.0, this means:

- ✅ **No API Costs** - Run AI models without per-token charges
- ✅ **Data Privacy** - All processing happens locally
- ✅ **Offline Capability** - Works without internet after initial setup
- ✅ **Consistent Performance** - No external API rate limits

## 📋 Prerequisites

- **RAM**: 8GB minimum (16GB recommended for better performance)
- **Storage**: 4GB free space for model files
- **Docker**: Docker Desktop installed and running
- **Internet**: For initial model download (~2GB)

## 🚀 Quick Setup

### Option 1: Automatic Setup (Recommended)

TracSeq 2.0 includes Ollama in the Docker Compose configuration. Simply run:

```bash
# Start all services including Ollama
docker-compose up -d

# Initialize Ollama with the required model
./deploy/azure/ollama-init.sh      # Linux/macOS
./deploy/azure/ollama-init.ps1     # Windows PowerShell
```

### Option 2: Manual Setup

If you prefer to set up Ollama manually:

```bash
# Start just the Ollama service
docker-compose up -d ollama

# Wait for Ollama to be ready
docker-compose logs -f ollama

# Download the model
docker-compose exec ollama ollama pull llama3.2:3b

# Test the model
docker-compose exec ollama ollama run llama3.2:3b "Hello, how are you?"
```

## 🔧 Configuration

### Environment Variables

The following environment variables control Ollama behavior:

```env
# Ollama Configuration
USE_OLLAMA=true                              # Enable Ollama
LLM_PROVIDER=ollama                          # Set provider to Ollama
OLLAMA_MODEL=llama3.2:3b                     # Model to use
OLLAMA_PORT=11434                            # Ollama service port
OLLAMA_BASE_URL=http://ollama:11434          # Internal service URL
LLM_TEMPERATURE=0.7                          # Response creativity (0.0-1.0)
MAX_TOKENS=2048                              # Maximum response length
```

### Model Options

TracSeq 2.0 is configured to work with these models:

| Model | Size | RAM Required | Use Case |
|-------|------|--------------|----------|
| `llama3.2:3b` | ~2GB | 8GB+ | **Recommended** - Fast, efficient |
| `llama3.2:1b` | ~1GB | 4GB+ | Lightweight, basic processing |
| `llama2:7b` | ~4GB | 16GB+ | Higher quality, slower |
| `llama2:13b` | ~8GB | 32GB+ | Best quality, requires more resources |

To change models, update the `OLLAMA_MODEL` environment variable and re-run the initialization script.

## 🛠️ Manual Commands

### Basic Ollama Operations

```bash
# List installed models
docker-compose exec ollama ollama list

# Pull a new model
docker-compose exec ollama ollama pull llama3.2:1b

# Remove a model
docker-compose exec ollama ollama rm llama3.2:3b

# Run interactive chat
docker-compose exec ollama ollama run llama3.2:3b

# Check Ollama version
docker-compose exec ollama ollama --version
```

### Testing Model Integration

```bash
# Test via API
curl -X POST http://localhost:11434/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama3.2:3b",
    "prompt": "Summarize the following lab document: Sample ID: ABC123, Type: DNA, Storage: -80C",
    "stream": false
  }'

# Test via RAG service
curl -X POST http://localhost:8000/api/chat \
  -H "Content-Type: application/json" \
  -d '{
    "message": "How do I process a DNA sample?",
    "context": "laboratory"
  }'
```

## 📊 Performance Optimization

### Resource Allocation

For optimal performance, allocate resources in `docker-compose.yml`:

```yaml
ollama:
  image: ollama/ollama:latest
  deploy:
    resources:
      limits:
        memory: 8G        # Adjust based on your RAM
        cpus: '2.0'       # Use available CPU cores
      reservations:
        memory: 4G
        cpus: '1.0'
```

### Model Performance Tips

1. **Start Small**: Begin with `llama3.2:3b` and upgrade if needed
2. **Monitor RAM**: Check `docker stats` to ensure sufficient memory
3. **SSD Storage**: Store models on SSD for faster loading
4. **Keep Models Loaded**: Avoid frequent restarts to keep models in memory

## 🔍 Troubleshooting

### Common Issues

**❌ "Model not found" Error**
```bash
# Solution: Pull the model first
docker-compose exec ollama ollama pull llama3.2:3b
```

**❌ Ollama Service Won't Start**
```bash
# Check Docker resources
docker system df
docker system prune  # If needed

# Check logs
docker-compose logs ollama

# Restart service
docker-compose restart ollama
```

**❌ "Out of Memory" Error**
```bash
# Use smaller model
docker-compose exec ollama ollama pull llama3.2:1b

# Update environment variable
echo "OLLAMA_MODEL=llama3.2:1b" >> .env

# Restart RAG service
docker-compose restart rag-service
```

**❌ Slow Response Times**
```bash
# Check system resources
docker stats

# Consider upgrading to more RAM
# Or use a smaller model
```

### Health Checks

```bash
# Check if Ollama is healthy
curl -f http://localhost:11434/api/version

# Check if model is loaded
curl -X POST http://localhost:11434/api/generate \
  -d '{"model": "llama3.2:3b", "prompt": "test", "stream": false}'

# Check RAG integration
curl -f http://localhost:8000/health
```

## 🔄 Model Management

### Updating Models

```bash
# Pull latest version of current model
docker-compose exec ollama ollama pull llama3.2:3b

# Switch to different model
export OLLAMA_MODEL=llama2:7b
docker-compose exec ollama ollama pull $OLLAMA_MODEL
docker-compose restart rag-service
```

### Backup and Restore

```bash
# Backup model data
docker volume create ollama_backup
docker run --rm -v ollama_data:/source -v ollama_backup:/backup alpine tar czf /backup/models.tar.gz -C /source .

# Restore model data
docker run --rm -v ollama_backup:/backup -v ollama_data:/target alpine tar xzf /backup/models.tar.gz -C /target
```

## 🎯 Integration with TracSeq 2.0

### RAG Document Processing

Ollama integrates seamlessly with TracSeq's RAG system for:

1. **Document Analysis** - Extract structured data from lab documents
2. **Sample Classification** - Categorize samples based on content
3. **Quality Assessment** - Validate document completeness
4. **Natural Language Queries** - Search samples using plain English

### Example Workflows

**Document Upload Processing:**
```
User uploads lab document → RAG extracts text → Ollama analyzes content → 
Structured data returned → Sample records created
```

**Natural Language Search:**
```
User asks "Show me all DNA samples from last week" → 
Ollama interprets query → Database search executed → Results returned
```

## 📈 Monitoring and Metrics

### Performance Metrics

Monitor Ollama performance through:

```bash
# Resource usage
docker stats ollama

# Response times
curl -w "%{time_total}" http://localhost:11434/api/version

# Model usage logs
docker-compose logs rag-service | grep "ollama"
```

### Setting Up Alerts

```bash
# Simple health check script
#!/bin/bash
if ! curl -f http://localhost:11434/api/version; then
    echo "Ollama service is down!" | mail -s "Alert" admin@lab.com
fi
```

## 🚀 Advanced Configuration

### Custom Models

To use custom or fine-tuned models:

```bash
# Create custom Dockerfile
FROM ollama/ollama
COPY ./custom-model /models/custom-model
RUN ollama create custom-model -f /models/custom-model/Modelfile

# Update docker-compose.yml
services:
  ollama:
    build: ./ollama-custom
    # ... rest of configuration
```

### GPU Acceleration

For NVIDIA GPU support:

```yaml
# docker-compose.yml
ollama:
  image: ollama/ollama:latest
  runtime: nvidia
  environment:
    - NVIDIA_VISIBLE_DEVICES=all
  deploy:
    resources:
      reservations:
        devices:
          - driver: nvidia
            count: 1
            capabilities: [gpu]
```

## 📞 Support

If you encounter issues:

1. **Check Logs**: `docker-compose logs ollama`
2. **Resource Monitor**: `docker stats`
3. **GitHub Issues**: [TracSeq Issues](https://github.com/your-repo/tracseq2.0/issues)
4. **Ollama Documentation**: [Official Docs](https://ollama.ai/docs)

## 🔗 Related Documentation

- [RAG Integration Guide](RAG_INTEGRATION.md)
- [Docker Setup Guide](DOCKER_INTEGRATION_GUIDE.md)
- [Azure Deployment Guide](../deploy/azure/README.md)
- [Environment Configuration](../deploy/tracseq.env.example)

---

**🎉 With Ollama, TracSeq 2.0 provides powerful AI capabilities without external dependencies or ongoing costs!** 
