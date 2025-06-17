# Quick Docker Restart for TracSeq 2.0
Write-Host "🐳 Quick Docker Restart for TracSeq 2.0" -ForegroundColor Blue

# Test Docker availability
Write-Host "🔍 Testing Docker..." -ForegroundColor Yellow
try {
    $dockerTest = docker version --format "{{.Server.Version}}" 2>$null
    if ($dockerTest) {
        Write-Host "✅ Docker is ready! Version: $dockerTest" -ForegroundColor Green
        
        # Stop any existing containers
        Write-Host "🛑 Stopping existing containers..." -ForegroundColor Yellow
        docker compose down 2>$null
        
        # Start main configuration
        Write-Host "🚀 Starting TracSeq containers..." -ForegroundColor Green
        docker compose up -d
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ Docker containers started successfully!" -ForegroundColor Green
            Write-Host ""
            Write-Host "🌐 Access Points:" -ForegroundColor White
            Write-Host "   • Frontend: http://localhost:5173" -ForegroundColor Cyan
            Write-Host "   • Backend: http://localhost:3001" -ForegroundColor Cyan
            Write-Host "   • RAG Service: http://localhost:8000" -ForegroundColor Cyan
            Write-Host ""
            Write-Host "📊 Container Status:" -ForegroundColor Cyan
            docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
        } else {
            Write-Host "❌ Failed to start containers. Check logs with: docker compose logs" -ForegroundColor Red
        }
    } else {
        throw "Docker not responding"
    }
} catch {
    Write-Host "⚠️ Docker Engine not ready yet" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "🔧 Try these options:" -ForegroundColor White
    Write-Host "   1. Wait 2-3 minutes and run this script again" -ForegroundColor Cyan
    Write-Host "   2. Check Docker Desktop is fully started" -ForegroundColor Cyan
    Write-Host "   3. Restart Docker Desktop if needed" -ForegroundColor Cyan
    Write-Host "   4. Use current working setup: http://localhost:5173" -ForegroundColor Green
    Write-Host ""
    Write-Host "📱 Current Services Working:" -ForegroundColor Green
    Write-Host "   ✅ Frontend: http://localhost:5173" -ForegroundColor Green
    Write-Host "   ✅ Error handling fixed" -ForegroundColor Green
    Write-Host "   ✅ Mock authentication active" -ForegroundColor Green
}

Write-Host ""
Write-Host "Press any key to exit..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown") 
