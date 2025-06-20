# ========================================
# Complete TracSeq Microservices Deployment Script
# ========================================

param(
    [Parameter(Position=0)]
    [string]$Action = "deploy",
    [switch]$SkipBuild,
    [switch]$QuickStart
)

# Configuration
$ProjectRoot = Get-Location
$Services = @(
    @{Name="Auth Service"; Path="auth_service"; Port=8080},
    @{Name="Sample Service"; Path="sample_service"; Port=8081},
    @{Name="Enhanced Storage Service"; Path="enhanced_storage_service"; Port=8082},
    @{Name="Template Service"; Path="template_service"; Port=8083},
    @{Name="Notification Service"; Path="notification_service"; Port=8085},
    @{Name="Enhanced RAG Service"; Path="enhanced_rag_service"; Port=8086},
    @{Name="Event Service"; Path="event_service"; Port=8087},
    @{Name="Transaction Service"; Path="transaction_service"; Port=8088},
    @{Name="API Gateway"; Path="api_gateway"; Port=8089},
    @{Name="Sequencing Service"; Path="sequencing_service"; Port=8090}
)

# Helper functions
function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Test-ServiceHealth {
    param([string]$Url, [string]$ServiceName)
    
    try {
        $response = Invoke-WebRequest -Uri $Url -Method GET -TimeoutSec 10
        if ($response.StatusCode -eq 200) {
            Write-Success "✅ $ServiceName is healthy"
            return $true
        }
    }
    catch {
        Write-Warning "⚠️  $ServiceName not responding yet"
        return $false
    }
    return $false
}

function Start-QuickEnhancedStorage {
    Write-Info "🚀 Starting Enhanced Storage Service (Quick Mode)..."
    
    # Check if Rust is installed
    try {
        cargo --version | Out-Null
        Write-Success "Rust toolchain found"
    }
    catch {
        Write-Info "Installing Rust toolchain..."
        # Download and install Rust
        Invoke-WebRequest -Uri "https://win.rustup.rs/" -OutFile "rustup-init.exe"
        Start-Process -FilePath "rustup-init.exe" -ArgumentList "-y" -Wait
        Remove-Item "rustup-init.exe"
        
        # Refresh environment
        $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "User")
    }
    
    # Start infrastructure if not running
    Set-Location "enhanced_storage_service"
    
    $containers = docker ps --format "table {{.Names}}" | Select-String "enhanced_storage_service"
    if (-not $containers) {
        Write-Info "Starting Enhanced Storage infrastructure..."
        docker-compose -f docker-compose.minimal.yml up -d
        Start-Sleep -Seconds 15
    }
    
    # Generate Cargo.lock if missing
    if (-not (Test-Path "Cargo.lock")) {
        Write-Info "Generating Cargo.lock..."
        cargo generate-lockfile
    }
    
    # Build and run the service
    Write-Info "Building Enhanced Storage Service..."
    cargo build --release
    
    Write-Info "Starting Enhanced Storage Service..."
    Start-Process -FilePath "powershell" -ArgumentList "-Command", "cd '$PWD'; $env:RUST_LOG='info'; cargo run" -WindowStyle Normal
    
    Set-Location $ProjectRoot
    Write-Success "Enhanced Storage Service started in new window"
}

function Deploy-CompleteStack {
    Write-Info "🚀 Deploying Complete TracSeq Microservices Stack..."
    
    # Check Docker
    try {
        docker --version | Out-Null
        docker info | Out-Null
    }
    catch {
        Write-Error "Docker is not running. Please start Docker Desktop first."
        exit 1
    }
    
    # Create unified docker-compose file
    Write-Info "Creating unified deployment configuration..."
    
    $unifiedCompose = @"
version: '3.8'

services:
  # PostgreSQL Database (Shared)
  postgres:
    image: postgres:15-alpine
    ports:
      - "5432:5432"
    environment:
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=postgres
      - POSTGRES_DB=lab_manager
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./enhanced_storage_service/scripts/init-db.sql:/docker-entrypoint-initdb.d/01-enhanced-storage.sql
    networks:
      - microservices_network
    restart: unless-stopped

  # Redis (Shared)
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    networks:
      - microservices_network
    restart: unless-stopped

  # Enhanced Storage Service
  enhanced-storage-service:
    build: ./enhanced_storage_service
    ports:
      - "8082:8082"
    environment:
      - DATABASE_URL=postgres://postgres:postgres@postgres:5432/lab_manager
      - RUST_LOG=info
      - RUST_BACKTRACE=1
    depends_on:
      - postgres
      - redis
    networks:
      - microservices_network
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8082/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  # API Gateway
  api-gateway:
    build: ./api_gateway
    ports:
      - "8089:8089"
    environment:
      - ENHANCED_STORAGE_SERVICE_URL=http://enhanced-storage-service:8082
      - AUTH_SERVICE_URL=http://auth-service:8080
      - SAMPLE_SERVICE_URL=http://sample-service:8081
    depends_on:
      - enhanced-storage-service
    networks:
      - microservices_network
    restart: unless-stopped

  # Monitoring
  grafana:
    image: grafana/grafana:10.1.0
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_USER=admin
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana
    networks:
      - microservices_network
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
  grafana_data:

networks:
  microservices_network:
    driver: bridge
"@

    $unifiedCompose | Out-File -FilePath "docker-compose.microservices.yml" -Encoding UTF8
    
    # Deploy the stack
    Write-Info "Deploying microservices stack..."
    docker-compose -f docker-compose.microservices.yml up -d --build
    
    Write-Info "Waiting for services to start..."
    Start-Sleep -Seconds 30
    
    # Health check
    Test-ServicesHealth
}

function Test-ServicesHealth {
    Write-Info "🔍 Checking service health..."
    
    $healthyServices = 0
    $totalServices = 0
    
    $healthChecks = @(
        @{Url="http://localhost:8082/health"; Name="Enhanced Storage Service"},
        @{Url="http://localhost:8089/health"; Name="API Gateway"},
        @{Url="http://localhost:3000/api/health"; Name="Grafana"}
    )
    
    foreach ($check in $healthChecks) {
        $totalServices++
        if (Test-ServiceHealth -Url $check.Url -ServiceName $check.Name) {
            $healthyServices++
        }
    }
    
    Write-Info "Health Check Results: $healthyServices/$totalServices services healthy"
    
    if ($healthyServices -eq $totalServices) {
        Write-Success "🎉 All services are healthy!"
        Show-ServiceUrls
    } else {
        Write-Warning "Some services are not responding yet. They may still be starting up."
    }
}

function Show-ServiceUrls {
    Write-Info "🌐 Service Access Points:"
    Write-Host ""
    Write-Host "🚀 Enhanced Storage Service (109 endpoints): http://localhost:8082" -ForegroundColor Cyan
    Write-Host "🌐 API Gateway (Unified access): http://localhost:8089" -ForegroundColor Cyan
    Write-Host "📊 Grafana Dashboard: http://localhost:3000 (admin/admin)" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "🔗 Key Enhanced Storage Endpoints:" -ForegroundColor Yellow
    Write-Host "  - Storage Overview: http://localhost:8082/storage/overview" -ForegroundColor White
    Write-Host "  - AI Platform: http://localhost:8082/ai/overview" -ForegroundColor White
    Write-Host "  - IoT Sensors: http://localhost:8082/iot/sensors" -ForegroundColor White
    Write-Host "  - Integrations: http://localhost:8082/integrations/overview" -ForegroundColor White
    Write-Host "  - Analytics: http://localhost:8082/analytics/overview" -ForegroundColor White
    Write-Host ""
    Write-Host "📋 Next Steps:" -ForegroundColor Green
    Write-Host "  1. Open Grafana dashboard for monitoring" -ForegroundColor White
    Write-Host "  2. Test Enhanced Storage endpoints" -ForegroundColor White
    Write-Host "  3. Configure frontend integration" -ForegroundColor White
    Write-Host "  4. Set up cross-service communication" -ForegroundColor White
}

function Stop-Services {
    Write-Info "🛑 Stopping microservices..."
    
    # Stop Enhanced Storage if running standalone
    Set-Location "enhanced_storage_service"
    docker-compose -f docker-compose.minimal.yml down
    
    Set-Location $ProjectRoot
    
    # Stop unified stack if exists
    if (Test-Path "docker-compose.microservices.yml") {
        docker-compose -f docker-compose.microservices.yml down
    }
    
    Write-Success "Services stopped"
}

function Test-Integration {
    Write-Info "🧪 Running integration tests..."
    
    $tests = @(
        @{
            Name = "Enhanced Storage Health"
            Url = "http://localhost:8082/health"
            Expected = "healthy"
        },
        @{
            Name = "Storage Overview"
            Url = "http://localhost:8082/storage/overview"
            Expected = "locations"
        },
        @{
            Name = "AI Platform"
            Url = "http://localhost:8082/ai/overview"
            Expected = "models"
        }
    )
    
    $passed = 0
    foreach ($test in $tests) {
        try {
            $response = Invoke-RestMethod -Uri $test.Url -Method GET -TimeoutSec 10
            if ($response -match $test.Expected) {
                Write-Success "✅ $($test.Name) - PASSED"
                $passed++
            } else {
                Write-Warning "⚠️  $($test.Name) - UNEXPECTED RESPONSE"
            }
        }
        catch {
            Write-Error "❌ $($test.Name) - FAILED"
        }
    }
    
    Write-Info "Integration Tests: $passed/$($tests.Count) passed"
}

# Main execution
switch ($Action.ToLower()) {
    "quick" {
        Write-Info "🚀 Quick Start - Enhanced Storage Service Only"
        Start-QuickEnhancedStorage
        Start-Sleep -Seconds 10
        Test-Integration
        Show-ServiceUrls
    }
    "deploy" {
        if ($QuickStart) {
            Write-Info "🚀 Quick Deploy - Core Services"
            Start-QuickEnhancedStorage
        } else {
            Write-Info "🚀 Full Deploy - Complete Microservices Stack"
            Deploy-CompleteStack
        }
    }
    "test" {
        Test-Integration
    }
    "health" {
        Test-ServicesHealth
    }
    "stop" {
        Stop-Services
    }
    "urls" {
        Show-ServiceUrls
    }
    default {
        Write-Host "TracSeq Microservices Deployment Script"
        Write-Host ""
        Write-Host "Usage: .\deploy-complete-microservices.ps1 [action]"
        Write-Host ""
        Write-Host "Actions:"
        Write-Host "  quick     - Quick start Enhanced Storage Service only"
        Write-Host "  deploy    - Full microservices deployment"
        Write-Host "  test      - Run integration tests"
        Write-Host "  health    - Check service health"
        Write-Host "  stop      - Stop all services"
        Write-Host "  urls      - Show service URLs"
        Write-Host ""
        Write-Host "Options:"
        Write-Host "  -QuickStart  - Deploy core services only"
        Write-Host ""
        Write-Host "Examples:"
        Write-Host "  .\deploy-complete-microservices.ps1 quick"
        Write-Host "  .\deploy-complete-microservices.ps1 deploy -QuickStart"
        Write-Host "  .\deploy-complete-microservices.ps1 deploy"
    }
} 
