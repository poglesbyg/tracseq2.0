# ChatBot Backend Integration Guide

## Overview

The TracSeq 2.0 ChatBot is now fully integrated with backend API endpoints, providing real-time streaming responses, document processing, sample creation, and protocol retrieval capabilities.

## Implemented API Endpoints

### 1. Chat Streaming Endpoint
**`POST /api/chat/stream`**

Provides real-time streaming responses using Server-Sent Events (SSE).

**Request:**
```javascript
FormData {
  message: string
  conversationId: string
  files?: File[] (optional)
}
```

**Response:** Event stream with chunks
```javascript
data: {"id": "...", "content": "chunk text", "type": "chunk", "timestamp": "..."}
data: {"id": "...", "type": "completion", "metadata": {...}}
data: [DONE]
```

**Frontend Implementation:**
- Uses native Fetch API with ReadableStream
- Processes SSE data in real-time
- Updates UI progressively as chunks arrive

### 2. Document Processing Endpoint
**`POST /api/documents/process`**

Processes uploaded laboratory documents and extracts structured data.

**Request:**
```javascript
FormData {
  file: File (PDF, Excel, CSV)
  metadata?: string (optional JSON)
}
```

**Response:**
```json
{
  "success": true,
  "extracted_data": {
    "document_type": "laboratory_submission",
    "extracted_fields": {
      "submitter": {...},
      "samples": [...],
      "storage_requirements": {...},
      "project_info": {...}
    }
  },
  "confidence": 0.92,
  "validation_errors": null
}
```

**Supported File Types:**
- PDF submission forms
- Excel sample sheets (.xlsx, .xls)
- CSV data files

### 3. Sample Creation Endpoint
**`POST /api/samples/create`**

Creates new samples from chat interactions.

**Request:**
```json
{
  "sample_type": "DNA",
  "volume": 50.0,
  "concentration": 125.0,
  "buffer": "TE",
  "storage_temperature": -20,
  "storage_location": {
    "freezer": "A",
    "shelf": "1",
    "box": "B2"
  },
  "project_id": "PROJ-2024-001",
  "principal_investigator": "Dr. Smith",
  "metadata": {...}
}
```

**Response:**
```json
{
  "success": true,
  "sample": {
    "id": "SAMP-20240703-A1B2C3",
    "barcode": "TSQSAMP20240703A1B2C3",
    "type": "DNA",
    "status": "registered",
    "created_at": "2024-07-03T12:00:00Z"
  },
  "actions": {
    "print_label": "/api/samples/{id}/label",
    "view_details": "/api/samples/{id}",
    "schedule_qc": "/api/samples/{id}/qc"
  }
}
```

### 4. Protocol List Endpoint
**`GET /api/protocols/list`**

Retrieves laboratory protocols and SOPs.

**Query Parameters:**
- `category`: Filter by category (extraction, qc, storage, etc.)
- `search`: Search by name or content
- `limit`: Number of results (default: 20)
- `offset`: Pagination offset

**Response:**
```json
{
  "protocols": [
    {
      "id": "SOP-001",
      "name": "DNA/RNA Extraction Protocol",
      "version": "2.3",
      "last_updated": "2024-01-15T00:00:00Z",
      "category": "extraction",
      "file_url": "/protocols/SOP-001-v2.3.pdf"
    }
  ],
  "total": 5,
  "limit": 20,
  "offset": 0,
  "categories": ["extraction", "library_prep", "qc", "storage", "sequencing"]
}
```

## Frontend Integration Details

### Chat Component Updates

1. **Real Streaming Implementation:**
```typescript
const response = await fetch('/api/chat/stream', {
  method: 'POST',
  body: formData,
});

const reader = response.body?.getReader();
const decoder = new TextDecoder();

while (true) {
  const { done, value } = await reader.read();
  if (done) break;
  
  // Process SSE chunks
  const text = decoder.decode(value);
  // Parse and update UI
}
```

2. **Action Handler Integration:**
```typescript
const handleActionClick = async (action) => {
  switch (action.action) {
    case 'create_sample':
      window.location.href = '/samples/new';
      break;
    case 'generate_labels':
      await fetch('/api/samples/labels', {...});
      break;
    // ... other actions
  }
};
```

## Architecture Flow

```
┌─────────────┐     ┌──────────────┐     ┌─────────────────┐
│   ChatBot   │────▶│ API Gateway  │────▶│ Backend Services│
│  (React UI) │◀────│ (FastAPI)    │◀────│ (Rust/Python)   │
└─────────────┘     └──────────────┘     └─────────────────┘
       │                    │                      │
       │   SSE Stream      │   Route to          │
       │   FormData        │   Services          │
       ▼                   ▼                      ▼
  [User Input]      [Auth/Proxy]          [RAG/Samples/Storage]
```

## Testing the Integration

### 1. Test Chat Streaming:
```bash
curl -X POST http://localhost:8089/api/chat/stream \
  -F "message=How do I create a DNA sample?" \
  -F "conversationId=test-123"
```

### 2. Test Document Processing:
```bash
curl -X POST http://localhost:8089/api/documents/process \
  -F "file=@sample_submission.pdf"
```

### 3. Test Sample Creation:
```bash
curl -X POST http://localhost:8089/api/samples/create \
  -H "Content-Type: application/json" \
  -d '{
    "sample_type": "DNA",
    "volume": 50,
    "concentration": 125,
    "buffer": "TE",
    "storage_temperature": -20,
    "storage_location": {"freezer": "A", "shelf": "1", "box": "B2"}
  }'
```

### 4. Test Protocol Listing:
```bash
curl http://localhost:8089/api/protocols/list?category=extraction
```

## Future Enhancements

### 1. Real RAG Integration
Currently using mock responses. Next steps:
- Connect to actual RAG service
- Implement vector search
- Add context retrieval

### 2. Enhanced Document Processing
- Integrate with OCR services
- Add support for more file formats
- Implement validation rules

### 3. Advanced Features
- Multi-turn conversation context
- User authentication integration
- Audit logging
- Rate limiting
- WebSocket support for bi-directional communication

## Configuration

### Environment Variables
```bash
# API Gateway
RAG_SERVICE_URL=http://rag-service:8000
SAMPLE_SERVICE_URL=http://sample-service:8080
STORAGE_SERVICE_URL=http://storage-service:8080

# Frontend
VITE_API_URL=http://localhost:8089
```

### Docker Services
All services are configured in `docker-compose.yml`:
- API Gateway: Port 8089
- Frontend: Port 3000
- Backend services: Various ports

## Troubleshooting

### Common Issues

1. **CORS Errors**
   - Ensure API Gateway has proper CORS configuration
   - Check allowed origins include frontend URL

2. **Streaming Not Working**
   - Verify nginx proxy doesn't buffer responses
   - Check `X-Accel-Buffering: no` header is set

3. **File Upload Failures**
   - Verify file size limits in nginx
   - Check content-type validation

### Debug Commands

```bash
# Check API Gateway logs
docker logs lims-gateway -f

# Test endpoint directly
docker exec -it lims-gateway curl http://localhost:8000/api/chat/health

# Check service connectivity
docker-compose ps
```

## Security Considerations

1. **Authentication**: Currently using mock auth. Implement JWT validation.
2. **File Validation**: Add virus scanning for uploads.
3. **Rate Limiting**: Implement per-user rate limits.
4. **Input Sanitization**: Validate all user inputs.
5. **HTTPS**: Use SSL/TLS in production.

---

*Backend integration completed. The ChatBot now has full API connectivity for laboratory management operations.* 