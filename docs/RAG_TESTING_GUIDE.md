# RAG Submissions Testing Guide

## Test Documents Created

### 1. **Comprehensive Example** (`example_lab_submission.txt`)
- **Size**: ~8KB, detailed laboratory form
- **Contains**: Multiple samples, pooled samples, complete metadata
- **Best for**: Testing full AI extraction capabilities
- **Expected results**: 3+ samples extracted with high confidence

### 2. **Simple Example** (`simple_lab_submission.txt`) 
- **Size**: ~1KB, basic submission form
- **Contains**: 3 samples with essential information
- **Best for**: Quick testing and validation
- **Expected results**: 3 samples extracted quickly

## How to Test

### Step 1: Start the Application
```bash
# Terminal 1: Start backend
cargo run

# Terminal 2: Start frontend  
cd frontend && npm run dev
```

### Step 2: Navigate to RAG Submissions
1. Open browser to `http://localhost:5173`
2. Click **"AI Submissions"** in the sidebar
3. You should see the AI-powered document submissions page

### Step 3: Test Document Upload

#### **Quick Test (Simple Document)**
1. **Upload**: Drag `simple_lab_submission.txt` to the upload area
2. **Settings**: 
   - Confidence Threshold: 0.7 (default)
   - Auto-create: ✅ checked
3. **Action**: Click **"Process & Extract"**
4. **Expected Result**: 3 samples created automatically

#### **Advanced Test (Comprehensive Document)**
1. **Upload**: Drag `example_lab_submission.txt` to the upload area  
2. **Settings**:
   - Confidence Threshold: 0.8 (higher for quality)
   - Auto-create: ❌ unchecked (preview first)
3. **Action**: Click **"Preview"** first
4. **Review**: Check extracted sample details
5. **Confirm**: Click **"Confirm & Create Samples"**
6. **Expected Result**: Multiple samples with rich metadata

### Step 4: Test Natural Language Queries
After processing documents, try these queries:

- **"How many DNA samples were submitted?"**
- **"Show me all high priority samples"**
- **"What samples need -80°C storage?"**
- **"List all samples from Dr. Johnson"**

## Creating Additional Test Documents

### PDF Version (recommended for realistic testing)
1. Copy content from `example_lab_submission.txt`
2. Paste into Google Docs or Word
3. Add some formatting (headers, tables)
4. Export as PDF
5. Test with more realistic document format

### DOCX Version
1. Open Microsoft Word
2. Create a form-like document with the sample data
3. Use tables for sample information
4. Save as `.docx`
5. Test Word document processing

### Custom Test Documents
Create documents with:
- **Missing information** (test warnings)
- **Low-quality text** (test confidence scoring) 
- **Multiple formats** (tables vs. paragraphs)
- **Different sample types** (RNA, protein, blood, tissue)

## Expected AI Extraction Results

### **From Simple Document**:
```
Sample 1:
- Name: "Liver Tissue RNA" 
- Barcode: Auto-generated (RNA-YYYYMMDDHHMMSS-XXX)
- Location: "Storage-minus80"

Sample 2: 
- Name: "Brain Tissue RNA"
- Barcode: Auto-generated  
- Location: "Storage-minus80"

Sample 3:
- Name: "Control DNA Sample"
- Barcode: Auto-generated (DNA-YYYYMMDDHHMMSS-XXX)
- Location: "Storage-minus80"
```

### **From Comprehensive Document**:
```
Sample 1:
- Name: "Patient 001 Germline DNA"
- Barcode: "DNA-240115123456-ABC" (from document)
- Location: "Storage-minus80"

Sample 2:
- Name: "Patient 001 Tumor DNA" 
- Barcode: Auto-generated
- Location: "Storage-minus80"

Pooled Sample:
- Name: "POOL-001"
- Contains: 3 individual samples
```

## Troubleshooting

### **No RAG System Available**
- Error: "RAG system unavailable"
- **Solution**: The RAG system at `http://localhost:8000` is not running
- **For testing**: The frontend will show appropriate error messages

### **Low Confidence Scores**
- **Cause**: Poorly formatted or unclear document text
- **Solution**: Use well-structured documents with clear labels

### **No Samples Extracted**
- **Cause**: Document doesn't contain recognizable sample information
- **Solution**: Ensure document has sample IDs, names, or clear sample sections

### **File Upload Issues**
- **Check**: File size under 50MB
- **Check**: File format is PDF, DOCX, or TXT
- **Check**: Backend is running and accessible

## Advanced Testing Scenarios

1. **Batch Processing**: Upload multiple documents sequentially
2. **Error Handling**: Try uploading unsupported file types
3. **Large Files**: Test with complex multi-page documents
4. **Edge Cases**: Documents with minimal or excessive information
5. **Performance**: Time the processing of different document sizes

## Success Criteria

✅ **File upload works smoothly**  
✅ **AI extracts sample information correctly**  
✅ **Confidence scores are reasonable (>0.7)**  
✅ **Samples are created in the system**  
✅ **Natural language queries return relevant answers**  
✅ **Error handling works for edge cases**  
✅ **UI is responsive and user-friendly** 
