@echo off
setlocal enabledelayedexpansion

REM TracSeq 2.0 Runner Script for Windows (Enhanced Batch Version)
REM This script manages both lab_manager and lab_submission_rag services

title TracSeq 2.0 Runner

REM Check execution policy and prefer PowerShell if available
powershell -Command "Get-ExecutionPolicy" >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    echo PowerShell detected. Checking execution policy...
    for /f "tokens=*" %%a in ('powershell -Command "Get-ExecutionPolicy"') do set POLICY=%%a
    
    if "!POLICY!"=="Restricted" (
        echo [WARNING] PowerShell execution policy is Restricted
        echo [INFO] Run: Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
        echo [INFO] Falling back to batch script...
        goto :batch_mode
    ) else (
        echo PowerShell execution policy allows scripts. Using PowerShell version...
        powershell -File "run.ps1" %*
        exit /b %ERRORLEVEL%
    )
)

REM Check if Git Bash is available
:batch_mode
where bash >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    echo Git Bash detected. Running shell script version...
    bash run.sh %*
    exit /b %ERRORLEVEL%
)

REM Native Windows batch implementation
echo Using native Windows batch implementation...
echo.

if "%1"=="" goto :help
if "%1"=="help" goto :help
if "%1"=="--help" goto :help
if "%1"=="-h" goto :help

REM Check if Docker is running
echo [INFO] Checking Docker...
docker info >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Docker is not running or not installed
    echo [INFO] Please start Docker Desktop and try again
    echo [INFO] Download from: https://www.docker.com/products/docker-desktop
    pause
    exit /b 1
)
echo [SUCCESS] Docker is running

REM Check required files
if not exist "lab_manager\docker-compose.yml" (
    echo [ERROR] File not found: lab_manager\docker-compose.yml
    pause
    exit /b 1
)
if not exist "lab_submission_rag\docker-compose.yml" (
    echo [ERROR] File not found: lab_submission_rag\docker-compose.yml
    pause
    exit /b 1
)
echo [SUCCESS] All required files found

REM Create required directories
echo [INFO] Creating required directories...
if not exist "lab_submission_rag\uploads" mkdir "lab_submission_rag\uploads"
if not exist "lab_submission_rag\exports" mkdir "lab_submission_rag\exports"  
if not exist "lab_submission_rag\logs" mkdir "lab_submission_rag\logs"
if not exist "lab_submission_rag\data" mkdir "lab_submission_rag\data"
if not exist "lab_manager\storage" mkdir "lab_manager\storage"
if not exist "lab_manager\uploads" mkdir "lab_manager\uploads"
echo [SUCCESS] Directories created

REM Create default .env file for RAG service if it doesn't exist
if not exist "lab_submission_rag\.env" (
    echo [WARNING] Creating default .env file for RAG service...
    (
        echo # LLM Configuration
        echo OPENAI_API_KEY=your_openai_key_here
        echo ANTHROPIC_API_KEY=your_anthropic_key_here
        echo.
        echo # Ollama Configuration ^(for local LLM^)
        echo USE_OLLAMA=true
        echo OLLAMA_MODEL=llama2
        echo OLLAMA_BASE_URL=http://localhost:11434
        echo.
        echo # LLM Parameters
        echo LLM_TEMPERATURE=0.7
        echo MAX_TOKENS=2048
    ) > "lab_submission_rag\.env"
    echo [WARNING] Please edit lab_submission_rag\.env with your API keys
)

REM Setup Ollama
call :setup_ollama

REM Route to appropriate command
if "%1"=="start-prod" goto :start_prod
if "%1"=="start-dev" goto :start_dev
if "%1"=="stop" goto :stop
if "%1"=="restart-prod" goto :restart_prod
if "%1"=="restart-dev" goto :restart_dev
if "%1"=="status" goto :status
if "%1"=="logs" goto :logs
if "%1"=="rebuild" goto :rebuild
if "%1"=="clean" goto :clean
if "%1"=="install-ollama" goto :install_ollama
if "%1"=="start-ollama" goto :start_ollama
if "%1"=="stop-ollama" goto :stop_ollama
if "%1"=="pull-model" goto :pull_model
if "%1"=="open" goto :open_interfaces

echo [ERROR] Unknown command: %1
goto :help

:start_prod
echo ================================
echo   TracSeq 2.0 - Starting Production Mode
echo ================================
echo.
echo [INFO] Starting Lab Manager (Production)...
cd /d "lab_manager"
docker-compose up -d frontend app db
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Failed to start Lab Manager
    cd /d ..
    pause
    exit /b 1
)
cd /d ..
echo [SUCCESS] Lab Manager started

echo.
echo [INFO] Starting RAG Service...
cd /d "lab_submission_rag"
docker-compose up -d
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Failed to start RAG Service
    cd /d ..
    pause
    exit /b 1
)
cd /d ..
echo [SUCCESS] RAG Service started

echo.
echo [SUCCESS] All services started successfully!
echo.
echo Services available at:
echo   Lab Manager Frontend: http://localhost:8080
echo   Lab Manager Backend:  http://localhost:3001
echo   RAG Service:          http://localhost:8000
echo   PostgreSQL:           localhost:5433
echo.
echo [INFO] Run "run.bat open" to open web interfaces
goto :end

:start_dev
echo ================================
echo   TracSeq 2.0 - Starting Development Mode
echo ================================
echo.
echo [INFO] Starting Lab Manager (Development)...
cd /d "lab_manager"
docker-compose up -d frontend-dev dev db
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Failed to start Lab Manager
    cd /d ..
    pause
    exit /b 1
)
cd /d ..
echo [SUCCESS] Lab Manager (Dev) started

echo.
echo [INFO] Starting RAG Service...
cd /d "lab_submission_rag"
docker-compose up -d
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Failed to start RAG Service
    cd /d ..
    pause
    exit /b 1
)
cd /d ..
echo [SUCCESS] RAG Service started

echo.
echo [SUCCESS] All services started successfully!
echo.
echo Services available at:
echo   Lab Manager Frontend: http://localhost:5173
echo   Lab Manager Backend:  http://localhost:3000
echo   RAG Service:          http://localhost:8000
echo   PostgreSQL:           localhost:5433
echo.
echo [INFO] Run "run.bat open dev" to open web interfaces
goto :end

:stop
echo ================================
echo   TracSeq 2.0 - Stopping Services
echo ================================
echo [INFO] Stopping all services...

cd /d "lab_manager"
docker-compose down
cd /d ..

cd /d "lab_submission_rag"
docker-compose down
cd /d ..

REM Stop Ollama if it's running
where ollama >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    echo [INFO] Stopping Ollama service...
    taskkill /f /im ollama.exe >nul 2>nul
)

echo [SUCCESS] All services stopped
goto :end

:restart_prod
echo [INFO] Restarting in production mode...
call :stop
timeout /t 2 /nobreak >nul
call :start_prod
goto :end

:restart_dev
echo [INFO] Restarting in development mode...
call :stop
timeout /t 2 /nobreak >nul
call :start_dev
goto :end

:status
echo ================================
echo   TracSeq 2.0 - Service Status
echo ================================
echo.
echo Lab Manager Services:
cd /d "lab_manager"
docker-compose ps
cd /d ..
echo.
echo RAG Service:
cd /d "lab_submission_rag"
docker-compose ps
cd /d ..
goto :end

:logs
if "%2"=="" (
    echo [ERROR] Please specify a service: lab-manager or rag
    echo Example: run.bat logs lab-manager
    goto :end
)

if "%2"=="lab-manager" (
    echo [INFO] Showing Lab Manager logs...
    cd /d "lab_manager"
    docker-compose logs -f
    cd /d ..
) else if "%2"=="rag" (
    echo [INFO] Showing RAG Service logs...
    cd /d "lab_submission_rag"
    docker-compose logs -f
    cd /d ..
) else (
    echo [ERROR] Invalid service. Use 'lab-manager' or 'rag'
)
goto :end

:rebuild
echo ================================
echo   TracSeq 2.0 - Rebuilding Services
echo ================================
echo [INFO] Rebuilding all services...

cd /d "lab_manager"
docker-compose build --no-cache
cd /d ..

cd /d "lab_submission_rag"
docker-compose build --no-cache
cd /d ..

echo [SUCCESS] All services rebuilt
goto :end

:clean
echo ================================
echo   TracSeq 2.0 - Cleaning Docker Resources
echo ================================
call :stop

echo [INFO] Cleaning up Docker resources...
docker system prune -f
echo [SUCCESS] Docker cleanup completed
goto :end

:open_interfaces
if "%2"=="dev" (
    echo [INFO] Opening development interfaces...
    start http://localhost:5173
    start http://localhost:3000
) else (
    echo [INFO] Opening production interfaces...
    start http://localhost:8080
    start http://localhost:3001
)
start http://localhost:8000
echo [SUCCESS] Web interfaces opened
goto :end

:setup_ollama
echo [INFO] Checking Ollama installation...
where ollama >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [WARNING] Ollama not found
    echo [INFO] Run 'run.bat install-ollama' to install it
    echo [INFO] Or download from: https://ollama.ai/download/windows
    goto :eof
)
echo [SUCCESS] Ollama is installed

REM Check if Ollama service is running
curl -s http://localhost:11434/api/version >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [INFO] Starting Ollama service...
    start "Ollama Service" /min ollama serve
    timeout /t 3 /nobreak >nul
) else (
    echo [SUCCESS] Ollama service is already running
)

REM Check if default model exists
set MODEL_NAME=llama2
if exist "lab_submission_rag\.env" (
    for /f "usebackq tokens=2 delims==" %%a in ("lab_submission_rag\.env") do (
        if "%%a" NEQ "" set MODEL_NAME=%%a
    )
)

ollama list | findstr /C:"%MODEL_NAME%" >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [WARNING] Model %MODEL_NAME% not found
    echo [INFO] Run 'run.bat pull-model' to download it
) else (
    echo [SUCCESS] Model %MODEL_NAME% is available
)
goto :eof

:install_ollama
echo ================================
echo   TracSeq 2.0 - Installing Ollama
echo ================================
echo [INFO] Downloading Ollama installer...

REM Use PowerShell to download if available, otherwise use bitsadmin
powershell -Command "& {Invoke-WebRequest -Uri 'https://ollama.ai/download/OllamaSetup.exe' -OutFile 'OllamaSetup.exe'}" >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [INFO] PowerShell download failed, trying bitsadmin...
    bitsadmin /transfer "Ollama Download" https://ollama.ai/download/OllamaSetup.exe "%CD%\OllamaSetup.exe"
)

if exist "OllamaSetup.exe" (
    echo [INFO] Running Ollama installer...
    start /wait OllamaSetup.exe
    del "OllamaSetup.exe"
    echo [SUCCESS] Ollama installation completed
    echo [INFO] Please restart your terminal and run 'run.bat pull-model'
) else (
    echo [ERROR] Failed to download Ollama installer
    echo [INFO] Please manually download from: https://ollama.ai/download/windows
)
goto :end

:start_ollama
echo [INFO] Starting Ollama service...
where ollama >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Ollama not installed. Run 'run.bat install-ollama' first
    goto :end
)
start "Ollama Service" /min ollama serve
echo [SUCCESS] Ollama service started
goto :end

:stop_ollama
echo [INFO] Stopping Ollama service...
taskkill /f /im ollama.exe >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    echo [SUCCESS] Ollama service stopped
) else (
    echo [INFO] Ollama service was not running
)
goto :end

:pull_model
set MODEL_NAME=llama2
if "%2" NEQ "" (
    set MODEL_NAME=%2
) else (
    if exist "lab_submission_rag\.env" (
        for /f "usebackq tokens=2 delims==" %%a in ("lab_submission_rag\.env") do (
            if "%%a" NEQ "" set MODEL_NAME=%%a
        )
    )
)

echo [INFO] Pulling Ollama model: %MODEL_NAME%
echo [INFO] This may take several minutes depending on model size...
ollama pull %MODEL_NAME%
if %ERRORLEVEL% EQU 0 (
    echo [SUCCESS] Model %MODEL_NAME% downloaded successfully
) else (
    echo [ERROR] Failed to download model %MODEL_NAME%
    echo [INFO] Available models: llama2, llama3, codellama, mistral, neural-chat
)
goto :end

:help
echo TracSeq 2.0 Runner Script for Windows (Enhanced Batch)
echo.
echo Usage: %~nx0 [COMMAND] [OPTIONS]
echo.
echo Commands:
echo   start-prod     Start all services in production mode
echo   start-dev      Start all services in development mode
echo   stop           Stop all services
echo   restart-prod   Restart all services in production mode
echo   restart-dev    Restart all services in development mode
echo   status         Show status of all services
echo   logs ^<service^>  Show logs (service: lab-manager or rag)
echo   rebuild        Rebuild all Docker images
echo   clean          Clean up Docker resources
echo   open [dev]     Open web interfaces in browser
echo   help           Show this help message
echo.
echo Ollama Commands:
echo   install-ollama Install Ollama for local LLM
echo   start-ollama   Start Ollama service
echo   stop-ollama    Stop Ollama service  
echo   pull-model     Download default model or: pull-model [model-name]
echo.
echo Services:
echo   - Lab Manager Frontend: http://localhost:8080 (prod) or http://localhost:5173 (dev)
echo   - Lab Manager Backend:  http://localhost:3001 (prod) or http://localhost:3000 (dev)
echo   - RAG Service:          http://localhost:8000
echo   - PostgreSQL:           localhost:5433
echo.
echo Examples:
echo   %~nx0 start-dev
echo   %~nx0 logs rag
echo   %~nx0 pull-model llama3
echo   %~nx0 open dev
echo.
echo Note: This script will automatically use PowerShell or Git Bash if available
echo       for enhanced functionality. Otherwise, it uses native Windows batch.
goto :end

:end 
