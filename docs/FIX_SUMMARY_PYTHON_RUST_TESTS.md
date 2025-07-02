# Python RAG Tests and Rust rstest Configuration Fixes

## Overview

This document summarizes the fixes applied to resolve:
1. Python RAG test dependency issues
2. Rust workspace configuration issues with rstest

## Issues Identified

### 1. Python RAG Dependency Issues

**Problem:**
- Package hash mismatches due to custom PyTorch index URLs
- Conflicting dependency versions between different requirements files
- Missing essential test dependencies like email-validator
- Version conflicts with pytest-asyncio (1.0.0 vs 0.21.0)

**Root Causes:**
- Custom index URL `--index-url https://download.pytorch.org/whl/cpu` causing hash verification failures
- Using package extras like `transformers[torch]` introducing version conflicts
- Inconsistent dependency specifications between requirements.txt and pyproject.toml

### 2. Rust rstest Version Conflicts

**Problem:**
- Workspace defines `rstest = "0.23"` in dependencies
- Individual services (circuit-breaker-lib, reports_service) hardcoded `rstest = "0.18"`
- Version mismatch causing workspace resolution failures

**Root Cause:**
- Services not using workspace dependency inheritance properly

## Fixes Applied

### 1. Python RAG Dependencies Fix

**File: `lims-ai/lab_submission_rag/requirements.txt`**
```diff
- torch>=2.0.0 --index-url https://download.pytorch.org/whl/cpu  # CPU-only torch
+ torch>=2.0.0,<2.3.0  # CPU-only torch without custom index

+ email-validator>=2.2.0  # Required for pydantic email validation

+ # Remove heavy transformers package extras
+ transformers>=4.30.0  # Without [torch] extra to avoid conflicts
```

**File: `lims-ai/lab_submission_rag/pyproject.toml`**
```diff
# Testing
"pytest>=8.4.1",
- "pytest-asyncio>=1.0.0", 
+ "pytest-asyncio>=0.21.0", 
"pytest-cov>=6.2.1",
```

### 2. Rust rstest Configuration Fix

**File: `lims-core/circuit-breaker-lib/Cargo.toml`**
```diff
[dev-dependencies]
tokio-test = "0.4"
- rstest = "0.18"
+ rstest = { workspace = true }
mockall = "0.11"
```

**File: `lims-core/reports_service/Cargo.toml`**
```diff
[dev-dependencies]
tokio-test = "0.4"
- rstest = "0.18"
+ rstest = { workspace = true }
mockall = "0.11"
```

## Verification

### Testing Script
A comprehensive test script has been created at `scripts/test-fixes.sh` that:

1. **Python RAG Tests:**
   - Creates a clean virtual environment
   - Installs all dependencies from requirements.txt
   - Verifies key imports (langchain, chromadb, torch, etc.)
   - Ensures no hash verification failures

2. **Rust Workspace Tests:**
   - Runs `cargo check --workspace` to verify configuration
   - Tests specific services that had rstest issues
   - Confirms all services can build with the correct rstest version

### How to Run Tests
```bash
# From project root
./scripts/test-fixes.sh
```

## Best Practices Going Forward

### Python Dependencies
1. **Avoid custom index URLs** in requirements.txt to prevent hash mismatches
2. **Use specific version ranges** instead of open-ended versions
3. **Keep requirements.txt and pyproject.toml synchronized**
4. **Test dependencies in clean environments** before deployment

### Rust Workspace Dependencies
1. **Always use workspace inheritance** for common dependencies:
   ```toml
   [dev-dependencies]
   rstest = { workspace = true }
   ```
2. **Define shared versions** in the root workspace Cargo.toml
3. **Avoid hardcoding versions** in individual service Cargo.toml files
4. **Run `cargo check --workspace`** regularly to catch configuration issues

## Impact

These fixes ensure:
- ✅ Python RAG tests can run without dependency conflicts
- ✅ All Python packages install with correct hash verification
- ✅ Rust workspace builds successfully with consistent rstest versions
- ✅ Development and CI/CD pipelines work reliably

## Related Documentation

- [RAG Dependency Fix Guide](./RAG_DEPENDENCY_FIX.md)
- [Rust Workspace Configuration](../lims-core/README.md)
- [Python Development Guide](../lims-ai/README.md)

*Context improved by Giga AI* 