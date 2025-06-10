# RAG Chatbot Test Checklist ✅

Use this checklist to verify that the RAG chatbot is working correctly.

## Pre-requisites Check

### System Requirements
- [ ] Lab Manager frontend is running (`http://localhost:5173` or `http://localhost`)
- [ ] Lab Manager backend is running (`http://localhost:3000`)
- [ ] RAG system is running (`http://localhost:8000`)
- [ ] No console errors in browser developer tools

### RAG System Deployment Check

#### Docker Deployment (Recommended)
- [ ] Docker and Docker Compose are installed
- [ ] RAG service container is running: `docker-compose ps`
- [ ] RAG service logs show no errors: `docker-compose logs rag-service`
- [ ] Port 8000 is accessible: `curl http://localhost:8000/health`

#### Python Deployment (Alternative)
- [ ] Python virtual environment is activated
- [ ] RAG service is running with uvicorn
- [ ] Required Python dependencies are installed

## Basic Functionality Tests

### 1. Chatbot Visibility
- [ ] Floating blue chat button appears in bottom-right corner
- [ ] Button has subtle pulse animation
- [ ] Hover shows tooltip "Ask Lab Assistant"

### 2. Chatbot Interface
- [ ] Clicking button opens chat window
- [ ] Chat window shows welcome message
- [ ] Suggested questions are displayed
- [ ] Input field is focused and ready for typing

### 3. Basic Interaction
- [ ] Can type a message in input field
- [ ] Pressing Enter sends the message
- [ ] Clicking send button works
- [ ] User message appears in chat with correct styling

### 4. AI Response
- [ ] Typing indicator appears after sending message
- [ ] AI response appears after processing
- [ ] Response has appropriate bot styling
- [ ] Conversation flows naturally

### 5. Window Controls
- [ ] Minimize button works (collapses to header only)
- [ ] Maximize button works (expands from minimized state)
- [ ] Close button works (hides entire chatbot)
- [ ] Can reopen chatbot after closing

## Test Questions

Try these questions to verify enhanced AI functionality:

### Enhanced Sample Management
```
How do I submit a new sample using the AI document processing feature?
```
Expected: Detailed workflow with specific steps, best practices, and tips

### Advanced Storage Requirements
```
What are the optimal storage conditions for different sample types and why?
```
Expected: Comprehensive temperature guidelines, scientific rationale, and Lab Manager integration

### Complex Workflow Help
```
I need to set up a sequencing job for 50 RNA samples. Walk me through the entire process.
```
Expected: Step-by-step workflow, quality requirements, sample sheet generation, and troubleshooting

### Contextual Intelligence Test
```
First ask: "What's the difference between batch and individual sample submission?"
Then ask: "Which method would you recommend for my use case?"
```
Expected: Detailed comparison, then context-aware recommendation based on previous question

### Multi-Step Problem Solving
```
I have a PDF form with sample information, but some data is missing. How should I proceed?
```
Expected: Comprehensive approach including RAG processing, manual completion, validation steps

### System Integration Knowledge
```
How do I export my sequencing results and integrate them with external analysis tools?
```
Expected: Export procedures, file formats, integration options, and best practices

## Error Handling Tests

### 1. Network Issues
- [ ] Disconnect from internet or stop RAG system
- [ ] Send a message
- [ ] Should show appropriate error message
- [ ] Error doesn't break the interface

### 2. Invalid Responses
- [ ] AI should handle unclear questions gracefully
- [ ] No crashes or blank responses
- [ ] Fallback messages work appropriately

## Performance Tests

- [ ] Chat opens/closes smoothly (< 300ms)
- [ ] Messages send without noticeable delay
- [ ] AI responses arrive in reasonable time (< 10s)
- [ ] No memory leaks during extended use

## Mobile/Responsive Tests

- [ ] Chatbot works on mobile devices
- [ ] Touch interactions work properly
- [ ] Chat window scales appropriately
- [ ] Text is readable on small screens

## Integration Tests

### 1. RAG System Integration
```bash
# Test RAG endpoint directly
curl -X POST http://localhost:3000/api/samples/rag/query \
  -H "Content-Type: application/json" \
  -d '{"query": "How do I submit a sample?"}'
```
- [ ] Returns valid JSON response
- [ ] Contains "answer" field
- [ ] Response is relevant to query

### 2. Health Check
```bash
curl http://localhost:3000/api/samples/rag/status
```
- [ ] Returns system status
- [ ] Shows operational status
- [ ] Includes RAG system information

### 3. Docker-Specific Tests

#### Container Health Checks
```bash
# Check container status
docker-compose ps
# Should show rag-service as "Up" and healthy

# Check internal health endpoint
docker-compose exec rag-service curl http://localhost:8000/health
# Should return {"status": "healthy"}

# Check logs for startup success
docker-compose logs rag-service | grep -i "started"
# Should show successful startup messages
```

#### Container Resource Usage
```bash
# Check container resource usage
docker stats --no-stream
# Verify reasonable CPU/memory usage

# Check disk usage of volumes
docker system df
# Verify volumes aren't consuming excessive space
```

- [ ] Container is running and healthy
- [ ] Internal health check passes
- [ ] Startup logs show success
- [ ] Resource usage is reasonable
- [ ] Volume storage is within limits

## Known Issues

Document any issues found during testing:

- [ ] Issue 1: ________________________
- [ ] Issue 2: ________________________
- [ ] Issue 3: ________________________

## Success Criteria

✅ **All basic functionality tests pass**
✅ **AI responses are relevant and helpful**
✅ **No console errors or crashes**
✅ **Mobile/desktop compatibility confirmed**
✅ **Performance is acceptable**

---

**Test Completed**: ___/___/___ by: ________________

**Overall Status**: 🟢 PASS / 🟡 PARTIAL / 🔴 FAIL

**Notes**:
_________________________________________________
_________________________________________________
_________________________________________________ 
