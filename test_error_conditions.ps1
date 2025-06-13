Write-Host "=== Error Conditions and Edge Cases Test Suite ==="
Write-Host "Testing validation, error handling, and boundary conditions"

$timestamp = Get-Date -Format "MMddHHmmss"
$errorTests = @()

function Test-ErrorCondition {
    param(
        [string]$testName,
        [string]$endpoint,
        [string]$jsonBody,
        [int]$expectedStatusCode,
        [string]$method = "POST"
    )
    
    Write-Host "`n--- $testName ---"
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:3000$endpoint" -Method $method -Body $jsonBody -ContentType "application/json" -UseBasicParsing
        Write-Host "❌ FAIL: Expected error but got success (Status: $($response.StatusCode))"
        $errorTests += @{Name = $testName; Result = "FAIL"; Expected = $expectedStatusCode; Actual = $response.StatusCode}
    } catch {
        $actualStatus = $_.Exception.Response.StatusCode.value__
        if ($actualStatus -eq $expectedStatusCode) {
            Write-Host "✅ PASS: Got expected error $expectedStatusCode"
            $errorTests += @{Name = $testName; Result = "PASS"; Expected = $expectedStatusCode; Actual = $actualStatus}
        } else {
            Write-Host "❌ FAIL: Expected $expectedStatusCode but got $actualStatus"
            $errorTests += @{Name = $testName; Result = "FAIL"; Expected = $expectedStatusCode; Actual = $actualStatus}
        }
    }
}

Write-Host "`n=== Testing Move Operation Error Conditions ==="

# Test 1: Empty barcode
Test-ErrorCondition "Move with Empty Barcode" "/api/storage/move" @"
{
  "barcode": "",
  "location_id": 2,
  "reason": "Test empty barcode",
  "moved_by": "test_admin"
}
"@ 400

# Test 2: Null/Missing barcode
Test-ErrorCondition "Move with Missing Barcode" "/api/storage/move" @"
{
  "location_id": 2,
  "reason": "Test missing barcode",
  "moved_by": "test_admin"
}
"@ 422

# Test 3: Invalid location ID
Test-ErrorCondition "Move to Invalid Location" "/api/storage/move" @"
{
  "barcode": "INVALID_TEST_$timestamp",
  "location_id": -1,
  "reason": "Test invalid location",
  "moved_by": "test_admin"
}
"@ 404

# Test 4: Missing moved_by field
Test-ErrorCondition "Move without moved_by Field" "/api/storage/move" @"
{
  "barcode": "TEST_$timestamp",
  "location_id": 2,
  "reason": "Test missing moved_by"
}
"@ 422

# Test 5: Extremely long barcode
$longBarcode = "VERY_LONG_BARCODE_" + ("X" * 200)
Test-ErrorCondition "Move with Extremely Long Barcode" "/api/storage/move" @"
{
  "barcode": "$longBarcode",
  "location_id": 2,
  "reason": "Test long barcode",
  "moved_by": "test_admin"
}
"@ 404

Write-Host "`n=== Testing Remove Operation Error Conditions ==="

# Test 6: Empty barcode for removal
Test-ErrorCondition "Remove with Empty Barcode" "/api/storage/remove" @"
{
  "barcode": "",
  "reason": "Test empty barcode removal",
  "removed_by": "test_admin"
}
"@ 400

# Test 7: Missing removed_by field
Test-ErrorCondition "Remove without removed_by Field" "/api/storage/remove" @"
{
  "barcode": "TEST_$timestamp",
  "reason": "Test missing removed_by"
}
"@ 422

# Test 8: Remove with special characters in barcode
Test-ErrorCondition "Remove with Special Characters" "/api/storage/remove" @"
{
  "barcode": "TEST@#$%^&*()_$timestamp",
  "reason": "Test special characters",
  "removed_by": "test_admin"
}
"@ 404

Write-Host "`n=== Testing Sample Creation Error Conditions ==="

# Test 9: Duplicate barcode creation
$duplicateBarcode = "DUPLICATE_$timestamp"

# First create a sample
$createJson = @"
{
  "samples": [
    {
      "name": "Original Sample",
      "barcode": "$duplicateBarcode",
      "location": "Freezer A (-80°C)"
    }
  ],
  "storage_location_id": 1,
  "template_name": "Duplicate Test",
  "stored_by": "test_admin"
}
"@

try {
    $response = Invoke-RestMethod -Uri "http://localhost:3000/api/samples/batch" -Method POST -Body $createJson -ContentType "application/json"
    Write-Host "✅ Created sample for duplicate test"
} catch {
    Write-Host "⚠️  Failed to create sample for duplicate test"
}

# Now try to create duplicate
Test-ErrorCondition "Create Sample with Duplicate Barcode" "/api/samples/batch" @"
{
  "samples": [
    {
      "name": "Duplicate Sample",
      "barcode": "$duplicateBarcode",
      "location": "Freezer A (-80°C)"
    }
  ],
  "storage_location_id": 1,
  "template_name": "Duplicate Test",
  "stored_by": "test_admin"
}
"@ 200  # This should succeed but report failed samples

# Test 10: Create sample with invalid storage location
Test-ErrorCondition "Create Sample with Invalid Storage Location" "/api/samples/batch" @"
{
  "samples": [
    {
      "name": "Invalid Location Sample",
      "barcode": "INVALID_LOC_$timestamp",
      "location": "Freezer A (-80°C)"
    }
  ],
  "storage_location_id": 999,
  "template_name": "Invalid Location Test",
  "stored_by": "test_admin"
}
"@ 500

Write-Host "`n=== Testing Malformed JSON ==="

# Test 11: Malformed JSON for move
Test-ErrorCondition "Move with Malformed JSON" "/api/storage/move" @"
{
  "barcode": "TEST_$timestamp",
  "location_id": 2,
  "reason": "Test malformed",
  "moved_by": "test_admin"
  // missing closing brace
"@ 400

# Test 12: Malformed JSON for remove
Test-ErrorCondition "Remove with Malformed JSON" "/api/storage/remove" @"
{
  "barcode": "TEST_$timestamp",
  "reason": "Test malformed"
  "removed_by": "test_admin"
}
"@ 400

Write-Host "`n=== Testing Data Type Mismatches ==="

# Test 13: String location_id in move
Test-ErrorCondition "Move with String Location ID" "/api/storage/move" @"
{
  "barcode": "TEST_$timestamp",
  "location_id": "not_a_number",
  "reason": "Test string location_id",
  "moved_by": "test_admin"
}
"@ 422

# Test 14: Array instead of string for barcode
Test-ErrorCondition "Move with Array Barcode" "/api/storage/move" @"
{
  "barcode": ["TEST_$timestamp"],
  "location_id": 2,
  "reason": "Test array barcode",
  "moved_by": "test_admin"
}
"@ 422

Write-Host "`n=== Testing Scan Operation Edge Cases ==="

# Test 15: Scan non-existent barcode
Test-ErrorCondition "Scan Non-Existent Barcode" "/api/storage/scan/NONEXISTENT_$timestamp" "" 404 "GET"

# Test 16: Scan with empty barcode
Test-ErrorCondition "Scan Empty Barcode" "/api/storage/scan/" "" 404 "GET"

# Test 17: Scan with special characters
Test-ErrorCondition "Scan Special Characters" "/api/storage/scan/TEST@#$%25^&*" "" 404 "GET"

Write-Host "`n=== Testing Boundary Conditions ==="

# Test 18: Move to location 0 (boundary test)
Test-ErrorCondition "Move to Location Zero" "/api/storage/move" @"
{
  "barcode": "BOUNDARY_$timestamp",
  "location_id": 0,
  "reason": "Test boundary location",
  "moved_by": "test_admin"
}
"@ 404

# Test 19: Negative location ID
Test-ErrorCondition "Move to Negative Location" "/api/storage/move" @"
{
  "barcode": "BOUNDARY_$timestamp",
  "location_id": -999,
  "reason": "Test negative location",
  "moved_by": "test_admin"
}
"@ 404

# Test 20: Maximum integer location ID
Test-ErrorCondition "Move to Maximum Integer Location" "/api/storage/move" @"
{
  "barcode": "BOUNDARY_$timestamp",
  "location_id": 2147483647,
  "reason": "Test max int location",
  "moved_by": "test_admin"
}
"@ 404

# Cleanup: Remove test sample
try {
    $removeJson = @"
{
  "barcode": "$duplicateBarcode",
  "reason": "Test cleanup",
  "removed_by": "test_admin"
}
"@
    $response = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/remove" -Method POST -Body $removeJson -ContentType "application/json"
    Write-Host "🧹 Cleaned up duplicate test sample"
} catch {
    Write-Host "⚠️  Could not clean up duplicate test sample"
}

# Results Summary
Write-Host "`n=== ERROR CONDITIONS TEST RESULTS ==="
$passCount = ($errorTests | Where-Object { $_.Result -eq "PASS" }).Count
$failCount = ($errorTests | Where-Object { $_.Result -eq "FAIL" }).Count
$totalTests = $errorTests.Count

Write-Host "Total Error Tests: $totalTests"
Write-Host "Correctly Handled: $passCount ✅"
Write-Host "Incorrectly Handled: $failCount ❌"
Write-Host "Error Handling Rate: $([math]::Round(($passCount / $totalTests) * 100, 1))%"

if ($failCount -gt 0) {
    Write-Host "`nIncorrectly Handled Errors:"
    $errorTests | Where-Object { $_.Result -eq "FAIL" } | ForEach-Object {
        Write-Host "  ❌ $($_.Name): Expected $($_.Expected), Got $($_.Actual)"
    }
}

Write-Host "`n🎯 Error conditions and edge cases testing completed!" 
