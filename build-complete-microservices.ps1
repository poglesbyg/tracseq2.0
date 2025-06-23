#!/usr/bin/env pwsh
# TracSeq 2.0 Complete Microservices Builder & Deployer
# Builds and deploys all 15 microservices with proper orchestration

param(
    [Parameter(Position=0)]
    [ValidateSet("build", "deploy", "stop", "restart", "logs", "status", "cleanup")]
    [string]$Action = "deploy",
    
    [switch]$SkipBuild,
    [switch]$SkipInfrastructure,
    [switch]$Monitoring,
    [string]$Service = ""
)

# Configuration
$ComposeFile = "docker-compose.complete-microservices.yml"
$Services = @{
    "Infrastructure" = @("postgres", "redis", "ollama")
    "Core" = @("auth-service", "event-service")
    "Business" = @("sample-service", "enhanced-storage-service", "template-service", "sequencing-service", "notification-service")
    "Specialized" = @("spreadsheet-versioning-service", "qaqc-service", "library-details-service")
    "AI" = @("enhanced-rag-service", "lab-submission-rag-service")
    "Gateway" = @("api-gateway", "lab-manager")
    "Monitoring" = @("prometheus", "grafana")
}

# Helper functions
function Write-Info { param($Message) Write-Host "[INFO] $Message" -ForegroundColor Blue }
function Write-Success { param($Message) Write-Host "[SUCCESS] $Message" -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host "[WARNING] $Message" -ForegroundColor Yellow }
function Write-Error-Custom { param($Message) Write-Host "[ERROR] $Message" -ForegroundColor Red }
function Write-Header { 
    param($Title)
    Write-Host "=" * 80 -ForegroundColor Blue
    Write-Host "  TracSeq 2.0 Complete Microservices - $Title" -ForegroundColor Blue
    Write-Host "=" * 80 -ForegroundColor Blue
}

function Test-Prerequisites {
    Write-Info "Checking prerequisites..."
    
    # Check Docker
    try {
        docker --version | Out-Null
        docker compose version | Out-Null
        docker info | Out-Null
        Write-Success "Docker is available and running"
    }
    catch {
        Write-Error-Custom "Docker is not available or not running. Please start Docker Desktop."
        exit 1
    }
    
    # Check Rust
    try {
        cargo --version | Out-Null
        Write-Success "Rust toolchain is available"
    }
    catch {
        Write-Warning "Rust toolchain not found. Services will build in Docker only."
    }
    
    # Check Python
    try {
        python --version | Out-Null
        Write-Success "Python is available"
    }
    catch {
        Write-Warning "Python not found. AI services will build in Docker only."
    }
    
    # Check Node.js
    try {
        node --version | Out-Null
        pnpm --version | Out-Null
        Write-Success "Node.js and pnpm are available"
    }
    catch {
        Write-Warning "Node.js/pnpm not found. Frontend will build in Docker only."
    }
}

function New-MissingDockerfiles {
    Write-Info "Creating missing Dockerfiles..."
    
    $rustServices = @(
        "auth_service", "sample_service", "sequencing_service", "notification_service",
        "template_service", "transaction_service", "event_service", 
        "spreadsheet_versioning_service", "qaqc_service", "library_details_service"
    )
    
    foreach ($service in $rustServices) {
        $dockerfilePath = "$service/Dockerfile"
        if (-not (Test-Path $dockerfilePath)) {
            Write-Info "Creating Dockerfile for $service..."
            
$rustDockerfile = @"
# Multi-stage build for $service
FROM rustlang/rust:nightly-bookworm as builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml ./
COPY $service/ ./$service/

# Build the service
WORKDIR /usr/src/app/$service
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary
COPY --from=builder /usr/src/app/target/release/$($service.Replace('_', '-')) ./service

# Expose default port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info

CMD ["./service"]
"@
            $rustDockerfile | Out-File -FilePath $dockerfilePath -Encoding UTF8
        }
    }
    
    # Python services Dockerfiles
    $pythonServices = @("enhanced_rag_service", "api_gateway")
    
    foreach ($service in $pythonServices) {
        $dockerfilePath = "$service/Dockerfile"
        if (-not (Test-Path $dockerfilePath)) {
            Write-Info "Creating Dockerfile for $service..."
            
$pythonDockerfile = @"
FROM python:3.11-slim

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements
COPY requirements.txt .

# Install Python dependencies
RUN pip install --no-cache-dir -r requirements.txt

# Copy source code
COPY . .

# Create directories
RUN mkdir -p uploads exports logs data

# Set environment variables
ENV PYTHONUNBUFFERED=1
ENV PYTHONOPTIMIZE=1

# Expose port
EXPOSE 8080

CMD ["python", "src/main.py"]
"@
            $pythonDockerfile | Out-File -FilePath $dockerfilePath -Encoding UTF8
        }
    }
    
    Write-Success "Dockerfiles created for missing services"
}

function Start-Infrastructure {
    if ($SkipInfrastructure) {
        Write-Info "Skipping infrastructure deployment"
        return
    }
    
    Write-Header "Phase 1: Infrastructure Services"
    Write-Info "Starting PostgreSQL, Redis, and Ollama..."
    
    docker compose -f $ComposeFile up -d postgres redis ollama
    
    Write-Info "Waiting for infrastructure to be healthy..."
    $timeout = 60
    $elapsed = 0
    
    do {
        Start-Sleep 5
        $elapsed += 5
        $healthyServices = (docker compose -f $ComposeFile ps --format json | ConvertFrom-Json | Where-Object {$_.Health -eq "healthy"}).Count
        Write-Info "Healthy infrastructure services: $healthyServices/3"
    } while ($healthyServices -lt 3 -and $elapsed -lt $timeout)
    
    if ($healthyServices -eq 3) {
        Write-Success "Infrastructure services are healthy"
    } else {
        Write-Warning "Some infrastructure services may not be fully ready"
    }
}

function Start-CoreServices {
    Write-Header "Phase 2: Core Services"
    Write-Info "Starting authentication and event services..."
    
    docker compose -f $ComposeFile up -d auth-service event-service
    
    Start-Sleep 30
    Test-ServiceHealth "http://localhost:8080/health" "Auth Service"
    Test-ServiceHealth "http://localhost:8087/health" "Event Service"
}

function Start-BusinessServices {
    Write-Header "Phase 3: Business Services"
    Write-Info "Starting business logic services..."
    
    $businessServices = $Services["Business"] -join " "
    docker compose -f $ComposeFile up -d $businessServices.Split(" ")
    
    Start-Sleep 45
    
    $healthChecks = @{
        "Sample Service" = "http://localhost:8081/health"
        "Enhanced Storage Service" = "http://localhost:8082/health"
        "Template Service" = "http://localhost:8083/health"
        "Sequencing Service" = "http://localhost:8084/health"
        "Notification Service" = "http://localhost:8085/health"
    }
    
    foreach ($service in $healthChecks.GetEnumerator()) {
        Test-ServiceHealth $service.Value $service.Key
    }
}

function Start-SpecializedServices {
    Write-Header "Phase 4: Specialized Services"
    Write-Info "Starting specialized and AI services..."
    
    $specializedServices = $Services["Specialized"] + $Services["AI"]
    $serviceNames = $specializedServices -join " "
    docker compose -f $ComposeFile up -d $specializedServices
    
    Start-Sleep 30
    
    $healthChecks = @{
        "Spreadsheet Versioning Service" = "http://localhost:8090/health"
        "QAQC Service" = "http://localhost:8091/health"
        "Library Details Service" = "http://localhost:8092/health"
        "Enhanced RAG Service" = "http://localhost:8086/health"
        "Lab Submission RAG Service" = "http://localhost:8000/health"
    }
    
    foreach ($service in $healthChecks.GetEnumerator()) {
        Test-ServiceHealth $service.Value $service.Key
    }
}

function Start-GatewayServices {
    Write-Header "Phase 5: Gateway and Main Services"
    Write-Info "Starting API Gateway and Lab Manager..."
    
    docker compose -f $ComposeFile up -d api-gateway lab-manager
    
    Start-Sleep 30
    Test-ServiceHealth "http://localhost:8089/health" "API Gateway"
    Test-ServiceHealth "http://localhost:3000/health" "Lab Manager"
}

function Start-MonitoringServices {
    if (-not $Monitoring) {
        Write-Info "Skipping monitoring services (use -Monitoring to enable)"
        return
    }
    
    Write-Header "Phase 6: Monitoring Stack"
    Write-Info "Starting Prometheus and Grafana..."
    
    docker compose -f $ComposeFile up -d prometheus grafana
    
    Start-Sleep 15
    Write-Success "Monitoring services started"
    Write-Info "Prometheus: http://localhost:9090"
    Write-Info "Grafana: http://localhost:3001 (admin/admin)"
}

function Test-ServiceHealth {
    param($Url, $ServiceName)
    
    try {
        $response = Invoke-WebRequest -Uri $Url -Method GET -TimeoutSec 10 -ErrorAction Stop
        if ($response.StatusCode -eq 200) {
            Write-Success "✅ $ServiceName is healthy"
            return $true
        }
    }
    catch {
        Write-Warning "⚠️  $ServiceName not responding: $($_.Exception.Message)"
        return $false
    }
    return $false
}

function Show-DeploymentSummary {
    Write-Header "Deployment Summary"
    
    Write-Host ""
    Write-Host "🌐 TracSeq 2.0 Complete Microservices Architecture" -ForegroundColor Green
    Write-Host "=" * 60
    
    Write-Host ""
    Write-Host "🔐 Core Services:" -ForegroundColor Yellow
    Write-Host "   Auth Service:           http://localhost:8080" -ForegroundColor Cyan
    Write-Host "   Event Service:          http://localhost:8087" -ForegroundColor Cyan
    
    Write-Host ""
    Write-Host "🧪 Laboratory Services:" -ForegroundColor Yellow
    Write-Host "   Sample Service:         http://localhost:8081" -ForegroundColor Cyan
    Write-Host "   Enhanced Storage:       http://localhost:8082" -ForegroundColor Cyan
    Write-Host "   Template Service:       http://localhost:8083" -ForegroundColor Cyan
    Write-Host "   Sequencing Service:     http://localhost:8084" -ForegroundColor Cyan
    Write-Host "   Notification Service:   http://localhost:8085" -ForegroundColor Cyan
    
    Write-Host ""
    Write-Host "📊 Specialized Services:" -ForegroundColor Yellow
    Write-Host "   Spreadsheet Versioning: http://localhost:8090" -ForegroundColor Cyan
    Write-Host "   QAQC Service:           http://localhost:8091" -ForegroundColor Cyan
    Write-Host "   Library Details:        http://localhost:8092" -ForegroundColor Cyan
    
    Write-Host ""
    Write-Host "🤖 AI/ML Services:" -ForegroundColor Yellow
    Write-Host "   Enhanced RAG Service:   http://localhost:8086" -ForegroundColor Cyan
    Write-Host "   Lab Submission RAG:     http://localhost:8000" -ForegroundColor Cyan
    
    Write-Host ""
    Write-Host "🚪 Gateway & Main:" -ForegroundColor Yellow
    Write-Host "   API Gateway:            http://localhost:8089" -ForegroundColor Cyan
    Write-Host "   Lab Manager:            http://localhost:3000" -ForegroundColor Cyan
    Write-Host "   Frontend (if running):  http://localhost:5173" -ForegroundColor Cyan
    
    if ($Monitoring) {
        Write-Host ""
        Write-Host "📊 Monitoring:" -ForegroundColor Yellow
        Write-Host "   Prometheus:             http://localhost:9090" -ForegroundColor Cyan
        Write-Host "   Grafana:                http://localhost:3001" -ForegroundColor Cyan
    }
    
    Write-Host ""
    Write-Host "📋 Next Steps:" -ForegroundColor Green
    Write-Host "   1. Test API Gateway endpoints" -ForegroundColor White
    Write-Host "   2. Access Grafana for monitoring" -ForegroundColor White
    Write-Host "   3. Check service logs: docker compose -f $ComposeFile logs [service]" -ForegroundColor White
    Write-Host "   4. Start frontend: cd lab_manager/frontend && pnpm dev" -ForegroundColor White
}

function Stop-AllServices {
    Write-Header "Stopping All Services"
    docker compose -f $ComposeFile down
    Write-Success "All services stopped"
}

function Show-ServiceStatus {
    Write-Header "Service Status"
    docker compose -f $ComposeFile ps
}

function Show-ServiceLogs {
    if ($Service) {
        Write-Info "Showing logs for $Service..."
        docker compose -f $ComposeFile logs -f $Service
    } else {
        Write-Info "Showing logs for all services..."
        docker compose -f $ComposeFile logs -f
    }
}

# Main execution
switch ($Action.ToLower()) {
    "build" {
        Test-Prerequisites
        New-MissingDockerfiles
        
        if (-not $SkipBuild) {
            Write-Header "Building All Services"
            docker compose -f $ComposeFile build
            Write-Success "All services built"
        }
    }
    
    "deploy" {
        Test-Prerequisites
        New-MissingDockerfiles
        
        if (-not $SkipBuild) {
            Write-Header "Building Services"
            docker compose -f $ComposeFile build
        }
        
        Start-Infrastructure
        Start-CoreServices
        Start-BusinessServices
        Start-SpecializedServices
        Start-GatewayServices
        Start-MonitoringServices
        
        Show-DeploymentSummary
    }
    
    "stop" { Stop-AllServices }
    "restart" { 
        Stop-AllServices
        Start-Sleep 5
        & $PSCommandPath -Action deploy -SkipBuild:$SkipBuild -Monitoring:$Monitoring
    }
    "status" { Show-ServiceStatus }
    "logs" { Show-ServiceLogs }
    "cleanup" {
        Stop-AllServices
        Write-Info "Removing all containers and networks..."
        docker compose -f $ComposeFile down --volumes --remove-orphans
        Write-Success "Cleanup completed"
    }
    
    default {
        Write-Error-Custom "Unknown action: $Action"
        Write-Host "Usage: .\build-complete-microservices.ps1 [build|deploy|stop|restart|logs|status|cleanup]"
        Write-Host "Options:"
        Write-Host "  -SkipBuild       Skip building images"
        Write-Host "  -Monitoring      Enable monitoring stack"
        Write-Host "  -Service <name>  Show logs for specific service"
        exit 1
    }
} 
