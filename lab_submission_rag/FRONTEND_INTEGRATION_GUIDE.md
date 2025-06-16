# Frontend Integration Guide for RAG Submissions

## 🎯 **COMPLETE SOLUTION: Make http://localhost:8080/rag-submissions Work**

Your lab_manager frontend needs to connect to our **RAG API Bridge** to display RAG submissions data.

## ✅ **Current Status**
- ✅ RAG API Bridge running on **port 3002**
- ✅ Database populated with sample submissions 
- ✅ API endpoints working perfectly
- ✅ CORS configured for lab_manager frontend

**Test Results:**
```bash
curl http://localhost:3002/api/rag/submissions
# Returns: 3 RAG submissions with proper data
```

## 🔧 **Integration Options**

### **Option 1: Frontend Environment Variable (Recommended)**

If your lab_manager frontend uses environment variables, add:

```bash
# Frontend .env file
REACT_APP_RAG_API_URL=http://localhost:3002
RAG_SUBMISSIONS_ENDPOINT=/api/rag/submissions
```

### **Option 2: Frontend Configuration File**

Update your frontend config (usually `src/config.js` or similar):

```javascript
// Frontend config
export const API_ENDPOINTS = {
  labManager: 'http://localhost:3001',
  ragSubmissions: 'http://localhost:3002/api/rag/submissions',
  ragStats: 'http://localhost:3002/api/rag/stats'
}
```

### **Option 3: Proxy Configuration**

Add to lab_manager backend (`package.json` or proxy config):

```json
{
  "proxy": {
    "/api/rag/*": {
      "target": "http://localhost:3002",
      "changeOrigin": true
    }
  }
}
```

## 📡 **Available API Endpoints**

Your frontend can now access:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/rag/submissions` | GET | List all RAG submissions |
| `/api/rag/submissions/{id}` | GET | Get submission details |
| `/api/rag/stats` | GET | Get RAG statistics |
| `/api/rag/process` | POST | Process new documents |

## 🧪 **Sample API Response**

```json
[
  {
    "id": "3b212883",
    "submission_id": "d550f579-a804-4239-9c40-e608afb6e964",
    "submitter_name": "Dr. Emily Johnson",
    "submitter_email": "e.johnson@hospital.edu",
    "sample_type": "Bacterial isolate",
    "sample_name": "Urgent Clinical Sample",
    "confidence_score": 0.95,
    "created_at": "2025-06-13T17:45:14.942058",
    "status": "completed"
  }
]
```

## 🔄 **Quick Test**

1. **Verify API Bridge is running:**
   ```bash
   curl http://localhost:3002/health
   # Should return: {"status": "healthy", "database": "connected"}
   ```

2. **Check frontend can access the API:**
   - Open browser dev tools on http://localhost:8080/rag-submissions
   - Try fetching: `fetch('http://localhost:3002/api/rag/submissions')`
   - Should return the RAG submissions data

3. **Update frontend code to call our API instead of the old port 8000**

## 🚀 **How to Make the Page Work Right Now**

### **Immediate Solution:**

1. **Frontend JavaScript Update:**
   Find where your frontend calls the RAG API (likely in the rag-submissions page component) and change:

   ```javascript
   // OLD (broken)
   fetch('http://localhost:8000/api/submissions')
   
   // NEW (working)
   fetch('http://localhost:3002/api/rag/submissions')
   ```

2. **Or update the lab_manager backend to proxy RAG requests:**
   Add this to your backend routes:

   ```javascript
   // In lab_manager backend
   app.get('/api/rag/submissions', async (req, res) => {
     const response = await fetch('http://localhost:3002/api/rag/submissions');
     const data = await response.json();
     res.json(data);
   });
   ```

## 🎯 **Expected Result**

After integration, http://localhost:8080/rag-submissions will show:

- **3 RAG submissions** from our database
- **Submitter names:** Dr. Emily Johnson, Dr. Sarah Chen, Dr. Maria Rodriguez  
- **Sample types:** Bacterial isolate, Stool sample, FFPE Tissue
- **Confidence scores:** 0.95, 0.88, 0.92
- **Status:** All marked as "completed"

## 🔄 **Services Status**

| Service | Port | Status | Purpose |
|---------|------|--------|---------|
| lab_manager frontend | 8080 | ✅ Running | User interface |
| lab_manager backend | 3001 | ✅ Running | Main API |
| **RAG API Bridge** | **3002** | ✅ **Running** | **RAG data provider** |
| PostgreSQL | 5433 | ✅ Running | Database |

## 🎉 **Success Criteria**

✅ RAG API Bridge operational  
✅ Database populated with sample data  
✅ API endpoints returning proper JSON  
✅ CORS configured for frontend access  
✅ Frontend page exists at http://localhost:8080/rag-submissions

**Next Step:** Update your frontend to call `localhost:3002` instead of `localhost:8000` and the RAG submissions page will work perfectly!

## 🆘 **Need Help?**

If you need help updating the frontend code, share the rag-submissions page component and I'll show you exactly what to change.

---

*This solution provides a clean API bridge that serves your fixed RAG system data to the lab_manager frontend without validation errors.* 
