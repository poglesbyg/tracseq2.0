# 🔧 TracSeq 2.0 Build Test Script (PowerShell)
# Tests Docker builds to ensure they work correctly

param(
    [switch]$Help
)

# Error handling
$ErrorActionPreference = "Stop"

Write-Host "🔧 Testing TracSeq 2.0 Docker Builds" -ForegroundColor Magenta
Write-Host "=====================================" -ForegroundColor Magenta
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

# Function to test backend build
function Test-BackendBuild {
    Write-Info "Testing lab_manager backend build..."
    
    Push-Location lab_manager
    
    try {
        # Test production build
        docker build -f Dockerfile -t tracseq-backend-test .
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Backend production build successful!"
        } else {
            throw "Backend production build failed!"
        }
        
        # Test development build
        docker build -f Dockerfile.dev -t tracseq-backend-dev-test .
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Backend development build successful!"
        } else {
            throw "Backend development build failed!"
        }
    }
    catch {
        Write-Error $_.Exception.Message
        Pop-Location
        throw
    }
    finally {
        Pop-Location
    }
}

# Function to test frontend build
function Test-FrontendBuild {
    Write-Info "Testing frontend build..."
    
    Push-Location lab_manager\frontend
    
    try {
        # Test production build
        docker build -f Dockerfile -t tracseq-frontend-test .
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Frontend production build successful!"
        } else {
            throw "Frontend production build failed!"
        }
        
        # Test development build
        docker build -f Dockerfile.dev -t tracseq-frontend-dev-test .
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Frontend development build successful!"
        } else {
            throw "Frontend development build failed!"
        }
    }
    catch {
        Write-Error $_.Exception.Message
        Pop-Location
        throw
    }
    finally {
        Pop-Location
    }
}

# Function to test RAG service build
function Test-RagBuild {
    Write-Info "Testing RAG service build..."
    
    Push-Location lab_submission_rag
    
    try {
        docker build -f Dockerfile -t tracseq-rag-test .
        if ($LASTEXITCODE -eq 0) {
            Write-Success "RAG service build successful!"
        } else {
            throw "RAG service build failed!"
        }
    }
    catch {
        Write-Error $_.Exception.Message
        Pop-Location
        throw
    }
    finally {
        Pop-Location
    }
}

# Function to clean up test images
function Remove-TestImages {
    Write-Info "Cleaning up test images..."
    
    $testImages = @(
        "tracseq-backend-test",
        "tracseq-backend-dev-test", 
        "tracseq-frontend-test",
        "tracseq-frontend-dev-test",
        "tracseq-rag-test"
    )
    
    foreach ($image in $testImages) {
        try {
            docker rmi $image 2>$null
        }
        catch {
            # Ignore errors - image might not exist
        }
    }
    
    Write-Success "Test images cleaned up!"
}

# Function to test docker-compose syntax
function Test-ComposeSyntax {
    Write-Info "Testing docker-compose file syntax..."
    
    docker-compose config --quiet
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Docker Compose syntax is valid!"
    } else {
        throw "Docker Compose syntax error!"
    }
}

# Function to show help
function Show-Help {
    Write-Host "TracSeq 2.0 Build Test Script (PowerShell)" -ForegroundColor Magenta
    Write-Host ""
    Write-Host "This script tests all Docker builds to ensure they work correctly."
    Write-Host ""
    Write-Host "Usage:"
    Write-Host "  .\scripts\test-build.ps1           # Run all build tests" -ForegroundColor Yellow
    Write-Host "  .\scripts\test-build.ps1 -Help     # Show this help" -ForegroundColor Yellow
    Write-Host ""
}

# Main test function
function Start-BuildTests {
    Write-Info "Starting build tests..."
    Write-Host ""
    
    try {
        # Test docker-compose syntax first
        Test-ComposeSyntax
        Write-Host ""
        
        # Test individual service builds
        Test-BackendBuild
        Write-Host ""
        
        Test-FrontendBuild
        Write-Host ""
        
        Test-RagBuild
        Write-Host ""
        
        # Clean up
        Remove-TestImages
        Write-Host ""
        
        Write-Success "🎉 All build tests passed!"
        Write-Host ""
        Write-Host "Next steps:"
        Write-Host "  1. Start services: docker-compose up -d" -ForegroundColor White
        Write-Host "  2. Initialize Ollama: .\deploy\azure\ollama-init.ps1" -ForegroundColor White
        Write-Host "  3. Test application: http://localhost:5173" -ForegroundColor White
    }
    catch {
        Write-Error "Build tests failed: $($_.Exception.Message)"
        Write-Host ""
        Write-Host "Troubleshooting tips:" -ForegroundColor Yellow
        Write-Host "  1. Ensure Docker Desktop is running" -ForegroundColor White
        Write-Host "  2. Check available disk space (need 5GB+)" -ForegroundColor White
        Write-Host "  3. Verify internet connection for downloads" -ForegroundColor White
        Write-Host "  4. Check Docker logs: docker-compose logs" -ForegroundColor White
        exit 1
    }
}

# Handle script parameters
if ($Help) {
    Show-Help
    exit 0
}

# Run main function
Start-BuildTests 
