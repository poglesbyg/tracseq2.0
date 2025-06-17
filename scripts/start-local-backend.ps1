# TracSeq 2.0 Local Backend Startup Script
# Starts the Rust backend with local development configuration

Write-Host "🦀 Starting TracSeq 2.0 Local Backend..." -ForegroundColor Green

# Set environment variables for local development
$env:DATABASE_URL = "sqlite:./lab_manager.db"
$env:JWT_SECRET = "dev-jwt-secret-change-in-production"
$env:RUST_LOG = "info"
$env:HOST = "0.0.0.0"
$env:PORT = "3001"
$env:STORAGE_PATH = "./storage"
$env:RAG_SERVICE_URL = "http://localhost:8000"
$env:RUST_ENV = "development"
$env:RUST_BACKTRACE = "1"
$env:CORS_ORIGINS = "http://localhost:5173,http://localhost:3000"

Write-Host "🔧 Environment Configuration:" -ForegroundColor Cyan
Write-Host "   Database: SQLite (./lab_manager.db)" -ForegroundColor White
Write-Host "   Port: 3001" -ForegroundColor White
Write-Host "   Storage: ./storage" -ForegroundColor White
Write-Host "   RAG Service: http://localhost:8000" -ForegroundColor White
Write-Host ""

# Create storage directory if it doesn't exist
if (-not (Test-Path "./storage")) {
    New-Item -ItemType Directory -Path "./storage" -Force | Out-Null
    Write-Host "✅ Created storage directory" -ForegroundColor Green
}

Write-Host "🚀 Starting backend server..." -ForegroundColor Yellow
Write-Host "   Access at: http://localhost:3001" -ForegroundColor Cyan
Write-Host "   Health check: http://localhost:3001/health" -ForegroundColor Cyan
Write-Host ""
Write-Host "Press Ctrl+C to stop the server" -ForegroundColor Gray
Write-Host ""

# Start the backend
cargo run 
