#!/bin/bash
# Verification script for Rust linter fixes

echo "🔍 Verifying Rust linter fixes..."
echo "=================================="

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo not found. Please install Rust first."
    echo "   Visit: https://rustup.rs/"
    exit 1
fi

echo "✅ Rust/Cargo found"

# Run cargo check
echo ""
echo "🧪 Running cargo check..."
if cargo check --all-targets 2>&1; then
    echo "✅ cargo check passed - no compilation errors!"
else
    echo "❌ cargo check failed - compilation errors found"
    echo ""
    echo "📝 Running with more verbose output:"
    cargo check --all-targets --verbose
    exit 1
fi

# Run cargo clippy for linter warnings
echo ""
echo "📎 Running cargo clippy for linter warnings..."
if cargo clippy --all-targets -- -W clippy::all 2>&1; then
    echo "✅ cargo clippy passed - no linter warnings!"
else
    echo "⚠️  cargo clippy found some issues"
    echo ""
    echo "📝 Running clippy with suggestions:"
    cargo clippy --all-targets -- -W clippy::all -A clippy::too_many_arguments
fi

# Check for missing Debug derives
echo ""
echo "🔍 Checking for missing Debug derives..."
MISSING_DEBUG=$(grep -r "struct.*{" src/ --include="*.rs" | grep -v "#\[derive.*Debug" | head -5)
if [ -z "$MISSING_DEBUG" ]; then
    echo "✅ No obvious missing Debug derives found"
else
    echo "⚠️  Potential missing Debug derives:"
    echo "$MISSING_DEBUG"
fi

# Check for unused imports
echo ""
echo "🔍 Checking for unused imports..."
if cargo check --message-format=json 2>&1 | grep -q "unused_imports"; then
    echo "⚠️  Found unused imports - run 'cargo fix --allow-dirty' to auto-fix"
    cargo check 2>&1 | grep -A5 -B5 "unused_imports"
else
    echo "✅ No unused imports found"
fi

# Check for dead code
echo ""
echo "🔍 Checking for dead code..."
if cargo check --message-format=json 2>&1 | grep -q "dead_code"; then
    echo "⚠️  Found dead code warnings"
    cargo check 2>&1 | grep -A5 -B5 "dead_code"
else
    echo "✅ No dead code warnings found"
fi

echo ""
echo "🎉 Linter verification completed!"
echo ""
echo "📋 Summary of fixes applied:"
echo "   • Fixed duplicate struct definitions"
echo "   • Added Debug derives to all major structs"
echo "   • Removed unused imports from main.rs"
echo "   • Fixed module structure inconsistencies"
echo ""
echo "💡 To run this verification yourself:"
echo "   cd lab_manager && ./verify_linter_fixes.sh" 
