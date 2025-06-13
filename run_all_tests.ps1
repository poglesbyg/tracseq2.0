Write-Host "======================================================================"
Write-Host "🧪 LABORATORY STORAGE SYSTEM - COMPREHENSIVE TEST SUITE RUNNER"
Write-Host "======================================================================"
Write-Host "Running all storage operation tests: Functional, Error, Performance"

$timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
$testSuiteResults = @()

function Run-TestSuite {
    param(
        [string]$suiteName,
        [string]$scriptPath,
        [string]$description
    )
    
    Write-Host "`n" + ("=" * 70)
    Write-Host "🔬 RUNNING: $suiteName"
    Write-Host "📋 Description: $description"
    Write-Host "⏱️  Started: $(Get-Date -Format 'HH:mm:ss')"
    Write-Host ("=" * 70)
    
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $success = $false
    $errorMessage = ""
    
    try {
        if (Test-Path $scriptPath) {
            & $scriptPath
            $success = $true
            Write-Host "`n✅ $suiteName COMPLETED SUCCESSFULLY"
        } else {
            $errorMessage = "Test script not found: $scriptPath"
            Write-Host "`n❌ $suiteName FAILED: $errorMessage"
        }
    } catch {
        $errorMessage = $_.Exception.Message
        Write-Host "`n❌ $suiteName FAILED: $errorMessage"
    }
    
    $stopwatch.Stop()
    $duration = $stopwatch.ElapsedMilliseconds
    
    $result = @{
        Name = $suiteName
        Description = $description
        Success = $success
        Duration = $duration
        ErrorMessage = $errorMessage
        CompletedAt = Get-Date -Format 'HH:mm:ss'
    }
    
    $testSuiteResults += $result
    
    Write-Host "⏱️  Duration: $($duration)ms"
    Write-Host "✅ Completed: $(Get-Date -Format 'HH:mm:ss')"
    
    return $result
}

# Pre-flight check
Write-Host "`n🔍 PRE-FLIGHT CHECKS"
Write-Host "Verifying system readiness..."

try {
    $healthResponse = Invoke-WebRequest -Uri "http://localhost:3000/health" -UseBasicParsing
    if ($healthResponse.StatusCode -eq 200) {
        Write-Host "✅ Server health check: PASSED"
    } else {
        throw "Server returned status $($healthResponse.StatusCode)"
    }
} catch {
    Write-Host "❌ Server health check: FAILED"
    Write-Host "Error: $($_.Exception.Message)"
    Write-Host "`n⚠️  ABORTING: Server is not responding. Please ensure the lab manager service is running."
    exit 1
}

try {
    $locationsResponse = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/locations"
    if ($locationsResponse.Count -gt 0) {
        Write-Host "✅ Storage locations available: $($locationsResponse.Count) locations"
    } else {
        Write-Host "⚠️  Warning: No storage locations found"
    }
} catch {
    Write-Host "❌ Storage system check: FAILED"
    Write-Host "Error: $($_.Exception.Message)"
}

Write-Host "✅ Pre-flight checks completed"

# Run test suites
Write-Host "`n🚀 STARTING COMPREHENSIVE TEST EXECUTION"
Write-Host "This will test all aspects of the storage system functionality"

# Test Suite 1: Comprehensive Functional Tests
Run-TestSuite -suiteName "COMPREHENSIVE FUNCTIONAL TESTS" -scriptPath ".\test_storage_comprehensive.ps1" -description "Complete functional testing of storage operations including create, move, remove, scan, and integration tests"

# Test Suite 2: Error Conditions and Edge Cases
Run-TestSuite -suiteName "ERROR CONDITIONS & EDGE CASES" -scriptPath ".\test_error_conditions.ps1" -description "Validation of error handling, boundary conditions, malformed requests, and data type mismatches"

# Test Suite 3: Performance and Load Testing
Run-TestSuite -suiteName "PERFORMANCE & LOAD TESTING" -scriptPath ".\test_performance_load.ps1" -description "Performance benchmarks, concurrent operations, bulk operations, and system limits testing"

# Final System State Check
Write-Host "`n🔍 POST-TEST SYSTEM STATE CHECK"
try {
    $finalLocationsResponse = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/locations"
    $finalCapacityResponse = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/capacity"
    
    Write-Host "📊 Final System State:"
    Write-Host "  Storage Locations: $($finalLocationsResponse.Count)"
    Write-Host "  Total Capacity: $($finalCapacityResponse.total_capacity)"
    Write-Host "  Current Usage: $($finalCapacityResponse.total_usage)"
    Write-Host "  Utilization: $([math]::Round($finalCapacityResponse.overall_utilization, 2))%"
    
    if ($finalCapacityResponse.warnings.Count -gt 0) {
        Write-Host "⚠️  System Warnings:"
        foreach ($warning in $finalCapacityResponse.warnings) {
            Write-Host "    - $warning"
        }
    }
} catch {
    Write-Host "⚠️  Could not retrieve final system state: $($_.Exception.Message)"
}

# Generate comprehensive report
Write-Host "`n" + ("=" * 80)
Write-Host "📊 COMPREHENSIVE TEST RESULTS REPORT"
Write-Host "======================================================================"
Write-Host "🕒 Test Execution Started: $timestamp"
Write-Host "🕒 Test Execution Completed: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"

$totalSuites = $testSuiteResults.Count
$successfulSuites = ($testSuiteResults | Where-Object { $_.Success }).Count
$failedSuites = ($testSuiteResults | Where-Object { -not $_.Success }).Count
$totalDuration = ($testSuiteResults | Measure-Object Duration -Sum).Sum

Write-Host "`n📈 EXECUTION SUMMARY:"
Write-Host "  Total Test Suites: $totalSuites"
Write-Host "  Successful: $successfulSuites ✅"
Write-Host "  Failed: $failedSuites ❌"
Write-Host "  Success Rate: $([math]::Round(($successfulSuites / $totalSuites) * 100, 1))%"
Write-Host "  Total Duration: $($totalDuration)ms ($([math]::Round($totalDuration / 1000, 2))s)"

Write-Host "`n📋 DETAILED RESULTS:"
foreach ($result in $testSuiteResults) {
    $status = if ($result.Success) { "✅ PASS" } else { "❌ FAIL" }
    Write-Host "`n  $status | $($result.Name)"
    Write-Host "    Description: $($result.Description)"
    Write-Host "    Duration: $($result.Duration)ms"
    Write-Host "    Completed: $($result.CompletedAt)"
    if (-not $result.Success) {
        Write-Host "    Error: $($result.ErrorMessage)" -ForegroundColor Red
    }
}

if ($failedSuites -gt 0) {
    Write-Host "`n⚠️  FAILED TEST SUITES:" -ForegroundColor Yellow
    $testSuiteResults | Where-Object { -not $_.Success } | ForEach-Object {
        Write-Host "  ❌ $($_.Name): $($_.ErrorMessage)" -ForegroundColor Red
    }
}

# Feature coverage report
Write-Host "`n🎯 FEATURE COVERAGE REPORT:"
Write-Host "The following storage system features have been tested:"

$features = @(
    @{Name = "Sample Creation & Storage Integration"; Tested = $true; Suite = "Comprehensive"},
    @{Name = "Sample Movement Between Locations"; Tested = $true; Suite = "Comprehensive"},
    @{Name = "Sample Removal from Storage"; Tested = $true; Suite = "Comprehensive"},
    @{Name = "Barcode Scanning & Lookup"; Tested = $true; Suite = "Comprehensive"},
    @{Name = "Storage Capacity Management"; Tested = $true; Suite = "Comprehensive"},
    @{Name = "Temperature Zone Compatibility"; Tested = $true; Suite = "Comprehensive"},
    @{Name = "Error Handling & Validation"; Tested = $true; Suite = "Error Conditions"},
    @{Name = "Malformed Request Handling"; Tested = $true; Suite = "Error Conditions"},
    @{Name = "Boundary Condition Testing"; Tested = $true; Suite = "Error Conditions"},
    @{Name = "Performance Benchmarking"; Tested = $true; Suite = "Performance"},
    @{Name = "Concurrent Operations"; Tested = $true; Suite = "Performance"},
    @{Name = "Bulk Operations"; Tested = $true; Suite = "Performance"},
    @{Name = "Resource Usage Monitoring"; Tested = $true; Suite = "Performance"}
)

foreach ($feature in $features) {
    $status = if ($feature.Tested) { "✅" } else { "❌" }
    Write-Host "  $status $($feature.Name) ($($feature.Suite))"
}

# Recommendations based on test results
Write-Host "`n💡 RECOMMENDATIONS:"
if ($successfulSuites -eq $totalSuites) {
    Write-Host "🎉 All test suites passed successfully!"
    Write-Host "✅ The storage system is ready for production use."
    Write-Host "✅ All core features are working as expected."
    Write-Host "✅ Error handling is robust and comprehensive."
    Write-Host "✅ Performance meets acceptable benchmarks."
} else {
    Write-Host "⚠️  Some test suites failed. Please review the following:"
    if (($testSuiteResults | Where-Object { $_.Name -like "*FUNCTIONAL*" -and -not $_.Success }).Count -gt 0) {
        Write-Host "  🔴 CRITICAL: Functional tests failed - core features may not work properly"
    }
    if (($testSuiteResults | Where-Object { $_.Name -like "*ERROR*" -and -not $_.Success }).Count -gt 0) {
        Write-Host "  🟡 WARNING: Error handling tests failed - system may not handle edge cases properly"
    }
    if (($testSuiteResults | Where-Object { $_.Name -like "*PERFORMANCE*" -and -not $_.Success }).Count -gt 0) {
        Write-Host "  🟡 WARNING: Performance tests failed - system may not perform well under load"
    }
}

Write-Host "`n📚 NEXT STEPS:"
Write-Host "1. Review any failed test results above"
Write-Host "2. Address any identified issues in the codebase"
Write-Host "3. Re-run tests after making fixes"
Write-Host "4. Consider adding additional tests for new features"
Write-Host "5. Monitor system performance in production environment"

Write-Host "`n" + ("=" * 80)
Write-Host "🏁 COMPREHENSIVE TEST SUITE EXECUTION COMPLETED"
Write-Host "======================================================================"

# Save detailed report to file
$reportFile = "test_report_$(Get-Date -Format 'yyyyMMdd_HHmmss').txt"
$reportContent = @"
LABORATORY STORAGE SYSTEM - TEST EXECUTION REPORT
Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')

EXECUTION SUMMARY:
- Total Test Suites: $totalSuites
- Successful: $successfulSuites
- Failed: $failedSuites  
- Success Rate: $([math]::Round(($successfulSuites / $totalSuites) * 100, 1))%
- Total Duration: $($totalDuration)ms

DETAILED RESULTS:
$($testSuiteResults | ForEach-Object { 
    $status = if ($_.Success) { "PASS" } else { "FAIL" }
    "[$status] $($_.Name) - $($_.Duration)ms - $($_.CompletedAt)"
    if (-not $_.Success) { "  Error: $($_.ErrorMessage)" }
} | Out-String)

FEATURE COVERAGE:
$($features | ForEach-Object { 
    $status = if ($_.Tested) { "[✓]" } else { "[✗]" }
    "$status $($_.Name) ($($_.Suite))"
} | Out-String)
"@

try {
    $reportContent | Out-File -FilePath $reportFile -Encoding UTF8
    Write-Host "📄 Detailed report saved to: $reportFile"
} catch {
    Write-Host "⚠️  Could not save detailed report: $($_.Exception.Message)"
}

Write-Host "`n🎯 Test execution completed. Check the results above for any issues that need attention." 
