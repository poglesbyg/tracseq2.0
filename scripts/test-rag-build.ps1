# 🧠 TracSeq 2.0 RAG Service Build Test Script (PowerShell)
# Tests RAG service builds with different dependency configurations

param(
    [switch]$Help,
    [switch]$Lite,
    [switch]$Standard
)

# Error handling
$ErrorActionPreference = "Stop"

Write-Host "🧠 Testing TracSeq 2.0 RAG Service Builds" -ForegroundColor Magenta
Write-Host "==========================================" -ForegroundColor Magenta
Write-Host ""

# Function to write colored output
function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

# Function to test standard RAG build
function Test-StandardRagBuild {
    Write-Info "Testing standard RAG service build..."
    
    Push-Location lab_submission_rag
    
    try {
        docker build -f Dockerfile -t tracseq-rag-standard-test .
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Standard RAG build successful!"
            Pop-Location
            return $true
        } else {
            throw "Standard RAG build failed!"
        }
    }
    catch {
        Write-Error $_.Exception.Message
        Pop-Location
        return $false
    }
}

# Function to test lightweight RAG build
function Test-LightweightRagBuild {
    Write-Info "Testing lightweight RAG service build..."
    
    Push-Location lab_submission_rag
    
    try {
        docker build -f Dockerfile.lite -t tracseq-rag-lite-test .
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Lightweight RAG build successful!"
            Pop-Location
            return $true
        } else {
            throw "Lightweight RAG build failed!"
        }
    }
    catch {
        Write-Error $_.Exception.Message
        Pop-Location
        return $false
    }
}

# Function to clean up test images
function Remove-TestImages {
    Write-Info "Cleaning up RAG test images..."
    
    $testImages = @(
        "tracseq-rag-standard-test",
        "tracseq-rag-lite-test"
    )
    
    foreach ($image in $testImages) {
        try {
            docker rmi $image 2>$null
        }
        catch {
            # Ignore errors - image might not exist
        }
    }
    
    Write-Success "RAG test images cleaned up!"
}

# Function to show troubleshooting tips
function Show-Troubleshooting {
    Write-Host ""
    Write-Warning "Troubleshooting RAG Service Build Issues:"
    Write-Host ""
    Write-Host "If you encounter dependency issues:" -ForegroundColor White
    Write-Host ""
    Write-Host "1. 📦 Use Lightweight Build:" -ForegroundColor White
    Write-Host "   docker-compose -f docker-compose.yml up --build rag-service" -ForegroundColor Cyan
    Write-Host "   # Change 'dockerfile: Dockerfile' to 'dockerfile: Dockerfile.lite'" -ForegroundColor Gray
    Write-Host ""
    Write-Host "2. 🔧 Manual Dependency Fix:" -ForegroundColor White
    Write-Host "   cd lab_submission_rag" -ForegroundColor Cyan
    Write-Host "   pip install --upgrade pip" -ForegroundColor Cyan
    Write-Host "   pip install -r requirements-lite.txt" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "3. 🏗️ Complete Clean Build:" -ForegroundColor White
    Write-Host "   docker system prune -f" -ForegroundColor Cyan
    Write-Host "   docker-compose build --no-cache rag-service" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "4. 🐛 Hash Verification Issues:" -ForegroundColor White
    Write-Host "   # Use requirements-lite.txt (already fixed)" -ForegroundColor Gray
    Write-Host "   # or disable hash checking with --trusted-host flags" -ForegroundColor Gray
    Write-Host ""
}

# Function to show help
function Show-Help {
    Write-Host "TracSeq 2.0 RAG Service Build Test Script (PowerShell)" -ForegroundColor Magenta
    Write-Host ""
    Write-Host "This script tests RAG service Docker builds and provides alternatives for dependency issues."
    Write-Host ""
    Write-Host "Usage:"
    Write-Host "  .\scripts\test-rag-build.ps1           # Run RAG build tests" -ForegroundColor Yellow
    Write-Host "  .\scripts\test-rag-build.ps1 -Help     # Show this help" -ForegroundColor Yellow
    Write-Host "  .\scripts\test-rag-build.ps1 -Lite     # Test lightweight build only" -ForegroundColor Yellow
    Write-Host "  .\scripts\test-rag-build.ps1 -Standard # Test standard build only" -ForegroundColor Yellow
    Write-Host ""
}

# Main test function
function Start-RagBuildTests {
    Write-Info "Starting RAG service build tests..."
    Write-Host ""
    
    try {
        # Try standard build first
        if (Test-StandardRagBuild) {
            Write-Host ""
            Write-Success "✅ Standard RAG build works - no changes needed!"
            Write-Host ""
            Write-Host "Your RAG service is ready to use with full dependencies." -ForegroundColor White
            Write-Host "To start: docker-compose up -d rag-service" -ForegroundColor White
        } else {
            Write-Host ""
            Write-Warning "⚠️ Standard RAG build failed - trying lightweight build..."
            Write-Host ""
            
            # Try lightweight build as fallback
            if (Test-LightweightRagBuild) {
                Write-Host ""
                Write-Success "✅ Lightweight RAG build successful!"
                Write-Host ""
                Write-Host "Recommendation: Use the lightweight build for better compatibility." -ForegroundColor White
                Write-Host "Edit docker-compose.yml:" -ForegroundColor White
                Write-Host "  Change: dockerfile: Dockerfile" -ForegroundColor Cyan
                Write-Host "  To:     dockerfile: Dockerfile.lite" -ForegroundColor Cyan
                Write-Host ""
            } else {
                Write-Error "❌ Both RAG builds failed!"
                Show-Troubleshooting
            }
        }
        
        # Clean up
        Remove-TestImages
        Write-Host ""
        
        Write-Info "RAG build test completed!"
    }
    catch {
        Write-Error "RAG build tests failed: $($_.Exception.Message)"
        Write-Host ""
        Write-Host "Troubleshooting tips:" -ForegroundColor Yellow
        Write-Host "  1. Ensure Docker Desktop is running" -ForegroundColor White
        Write-Host "  2. Check available disk space (need 5GB+)" -ForegroundColor White
        Write-Host "  3. Verify internet connection for downloads" -ForegroundColor White
        Write-Host "  4. Try the lightweight build: -Lite flag" -ForegroundColor White
        exit 1
    }
}

# Handle script parameters
if ($Help) {
    Show-Help
    exit 0
}

if ($Lite) {
    Write-Info "Testing lightweight RAG build only..."
    Test-LightweightRagBuild
    Remove-TestImages
    exit 0
}

if ($Standard) {
    Write-Info "Testing standard RAG build only..."
    Test-StandardRagBuild
    Remove-TestImages
    exit 0
}

# Run main function
Start-RagBuildTests 
