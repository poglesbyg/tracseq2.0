# Test AI Document Processing with Real Extraction

Write-Host "🤖 Testing AI Document Processing" -ForegroundColor Green
Write-Host "=================================" -ForegroundColor Green

# Read the sample document
$documentText = Get-Content "sample_lab_document.txt" -Raw

# Test the new document processing endpoint
$processData = @{
    text = $documentText
    filename = "sample_lab_document.txt"
} | ConvertTo-Json

$headers = @{
    "Content-Type" = "application/json"
}

Write-Host "`n📄 Processing document with AI..." -ForegroundColor Cyan
Write-Host "Document length: $($documentText.Length) characters" -ForegroundColor White

try {
    $startTime = Get-Date
    $response = Invoke-RestMethod -Uri "http://localhost:8000/api/rag/process" -Method POST -Body $processData -Headers $headers
    $endTime = Get-Date
    $actualProcessingTime = ($endTime - $startTime).TotalSeconds
    
    Write-Host "`n✅ AI Processing Results:" -ForegroundColor Green
    Write-Host "=================================" -ForegroundColor Green
    Write-Host "Success: $($response.success)" -ForegroundColor $(if($response.success) { "Green" } else { "Red" })
    Write-Host "Confidence Score: $([Math]::Round($response.confidence_score, 1))%" -ForegroundColor Yellow
    Write-Host "Samples Found: $($response.samples_found)" -ForegroundColor Cyan
    Write-Host "Processing Time: $([Math]::Round($response.processing_time, 2))s" -ForegroundColor White
    Write-Host "Actual Time: $([Math]::Round($actualProcessingTime, 2))s" -ForegroundColor White
    Write-Host "Message: $($response.message)" -ForegroundColor White
    
    Write-Host "`n👤 Submitter Information:" -ForegroundColor Cyan
    Write-Host "Name: $($response.submitter_info.name)" -ForegroundColor White
    Write-Host "Email: $($response.submitter_info.email)" -ForegroundColor White
    Write-Host "Phone: $($response.submitter_info.phone)" -ForegroundColor White
    Write-Host "Institution: $($response.submitter_info.institution)" -ForegroundColor White
    Write-Host "Project: $($response.submitter_info.project_name)" -ForegroundColor White
    
    Write-Host "`n🧪 Sample Information:" -ForegroundColor Cyan
    Write-Host "Sample ID: $($response.sample_info.sample_id)" -ForegroundColor White
    Write-Host "Sample Type: $($response.sample_info.sample_type)" -ForegroundColor White
    Write-Host "Concentration: $($response.sample_info.concentration)" -ForegroundColor White
    Write-Host "Volume: $($response.sample_info.volume)" -ForegroundColor White
    Write-Host "Storage: $($response.sample_info.storage_conditions)" -ForegroundColor White
    
    Write-Host "`n🧬 Sequencing Information:" -ForegroundColor Cyan
    Write-Host "Platform: $($response.sequencing_info.platform)" -ForegroundColor White
    Write-Host "Analysis Type: $($response.sequencing_info.analysis_type)" -ForegroundColor White
    Write-Host "Coverage: $($response.sequencing_info.coverage)" -ForegroundColor White
    Write-Host "Read Length: $($response.sequencing_info.read_length)" -ForegroundColor White
    
    Write-Host "`n📊 Raw Extracted Data:" -ForegroundColor Yellow
    $response.extracted_data | ConvertTo-Json -Depth 3 | Write-Host -ForegroundColor Gray
    
} catch {
    Write-Host "❌ Document processing failed: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "Response: $($_.Exception.Response.StatusCode)" -ForegroundColor Red
}

Write-Host "`n🎯 AI Document Processing Features:" -ForegroundColor Yellow
Write-Host "✅ Automatic information extraction from lab documents" -ForegroundColor Green
Write-Host "✅ Confidence scoring based on completeness" -ForegroundColor Green
Write-Host "✅ Structured data output for easy integration" -ForegroundColor Green
Write-Host "✅ Support for various document formats" -ForegroundColor Green
Write-Host "✅ Real-time processing with local LLM" -ForegroundColor Green

Write-Host "`n🌐 Test this in the web interface:" -ForegroundColor Cyan
Write-Host "1. Go to http://localhost:5173" -ForegroundColor White
Write-Host "2. Look for document upload or AI processing feature" -ForegroundColor White
Write-Host "3. Upload a lab submission document" -ForegroundColor White
Write-Host "4. Review the extracted information" -ForegroundColor White 
