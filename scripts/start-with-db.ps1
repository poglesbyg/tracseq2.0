# TracSeq 2.0 Backend with Database
# Starts PostgreSQL in Docker and Rust backend locally

Write-Host "🐘 Starting TracSeq 2.0 with PostgreSQL Database..." -ForegroundColor Blue

# Function to wait for Docker
function Wait-ForDocker {
    Write-Host "⏳ Waiting for Docker to be ready..." -ForegroundColor Yellow
    $maxAttempts = 30
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
        Start-Sleep -Seconds 2
        Write-Host "   Attempt $attempt/$maxAttempts..." -ForegroundColor Gray
        
    } while ($attempt -lt $maxAttempts)
    
    Write-Host "❌ Docker is not responding. Trying alternative approach..." -ForegroundColor Yellow
    return $false
}

# Check if Docker is ready
$dockerReady = Wait-ForDocker

if ($dockerReady) {
    Write-Host "🐳 Starting PostgreSQL database..." -ForegroundColor Cyan
    
    # Stop any existing postgres container
    docker stop tracseq-postgres 2>$null
    docker rm tracseq-postgres 2>$null
    
    # Start PostgreSQL container
    docker run -d `
        --name tracseq-postgres `
        -e POSTGRES_USER=postgres `
        -e POSTGRES_PASSWORD=postgres `
        -e POSTGRES_DB=lab_manager `
        -p 5432:5432 `
        postgres:15
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ PostgreSQL started successfully" -ForegroundColor Green
        Write-Host "⏳ Waiting for database to be ready..." -ForegroundColor Yellow
        Start-Sleep -Seconds 10
        
        # Set environment variables for PostgreSQL
        $env:DATABASE_URL = "postgres://postgres:postgres@localhost:5432/lab_manager"
        $env:JWT_SECRET = "dev-jwt-secret-change-in-production"
        $env:RUST_LOG = "info"
        $env:HOST = "0.0.0.0"
        $env:PORT = "3001"
        $env:STORAGE_PATH = "./storage"
        $env:RAG_SERVICE_URL = "http://localhost:8000"
        $env:RUST_ENV = "development"
        $env:RUST_BACKTRACE = "1"
        
        Write-Host "🔧 Environment Configuration:" -ForegroundColor Cyan
        Write-Host "   Database: PostgreSQL (localhost:5432)" -ForegroundColor White
        Write-Host "   Port: 3001" -ForegroundColor White
        Write-Host "   Storage: ./storage" -ForegroundColor White
        Write-Host ""
    } else {
        Write-Host "❌ Failed to start PostgreSQL" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "⚠️ Docker not available, trying without database..." -ForegroundColor Yellow
    Write-Host "This might fail - consider installing Docker Desktop" -ForegroundColor Yellow
    
    # Try without database (will likely fail)
    $env:DATABASE_URL = ""
    $env:JWT_SECRET = "dev-jwt-secret-change-in-production"
    $env:RUST_LOG = "info"
    $env:HOST = "0.0.0.0"
    $env:PORT = "3001"
    $env:STORAGE_PATH = "./storage"
    $env:RAG_SERVICE_URL = "http://localhost:8000"
}

# Create storage directory
if (-not (Test-Path "./storage")) {
    New-Item -ItemType Directory -Path "./storage" -Force | Out-Null
    Write-Host "✅ Created storage directory" -ForegroundColor Green
}

Write-Host "🦀 Starting Rust backend..." -ForegroundColor Green
Write-Host "   Access at: http://localhost:3001" -ForegroundColor Cyan
Write-Host "   Health check: http://localhost:3001/health" -ForegroundColor Cyan
Write-Host ""
Write-Host "Press Ctrl+C to stop" -ForegroundColor Gray
Write-Host ""

# Start the backend
cargo run 
