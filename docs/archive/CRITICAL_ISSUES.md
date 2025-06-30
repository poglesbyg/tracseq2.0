# Critical Issues - TracSeq 2.0

## 🔴 MUST FIX BEFORE PRODUCTION

### 1. Python Syntax Error (Blocks Execution)
**File**: `lab_submission_rag/example_enhanced_usage.py`  
**Line**: 321  
**Error**: IndentationError  
**Fix**: Review and correct indentation at line 321

### 2. Rust Compilation Warnings (404 total)
**Most Affected Services**:
- `transaction_service`: 60 warnings
- `spreadsheet_versioning_service`: 40 warnings  
- `qaqc_service`: 15 warnings
- `lab_manager`: 6 warnings

**Common Issues**:
- Unused variables (prefix with `_`)
- Dead code (remove or implement)
- Unused imports (remove)

### 3. Missing Implementations (TODOs)
**Critical TODOs Found**:
- `spreadsheet_versioning_service/src/services/merge_engine.rs`:
  - `auto_merge()` - Not implemented
  - `create_merge_request()` - Not implemented
  - `merge_with_precedence()` - Not implemented

### 4. Test Infrastructure Issues
- Missing test database setup
- Incomplete test utilities in `qaqc_service/tests/test_utils.rs`
- No E2E test coverage for critical workflows

### 5. Service Health & Monitoring
- No health check endpoints implemented
- Missing metrics collection
- No distributed tracing setup

## 🟡 HIGH PRIORITY (Fix within 1 week)

### 1. Database Migration Verification
Check pending migrations in:
- `auth_service/migrations`
- `lab_manager/migrations`
- `sample_service/migrations`
- `transaction_service/migrations`

### 2. Python Environment Setup
```bash
# Setup virtual environments
cd lab_submission_rag && python3 -m venv venv && source venv/bin/activate && pip install -r requirements.txt
cd ../enhanced_rag_service && python3 -m venv venv && source venv/bin/activate && pip install -e .
```

### 3. API Gateway Configuration
- Review and update routing rules
- Implement rate limiting
- Add authentication middleware

### 4. Security Audit
- Review all endpoints for proper authentication
- Check for SQL injection vulnerabilities
- Implement input validation

## 🟢 IMPORTANT (Fix within 2 weeks)

### 1. Performance Optimization
- Add database indexes
- Implement caching layer
- Optimize query patterns

### 2. Documentation
- Generate API documentation
- Create deployment guides
- Document environment variables

### 3. Monitoring Setup
- Configure Prometheus metrics
- Setup log aggregation
- Implement alerting rules

## Quick Fixes Available

Run these commands for immediate improvements:

```bash
# Fix most Rust warnings automatically
cargo fix --workspace --allow-dirty

# Check frontend status
cd frontend && pnpm typecheck && pnpm lint

# Verify Python syntax
cd lab_submission_rag && python3 -m py_compile *.py

# Use the cleanup helper script
./cleanup_helper.sh
```

## Verification Checklist

- [ ] All services compile without errors
- [ ] Python syntax errors fixed
- [ ] Critical TODOs implemented
- [ ] Health endpoints available
- [ ] Database migrations current
- [ ] Tests passing
- [ ] Documentation updated

## Support Resources

- **Cleanup Helper**: `./cleanup_helper.sh`
- **Full Recommendations**: `CLEANUP_RECOMMENDATIONS.md`
- **Build Fixes History**: `BUILD_FIXES_SUMMARY.md`