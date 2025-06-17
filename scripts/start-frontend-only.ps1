# TracSeq 2.0 Frontend-Only Development Script
# Starts frontend and RAG service (no Rust backend required)

Write-Host "🚀 Starting TracSeq 2.0 Frontend Development Mode..." -ForegroundColor Green
Write-Host "ℹ️ This mode runs without the Rust backend - using mock authentication" -ForegroundColor Cyan

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

# Check prerequisites (only Node.js and Python needed)
Write-Host "📋 Checking prerequisites..." -ForegroundColor Yellow

if (!(Get-Command python -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Python not found. Please install Python 3.9 or higher" -ForegroundColor Red
    exit 1
}

if (!(Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Node.js not found. Please install Node.js 18 or higher" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Prerequisites check passed" -ForegroundColor Green

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
Write-Host "🎉 TracSeq 2.0 Frontend Development Mode Started!" -ForegroundColor Green
Write-Host ""
Write-Host "📱 Services Running:" -ForegroundColor White
Write-Host "   ✅ Frontend (React/Vite): http://localhost:5173" -ForegroundColor Green
Write-Host "   ✅ RAG Service (Python): http://localhost:8000" -ForegroundColor Green
Write-Host "   ⚠️ Backend API (Rust): Not available - using mock auth" -ForegroundColor Yellow
Write-Host ""
Write-Host "🔧 Features Available:" -ForegroundColor White
Write-Host "   ✅ Document upload and RAG processing" -ForegroundColor Green
Write-Host "   ✅ AI-powered document extraction" -ForegroundColor Green
Write-Host "   ✅ Frontend UI with mock authentication" -ForegroundColor Green
Write-Host "   ⚠️ Sample management (limited - no database)" -ForegroundColor Yellow
Write-Host ""
Write-Host "📚 Next Steps:" -ForegroundColor White
Write-Host "   • Open: http://localhost:5173" -ForegroundColor Cyan
Write-Host "   • Upload documents for RAG processing" -ForegroundColor Cyan
Write-Host "   • To get full functionality: Install Rust and run ./start-dev.ps1" -ForegroundColor Yellow
Write-Host ""
Write-Host "🛠️ To Install Rust (Optional):" -ForegroundColor White
Write-Host "   1. Visit: https://rustup.rs/" -ForegroundColor Cyan
Write-Host "   2. Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" -ForegroundColor Gray
Write-Host "   3. Restart terminal and run: ./start-dev.ps1" -ForegroundColor Gray
Write-Host ""

# Wait for user input before closing
Write-Host "Press any key to exit..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown") 
