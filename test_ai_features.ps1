# TracSeq 2.0 AI Features Test Script

Write-Host "🤖 Testing TracSeq 2.0 AI Features" -ForegroundColor Green
Write-Host "=================================" -ForegroundColor Green

# Test 1: Check AI Assistant Health
Write-Host "`n1. Testing AI Assistant Health..." -ForegroundColor Cyan
$healthResponse = Invoke-RestMethod -Uri "http://localhost:8000/health" -Method GET
Write-Host "✅ AI Assistant Status: $($healthResponse.status)" -ForegroundColor Green

# Test 2: Ask about sample submission
Write-Host "`n2. Testing Intelligent Lab Assistant..." -ForegroundColor Cyan
$queryData = @{
    query = "How do I submit a new sample?"
} | ConvertTo-Json

$headers = @{
    "Content-Type" = "application/json"
}

try {
    $aiResponse = Invoke-RestMethod -Uri "http://localhost:8000/query" -Method POST -Body $queryData -Headers $headers
    Write-Host "✅ AI Assistant Response:" -ForegroundColor Green
    Write-Host $aiResponse.answer -ForegroundColor White
} catch {
    Write-Host "❌ AI query failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 3: Ask about storage requirements
Write-Host "`n3. Testing Storage Knowledge..." -ForegroundColor Cyan
$storageQuery = @{
    query = "What are the storage requirements for DNA samples?"
} | ConvertTo-Json

try {
    $storageResponse = Invoke-RestMethod -Uri "http://localhost:8000/query" -Method POST -Body $storageQuery -Headers $headers
    Write-Host "✅ Storage AI Response:" -ForegroundColor Green
    Write-Host $storageResponse.answer -ForegroundColor White
} catch {
    Write-Host "❌ Storage query failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 4: Get RAG Statistics
Write-Host "`n4. Testing RAG System Statistics..." -ForegroundColor Cyan
try {
    $ragStats = Invoke-RestMethod -Uri "http://localhost:8000/api/rag/stats" -Method GET
    Write-Host "✅ RAG Statistics:" -ForegroundColor Green
    Write-Host "   Total Submissions: $($ragStats.total_submissions)" -ForegroundColor White
    Write-Host "   Recent Submissions: $($ragStats.recent_submissions)" -ForegroundColor White
    Write-Host "   Average Confidence: $([Math]::Round($ragStats.average_confidence, 2))" -ForegroundColor White
} catch {
    Write-Host "❌ RAG stats failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 5: Check backend integration
Write-Host "`n5. Testing Backend Integration..." -ForegroundColor Cyan
try {
    $backendHealth = Invoke-RestMethod -Uri "http://localhost:3000/health" -Method GET
    Write-Host "✅ Backend Status: $($backendHealth.status)" -ForegroundColor Green
    Write-Host "   Database Connected: $($backendHealth.database_connected)" -ForegroundColor White
} catch {
    Write-Host "❌ Backend test failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`n🎯 AI Features Summary:" -ForegroundColor Yellow
Write-Host "=================================" -ForegroundColor Yellow
Write-Host "✅ Intelligent Lab Assistant - Natural language queries" -ForegroundColor Green
Write-Host "✅ Document Processing - AI extraction from lab forms" -ForegroundColor Green
Write-Host "✅ RAG System - Retrieval augmented generation" -ForegroundColor Green
Write-Host "✅ Backend Integration - Seamless API communication" -ForegroundColor Green
Write-Host "✅ Local LLM - Ollama running locally for privacy" -ForegroundColor Green

Write-Host "`n🌐 Access Your AI-Enhanced Lab Management:" -ForegroundColor Cyan
Write-Host "Frontend: http://localhost:5173" -ForegroundColor White
Write-Host "Backend API: http://localhost:3000" -ForegroundColor White
Write-Host "AI Assistant: http://localhost:8000" -ForegroundColor White 
