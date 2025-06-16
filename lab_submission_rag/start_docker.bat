@echo off
REM 🐳 Laboratory Submission RAG System - Docker Quick Start (Windows)
REM Ultra-lightweight setup with Ollama

echo 🧬 Laboratory Submission RAG System
echo 🐳 Docker Quick Start (Windows)
echo ==================================================

REM Check if Docker is installed
docker --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ Docker is not installed. Please install Docker Desktop:
    echo    https://docs.docker.com/desktop/windows/
    pause
    exit /b 1
)

REM Check if Docker Compose is available
docker compose version >nul 2>&1
if %errorlevel% neq 0 (
    docker-compose --version >nul 2>&1
    if %errorlevel% neq 0 (
        echo ❌ Docker Compose is not available. Please install Docker Compose:
        echo    https://docs.docker.com/compose/install/
        pause
        exit /b 1
    )
    set COMPOSE_CMD=docker-compose
) else (
    set COMPOSE_CMD=docker compose
)

echo ✅ Docker is ready

REM Create necessary directories
echo 📁 Creating directories...
if not exist "data" mkdir data
if not exist "uploads" mkdir uploads
if not exist "exports" mkdir exports
if not exist "demo" mkdir demo

echo 🚀 Starting Docker containers...

REM Stop any existing containers
%COMPOSE_CMD% -f docker-compose-simple.yml down >nul 2>&1

REM Start the services
%COMPOSE_CMD% -f docker-compose-simple.yml up -d

echo ✅ Containers started!
echo.
echo 📥 Downloading Ollama model (first time only)...
echo    This may take 2-5 minutes depending on your internet connection...

REM Wait for model download (simplified for Windows)
timeout /t 5 /nobreak >nul
echo Waiting for model download to complete...

REM Check for completion (simple approach)
for /l %%i in (1,1,12) do (
    timeout /t 5 /nobreak >nul
    docker logs simple-rag-model-downloader 2>nul | findstr "successfully" >nul
    if %errorlevel% equ 0 (
        echo ✅ Model download completed!
        goto model_ready
    )
    echo .
)

:model_ready
echo.
echo 🔍 Checking system health...

REM Wait for services to be ready
for /l %%i in (1,1,12) do (
    timeout /t 5 /nobreak >nul
    curl -s http://localhost:8000/health >nul 2>&1
    if %errorlevel% equ 0 (
        echo ✅ System is ready!
        goto system_ready
    )
    echo .
)

:system_ready
echo.
echo 🎉 Laboratory Submission RAG System is ready!
echo.
echo 🌐 Web Interface: http://localhost:8000
echo 🏥 Health Check:  http://localhost:8000/health
echo.
echo 📊 Quick Status Check:

REM Show container status
%COMPOSE_CMD% -f docker-compose-simple.yml ps

echo.
echo 🔧 Useful Commands:
echo    View logs:        %COMPOSE_CMD% -f docker-compose-simple.yml logs
echo    Stop system:      %COMPOSE_CMD% -f docker-compose-simple.yml down
echo    Restart system:   %COMPOSE_CMD% -f docker-compose-simple.yml restart
echo    System stats:     docker stats
echo.
echo 📚 Documentation: README_DOCKER.md
echo.
echo 🚀 Ready to process laboratory submissions!
echo    Open http://localhost:8000 in your browser to get started.
echo.
pause 
