# TracSeq 2.0 Development Startup Script
# Starts all required services for local development

Write-Host "🚀 Starting TracSeq 2.0 Development Environment..." -ForegroundColor Green

# Function to check if a port is in use
function Test-Port {
    param([int]$Port)
    try {
        $tcpConnection = Test-NetConnection -ComputerName localhost -Port $Port -InformationLevel Quiet -WarningAction SilentlyContinue
        return $tcpConnection
    } catch {
        return $false
    }
}

# Check prerequisites
Write-Host "📋 Checking prerequisites..." -ForegroundColor Yellow

# Check if Rust is installed
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Rust/Cargo not found. Please install Rust from https://rustup.rs/" -ForegroundColor Red
    exit 1
}

# Check if Python is installed
if (!(Get-Command python -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Python not found. Please install Python 3.9 or higher" -ForegroundColor Red
    exit 1
}

# Check if Node.js is installed
if (!(Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Node.js not found. Please install Node.js 18 or higher" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Prerequisites check passed" -ForegroundColor Green

# Start services
Write-Host "🔧 Starting services..." -ForegroundColor Yellow

# Start RAG Service (Python)
Write-Host "📊 Starting RAG Service on port 8000..." -ForegroundColor Cyan
if (Test-Port -Port 8000) {
    Write-Host "✅ RAG service already running on port 8000" -ForegroundColor Green
} else {
    Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd lab_submission_rag; python api/main.py" -WindowStyle Minimized
    Start-Sleep -Seconds 3
    if (Test-Port -Port 8000) {
        Write-Host "✅ RAG service started successfully" -ForegroundColor Green
    } else {
        Write-Host "⚠️ RAG service may still be starting..." -ForegroundColor Yellow
    }
}

# Start Lab Manager Backend (Rust)
Write-Host "🦀 Starting Lab Manager Backend on port 3001..." -ForegroundColor Cyan
if (Test-Port -Port 3001) {
    Write-Host "✅ Lab Manager backend already running on port 3001" -ForegroundColor Green
} else {
    Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd lab_manager; cargo run" -WindowStyle Minimized
    Start-Sleep -Seconds 5
    if (Test-Port -Port 3001) {
        Write-Host "✅ Lab Manager backend started successfully" -ForegroundColor Green
    } else {
        Write-Host "⚠️ Lab Manager backend may still be starting..." -ForegroundColor Yellow
    }
}

# Start Frontend Development Server (React/Vite)
Write-Host "⚛️ Starting Frontend Development Server on port 5173..." -ForegroundColor Cyan
if (Test-Port -Port 5173) {
    Write-Host "✅ Frontend development server already running on port 5173" -ForegroundColor Green
} else {
    Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd lab_manager/frontend; npm run dev" -WindowStyle Minimized
    Start-Sleep -Seconds 3
    if (Test-Port -Port 5173) {
        Write-Host "✅ Frontend development server started successfully" -ForegroundColor Green
    } else {
        Write-Host "⚠️ Frontend development server may still be starting..." -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "🎉 TracSeq 2.0 Development Environment Started!" -ForegroundColor Green
Write-Host ""
Write-Host "📱 Services:" -ForegroundColor White
Write-Host "   • Frontend (React/Vite): http://localhost:5173" -ForegroundColor Cyan
Write-Host "   • Lab Manager API (Rust): http://localhost:3001" -ForegroundColor Cyan
Write-Host "   • RAG Service (Python): http://localhost:8000" -ForegroundColor Cyan
Write-Host ""
Write-Host "🔧 Development Tools:" -ForegroundColor White
Write-Host "   • API Documentation: http://localhost:3001/docs" -ForegroundColor Cyan
Write-Host "   • RAG API Docs: http://localhost:8000/docs" -ForegroundColor Cyan
Write-Host ""
Write-Host "📚 Useful Commands:" -ForegroundColor White
Write-Host "   • Stop all services: ./stop-dev.ps1" -ForegroundColor Yellow
Write-Host "   • View logs: Check the PowerShell windows" -ForegroundColor Yellow
Write-Host "   • Restart a service: Close its window and run this script again" -ForegroundColor Yellow
Write-Host ""
Write-Host "🌐 Open http://localhost:5173 in your browser to get started!" -ForegroundColor Magenta
Write-Host ""

# Wait for user input before closing
Write-Host "Press any key to exit..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown") 
