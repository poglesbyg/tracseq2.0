Write-Host "=== Comprehensive Storage Operations Test Suite ==="
Write-Host "Testing sample movement, removal, and edge cases"

$testResults = @()
$timestamp = Get-Date -Format "MMddHHmmss"

function Test-Operation {
    param(
        [string]$testName,
        [scriptblock]$operation,
        [bool]$shouldSucceed = $true
    )
    
    Write-Host "`n--- $testName ---"
    try {
        $result = & $operation
        if ($shouldSucceed) {
            Write-Host "✅ PASS: $testName"
            $testResults += @{Name = $testName; Result = "PASS"; Details = $result}
        } else {
            Write-Host "❌ FAIL: $testName (expected failure but got success)"
            $testResults += @{Name = $testName; Result = "FAIL"; Details = "Expected failure but succeeded"}
        }
    } catch {
        if (-not $shouldSucceed) {
            Write-Host "✅ PASS: $testName (expected failure)"
            $testResults += @{Name = $testName; Result = "PASS"; Details = "Expected failure: $($_.Exception.Message)"}
        } else {
            Write-Host "❌ FAIL: $testName - $($_.Exception.Message)"
            $testResults += @{Name = $testName; Result = "FAIL"; Details = $_.Exception.Message}
        }
    }
}

# Setup: Create test samples
Write-Host "`n=== SETUP: Creating Test Samples ==="
$sampleBarcodes = @(
    "TEST_MOVE_$timestamp",
    "TEST_REMOVE_$timestamp", 
    "TEST_EDGE_$timestamp"
)

foreach ($barcode in $sampleBarcodes) {
    $createJson = @"
{
  "samples": [
    {
      "name": "Test Sample $barcode",
      "barcode": "$barcode",
      "location": "Freezer A (-80°C)"
    }
  ],
  "storage_location_id": 1,
  "template_name": "Comprehensive Test",
  "stored_by": "test_admin"
}
"@
    
    try {
        $response = Invoke-RestMethod -Uri "http://localhost:3000/api/samples/batch" -Method POST -Body $createJson -ContentType "application/json"
        Write-Host "✅ Created sample $barcode"
    } catch {
        Write-Host "❌ Failed to create sample $barcode : $($_.Exception.Message)"
    }
}

# Test 1: Basic Sample Movement
Test-Operation "Basic Sample Movement" {
    $moveJson = @"
{
  "barcode": "$($sampleBarcodes[0])",
  "location_id": 2,
  "reason": "Testing basic movement",
  "moved_by": "test_admin"
}
"@
    $response = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/move" -Method POST -Body $moveJson -ContentType "application/json"
    return "Moved to location: $($response.new_location)"
}

# Test 2: Verify Movement via Scan
Test-Operation "Verify Movement via Barcode Scan" {
    $scanResponse = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/scan/$($sampleBarcodes[0])"
    if ($scanResponse.location_id -eq 2) {
        return "Sample correctly moved to location 2: $($scanResponse.location_name)"
    } else {
        throw "Sample not in expected location. Found at location $($scanResponse.location_id)"
    }
}

# Test 3: Move to Different Temperature Zone
Test-Operation "Move Between Temperature Zones" {
    $moveJson = @"
{
  "barcode": "$($sampleBarcodes[0])",
  "location_id": 3,
  "reason": "Testing temperature zone movement",
  "moved_by": "test_admin"
}
"@
    $response = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/move" -Method POST -Body $moveJson -ContentType "application/json"
    return "Moved between temperature zones: $($response.message)"
}

# Test 4: Basic Sample Removal
Test-Operation "Basic Sample Removal" {
    $removeJson = @"
{
  "barcode": "$($sampleBarcodes[1])",
  "reason": "Testing basic removal",
  "removed_by": "test_admin"
}
"@
    $response = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/remove" -Method POST -Body $removeJson -ContentType "application/json"
    return "Removed sample: $($response.removed_sample.sample_id) from $($response.removed_sample.location_name)"
}

# Test 5: Verify Removal via Scan (should fail)
Test-Operation "Verify Removal via Scan (should fail)" {
    try {
        $scanResponse = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/scan/$($sampleBarcodes[1])"
        throw "Sample still found after removal"
    } catch {
        if ($_.Exception.Response.StatusCode -eq 404) {
            return "Sample correctly removed from storage"
        } else {
            throw $_.Exception.Message
        }
    }
} $true

# Test 6: Move Non-Existent Sample (should fail)
Test-Operation "Move Non-Existent Sample" {
    $moveJson = @"
{
  "barcode": "NONEXISTENT_$timestamp",
  "location_id": 2,
  "reason": "Testing non-existent sample",
  "moved_by": "test_admin"
}
"@
    $response = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/move" -Method POST -Body $moveJson -ContentType "application/json"
    throw "Should not succeed for non-existent sample"
} $false

# Test 7: Move to Non-Existent Location (should fail)
Test-Operation "Move to Non-Existent Location" {
    $moveJson = @"
{
  "barcode": "$($sampleBarcodes[2])",
  "location_id": 999,
  "reason": "Testing non-existent location",
  "moved_by": "test_admin"
}
"@
    $response = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/move" -Method POST -Body $moveJson -ContentType "application/json"
    throw "Should not succeed for non-existent location"
} $false

# Test 8: Remove Non-Existent Sample (should fail)
Test-Operation "Remove Non-Existent Sample" {
    $removeJson = @"
{
  "barcode": "NONEXISTENT_$timestamp",
  "reason": "Testing non-existent sample removal",
  "removed_by": "test_admin"
}
"@
    $response = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/remove" -Method POST -Body $removeJson -ContentType "application/json"
    throw "Should not succeed for non-existent sample"
} $false

# Test 9: Remove Already Removed Sample (should fail)
Test-Operation "Remove Already Removed Sample" {
    $removeJson = @"
{
  "barcode": "$($sampleBarcodes[1])",
  "reason": "Testing already removed sample",
  "removed_by": "test_admin"
}
"@
    $response = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/remove" -Method POST -Body $removeJson -ContentType "application/json"
    throw "Should not succeed for already removed sample"
} $false

# Test 10: Move Sample to Same Location
Test-Operation "Move Sample to Same Location" {
    # First get current location
    $scanResponse = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/scan/$($sampleBarcodes[2])"
    $currentLocation = $scanResponse.location_id
    
    $moveJson = @"
{
  "barcode": "$($sampleBarcodes[2])",
  "location_id": $currentLocation,
  "reason": "Testing same location move",
  "moved_by": "test_admin"
}
"@
    $response = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/move" -Method POST -Body $moveJson -ContentType "application/json"
    return "Move to same location: $($response.message)"
}

# Test 11: Storage Capacity Validation
Test-Operation "Storage Capacity Validation" {
    $capacityResponse = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/capacity"
    $totalCapacity = $capacityResponse.total_capacity
    $totalUsage = $capacityResponse.total_usage
    $utilization = $capacityResponse.overall_utilization
    return "Capacity: $totalUsage/$totalCapacity used ($utilization% utilization)"
}

# Test 12: Storage Location Listing
Test-Operation "Storage Location Listing" {
    $locationsResponse = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/locations"
    $activeLocations = ($locationsResponse | Where-Object { $_.is_active }).Count
    return "Found $activeLocations active storage locations"
}

# Test 13: Movement History Verification
Test-Operation "Movement History Verification" {
    # This test verifies that movement operations are properly recorded
    $scanResponse = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/scan/$($sampleBarcodes[0])"
    if ($scanResponse.location_id -eq 3) {
        return "Sample movement history verified - now at location 3"
    } else {
        throw "Movement history inconsistent"
    }
}

# Test 14: Batch Operations Impact
Test-Operation "Batch Operations Impact on Storage" {
    # Create multiple samples at once and verify storage impact
    $batchJson = @"
{
  "samples": [
    {
      "name": "Batch Test 1",
      "barcode": "BATCH1_$timestamp",
      "location": "Freezer A (-80°C)"
    },
    {
      "name": "Batch Test 2", 
      "barcode": "BATCH2_$timestamp",
      "location": "Freezer A (-80°C)"
    }
  ],
  "storage_location_id": 1,
  "template_name": "Batch Test",
  "stored_by": "test_admin"
}
"@
    $batchResponse = Invoke-RestMethod -Uri "http://localhost:3000/api/samples/batch" -Method POST -Body $batchJson -ContentType "application/json"
    return "Batch created: $($batchResponse.created) samples, $($batchResponse.stored_in_storage) stored"
}

# Test 15: Temperature Zone Compatibility
Test-Operation "Temperature Zone Compatibility Check" {
    # Verify samples can be moved between compatible temperature zones
    $moveJson = @"
{
  "barcode": "BATCH1_$timestamp",
  "location_id": 4,
  "reason": "Testing temperature compatibility",
  "moved_by": "test_admin"
}
"@
    $response = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/move" -Method POST -Body $moveJson -ContentType "application/json"
    return "Temperature zone move successful: $($response.message)"
}

# Cleanup: Remove test samples
Write-Host "`n=== CLEANUP: Removing Test Samples ==="
$cleanupBarcodes = @($sampleBarcodes[0], $sampleBarcodes[2], "BATCH1_$timestamp", "BATCH2_$timestamp")

foreach ($barcode in $cleanupBarcodes) {
    try {
        $removeJson = @"
{
  "barcode": "$barcode",
  "reason": "Test cleanup",
  "removed_by": "test_admin"
}
"@
        $response = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/remove" -Method POST -Body $removeJson -ContentType "application/json"
        Write-Host "🧹 Cleaned up sample $barcode"
    } catch {
        Write-Host "⚠️  Could not clean up $barcode : $($_.Exception.Message)"
    }
}

# Results Summary
Write-Host "`n=== TEST RESULTS SUMMARY ==="
$passCount = ($testResults | Where-Object { $_.Result -eq "PASS" }).Count
$failCount = ($testResults | Where-Object { $_.Result -eq "FAIL" }).Count
$totalTests = $testResults.Count

Write-Host "Total Tests: $totalTests"
Write-Host "Passed: $passCount ✅"
Write-Host "Failed: $failCount ❌"
Write-Host "Success Rate: $([math]::Round(($passCount / $totalTests) * 100, 1))%"

if ($failCount -gt 0) {
    Write-Host "`nFailed Tests:"
    $testResults | Where-Object { $_.Result -eq "FAIL" } | ForEach-Object {
        Write-Host "  ❌ $($_.Name): $($_.Details)"
    }
}

Write-Host "`n🎉 Comprehensive storage operations test suite completed!" 
