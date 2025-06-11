# TracSeq 2.0 Lab Manager Runner Script for Windows (PowerShell)
# This script runs from within the lab_manager directory

param(
    [Parameter(Position=0)]
    [string]$Command = "help",
    
    [Parameter(Position=1)]
    [string]$Service = "",
    
    [switch]$Force
)

# Set console title
$Host.UI.RawUI.WindowTitle = "TracSeq 2.0 Lab Manager"

# Colors and formatting
function Write-Info { param($Message) Write-Host "[INFO] $Message" -ForegroundColor Blue }
function Write-Success { param($Message) Write-Host "[SUCCESS] $Message" -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host "[WARNING] $Message" -ForegroundColor Yellow }
function Write-Error-Custom { param($Message) Write-Host "[ERROR] $Message" -ForegroundColor Red }
function Write-Header { 
    param($Title)
    Write-Host "================================" -ForegroundColor Blue
    Write-Host "  TracSeq 2.0 - $Title" -ForegroundColor Blue
    Write-Host "================================" -ForegroundColor Blue
}

# Function to check if Docker is running and available
function Test-Docker {
    try {
        $null = Get-Command docker -ErrorAction Stop
        $dockerInfo = docker info 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Docker is running"
            return $true
        }
    }
    catch { }
    
    Write-Error-Custom "Docker is not running or not found"
    Write-Info "Please install Docker Desktop and ensure it's running"
    Write-Info "Download Docker Desktop from: https://www.docker.com/products/docker-desktop"
    return $false
}

# Function to get the correct docker-compose command
function Get-DockerComposeCommand {
    # Try docker compose first (newer versions)
    try {
        $null = docker compose version 2>$null
        if ($LASTEXITCODE -eq 0) {
            return "docker compose"
        }
    }
    catch { }
    
    # Try docker-compose (older versions)  
    try {
        $null = Get-Command docker-compose -ErrorAction Stop
        $null = docker-compose version 2>$null
        if ($LASTEXITCODE -eq 0) {
            return "docker-compose"
        }
    }
    catch { }
    
    # Fallback to docker compose
    return "docker compose"
}

# Function to check if required files exist
function Test-RequiredFiles {
    if (-not (Test-Path "docker-compose.yml")) {
        Write-Error-Custom "File not found: docker-compose.yml"
        Write-Info "Please run this script from the lab_manager directory"
        return $false
    }
    
    if (-not (Test-Path "Cargo.toml")) {
        Write-Error-Custom "File not found: Cargo.toml"
        Write-Info "Please run this script from the lab_manager directory"
        return $false
    }
    
    Write-Success "All required files found"
    return $true
}

# Function to create required directories
function New-RequiredDirectories {
    Write-Info "Creating required directories..."
    
    $directories = @(
        "storage",
        "uploads",
        "target"
    )
    
    foreach ($dir in $directories) {
        if (-not (Test-Path $dir)) {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
        }
    }
    
    Write-Success "Directories created"
}

# Function to start services in production mode
function Start-ProductionServices {
    Write-Header "Starting Production Mode"
    
    if (-not (Test-Docker)) { return }
    if (-not (Test-RequiredFiles)) { return }
    
    $composeCmd = Get-DockerComposeCommand
    New-RequiredDirectories
    
    Write-Info "Starting Lab Manager services (Production)..."
    Invoke-Expression "$composeCmd up -d frontend app db"
    
    Write-Success "Lab Manager services started successfully!"
    Write-Host ""
    Write-Host "Services available at:" -ForegroundColor Green
    Write-Host "  Frontend: http://localhost:8080" -ForegroundColor Cyan
    Write-Host "  Backend:  http://localhost:3001" -ForegroundColor Cyan
    Write-Host "  Database: localhost:5433" -ForegroundColor Cyan
}

# Function to start services in development mode
function Start-DevelopmentServices {
    Write-Header "Starting Development Mode"
    
    if (-not (Test-Docker)) { return }
    if (-not (Test-RequiredFiles)) { return }
    
    $composeCmd = Get-DockerComposeCommand
    New-RequiredDirectories
    
    Write-Info "Starting Lab Manager services (Development)..."
    Invoke-Expression "$composeCmd up -d frontend-dev dev db"
    
    Write-Success "Lab Manager services started successfully!"
    Write-Host ""
    Write-Host "Services available at:" -ForegroundColor Green
    Write-Host "  Frontend: http://localhost:5173" -ForegroundColor Cyan
    Write-Host "  Backend:  http://localhost:3000" -ForegroundColor Cyan
    Write-Host "  Database: localhost:5433" -ForegroundColor Cyan
}

# Function to run tests
function Invoke-Tests {
    Write-Header "Running Tests"
    
    if (-not (Test-Docker)) { return }
    if (-not (Test-RequiredFiles)) { return }
    
    $composeCmd = Get-DockerComposeCommand
    
    Write-Info "Setting up test environment..."
    
    # Make sure database is running
    Write-Info "Starting test database..."
    Invoke-Expression "$composeCmd up -d db"
    Start-Sleep -Seconds 5
    
    Write-Info "Building test container..."
    Invoke-Expression "$composeCmd build dev"
    
    Write-Info "Running tests in Docker container..."
    
    # Run tests in a fresh container with environment variables
    $testCommand = "$composeCmd run --rm -e TEST_DATABASE_URL=postgres://postgres:postgres@db:5432/lab_manager -e RUST_LOG=info -e JWT_SECRET=test-jwt-secret-key dev cargo test -- --nocapture"
    
    try {
        Invoke-Expression $testCommand
        if ($LASTEXITCODE -eq 0) {
            Write-Success "All tests passed!"
        } else {
            Write-Error-Custom "Some tests failed. Check the output above."
        }
    }
    catch {
        Write-Error-Custom "Test execution failed: $($_.Exception.Message)"
    }
}

# Function to run specific test categories
function Invoke-SpecificTests {
    param($TestCategory)
    
    Write-Header "Running $TestCategory Tests"
    
    if (-not (Test-Docker)) { return }
    if (-not (Test-RequiredFiles)) { return }
    
    $composeCmd = Get-DockerComposeCommand
    
    # Make sure database is running
    Invoke-Expression "$composeCmd up -d db"
    Start-Sleep -Seconds 3
    
    Write-Info "Building test container..."
    Invoke-Expression "$composeCmd build dev"
    
    $testCommand = "$composeCmd run --rm -e TEST_DATABASE_URL=postgres://postgres:postgres@db:5432/lab_manager -e RUST_LOG=info -e JWT_SECRET=test-jwt-secret-key dev cargo test $TestCategory -- --nocapture"
    
    try {
        Invoke-Expression $testCommand
        if ($LASTEXITCODE -eq 0) {
            Write-Success "$TestCategory tests passed!"
        } else {
            Write-Error-Custom "Some $TestCategory tests failed."
        }
    }
    catch {
        Write-Error-Custom "Test execution failed: $($_.Exception.Message)"
    }
}

# Function to stop all services
function Stop-AllServices {
    Write-Header "Stopping Services"
    
    Write-Info "Stopping all services..."
    
    $composeCmd = Get-DockerComposeCommand
    Invoke-Expression "$composeCmd down"
    
    Write-Success "All services stopped"
}

# Function to show service status
function Show-ServiceStatus {
    Write-Header "Service Status"
    
    if (-not (Test-Docker)) { return }
    
    $composeCmd = Get-DockerComposeCommand
    Invoke-Expression "$composeCmd ps"
}

# Function to show logs
function Show-ServiceLogs {
    param($ServiceName)
    
    if (-not (Test-Docker)) { return }
    
    $composeCmd = Get-DockerComposeCommand
    
    if ($ServiceName) {
        Write-Info "Showing logs for $ServiceName..."
        Invoke-Expression "$composeCmd logs -f $ServiceName"
    } else {
        Write-Info "Showing all service logs..."
        Invoke-Expression "$composeCmd logs -f"
    }
}

# Function to rebuild services
function Invoke-RebuildServices {
    Write-Header "Rebuilding Services"
    
    if (-not (Test-Docker)) { return }
    
    Write-Info "Rebuilding all services..."
    
    $composeCmd = Get-DockerComposeCommand
    Invoke-Expression "$composeCmd build --no-cache"
    
    Write-Success "All services rebuilt"
}

# Function to clean Docker resources
function Clear-DockerResources {
    Write-Header "Cleaning Docker Resources"
    
    Stop-AllServices
    
    Write-Info "Cleaning up Docker resources..."
    docker system prune -f
    
    Write-Success "Docker cleanup completed"
}

# Function to show help
function Show-Help {
    Write-Host "TracSeq 2.0 Lab Manager Runner Script for Windows" -ForegroundColor Blue
    Write-Host ""
    Write-Host "Usage: .\run.ps1 [COMMAND] [SERVICE/TEST_TYPE]" -ForegroundColor Green
    Write-Host ""
    Write-Host "Service Commands:" -ForegroundColor Yellow
    Write-Host "  start-prod     Start services in production mode"
    Write-Host "  start-dev      Start services in development mode"
    Write-Host "  stop           Stop all services"
    Write-Host "  restart-prod   Restart services in production mode"
    Write-Host "  restart-dev    Restart services in development mode"
    Write-Host "  status         Show status of all services"
    Write-Host "  logs [service] Show logs (optional: specify service name)"
    Write-Host "  rebuild        Rebuild all Docker images"
    Write-Host "  clean          Clean up Docker resources"
    Write-Host ""
    Write-Host "Test Commands:" -ForegroundColor Yellow
    Write-Host "  test           Run all tests"
    Write-Host "  test-auth      Run authentication tests"
    Write-Host "  test-validation Run validation tests"
    Write-Host "  test-storage   Run storage tests"
    Write-Host "  test-sequencing Run sequencing tests"
    Write-Host "  test-templates Run template tests"
    Write-Host ""
    Write-Host "Services:" -ForegroundColor Yellow
    Write-Host "  - Frontend: http://localhost:8080 (prod) or http://localhost:5173 (dev)"
    Write-Host "  - Backend:  http://localhost:3001 (prod) or http://localhost:3000 (dev)"
    Write-Host "  - Database: localhost:5433"
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Cyan
    Write-Host "  .\run.ps1 start-dev"
    Write-Host "  .\run.ps1 test"
    Write-Host "  .\run.ps1 test-auth"
    Write-Host "  .\run.ps1 logs dev"
}

# Function to open web interfaces
function Open-WebInterfaces {
    param($Mode)
    
    if ($Mode -eq "dev") {
        Start-Process "http://localhost:5173"  # Frontend Dev
        Start-Process "http://localhost:3000"  # Backend Dev
    }
    else {
        Start-Process "http://localhost:8080"  # Frontend Prod
        Start-Process "http://localhost:3001"  # Backend Prod
    }
}

# Main script execution
switch ($Command.ToLower()) {
    "start-prod" { 
        Start-ProductionServices 
        if ($?) { Open-WebInterfaces "prod" }
    }
    "start-dev" { 
        Start-DevelopmentServices 
        if ($?) { Open-WebInterfaces "dev" }
    }
    "stop" { Stop-AllServices }
    "restart-prod" { 
        Stop-AllServices
        Start-Sleep -Seconds 2
        Start-ProductionServices 
    }
    "restart-dev" { 
        Stop-AllServices
        Start-Sleep -Seconds 2
        Start-DevelopmentServices 
    }
    "status" { Show-ServiceStatus }
    "logs" { Show-ServiceLogs $Service }
    "rebuild" { Invoke-RebuildServices }
    "clean" { Clear-DockerResources }
    "test" { Invoke-Tests }
    "test-auth" { Invoke-SpecificTests "auth" }
    "test-validation" { Invoke-SpecificTests "validation" }
    "test-storage" { Invoke-SpecificTests "storage" }
    "test-sequencing" { Invoke-SpecificTests "sequencing" }
    "test-templates" { Invoke-SpecificTests "template" }
    "help" { Show-Help }
    "" { Show-Help }
    default { 
        Write-Error-Custom "Unknown command: $Command"
        Show-Help
        exit 1
    }
} 
