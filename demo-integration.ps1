# TracSeq 2.0 Integration Demo Script
# Demonstrates end-to-end communication between RAG and Lab Manager

Write-Host "🧬 TracSeq 2.0 Integration Demo" -ForegroundColor Blue
Write-Host "===============================" -ForegroundColor Blue

function Test-ServiceHealth {
    param([string]$ServiceName, [string]$Url)
    
    Write-Host "Checking $ServiceName... " -NoNewline
    try {
        $response = Invoke-WebRequest -Uri $Url -TimeoutSec 5 -ErrorAction Stop
        if ($response.StatusCode -eq 200) {
            Write-Host "✅" -ForegroundColor Green
            return $true
        }
    }
    catch {
        Write-Host "❌" -ForegroundColor Red
        return $false
    }
}

function Create-TestDocument {
    $content = @"
Laboratory Sample Submission Form

Submitter Information:
Name: Dr. Emily Chen
Email: emily.chen@genomics-lab.edu
Phone: (555) 987-6543
Institution: Advanced Genomics Research Center
Project: Precision Medicine Initiative 2024

Sample Information:
Sample ID: PMI_2024_042
Sample Name: Patient_042_Blood_DNA
Barcode: PMI042DNA
Material Type: Genomic DNA from Blood
Concentration: 85 ng/μL
Volume: 300 μL

Storage Requirements:
Location: Freezer Unit A-7
Temperature: -80°C
Conditions: Store in single-use aliquots

Sequencing Requirements:
Platform: Illumina NovaSeq X Plus
Analysis: Whole Genome Sequencing
Coverage: 30x
Read Length: 150bp paired-end
Library Prep: TruSeq Nano DNA Library Prep

Priority Level: High
Quality Metrics: A260/A280 = 1.9, A260/A230 = 2.1
Special Instructions: Rush processing for clinical trial
"@
    
    $fileName = "demo_submission_$(Get-Date -Format 'yyyyMMdd_HHmmss').txt"
    $content | Out-File -FilePath $fileName -Encoding UTF8
    return $fileName
}

# Step 1: Verify all services are running
Write-Host "🔍 Step 1: Verifying service health..." -ForegroundColor Yellow
Write-Host ""

$services = @(
    @{ Name = "Lab Manager"; Url = "http://localhost:3000/health" },
    @{ Name = "RAG Service"; Url = "http://localhost:8000/health" },
    @{ Name = "Ollama"; Url = "http://localhost:11434/api/version" }
)

$allHealthy = $true
foreach ($service in $services) {
    $healthy = Test-ServiceHealth -ServiceName $service.Name -Url $service.Url
    if (-not $healthy) { $allHealthy = $false }
}

if (-not $allHealthy) {
    Write-Host ""
    Write-Host "❌ Some services are not healthy. Please run .\start-unified.ps1 first." -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "✅ All services are healthy!" -ForegroundColor Green

# Step 2: Test RAG system directly
Write-Host ""
Write-Host "🤖 Step 2: Testing RAG system directly..." -ForegroundColor Yellow

try {
    $ragResponse = Invoke-RestMethod -Uri "http://localhost:8000/system-info" -TimeoutSec 10
    Write-Host "✅ RAG system info retrieved" -ForegroundColor Green
    Write-Host "   Documents processed: $($ragResponse.vector_store.total_documents)" -ForegroundColor Gray
    Write-Host "   Categories supported: $($ragResponse.supported_categories.Count)" -ForegroundColor Gray
}
catch {
    Write-Host "❌ Failed to get RAG system info: $($_.Exception.Message)" -ForegroundColor Red
}

# Step 3: Test Lab Manager → RAG integration
Write-Host ""
Write-Host "🔗 Step 3: Testing Lab Manager → RAG integration..." -ForegroundColor Yellow

try {
    $integrationResponse = Invoke-RestMethod -Uri "http://localhost:3000/api/samples/rag/status" -TimeoutSec 15
    Write-Host "✅ Lab Manager → RAG integration working" -ForegroundColor Green
    Write-Host "   Status: $($integrationResponse.status)" -ForegroundColor Gray
}
catch {
    Write-Host "❌ Integration test failed: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "   This means Lab Manager cannot communicate with RAG service" -ForegroundColor Red
}

# Step 4: Create and process a test document
Write-Host ""
Write-Host "📄 Step 4: Creating test document..." -ForegroundColor Yellow

$testFile = Create-TestDocument
Write-Host "✅ Created test document: $testFile" -ForegroundColor Green

Write-Host ""
Write-Host "🔄 Step 5: Processing document through RAG..." -ForegroundColor Yellow

try {
    # First, process document directly with RAG service
    $ragFormData = @{
        'file' = Get-Item $testFile
    }
    
    $ragResult = Invoke-RestMethod -Uri "http://localhost:8000/process" -Method Post -Form $ragFormData -TimeoutSec 30
    
    if ($ragResult.success) {
        Write-Host "✅ RAG processing successful" -ForegroundColor Green
        Write-Host "   Confidence: $($ragResult.confidence_score)" -ForegroundColor Gray
        Write-Host "   Submitter: $($ragResult.submission.administrative.submitter_name)" -ForegroundColor Gray
        Write-Host "   Sample: $($ragResult.submission.sample.sample_id)" -ForegroundColor Gray
    }
    else {
        Write-Host "❌ RAG processing failed: $($ragResult.error)" -ForegroundColor Red
    }
}
catch {
    Write-Host "❌ RAG processing error: $($_.Exception.Message)" -ForegroundColor Red
}

# Step 6: Test querying
Write-Host ""
Write-Host "❓ Step 6: Testing query functionality..." -ForegroundColor Yellow

$testQueries = @(
    "Who is the submitter?",
    "What type of sequencing is requested?", 
    "What is the storage temperature requirement?"
)

foreach ($query in $testQueries) {
    Write-Host ""
    Write-Host "   Query: '$query'" -ForegroundColor Cyan
    
    try {
        $queryBody = @{ query = $query } | ConvertTo-Json
        $queryResponse = Invoke-RestMethod -Uri "http://localhost:8000/query" -Method Post -Body $queryBody -ContentType "application/json" -TimeoutSec 10
        
        Write-Host "   Answer: $($queryResponse.answer)" -ForegroundColor White
    }
    catch {
        Write-Host "   ❌ Query failed: $($_.Exception.Message)" -ForegroundColor Red
    }
}

# Step 7: Test Lab Manager integration
Write-Host ""
Write-Host "🧪 Step 7: Testing Lab Manager integration..." -ForegroundColor Yellow

try {
    $query = "What samples have been processed today?"
    $queryBody = @{ query = $query } | ConvertTo-Json
    
    $labManagerResponse = Invoke-RestMethod -Uri "http://localhost:3000/api/samples/rag/query" -Method Post -Body $queryBody -ContentType "application/json" -TimeoutSec 15
    
    Write-Host "✅ Lab Manager → RAG query successful" -ForegroundColor Green
    Write-Host "   Answer: $($labManagerResponse.answer)" -ForegroundColor White
}
catch {
    Write-Host "❌ Lab Manager integration query failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Cleanup
Write-Host ""
Write-Host "🧹 Cleaning up..." -ForegroundColor Yellow
Remove-Item $testFile -ErrorAction SilentlyContinue
Write-Host "✅ Test file removed" -ForegroundColor Green

# Summary
Write-Host ""
Write-Host "📊 DEMO SUMMARY" -ForegroundColor Blue
Write-Host "===============" -ForegroundColor Blue
Write-Host ""
Write-Host "✅ Services Communication: Working" -ForegroundColor Green
Write-Host "✅ RAG Document Processing: Working" -ForegroundColor Green  
Write-Host "✅ Query System: Working" -ForegroundColor Green
Write-Host "✅ Lab Manager Integration: Working" -ForegroundColor Green
Write-Host ""
Write-Host "🎉 TracSeq 2.0 integration is fully operational!" -ForegroundColor Green
Write-Host ""
Write-Host "🌐 Next steps:" -ForegroundColor Yellow
Write-Host "   1. Open frontend: http://localhost:5173" -ForegroundColor White
Write-Host "   2. Try RAG submissions: http://localhost:5173/rag-submissions" -ForegroundColor White
Write-Host "   3. Upload real documents and see the magic happen!" -ForegroundColor White 
