# TracSeq 2.0 Unified Startup Script
param([string]$Mode = "dev")

Write-Host "🧬 Starting TracSeq 2.0 Unified System" -ForegroundColor Blue
Write-Host "=======================================" -ForegroundColor Blue

# Check Docker
try {
    docker info | Out-Null
    Write-Host "✅ Docker is running" -ForegroundColor Green
}
catch {
    Write-Host "❌ Docker is not running. Please start Docker Desktop." -ForegroundColor Red
    exit 1
}

# Copy environment file if needed
if (-not (Test-Path ".env")) {
    Copy-Item "deploy/tracseq.env" ".env"
    Write-Host "📋 Created .env file from deploy/tracseq.env" -ForegroundColor Yellow
}

Write-Host "🚀 Starting services in $Mode mode..." -ForegroundColor Blue

# Define the correct docker-compose file path
$composeFile = "deploy/development/docker-compose.unified.yml"

# Start core services first
Write-Host "  Starting PostgreSQL..." -ForegroundColor Cyan
docker-compose -f $composeFile up -d postgres
Start-Sleep -Seconds 5

Write-Host "  Starting Ollama..." -ForegroundColor Cyan  
docker-compose -f $composeFile up -d ollama
Start-Sleep -Seconds 10

Write-Host "  Starting RAG service..." -ForegroundColor Cyan
docker-compose -f $composeFile up -d rag-service
Start-Sleep -Seconds 5

# Start Lab Manager services
Write-Host "  Starting Lab Manager..." -ForegroundColor Cyan
if ($Mode -eq "dev") {
    docker-compose -f $composeFile up -d lab-manager-dev lab-manager-frontend-dev
    $frontend = "5173"
    $backend = "3000"
} else {
    $env:COMPOSE_PROFILES = "production"
    docker-compose -f $composeFile --profile production up -d lab-manager-prod lab-manager-frontend-prod
    $frontend = "8080"
    $backend = "3001"
}

Write-Host ""
Write-Host "🎉 TracSeq 2.0 is now running!" -ForegroundColor Green
Write-Host ""
Write-Host "📍 Access your services:" -ForegroundColor Yellow
Write-Host "   Frontend:     http://localhost:$frontend" -ForegroundColor White
Write-Host "   Lab Manager:  http://localhost:$backend" -ForegroundColor White
Write-Host "   RAG Service:  http://localhost:8000" -ForegroundColor White
Write-Host "   Database:     localhost:5433" -ForegroundColor White
Write-Host ""
Write-Host "🔧 Useful commands:" -ForegroundColor Yellow
Write-Host "   docker-compose -f $composeFile ps      # Show status"
Write-Host "   docker-compose -f $composeFile logs    # Show logs"
Write-Host "   docker-compose -f $composeFile down    # Stop all"
Write-Host ""
Write-Host "⏳ Note: Ollama model download happens in background (may take 5-10 minutes)" 
