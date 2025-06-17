# 🤖 Ollama Model Initialization Script for TracSeq 2.0 (PowerShell)
# This script downloads and initializes the required model for local deployment

param(
    [switch]$Help,
    [switch]$TestOnly,
    [string]$ModelName = $env:OLLAMA_MODEL,
    [string]$OllamaUrl = $env:OLLAMA_BASE_URL
)

# Set defaults if not provided
if (-not $ModelName) { $ModelName = "llama3.2:3b" }
if (-not $OllamaUrl) { $OllamaUrl = "http://localhost:11434" }

# Error handling
$ErrorActionPreference = "Stop"

# Function to write colored output
function Write-Info {
    param([string]$Message)
    Write-Host $Message -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host $Message -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host $Message -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host $Message -ForegroundColor Red
}

# Function to check if Ollama is ready
function Wait-ForOllama {
    Write-Info "⏳ Waiting for Ollama service to be ready..."
    $maxAttempts = 30
    $attempt = 1
    
    while ($attempt -le $maxAttempts) {
        try {
            $response = Invoke-RestMethod -Uri "$OllamaUrl/api/version" -Method Get -TimeoutSec 5
            if ($response) {
                Write-Success "✅ Ollama service is ready!"
                return
            }
        }
        catch {
            Write-Host "   Attempt $attempt/$maxAttempts - Ollama not ready yet..." -ForegroundColor Yellow
            Start-Sleep -Seconds 10
            $attempt++
        }
    }
    
    Write-Error "❌ Ollama service failed to start after $maxAttempts attempts"
    Write-Host "Please check if the Ollama container is running:" -ForegroundColor Red
    Write-Host "   docker-compose ps ollama" -ForegroundColor White
    Write-Host "   docker-compose logs ollama" -ForegroundColor White
    exit 1
}

# Function to download model
function Download-Model {
    Write-Info "📥 Downloading model: $ModelName"
    Write-Host "   This may take a few minutes depending on your internet connection..." -ForegroundColor Gray
    Write-Host ""
    
    try {
        $body = @{
            name = $ModelName
        } | ConvertTo-Json
        
        $response = Invoke-RestMethod -Uri "$OllamaUrl/api/pull" -Method Post -Body $body -ContentType "application/json" -TimeoutSec 1800
        
        Write-Host ""
        Write-Success "✅ Model $ModelName downloaded successfully!"
    }
    catch {
        Write-Host ""
        Write-Error "❌ Failed to download model $ModelName"
        Write-Host "Please check your internet connection and try again." -ForegroundColor Red
        Write-Host "Error details: $($_.Exception.Message)" -ForegroundColor Red
        exit 1
    }
}

# Function to test model
function Test-Model {
    Write-Host ""
    Write-Info "🧪 Testing model functionality..."
    
    $testPrompt = "Hello, this is a test. Please respond with 'Model is working correctly.'"
    
    try {
        $body = @{
            model = $ModelName
            prompt = $testPrompt
            stream = $false
        } | ConvertTo-Json
        
        $response = Invoke-RestMethod -Uri "$OllamaUrl/api/generate" -Method Post -Body $body -ContentType "application/json" -TimeoutSec 60
        
        if ($response.response) {
            Write-Success "✅ Model test successful!"
            Write-Host "   Response: $($response.response)" -ForegroundColor White
            Write-Host ""
            Write-Success "🎉 Ollama is ready for TracSeq 2.0!"
            Write-Host ""
            Write-Host "📊 Model Information:" -ForegroundColor Cyan
            Write-Host "   Model: $ModelName" -ForegroundColor White
            Write-Host "   Endpoint: $OllamaUrl" -ForegroundColor White
            Write-Host "   Status: Ready for RAG processing" -ForegroundColor White
        }
        else {
            Write-Warning "⚠️  Model downloaded but test failed"
            Write-Host "   The model may still be initializing" -ForegroundColor Yellow
            Write-Host "   RAG service will automatically retry when ready" -ForegroundColor Yellow
        }
    }
    catch {
        Write-Warning "⚠️  Model test failed: $($_.Exception.Message)"
        Write-Host "   The model may still be initializing" -ForegroundColor Yellow
        Write-Host "   RAG service will automatically retry when ready" -ForegroundColor Yellow
    }
}

# Function to show usage information
function Show-Usage {
    Write-Host ""
    Write-Host "🔧 Usage Information:" -ForegroundColor Cyan
    Write-Host "===================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Environment Variables:"
    Write-Host "   OLLAMA_MODEL      - Model to download (default: llama3.2:3b)" -ForegroundColor White
    Write-Host "   OLLAMA_BASE_URL   - Ollama service URL (default: http://localhost:11434)" -ForegroundColor White
    Write-Host ""
    Write-Host "Docker Compose:"
    Write-Host "   Start services:   docker-compose up -d" -ForegroundColor White
    Write-Host "   View logs:        docker-compose logs ollama" -ForegroundColor White
    Write-Host "   Stop services:    docker-compose down" -ForegroundColor White
    Write-Host ""
    Write-Host "Manual Model Management:"
    Write-Host "   List models:      ollama list" -ForegroundColor White
    Write-Host "   Pull model:       ollama pull $ModelName" -ForegroundColor White
    Write-Host "   Remove model:     ollama rm $ModelName" -ForegroundColor White
    Write-Host ""
}

# Function to show help
function Show-Help {
    Write-Host "Ollama Model Initialization Script for TracSeq 2.0" -ForegroundColor Magenta
    Write-Host ""
    Write-Host "This script downloads and tests the required LLM model for local inference."
    Write-Host ""
    Write-Host "Parameters:"
    Write-Host "   -ModelName    : Model to download (default: llama3.2:3b)" -ForegroundColor White
    Write-Host "   -OllamaUrl    : Ollama service URL (default: http://localhost:11434)" -ForegroundColor White
    Write-Host "   -TestOnly     : Only test existing model, don't download" -ForegroundColor White
    Write-Host "   -Help         : Show this help message" -ForegroundColor White
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "   .\ollama-init.ps1" -ForegroundColor Yellow
    Write-Host "   .\ollama-init.ps1 -ModelName 'llama2:7b'" -ForegroundColor Yellow
    Write-Host "   .\ollama-init.ps1 -TestOnly" -ForegroundColor Yellow
    Write-Host ""
    Show-Usage
}

# Main execution function
function Start-OllamaInit {
    Write-Host "🤖 Initializing Ollama for TracSeq 2.0" -ForegroundColor Magenta
    Write-Host "======================================" -ForegroundColor Magenta
    Write-Host ""
    
    Write-Host "Configuration:"
    Write-Host "   Model: $ModelName" -ForegroundColor White
    Write-Host "   Ollama URL: $OllamaUrl" -ForegroundColor White
    Write-Host ""
    
    Wait-ForOllama
    
    if (-not $TestOnly) {
        Download-Model
    }
    
    Test-Model
    Show-Usage
    
    Write-Success "✨ Ollama initialization complete!"
    Write-Host "   Your TracSeq 2.0 system is ready for AI-powered document processing!" -ForegroundColor White
}

# Handle script parameters
if ($Help) {
    Show-Help
    exit 0
}

if ($TestOnly) {
    Write-Info "🧪 Testing existing model only..."
    Wait-ForOllama
    Test-Model
    exit 0
}

# Run main function
try {
    Start-OllamaInit
}
catch {
    Write-Error "Script failed: $($_.Exception.Message)"
    exit 1
} 
