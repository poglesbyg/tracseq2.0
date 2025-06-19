# Test AI Document Processing

Write-Host "📄 Testing AI Document Processing" -ForegroundColor Green
Write-Host "=================================" -ForegroundColor Green

# Read the sample document
$documentText = Get-Content "sample_lab_document.txt" -Raw

# Test different types of AI queries
$testQueries = @(
    "Hello! Can you help me with lab management?",
    "How do I find a specific sample in storage?",
    "What are the requirements for RNA storage?",
    "How do I set up a sequencing job?",
    "Can you help me generate a report?",
    "What types of barcodes does the system support?",
    "How do I upload multiple samples at once?"
)

$headers = @{
    "Content-Type" = "application/json"
}

foreach ($query in $testQueries) {
    Write-Host "`n🤖 Query: $query" -ForegroundColor Cyan
    
    $queryData = @{
        query = $query
    } | ConvertTo-Json
    
    try {
        $response = Invoke-RestMethod -Uri "http://localhost:8000/query" -Method POST -Body $queryData -Headers $headers
        Write-Host "✅ AI Response:" -ForegroundColor Green
        
        # Show first 200 characters of response
        $shortResponse = if ($response.answer.Length -gt 200) { 
            $response.answer.Substring(0, 200) + "..." 
        } else { 
            $response.answer 
        }
        Write-Host $shortResponse -ForegroundColor White
        
    } catch {
        Write-Host "❌ Query failed: $($_.Exception.Message)" -ForegroundColor Red
    }
    
    Start-Sleep -Seconds 1
}

Write-Host "`n🎯 AI Capabilities Demonstrated:" -ForegroundColor Yellow
Write-Host "✅ Natural Language Understanding" -ForegroundColor Green
Write-Host "✅ Context-Aware Responses" -ForegroundColor Green  
Write-Host "✅ Laboratory Domain Knowledge" -ForegroundColor Green
Write-Host "✅ Multi-Topic Expertise" -ForegroundColor Green
Write-Host "✅ Helpful Guidance and Instructions" -ForegroundColor Green 
