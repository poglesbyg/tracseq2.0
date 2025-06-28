import uvicorn
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

app = FastAPI()
app.add_middleware(CORSMiddleware, allow_origins=['*'], allow_methods=['*'], allow_headers=['*'])

@app.get('/health')
async def health():
    return {'status': 'healthy', 'service': 'minimal-rag'}

@app.get('/api/rag/submissions')
async def submissions():
    return []

@app.get('/api/rag/stats')
async def stats():
    return {'total_submissions': 0, 'status': 'operational'}

@app.post('/query')
async def query():
    return {'answer': 'Hello! This is a minimal RAG service for demonstration.'}

if __name__ == '__main__':
    print('Starting minimal RAG service on port 8087')
    uvicorn.run(app, host='0.0.0.0', port=8087)
