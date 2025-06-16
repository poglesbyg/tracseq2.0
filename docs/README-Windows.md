# TracSeq 2.0 - Windows Setup & Usage Guide

This guide provides Windows-specific instructions for running TracSeq 2.0.

## 🚀 Quick Start

**Double-click `start-tracseq.cmd`** for the easiest setup experience!

Or use one of the methods below:

### Method 1: PowerShell (Recommended)
```powershell
.\run.ps1 start-dev
```

### Method 2: Windows Batch
```cmd
run.bat start-dev
```

### Method 3: Git Bash (if installed)
```bash
./run.sh start-dev
```

## 📋 Prerequisites

### Required
- **Docker Desktop for Windows** - [Download here](https://www.docker.com/products/docker-desktop)
- **Windows 10/11** with WSL2 enabled

### Optional (for enhanced experience)
- **PowerShell 5.1+** (usually pre-installed)
- **Git for Windows** (includes Git Bash)
- **Ollama** (for local AI models) - Auto-installed by scripts

## 🔧 Windows-Specific Setup

### 1. Docker Desktop Configuration
- Install Docker Desktop and enable WSL2 integration
- In Docker Desktop settings:
  - Enable "Use WSL 2 based engine"
  - Enable integration with your WSL distribution
  - Allocate at least 4GB RAM to Docker

### 2. PowerShell Execution Policy (if needed)
If you get PowerShell execution policy errors:
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### 3. Windows Defender / Antivirus
Add exclusions for:
- Docker Desktop installation folder
- TracSeq 2.0 project folder
- Docker volumes (usually `C:\ProgramData\Docker`)

## 🎮 Usage Commands

### Starting Services
```powershell
# Development mode (hot reload)
.\run.ps1 start-dev

# Production mode
.\run.ps1 start-prod

# Quick start with auto-open browsers
.\run.ps1 start-dev
```

### Managing Services
```powershell
# Check status
.\run.ps1 status

# View logs
.\run.ps1 logs lab-manager
.\run.ps1 logs rag

# Stop all services
.\run.ps1 stop

# Restart services
.\run.ps1 restart-dev
```

### Ollama AI Models
```powershell
# Install Ollama
.\run.ps1 install-ollama

# Download AI model
.\run.ps1 pull-model llama2

# Start Ollama service
.\run.ps1 start-ollama
```

### Maintenance
```powershell
# Rebuild all containers
.\run.ps1 rebuild

# Clean up Docker resources
.\run.ps1 clean
```

## 🌐 Access URLs

### Development Mode
- **Frontend**: http://localhost:5173
- **Backend API**: http://localhost:3000
- **RAG Service**: http://localhost:8000
- **Database**: localhost:5433

### Production Mode
- **Frontend**: http://localhost:8080
- **Backend API**: http://localhost:3001
- **RAG Service**: http://localhost:8000
- **Database**: localhost:5433

## 🔍 Troubleshooting

### Docker Issues

**Problem**: "Docker is not running"
```powershell
# Solution: Start Docker Desktop
# Check: Docker Desktop system tray icon should be green
```

**Problem**: WSL2 errors
```powershell
# Update WSL2
wsl --update

# Install Ubuntu distribution
wsl --install Ubuntu

# Restart Docker Desktop
```

### Port Conflicts

**Problem**: Port already in use
```powershell
# Find process using port
netstat -ano | findstr :5173

# Kill process (replace PID)
taskkill /PID 1234 /F
```

### PowerShell Issues

**Problem**: Execution policy restrictions
```powershell
# Allow scripts for current user
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser

# Or run with bypass
powershell -ExecutionPolicy Bypass -File run.ps1 start-dev
```

### Performance Issues

**Problem**: Slow Docker performance
- Increase Docker Desktop memory allocation (Settings > Resources)
- Enable WSL2 integration
- Move project to WSL2 filesystem for better performance:
  ```bash
  # In WSL2 terminal
  cd /mnt/c/Users/YourName/
  git clone <repo-url>
  cd tracseq2.0
  ./run.sh start-dev
  ```

### Network Issues

**Problem**: Services can't communicate
```powershell
# Reset Docker networks
docker network prune -f

# Restart services
.\run.ps1 restart-dev
```

### RAG/AI Issues

**Problem**: ChatBot not responding
```powershell
# Check Ollama status
ollama list

# Restart Ollama
.\run.ps1 stop-ollama
.\run.ps1 start-ollama

# Check RAG service logs
.\run.ps1 logs rag
```

## 📁 File Structure

```
tracseq2.0/
├── start-tracseq.cmd     # Quick start script
├── run.ps1               # PowerShell runner (recommended)
├── run.bat               # Windows batch runner
├── run.sh                # Git Bash runner
├── README-Windows.md     # This file
├── lab_manager/          # Backend application
└── lab_submission_rag/   # AI/RAG service
```

## 🔒 Security Notes

- Default setup uses local AI models (Ollama) for privacy
- No external API keys required for basic functionality
- All services run locally on your machine
- Database data is persisted in Docker volumes

## 💡 Tips for Windows Users

1. **Use PowerShell ISE or Windows Terminal** for better experience
2. **Run as Administrator** if you encounter permission issues
3. **Disable Windows Fast Startup** if Docker has issues after restart
4. **Use WSL2** for better performance with Docker
5. **Keep Docker Desktop updated** for latest Windows compatibility

## 🆘 Getting Help

If you encounter issues:

1. Check the troubleshooting section above
2. Run `.\run.ps1 status` to check service health
3. Check logs: `.\run.ps1 logs lab-manager` or `.\run.ps1 logs rag`
4. Try rebuilding: `.\run.ps1 clean` then `.\run.ps1 start-dev`

## 🎯 Development vs Production

**Use Development Mode** (`start-dev`) when:
- Developing or testing
- You want hot reload
- You need debugging features

**Use Production Mode** (`start-prod`) when:
- Running in a production environment
- You want optimized performance
- You don't need hot reload

---

**Happy coding with TracSeq 2.0 on Windows! 🚀** 
