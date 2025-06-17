# TracSeq 2.0 Development Stop Script
# Stops all development services

Write-Host "🛑 Stopping TracSeq 2.0 Development Environment..." -ForegroundColor Red

# Function to stop processes on a specific port
function Stop-ProcessOnPort {
    param([int]$Port, [string]$ServiceName)
    
    try {
        $connections = Get-NetTCPConnection -LocalPort $Port -ErrorAction SilentlyContinue
        if ($connections) {
            foreach ($connection in $connections) {
                $process = Get-Process -Id $connection.OwningProcess -ErrorAction SilentlyContinue
                if ($process) {
                    Write-Host "🔪 Stopping $ServiceName (PID: $($process.Id))..." -ForegroundColor Yellow
                    Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
                    Write-Host "✅ $ServiceName stopped" -ForegroundColor Green
                } else {
                    Write-Host "⚠️ Could not find process for $ServiceName on port $Port" -ForegroundColor Yellow
                }
            }
        } else {
            Write-Host "ℹ️ $ServiceName not running on port $Port" -ForegroundColor Cyan
        }
    } catch {
        Write-Host "⚠️ Error stopping $ServiceName on port $Port`: $($_.Exception.Message)" -ForegroundColor Yellow
    }
}

# Stop all services
Write-Host "🔍 Finding and stopping services..." -ForegroundColor Yellow

# Stop Frontend (port 5173)
Stop-ProcessOnPort -Port 5173 -ServiceName "Frontend Development Server"

# Stop Lab Manager Backend (port 3001)
Stop-ProcessOnPort -Port 3001 -ServiceName "Lab Manager Backend"

# Stop RAG Service (port 8000) 
Stop-ProcessOnPort -Port 8000 -ServiceName "RAG Service"

# Also try to stop any cargo or npm processes
Write-Host "🧹 Cleaning up any remaining processes..." -ForegroundColor Yellow

try {
    Get-Process | Where-Object { $_.ProcessName -eq "cargo" -or $_.ProcessName -eq "node" -or $_.ProcessName -eq "python" } | ForEach-Object {
        $commandLine = (Get-WmiObject Win32_Process -Filter "ProcessId = $($_.Id)").CommandLine
        if ($commandLine -and ($commandLine -match "lab_manager" -or $commandLine -match "lab_submission_rag" -or $commandLine -match "vite")) {
            Write-Host "🔪 Stopping $($_.ProcessName) (PID: $($_.Id))..." -ForegroundColor Yellow
            Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue
        }
    }
} catch {
    Write-Host "⚠️ Error during cleanup: $($_.Exception.Message)" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "✅ TracSeq 2.0 Development Environment Stopped!" -ForegroundColor Green
Write-Host ""
Write-Host "📚 Next Steps:" -ForegroundColor White
Write-Host "   • To restart services: ./start-dev.ps1" -ForegroundColor Cyan
Write-Host "   • To start individual services manually:" -ForegroundColor Cyan
Write-Host "     - RAG Service: cd lab_submission_rag && python api/main.py" -ForegroundColor Gray
Write-Host "     - Backend: cd lab_manager && cargo run" -ForegroundColor Gray
Write-Host "     - Frontend: cd lab_manager/frontend && npm run dev" -ForegroundColor Gray
Write-Host ""

# Wait for user input before closing
Write-Host "Press any key to exit..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown") 
