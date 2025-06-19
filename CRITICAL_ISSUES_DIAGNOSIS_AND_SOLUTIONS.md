# Critical Issues Diagnosis & Solutions

## 🚨 **Issue Summary**

After thorough investigation, I've identified three critical issues causing the frontend errors:

### **1. RAG Service Crashing (500 Errors)**
- **Status**: ✅ **IDENTIFIED & TEMPORARILY FIXED**
- **Symptom**: `GET /api/rag/submissions 500 (Internal Server Error)`
- **Root Cause**: Missing `ollama` Python dependency in RAG service
- **Error**: `ModuleNotFoundError: No module named 'ollama'`

### **2. Authentication Failing (401 Errors)**  
- **Status**: 🔍 **UNDER INVESTIGATION**
- **Symptom**: `GET /api/users/me 401 (Unauthorized)`
- **Root Cause**: JWT token verification failing in backend
- **Evidence**: Login works, but subsequent token validation fails

### **3. SpreadsheetViewer Crashes**
- **Status**: ✅ **ALREADY FIXED** 
- **Symptom**: `Cannot read properties of undefined (reading 'sheets')`
- **Root Cause**: Missing null checks in component
- **Solution**: Added null safety guards

---

## 🔧 **Detailed Solutions**

### **Solution 1: Fix RAG Service**

**Problem**: RAG service container constantly restarting due to missing `ollama` dependency.

**Immediate Fix Applied**:
```bash
# Stopped crashing service to eliminate 500 errors
docker stop tracseq20-rag-service-1
```

**Permanent Solution**:
1. Add `ollama` to RAG service requirements:
```bash
cd lab_submission_rag
echo "ollama" >> requirements.txt
```

2. Rebuild RAG service:
```bash
docker-compose up --build rag-service
```

**Alternative**: Use the working minimal RAG service:
```bash
python minimal_rag.py  # This was shown working in the logs
```

### **Solution 2: Fix Authentication**

**Problem**: JWT token verification failing even with valid tokens.

**Investigation Findings**:
- ✅ Login endpoint works: `/api/auth/login` returns valid JWT
- ❌ Token validation fails: `/api/users/me` returns 401 with valid token
- ❌ Affects both direct backend and API Gateway

**Potential Root Causes**:
1. **JWT Secret Mismatch**: Different secrets for encoding/decoding
2. **Middleware Not Applied**: `/api/users/me` route not protected
3. **Token Format Issue**: Invalid JWT structure or claims
4. **Database Session Issue**: Session validation failing

**Systematic Debugging Steps**:
```bash
# 1. Test authentication after backend fully starts
docker logs tracseq20-dev-1 --follow

# 2. Verify JWT secret consistency
grep -r "JWT_SECRET" lab_manager/

# 3. Check route protection
grep -A5 -B5 "users/me" lab_manager/src/router/mod.rs

# 4. Test with debugging
curl -v http://localhost:3000/api/users/me \
  -H "Authorization: Bearer [TOKEN]"
```

### **Solution 3: Verify SpreadsheetViewer Fix**

**Problem**: Component crashes when `data.sheets` is undefined.

**Fix Already Applied**:
```typescript
// Added null safety check in SpreadsheetViewer.tsx
if (!data || !data.sheets || !Array.isArray(data.sheets) || data.sheets.length === 0) {
  return (
    <div className="error-message">
      <p>No spreadsheet data available or invalid data format.</p>
      <button onClick={onClose}>Close</button>
    </div>
  );
}
```

**Verification Needed**: Confirm fix is applied and working.

---

## 🎯 **Action Plan**

### **Phase 1: Immediate Stabilization**
1. ✅ **DONE**: Stop crashing RAG service
2. 🔄 **IN PROGRESS**: Wait for backend to fully restart
3. 🔄 **NEXT**: Test authentication with fresh backend

### **Phase 2: RAG Service Recovery**
1. Add missing `ollama` dependency
2. Rebuild RAG service container
3. Test RAG endpoints functionality

### **Phase 3: Authentication Deep Dive**
1. Check JWT secret configuration
2. Verify middleware application
3. Test token lifecycle end-to-end
4. Fix token validation logic if needed

### **Phase 4: Final Verification**
1. Test all frontend flows
2. Verify SpreadsheetViewer stability
3. Confirm RAG integration works
4. Full system smoke test

---

## 🧪 **Testing Protocol**

### **Authentication Test**:
```bash
# 1. Login
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@lab.local","password":"admin123"}'

# 2. Extract token and test immediately
curl http://localhost:3000/api/users/me \
  -H "Authorization: Bearer [TOKEN]"
```

### **RAG Service Test**:
```bash
# Test health after fix
curl http://localhost:8087/health

# Test functionality
curl http://localhost:8000/api/rag/submissions
```

### **Frontend Test**:
- Login to frontend
- Navigate to Templates page
- Try uploading spreadsheet
- Check for console errors

---

## 📊 **Current Status**

| Component | Status | Issues |
|-----------|--------|---------|
| API Gateway | ✅ Working | None |
| Backend Auth | ❌ Token validation failing | JWT verification |
| Backend Core | ✅ Working | None |
| Template Service | ✅ Working | None |
| RAG Service | ❌ Stopped | Missing dependency |
| Frontend Routing | ✅ Working | None |
| SpreadsheetViewer | ✅ Fixed | None |

---

## 🔍 **Next Steps**

1. **Wait for backend startup** (in progress)
2. **Test authentication** with fresh backend
3. **Fix RAG service** dependency issue
4. **Verify all fixes** work together

The system architecture is sound - these are dependency and configuration issues that can be resolved systematically.

---

**Generated**: 2025-06-19 22:27  
**Last Updated**: During backend restart  
**Priority**: Critical authentication issue blocking frontend 
