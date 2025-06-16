# TracSeq 2.0 Runner Script for Windows (PowerShell)
# This script manages both lab_manager and lab_submission_rag services

param(
    [Parameter(Position=0)]
    [string]$Command = "help",
    
    [Parameter(Position=1)]
    [string]$Service = "",
    
    [switch]$Force
)

# Set console title
$Host.UI.RawUI.WindowTitle = "TracSeq 2.0 Runner"

# Colors and formatting
function Write-Info { param($Message) Write-Host "[INFO] $Message" -ForegroundColor Blue }
function Write-Success { param($Message) Write-Host "[SUCCESS] $Message" -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host "[WARNING] $Message" -ForegroundColor Yellow }
function Write-Error-Custom { param($Message) Write-Host "[ERROR] $Message" -ForegroundColor Red }
function Write-Header { 
    param($Title)
    Write-Host "================================" -ForegroundColor Blue
    Write-Host "  TracSeq 2.0 - $Title" -ForegroundColor Blue
    Write-Host "================================" -ForegroundColor Blue
}

# Function to initialize Docker paths for Windows
function Initialize-DockerPaths {
    $dockerPaths = @(
        "C:\Program Files\Docker\Docker\resources\bin",
        "$env:ProgramFiles\Docker\Docker\resources\bin",
        "$env:USERPROFILE\AppData\Local\Docker\Docker\resources\bin"
    )
    
    foreach ($path in $dockerPaths) {
        if (Test-Path "$path\docker.exe") {
            $env:PATH = "$path;$env:PATH"
            Write-Info "Found Docker at: $path"
            return $true
        }
    }
    return $false
}

# Function to check if Docker is running and available
function Test-Docker {
    # First try to find Docker in common locations
    $dockerFound = $false
    try {
        $null = Get-Command docker -ErrorAction Stop
        $dockerFound = $true
    }
    catch {
        # Try to initialize Docker paths
        if (Initialize-DockerPaths) {
            try {
                $null = Get-Command docker -ErrorAction Stop
                $dockerFound = $true
            }
            catch { }
        }
    }
    
    if (-not $dockerFound) {
        Write-Error-Custom "Docker command not found"
        Write-Info "Please install Docker Desktop and ensure it's running"
        Write-Info "Download Docker Desktop from: https://www.docker.com/products/docker-desktop"
        Write-Info "Alternatively, use run.bat which will use Git Bash"
        return $false
    }
    
    # Check if Docker is running
    try {
        $dockerInfo = docker info 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Docker is running"
            return $true
        }
    }
    catch { }
    
    Write-Error-Custom "Docker is not running"
    Write-Info "Please start Docker Desktop and try again"
    return $false
}

# Function to get the correct docker-compose command
function Get-DockerComposeCommand {
    # Try docker compose first (newer versions)
    try {
        $null = docker compose version 2>$null
        if ($LASTEXITCODE -eq 0) {
            return "docker compose"
        }
    }
    catch { }
    
    # Try docker-compose (older versions)
    try {
        $null = Get-Command docker-compose -ErrorAction Stop
        $null = docker-compose version 2>$null
        if ($LASTEXITCODE -eq 0) {
            return "docker-compose"
        }
    }
    catch { }
    
    # Fallback to docker compose
    return "docker compose"
}

# Function to check if required files exist
function Test-RequiredFiles {
    $labManagerCompose = "lab_manager\docker-compose.yml"
    $ragCompose = "lab_submission_rag\docker-compose.yml"
    
    if (-not (Test-Path $labManagerCompose)) {
        Write-Error-Custom "File not found: $labManagerCompose"
        return $false
    }
    
    if (-not (Test-Path $ragCompose)) {
        Write-Error-Custom "File not found: $ragCompose"
        return $false
    }
    
    Write-Success "All required files found"
    return $true
}

# Function to create required directories
function New-RequiredDirectories {
    Write-Info "Creating required directories..."
    
    $directories = @(
        "lab_submission_rag\uploads",
        "lab_submission_rag\exports", 
        "lab_submission_rag\logs",
        "lab_submission_rag\data",
        "lab_manager\storage",
        "lab_manager\uploads"
    )
    
    foreach ($dir in $directories) {
        if (-not (Test-Path $dir)) {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
        }
    }
    
    Write-Success "Directories created"
}

# Function to setup RAG environment
function Initialize-RagEnvironment {
    Write-Info "Setting up RAG service environment..."
    
    $envFile = "lab_submission_rag\.env"
    if (-not (Test-Path $envFile)) {
        Write-Warning "Creating default .env file for RAG service"
        
        $envContent = @"
# LLM Configuration
OPENAI_API_KEY=your_openai_key_here
ANTHROPIC_API_KEY=your_anthropic_key_here

# Ollama Configuration (for local LLM)
USE_OLLAMA=true
OLLAMA_MODEL=llama2
OLLAMA_BASE_URL=http://localhost:11434

# LLM Parameters
LLM_TEMPERATURE=0.7
MAX_TOKENS=2048
"@
        $envContent | Out-File -FilePath $envFile -Encoding UTF8
        Write-Warning "Please edit $envFile with your API keys"
    }
    
    Initialize-Ollama
}

# Function to check and setup Ollama
function Initialize-Ollama {
    Write-Info "Checking Ollama installation..."
    
    # Check if Ollama is installed
    try {
        $null = Get-Command ollama -ErrorAction Stop
        Write-Success "Ollama is installed"
    }
    catch {
        Write-Warning "Ollama not found"
        Write-Info "Run: .\run.ps1 install-ollama"
        Write-Info "Or download from: https://ollama.ai/download/windows"
        return
    }
    
    # Check if Ollama service is running
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:11434/api/version" -TimeoutSec 2 -ErrorAction Stop
        Write-Success "Ollama service is already running"
    }
    catch {
        Write-Info "Starting Ollama service..."
        Start-Process -FilePath "ollama" -ArgumentList "serve" -WindowStyle Minimized
        Start-Sleep -Seconds 3
    }
    
    # Check if default model exists
    $modelName = "llama2"
    $envFile = "lab_submission_rag\.env"
    if (Test-Path $envFile) {
        $envContent = Get-Content $envFile
        $modelLine = $envContent | Where-Object { $_ -like "OLLAMA_MODEL=*" }
        if ($modelLine) {
            $modelName = ($modelLine -split "=")[1].Trim()
        }
    }
    
    try {
        $models = ollama list 2>$null
        if ($models -like "*$modelName*") {
            Write-Success "Model $modelName is available"
        }
        else {
            Write-Warning "Model $modelName not found"
            Write-Info "Run: .\run.ps1 pull-model"
        }
    }
    catch {
        Write-Warning "Could not check Ollama models"
    }
}

# Function to start services in production mode
function Start-ProductionServices {
    Write-Header "Starting Production Mode"
    
    if (-not (Test-Docker)) { return }
    if (-not (Test-RequiredFiles)) { return }
    
    $composeCmd = Get-DockerComposeCommand
    Initialize-RagEnvironment
    New-RequiredDirectories
    
    Write-Info "Starting Lab Manager (Production)..."
    Set-Location "lab_manager"
    Invoke-Expression "$composeCmd up -d frontend app db"
    Set-Location ".."
    
    Write-Info "Starting RAG Service..."
    Set-Location "lab_submission_rag"
    Invoke-Expression "$composeCmd up -d"
    Set-Location ".."
    
    Write-Success "All services started successfully!"
    Write-Host ""
    Write-Host "Services available at:" -ForegroundColor Green
    Write-Host "  Lab Manager Frontend: http://localhost:8080" -ForegroundColor Cyan
    Write-Host "  Lab Manager Backend:  http://localhost:3001" -ForegroundColor Cyan
    Write-Host "  RAG Service:          http://localhost:8000" -ForegroundColor Cyan
    Write-Host "  PostgreSQL:           localhost:5433" -ForegroundColor Cyan
}

# Function to start services in development mode
function Start-DevelopmentServices {
    Write-Header "Starting Development Mode"
    
    if (-not (Test-Docker)) { return }
    if (-not (Test-RequiredFiles)) { return }
    
    $composeCmd = Get-DockerComposeCommand
    Initialize-RagEnvironment
    New-RequiredDirectories
    
    Write-Info "Starting Lab Manager (Development)..."
    Set-Location "lab_manager"
    Invoke-Expression "$composeCmd up -d frontend-dev dev db"
    Set-Location ".."
    
    Write-Info "Starting RAG Service..."
    Set-Location "lab_submission_rag"
    Invoke-Expression "$composeCmd up -d"
    Set-Location ".."
    
    Write-Success "All services started successfully!"
    Write-Host ""
    Write-Host "Services available at:" -ForegroundColor Green
    Write-Host "  Lab Manager Frontend: http://localhost:5173" -ForegroundColor Cyan
    Write-Host "  Lab Manager Backend:  http://localhost:3000" -ForegroundColor Cyan
    Write-Host "  RAG Service:          http://localhost:8000" -ForegroundColor Cyan
    Write-Host "  PostgreSQL:           localhost:5433" -ForegroundColor Cyan
}

# Function to stop all services
function Stop-AllServices {
    Write-Header "Stopping Services"
    
    Write-Info "Stopping all services..."
    
    $composeCmd = Get-DockerComposeCommand
    
    Set-Location "lab_manager"
    Invoke-Expression "$composeCmd down"
    Set-Location ".."
    
    Set-Location "lab_submission_rag"
    Invoke-Expression "$composeCmd down"
    Set-Location ".."
    
    # Stop Ollama if running
    try {
        Stop-Process -Name "ollama" -Force -ErrorAction SilentlyContinue
        Write-Info "Ollama service stopped"
    }
    catch {
        # Ollama wasn't running
    }
    
    Write-Success "All services stopped"
}

# Function to show service status
function Show-ServiceStatus {
    Write-Header "Service Status"
    
    if (-not (Test-Docker)) { return }
    
    $composeCmd = Get-DockerComposeCommand
    
    Write-Host ""
    Write-Host "Lab Manager Services:" -ForegroundColor Yellow
    Set-Location "lab_manager"
    Invoke-Expression "$composeCmd ps"
    Set-Location ".."
    
    Write-Host ""
    Write-Host "RAG Service:" -ForegroundColor Yellow
    Set-Location "lab_submission_rag"
    Invoke-Expression "$composeCmd ps"
    Set-Location ".."
}

# Function to show logs
function Show-ServiceLogs {
    param($ServiceName)
    
    if (-not (Test-Docker)) { return }
    
    $composeCmd = Get-DockerComposeCommand
    
    if ($ServiceName -eq "lab-manager") {
        Write-Info "Showing Lab Manager logs..."
        Set-Location "lab_manager"
        Invoke-Expression "$composeCmd logs -f"
        Set-Location ".."
    }
    elseif ($ServiceName -eq "rag") {
        Write-Info "Showing RAG Service logs..."
        Set-Location "lab_submission_rag"
        Invoke-Expression "$composeCmd logs -f"
        Set-Location ".."
    }
    else {
        Write-Error-Custom "Invalid service. Use 'lab-manager' or 'rag'"
        exit 1
    }
}

# Function to rebuild services
function Invoke-RebuildServices {
    Write-Header "Rebuilding Services"
    
    if (-not (Test-Docker)) { return }
    
    Write-Info "Rebuilding all services..."
    
    $composeCmd = Get-DockerComposeCommand
    
    Set-Location "lab_manager"
    Invoke-Expression "$composeCmd build --no-cache"
    Set-Location ".."
    
    Set-Location "lab_submission_rag"
    Invoke-Expression "$composeCmd build --no-cache"
    Set-Location ".."
    
    Write-Success "All services rebuilt"
}

# Function to clean Docker resources
function Clear-DockerResources {
    Write-Header "Cleaning Docker Resources"
    
    Stop-AllServices
    
    Write-Info "Cleaning up Docker resources..."
    docker system prune -f
    
    Write-Success "Docker cleanup completed"
}

# Function to install Ollama
function Install-Ollama {
    Write-Header "Installing Ollama"
    
    Write-Info "Downloading Ollama installer..."
    
    try {
        $installerPath = "OllamaSetup.exe"
        Invoke-WebRequest -Uri "https://ollama.ai/download/OllamaSetup.exe" -OutFile $installerPath
        
        Write-Info "Running Ollama installer..."
        Start-Process -FilePath $installerPath -Wait
        Remove-Item $installerPath
        
        Write-Success "Ollama installation completed"
        Write-Info "Please restart your terminal and run: .\run.ps1 pull-model"
    }
    catch {
        Write-Error-Custom "Failed to download Ollama installer: $($_.Exception.Message)"
        Write-Info "Please manually download from: https://ollama.ai/download/windows"
    }
}

# Function to start Ollama service
function Start-OllamaService {
    Write-Info "Starting Ollama service..."
    
    try {
        Get-Command ollama -ErrorAction Stop | Out-Null
        Start-Process -FilePath "ollama" -ArgumentList "serve" -WindowStyle Minimized
        Write-Success "Ollama service started"
    }
    catch {
        Write-Error-Custom "Ollama not installed. Run: .\run.ps1 install-ollama"
    }
}

# Function to stop Ollama service
function Stop-OllamaService {
    Write-Info "Stopping Ollama service..."
    
    try {
        Stop-Process -Name "ollama" -Force -ErrorAction SilentlyContinue
        Write-Success "Ollama service stopped"
    }
    catch {
        Write-Info "Ollama service was not running"
    }
}

# Function to pull Ollama model
function Get-OllamaModel {
    param($ModelName)
    
    if (-not $ModelName) {
        $ModelName = "llama2"
        $envFile = "lab_submission_rag\.env"
        if (Test-Path $envFile) {
            $envContent = Get-Content $envFile
            $modelLine = $envContent | Where-Object { $_ -like "OLLAMA_MODEL=*" }
            if ($modelLine) {
                $ModelName = ($modelLine -split "=")[1].Trim()
            }
        }
    }
    
    Write-Info "Pulling Ollama model: $ModelName"
    Write-Info "This may take several minutes depending on model size..."
    
    try {
        ollama pull $ModelName
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Model $ModelName downloaded successfully"
        }
        else {
            throw "Ollama pull failed"
        }
    }
    catch {
        Write-Error-Custom "Failed to download model $ModelName"
        Write-Info "Available models: llama2, llama3, codellama, mistral, neural-chat"
    }
}

# Function to show help
function Show-Help {
    Write-Host "TracSeq 2.0 Runner Script for Windows (PowerShell)" -ForegroundColor Blue
    Write-Host ""
    Write-Host "Usage: .\run.ps1 [COMMAND]" -ForegroundColor Green
    Write-Host ""
    Write-Host "Commands:" -ForegroundColor Yellow
    Write-Host "  start-prod     Start all services in production mode"
    Write-Host "  start-dev      Start all services in development mode"
    Write-Host "  stop           Stop all services"
    Write-Host "  restart-prod   Restart all services in production mode"
    Write-Host "  restart-dev    Restart all services in development mode"
    Write-Host "  status         Show status of all services"
    Write-Host "  logs <service> Show logs (service: lab-manager or rag)"
    Write-Host "  rebuild        Rebuild all Docker images"
    Write-Host "  clean          Clean up Docker resources"
    Write-Host "  help           Show this help message"
    Write-Host ""
    Write-Host "Ollama Commands:" -ForegroundColor Yellow
    Write-Host "  install-ollama Install Ollama for local LLM"
    Write-Host "  start-ollama   Start Ollama service"
    Write-Host "  stop-ollama    Stop Ollama service"
    Write-Host "  pull-model     Download default model or: pull-model <model-name>"
    Write-Host ""
    Write-Host "Services:" -ForegroundColor Yellow
    Write-Host "  - Lab Manager Frontend: http://localhost:8080 (prod) or http://localhost:5173 (dev)"
    Write-Host "  - Lab Manager Backend:  http://localhost:3001 (prod) or http://localhost:3000 (dev)"
    Write-Host "  - RAG Service:          http://localhost:8000"
    Write-Host "  - PostgreSQL:           localhost:5433"
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Cyan
    Write-Host "  .\run.ps1 start-dev"
    Write-Host "  .\run.ps1 logs rag"
    Write-Host "  .\run.ps1 pull-model llama3"
}

# Function to open web interfaces
function Open-WebInterfaces {
    param($Mode)
    
    if ($Mode -eq "dev") {
        Start-Process "http://localhost:5173"  # Frontend Dev
        Start-Process "http://localhost:3000"  # Backend Dev
    }
    else {
        Start-Process "http://localhost:8080"  # Frontend Prod
        Start-Process "http://localhost:3001"  # Backend Prod
    }
    Start-Process "http://localhost:8000"      # RAG Service
}

# Main script execution
switch ($Command.ToLower()) {
    "start-prod" { 
        Start-ProductionServices 
        if ($?) { Open-WebInterfaces "prod" }
    }
    "start-dev" { 
        Start-DevelopmentServices 
        if ($?) { Open-WebInterfaces "dev" }
    }
    "stop" { Stop-AllServices }
    "restart-prod" { 
        Stop-AllServices
        Start-Sleep -Seconds 2
        Start-ProductionServices 
    }
    "restart-dev" { 
        Stop-AllServices
        Start-Sleep -Seconds 2
        Start-DevelopmentServices 
    }
    "status" { Show-ServiceStatus }
    "logs" { 
        if (-not $Service) {
            Write-Error-Custom "Please specify a service: lab-manager or rag"
            exit 1
        }
        Show-ServiceLogs $Service 
    }
    "rebuild" { Invoke-RebuildServices }
    "clean" { Clear-DockerResources }
    "install-ollama" { Install-Ollama }
    "start-ollama" { Start-OllamaService }
    "stop-ollama" { Stop-OllamaService }
    "pull-model" { Get-OllamaModel $Service }
    "help" { Show-Help }
    "" { Show-Help }
    default { 
        Write-Error-Custom "Unknown command: $Command"
        Show-Help
        exit 1
    }
} 
