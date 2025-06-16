# TracSeq 2.0 Integrated Runner Script
# Manages both lab_manager and lab_submission_rag services in unified Docker setup

param(
    [Parameter(Position=0)]
    [string]$Command = "help",
    
    [Parameter(Position=1)]
    [string]$Mode = "dev",
    
    [switch]$Force,
    [switch]$NoPull
)

# Set console title
$Host.UI.RawUI.WindowTitle = "TracSeq 2.0 Integrated Runner"

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

# Function to check if Docker is running
function Test-Docker {
    try {
        $null = docker info 2>$null
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

# Function to check for required files
function Test-RequiredFiles {
    $requiredFiles = @(
        "docker-compose.unified.yml",
        "tracseq.env",
        "lab_manager/Dockerfile",
        "lab_submission_rag/Dockerfile"
    )
    
    foreach ($file in $requiredFiles) {
        if (-not (Test-Path $file)) {
            Write-Error-Custom "Required file not found: $file"
            return $false
        }
    }
    
    Write-Success "All required files found"
    return $true
}

# Function to initialize environment
function Initialize-Environment {
    Write-Info "Initializing TracSeq 2.0 environment..."
    
    # Create required directories
    $directories = @(
        "data/postgres",
        "data/ollama", 
        "data/rag_uploads",
        "data/rag_exports",
        "data/rag_logs",
        "data/rag_data",
        "data/lab_storage"
    )
    
    foreach ($dir in $directories) {
        if (-not (Test-Path $dir)) {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
        }
    }
    
    # Copy environment file if it doesn't exist
    if (-not (Test-Path ".env")) {
        Copy-Item "tracseq.env" ".env"
        Write-Warning "Created .env file from tracseq.env - please review and update as needed"
    }
    
    Write-Success "Environment initialized"
}

# Function to pull/update Ollama models
function Initialize-OllamaModels {
    param($ModelName = "llama3.2:3b")
    
    Write-Info "Checking Ollama model: $ModelName"
    
    # Wait for Ollama to be ready
    $maxAttempts = 30
    $attempt = 0
    
    do {
        $attempt++
        try {
            $response = Invoke-WebRequest -Uri "http://localhost:11434/api/version" -TimeoutSec 2 -ErrorAction Stop
            Write-Success "Ollama is ready"
            break
        }
        catch {
            if ($attempt -eq $maxAttempts) {
                Write-Warning "Ollama may not be fully ready, but continuing..."
                return
            }
            Write-Info "Waiting for Ollama to start... ($attempt/$maxAttempts)"
            Start-Sleep -Seconds 2
        }
    } while ($attempt -lt $maxAttempts)
    
    # Pull the model
    Write-Info "Pulling model $ModelName (this may take several minutes)..."
    try {
        docker exec tracseq_ollama ollama pull $ModelName
        Write-Success "Model $ModelName is ready"
    }
    catch {
        Write-Warning "Failed to pull model, but system will continue"
    }
}

# Function to start services
function Start-Services {
    param($Mode)
    
    Write-Header "Starting TracSeq 2.0 ($Mode mode)"
    
    if (-not (Test-Docker)) { return }
    if (-not (Test-RequiredFiles)) { return }
    
    Initialize-Environment
    
    # Set compose file and profile
    $composeFile = "docker-compose.unified.yml"
    $services = @("postgres", "ollama", "rag-service")
    
    if ($Mode -eq "dev") {
        $services += @("lab-manager-dev", "lab-manager-frontend-dev")
        $frontendPort = "5173"
        $backendPort = "3000"
    }
    else {
        $services += @("lab-manager-prod", "lab-manager-frontend-prod")
        $frontendPort = "8080"
        $backendPort = "3001"
        $env:COMPOSE_PROFILES = "production"
    }
    
    Write-Info "Starting core services..."
    
    # Start services in dependency order
    Write-Info "Starting PostgreSQL..."
    docker-compose -f $composeFile up -d postgres
    Start-Sleep -Seconds 5
    
    Write-Info "Starting Ollama..."
    docker-compose -f $composeFile up -d ollama
    Start-Sleep -Seconds 10
    
    Write-Info "Starting RAG service..."
    docker-compose -f $composeFile up -d rag-service
    Start-Sleep -Seconds 5
    
    Write-Info "Starting Lab Manager services..."
    if ($Mode -eq "dev") {
        docker-compose -f $composeFile up -d lab-manager-dev lab-manager-frontend-dev
    }
    else {
        docker-compose -f $composeFile --profile production up -d lab-manager-prod lab-manager-frontend-prod
    }
    
    Write-Success "All services started!"
    Write-Host ""
    Write-Host "Services available at:" -ForegroundColor Green
    Write-Host "  Frontend:     http://localhost:$frontendPort" -ForegroundColor Cyan
    Write-Host "  Backend:      http://localhost:$backendPort" -ForegroundColor Cyan  
    Write-Host "  RAG Service:  http://localhost:8000" -ForegroundColor Cyan
    Write-Host "  PostgreSQL:   localhost:5433" -ForegroundColor Cyan
    Write-Host "  Ollama:       http://localhost:11434" -ForegroundColor Cyan
    Write-Host ""
    
    # Initialize Ollama models in background
    Start-Job -ScriptBlock {
        param($ModelName)
        Start-Sleep -Seconds 30  # Wait for services to stabilize
        
        # Get model from environment or use default
        $model = $ModelName
        if (-not $model) {
            $envContent = Get-Content ".env" -ErrorAction SilentlyContinue
            $modelLine = $envContent | Where-Object { $_ -like "OLLAMA_MODEL=*" }
            if ($modelLine) {
                $model = ($modelLine -split "=")[1].Trim()
            } else {
                $model = "llama3.2:3b"
            }
        }
        
        Write-Host "[BACKGROUND] Pulling Ollama model: $model"
        docker exec tracseq_ollama ollama pull $model
        Write-Host "[BACKGROUND] Model $model ready"
    } -ArgumentList "llama3.2:3b" | Out-Null
    
    Write-Info "Model download started in background"
    Write-Info "Run 'docker logs tracseq_ollama' to monitor progress"
}

# Function to stop services
function Stop-Services {
    Write-Header "Stopping TracSeq 2.0"
    
    Write-Info "Stopping all services..."
    docker-compose -f docker-compose.unified.yml down
    
    Write-Success "All services stopped"
}

# Function to show service status
function Show-ServiceStatus {
    Write-Header "TracSeq 2.0 Service Status"
    
    if (-not (Test-Docker)) { return }
    
    docker-compose -f docker-compose.unified.yml ps
}

# Function to show logs
function Show-ServiceLogs {
    param($ServiceName)
    
    if (-not (Test-Docker)) { return }
    
    if ($ServiceName) {
        Write-Info "Showing logs for $ServiceName..."
        docker-compose -f docker-compose.unified.yml logs -f $ServiceName
    }
    else {
        Write-Info "Showing logs for all services..."
        docker-compose -f docker-compose.unified.yml logs -f
    }
}

# Function to run integration tests
function Test-Integration {
    Write-Header "Testing TracSeq 2.0 Integration"
    
    $tests = @(
        @{
            Name = "PostgreSQL Database"
            Test = { docker exec tracseq_postgres pg_isready -U postgres }
        },
        @{
            Name = "Ollama Service"
            Test = { Invoke-WebRequest -Uri "http://localhost:11434/api/version" -TimeoutSec 5 }
        },
        @{
            Name = "RAG Service"
            Test = { Invoke-WebRequest -Uri "http://localhost:8000/health" -TimeoutSec 5 }
        },
        @{
            Name = "Lab Manager Backend"
            Test = { Invoke-WebRequest -Uri "http://localhost:3000/health" -TimeoutSec 5 }
        }
    )
    
    foreach ($test in $tests) {
        Write-Host "Testing $($test.Name)... " -NoNewline
        try {
            & $test.Test | Out-Null
            Write-Host "✅" -ForegroundColor Green
        }
        catch {
            Write-Host "❌" -ForegroundColor Red
            Write-Warning "  $($_.Exception.Message)"
        }
    }
    
    # Test RAG-Lab Manager integration
    Write-Host "Testing RAG-Lab Manager Integration... " -NoNewline
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:3000/api/samples/rag/status" -TimeoutSec 10
        if ($response.StatusCode -eq 200) {
            Write-Host "✅" -ForegroundColor Green
            Write-Success "Integration test passed!"
        }
        else {
            throw "Unexpected status code: $($response.StatusCode)"
        }
    }
    catch {
        Write-Host "❌" -ForegroundColor Red
        Write-Warning "Integration test failed: $($_.Exception.Message)"
    }
}

# Function to open web interfaces
function Open-Interfaces {
    param($Mode)
    
    if ($Mode -eq "dev") {
        $frontendUrl = "http://localhost:5173"
        $backendUrl = "http://localhost:3000"
    }
    else {
        $frontendUrl = "http://localhost:8080"
        $backendUrl = "http://localhost:3001"
    }
    
    Write-Info "Opening web interfaces..."
    Start-Process $frontendUrl
    Start-Process "$frontendUrl/rag-submissions"
    Start-Process "http://localhost:8000"
    
    Write-Success "Web interfaces opened"
}

# Function to show help
function Show-Help {
    Write-Host "TracSeq 2.0 Integrated Runner Script" -ForegroundColor Blue
    Write-Host ""
    Write-Host "Usage: .\run-integrated.ps1 [COMMAND] [MODE]" -ForegroundColor Green
    Write-Host ""
    Write-Host "Commands:" -ForegroundColor Yellow
    Write-Host "  start          Start all services (default: dev mode)"
    Write-Host "  stop           Stop all services"
    Write-Host "  restart        Restart all services"
    Write-Host "  status         Show status of all services"
    Write-Host "  logs [service] Show logs (optional: specific service)"
    Write-Host "  test           Run integration tests"
    Write-Host "  open           Open web interfaces"
    Write-Host "  pull-model     Pull Ollama model"
    Write-Host "  init           Initialize environment only"
    Write-Host "  help           Show this help message"
    Write-Host ""
    Write-Host "Modes:" -ForegroundColor Yellow
    Write-Host "  dev            Development mode (hot reload, port 5173)"
    Write-Host "  prod           Production mode (optimized, port 8080)"
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Cyan
    Write-Host "  .\run-integrated.ps1 start dev    # Start in development mode"
    Write-Host "  .\run-integrated.ps1 start prod   # Start in production mode"
    Write-Host "  .\run-integrated.ps1 logs rag-service  # Show RAG service logs"
    Write-Host "  .\run-integrated.ps1 test         # Test integration"
    Write-Host ""
    Write-Host "Services:" -ForegroundColor Green
    Write-Host "  - PostgreSQL:   localhost:5433"
    Write-Host "  - Ollama:       localhost:11434"
    Write-Host "  - RAG Service:  localhost:8000"
    Write-Host "  - Lab Manager:  localhost:3000 (dev) / localhost:3001 (prod)"
    Write-Host "  - Frontend:     localhost:5173 (dev) / localhost:8080 (prod)"
}

# Main command routing
switch ($Command.ToLower()) {
    "start" {
        Start-Services $Mode
    }
    "stop" {
        Stop-Services
    }
    "restart" {
        Stop-Services
        Start-Sleep -Seconds 3
        Start-Services $Mode
    }
    "status" {
        Show-ServiceStatus
    }
    "logs" {
        Show-ServiceLogs $Mode
    }
    "test" {
        Test-Integration
    }
    "open" {
        Open-Interfaces $Mode
    }
    "pull-model" {
        Initialize-OllamaModels $Mode
    }
    "init" {
        Initialize-Environment
    }
    "help" {
        Show-Help
    }
    default {
        Write-Error-Custom "Unknown command: $Command"
        Write-Host ""
        Show-Help
    }
} 
