# TracSeq ChatBot Real-Time Features and Integrations

## Overview

The TracSeq ChatBot has been enhanced with real-time capabilities, authentication, and direct integration with TracSeq services. This document outlines the new features and their implementation.

## Key Features Implemented

### 1. Real Service Connections

#### RAG Service Integration
- **Endpoint**: `/api/chat/stream`
- **Features**:
  - Direct connection to RAG service for AI-powered responses
  - Fallback mechanism when RAG service is unavailable
  - Conversation history persistence in PostgreSQL
  - Source citation and confidence scoring

#### Sample Service Integration
- **Endpoint**: `/api/samples/create`
- **Features**:
  - Direct sample creation from chat interface
  - Real-time validation and barcode generation
  - Automatic storage allocation
  - Integration with lab_manager service

#### Document Processing
- **Endpoint**: `/api/documents/process`
- **Features**:
  - PDF/Excel/CSV file upload support
  - Automatic data extraction
  - Laboratory submission form processing
  - Quality metric validation

### 2. Authentication System

#### JWT Token Implementation
- **Module**: `middleware/auth.py`
- **Features**:
  - JWT token generation and validation
  - User context in all chat interactions
  - Session-based conversation tracking
  - Role-based access control ready

#### Protected Endpoints
- `/api/auth/me` - Get current user info
- `/api/users/me` - Alternative user endpoint
- All chat endpoints include user context

### 3. WebSocket Support

#### Real-Time Communication
- **Endpoint**: `/ws/chat/{conversation_id}`
- **Features**:
  - Bi-directional real-time messaging
  - Typing indicators
  - User presence tracking
  - Multi-user conversation support

#### Connection Management
- Automatic reconnection handling
- Connection status indicators
- Message queuing for offline mode

### 4. Database Integration

#### Chat Tables
```sql
-- Chat messages table
chat_messages (
  id UUID PRIMARY KEY,
  conversation_id VARCHAR(255),
  role VARCHAR(50),
  content TEXT,
  metadata JSONB,
  created_at TIMESTAMPTZ,
  updated_at TIMESTAMPTZ
)

-- Conversations table  
chat_conversations (
  id VARCHAR(255) PRIMARY KEY,
  user_id VARCHAR(255),
  title VARCHAR(500),
  metadata JSONB,
  created_at TIMESTAMPTZ,
  updated_at TIMESTAMPTZ,
  last_message_at TIMESTAMPTZ
)
```

## API Examples

### 1. Streaming Chat Response
```bash
curl -X POST http://localhost:8089/api/chat/stream \
  -H "Authorization: Bearer <token>" \
  -F "message=Create a DNA sample" \
  -F "conversationId=conv_123"
```

Response (Server-Sent Events):
```
data: {"id":"msg_1","content":"I'll help","type":"chunk","timestamp":"2024-01-15T10:00:00Z"}
data: {"id":"msg_1","content":" you create","type":"chunk","timestamp":"2024-01-15T10:00:01Z"}
data: {"id":"msg_1","type":"completion","metadata":{"confidence":0.95,"sources":["protocol_001"]}}
data: [DONE]
```

### 2. WebSocket Connection
```javascript
const ws = new WebSocket('ws://localhost:8089/ws/chat/conv_123?token=<jwt_token>');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  switch(data.type) {
    case 'connection':
      console.log('Connected:', data.user);
      break;
    case 'message':
      console.log('New message:', data.content);
      break;
    case 'typing':
      console.log('User typing:', data.user.name);
      break;
  }
};

// Send message
ws.send(JSON.stringify({
  type: 'message',
  content: 'Hello, TracSeq!'
}));
```

### 3. Sample Creation from Chat
```bash
curl -X POST http://localhost:8089/api/samples/create \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "sample_type": "DNA",
    "volume": 50,
    "concentration": 125,
    "buffer": "TE",
    "storage_temperature": -20,
    "storage_location": {
      "freezer": "A1",
      "rack": "B2",
      "box": "C3"
    },
    "project_id": "PROJ-001",
    "principal_investigator": "Dr. Smith"
  }'
```

### 4. Document Processing
```bash
curl -X POST http://localhost:8089/api/documents/process \
  -H "Authorization: Bearer <token>" \
  -F "file=@submission_form.pdf" \
  -F "metadata={\"project\":\"PROJ-001\"}"
```

## Frontend Integration

### WebSocket in React
```typescript
// Initialize WebSocket connection
useEffect(() => {
  const token = localStorage.getItem('authToken');
  const ws = new WebSocket(`ws://localhost:8089/ws/chat/${conversationId}?token=${token}`);
  
  ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    // Handle different message types
  };
  
  return () => ws.close();
}, [conversationId]);
```

### Authentication Flow
```typescript
// Login and store token
const response = await fetch('/api/auth/login', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ email, password })
});

const { token, user } = await response.json();
localStorage.setItem('authToken', token);
```

## Security Considerations

1. **JWT Token Security**:
   - Tokens expire after 24 hours
   - Include user roles and permissions
   - Validated on every request

2. **WebSocket Security**:
   - Token-based authentication via query params
   - Connection-level authorization
   - Message validation and sanitization

3. **Data Protection**:
   - All chat history encrypted at rest
   - User isolation for conversations
   - Audit logging for sensitive operations

## Performance Optimizations

1. **Streaming Responses**:
   - Server-Sent Events for real-time streaming
   - Chunked responses for better UX
   - Client-side buffering

2. **WebSocket Efficiency**:
   - Message batching
   - Compression support
   - Automatic reconnection with exponential backoff

3. **Database Optimization**:
   - Indexed conversation lookups
   - Efficient message pagination
   - Connection pooling

## Future Enhancements

1. **Enhanced Collaboration**:
   - Screen sharing for lab procedures
   - Voice/video chat integration
   - Collaborative sample editing

2. **Advanced AI Features**:
   - Multi-modal input (images, charts)
   - Predictive sample suggestions
   - Automated workflow generation

3. **Integration Expansion**:
   - Direct LIMS integration
   - Equipment status monitoring
   - Automated report generation

## Deployment Considerations

1. **Environment Variables**:
   ```bash
   JWT_SECRET=<secure-random-key>
   DATABASE_URL=postgres://user:pass@host:5432/tracseq
   RAG_SERVICE_URL=http://rag-service:8000
   REDIS_URL=redis://redis:6379
   ```

2. **Docker Configuration**:
   - WebSocket support in nginx/reverse proxy
   - Proper CORS headers for SSE
   - Session affinity for WebSocket connections

3. **Monitoring**:
   - WebSocket connection metrics
   - Chat response time tracking
   - Error rate monitoring

## Testing

### Unit Tests
```python
# Test JWT authentication
def test_jwt_authentication():
    token = create_token({"id": "1", "email": "test@example.com"})
    user = decode_token(token)
    assert user["email"] == "test@example.com"

# Test WebSocket connection
async def test_websocket_chat():
    async with websockets.connect("ws://localhost:8089/ws/chat/test") as ws:
        await ws.send(json.dumps({"type": "message", "content": "Test"}))
        response = await ws.recv()
        assert json.loads(response)["type"] == "message"
```

### Integration Tests
- Test RAG service fallback
- Test sample creation flow
- Test document processing pipeline
- Test real-time message delivery

## Troubleshooting

### Common Issues

1. **WebSocket Connection Failed**:
   - Check CORS configuration
   - Verify token is valid
   - Ensure WebSocket upgrade headers

2. **RAG Service Timeout**:
   - Check service health endpoint
   - Verify network connectivity
   - Review resource limits

3. **Authentication Errors**:
   - Verify JWT secret matches
   - Check token expiration
   - Validate user permissions

### Debug Endpoints
- `/api/health` - Overall system health
- `/api/chat/health` - Chat service status
- `/api/debug/routes` - Available routes
- `/ws/debug` - WebSocket test endpoint

*Context improved by Giga AI* 