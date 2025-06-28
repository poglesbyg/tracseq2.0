#!/usr/bin/env python3
"""
Simple Web Interface for Laboratory Submission RAG System

A lightweight FastAPI web interface for the Docker deployment.
"""

import json
import logging
from pathlib import Path
from typing import Any

import uvicorn
from fastapi import FastAPI, File, HTTPException, UploadFile
from fastapi.responses import HTMLResponse, JSONResponse
from pydantic import BaseModel

# Import our RAG system
from simple_lab_rag import LightweightLabRAG

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Initialize FastAPI app
app = FastAPI(
    title="Laboratory Submission RAG System",
    description="Lightweight RAG system for processing laboratory submissions",
    version="1.0.0",
)

# Initialize RAG system
rag_system = None


# Models for API
class QueryRequest(BaseModel):
    question: str


class QueryResponse(BaseModel):
    answer: str
    confidence: float
    sources: list[str] = []


class ProcessingResponse(BaseModel):
    success: bool
    submission_id: str | None = None
    message: str
    extracted_data: dict[str, Any] | None = None


class HealthResponse(BaseModel):
    status: str
    version: str
    ollama_connected: bool
    documents_processed: int


# Initialize RAG system
async def initialize_rag():
    """Initialize the RAG system"""
    global rag_system
    try:
        rag_system = LightweightLabRAG()
        await rag_system.initialize()
        logger.info("RAG system initialized successfully")
    except Exception as e:
        logger.error(f"Failed to initialize RAG system: {e}")
        # Don't fail completely, allow the system to work in demo mode
        rag_system = LightweightLabRAG()


# Startup event
@app.on_event("startup")
async def startup_event():
    await initialize_rag()


# Health check endpoint
@app.get("/health", response_model=HealthResponse)
async def health_check():
    """Health check endpoint"""
    global rag_system

    if not rag_system:
        raise HTTPException(status_code=503, detail="RAG system not initialized")

    try:
        # Check Ollama connection
        ollama_connected = await rag_system._check_ollama_connection()

        # Get document count
        docs_count = len(rag_system.submissions)

        return HealthResponse(
            status="healthy",
            version="1.0.0",
            ollama_connected=ollama_connected,
            documents_processed=docs_count,
        )
    except Exception as e:
        logger.error(f"Health check failed: {e}")
        return HealthResponse(
            status="degraded", version="1.0.0", ollama_connected=False, documents_processed=0
        )


# Main web interface
@app.get("/", response_class=HTMLResponse)
async def main_page():
    """Serve the main web interface"""
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Laboratory Submission RAG System</title>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <style>
            body {
                font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                margin: 0;
                padding: 20px;
                background-color: #f5f5f5;
                color: #333;
            }
            .container {
                max-width: 1200px;
                margin: 0 auto;
                background: white;
                border-radius: 10px;
                box-shadow: 0 4px 6px rgba(0,0,0,0.1);
                overflow: hidden;
            }
            .header {
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                color: white;
                padding: 30px;
                text-align: center;
            }
            .header h1 {
                margin: 0 0 10px 0;
                font-size: 2.5em;
            }
            .header p {
                margin: 0;
                opacity: 0.9;
                font-size: 1.1em;
            }
            .content {
                padding: 30px;
            }
            .section {
                margin-bottom: 40px;
                padding: 20px;
                border: 1px solid #e0e0e0;
                border-radius: 8px;
                background: #fafafa;
            }
            .section h2 {
                margin-top: 0;
                color: #667eea;
                border-bottom: 2px solid #667eea;
                padding-bottom: 10px;
            }
            .form-group {
                margin-bottom: 20px;
            }
            label {
                display: block;
                margin-bottom: 5px;
                font-weight: bold;
                color: #555;
            }
            input[type="file"], textarea, input[type="text"] {
                width: 100%;
                padding: 12px;
                border: 2px solid #ddd;
                border-radius: 5px;
                font-size: 16px;
                transition: border-color 0.3s;
            }
            input[type="file"]:focus, textarea:focus, input[type="text"]:focus {
                outline: none;
                border-color: #667eea;
            }
            textarea {
                height: 120px;
                resize: vertical;
            }
            button {
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                color: white;
                padding: 12px 30px;
                border: none;
                border-radius: 5px;
                cursor: pointer;
                font-size: 16px;
                font-weight: bold;
                transition: transform 0.2s;
            }
            button:hover {
                transform: translateY(-2px);
                box-shadow: 0 4px 8px rgba(0,0,0,0.2);
            }
            .result {
                margin-top: 20px;
                padding: 15px;
                border-radius: 5px;
                border: 1px solid #ddd;
            }
            .result.success {
                background: #d4edda;
                border-color: #c3e6cb;
                color: #155724;
            }
            .result.error {
                background: #f8d7da;
                border-color: #f5c6cb;
                color: #721c24;
            }
            .loading {
                display: none;
                text-align: center;
                padding: 20px;
            }
            .spinner {
                border: 4px solid #f3f3f3;
                border-top: 4px solid #667eea;
                border-radius: 50%;
                width: 40px;
                height: 40px;
                animation: spin 1s linear infinite;
                margin: 0 auto 10px;
            }
            @keyframes spin {
                0% { transform: rotate(0deg); }
                100% { transform: rotate(360deg); }
            }
            .stats {
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                gap: 20px;
                margin: 20px 0;
            }
            .stat-card {
                background: white;
                padding: 20px;
                border-radius: 8px;
                border: 1px solid #e0e0e0;
                text-align: center;
            }
            .stat-value {
                font-size: 2em;
                font-weight: bold;
                color: #667eea;
            }
            .stat-label {
                color: #666;
                margin-top: 5px;
            }
        </style>
    </head>
    <body>
        <div class="container">
            <div class="header">
                <h1>🧬 Laboratory Submission RAG</h1>
                <p>Intelligent document processing for laboratory submissions</p>
            </div>
            
            <div class="content">
                <!-- System Status -->
                <div class="section">
                    <h2>📊 System Status</h2>
                    <div class="stats" id="stats">
                        <div class="stat-card">
                            <div class="stat-value" id="status">Loading...</div>
                            <div class="stat-label">System Status</div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-value" id="ollama-status">Loading...</div>
                            <div class="stat-label">Ollama Connection</div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-value" id="doc-count">0</div>
                            <div class="stat-label">Documents Processed</div>
                        </div>
                    </div>
                </div>

                <!-- Document Upload -->
                <div class="section">
                    <h2>📄 Upload Laboratory Document</h2>
                    <form id="uploadForm" enctype="multipart/form-data">
                        <div class="form-group">
                            <label for="file">Select Document (PDF, DOCX, or TXT):</label>
                            <input type="file" id="file" name="file" accept=".pdf,.docx,.txt,.doc" required>
                        </div>
                        <button type="submit">Process Document</button>
                    </form>
                    <div class="loading" id="uploadLoading">
                        <div class="spinner"></div>
                        <p>Processing document...</p>
                    </div>
                    <div id="uploadResult"></div>
                </div>

                <!-- Query Interface -->
                <div class="section">
                    <h2>❓ Ask Questions</h2>
                    <form id="queryForm">
                        <div class="form-group">
                            <label for="question">Ask about your laboratory submissions:</label>
                            <textarea id="question" name="question" placeholder="e.g., What samples were submitted by Dr. Smith? What sequencing methods were requested?" required></textarea>
                        </div>
                        <button type="submit">Ask Question</button>
                    </form>
                    <div class="loading" id="queryLoading">
                        <div class="spinner"></div>
                        <p>Searching for answer...</p>
                    </div>
                    <div id="queryResult"></div>
                </div>
            </div>
        </div>

        <script>
            // Load system status
            async function loadStatus() {
                try {
                    const response = await fetch('/health');
                    const data = await response.json();
                    
                    document.getElementById('status').textContent = data.status === 'healthy' ? '✅' : '⚠️';
                    document.getElementById('ollama-status').textContent = data.ollama_connected ? '✅' : '❌';
                    document.getElementById('doc-count').textContent = data.documents_processed;
                } catch (error) {
                    document.getElementById('status').textContent = '❌';
                    document.getElementById('ollama-status').textContent = '❌';
                }
            }

            // Upload form handler
            document.getElementById('uploadForm').addEventListener('submit', async function(e) {
                e.preventDefault();
                
                const formData = new FormData();
                const fileInput = document.getElementById('file');
                
                if (!fileInput.files[0]) {
                    alert('Please select a file');
                    return;
                }
                
                formData.append('file', fileInput.files[0]);
                
                const loading = document.getElementById('uploadLoading');
                const result = document.getElementById('uploadResult');
                
                loading.style.display = 'block';
                result.innerHTML = '';
                
                try {
                    const response = await fetch('/upload', {
                        method: 'POST',
                        body: formData
                    });
                    
                    const data = await response.json();
                    
                    if (data.success) {
                        result.innerHTML = `
                            <div class="result success">
                                <h3>✅ Document Processed Successfully!</h3>
                                <p><strong>Submission ID:</strong> ${data.submission_id}</p>
                                <p>${data.message}</p>
                                ${data.extracted_data ? `<pre>${JSON.stringify(data.extracted_data, null, 2)}</pre>` : ''}
                            </div>
                        `;
                        loadStatus(); // Refresh status
                    } else {
                        result.innerHTML = `
                            <div class="result error">
                                <h3>❌ Processing Failed</h3>
                                <p>${data.message}</p>
                            </div>
                        `;
                    }
                } catch (error) {
                    result.innerHTML = `
                        <div class="result error">
                            <h3>❌ Error</h3>
                            <p>Failed to process document: ${error.message}</p>
                        </div>
                    `;
                } finally {
                    loading.style.display = 'none';
                }
            });

            // Query form handler
            document.getElementById('queryForm').addEventListener('submit', async function(e) {
                e.preventDefault();
                
                const question = document.getElementById('question').value.trim();
                
                if (!question) {
                    alert('Please enter a question');
                    return;
                }
                
                const loading = document.getElementById('queryLoading');
                const result = document.getElementById('queryResult');
                
                loading.style.display = 'block';
                result.innerHTML = '';
                
                try {
                    const response = await fetch('/query', {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json',
                        },
                        body: JSON.stringify({ question: question })
                    });
                    
                    const data = await response.json();
                    
                    result.innerHTML = `
                        <div class="result success">
                            <h3>💡 Answer</h3>
                            <p>${data.answer}</p>
                            ${data.confidence ? `<p><strong>Confidence:</strong> ${(data.confidence * 100).toFixed(1)}%</p>` : ''}
                            ${data.sources && data.sources.length > 0 ? `
                                <p><strong>Sources:</strong></p>
                                <ul>
                                    ${data.sources.map(source => `<li>${source}</li>`).join('')}
                                </ul>
                            ` : ''}
                        </div>
                    `;
                } catch (error) {
                    result.innerHTML = `
                        <div class="result error">
                            <h3>❌ Error</h3>
                            <p>Failed to get answer: ${error.message}</p>
                        </div>
                    `;
                } finally {
                    loading.style.display = 'none';
                }
            });

            // Load status on page load
            loadStatus();
            
            // Refresh status every 30 seconds
            setInterval(loadStatus, 30000);
        </script>
    </body>
    </html>
    """
    return HTMLResponse(content=html_content)


# Upload endpoint
@app.post("/upload", response_model=ProcessingResponse)
async def upload_document(file: UploadFile = File(...)):
    """Upload and process a laboratory document"""
    global rag_system

    if not rag_system:
        raise HTTPException(status_code=503, detail="RAG system not initialized")

    try:
        # Save uploaded file temporarily
        upload_dir = Path("/app/uploads")
        upload_dir.mkdir(exist_ok=True)

        file_path = upload_dir / file.filename
        content = await file.read()

        with open(file_path, "wb") as f:
            f.write(content)

        # Process the document
        result = await rag_system.process_document(str(file_path))

        # Clean up temporary file
        file_path.unlink(exist_ok=True)

        return ProcessingResponse(
            success=result.success,
            submission_id=result.submission_id,
            message=f"Document processed successfully! Confidence: {result.confidence_score:.2f}",
            extracted_data=result.extracted_data,
        )

    except Exception as e:
        logger.error(f"Upload processing failed: {e}")
        return ProcessingResponse(success=False, message=f"Failed to process document: {str(e)}")


# Query endpoint
@app.post("/query", response_model=QueryResponse)
async def query_submissions(request: QueryRequest):
    """Query the RAG system"""
    global rag_system

    if not rag_system:
        raise HTTPException(status_code=503, detail="RAG system not initialized")

    try:
        answer = await rag_system.query(request.question)

        return QueryResponse(
            answer=answer,
            confidence=0.8,  # Default confidence
            sources=[],  # Could be enhanced to include sources
        )

    except Exception as e:
        logger.error(f"Query failed: {e}")
        raise HTTPException(status_code=500, detail=f"Query failed: {str(e)}")


# Export endpoint
@app.get("/export")
async def export_data():
    """Export all processed submissions"""
    global rag_system

    if not rag_system:
        raise HTTPException(status_code=503, detail="RAG system not initialized")

    try:
        # Export to JSON
        export_path = await rag_system.export_submissions()

        # Read and return the exported data
        with open(export_path, encoding="utf-8") as f:
            data = json.load(f)

        return JSONResponse(content=data)

    except Exception as e:
        logger.error(f"Export failed: {e}")
        raise HTTPException(status_code=500, detail=f"Export failed: {str(e)}")


if __name__ == "__main__":
    # Run the web server
    uvicorn.run(app, host="0.0.0.0", port=8000, log_level="info")
