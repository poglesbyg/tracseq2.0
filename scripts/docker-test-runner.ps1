# TracSeq 2.0 Docker Test Runner
# Comprehensive testing environment using Docker containers

Write-Host "🐳 TracSeq 2.0 Docker Test Runner" -ForegroundColor Blue
Write-Host "=================================" -ForegroundColor Blue

# Function to wait for Docker to be ready
function Wait-ForDockerReady {
    Write-Host "⏳ Waiting for Docker Desktop to be ready..." -ForegroundColor Yellow
    $maxAttempts = 60  # Increased for Docker startup
    $attempt = 0
    
    do {
        try {
            $result = docker version --format "{{.Server.Version}}" 2>$null
            if ($result) {
                Write-Host "✅ Docker is ready! Version: $result" -ForegroundColor Green
                return $true
            }
        } catch {
            # Continue waiting
        }
        
        $attempt++
        Start-Sleep -Seconds 3
        if ($attempt % 10 -eq 0) {
            Write-Host "   Still waiting... ($attempt/$maxAttempts)" -ForegroundColor Gray
        }
        
    } while ($attempt -lt $maxAttempts)
    
    Write-Host "❌ Docker is not responding after $maxAttempts attempts" -ForegroundColor Red
    return $false
}

# Function to setup environment
function Setup-Environment {
    Write-Host "🔧 Setting up environment..." -ForegroundColor Cyan
    
    # Copy environment file
    if (Test-Path "docker.env") {
        Copy-Item "docker.env" ".env" -Force
        Write-Host "✅ Environment file created" -ForegroundColor Green
    }
    
    # Stop any running services
    Write-Host "🛑 Stopping any running services..." -ForegroundColor Yellow
    Get-Process -Name "node", "python", "cargo" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    
    Write-Host "✅ Environment setup complete" -ForegroundColor Green
}

# Function to run Docker tests
function Start-DockerTests {
    param([string]$ConfigType = "unified")
    
    Write-Host "🚀 Starting Docker tests with $ConfigType configuration..." -ForegroundColor Green
    
    $composeFile = switch ($ConfigType) {
        "unified" { "deploy/development/docker-compose.unified.yml" }
        "main" { "docker-compose.yml" }
        "windows" { "lab_manager/docker-compose.windows.yml" }
        default { "deploy/development/docker-compose.unified.yml" }
    }
    
    if (-not (Test-Path $composeFile)) {
        Write-Host "❌ Compose file not found: $composeFile" -ForegroundColor Red
        return $false
    }
    
    Write-Host "📋 Using configuration: $composeFile" -ForegroundColor Cyan
    
    # Pull images first
    Write-Host "📦 Pulling required images..." -ForegroundColor Yellow
    docker compose -f $composeFile pull 2>$null
    
    # Start services
    Write-Host "🏗️ Building and starting services..." -ForegroundColor Yellow
    docker compose -f $composeFile up -d --build
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Services started successfully!" -ForegroundColor Green
        return $true
    } else {
        Write-Host "❌ Failed to start services" -ForegroundColor Red
        return $false
    }
}

# Function to monitor services
function Monitor-Services {
    Write-Host "📊 Monitoring services startup..." -ForegroundColor Cyan
    
    $services = @(
        @{Name="PostgreSQL"; Port=5433; MaxWait=60},
        @{Name="Ollama"; Port=11434; MaxWait=120},
        @{Name="RAG Service"; Port=8000; MaxWait=180},
        @{Name="Backend"; Port=3000; MaxWait=120},
        @{Name="Frontend"; Port=5173; MaxWait=60}
    )
    
    foreach ($service in $services) {
        Write-Host "⏳ Waiting for $($service.Name) on port $($service.Port)..." -ForegroundColor Yellow
        
        $waited = 0
        do {
            try {
                $connection = Test-NetConnection -ComputerName localhost -Port $service.Port -InformationLevel Quiet -WarningAction SilentlyContinue
                if ($connection) {
                    Write-Host "✅ $($service.Name) is ready!" -ForegroundColor Green
                    break
                }
            } catch {
                # Continue waiting
            }
            
            Start-Sleep -Seconds 5
            $waited += 5
            
            if ($waited % 30 -eq 0) {
                Write-Host "   Still waiting for $($service.Name)... ($waited/$($service.MaxWait)s)" -ForegroundColor Gray
            }
            
        } while ($waited -lt $service.MaxWait)
        
        if ($waited -ge $service.MaxWait) {
            Write-Host "⚠️ $($service.Name) took longer than expected to start" -ForegroundColor Yellow
        }
    }
}

# Function to run tests
function Run-Tests {
    Write-Host "🧪 Running comprehensive tests..." -ForegroundColor Magenta
    
    # Health checks
    Write-Host "🔍 Health Checks:" -ForegroundColor Cyan
    
    $healthChecks = @(
        @{Name="PostgreSQL"; URL="http://localhost:5433"; Type="port"},
        @{Name="RAG Service"; URL="http://localhost:8000/health"; Type="http"},
        @{Name="Backend API"; URL="http://localhost:3000/health"; Type="http"},
        @{Name="Frontend"; URL="http://localhost:5173"; Type="http"}
    )
    
    foreach ($check in $healthChecks) {
        try {
            if ($check.Type -eq "http") {
                $response = Invoke-WebRequest -Uri $check.URL -TimeoutSec 10 -UseBasicParsing
                if ($response.StatusCode -eq 200) {
                    Write-Host "   ✅ $($check.Name): Healthy" -ForegroundColor Green
                } else {
                    Write-Host "   ⚠️ $($check.Name): Status $($response.StatusCode)" -ForegroundColor Yellow
                }
            } else {
                $connection = Test-NetConnection -ComputerName localhost -Port 5433 -InformationLevel Quiet
                if ($connection) {
                    Write-Host "   ✅ $($check.Name): Port accessible" -ForegroundColor Green
                } else {
                    Write-Host "   ❌ $($check.Name): Port not accessible" -ForegroundColor Red
                }
            }
        } catch {
            Write-Host "   ❌ $($check.Name): $($_.Exception.Message)" -ForegroundColor Red
        }
    }
    
    # Container status
    Write-Host ""
    Write-Host "📊 Container Status:" -ForegroundColor Cyan
    docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
    
    # Service logs (last 10 lines)
    Write-Host ""
    Write-Host "📜 Recent Service Logs:" -ForegroundColor Cyan
    docker compose logs --tail=5 rag-service 2>$null
}

# Function to cleanup
function Cleanup-Tests {
    Write-Host "🧹 Cleaning up test environment..." -ForegroundColor Yellow
    
    # Stop all containers
    docker compose -f deploy/development/docker-compose.unified.yml down 2>$null
    docker compose down 2>$null
    docker compose -f lab_manager/docker-compose.yml down 2>$null
    
    # Prune containers and networks
    docker container prune -f 2>$null
    docker network prune -f 2>$null
    
    Write-Host "✅ Cleanup complete" -ForegroundColor Green
}

# Main execution
Write-Host "🤔 Select test configuration:" -ForegroundColor Cyan
Write-Host "   1. Unified Development (Recommended) - All services with proper networking" -ForegroundColor White
Write-Host "   2. Main Configuration - Production-like setup" -ForegroundColor White
Write-Host "   3. Windows Optimized - Windows-specific configuration" -ForegroundColor White
Write-Host "   4. Cleanup Only - Stop all containers and cleanup" -ForegroundColor White
Write-Host ""

$choice = Read-Host "Enter choice (1-4) or press Enter for Unified Development"

if ($choice -eq "4") {
    Cleanup-Tests
    Write-Host ""
    Write-Host "Press any key to exit..." -ForegroundColor Gray
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    return
}

$configType = switch ($choice) {
    "2" { "main" }
    "3" { "windows" } 
    default { "unified" }
}

# Wait for Docker
if (-not (Wait-ForDockerReady)) {
    Write-Host ""
    Write-Host "🔧 Docker Troubleshooting:" -ForegroundColor Yellow
    Write-Host "   1. Ensure Docker Desktop is installed and running" -ForegroundColor White
    Write-Host "   2. Wait for Docker Desktop to show 'Engine running'" -ForegroundColor White
    Write-Host "   3. Try restarting Docker Desktop" -ForegroundColor White
    Write-Host "   4. Check Docker Desktop settings (WSL2, Hyper-V)" -ForegroundColor White
    Write-Host ""
    Write-Host "Press any key to exit..." -ForegroundColor Gray
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    return
}

# Setup environment
Setup-Environment

# Start services
if (Start-DockerTests -ConfigType $configType) {
    # Monitor startup
    Monitor-Services
    
    # Run tests
    Run-Tests
    
    Write-Host ""
    Write-Host "🎉 Docker test environment is ready!" -ForegroundColor Green
    Write-Host ""
    Write-Host "🌐 Access Points:" -ForegroundColor White
    Write-Host "   • Frontend: http://localhost:5173" -ForegroundColor Cyan
    Write-Host "   • Backend API: http://localhost:3000" -ForegroundColor Cyan
    Write-Host "   • RAG Service: http://localhost:8000" -ForegroundColor Cyan
    Write-Host "   • Database: localhost:5433" -ForegroundColor Cyan
    Write-Host "   • Ollama: http://localhost:11434" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "📚 Useful Commands:" -ForegroundColor White
    Write-Host "   • View logs: docker compose logs -f [service-name]" -ForegroundColor Yellow
    Write-Host "   • Restart service: docker compose restart [service-name]" -ForegroundColor Yellow
    Write-Host "   • Stop all: docker compose down" -ForegroundColor Yellow
    Write-Host "   • Container status: docker ps" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "🧪 All tests running in containerized environment!" -ForegroundColor Magenta
    
} else {
    Write-Host ""
    Write-Host "❌ Failed to start Docker test environment" -ForegroundColor Red
    Write-Host ""
    Write-Host "🔧 Troubleshooting:" -ForegroundColor Yellow
    Write-Host "   • Check logs: docker compose logs" -ForegroundColor White
    Write-Host "   • Check available space: docker system df" -ForegroundColor White
    Write-Host "   • Try cleanup: docker system prune" -ForegroundColor White
    Write-Host "   • Check Docker Desktop resources (CPU, Memory)" -ForegroundColor White
}

Write-Host ""
Write-Host "Press any key to exit..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown") 
