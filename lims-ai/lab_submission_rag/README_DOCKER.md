# 🐳 Docker Deployment - Laboratory Submission RAG System

Ultra-lightweight Laboratory Submission RAG System with Docker Compose and Ollama.

## 🚀 Quick Start

### Prerequisites
- Docker and Docker Compose installed
- 4GB+ RAM available for Ollama model
- ~3GB disk space for model storage

### Launch the System

1. **Start everything**:
   ```bash
   docker-compose -f docker-compose-simple.yml up -d
   ```

2. **Wait for model download** (first time only):
   ```bash
   # Check if model is being downloaded
   docker logs simple-rag-model-downloader
   
   # Should show: "Model downloaded successfully!"
   ```

3. **Access the web interface**:
   Open http://localhost:8000 in your browser

4. **Health check**:
   Visit http://localhost:8000/health

## 📁 Project Structure

```
lab_submission_rag/
├── docker-compose-simple.yml    # Docker orchestration
├── Dockerfile.simple            # RAG app container
├── web_interface.py             # Web UI
├── simple_lab_rag.py           # Core RAG system
├── requirements-lite.txt        # Dependencies
├── data/                        # Persistent data
├── uploads/                     # Document uploads
└── exports/                     # Export files
```

## 🖥️ Using the Web Interface

### 1. Upload Documents
- Support formats: PDF, DOCX, TXT
- Drag & drop or click to select
- Automatic processing and extraction

### 2. Ask Questions
Natural language queries like:
- "Who submitted samples yesterday?"
- "What sequencing platforms were requested?"
- "Show me all DNA samples"
- "What's the concentration of sample XYZ?"

### 3. System Monitoring
- Real-time health status
- Ollama connection status
- Document processing count

## 🔧 Configuration

### Environment Variables
```bash
# Ollama Configuration
OLLAMA_BASE_URL=http://ollama:11434
OLLAMA_MODEL=llama3.2:3b
USE_OLLAMA=true

# Data Storage
DATA_DIRECTORY=/app/data
```

### Custom Model
To use a different Ollama model:

1. Edit `docker-compose-simple.yml`:
   ```yaml
   environment:
     - OLLAMA_MODEL=llama3.2:1b  # Lighter model
   ```

2. Update model downloader:
   ```yaml
   model-downloader:
     command: >
       sh -c "
         sleep 30 &&
         ollama pull llama3.2:1b &&
         echo 'Model downloaded!'
       "
   ```

## 🔍 Available Models

| Model | Size | RAM Required | Best For |
|-------|------|--------------|----------|
| `llama3.2:1b` | 1.3GB | 2GB | Fastest |
| `llama3.2:3b` | 2.0GB | 4GB | Balanced |
| `phi3:3.8b` | 2.3GB | 4GB | Code-friendly |
| `mistral:7b` | 4.1GB | 8GB | Most capable |

## 📊 API Endpoints

### Health Check
```bash
curl http://localhost:8000/health
```

### Upload Document
```bash
curl -X POST \
  -F "file=@document.pdf" \
  http://localhost:8000/upload
```

### Query System
```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"question": "Who is the submitter?"}' \
  http://localhost:8000/query
```

### Export Data
```bash
curl http://localhost:8000/export
```

## 🛠️ Troubleshooting

### Common Issues

1. **Ollama not connecting**:
   ```bash
   # Check Ollama container logs
   docker logs simple-rag-ollama
   
   # Restart services
   docker-compose -f docker-compose-simple.yml restart
   ```

2. **Model download failed**:
   ```bash
   # Manual model download
   docker exec simple-rag-ollama ollama pull llama3.2:3b
   ```

3. **Out of memory**:
   ```bash
   # Use smaller model
   docker exec simple-rag-ollama ollama pull llama3.2:1b
   ```

4. **Permission errors**:
   ```bash
   # Fix data directory permissions
   sudo chown -R $USER:$USER ./data ./uploads ./exports
   ```

### Logs and Debugging

```bash
# View all logs
docker-compose -f docker-compose-simple.yml logs

# Follow specific service
docker-compose -f docker-compose-simple.yml logs -f simple-rag

# Check Ollama health
docker exec simple-rag-ollama ollama list
```

## ⚡ Performance Optimization

### For Limited Resources
```yaml
# In docker-compose-simple.yml
services:
  simple-rag:
    environment:
      - OLLAMA_MODEL=llama3.2:1b  # Smaller model
    deploy:
      resources:
        limits:
          memory: 2G
  
  ollama:
    deploy:
      resources:
        limits:
          memory: 3G
```

### For Production
```yaml
services:
  simple-rag:
    environment:
      - OLLAMA_MODEL=mistral:7b  # More capable
    deploy:
      replicas: 2  # Load balancing
```

## 🔄 Maintenance

### Backup Data
```bash
# Backup processed submissions
docker cp simple-rag-app:/app/data ./backup-data
docker cp simple-rag-app:/app/exports ./backup-exports
```

### Update System
```bash
# Pull latest images
docker-compose -f docker-compose-simple.yml pull

# Restart with new images
docker-compose -f docker-compose-simple.yml up -d
```

### Clean Up
```bash
# Stop all services
docker-compose -f docker-compose-simple.yml down

# Remove volumes (WARNING: deletes all data)
docker-compose -f docker-compose-simple.yml down -v

# Remove images
docker image prune -f
```

## 📈 Scaling

### Horizontal Scaling
```yaml
# docker-compose-simple.yml
services:
  simple-rag:
    deploy:
      replicas: 3
  
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    depends_on:
      - simple-rag
```

### Multiple Models
```yaml
services:
  ollama-fast:
    image: ollama/ollama
    command: ollama run llama3.2:1b
  
  ollama-smart:
    image: ollama/ollama  
    command: ollama run mistral:7b
```

## 🔐 Security Notes

- Web interface has no authentication by default
- Consider adding reverse proxy with SSL
- Ollama runs locally (no external API calls)
- Documents are processed locally

## 📚 Next Steps

1. **Add Authentication**: Integrate with OAuth or basic auth
2. **Enhanced UI**: Custom branding and advanced features  
3. **Batch Processing**: Upload multiple documents at once
4. **Advanced Analytics**: Dashboard with metrics and insights
5. **Integration**: Connect with LIMS or lab management systems

## 🆘 Support

For issues:
1. Check logs: `docker-compose logs`
2. Verify health: `curl localhost:8000/health`
3. Test Ollama: `docker exec simple-rag-ollama ollama list`
4. Review resources: `docker stats`

---

*Context improved by Giga AI* 
