# TracSeq 2.0 Integration Test Script
# Tests communication between RAG system and Lab Manager

Write-Host "🧪 TracSeq 2.0 Integration Tests" -ForegroundColor Blue
Write-Host "================================" -ForegroundColor Blue

$testResults = @()

function Test-Service {
    param(
        [string]$Name,
        [string]$Url,
        [int]$ExpectedStatus = 200,
        [int]$TimeoutSec = 10
    )
    
    Write-Host "Testing $Name... " -NoNewline
    
    try {
        $response = Invoke-WebRequest -Uri $Url -TimeoutSec $TimeoutSec -ErrorAction Stop
        
        if ($response.StatusCode -eq $ExpectedStatus) {
            Write-Host "✅ PASS" -ForegroundColor Green
            return @{ Name = $Name; Status = "PASS"; Details = "HTTP $($response.StatusCode)" }
        } else {
            Write-Host "❌ FAIL" -ForegroundColor Red
            return @{ Name = $Name; Status = "FAIL"; Details = "HTTP $($response.StatusCode), expected $ExpectedStatus" }
        }
    }
    catch {
        Write-Host "❌ FAIL" -ForegroundColor Red
        return @{ Name = $Name; Status = "FAIL"; Details = $_.Exception.Message }
    }
}

function Test-DatabaseConnection {
    Write-Host "Testing PostgreSQL connection... " -NoNewline
    
    try {
        $result = docker exec tracseq_postgres pg_isready -U postgres 2>$null
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ PASS" -ForegroundColor Green
            return @{ Name = "PostgreSQL"; Status = "PASS"; Details = "Database ready" }
        } else {
            Write-Host "❌ FAIL" -ForegroundColor Red
            return @{ Name = "PostgreSQL"; Status = "FAIL"; Details = "Database not ready" }
        }
    }
    catch {
        Write-Host "❌ FAIL" -ForegroundColor Red
        return @{ Name = "PostgreSQL"; Status = "FAIL"; Details = $_.Exception.Message }
    }
}

function Test-RagLabManagerIntegration {
    Write-Host "Testing RAG-Lab Manager integration... " -NoNewline
    
    try {
        # Test the RAG service endpoint from Lab Manager perspective
        $response = Invoke-WebRequest -Uri "http://localhost:3000/api/samples/rag/status" -TimeoutSec 15 -ErrorAction Stop
        
        if ($response.StatusCode -eq 200) {
            $content = $response.Content | ConvertFrom-Json
            Write-Host "✅ PASS" -ForegroundColor Green
            return @{ Name = "RAG-Lab Manager Integration"; Status = "PASS"; Details = "Integration working" }
        } else {
            Write-Host "❌ FAIL" -ForegroundColor Red
            return @{ Name = "RAG-Lab Manager Integration"; Status = "FAIL"; Details = "HTTP $($response.StatusCode)" }
        }
    }
    catch {
        Write-Host "❌ FAIL" -ForegroundColor Red
        return @{ Name = "RAG-Lab Manager Integration"; Status = "FAIL"; Details = $_.Exception.Message }
    }
}

function Test-OllamaModel {
    Write-Host "Testing Ollama model availability... " -NoNewline
    
    try {
        # Test if Ollama has models
        $response = Invoke-RestMethod -Uri "http://localhost:11434/api/tags" -TimeoutSec 10 -ErrorAction Stop
        
        if ($response.models -and $response.models.Count -gt 0) {
            $modelNames = $response.models | ForEach-Object { $_.name }
            Write-Host "✅ PASS" -ForegroundColor Green
            return @{ Name = "Ollama Models"; Status = "PASS"; Details = "Models: $($modelNames -join ', ')" }
        } else {
            Write-Host "⚠️ WARNING" -ForegroundColor Yellow
            return @{ Name = "Ollama Models"; Status = "WARNING"; Details = "No models installed yet" }
        }
    }
    catch {
        Write-Host "❌ FAIL" -ForegroundColor Red
        return @{ Name = "Ollama Models"; Status = "FAIL"; Details = $_.Exception.Message }
    }
}

# Wait for services to be ready
Write-Host "⏳ Waiting for services to start..." -ForegroundColor Yellow
Start-Sleep -Seconds 15

Write-Host ""
Write-Host "🔍 Running integration tests..." -ForegroundColor Blue
Write-Host ""

# Test core services
$testResults += Test-DatabaseConnection
$testResults += Test-Service -Name "Ollama API" -Url "http://localhost:11434/api/version"
$testResults += Test-Service -Name "RAG Service Health" -Url "http://localhost:8000/health"
$testResults += Test-Service -Name "Lab Manager Health" -Url "http://localhost:3000/health"

# Test frontend availability
$testResults += Test-Service -Name "Frontend (Dev)" -Url "http://localhost:5173" -TimeoutSec 20
$testResults += Test-Service -Name "Frontend (Prod)" -Url "http://localhost:8080" -TimeoutSec 20

# Test specific integrations
$testResults += Test-OllamaModel
$testResults += Test-RagLabManagerIntegration

# Test RAG service endpoints
Write-Host ""
Write-Host "🔬 Testing RAG service endpoints..." -ForegroundColor Blue
$testResults += Test-Service -Name "RAG System Info" -Url "http://localhost:8000/system-info"
$testResults += Test-Service -Name "RAG Submissions" -Url "http://localhost:8000/submissions"

Write-Host ""
Write-Host "📊 TEST RESULTS SUMMARY" -ForegroundColor Blue
Write-Host "========================" -ForegroundColor Blue

$passCount = ($testResults | Where-Object { $_.Status -eq "PASS" }).Count
$failCount = ($testResults | Where-Object { $_.Status -eq "FAIL" }).Count
$warnCount = ($testResults | Where-Object { $_.Status -eq "WARNING" }).Count

foreach ($result in $testResults) {
    $color = switch ($result.Status) {
        "PASS" { "Green" }
        "FAIL" { "Red" }
        "WARNING" { "Yellow" }
        default { "White" }
    }
    
    Write-Host "  $($result.Status.PadRight(8)) $($result.Name)" -ForegroundColor $color
    if ($result.Details) {
        Write-Host "           $($result.Details)" -ForegroundColor Gray
    }
}

Write-Host ""
Write-Host "Summary: $passCount passed, $failCount failed, $warnCount warnings" -ForegroundColor $(if ($failCount -eq 0) { "Green" } else { "Red" })

if ($failCount -eq 0) {
    Write-Host ""
    Write-Host "🎉 All critical tests passed! TracSeq 2.0 integration is working correctly." -ForegroundColor Green
    Write-Host ""
    Write-Host "🌐 You can now access:" -ForegroundColor Yellow
    Write-Host "   Frontend: http://localhost:5173 (dev) or http://localhost:8080 (prod)"
    Write-Host "   RAG Submissions: http://localhost:5173/rag-submissions"
    Write-Host "   Lab Manager API: http://localhost:3000/api"
    Write-Host "   RAG Service: http://localhost:8000"
} else {
    Write-Host ""
    Write-Host "❌ Some tests failed. Please check the services and try again." -ForegroundColor Red
    Write-Host ""
    Write-Host "🔧 Troubleshooting:" -ForegroundColor Yellow
    Write-Host "   1. Check service logs: docker-compose -f docker-compose.unified.yml logs"
    Write-Host "   2. Verify all containers are running: docker-compose -f docker-compose.unified.yml ps"
    Write-Host "   3. Restart services if needed: .\start-unified.ps1"
} 
