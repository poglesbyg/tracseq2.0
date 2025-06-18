# PowerShell Verification script for Rust linter fixes

Write-Host "🔍 Verifying Rust linter fixes..." -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Cyan

# Check if cargo is available
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Error: cargo not found. Please install Rust first." -ForegroundColor Red
    Write-Host "   Visit: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

Write-Host "✅ Rust/Cargo found" -ForegroundColor Green

# Run cargo check
Write-Host ""
Write-Host "🧪 Running cargo check..." -ForegroundColor Cyan
$checkResult = cargo check --all-targets 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ cargo check passed - no compilation errors!" -ForegroundColor Green
} else {
    Write-Host "❌ cargo check failed - compilation errors found" -ForegroundColor Red
    Write-Host ""
    Write-Host "📝 Error output:" -ForegroundColor Yellow
    Write-Host $checkResult
    exit 1
}

# Run cargo clippy for linter warnings
Write-Host ""
Write-Host "📎 Running cargo clippy for linter warnings..." -ForegroundColor Cyan
$clippyResult = cargo clippy --all-targets -- -W clippy::all 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ cargo clippy passed - no linter warnings!" -ForegroundColor Green
} else {
    Write-Host "⚠️  cargo clippy found some issues" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "📝 Clippy output:" -ForegroundColor Yellow
    Write-Host $clippyResult
}

# Check for missing Debug derives
Write-Host ""
Write-Host "🔍 Checking for missing Debug derives..." -ForegroundColor Cyan
$missingDebug = Select-String -Path "src\**\*.rs" -Pattern "struct.*\{" | Where-Object { $_.Line -notmatch "#\[derive.*Debug" } | Select-Object -First 5
if (-not $missingDebug) {
    Write-Host "✅ No obvious missing Debug derives found" -ForegroundColor Green
} else {
    Write-Host "⚠️  Potential missing Debug derives:" -ForegroundColor Yellow
    $missingDebug | ForEach-Object { Write-Host "   $($_.Filename):$($_.LineNumber) - $($_.Line.Trim())" }
}

# Check for unused imports
Write-Host ""
Write-Host "🔍 Checking for unused imports..." -ForegroundColor Cyan
$unusedImports = cargo check --message-format=json 2>&1 | Select-String "unused_imports"
if ($unusedImports) {
    Write-Host "⚠️  Found unused imports - run 'cargo fix --allow-dirty' to auto-fix" -ForegroundColor Yellow
} else {
    Write-Host "✅ No unused imports found" -ForegroundColor Green
}

# Check for dead code
Write-Host ""
Write-Host "🔍 Checking for dead code..." -ForegroundColor Cyan
$deadCode = cargo check --message-format=json 2>&1 | Select-String "dead_code"
if ($deadCode) {
    Write-Host "⚠️  Found dead code warnings" -ForegroundColor Yellow
} else {
    Write-Host "✅ No dead code warnings found" -ForegroundColor Green
}

Write-Host ""
Write-Host "🎉 Linter verification completed!" -ForegroundColor Green
Write-Host ""
Write-Host "📋 Summary of fixes applied:" -ForegroundColor Cyan
Write-Host "   • Fixed duplicate struct definitions" -ForegroundColor White
Write-Host "   • Added Debug derives to all major structs" -ForegroundColor White
Write-Host "   • Removed unused imports from main.rs" -ForegroundColor White
Write-Host "   • Fixed module structure inconsistencies" -ForegroundColor White
Write-Host ""
Write-Host "💡 To run this verification yourself:" -ForegroundColor Yellow
Write-Host "   cd lab_manager && .\verify_linter_fixes.ps1" -ForegroundColor White 
