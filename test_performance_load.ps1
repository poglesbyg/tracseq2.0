Write-Host "=== Performance and Load Testing Suite ==="
Write-Host "Testing concurrent operations, bulk operations, and system limits"

$timestamp = Get-Date -Format "MMddHHmmss"
$performanceResults = @()

function Measure-Operation {
    param(
        [string]$testName,
        [scriptblock]$operation,
        [int]$iterations = 1
    )
    
    Write-Host "`n--- $testName ---"
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $successCount = 0
    $errorCount = 0
    $errors = @()
    
    for ($i = 1; $i -le $iterations; $i++) {
        try {
            & $operation -Iteration $i
            $successCount++
        } catch {
            $errorCount++
            $errors += $_.Exception.Message
        }
    }
    
    $stopwatch.Stop()
    $avgTime = if ($successCount -gt 0) { $stopwatch.ElapsedMilliseconds / $successCount } else { 0 }
    
    $result = @{
        Name = $testName
        TotalTime = $stopwatch.ElapsedMilliseconds
        SuccessCount = $successCount
        ErrorCount = $errorCount
        AverageTime = $avgTime
        Iterations = $iterations
        Errors = $errors
    }
    
    $performanceResults += $result
    
    Write-Host "⏱️  Total Time: $($stopwatch.ElapsedMilliseconds)ms"
    Write-Host "✅ Success: $successCount/$iterations"
    Write-Host "❌ Errors: $errorCount"
    Write-Host "📊 Average: $([math]::Round($avgTime, 2))ms per operation"
    
    return $result
}

# Performance Test 1: Bulk Sample Creation
Write-Host "`n=== Testing Bulk Sample Creation Performance ==="
Measure-Operation "Create 10 Samples in Batch" {
    param($Iteration)
    
    $batchJson = @"
{
  "samples": [
    {
      "name": "Perf Test Sample 1-$Iteration",
      "barcode": "PERF1_${Iteration}_$timestamp",
      "location": "Freezer A (-80°C)"
    },
    {
      "name": "Perf Test Sample 2-$Iteration",
      "barcode": "PERF2_${Iteration}_$timestamp",
      "location": "Freezer A (-80°C)"
    },
    {
      "name": "Perf Test Sample 3-$Iteration",
      "barcode": "PERF3_${Iteration}_$timestamp",
      "location": "Freezer A (-80°C)"
    },
    {
      "name": "Perf Test Sample 4-$Iteration",
      "barcode": "PERF4_${Iteration}_$timestamp",
      "location": "Freezer A (-80°C)"
    },
    {
      "name": "Perf Test Sample 5-$Iteration",
      "barcode": "PERF5_${Iteration}_$timestamp",
      "location": "Freezer A (-80°C)"
    },
    {
      "name": "Perf Test Sample 6-$Iteration",
      "barcode": "PERF6_${Iteration}_$timestamp",
      "location": "Freezer A (-80°C)"
    },
    {
      "name": "Perf Test Sample 7-$Iteration",
      "barcode": "PERF7_${Iteration}_$timestamp",
      "location": "Freezer A (-80°C)"
    },
    {
      "name": "Perf Test Sample 8-$Iteration",
      "barcode": "PERF8_${Iteration}_$timestamp",
      "location": "Freezer A (-80°C)"
    },
    {
      "name": "Perf Test Sample 9-$Iteration",
      "barcode": "PERF9_${Iteration}_$timestamp",
      "location": "Freezer A (-80°C)"
    },
    {
      "name": "Perf Test Sample 10-$Iteration",
      "barcode": "PERF10_${Iteration}_$timestamp",
      "location": "Freezer A (-80°C)"
    }
  ],
  "storage_location_id": 1,
  "template_name": "Performance Test Batch",
  "stored_by": "perf_tester"
}
"@
    
    $response = Invoke-RestMethod -Uri "http://localhost:3000/api/samples/batch" -Method POST -Body $batchJson -ContentType "application/json"
    if ($response.created -ne 10) {
        throw "Expected 10 samples created, got $($response.created)"
    }
} 5

# Performance Test 2: Rapid Movement Operations
Write-Host "`n=== Testing Rapid Movement Operations ==="
Measure-Operation "Move Samples Rapidly" {
    param($Iteration)
    
    $barcode = "PERF1_${Iteration}_$timestamp"
    $targetLocation = if ($Iteration % 2 -eq 0) { 2 } else { 3 }
    
    $moveJson = @"
{
  "barcode": "$barcode",
  "location_id": $targetLocation,
  "reason": "Performance test movement $Iteration",
  "moved_by": "perf_tester"
}
"@
    
    $response = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/move" -Method POST -Body $moveJson -ContentType "application/json"
    if (-not $response.success) {
        throw "Movement failed: $($response.message)"
    }
} 5

# Performance Test 3: Concurrent Barcode Scans
Write-Host "`n=== Testing Concurrent Barcode Scanning ==="
Measure-Operation "Scan Multiple Barcodes" {
    param($Iteration)
    
    $barcode = "PERF$($Iteration)_1_$timestamp"
    $response = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/scan/$barcode"
    if (-not $response.barcode) {
        throw "Scan failed for $barcode"
    }
} 10

# Performance Test 4: Storage Location Queries
Write-Host "`n=== Testing Storage Location Query Performance ==="
Measure-Operation "Query Storage Locations" {
    param($Iteration)
    
    $response = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/locations"
    if ($response.Count -eq 0) {
        throw "No storage locations returned"
    }
} 20

# Performance Test 5: Capacity Overview Queries
Write-Host "`n=== Testing Capacity Overview Performance ==="
Measure-Operation "Query Capacity Overview" {
    param($Iteration)
    
    $response = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/capacity"
    if (-not $response.total_capacity) {
        throw "Invalid capacity response"
    }
} 15

# Load Test 1: Concurrent Operations
Write-Host "`n=== Testing Concurrent Operations ==="

# Create samples for concurrent testing
$concurrentBarcodes = @()
for ($i = 1; $i -le 5; $i++) {
    $concurrentBarcodes += "CONCURRENT_${i}_$timestamp"
}

# Create concurrent test samples
foreach ($barcode in $concurrentBarcodes) {
    $createJson = @"
{
  "samples": [
    {
      "name": "Concurrent Test Sample",
      "barcode": "$barcode",
      "location": "Freezer A (-80°C)"
    }
  ],
  "storage_location_id": 1,
  "template_name": "Concurrent Test",
  "stored_by": "concurrent_tester"
}
"@
    
    try {
        $response = Invoke-RestMethod -Uri "http://localhost:3000/api/samples/batch" -Method POST -Body $createJson -ContentType "application/json"
    } catch {
        Write-Host "⚠️  Failed to create concurrent test sample $barcode"
    }
}

# Test concurrent movements
Write-Host "Testing concurrent sample movements..."
$jobs = @()
foreach ($barcode in $concurrentBarcodes) {
    $job = Start-Job -ScriptBlock {
        param($barcode, $timestamp)
        
        $moveJson = @"
{
  "barcode": "$barcode",
  "location_id": 2,
  "reason": "Concurrent movement test",
  "moved_by": "concurrent_tester"
}
"@
        
        try {
            $response = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/move" -Method POST -Body $moveJson -ContentType "application/json"
            return @{Success = $true; Barcode = $barcode; Message = $response.message}
        } catch {
            return @{Success = $false; Barcode = $barcode; Error = $_.Exception.Message}
        }
    } -ArgumentList $barcode, $timestamp
    $jobs += $job
}

# Wait for all jobs to complete
$concurrentResults = $jobs | Wait-Job | Receive-Job
$jobs | Remove-Job

$successfulMoves = ($concurrentResults | Where-Object { $_.Success }).Count
$failedMoves = ($concurrentResults | Where-Object { -not $_.Success }).Count

Write-Host "Concurrent movements completed:"
Write-Host "✅ Successful: $successfulMoves"
Write-Host "❌ Failed: $failedMoves"

# Memory and Resource Test
Write-Host "`n=== Testing Resource Usage ==="
$beforeMemory = [System.GC]::GetTotalMemory($false)

# Simulate heavy load
Measure-Operation "Heavy Load Simulation" {
    param($Iteration)
    
    # Multiple operations in sequence
    $response1 = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/locations"
    $response2 = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/capacity"
    $scanResponse = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/scan/CONCURRENT_1_$timestamp"
    
    if (-not ($response1 -and $response2 -and $scanResponse)) {
        throw "One or more operations failed in heavy load test"
    }
} 10

$afterMemory = [System.GC]::GetTotalMemory($true)
$memoryDiff = $afterMemory - $beforeMemory

Write-Host "Memory usage during test:"
Write-Host "Before: $([math]::Round($beforeMemory / 1MB, 2)) MB"
Write-Host "After: $([math]::Round($afterMemory / 1MB, 2)) MB"
Write-Host "Difference: $([math]::Round($memoryDiff / 1MB, 2)) MB"

# Cleanup: Remove all performance test samples
Write-Host "`n=== CLEANUP: Removing Performance Test Samples ==="
$allTestBarcodes = @()

# Add batch test samples
for ($iteration = 1; $iteration -le 5; $iteration++) {
    for ($sample = 1; $sample -le 10; $sample++) {
        $allTestBarcodes += "PERF${sample}_${iteration}_$timestamp"
    }
}

# Add concurrent test samples
$allTestBarcodes += $concurrentBarcodes

foreach ($barcode in $allTestBarcodes) {
    try {
        $removeJson = @"
{
  "barcode": "$barcode",
  "reason": "Performance test cleanup",
  "removed_by": "perf_tester"
}
"@
        $response = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/remove" -Method POST -Body $removeJson -ContentType "application/json"
    } catch {
        # Ignore cleanup errors
    }
}

Write-Host "🧹 Performance test samples cleanup completed"

# Performance Results Summary
Write-Host "`n=== PERFORMANCE TEST RESULTS SUMMARY ==="
Write-Host "Test Results:"

foreach ($result in $performanceResults) {
    Write-Host "`n📊 $($result.Name):"
    Write-Host "  Total Time: $($result.TotalTime)ms"
    Write-Host "  Success Rate: $($result.SuccessCount)/$($result.Iterations) ($([math]::Round(($result.SuccessCount / $result.Iterations) * 100, 1))%)"
    Write-Host "  Average Time: $([math]::Round($result.AverageTime, 2))ms per operation"
    if ($result.ErrorCount -gt 0) {
        Write-Host "  Errors: $($result.ErrorCount)"
    }
}

# Performance benchmarks
$avgBatchTime = ($performanceResults | Where-Object { $_.Name -like "*Batch*" } | Select-Object -First 1).AverageTime
$avgMoveTime = ($performanceResults | Where-Object { $_.Name -like "*Move*" } | Select-Object -First 1).AverageTime
$avgScanTime = ($performanceResults | Where-Object { $_.Name -like "*Scan*" } | Select-Object -First 1).AverageTime

Write-Host "`n🎯 Performance Benchmarks:"
Write-Host "  Batch Creation (10 samples): $([math]::Round($avgBatchTime, 2))ms"
Write-Host "  Sample Movement: $([math]::Round($avgMoveTime, 2))ms"
Write-Host "  Barcode Scanning: $([math]::Round($avgScanTime, 2))ms"

if ($avgBatchTime -lt 5000) { Write-Host "✅ Batch creation performance: GOOD" } else { Write-Host "⚠️  Batch creation performance: NEEDS ATTENTION" }
if ($avgMoveTime -lt 1000) { Write-Host "✅ Movement performance: GOOD" } else { Write-Host "⚠️  Movement performance: NEEDS ATTENTION" }
if ($avgScanTime -lt 500) { Write-Host "✅ Scan performance: GOOD" } else { Write-Host "⚠️  Scan performance: NEEDS ATTENTION" }

Write-Host "`n🚀 Performance and load testing completed!" 
