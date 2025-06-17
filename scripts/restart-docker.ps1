# TracSeq 2.0 Docker Restart Script
# Restarts Docker containers for the TracSeq application

Write-Host "🐳 Restarting TracSeq 2.0 Docker Application..." -ForegroundColor Blue

# Function to wait for Docker to be ready
function Wait-ForDocker {
    Write-Host "⏳ Waiting for Docker to be ready..." -ForegroundColor Yellow
    $maxAttempts = 30
    $attempt = 0
    
    do {
        try {
            $result = docker version --format "{{.Server.Version}}" 2>$null
            if ($result) {
                Write-Host "✅ Docker is ready (Version: $result)" -ForegroundColor Green
                return $true
            }
        } catch {
            # Continue waiting
        }
        
        $attempt++
        Start-Sleep -Seconds 2
        Write-Host "   Attempt $attempt/$maxAttempts..." -ForegroundColor Gray
        
    } while ($attempt -lt $maxAttempts)
    
    Write-Host "❌ Docker is not responding after $maxAttempts attempts" -ForegroundColor Red
    return $false
}

# Function to stop and remove containers
function Stop-TracSeqContainers {
    Write-Host "🛑 Stopping existing TracSeq containers..." -ForegroundColor Yellow
    
    # Stop containers from different compose files
    docker compose down 2>$null
    docker compose -f deploy/development/docker-compose.unified.yml down 2>$null
    docker compose -f lab_manager/docker-compose.yml down 2>$null
    docker compose -f lab_manager/docker-compose.windows.yml down 2>$null
    docker compose -f lab_manager/docker-compose.lightweight.yml down 2>$null
    
    # Remove any orphaned containers
    Write-Host "🧹 Cleaning up orphaned containers..." -ForegroundColor Yellow
    docker container prune -f 2>$null
    
    Write-Host "✅ Containers stopped and cleaned up" -ForegroundColor Green
}

# Function to start containers
function Start-TracSeqContainers {
    param([string]$ComposeFile = "docker-compose.yml")
    
    Write-Host "🚀 Starting TracSeq containers using $ComposeFile..." -ForegroundColor Green
    
    try {
        if ($ComposeFile -eq "docker-compose.yml") {
            docker compose up -d
        } else {
            docker compose -f $ComposeFile up -d
        }
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ Containers started successfully" -ForegroundColor Green
            return $true
        } else {
            Write-Host "❌ Failed to start containers" -ForegroundColor Red
            return $false
        }
    } catch {
        Write-Host "❌ Error starting containers: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

# Function to show container status
function Show-ContainerStatus {
    Write-Host "📊 Container Status:" -ForegroundColor Cyan
    docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
}

# Main restart process
Write-Host "🔍 Checking Docker status..." -ForegroundColor Cyan

if (-not (Wait-ForDocker)) {
    Write-Host "❌ Docker is not available. Please:" -ForegroundColor Red
    Write-Host "   1. Ensure Docker Desktop is running" -ForegroundColor Yellow
    Write-Host "   2. Wait for Docker Desktop to fully start" -ForegroundColor Yellow
    Write-Host "   3. Try running this script again" -ForegroundColor Yellow
    exit 1
}

# Stop existing containers
Stop-TracSeqContainers

# Determine which compose file to use
Write-Host "🤔 Select Docker configuration:" -ForegroundColor Cyan
Write-Host "   1. Main (docker-compose.yml) - Default" -ForegroundColor White
Write-Host "   2. Development Unified (deploy/development/docker-compose.unified.yml)" -ForegroundColor White
Write-Host "   3. Windows Specific (lab_manager/docker-compose.windows.yml)" -ForegroundColor White
Write-Host "   4. Lightweight (lab_manager/docker-compose.lightweight.yml)" -ForegroundColor White
Write-Host ""

$choice = Read-Host "Enter choice (1-4) or press Enter for default"

$composeFile = switch ($choice) {
    "2" { "deploy/development/docker-compose.unified.yml" }
    "3" { "lab_manager/docker-compose.windows.yml" }
    "4" { "lab_manager/docker-compose.lightweight.yml" }
    default { "docker-compose.yml" }
}

Write-Host "📋 Using configuration: $composeFile" -ForegroundColor Cyan

# Start containers
if (Start-TracSeqContainers -ComposeFile $composeFile) {
    # Wait a moment for containers to fully start
    Write-Host "⏳ Waiting for containers to initialize..." -ForegroundColor Yellow
    Start-Sleep -Seconds 10
    
    # Show status
    Show-ContainerStatus
    
    Write-Host ""
    Write-Host "🎉 TracSeq 2.0 Docker Application Restarted Successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "🌐 Access Points:" -ForegroundColor White
    Write-Host "   • Frontend: http://localhost:5173" -ForegroundColor Cyan
    Write-Host "   • Backend API: http://localhost:3001" -ForegroundColor Cyan
    Write-Host "   • RAG Service: http://localhost:8000" -ForegroundColor Cyan
    Write-Host "   • Health Check: http://localhost:3001/health" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "📚 Useful Commands:" -ForegroundColor White
    Write-Host "   • View logs: docker compose logs -f" -ForegroundColor Yellow
    Write-Host "   • Stop services: docker compose down" -ForegroundColor Yellow
    Write-Host "   • Restart specific service: docker compose restart <service-name>" -ForegroundColor Yellow
    Write-Host "   • Check status: docker ps" -ForegroundColor Yellow
    
} else {
    Write-Host ""
    Write-Host "❌ Failed to restart TracSeq Docker application" -ForegroundColor Red
    Write-Host ""
    Write-Host "🔧 Troubleshooting Steps:" -ForegroundColor Yellow
    Write-Host "   1. Check Docker Desktop is running" -ForegroundColor White
    Write-Host "   2. Check compose file exists: Test-Path $composeFile" -ForegroundColor White
    Write-Host "   3. View detailed logs: docker compose logs" -ForegroundColor White
    Write-Host "   4. Check system resources (CPU, Memory, Disk)" -ForegroundColor White
    Write-Host "   5. Try a different compose configuration" -ForegroundColor White
}

Write-Host ""
Write-Host "Press any key to exit..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown") 
