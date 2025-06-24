# TracSeq 2.0 Development Environment Setup Script (PowerShell)
# This script sets up all necessary environment variables for development

Write-Host "Setting up TracSeq 2.0 Development Environment..." -ForegroundColor Green

# Load environment variables from services.env
$envFile = Join-Path $PSScriptRoot "..\deploy\services.env"

if (Test-Path $envFile) {
    Write-Host "Loading environment variables from $envFile" -ForegroundColor Yellow
    
    Get-Content $envFile | ForEach-Object {
        if ($_ -match "^([^#][^=]+)=(.*)$") {
            $name = $matches[1].Trim()
            $value = $matches[2].Trim()
            
            # Skip empty lines and comments
            if ($name -and $value) {
                [Environment]::SetEnvironmentVariable($name, $value, "Process")
                Write-Host "Set $name" -ForegroundColor Cyan
            }
        }
    }
} else {
    Write-Host "Environment file not found: $envFile" -ForegroundColor Red
    Write-Host "Creating default environment variables..." -ForegroundColor Yellow
    
    # Core database configuration
    $env:DATABASE_URL = "postgresql://postgres:postgres@localhost:5433/lab_manager"
    $env:SQLX_OFFLINE = "false"
    $env:RUST_LOG = "info"
    
    # Service ports
    $env:LAB_MANAGER_PORT = "3000"
    $env:AUTH_SERVICE_PORT = "8001"
    $env:SAMPLE_SERVICE_PORT = "8002"
    $env:SEQUENCING_SERVICE_PORT = "8003"
    $env:TRANSACTION_SERVICE_PORT = "8006"
    $env:EVENT_SERVICE_PORT = "8008"
    
    # Security
    $env:JWT_SECRET = "tracseq_jwt_secret_2024_change_in_production"
    
    Write-Host "Default environment variables set" -ForegroundColor Green
}

# Display current environment
Write-Host "`nCurrent Environment Configuration:" -ForegroundColor Green
Write-Host "DATABASE_URL: $env:DATABASE_URL" -ForegroundColor White
Write-Host "RUST_LOG: $env:RUST_LOG" -ForegroundColor White
Write-Host "LAB_MANAGER_PORT: $env:LAB_MANAGER_PORT" -ForegroundColor White
Write-Host "SEQUENCING_SERVICE_PORT: $env:SEQUENCING_SERVICE_PORT" -ForegroundColor White

Write-Host "`nEnvironment setup complete!" -ForegroundColor Green
Write-Host "You can now run: cargo build --workspace" -ForegroundColor Yellow 
