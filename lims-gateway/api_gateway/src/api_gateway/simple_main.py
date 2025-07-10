#!/usr/bin/env python3
"""
Simple API Gateway for TracSeq 2.0
Minimal working implementation for demonstration
"""

import json
import os
import sys
import time
import uuid
import asyncio
from datetime import datetime, timedelta
from typing import Any, Dict, Optional, List, AsyncGenerator

import httpx
import uvicorn
import logging
from fastapi import FastAPI, HTTPException, Depends, Request, Response, WebSocket, WebSocketDisconnect, BackgroundTasks, Form, File, UploadFile, Query
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse, StreamingResponse
from fastapi.exceptions import RequestValidationError
from contextlib import asynccontextmanager
from pydantic import BaseModel, Field

# Set up logger
logger = logging.getLogger(__name__)
import asyncpg
import jwt
import base64

# Import auth middleware
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
try:
    # Try relative import first (when running as module)
    from .middleware.auth import get_current_user, create_token
    from .routes.websocket_chat import websocket_chat_endpoint, manager
except ImportError:
    # Fall back to absolute import (when running directly)
    from middleware.auth import get_current_user, create_token
    from routes.websocket_chat import websocket_chat_endpoint, manager

# Import standardized database configuration
if os.getenv("DISABLE_STANDARDIZED_DB") == "true":
    # Force simple database configuration for testing
    STANDARDIZED_DB = False
    print("Using simple database configuration (testing mode)")
    DATABASE_URL = os.getenv("DATABASE_URL", "postgres://postgres:postgres@lims-postgres:5432/lims_db")
    db_pool = None

    # Initialize database pool on startup
    async def init_db():
        global db_pool
        try:
            db_pool = await asyncpg.create_pool(DATABASE_URL, min_size=2, max_size=10)
        except Exception as e:
            print(f"Database connection failed: {e}")
            db_pool = None

    # Close database pool on shutdown
    async def close_db():
        global db_pool
        if db_pool:
            await db_pool.close()
else:
    try:
        from api_gateway.core.database import init_database, close_database, get_db_health_status, get_db_connection, get_db_info
        STANDARDIZED_DB = True
        print("Using standardized database configuration")
    except ImportError:
        # Fallback to simple database configuration
        STANDARDIZED_DB = False
        print("Using simple database configuration fallback")
        DATABASE_URL = os.getenv("DATABASE_URL", "postgres://postgres:postgres@lims-postgres:5432/lims_db")
        db_pool = None

        # Initialize database pool on startup
        async def init_db():
            global db_pool
            try:
                db_pool = await asyncpg.create_pool(DATABASE_URL, min_size=2, max_size=10)
            except Exception as e:
                print(f"Database connection failed: {e}")
                db_pool = None

        # Close database pool on shutdown
        async def close_db():
            global db_pool
            if db_pool:
                await db_pool.close()

# Database helper functions
from contextlib import asynccontextmanager

@asynccontextmanager
async def get_database_connection():
    """Get a database connection using the appropriate method."""
    if STANDARDIZED_DB:
        async with get_db_connection() as conn:
            yield conn
    else:
        global db_pool
        if db_pool:
            async with db_pool.acquire() as conn:
                yield conn
        else:
            raise RuntimeError("Database pool not initialized")

async def check_database_health():
    """Check database health using the appropriate method."""
    if STANDARDIZED_DB:
        status = get_db_health_status()
        return status.get("healthy", False)
    else:
        global db_pool
        if db_pool:
            try:
                async with db_pool.acquire() as conn:
                    await conn.execute("SELECT 1")
                return True
            except Exception:
                return False
        return False

async def get_database_info():
    """Get database information using the appropriate method."""
    if STANDARDIZED_DB:
        return await get_db_info()
    else:
        global db_pool
        if db_pool:
            try:
                async with db_pool.acquire() as conn:
                    result = await conn.fetchrow("SELECT current_database(), current_user, version()")
                    return {
                        "database_name": result[0],
                        "current_user": result[1],
                        "version": result[2],
                        "fallback_mode": True
                    }
            except Exception as e:
                return {"error": str(e), "fallback_mode": True}
        return {"error": "Database pool not initialized", "fallback_mode": True}

# Initialize FastAPI app
app = FastAPI(
    title="TracSeq 2.0 API Gateway",
    description="Central routing hub for TracSeq microservices",
    version="2.0.0"
)

# Startup and shutdown events
@app.on_event("startup")
async def startup_event():
    if STANDARDIZED_DB:
        await init_database()
    else:
        await init_db()

@app.on_event("shutdown")
async def shutdown_event():
    if STANDARDIZED_DB:
        await close_database()
    else:
        await close_db()

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # In production, specify exact origins
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Add exception handler for validation errors
@app.exception_handler(RequestValidationError)
async def validation_exception_handler(request: Request, exc: RequestValidationError):
    """Log validation errors to understand 400 errors"""
    print(f"Validation error: {exc.errors()}")
    print(f"Request URL: {request.url}")
    print(f"Request method: {request.method}")
    return JSONResponse(
        status_code=400,
        content={"detail": exc.errors(), "body": exc.body if hasattr(exc, 'body') else None}
    )

# Add a general exception handler
@app.exception_handler(Exception)
async def general_exception_handler(request: Request, exc: Exception):
    """Log all exceptions"""
    print(f"General exception: {exc}")
    print(f"Request URL: {request.url}")
    print(f"Request method: {request.method}")
    import traceback
    traceback.print_exc()
    return JSONResponse(
        status_code=500,
        content={"detail": str(exc)}
    )

# Service URLs from environment
GATEWAY_HOST = os.getenv("GATEWAY_HOST", "0.0.0.0")
GATEWAY_PORT = int(os.getenv("GATEWAY_PORT", "8000"))
GATEWAY_DEBUG = os.getenv("GATEWAY_DEBUG", "false").lower() == "true"

# Service discovery URLs
AUTH_SERVICE_URL = os.getenv("AUTH_SERVICE_URL", "http://auth-service:8080")
SAMPLE_SERVICE_URL = os.getenv("SAMPLE_SERVICE_URL", "http://sample-service:8081")
STORAGE_SERVICE_URL = os.getenv("STORAGE_SERVICE_URL", "http://storage-service:8082")
TEMPLATE_SERVICE_URL = os.getenv("TEMPLATE_SERVICE_URL", "http://template-service:8083")
SEQUENCING_SERVICE_URL = os.getenv("SEQUENCING_SERVICE_URL", "http://tracseq-sequencing:8084")
NOTIFICATION_SERVICE_URL = os.getenv("NOTIFICATION_SERVICE_URL", "http://tracseq-notification:8085")
RAG_SERVICE_URL = os.getenv("RAG_SERVICE_URL", "http://tracseq-rag:8000")
EVENT_SERVICE_URL = os.getenv("EVENT_SERVICE_URL", "http://tracseq-events:8087")
TRANSACTION_SERVICE_URL = os.getenv("TRANSACTION_SERVICE_URL", "http://tracseq-transactions:8088")
COGNITIVE_ASSISTANT_URL = os.getenv("COGNITIVE_ASSISTANT_URL", "http://cognitive-assistant:8000")
REPORTS_SERVICE_URL = os.getenv("REPORTS_SERVICE_URL", "http://reports-service:8000")

# Lab manager is deployed as sequencing service
LAB_MANAGER_URL = os.getenv("LAB_MANAGER_URL", SEQUENCING_SERVICE_URL)
PROJECT_SERVICE_URL = os.getenv("PROJECT_SERVICE_URL", LAB_MANAGER_URL)
LIBRARY_PREP_SERVICE_URL = os.getenv("LIBRARY_PREP_SERVICE_URL", LAB_MANAGER_URL)
QAQC_SERVICE_URL = os.getenv("QAQC_SERVICE_URL", LAB_MANAGER_URL)
FLOW_CELL_SERVICE_URL = os.getenv("FLOW_CELL_SERVICE_URL", LAB_MANAGER_URL)

# Pydantic models for API requests
class LoginRequest(BaseModel):
    email: str
    password: str

class LoginResponse(BaseModel):
    token: str
    user: Dict[str, Any]

# Mock data for demonstration
MOCK_USERS = {
    "admin@tracseq.com": {
        "id": "1",
        "email": "admin@tracseq.com",
        "name": "Admin User",
        "role": "admin",
        "password": "admin123"  # In production, this would be hashed
    },
    "user@tracseq.com": {
        "id": "2",
        "email": "user@tracseq.com",
        "name": "Lab User",
        "role": "user",
        "password": "user123"
    }
}

# In-memory storage for uploaded datasets (in production, this would be in a database)
uploaded_datasets = {}

@app.get("/")
async def root():
    """Root endpoint"""
    return {
        "service": "TracSeq 2.0 API Gateway",
        "status": "running",
        "version": "2.0.0",
        "description": "Central routing hub for TracSeq microservices"
    }

@app.get("/health")
async def health():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "service": "api-gateway",
        "timestamp": datetime.now().isoformat()
    }

@app.get("/api/health")
async def api_health():
    """API health check endpoint for frontend"""
    # Check database health
    db_healthy = await check_database_health()
    
    # Get detailed database information
    db_info = await get_database_info()
    
    return {
        "status": "healthy" if db_healthy else "degraded",
        "service": "api-gateway",
        "timestamp": datetime.now().isoformat(),
        "database": {
            "healthy": db_healthy,
            "type": "standardized" if STANDARDIZED_DB else "simple",
            "info": db_info
        },
        "features": {
            "standardized_db": STANDARDIZED_DB,
            "enhanced_monitoring": STANDARDIZED_DB,
        }
    }



# TEST ENDPOINT - Simple storage samples endpoint
@app.get("/api/test/samples")
async def get_test_samples_simple():
    """Simple storage samples endpoint - TEST"""
    return {
        "data": [
            {
                "id": "SMPL-001",
                "barcode": "BC001",
                "sample_type": "DNA",
                "location_name": "Freezer A1 (-80°C)",
                "status": "stored",
                "stored_at": "2024-01-15T10:30:00Z"
            },
            {
                "id": "SMPL-002", 
                "barcode": "BC002",
                "sample_type": "RNA",
                "location_name": "Refrigerator B2 (4°C)",
                "status": "stored",
                "stored_at": "2024-01-16T14:20:00Z"
            }
        ],
        "total": 2,
        "page": 1,
        "per_page": 20
    }

# REMOVED: Duplicate storage utilization endpoint - using comprehensive version later in file

# REMOVED: Duplicate mobile samples endpoint - using comprehensive version later in file

# REMOVED: Duplicate store sample endpoint - using comprehensive version later in file

# ============================================================================
# Storage Management Endpoints (Early Definition) - REMOVED DUE TO CONFLICTS
# ============================================================================
# NOTE: Storage endpoints moved to after storage locations section to avoid conflicts

# Service-specific health endpoints
@app.get("/api/templates/health")
async def templates_health():
    """Templates service health check"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{TEMPLATE_SERVICE_URL}/health", timeout=5.0)
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Templates service unavailable: {str(e)}")

@app.get("/api/notifications/health")
async def notifications_health():
    """Notifications service health check"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{NOTIFICATION_SERVICE_URL}/health", timeout=5.0)
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Notifications service unavailable: {str(e)}")

@app.get("/api/sequencing/health")
async def sequencing_health():
    """Sequencing service health check"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{SEQUENCING_SERVICE_URL}/health", timeout=5.0)
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Sequencing service unavailable: {str(e)}")

@app.get("/api/qaqc/health")
async def qaqc_health():
    """QA/QC service health check"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{QAQC_SERVICE_URL}/health", timeout=5.0)
            response.raise_for_status()
            # Handle plain text response
            content_type = response.headers.get("content-type", "")
            if "text/" in content_type:
                return {"status": "healthy", "message": response.text.strip()}
            return response.json()
    except httpx.HTTPStatusError as e:
        raise HTTPException(status_code=e.response.status_code, detail=str(e))
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"QA/QC service unavailable: {str(e)}")

@app.get("/api/transactions/health")
async def transactions_health():
    """Transaction service health check"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{TRANSACTION_SERVICE_URL}/health", timeout=5.0)
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Transaction service unavailable: {str(e)}")

@app.get("/api/events/health")
async def events_health():
    """Event service health check"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{EVENT_SERVICE_URL}/health", timeout=5.0)
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Event service unavailable: {str(e)}")

# Authentication endpoints
@app.post("/api/auth/login")
async def login(request: Request):
    """User login endpoint - flexible payload handling"""
    try:
        # Try to get JSON body
        body = await request.json()

        # Handle different payload formats
        email = body.get("email") or body.get("username")
        password = body.get("password")

        if not email or not password:
            raise HTTPException(status_code=400, detail="Email and password are required")

        user = MOCK_USERS.get(email)

        if not user or user["password"] != password:
            raise HTTPException(status_code=401, detail="Invalid credentials")

        # Generate JWT token
        token = create_token({
            "id": user["id"],
            "email": user["email"],
            "name": user["name"],
            "role": user["role"]
        })

        return {
            "data": {
                "token": token,
                "user": {
                    "id": user["id"],
                    "email": user["email"],
                    "name": user["name"],
                    "role": user["role"]
                }
            }
        }

    except ValueError:
        raise HTTPException(status_code=400, detail="Invalid JSON payload")
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Login error: {e!s}")

@app.get("/api/auth/me")
async def get_current_user_endpoint(request: Request):
    """Get current user info"""
    user = await get_current_user(request)
    if not user:
        raise HTTPException(status_code=401, detail="Not authenticated")
    
    return {
        "id": user.id,
        "email": user.email,
        "name": user.name,
        "role": user.role
    }

# Proxy route for /api/users/me to auth service  
@app.get("/api/users/me")
async def proxy_users_me(request: Request):
    """Proxy /api/users/me to auth service /auth/me"""
    try:
        async with httpx.AsyncClient() as client:
            url = f"{AUTH_SERVICE_URL}/auth/me"
            
            response = await client.request(
                method="GET",
                url=url,
                headers=dict(request.headers),
                timeout=30.0
            )
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Auth service unavailable: {e!s}")

# =============================================================================
# Chat API Endpoints for TracSeq ChatBot Integration
# =============================================================================

@app.post("/api/chat/stream")
async def chat_stream(
    request: Request,
    message: str = Form(...),
    conversationId: str = Form(...),
    files: Optional[List[UploadFile]] = File(None)
):
    """
    Stream chat responses with support for file uploads.
    
    This endpoint handles:
    - Real-time streaming of AI responses
    - File upload processing
    - Context-aware responses based on conversation history
    """
    async def generate_stream() -> AsyncGenerator[str, None]:
        # Initial response
        response_id = str(uuid.uuid4())
        
        # Get authenticated user context
        user = await get_current_user(request)
        user_context = {
            "user_id": user.id if user else "anonymous",
            "user_email": user.email if user else "anonymous@tracseq.com",
            "user_name": user.name if user else "Anonymous User",
            "user_role": user.role if user else "guest"
        }
        
        try:
            # Process any uploaded files first
            file_context = ""
            file_data = []
            if files:
                for file in files:
                    content = await file.read()
                    file_context += f"\n[Processing file: {file.filename} ({len(content)} bytes)]"
                    file_data.append({
                        "filename": file.filename,
                        "content": content.decode('utf-8', errors='ignore') if file.content_type and 'text' in file.content_type else None,
                        "content_type": file.content_type,
                        "size": len(content)
                    })
            
            # Prepare context for RAG service
            async with httpx.AsyncClient() as client:
                # Get conversation history if exists
                history = []
                try:
                    async with get_database_connection() as conn:
                        # First ensure conversation exists
                        await conn.execute("""
                            INSERT INTO chat_conversations (id, user_id, title, created_at, updated_at)
                            VALUES ($1, $2, $3, $4, $4)
                            ON CONFLICT (id) DO UPDATE SET
                                last_message_at = EXCLUDED.updated_at,
                                updated_at = EXCLUDED.updated_at
                        """, conversationId, user_context['user_id'], f"Chat at {datetime.utcnow().isoformat()}", datetime.utcnow())
                        
                        rows = await conn.fetch("""
                            SELECT role, content, created_at 
                            FROM chat_messages 
                            WHERE conversation_id = $1 
                            ORDER BY created_at DESC 
                            LIMIT 10
                        """, conversationId)
                        history = [{"role": row['role'], "content": row['content']} for row in reversed(rows)]
                except:
                    pass
                
                # Call RAG service
                rag_payload = {
                    "query": message,
                    "session_id": conversationId,
                    "context": {
                        "conversation_history": history,
                        "files": file_data
                    },
                    "stream": True
                }
                
                # Try to connect to RAG service
                try:
                    response = await client.post(
                        f"{RAG_SERVICE_URL}/query",
                        json=rag_payload,
                        timeout=30.0
                    )
                    
                    if response.status_code == 200:
                        # Process RAG response
                        data = response.json()
                        if 'answer' in data:
                            content = data['answer']
                            confidence = data.get('confidence_score', 0.85)
                            sources = data.get('sources', [])
                            
                            # Stream the response in chunks
                            words = content.split(' ')
                            for i in range(0, len(words), 5):  # Send 5 words at a time
                                chunk_words = words[i:i+5]
                                chunk_text = ' '.join(chunk_words) + ' '
                                
                                chunk = {
                                    "id": response_id,
                                    "content": chunk_text,
                                    "type": "chunk",
                                    "timestamp": datetime.utcnow().isoformat()
                                }
                                yield f"data: {json.dumps(chunk)}\n\n"
                                await asyncio.sleep(0.05)
                            
                            # Send completion with metadata
                            completion = {
                                "id": response_id,
                                "type": "completion",
                                "timestamp": datetime.utcnow().isoformat(),
                                "metadata": {
                                    "conversationId": conversationId,
                                    "confidence": confidence,
                                    "modelUsed": "rag-llama3.2",
                                    "processingTime": 2.5,
                                    "sources": sources
                                }
                            }
                            yield f"data: {json.dumps(completion)}\n\n"
                            
                            # Store in database
                            try:
                                async with get_database_connection() as conn:
                                    await conn.execute("""
                                        INSERT INTO chat_messages (
                                            conversation_id, role, content, metadata, created_at
                                        ) VALUES ($1, $2, $3, $4, $5)
                                    """, conversationId, "user", message, json.dumps({"files": len(file_data)}), datetime.utcnow())
                                    
                                    await conn.execute("""
                                        INSERT INTO chat_messages (
                                            conversation_id, role, content, metadata, created_at
                                        ) VALUES ($1, $2, $3, $4, $5)
                                    """, conversationId, "assistant", content, json.dumps({"confidence": confidence, "sources": sources}), datetime.utcnow())
                            except:
                                pass
                        else:
                            raise Exception("Invalid RAG response format")
                    else:
                        raise Exception(f"RAG service returned {response.status_code}")
                        
                except Exception as e:
                    # Fallback to mock response if RAG service is unavailable
                    print(f"RAG service error: {e}")
                    fallback_content = f"I understand you're asking about '{message}'.{file_context}\n\nI'm currently unable to connect to my knowledge base, but I can help you with:\n\n1. **Sample Registration**: Create and track new samples\n2. **Document Processing**: Extract data from PDFs\n3. **Protocol Access**: View standard operating procedures\n4. **Quality Control**: Validate sample metrics\n\nPlease try again in a moment or proceed with one of these options."
                    
                    # Stream fallback response
                    words = fallback_content.split(' ')
                    for i in range(0, len(words), 5):
                        chunk_words = words[i:i+5]
                        chunk_text = ' '.join(chunk_words) + ' '
                        
                        chunk = {
                            "id": response_id,
                            "content": chunk_text,
                            "type": "chunk",
                            "timestamp": datetime.utcnow().isoformat()
                        }
                        yield f"data: {json.dumps(chunk)}\n\n"
                        await asyncio.sleep(0.05)
                    
                    completion = {
                        "id": response_id,
                        "type": "completion",
                        "timestamp": datetime.utcnow().isoformat(),
                        "metadata": {
                            "conversationId": conversationId,
                            "confidence": 0.5,
                            "modelUsed": "fallback",
                            "processingTime": 0.1,
                            "error": str(e)
                        }
                    }
                    yield f"data: {json.dumps(completion)}\n\n"
                
                yield "data: [DONE]\n\n"
                
        except Exception as e:
            error_chunk = {
                "id": response_id,
                "content": f"Error: {str(e)}",
                "type": "error",
                "timestamp": datetime.utcnow().isoformat()
            }
            yield f"data: {json.dumps(error_chunk)}\n\n"
            yield "data: [DONE]\n\n"
    
    return StreamingResponse(
        generate_stream(),
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
            "X-Accel-Buffering": "no"  # Disable nginx buffering
        }
    )

@app.post("/api/documents/process")
async def process_document(
    file: UploadFile = File(...),
    background_tasks: BackgroundTasks = BackgroundTasks(),
    metadata: Optional[str] = Form(None)
):
    """
    Process uploaded documents (PDFs, Excel, etc.) and extract laboratory data.
    
    Supports:
    - PDF submission forms
    - Excel sample sheets
    - CSV data files
    """
    try:
        # Validate file type
        allowed_types = [
            "application/pdf",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "application/vnd.ms-excel",
            "text/csv"
        ]
        
        if file.content_type not in allowed_types:
            raise HTTPException(
                status_code=400,
                detail=f"Unsupported file type: {file.content_type}"
            )
        
        # Read file content
        content = await file.read()
        
        # In production, this would send to the RAG service for processing
        # For now, return mock extracted data
        extracted_data = {
            "document_type": "laboratory_submission",
            "extracted_fields": {
                "submitter": {
                    "name": "Dr. Jane Smith",
                    "institution": "Central Research Lab",
                    "email": "jane.smith@research.edu"
                },
                "samples": [{
                    "type": "DNA Extract",
                    "volume": 50.0,
                    "volume_unit": "µL",
                    "concentration": 125.0,
                    "concentration_unit": "ng/µL",
                    "quality_metrics": {
                        "A260_280": 1.85,
                        "A260_230": 2.10,
                        "RIN": 9.2
                    }
                }],
                "storage_requirements": {
                    "temperature": -20,
                    "container": "1.5mL Eppendorf tube",
                    "special_handling": "Avoid freeze-thaw cycles"
                },
                "project_info": {
                    "project_id": "PROJ-2024-001",
                    "funding_source": "NIH R01-123456"
                }
            }
        }
        
        result = {
            "success": True,
            "extracted_data": extracted_data,
            "confidence": 0.92,
            "validation_errors": None
        }
        
        return result
        
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Processing failed: {str(e)}")

class SampleCreationRequest(BaseModel):
    sample_type: str
    volume: float
    concentration: float
    buffer: str
    storage_temperature: float
    storage_location: Dict[str, str]
    project_id: Optional[str] = None
    principal_investigator: Optional[str] = None
    metadata: Optional[Dict[str, Any]] = None

@app.post("/api/samples/create")
async def create_sample_from_chat(
    request: Request,
    sample_data: SampleCreationRequest
):
    """
    Create a new sample based on chat interaction data.
    
    This endpoint is called when users confirm sample creation
    through the chat interface.
    """
    try:
        # Get user info from auth header (if available)
        auth_header = request.headers.get("Authorization", "")
        user_info = {"id": "chatbot_user", "email": "chatbot@tracseq.com"}
        
        if auth_header.startswith("Bearer "):
            token = auth_header.split(" ")[1]
            # In production, decode JWT token here
            # For now, use mock user info
            user_info = {"id": "1", "email": "admin@tracseq.com"}
        
        # Prepare sample creation payload for the sample service
        sample_payload = {
            "name": f"{sample_data.sample_type} Sample",
            "sample_type": sample_data.sample_type,
            "volume": sample_data.volume,
            "volume_unit": "µL",
            "concentration": sample_data.concentration,
            "concentration_unit": "ng/µL",
            "buffer": sample_data.buffer,
            "storage_temperature": sample_data.storage_temperature,
            "storage_location": json.dumps(sample_data.storage_location),
            "project_id": sample_data.project_id,
            "principal_investigator": sample_data.principal_investigator,
            "metadata": json.dumps(sample_data.metadata) if sample_data.metadata else "{}",
            "created_by": user_info["email"]
        }
        
        # Try to call the real sample service
        async with httpx.AsyncClient() as client:
            try:
                # Add auth header if available
                headers = {}
                if auth_header:
                    headers["Authorization"] = auth_header
                
                response = await client.post(
                    f"{LAB_MANAGER_URL}/api/samples",  # Using lab_manager as sample service
                    json=sample_payload,
                    headers=headers,
                    timeout=10.0
                )
                
                if response.status_code == 200 or response.status_code == 201:
                    data = response.json()
                    
                    # Store sample creation in chat history
                    try:
                        async with get_database_connection() as conn:
                            await conn.execute("""
                                INSERT INTO chat_messages (
                                    conversation_id, role, content, metadata, created_at
                                ) VALUES ($1, $2, $3, $4, $5)
                            """, 
                            request.headers.get("X-Conversation-Id", "system"),
                            "system",
                            f"Sample created: {data.get('barcode', 'Unknown')}",
                            json.dumps({"sample_id": data.get('id'), "action": "sample_created"}),
                            datetime.utcnow()
                            )
                    except:
                        pass
                    
                    return {
                        "success": True,
                        "sample": data,
                        "actions": {
                            "print_label": f"/api/samples/{data.get('id')}/label",
                            "view_details": f"/api/samples/{data.get('id')}",
                            "schedule_qc": f"/api/samples/{data.get('id')}/qc"
                        }
                    }
                else:
                    raise Exception(f"Sample service returned {response.status_code}: {response.text}")
                    
            except httpx.RequestError as e:
                # Fallback to mock response if service is unavailable
                print(f"Sample service error: {e}")
                sample_id = f"SAMP-{datetime.utcnow().strftime('%Y%m%d')}-{uuid.uuid4().hex[:6].upper()}"
                barcode = f"TSQ{sample_id.replace('-', '')}"
                
                response = {
                    "success": True,
                    "sample": {
                        "id": sample_id,
                        "barcode": barcode,
                        "type": sample_data.sample_type,
                        "volume": sample_data.volume,
                        "concentration": sample_data.concentration,
                        "buffer": sample_data.buffer,
                        "storage": {
                            "temperature": sample_data.storage_temperature,
                            "location": sample_data.storage_location
                        },
                        "project_id": sample_data.project_id,
                        "principal_investigator": sample_data.principal_investigator,
                        "created_at": datetime.utcnow().isoformat(),
                        "status": "registered"
                    },
                    "actions": {
                        "print_label": f"/api/samples/{sample_id}/label",
                        "view_details": f"/api/samples/{sample_id}",
                        "schedule_qc": f"/api/samples/{sample_id}/qc"
                    }
                }
                
                return response
        
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Sample creation failed: {str(e)}")

@app.get("/api/protocols/list")
async def list_protocols(
    category: Optional[str] = None,
    search: Optional[str] = None,
    limit: int = 20,
    offset: int = 0
):
    """
    Retrieve available laboratory protocols and SOPs.
    
    Query parameters:
    - category: Filter by protocol category (e.g., 'extraction', 'qc', 'storage')
    - search: Search protocols by name or content
    - limit: Number of results to return
    - offset: Pagination offset
    """
    # Mock protocol data
    # In production, this would query the protocol database
    protocols = [
        {
            "id": "SOP-001",
            "name": "DNA/RNA Extraction Protocol",
            "version": "2.3",
            "last_updated": datetime(2024, 1, 15).isoformat(),
            "category": "extraction",
            "file_url": "/protocols/SOP-001-v2.3.pdf"
        },
        {
            "id": "SOP-005",
            "name": "Library Preparation Protocol",
            "version": "1.8",
            "last_updated": datetime(2024, 2, 20).isoformat(),
            "category": "library_prep",
            "file_url": "/protocols/SOP-005-v1.8.pdf"
        },
        {
            "id": "SOP-009",
            "name": "Quality Control Standards",
            "version": "3.1",
            "last_updated": datetime(2023, 12, 10).isoformat(),
            "category": "qc",
            "file_url": "/protocols/SOP-009-v3.1.pdf"
        },
        {
            "id": "SOP-012",
            "name": "Sample Storage Guidelines",
            "version": "2.0",
            "last_updated": datetime(2024, 1, 5).isoformat(),
            "category": "storage",
            "file_url": "/protocols/SOP-012-v2.0.pdf"
        },
        {
            "id": "SOP-015",
            "name": "Sequencing Run Setup",
            "version": "1.5",
            "last_updated": datetime(2024, 3, 1).isoformat(),
            "category": "sequencing",
            "file_url": "/protocols/SOP-015-v1.5.pdf"
        }
    ]
    
    # Apply filters
    filtered_protocols = protocols
    
    if category:
        filtered_protocols = [p for p in filtered_protocols if p["category"] == category]
    
    if search:
        search_lower = search.lower()
        filtered_protocols = [
            p for p in filtered_protocols 
            if search_lower in p["name"].lower() or search_lower in p["category"].lower()
        ]
    
    # Apply pagination
    total = len(filtered_protocols)
    paginated = filtered_protocols[offset:offset + limit]
    
    return {
        "protocols": paginated,
        "total": total,
        "limit": limit,
        "offset": offset,
        "categories": list(set(p["category"] for p in protocols))
    }

@app.get("/api/chat/health")
async def chat_health():
    """Check chat service health status."""
    return {
        "status": "healthy",
        "service": "chat",
        "timestamp": datetime.utcnow().isoformat(),
        "features": {
            "streaming": True,
            "file_upload": True,
            "document_processing": True,
            "sample_creation": True,
            "protocol_access": True
        }
    }

# =============================================================================
# WebSocket Chat Endpoint
# =============================================================================

@app.websocket("/ws/chat/{conversation_id}")
async def websocket_endpoint(websocket: WebSocket, conversation_id: str):
    """WebSocket endpoint for real-time chat communication."""
    # Get user from websocket query params or headers
    auth_user = None
    try:
        # Try to get token from query params first (for WebSocket)
        token = websocket.query_params.get("token")
        if token:
            from middleware.auth import decode_token, AuthUser
            payload = decode_token(token)
            if payload:
                auth_user = AuthUser(
                    user_id=payload.get("sub", payload.get("id", "")),
                    email=payload.get("email", ""),
                    name=payload.get("name", ""),
                    role=payload.get("role", "user")
                )
    except:
        pass
    
    await websocket_chat_endpoint(websocket, conversation_id, None, auth_user)

# =============================================================================
# End of Chat API Endpoints
# =============================================================================

# Dashboard endpoints
@app.get("/api/dashboard/stats")
async def get_dashboard_stats():
    """Get dashboard statistics"""
    return {
        "totalSamples": 1247,
        "activeSamples": 89,
        "completedSamples": 1158,
        "pendingSamples": 23,
        "activeJobs": 12,
        "completedJobs": 456,
        "storageUtilization": 78.5,
        "systemHealth": "healthy",
        "lastUpdated": datetime.now().isoformat()
    }

# Samples endpoints
@app.get("/api/samples")
async def get_samples(request: Request):
    """Get all samples - proxy to sample service"""
    try:
        # Forward request to sample service
        async with httpx.AsyncClient() as client:
            # Build query parameters
            query_params = dict(request.query_params)
            
            response = await client.get(
                f"{SAMPLE_SERVICE_URL}/samples",
                params=query_params,
                headers={"Authorization": request.headers.get("Authorization", "")},
                timeout=30.0
            )
            
            if response.status_code == 200:
                data = response.json()
                # Sample service returns {data: {samples: [...], ...}, ...}
                # Transform to match frontend expectations
                samples_data = data.get('data', {}).get('samples', [])
                
                return {
                    "data": {
                        "data": samples_data,
                        "samples": samples_data,
                        "pagination": {
                            "total": len(samples_data),
                            "page": 1,
                            "per_page": len(samples_data)
                        }
                    },
                    "samples": samples_data,
                    "totalCount": len(samples_data),
                    "page": 1,
                    "pageSize": len(samples_data)
                }
            else:
                print(f"Sample service returned {response.status_code}: {response.text}")
                # Return empty response on error
                return {
                    "data": {
                        "data": [],
                        "samples": [],
                        "pagination": {
                            "total": 0,
                            "page": 1,
                            "per_page": 10
                        }
                    },
                    "samples": [],
                    "totalCount": 0,
                    "page": 1,
                    "pageSize": 10
                }
                
    except Exception as e:
        print(f"Error fetching samples from service: {e}")
        # Return empty response on error
        return {
            "data": {
                "data": [],
                "samples": [],
                "pagination": {
                    "total": 0,
                    "page": 1,
                    "per_page": 10
                }
            },
            "samples": [],
            "totalCount": 0,
            "page": 1,
            "pageSize": 10
        }

@app.post("/api/samples")
async def create_sample(request: Request):
    """Create a new sample"""
    try:
        # Get request data
        data = await request.json()
        
        # Add user context
        user = await get_current_user(request)
        if user:
            data['created_by'] = user['id']
        
        # Forward to lab_manager (acting as sample service)
        async with httpx.AsyncClient() as client:
            response = await client.post(
                f"{LAB_MANAGER_URL}/api/samples",
                json=data,
                headers={"Authorization": request.headers.get("Authorization", "")}
            )
            
            return JSONResponse(
                content=response.json(),
                status_code=response.status_code
            )
            
    except Exception as e:
        return JSONResponse(
            content={"error": f"Failed to create sample: {str(e)}"},
            status_code=500
        )

@app.post("/api/samples/batch")
async def create_samples_batch(request: Request):
    """Create multiple samples in batch"""
    try:
        # Get request data
        data = await request.json()
        
        # Transform the data to match the samples service expected format
        if isinstance(data, list):
            # If data is a list, wrap it in the expected format
            samples_data = {"samples": data}
        else:
            # If data is already an object, use it as is
            samples_data = data
        
        # Forward to sample service
        async with httpx.AsyncClient() as client:
            response = await client.post(
                f"{SAMPLE_SERVICE_URL}/samples/batch",
                json=samples_data,
                headers={"Authorization": request.headers.get("Authorization", "")},
                timeout=30.0
            )
            
            # Try to parse JSON response
            try:
                result = response.json()
                return JSONResponse(
                    content=result,
                    status_code=response.status_code
                )
            except:
                # If sample service returns non-JSON or is unavailable, provide fallback
                if response.status_code == 200 or response.status_code == 201:
                    # Service returned success but non-JSON, create mock successful response
                    sample_list = data if isinstance(data, list) else data.get("samples", [])
                    mock_samples = []
                    for i, sample_data in enumerate(sample_list):
                        sample_id = f"SMPL-{datetime.utcnow().strftime('%Y%m%d')}-{i+1:03d}"
                        mock_samples.append({
                            "id": sample_id,
                            "name": sample_data.get("name", f"Sample {i+1}"),
                            "barcode": sample_data.get("barcode", f"BC{sample_id[-6:]}"),
                            "sample_type": sample_data.get("sample_type", "DNA"),
                            "storage_location_id": sample_data.get("storage_location_id", "freezer-a1"),
                            "template_id": sample_data.get("template_id", "61c70134-9e23-4a51-9c03-9d7943299766"),
                            "status": "created",
                            "created_at": datetime.utcnow().isoformat(),
                            "metadata": sample_data.get("metadata", {})
                        })
                    
                    return JSONResponse(
                        content={
                            "success": True,
                            "data": {
                                "created_samples": mock_samples,
                                "failed_samples": [],
                                "total_created": len(mock_samples),
                                "total_failed": 0
                            },
                            "message": f"Batch processing completed: {len(mock_samples)} created, 0 failed",
                            "fallback": True
                        },
                        status_code=200
                    )
                else:
                    # Service returned error, provide fallback error response
                    return JSONResponse(
                        content={
                            "success": False,
                            "error": "Sample service unavailable, please try again later",
                            "fallback": True
                        },
                        status_code=503
                    )
            
    except Exception as e:
        print(f"Error creating samples batch: {e}")
        return JSONResponse(
            content={
                "success": False,
                "error": f"Failed to create samples batch: {str(e)}",
                "fallback": True
            },
            status_code=500
        )

@app.put("/api/samples/{sample_id}")
async def update_sample(sample_id: str, request: Request):
    """Update an individual sample"""
    try:
        # Get request data
        data = await request.json()
        
        # Forward to sample service
        async with httpx.AsyncClient() as client:
            response = await client.put(
                f"{SAMPLE_SERVICE_URL}/samples/{sample_id}",
                json=data,
                headers={"Authorization": request.headers.get("Authorization", "")},
                timeout=30.0
            )
            
            # Try to parse JSON response
            try:
                result = response.json()
                return JSONResponse(
                    content=result,
                    status_code=response.status_code
                )
            except:
                # If sample service returns non-JSON or is unavailable, provide fallback
                if response.status_code == 200:
                    # Service returned success but non-JSON, create mock successful response
                    return JSONResponse(
                        content={
                            "success": True,
                            "data": {
                                "id": sample_id,
                                "name": data.get("name", "Updated Sample"),
                                "barcode": data.get("barcode", f"BC{sample_id[-6:]}"),
                                "sample_type": data.get("sample_type", "DNA"),
                                "storage_location_id": data.get("storage_location_id"),
                                "template_id": data.get("template_id"),
                                "status": data.get("status", "updated"),
                                "updated_at": datetime.utcnow().isoformat(),
                                "metadata": data.get("metadata", {})
                            },
                            "message": f"Sample {sample_id} updated successfully",
                            "fallback": True
                        },
                        status_code=200
                    )
                else:
                    # Service returned error, provide fallback error response
                    return JSONResponse(
                        content={
                            "success": False,
                            "error": "Sample service unavailable, please try again later",
                            "fallback": True
                        },
                        status_code=503
                    )
            
    except Exception as e:
        print(f"Error updating sample {sample_id}: {e}")
        return JSONResponse(
            content={
                "success": False,
                "error": f"Failed to update sample: {str(e)}",
                "fallback": True
            },
            status_code=500
        )

@app.get("/api/samples/{sample_id}")
async def get_sample(sample_id: str, request: Request):
    """Get an individual sample by ID"""
    try:
        # Forward to sample service
        async with httpx.AsyncClient() as client:
            response = await client.get(
                f"{SAMPLE_SERVICE_URL}/samples/{sample_id}",
                headers={"Authorization": request.headers.get("Authorization", "")},
                timeout=30.0
            )
            
            # Try to parse JSON response
            try:
                result = response.json()
                return JSONResponse(
                    content=result,
                    status_code=response.status_code
                )
            except:
                # If sample service returns non-JSON or is unavailable, provide fallback
                if response.status_code == 200:
                    # Service returned success but non-JSON, create mock response
                    return JSONResponse(
                        content={
                            "success": True,
                            "data": {
                                "id": sample_id,
                                "name": f"Sample {sample_id[-6:]}",
                                "barcode": f"BC{sample_id[-6:]}",
                                "sample_type": "DNA",
                                "status": "pending",
                                "created_at": datetime.utcnow().isoformat(),
                                "metadata": {}
                            },
                            "fallback": True
                        },
                        status_code=200
                    )
                else:
                    # Service returned error
                    return JSONResponse(
                        content={
                            "success": False,
                            "error": f"Sample {sample_id} not found",
                            "fallback": True
                        },
                        status_code=404
                    )
            
    except Exception as e:
        print(f"Error getting sample {sample_id}: {e}")
        return JSONResponse(
            content={
                "success": False,
                "error": f"Failed to get sample: {str(e)}",
                "fallback": True
            },
            status_code=500
        )

# Templates endpoints
@app.get("/api/templates")
async def get_templates():
    """Get all templates by proxying to template service"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(
                f"{TEMPLATE_SERVICE_URL}/templates",
                timeout=10.0
            )
            
            if response.status_code == 200:
                data = response.json()
                # Template service returns {data: [...], pagination: {...}}
                # Frontend expects just the array
                if isinstance(data, dict) and 'data' in data:
                    return data['data']
                return data
            else:
                print(f"Template service returned {response.status_code}")
                return []
                
    except Exception as e:
        print(f"Error fetching templates: {e}")
        # Fallback to empty array
        return []

# Sequencing endpoints
@app.get("/api/sequencing/jobs")
async def get_sequencing_jobs():
    """Get sequencing jobs"""
    jobs_data = [
        {
            "id": "SEQ-001",
            "name": "Whole Genome Sequencing - Batch 1",
            "status": "Running",
            "progress": 67,
            "sampleCount": 24,
            "startTime": (datetime.now() - timedelta(hours=4)).isoformat(),
            "estimatedCompletion": (datetime.now() + timedelta(hours=2)).isoformat(),
            "instrument": "Illumina NovaSeq"
        },
        {
            "id": "SEQ-002",
            "name": "RNA-seq Analysis - Project Alpha",
            "status": "Completed",
            "progress": 100,
            "sampleCount": 48,
            "startTime": (datetime.now() - timedelta(days=2)).isoformat(),
            "completionTime": (datetime.now() - timedelta(hours=6)).isoformat(),
            "instrument": "Illumina MiSeq"
        },
        {
            "id": "SEQ-003",
            "name": "Targeted Sequencing - Panel B",
            "status": "Queued",
            "progress": 0,
            "sampleCount": 12,
            "queuePosition": 2,
            "estimatedStart": (datetime.now() + timedelta(hours=6)).isoformat(),
            "instrument": "Ion Torrent"
        }
    ]

    # Return both formats for compatibility
    return {
        "data": jobs_data,  # For frontend expecting .data.filter()
        "jobs": jobs_data,  # For other consumers
        "totalCount": 45,
        "activeJobs": 12,
        "queuedJobs": 8
    }

@app.post("/api/sequencing/jobs")
async def create_sequencing_job(request: Request):
    """Create a new sequencing job"""
    return {
        "id": f"SEQ-{datetime.now().strftime('%Y%m%d%H%M%S')}",
        "status": "created",
        "message": "Sequencing job created successfully"
    }

# Storage endpoints
@app.get("/api/storage/locations")
async def get_storage_locations(
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100),
    container_type: str = Query(None)
):
    """Get all storage locations with pagination - also handles containers"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(
                f"{STORAGE_SERVICE_URL}/api/storage/locations",
                params={"page": page, "per_page": per_page}
            )
            
            # Try to parse JSON response
            try:
                return JSONResponse(
                    content=response.json(),
                    status_code=response.status_code
                )
            except:
                # If storage service returns non-JSON, provide fallback data
                # Handle containers request via locations endpoint
                if container_type == "freezer":
                    fallback_locations = [
                        {
                            "id": "freezer-1",
                            "name": "ULF-A-Freezer-1",
                            "container_type": "freezer",
                            "parent_id": None,
                            "location_id": "lab-a",
                            "capacity": 1000,
                            "current_usage": 750,
                            "status": "active",
                            "min_temperature_celsius": -85.0,
                            "max_temperature_celsius": -75.0,
                            "barcode": "FRZ001",
                            "description": "Ultra-low temperature freezer",
                            "created_at": "2024-01-01T00:00:00Z",
                            "updated_at": "2024-01-01T00:00:00Z"
                        },
                        {
                            "id": "freezer-2",
                            "name": "ULF-A-Freezer-2",
                            "container_type": "freezer",
                            "parent_id": None,
                            "location_id": "lab-a",
                            "capacity": 1000,
                            "current_usage": 600,
                            "status": "active",
                            "min_temperature_celsius": -85.0,
                            "max_temperature_celsius": -75.0,
                            "barcode": "FRZ002",
                            "description": "Ultra-low temperature freezer",
                            "created_at": "2024-01-01T00:00:00Z",
                            "updated_at": "2024-01-01T00:00:00Z"
                        },
                        {
                            "id": "freezer-3",
                            "name": "ULF-A-Freezer-3",
                            "container_type": "freezer",
                            "parent_id": None,
                            "location_id": "lab-a",
                            "capacity": 1000,
                            "current_usage": 400,
                            "status": "active",
                            "min_temperature_celsius": -85.0,
                            "max_temperature_celsius": -75.0,
                            "barcode": "FRZ003",
                            "description": "Ultra-low temperature freezer",
                            "created_at": "2024-01-01T00:00:00Z",
                            "updated_at": "2024-01-01T00:00:00Z"
                        }
                    ]
                else:
                    fallback_locations = [
                        {
                            "id": "freezer-a1",
                            "name": "Freezer A1 (-80°C)",
                            "description": "Ultra-low temperature freezer for long-term storage",
                            "location_type": "freezer",
                            "temperature_zone": "-80C",
                            "max_capacity": 1000,
                            "current_capacity": 250,
                            "utilization_percentage": 25.0,
                            "status": "active",
                            "coordinates": {"x": 10, "y": 20},
                            "created_at": "2024-01-01T00:00:00Z",
                            "updated_at": "2024-01-01T00:00:00Z"
                        },
                        {
                            "id": "fridge-b2",
                            "name": "Refrigerator B2 (4°C)",
                            "description": "Standard refrigerator for short-term storage",
                            "location_type": "refrigerator",
                            "temperature_zone": "4C",
                            "max_capacity": 500,
                            "current_capacity": 150,
                            "utilization_percentage": 30.0,
                            "status": "active",
                            "coordinates": {"x": 30, "y": 40},
                            "created_at": "2024-01-01T00:00:00Z",
                            "updated_at": "2024-01-01T00:00:00Z"
                        },
                        {
                            "id": "room-temp-c3",
                            "name": "Room Temperature C3 (20°C)",
                            "description": "Room temperature storage for stable samples",
                            "location_type": "room_temp",
                            "temperature_zone": "RT",
                            "max_capacity": 200,
                            "current_capacity": 75,
                            "utilization_percentage": 37.5,
                            "status": "active",
                            "coordinates": {"x": 50, "y": 60},
                            "created_at": "2024-01-01T00:00:00Z",
                            "updated_at": "2024-01-01T00:00:00Z"
                        }
                    ]
                
                return JSONResponse(
                    content={
                        "data": fallback_locations,
                        "locations": fallback_locations,
                        "containers": fallback_locations if container_type else [],
                        "total": len(fallback_locations),
                        "page": page,
                        "per_page": per_page,
                        "total_pages": 1,
                        "message": f"Using fallback {'containers' if container_type else 'locations'} data - storage service unavailable"
                    },
                    status_code=200
                )
                
    except Exception as e:
        # Complete fallback if storage service is unreachable
        # Handle containers request via locations endpoint
        if container_type == "freezer":
            fallback_locations = [
                {
                    "id": "freezer-1",
                    "name": "ULF-A-Freezer-1",
                    "container_type": "freezer",
                    "parent_id": None,
                    "location_id": "lab-a",
                    "capacity": 1000,
                    "current_usage": 750,
                    "status": "active",
                    "min_temperature_celsius": -85.0,
                    "max_temperature_celsius": -75.0,
                    "barcode": "FRZ001",
                    "description": "Ultra-low temperature freezer",
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z"
                },
                {
                    "id": "freezer-2",
                    "name": "ULF-A-Freezer-2",
                    "container_type": "freezer",
                    "parent_id": None,
                    "location_id": "lab-a",
                    "capacity": 1000,
                    "current_usage": 600,
                    "status": "active",
                    "min_temperature_celsius": -85.0,
                    "max_temperature_celsius": -75.0,
                    "barcode": "FRZ002",
                    "description": "Ultra-low temperature freezer",
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z"
                },
                {
                    "id": "freezer-3",
                    "name": "ULF-A-Freezer-3",
                    "container_type": "freezer",
                    "parent_id": None,
                    "location_id": "lab-a",
                    "capacity": 1000,
                    "current_usage": 400,
                    "status": "active",
                    "min_temperature_celsius": -85.0,
                    "max_temperature_celsius": -75.0,
                    "barcode": "FRZ003",
                    "description": "Ultra-low temperature freezer",
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z"
                }
            ]
        else:
            fallback_locations = [
                {
                    "id": "freezer-a1",
                    "name": "Freezer A1 (-80°C)",
                    "description": "Ultra-low temperature freezer for long-term storage",
                    "location_type": "freezer",
                    "temperature_zone": "-80C",
                    "max_capacity": 1000,
                    "current_capacity": 250,
                    "utilization_percentage": 25.0,
                    "status": "active",
                    "coordinates": {"x": 10, "y": 20},
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z"
                },
                {
                    "id": "fridge-b2",
                    "name": "Refrigerator B2 (4°C)",
                    "description": "Standard refrigerator for short-term storage",
                    "location_type": "refrigerator",
                    "temperature_zone": "4C",
                    "max_capacity": 500,
                    "current_capacity": 150,
                    "utilization_percentage": 30.0,
                    "status": "active",
                    "coordinates": {"x": 30, "y": 40},
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z"
                },
                {
                    "id": "room-temp-c3",
                    "name": "Room Temperature C3 (20°C)",
                    "description": "Room temperature storage for stable samples",
                    "location_type": "room_temp",
                    "temperature_zone": "RT",
                    "max_capacity": 200,
                    "current_capacity": 75,
                    "utilization_percentage": 37.5,
                    "status": "active",
                    "coordinates": {"x": 50, "y": 60},
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z"
                }
            ]
        
        return JSONResponse(
            content={
                "data": fallback_locations,
                "locations": fallback_locations,
                "containers": fallback_locations if container_type else [],
                "total": len(fallback_locations),
                "page": page,
                "per_page": per_page,
                "total_pages": 1,
                "message": f"Using fallback {'containers' if container_type else 'locations'} data - storage service error: {str(e)}"
            },
            status_code=200
        )

@app.post("/api/storage/locations")
async def create_storage_location(request: Request):
    """Create a new storage location"""
    try:
        data = await request.json()
        
        async with httpx.AsyncClient() as client:
            response = await client.post(
                f"{STORAGE_SERVICE_URL}/api/storage/locations",
                json=data,
                headers={"Authorization": request.headers.get("Authorization", "")},
                timeout=30.0
            )
            
            # Try to parse JSON response
            try:
                return JSONResponse(
                    content=response.json(),
                    status_code=response.status_code
                )
            except:
                # If storage service returns non-JSON or is unavailable, provide fallback
                if response.status_code == 200 or response.status_code == 201:
                    # Service returned success but non-JSON, create mock successful response
                    location_id = f"loc-{datetime.utcnow().strftime('%Y%m%d%H%M%S')}"
                    mock_location = {
                        "id": location_id,
                        "name": data.get("name", "New Location"),
                        "description": data.get("description", ""),
                        "location_type": data.get("location_type", "freezer"),
                        "temperature_zone": data.get("temperature_zone", "-80C"),
                        "max_capacity": data.get("max_capacity", 100),
                        "current_capacity": 0,
                        "utilization_percentage": 0.0,
                        "status": "active",
                        "coordinates": {"x": 0, "y": 0},
                        "created_at": datetime.utcnow().isoformat(),
                        "updated_at": datetime.utcnow().isoformat()
                    }
                    
                    return JSONResponse(
                        content={
                            "success": True,
                            "data": mock_location,
                            "message": "Storage location created successfully",
                            "fallback": True
                        },
                        status_code=201
                    )
                else:
                    # Service returned error, provide fallback error response
                    return JSONResponse(
                        content={
                            "success": False,
                            "error": "Storage service unavailable, please try again later",
                            "fallback": True
                        },
                        status_code=503
                    )
            
    except Exception as e:
        print(f"Error creating storage location: {e}")
        return JSONResponse(
            content={
                "success": False,
                "error": f"Failed to create storage location: {str(e)}",
                "fallback": True
            },
            status_code=500
        )

@app.get("/api/storage/locations/{location_id}")
async def get_storage_location(location_id: str):
    """Get a specific storage location"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(
                f"{STORAGE_SERVICE_URL}/api/storage/locations/{location_id}"
            )
            return JSONResponse(
                content=response.json(),
                status_code=response.status_code
            )
    except Exception as e:
        return JSONResponse(
            content={"error": f"Failed to fetch storage location: {str(e)}"},
            status_code=500
        )

@app.put("/api/storage/locations/{location_id}")
async def update_storage_location(location_id: str, request: Request):
    """Update a storage location"""
    try:
        data = await request.json()
        
        async with httpx.AsyncClient() as client:
            response = await client.put(
                f"{STORAGE_SERVICE_URL}/storage/locations/{location_id}",
                json=data,
                headers={"Authorization": request.headers.get("Authorization", "")}
            )
            return JSONResponse(
                content=response.json(),
                status_code=response.status_code
            )
    except Exception as e:
        return JSONResponse(
            content={"error": f"Failed to update storage location: {str(e)}"},
            status_code=500
        )

@app.delete("/api/storage/locations/{location_id}")
async def delete_storage_location(location_id: str, request: Request):
    """Delete a storage location"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.delete(
                f"{STORAGE_SERVICE_URL}/storage/locations/{location_id}",
                headers={"Authorization": request.headers.get("Authorization", "")}
            )
            return JSONResponse(
                content=response.json(),
                status_code=response.status_code
            )
    except Exception as e:
        return JSONResponse(
            content={"error": f"Failed to delete storage location: {str(e)}"},
            status_code=500
        )

@app.get("/api/storage/locations/{location_id}/capacity")
async def get_location_capacity(location_id: str):
    """Get capacity information for a storage location"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(
                f"{STORAGE_SERVICE_URL}/storage/locations/{location_id}/capacity"
            )
            return JSONResponse(
                content=response.json(),
                status_code=response.status_code
            )
    except Exception as e:
        return JSONResponse(
            content={"error": f"Failed to fetch location capacity: {str(e)}"},
            status_code=500
        )

# TEST: Simple storage sample endpoint for debugging
@app.get("/api/storage/test-samples")
async def test_storage_samples():
    """Test endpoint to verify registration"""
    return {"message": "Storage samples test endpoint working", "data": []}

# Additional storage endpoints (working versions)
@app.get("/api/storage/samples")
async def get_storage_samples_working(
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100),
    container_type: str = Query(None)
):
    """Get all storage samples with pagination - Working version - Also handles containers"""
    
    # HIERARCHICAL STORAGE FIX: Return containers when container_type is specified
    if container_type == "freezer":
        mock_containers = [
            {
                "id": "freezer-1",
                "name": "ULF-A-Freezer-1",
                "container_type": "freezer",
                "parent_id": None,
                "location_id": "lab-a",
                "capacity": 1000,
                "current_usage": 750,
                "status": "active",
                "min_temperature_celsius": -85.0,
                "max_temperature_celsius": -75.0,
                "barcode": "FRZ001",
                "description": "Ultra-low temperature freezer",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            },
            {
                "id": "freezer-2",
                "name": "ULF-A-Freezer-2",
                "container_type": "freezer", 
                "parent_id": None,
                "location_id": "lab-a",
                "capacity": 1000,
                "current_usage": 600,
                "status": "active",
                "min_temperature_celsius": -85.0,
                "max_temperature_celsius": -75.0,
                "barcode": "FRZ002",
                "description": "Ultra-low temperature freezer",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            },
            {
                "id": "freezer-3",
                "name": "ULF-A-Freezer-3",
                "container_type": "freezer",
                "parent_id": None,
                "location_id": "lab-a",
                "capacity": 1000,
                "current_usage": 400,
                "status": "active",
                "min_temperature_celsius": -85.0,
                "max_temperature_celsius": -75.0,
                "barcode": "FRZ003",
                "description": "Ultra-low temperature freezer",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        ]
        
        return {
            "data": mock_containers,
            "containers": mock_containers,
            "total": len(mock_containers),
            "page": page,
            "per_page": per_page,
            "message": "Hierarchical storage containers via samples endpoint"
        }
    
    # Default: Return samples
    mock_samples = [
        {
            "id": "SMPL-001",
            "barcode": "BC001",
            "sample_type": "DNA",
            "location_id": "freezer-a1",
            "location_name": "Freezer A1 (-80°C)",
            "status": "stored",
            "stored_at": "2024-01-15T10:30:00Z",
            "volume": 100.0,
            "concentration": 50.0
        },
        {
            "id": "SMPL-002", 
            "barcode": "BC002",
            "sample_type": "RNA",
            "location_id": "fridge-b2",
            "location_name": "Refrigerator B2 (4°C)",
            "status": "stored",
            "stored_at": "2024-01-16T14:20:00Z",
            "volume": 75.0,
            "concentration": 30.0
        }
    ]
    
    return {
        "data": mock_samples,
        "total": len(mock_samples),
        "page": page,
        "per_page": per_page
    }

@app.get("/api/storage/analytics/utilization")
async def get_storage_utilization_working():
    """Get storage utilization analytics - Working version"""
    mock_utilization = {
        "data": {
            "total_capacity": 5000,
            "total_used": 3200,
            "utilization_percentage": 64.0,
            "zones": [
                {"name": "-80C", "capacity": 1000, "used": 800, "utilization": 80.0},
                {"name": "-20C", "capacity": 800, "used": 600, "utilization": 75.0},
                {"name": "4C", "capacity": 1200, "used": 900, "utilization": 75.0},
                {"name": "RT", "capacity": 2000, "used": 900, "utilization": 45.0}
            ]
        }
    }
    
    return mock_utilization

# CONTAINERS ALIAS: Map containers request to locations (WORKING FIX)
# DISABLED: Duplicate endpoint - using the one at line 2309
# @app.get("/api/storage/containers")
async def DISABLED_get_storage_containers_alias(
    container_type: str = Query(None),
    parent_id: str = Query(None), 
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100)
):
    '''Get storage containers - alias to locations for frontend compatibility'''
    # Return hierarchical storage containers data from database
    containers = [
        {
            "id": "freezer-1",
            "name": "ULF-A-Freezer-1",
            "container_type": "freezer",
            "parent_id": None,
            "location_id": "lab-a",
            "capacity": 1000,
            "current_usage": 750,
            "status": "active",
            "min_temperature_celsius": -85.0,
            "max_temperature_celsius": -75.0,
            "barcode": "FRZ001",
            "description": "Ultra-low temperature freezer",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        },
        {
            "id": "freezer-2",
            "name": "ULF-A-Freezer-2", 
            "container_type": "freezer",
            "parent_id": None,
            "location_id": "lab-a",
            "capacity": 1000,
            "current_usage": 600,
            "status": "active",
            "min_temperature_celsius": -85.0,
            "max_temperature_celsius": -75.0,
            "barcode": "FRZ002",
            "description": "Ultra-low temperature freezer",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        },
        {
            "id": "freezer-3",
            "name": "ULF-A-Freezer-3",
            "container_type": "freezer",
            "parent_id": None,
            "location_id": "lab-a",
            "capacity": 1000,
            "current_usage": 400,
            "status": "active",
            "min_temperature_celsius": -85.0,
            "max_temperature_celsius": -75.0,
            "barcode": "FRZ003",
            "description": "Ultra-low temperature freezer",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        },
        {
            "id": "refrigerator-1",
            "name": "REF-A-Refrigerator-1",
            "container_type": "freezer",
            "parent_id": None,
            "location_id": "lab-a",
            "capacity": 500,
            "current_usage": 200,
            "status": "active",
            "min_temperature_celsius": 2.0,
            "max_temperature_celsius": 6.0,
            "barcode": "REF001",
            "description": "Refrigerated storage",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        }
    ]
    
    # Filter by container_type if specified
    if container_type:
        containers = [c for c in containers if c["container_type"] == container_type]
    
    # Filter by parent_id if specified
    if parent_id:
        containers = [c for c in containers if c.get("parent_id") == parent_id]
    
    # Apply pagination
    start_idx = (page - 1) * per_page
    end_idx = start_idx + per_page
    paginated_containers = containers[start_idx:end_idx]
    
    return {
        "data": paginated_containers,
        "containers": paginated_containers,
        "total": len(containers),
        "page": page,
        "per_page": per_page,
        "total_pages": (len(containers) + per_page - 1) // per_page,
        "message": "Hierarchical storage containers from TracSeq 2.0"
    }

# WORKING HIERARCHICAL STORAGE: Containers endpoint using a different route
@app.get("/api/storage/hierarchical/containers")
async def get_hierarchical_containers(
    container_type: str = Query(None)
):
    """Get hierarchical storage containers - using working endpoint pattern"""
    containers = [
        {
            "id": "freezer-1",
            "name": "ULF-A-Freezer-1",
            "container_type": "freezer",
            "parent_id": None,
            "location_id": "lab-a",
            "capacity": 1000,
            "current_usage": 750,
            "status": "active",
            "min_temperature_celsius": -85.0,
            "max_temperature_celsius": -75.0,
            "barcode": "FRZ001",
            "description": "Ultra-low temperature freezer",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        },
        {
            "id": "freezer-2",
            "name": "ULF-A-Freezer-2", 
            "container_type": "freezer",
            "parent_id": None,
            "location_id": "lab-a",
            "capacity": 1000,
            "current_usage": 600,
            "status": "active",
            "min_temperature_celsius": -85.0,
            "max_temperature_celsius": -75.0,
            "barcode": "FRZ002",
            "description": "Ultra-low temperature freezer",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        }
    ]
    
    return {
        "data": containers,
        "containers": containers,
        "total": len(containers),
        "message": "Hierarchical storage containers working!"
    }

# HIERARCHICAL STORAGE: Containers endpoint (simplified)
# DISABLED: Duplicate endpoint - using the one at line 2309
# @app.get("/api/storage/containers")
async def DISABLED_get_storage_containers_simple(
    container_type: str = Query(None)
):
    """Get storage containers with hierarchical filtering"""
    return {
        "data": [
            {
                "id": "freezer-1",
                "name": "ULF-A-Freezer-1", 
                "container_type": "freezer",
                "status": "active"
            }
        ],
        "message": "Simple hierarchical storage data"
    }

@app.get("/api/storage/mobile/samples")
async def get_mobile_samples_working(
    status: str = Query(None),
    sample_type: str = Query(None),
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100)
):
    """Get samples optimized for mobile interface - Working version"""
    mock_mobile_samples = [
        {
            "id": "SMPL-001",
            "barcode": "BC001",
            "type": "DNA",
            "location": "A1",
            "status": "stored",
            "temp": "-80°C"
        },
        {
            "id": "SMPL-002",
            "barcode": "BC002", 
            "type": "RNA",
            "location": "B2",
            "status": "stored",
            "temp": "4°C"
        }
    ]
    
    return {
        "data": mock_mobile_samples,
        "total": len(mock_mobile_samples),
        "page": page,
        "per_page": per_page
    }

# Store samples endpoint
@app.post("/api/storage/samples")
async def store_sample_in_storage(request: Request):
    """Store a sample in storage location"""
    try:
        data = await request.json()
        
        # Mock storage response - in production this would store in database
        sample_id = f"STORED-{int(time.time())}"
        
        response_data = {
            "success": True,
            "message": "Sample stored successfully",
            "data": {
                "id": sample_id,
                "barcode": data.get("barcode"),
                "sample_type": data.get("sample_type"),
                "storage_location_id": data.get("storage_location_id"),
                "temperature_requirements": data.get("temperature_requirements"),
                "status": "stored",
                "stored_at": datetime.now().isoformat(),
                "metadata": data.get("metadata", {})
            }
        }
        
        return JSONResponse(content=response_data, status_code=201)
        
    except Exception as e:
        return JSONResponse(
            content={"error": f"Failed to store sample: {str(e)}"},
            status_code=500
        )

# Sample retrieval endpoint
@app.post("/api/storage/samples/{sample_id}/retrieve")
async def retrieve_sample_from_storage(sample_id: str, request: Request):
    """Retrieve a sample from storage"""
    try:
        response_data = {
            "success": True,
            "message": f"Sample {sample_id} retrieved successfully",
            "data": {
                "id": sample_id,
                "status": "retrieved",
                "retrieved_at": datetime.now().isoformat(),
                "retrieved_by": "current_user"
            }
        }
        
        return JSONResponse(content=response_data, status_code=200)
        
    except Exception as e:
        return JSONResponse(
            content={"error": f"Failed to retrieve sample: {str(e)}"},
            status_code=500
        )

# Sample movement endpoint
@app.post("/api/storage/samples/{sample_id}/move")
async def move_sample_in_storage(sample_id: str, request: Request):
    """Move a sample to a new storage location"""
    try:
        data = await request.json()
        
        response_data = {
            "success": True,
            "message": f"Sample {sample_id} moved successfully",
            "data": {
                "id": sample_id,
                "old_location": "previous_location",
                "new_location_id": data.get("new_location_id"),
                "reason": data.get("reason"),
                "moved_at": datetime.now().isoformat(),
                "moved_by": "current_user"
            }
        }
        
        return JSONResponse(content=response_data, status_code=200)
        
    except Exception as e:
        return JSONResponse(
            content={"error": f"Failed to move sample: {str(e)}"},
            status_code=500
        )

# Sample search endpoint
@app.get("/api/storage/samples/search")
async def search_samples_in_storage(
    barcode: str = Query(None),
    location_id: str = Query(None),
    sample_type: str = Query(None),
    status: str = Query(None)
):
    """Search for samples in storage"""
    try:
        # Mock search results
        mock_results = [
            {
                "id": "SMPL-001",
                "barcode": barcode or "BC001",
                "sample_type": "DNA",
                "storage_location_id": "freezer-a1",
                "status": "stored",
                "stored_at": "2024-01-15T10:30:00Z"
            }
        ]
        
        return {
            "success": True,
            "data": mock_results,
            "total": len(mock_results)
        }
        
    except Exception as e:
        return JSONResponse(
            content={"error": f"Failed to search samples: {str(e)}"},
            status_code=500
        )

# TEST: Simple endpoint to verify registration works
@app.get("/api/storage/test-containers")
async def test_containers_endpoint():
    """Test endpoint to verify new endpoints work"""
    return {"message": "Test containers endpoint working!", "timestamp": datetime.now().isoformat()}

# Storage containers endpoint - HIERARCHICAL STORAGE
@app.get("/api/storage/containers")
async def get_storage_containers(
    container_type: str = Query(None),
    parent_id: str = Query(None),
    page: int = Query(1, ge=1),
    per_page: int = Query(20, ge=1, le=100)
):
    """Get storage containers with hierarchical filtering"""
    # Fallback mock data for hierarchical storage containers
    mock_containers = []
    
    if container_type == "freezer" or container_type is None:
        mock_containers.extend([
            {
                "id": "freezer-1",
                "name": "ULF-A-Freezer-1",
                "container_type": "freezer",
                "parent_id": None,
                "location_id": "lab-a",
                "capacity": 1000,
                "current_usage": 750,
                "status": "active",
                "min_temperature_celsius": -85.0,
                "max_temperature_celsius": -75.0,
                "barcode": "FRZ001",
                "description": "Ultra-low temperature freezer",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            },
            {
                "id": "freezer-2", 
                "name": "ULF-A-Freezer-2",
                "container_type": "freezer",
                "parent_id": None,
                "location_id": "lab-a",
                "capacity": 1000,
                "current_usage": 600,
                "status": "active",
                "min_temperature_celsius": -85.0,
                "max_temperature_celsius": -75.0,
                "barcode": "FRZ002",
                "description": "Ultra-low temperature freezer",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            },
            {
                "id": "freezer-3",
                "name": "ULF-A-Freezer-3",
                "container_type": "freezer",
                "parent_id": None,
                "location_id": "lab-a",
                "capacity": 1000,
                "current_usage": 400,
                "status": "active",
                "min_temperature_celsius": -85.0,
                "max_temperature_celsius": -75.0,
                "barcode": "FRZ003",
                "description": "Ultra-low temperature freezer",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            },
            {
                "id": "refrigerator-1",
                "name": "REF-A-Refrigerator-1",
                "container_type": "freezer",
                "parent_id": None,
                "location_id": "lab-a",
                "capacity": 500,
                "current_usage": 200,
                "status": "active",
                "min_temperature_celsius": 2.0,
                "max_temperature_celsius": 6.0,
                "barcode": "REF001",
                "description": "Refrigerated storage",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        ])
    
    if container_type == "rack" or container_type is None:
        mock_containers.extend([
            {
                "id": "rack-1",
                "name": "ULF-A-F1-R1",
                "container_type": "rack",
                "parent_id": "freezer-1",
                "location_id": "lab-a",
                "capacity": 100,
                "current_usage": 75,
                "status": "active",
                "min_temperature_celsius": None,
                "max_temperature_celsius": None,
                "barcode": "R001",
                "description": "Storage rack in freezer 1",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            },
            {
                "id": "rack-2",
                "name": "ULF-A-F1-R2",
                "container_type": "rack",
                "parent_id": "freezer-1",
                "location_id": "lab-a",
                "capacity": 100,
                "current_usage": 60,
                "status": "active",
                "min_temperature_celsius": None,
                "max_temperature_celsius": None,
                "barcode": "R002",
                "description": "Storage rack in freezer 1",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        ])
    
    if container_type == "box" or container_type is None:
        mock_containers.extend([
            {
                "id": "box-1",
                "name": "ULF-A-F1-R1-Box-1",
                "container_type": "box",
                "parent_id": "rack-1",
                "location_id": "lab-a",
                "capacity": 100,
                "current_usage": 25,
                "status": "active",
                "min_temperature_celsius": None,
                "max_temperature_celsius": None,
                "barcode": "B001",
                "description": "Storage box in rack 1",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            },
            {
                "id": "box-2",
                "name": "ULF-A-F1-R1-Box-2",
                "container_type": "box",
                "parent_id": "rack-1",
                "location_id": "lab-a",
                "capacity": 100,
                "current_usage": 50,
                "status": "active",
                "min_temperature_celsius": None,
                "max_temperature_celsius": None,
                "barcode": "B002",
                "description": "Storage box in rack 1",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        ])
    
    # Filter by parent_id if specified
    if parent_id:
        mock_containers = [c for c in mock_containers if c["parent_id"] == parent_id]
    
    # Apply pagination
    start_idx = (page - 1) * per_page
    end_idx = start_idx + per_page
    paginated_containers = mock_containers[start_idx:end_idx]
    
    return JSONResponse(
        content={
            "data": paginated_containers,
            "containers": paginated_containers,
            "total": len(mock_containers),
            "page": page,
            "per_page": per_page,
            "total_pages": (len(mock_containers) + per_page - 1) // per_page,
            "message": "Using fallback hierarchical storage data"
        },
        status_code=200
    )

# Storage container details endpoint
@app.get("/api/storage/containers/{container_id}")
async def get_storage_container(container_id: str, include_samples: bool = Query(False)):
    """Get specific storage container details with hierarchical children"""
    
    # Determine container type and generate appropriate children
    # Check in order from most specific to least specific
    print(f"DEBUG: Determining container type for ID: {container_id}")
    if container_id.startswith("pos-"):
        container_type = "position"
        print(f"DEBUG: Identified as position")
        children = []  # Positions don't have children
    elif container_id.startswith("box-"):
        container_type = "box"
        print(f"DEBUG: Identified as box")
        # Boxes contain positions
        children = [
            {
                "id": f"pos-{container_id}-{i}",
                "name": f"Position {chr(65 + i // 5)}{(i % 5) + 1}",
                "container_type": "position",
                "parent_container_id": container_id,
                "capacity": 1,
                "occupied_count": 1 if i < 15 else 0,  # First 15 positions occupied
                "status": "active" if i < 15 else "available",
                "barcode": f"POS{chr(65 + i // 5)}{(i % 5) + 1}",
                "description": f"Sample position {chr(65 + i // 5)}{(i % 5) + 1} in {container_id}",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
            for i in range(25)  # 25 positions per box (5x5 grid)
        ]
    elif container_id.startswith("rack-"):
        container_type = "rack"
        print(f"DEBUG: Identified as rack")
        # Racks contain boxes
        children = [
            {
                "id": f"box-{container_id}-{i}",
                "name": f"Box {i + 1}",
                "container_type": "box",
                "parent_container_id": container_id,
                "capacity": 25,
                "occupied_count": 15 + i * 2,
                "status": "active",
                "barcode": f"BOX{i + 1:03d}",
                "description": f"Storage box {i + 1} in {container_id}",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
            for i in range(5)  # 5 boxes per rack
        ]
    elif container_id.startswith("freezer-"):
        container_type = "freezer"
        print(f"DEBUG: Identified as freezer")
        # Freezers contain racks
        children = [
            {
                "id": f"rack-{container_id}-{i}",
                "name": f"Rack {chr(65 + i)}",
                "container_type": "rack",
                "parent_container_id": container_id,
                "capacity": 100,
                "occupied_count": 40 + i * 10,
                "status": "active",
                "barcode": f"RCK{chr(65 + i)}0{i + 1}",
                "description": f"Storage rack {chr(65 + i)} in {container_id}",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
            for i in range(4)  # 4 racks per freezer
        ]
    else:
        container_type = "unknown"
        children = []
    
    # Generate sample data if requested and if this is a position-level container
    samples = []
    if include_samples and container_type in ["position", "box"]:
        samples = [
            {
                "id": f"sample-{container_id}-{i}",
                "sample_id": f"sample-{container_id}-{i}",
                "position_identifier": f"pos-{container_id}-{i}",
                "assigned_at": "2024-01-15T10:30:00Z",
                "status": "occupied",
                "notes": f"Sample {i + 1} stored in {container_id}"
            }
            for i in range(min(3, 15) if container_type == "box" else 1)  # 1 sample per position, up to 3 samples per box
        ]
    
    # Determine temperature zone based on container type
    if "freezer" in container_id:
        temp_zone = "-80C" if "1" in container_id else "-20C" if "2" in container_id else "4C"
        min_temp = -85.0 if temp_zone == "-80C" else -25.0 if temp_zone == "-20C" else 2.0
        max_temp = -75.0 if temp_zone == "-80C" else -15.0 if temp_zone == "-20C" else 6.0
    else:
        temp_zone = "-80C"  # Default for child containers
        min_temp = -85.0
        max_temp = -75.0
    
    # Generate proper container name based on type
    if container_type == "freezer":
        container_name = f"ULF-A-Freezer-{container_id.split('-')[-1]}"
    elif container_type == "rack":
        rack_letter = container_id.split('-')[-1]
        container_name = f"Rack {chr(65 + int(rack_letter)) if rack_letter.isdigit() else rack_letter.upper()}"
    elif container_type == "box":
        box_number = container_id.split('-')[-1]
        container_name = f"Box {box_number}"
    else:
        container_name = f"Position {container_id.split('-')[-1]}"
    
    return JSONResponse(content={
        "data": {
            "container": {
                "id": container_id,
                "name": container_name,
                "container_type": container_type,
                "parent_container_id": None if "freezer" in container_id else extract_parent_id(container_id),
                "location_id": "lab-a",
                "capacity": 1000 if container_type == "freezer" else 100 if container_type == "rack" else 25 if container_type == "box" else 1,
                "occupied_count": len(children) if children else len(samples),
                "status": "active",
                "temperature_zone": temp_zone,
                "min_temperature_celsius": min_temp,
                "max_temperature_celsius": max_temp,
                "barcode": f"BC{container_id.upper().replace('-', '')}",
                "description": f"Hierarchical storage {container_type} {container_id}",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            },
            "children": children,
            "samples": samples,
            "capacity_info": {
                "total_capacity": 1000 if container_type == "freezer" else 100 if container_type == "rack" else 25 if container_type == "box" else 1,
                "used_capacity": len(children) if children else len(samples),
                "available_capacity": (1000 if container_type == "freezer" else 100 if container_type == "rack" else 25 if container_type == "box" else 1) - (len(children) if children else len(samples)),
                "utilization_percentage": ((len(children) if children else len(samples)) / (1000 if container_type == "freezer" else 100 if container_type == "rack" else 25 if container_type == "box" else 1)) * 100
            }
        }
    }, status_code=200)

def extract_parent_id(container_id: str) -> str:
    """Extract parent container ID from hierarchical container ID"""
    parts = container_id.split('-')
    if "box" in container_id:
        # box-rack-freezer-1-0-2 -> rack-freezer-1-0
        return '-'.join(parts[1:-1])
    elif "rack" in container_id:
        # rack-freezer-1-0 -> freezer-1
        return '-'.join(parts[1:-1])
    elif "pos" in container_id:
        # pos-box-rack-freezer-1-0-2-5 -> box-rack-freezer-1-0-2
        return '-'.join(parts[1:-1])
    return ""

@app.get("/api/storage/containers/{container_id}/grid")
async def get_storage_container_grid(container_id: str, include_empty: bool = Query(True)):
    """Get grid view for box-level storage containers"""
    
    # Only boxes have grid views
    if "box" not in container_id:
        raise HTTPException(status_code=404, detail="Grid view only available for box containers")
    
    # Generate 5x5 grid positions for the box
    positions = []
    for row in range(1, 6):  # 1-5 rows
        for col in range(1, 6):  # 1-5 columns
            position_id = f"pos-{container_id}-{(row-1)*5 + col-1}"
            position_identifier = f"{chr(64 + row)}{col}"  # A1, A2, B1, B2, etc.
            
            # First 15 positions are occupied, rest are available
            is_occupied = (row-1)*5 + col-1 < 15
            
            position = {
                "container_id": position_id,
                "position_identifier": position_identifier,
                "row": row,
                "column": col,
                "is_occupied": is_occupied,
                "temperature_zone": "-80C",
                "status": "occupied" if is_occupied else "available"
            }
            
            # Add sample info if occupied
            if is_occupied:
                sample_idx = (row-1)*5 + col-1
                position["sample"] = {
                    "id": f"sample-{container_id}-{sample_idx}",
                    "name": f"Sample {sample_idx + 1}",
                    "barcode": f"SMPL{sample_idx + 1:04d}",
                    "sample_type": "DNA" if sample_idx % 2 == 0 else "RNA",
                    "status": "stored",
                    "volume": 100.0 - sample_idx * 5,
                    "concentration": 50.0 + sample_idx * 2,
                    "stored_at": "2024-01-15T10:30:00Z"
                }
            
            positions.append(position)
    
    return JSONResponse(content={
        "data": {
            "container_id": container_id,
            "grid_dimensions": {
                "rows": 5,
                "columns": 5
            },
            "positions": positions,
            "total_positions": 25,
            "occupied_positions": 15,
            "available_positions": 10,
            "utilization_percentage": 60.0
        }
    }, status_code=200)

# Storage Service Proxy (forward to actual storage service)
storage_service_url = os.getenv("STORAGE_SERVICE_URL", "http://storage-service:8000")

@app.get("/api/storage/health")
async def storage_health():
    """Check storage service health"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{storage_service_url}/api/storage/health", timeout=5.0)
            return response.json()
    except Exception as e:
        return {"status": "unhealthy", "error": str(e)}, 503

@app.get("/api/storage/status")
async def storage_status():
    """Get storage service status"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{storage_service_url}/api/storage/status", timeout=5.0)
            return response.json()
    except Exception as e:
        return {"operational": False, "error": str(e)}, 503

# Reports endpoints
@app.get("/api/reports")
async def get_reports():
    """Get all generated reports from database"""
    try:
        async with get_database_connection() as conn:
            # Query all reports from the database
            rows = await conn.fetch("""
                SELECT 
                    id, definition_id, name, description, status, 
                    format, parameters, file_path, file_size, 
                    generated_by, started_at, completed_at, created_at
                FROM generated_reports
                ORDER BY created_at DESC
            """)
            
            # Convert rows to list of dicts
            reports = []
            for row in rows:
                report = dict(row)
                # Convert UUID to string for JSON serialization
                report['id'] = str(report['id'])
                if report['definition_id']:
                    report['definition_id'] = str(report['definition_id'])
                if report['generated_by']:
                    report['generated_by'] = str(report['generated_by'])
                # Convert dates to ISO format
                if report['started_at']:
                    report['started_at'] = report['started_at'].isoformat()
                if report['completed_at']:
                    report['completed_at'] = report['completed_at'].isoformat()
                if report['created_at']:
                    report['created_at'] = report['created_at'].isoformat()
                reports.append(report)
                
            return reports
    except Exception as e:
        print(f"Error fetching reports: {e}")
        return []

@app.get("/api/reports/templates")
async def get_report_templates():
    """Get comprehensive report templates covering all database tables."""
    templates = [
        # Sample Management Templates
        {
            "id": "RPT-001",
            "name": "Sample Status Summary",
            "description": "Overview of all samples grouped by status",
            "category": "samples",
            "sql": "SELECT status, COUNT(*) as count, \n       AVG(EXTRACT(EPOCH FROM (NOW() - created_at))/86400) as avg_age_days\nFROM samples \nGROUP BY status \nORDER BY count DESC;",
            "tags": ["samples", "status", "overview"]
        },
        {
            "id": "RPT-002",
            "name": "Sample Audit Trail",
            "description": "Recent sample audit activities and changes",
            "category": "samples",
            "sql": "SELECT s.barcode, s.name, sal.action, sal.old_values, sal.new_values, sal.performed_at, sal.performed_by\nFROM sample_audit_log sal\nJOIN samples s ON s.id = sal.sample_id\nWHERE sal.performed_at >= NOW() - INTERVAL '7 days'\nORDER BY sal.performed_at DESC\nLIMIT 100;",
            "tags": ["samples", "audit", "history"]
        },
        {
            "id": "RPT-003",
            "name": "Sample Validation Results",
            "description": "Quality control validation results for samples",
            "category": "samples",
            "sql": "SELECT s.barcode, s.name, svr.validation_status, svr.error_count, svr.warning_count, svr.validated_at\nFROM sample_validation_results svr\nJOIN samples s ON s.id = svr.sample_id\nWHERE svr.validated_at >= NOW() - INTERVAL '30 days'\nORDER BY svr.validated_at DESC;",
            "tags": ["samples", "validation", "quality"]
        },
        
        # Storage Management Templates
        {
            "id": "RPT-010",
            "name": "Storage Utilization by Zone",
            "description": "Current storage usage across temperature zones",
            "category": "storage",
            "sql": "SELECT sl.temperature_zone, \n       COUNT(*) as total_locations,\n       SUM(sl.capacity) as total_capacity,\n       SUM(sl.current_usage) as total_used,\n       ROUND(AVG(sl.current_usage::float / sl.capacity * 100), 2) as avg_utilization_pct\nFROM storage_locations sl\nGROUP BY sl.temperature_zone\nORDER BY avg_utilization_pct DESC;",
            "tags": ["storage", "utilization", "capacity"]
        },
        {
            "id": "RPT-011",
            "name": "Storage Container Hierarchy",
            "description": "Hierarchical view of storage containers and their contents",
            "category": "storage",
            "sql": "SELECT sc.name, sc.container_type, sc.parent_container_id, \n       COUNT(sp.id) as sample_positions,\n       sc.temperature_min, sc.temperature_max\nFROM storage_containers sc\nLEFT JOIN sample_positions sp ON sp.container_id = sc.id\nGROUP BY sc.id, sc.name, sc.container_type, sc.parent_container_id, sc.temperature_min, sc.temperature_max\nORDER BY sc.container_type, sc.name;",
            "tags": ["storage", "containers", "hierarchy"]
        },
        {
            "id": "RPT-012",
            "name": "Sample Position Tracking",
            "description": "Current positions of all stored samples",
            "category": "storage",
            "sql": "SELECT s.barcode, s.name, sc.name as container_name, \n       sp.position_x, sp.position_y, sp.position_z,\n       sp.stored_at, sp.last_accessed\nFROM sample_positions sp\nJOIN samples s ON s.id = sp.sample_id\nJOIN storage_containers sc ON sc.id = sp.container_id\nWHERE sp.status = 'occupied'\nORDER BY sp.stored_at DESC;",
            "tags": ["storage", "positions", "tracking"]
        },
        
        # User Management Templates
        {
            "id": "RPT-020",
            "name": "User Activity Summary",
            "description": "User login and activity patterns",
            "category": "users",
            "sql": "SELECT u.email, u.role, \n       COUNT(ual.id) as total_activities,\n       MAX(ual.timestamp) as last_activity,\n       COUNT(DISTINCT DATE(ual.timestamp)) as active_days\nFROM users u\nLEFT JOIN user_activity_log ual ON ual.user_id = u.id\nWHERE ual.timestamp >= NOW() - INTERVAL '30 days'\nGROUP BY u.id, u.email, u.role\nORDER BY total_activities DESC;",
            "tags": ["users", "activity", "analytics"]
        },
        {
            "id": "RPT-021",
            "name": "Security Audit Report",
            "description": "Security events and authentication attempts",
            "category": "security",
            "sql": "SELECT sal.event_type, sal.user_id, sal.ip_address, \n       sal.user_agent, sal.timestamp, sal.success\nFROM security_audit_log sal\nWHERE sal.timestamp >= NOW() - INTERVAL '7 days'\nORDER BY sal.timestamp DESC\nLIMIT 100;",
            "tags": ["security", "audit", "authentication"]
        },
        
        # Template Management Templates
        {
            "id": "RPT-030",
            "name": "Template Usage Statistics",
            "description": "Most frequently used laboratory templates",
            "category": "templates",
            "sql": "SELECT t.name, t.category, t.version,\n       COUNT(ti.id) as usage_count,\n       MAX(ti.created_at) as last_used,\n       COUNT(DISTINCT ti.created_by) as unique_users\nFROM templates t\nLEFT JOIN template_instances ti ON ti.template_id = t.id\nGROUP BY t.id, t.name, t.category, t.version\nORDER BY usage_count DESC;",
            "tags": ["templates", "usage", "statistics"]
        },
        {
            "id": "RPT-031",
            "name": "Template Field Analysis",
            "description": "Analysis of template fields and their usage patterns",
            "category": "templates",
            "sql": "SELECT tf.field_name, tf.field_type, tf.is_required,\n       COUNT(DISTINCT tf.template_id) as template_count,\n       AVG(CASE WHEN tf.is_required THEN 1 ELSE 0 END) as required_percentage\nFROM template_fields tf\nGROUP BY tf.field_name, tf.field_type, tf.is_required\nORDER BY template_count DESC, tf.field_name;",
            "tags": ["templates", "fields", "analysis"]
        },
        
        # Spreadsheet Analysis Templates
        {
            "id": "RPT-040",
            "name": "Spreadsheet Upload Summary",
            "description": "Overview of uploaded spreadsheet datasets",
            "category": "spreadsheets",
            "sql": "SELECT sd.name, sd.file_type, sd.total_rows, sd.total_columns,\n       sd.upload_status, sd.uploaded_by, sd.created_at\nFROM spreadsheet_datasets sd\nORDER BY sd.created_at DESC\nLIMIT 50;",
            "tags": ["spreadsheets", "uploads", "datasets"]
        },
        {
            "id": "RPT-041",
            "name": "Spreadsheet Data Analysis",
            "description": "Analysis results from spreadsheet processing",
            "category": "spreadsheets",
            "sql": "SELECT sa.dataset_id, sa.analysis_type, sa.result_summary,\n       sa.confidence_score, sa.created_at\nFROM spreadsheet_analysis sa\nJOIN spreadsheet_datasets sd ON sd.id = sa.dataset_id\nWHERE sa.created_at >= NOW() - INTERVAL '30 days'\nORDER BY sa.created_at DESC;",
            "tags": ["spreadsheets", "analysis", "ai"]
        },
        
        # Barcode Management Templates
        {
            "id": "RPT-050",
            "name": "Barcode Generation Report",
            "description": "Recently generated barcodes and their assignments",
            "category": "barcodes",
            "sql": "SELECT bs.prefix, bs.current_value, bs.increment_by,\n       COUNT(ba.id) as audit_entries,\n       MAX(ba.timestamp) as last_generated\nFROM barcode_sequences bs\nLEFT JOIN barcode_audit ba ON ba.sequence_id = bs.id\nGROUP BY bs.id, bs.prefix, bs.current_value, bs.increment_by\nORDER BY last_generated DESC NULLS LAST;",
            "tags": ["barcodes", "generation", "tracking"]
        },
        
        # System Performance Templates
        {
            "id": "RPT-060",
            "name": "Database Table Statistics",
            "description": "Row counts and storage usage for all tables",
            "category": "system",
            "sql": "SELECT schemaname, tablename, n_tup_ins as inserts, n_tup_upd as updates, \n       n_tup_del as deletes, n_live_tup as live_rows, n_dead_tup as dead_rows\nFROM pg_stat_user_tables\nORDER BY n_live_tup DESC;",
            "tags": ["system", "performance", "statistics"]
        },
        {
            "id": "RPT-061",
            "name": "Rate Limiting Statistics",
            "description": "API rate limiting usage and patterns",
            "category": "system",
            "sql": "SELECT rl.key, rl.limit_value, rl.current_count, rl.reset_time,\n       CASE WHEN rl.current_count >= rl.limit_value THEN 'EXCEEDED' ELSE 'OK' END as status\nFROM rate_limits rl\nWHERE rl.reset_time > NOW()\nORDER BY rl.current_count DESC;",
            "tags": ["system", "rate-limiting", "api"]
        },
        
        # Advanced Analytics Templates
        {
            "id": "RPT-070",
            "name": "Cross-Table Relationship Analysis",
            "description": "Relationships between samples, storage, and templates",
            "category": "analytics",
            "sql": "SELECT \n    COUNT(DISTINCT s.id) as total_samples,\n    COUNT(DISTINCT sp.container_id) as containers_used,\n    COUNT(DISTINCT ti.template_id) as templates_used,\n    COUNT(DISTINCT u.id) as active_users\nFROM samples s\nLEFT JOIN sample_positions sp ON sp.sample_id = s.id\nLEFT JOIN template_instances ti ON ti.created_at >= s.created_at - INTERVAL '1 day' AND ti.created_at <= s.created_at + INTERVAL '1 day'\nLEFT JOIN users u ON u.id = s.created_by\nWHERE s.created_at >= NOW() - INTERVAL '30 days';",
            "tags": ["analytics", "relationships", "overview"]
        },
        {
            "id": "RPT-071",
            "name": "Temporal Activity Patterns",
            "description": "Activity patterns by hour of day and day of week",
            "category": "analytics", 
            "sql": "SELECT \n    EXTRACT(DOW FROM created_at) as day_of_week,\n    EXTRACT(HOUR FROM created_at) as hour_of_day,\n    COUNT(*) as activity_count\nFROM (\n    SELECT created_at FROM samples\n    UNION ALL\n    SELECT created_at FROM template_instances\n    UNION ALL\n    SELECT timestamp as created_at FROM user_activity_log\n) activities\nWHERE created_at >= NOW() - INTERVAL '30 days'\nGROUP BY day_of_week, hour_of_day\nORDER BY day_of_week, hour_of_day;",
            "tags": ["analytics", "temporal", "patterns"]
        }
    ]
    
    # Add metadata
    for template in templates:
        template["created_at"] = datetime.now().isoformat()
        template["updated_at"] = datetime.now().isoformat()
    
    # Group templates by category
    categories = {}
    for template in templates:
        category = template["category"]
        if category not in categories:
            categories[category] = []
        categories[category].append(template)
    
    return {
        "data": templates,
        "templates": templates,  # For backward compatibility
        "categories": categories,
        "totalCount": len(templates),
        "last_updated": datetime.now().isoformat()
    }

@app.get("/api/reports/schema")
async def get_database_schema():
    """Get comprehensive database schema information from actual database."""
    try:
        async with get_database_connection() as conn:
            # Get all tables in the public schema
            tables_query = """
                SELECT table_name, table_type, 
                       obj_description(c.oid) as table_comment
                FROM information_schema.tables t
                LEFT JOIN pg_class c ON c.relname = t.table_name
                WHERE table_schema = 'public' 
                AND table_type = 'BASE TABLE'
                ORDER BY table_name
            """
            tables = await conn.fetch(tables_query)
            
            schema_tables = []
            for table in tables:
                table_name = table['table_name']
                
                # Get columns for this table
                columns_query = """
                    SELECT 
                        column_name,
                        data_type,
                        is_nullable,
                        column_default,
                        character_maximum_length,
                        numeric_precision,
                        numeric_scale,
                        col_description(pgc.oid, ordinal_position) as column_comment
                    FROM information_schema.columns c
                    LEFT JOIN pg_class pgc ON pgc.relname = c.table_name
                    WHERE table_schema = 'public' 
                    AND table_name = $1
                    ORDER BY ordinal_position
                """
                columns = await conn.fetch(columns_query, table_name)
                
                # Get primary key information
                pk_query = """
                    SELECT column_name
                    FROM information_schema.table_constraints tc
                    JOIN information_schema.key_column_usage kcu 
                        ON tc.constraint_name = kcu.constraint_name
                    WHERE tc.table_schema = 'public'
                    AND tc.table_name = $1
                    AND tc.constraint_type = 'PRIMARY KEY'
                """
                primary_keys = await conn.fetch(pk_query, table_name)
                pk_columns = [pk['column_name'] for pk in primary_keys]
                
                # Get foreign key information
                fk_query = """
                    SELECT 
                        kcu.column_name,
                        ccu.table_name AS foreign_table_name,
                        ccu.column_name AS foreign_column_name
                    FROM information_schema.table_constraints tc
                    JOIN information_schema.key_column_usage kcu 
                        ON tc.constraint_name = kcu.constraint_name
                    JOIN information_schema.constraint_column_usage ccu 
                        ON ccu.constraint_name = tc.constraint_name
                    WHERE tc.table_schema = 'public'
                    AND tc.table_name = $1
                    AND tc.constraint_type = 'FOREIGN KEY'
                """
                foreign_keys = await conn.fetch(fk_query, table_name)
                fk_map = {fk['column_name']: {
                    'references_table': fk['foreign_table_name'],
                    'references_column': fk['foreign_column_name']
                } for fk in foreign_keys}
                
                # Get table row count
                try:
                    count_result = await conn.fetchval(f'SELECT COUNT(*) FROM "{table_name}"')
                    row_count = count_result
                except:
                    row_count = 0
                
                # Format columns
                formatted_columns = []
                for col in columns:
                    column_info = {
                        "name": col['column_name'],
                        "data_type": col['data_type'],
                        "type": col['data_type'],  # For backward compatibility
                        "is_nullable": col['is_nullable'] == 'YES',
                        "nullable": col['is_nullable'] == 'YES',  # For backward compatibility
                        "is_primary_key": col['column_name'] in pk_columns,
                        "default": col['column_default'],
                        "comment": col['column_comment']
                    }
                    
                    # Add length/precision info
                    if col['character_maximum_length']:
                        column_info['max_length'] = col['character_maximum_length']
                    if col['numeric_precision']:
                        column_info['precision'] = col['numeric_precision']
                    if col['numeric_scale']:
                        column_info['scale'] = col['numeric_scale']
                    
                    # Add foreign key info
                    if col['column_name'] in fk_map:
                        column_info['foreign_key'] = fk_map[col['column_name']]
                    
                    formatted_columns.append(column_info)
                
                schema_tables.append({
                    "name": table_name,
                    "type": table['table_type'],
                    "comment": table['table_comment'],
                    "row_count": row_count,
                    "columns": formatted_columns,
                    "primary_keys": pk_columns
                })
            
            return {
                "tables": schema_tables,
                "database_name": "lims_db",
                "total_tables": len(schema_tables),
                "last_updated": datetime.now().isoformat()
            }
            
    except Exception as e:
        print(f"Error fetching database schema: {e}")
        # Fallback to basic schema
        return {
            "tables": [
                {
                    "name": "samples",
                    "columns": [
                        {"name": "id", "type": "uuid", "nullable": False, "is_primary_key": True},
                        {"name": "name", "type": "varchar", "nullable": False},
                        {"name": "type", "type": "varchar", "nullable": False},
                        {"name": "status", "type": "varchar", "nullable": False},
                        {"name": "created_at", "type": "timestamp", "nullable": False},
                        {"name": "updated_at", "type": "timestamp", "nullable": False}
                    ]
                }
            ],
            "error": f"Could not fetch schema: {str(e)}"
        }

@app.post("/api/reports/execute")
async def execute_report(request: Request):
    """Execute a custom SQL report safely with read-only queries."""
    try:
        body = await request.json()
        sql_query = body.get("sql", "").strip()
        
        if not sql_query:
            raise HTTPException(status_code=400, detail="SQL query is required")
        
        # Security: Only allow SELECT statements
        sql_lower = sql_query.lower().strip()
        if not sql_lower.startswith('select'):
            raise HTTPException(
                status_code=403, 
                detail="Only SELECT queries are allowed for security reasons"
            )
        
        # Additional security checks
        forbidden_keywords = ['insert', 'update', 'delete', 'drop', 'create', 'alter', 'truncate', 'grant', 'revoke']
        for keyword in forbidden_keywords:
            if keyword in sql_lower:
                raise HTTPException(
                    status_code=403,
                    detail=f"Forbidden keyword '{keyword}' detected. Only SELECT queries are allowed."
                )
        
        start_time = time.time()
        
        async with get_database_connection() as conn:
            # Set a query timeout for safety
            await conn.execute("SET statement_timeout = '30s'")
            
            # Execute the query
            rows = await conn.fetch(sql_query)
            
            execution_time = (time.time() - start_time) * 1000  # Convert to milliseconds
            
            # Convert rows to list of dicts
            data = []
            columns = []
            
            if rows:
                # Get column names from the first row
                columns = list(rows[0].keys())
                
                # Convert each row to a dict
                for row in rows:
                    row_dict = {}
                    for col in columns:
                        value = row[col]
                        # Convert special types to JSON serializable format
                        if hasattr(value, 'isoformat'):  # datetime objects
                            value = value.isoformat()
                        elif isinstance(value, (bytes, memoryview)):
                            value = str(value)
                        elif value is None:
                            value = None
                        else:
                            value = value
                        row_dict[col] = value
                    data.append(row_dict)
            
            return {
                "success": True,
                "data": data,
                "rows": data,  # For backward compatibility
                "columns": columns,
                "row_count": len(data),
                "rowCount": len(data),  # For backward compatibility
                "execution_time_ms": round(execution_time, 2),
                "executionTime": round(execution_time, 2),  # For backward compatibility
                "query": sql_query,
                "timestamp": datetime.now().isoformat()
            }
            
    except HTTPException:
        # Re-raise HTTP exceptions
        raise
    except Exception as e:
        error_message = str(e)
        print(f"SQL execution error: {error_message}")
        
        # Return a proper error response
        raise HTTPException(
            status_code=500,
            detail=f"Query execution failed: {error_message}"
        )

# RAG Service endpoints
@app.get("/api/rag/submissions")
async def get_rag_submissions():
    """Get RAG submissions"""
    submissions_data = [
        {
            "id": "RAG-001",
            "filename": "lab_report_2024_01.pdf",
            "status": "Processed",
            "submittedDate": (datetime.now() - timedelta(days=2)).isoformat(),
            "processedDate": (datetime.now() - timedelta(days=2, hours=2)).isoformat(),
            "extractedFields": 15,
            "confidenceScore": 0.92,
            "submittedBy": "Dr. Smith"
        },
        {
            "id": "RAG-002",
            "filename": "sample_manifest_batch_47.xlsx",
            "status": "Processing",
            "submittedDate": (datetime.now() - timedelta(hours=4)).isoformat(),
            "extractedFields": 8,
            "confidenceScore": 0.87,
            "submittedBy": "Lab Tech Johnson"
        },
        {
            "id": "RAG-003",
            "filename": "quality_control_summary.docx",
            "status": "Pending",
            "submittedDate": (datetime.now() - timedelta(minutes=30)).isoformat(),
            "submittedBy": "Dr. Williams"
        }
    ]

    # Return both formats for compatibility
    return {
        "data": submissions_data,  # For frontend expecting .data.filter()
        "submissions": submissions_data,  # For other consumers
        "totalCount": 127,
        "processing": 3,
        "completed": 118,
        "failed": 6
    }

@app.get("/api/rag/submissions/{submission_id}")
async def get_rag_submission_detail(submission_id: str):
    """Get detailed RAG submission information"""
    # Mock detailed submission data
    submission_details = {
        "RAG-001": {
            "id": "RAG-001",
            "submission_id": "RAG-001",
            "source_document": "lab_report_2024_01.pdf",
            "submitter_name": "Dr. Smith",
            "submitter_email": "dr.smith@lab.com",
            "confidence_score": 0.92,
            "processing_time": 2.3,
            "created_at": (datetime.now() - timedelta(days=2)).isoformat(),
            "status": "Processed",
            "samples_created": 15,
            "extracted_data": {
                "administrative_info": {
                    "submitter_name": "Dr. Smith",
                    "submitter_email": "dr.smith@lab.com",
                    "project_name": "Cancer Research Study 2024",
                    "institution": "Advanced Medical Research Lab"
                },
                "source_material": {
                    "sample_type": "Blood",
                    "source_organism": "Human",
                    "collection_date": "2024-01-15",
                    "collection_method": "Venipuncture"
                },
                "sample_details": {
                    "sample_count": 15,
                    "volume_per_sample": "5ml",
                    "container_type": "EDTA tubes",
                    "storage_temperature": "-80°C"
                }
            }
        },
        "RAG-002": {
            "id": "RAG-002",
            "submission_id": "RAG-002", 
            "source_document": "sample_manifest_batch_47.xlsx",
            "submitter_name": "Lab Tech Johnson",
            "submitter_email": "johnson@lab.com",
            "confidence_score": 0.87,
            "processing_time": 1.8,
            "created_at": (datetime.now() - timedelta(hours=4)).isoformat(),
            "status": "Processing",
            "samples_created": 8,
            "extracted_data": {
                "administrative_info": {
                    "submitter_name": "Lab Tech Johnson",
                    "submitter_email": "johnson@lab.com",
                    "project_name": "Batch 47 Processing",
                    "institution": "Clinical Testing Laboratory"
                },
                "source_material": {
                    "sample_type": "Tissue",
                    "source_organism": "Human",
                    "collection_date": "2024-06-28"
                }
            }
        },
        "RAG-003": {
            "id": "RAG-003",
            "submission_id": "RAG-003",
            "source_document": "quality_control_summary.docx",
            "submitter_name": "Dr. Williams",
            "submitter_email": "williams@lab.com",
            "confidence_score": 0.75,
            "processing_time": 0.0,
            "created_at": (datetime.now() - timedelta(minutes=30)).isoformat(),
            "status": "Pending",
            "samples_created": 0,
            "extracted_data": {}
        }
    }
    
    if submission_id not in submission_details:
        raise HTTPException(status_code=404, detail="Submission not found")
    
    return submission_details[submission_id]

@app.post("/api/rag/process")
async def process_rag_document(request: Request):
    """Process a document with RAG system"""
    try:
        # Handle multipart form data (file upload)
        form = await request.form()
        
        # Check if file is present
        file = form.get("file")
        if not file:
            raise HTTPException(status_code=400, detail="No file uploaded")
        
        # Get processing parameters
        auto_create_str = form.get("auto_create", "false")
        auto_create = str(auto_create_str).lower() == "true" if auto_create_str else False
        
        confidence_threshold_str = form.get("confidence_threshold", "0.8")
        confidence_threshold = float(str(confidence_threshold_str)) if confidence_threshold_str else 0.8
        
        # Generate document ID
        document_id = f"DOC-{datetime.now().strftime('%Y%m%d%H%M%S')}"
        
        # Mock extraction result (in production, this would process the actual file)
        extraction_result = {
            "success": True,
            "id": document_id,
            "status": "completed",
            "message": "Document processed successfully",
            "confidence_score": 0.92,
            "processing_time": 2.5,
            "samples": [
                {
                    "name": "Sample 001 from document",
                    "barcode": f"BC-{datetime.now().strftime('%Y%m%d')}-001",
                    "location": "Freezer A1",
                    "metadata": {
                        "type": "DNA",
                        "volume": "100 μL",
                        "concentration": "50 ng/μL"
                    }
                },
                {
                    "name": "Sample 002 from document", 
                    "barcode": f"BC-{datetime.now().strftime('%Y%m%d')}-002",
                    "location": "Freezer A2",
                    "metadata": {
                        "type": "RNA",
                        "volume": "50 μL",
                        "concentration": "75 ng/μL"
                    }
                }
            ],
            "validation_warnings": [] if confidence_threshold <= 0.9 else ["Some fields extracted with lower confidence"],
            "extraction_result": {
                "success": True,
                "confidence_score": 0.92,
                "warnings": [],
                "source_document": getattr(file, 'filename', 'uploaded_document')
            }
        }
        
        return extraction_result

    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=400, detail=f"Error processing document: {e!s}")

@app.post("/api/samples/rag/query")
async def query_rag_samples(request: Request):
    """Query RAG system for sample information"""
    try:
        # Get the request body
        body = await request.json()
        query = body.get("query", "")
        
        # Add logging
        print(f"[RAG QUERY] Received query: '{query}'", file=sys.stderr)
        
        # Mock RAG response based on query
        query_lower = query.lower()
        
        # Initialize related_samples
        related_samples = []
        
        # Check for AI document processing related queries
        ai_keywords = ["ai document", "document processing", "ai processing", "rag submission", "upload document"]
        is_ai_query = any(keyword in query_lower for keyword in ai_keywords)
        is_submit_query = "submit" in query_lower and ("sample" in query_lower or "document" in query_lower)
        
        print(f"[RAG QUERY] is_ai_query={is_ai_query}, is_submit_query={is_submit_query}", file=sys.stderr)
        
        if is_ai_query or is_submit_query:
            response_text = """To submit a new sample using AI document processing:

1. **Navigate to RAG Submissions**: Click on 'AI Document Submissions' from the main dashboard or go to the RAG Submissions page.

2. **Upload Your Document**: 
   - Drag and drop your document (PDF, DOCX, or TXT) into the upload area
   - Or click "Upload a file" to browse and select your document
   - Files up to 50MB are supported

3. **Configure Processing Options**:
   - Set the confidence threshold (default: 0.8) - lower values accept more extracted data
   - Check "Automatically create samples" if you want samples created immediately after extraction

4. **Process the Document**:
   - Click "Preview" to see what will be extracted without creating samples
   - Click "Process & Extract" to extract and create samples

5. **Review Results**:
   - The AI will extract sample information including names, barcodes, locations, and metadata
   - Check the confidence scores and any validation warnings
   - If in preview mode, you can "Confirm & Create Samples" after review

The AI system uses advanced language models to understand laboratory documents and extract structured data automatically. This saves time compared to manual data entry and reduces errors."""
        elif "samples" in query.lower():
            response_text = f"Based on your query '{query}', I found information about laboratory samples. Currently, there are 1,247 samples in the system with 89 active samples and 1,158 completed samples. The most recent samples were submitted by Dr. Smith and include DNA, RNA, and protein samples."
            related_samples = [
                {
                    "id": "SMPL-001",
                    "name": "Sample 001",
                    "type": "DNA",
                    "status": "Processing",
                    "submittedBy": "Dr. Smith",
                    "relevance": 0.95
                },
                {
                    "id": "SMPL-002",
                    "name": "Sample 002",
                    "type": "RNA",
                    "status": "Completed",
                    "submittedBy": "Dr. Johnson",
                    "relevance": 0.88
                }
            ]
        elif "storage" in query.lower():
            response_text = f"Regarding storage for '{query}', we have multiple storage locations including Freezer A1 (-80°C) with 750/1000 capacity and Refrigerator B2 (4°C) with 320/500 capacity. All storage units are operating normally."
            related_samples = [
                {
                    "id": "STOR-001",
                    "name": "Freezer A1",
                    "status": "Normal",
                    "relevance": 0.90
                }
            ]
        elif "sequencing" in query.lower():
            response_text = f"For sequencing information related to '{query}', there are currently 12 active sequencing jobs with 45 total jobs in the system. The most recent job is 'Whole Genome Sequencing - Batch 1' running at 67% progress on Illumina NovaSeq."
            related_samples = [
                {
                    "id": "SEQ-001",
                    "name": "Whole Genome Sequencing - Batch 1",
                    "status": "Running",
                    "progress": 67,
                    "relevance": 0.93
                }
            ]
        else:
            response_text = f"I understand you're asking about '{query}'. While I don't have specific information about this topic, I can help you with questions about samples, storage, sequencing, or other laboratory management topics."
            related_samples = []

        query_result = {
            "id": f"QRES-{datetime.now().strftime('%Y%m%d%H%M%S')}",
            "query": query,
            "response": response_text,
            "confidence": 0.85,
            "sources": [
                {
                    "document": "laboratory_database",
                    "section": "current_status",
                    "relevance": 0.92
                }
            ],
            "relatedItems": related_samples,
            "timestamp": datetime.now().isoformat()
        }

        # Return both single result and data array format for compatibility
        return {
            "data": [query_result],  # For frontend expecting .data.filter()
            "result": query_result,  # Single result for other consumers
            "relatedSamples": related_samples  # Direct access to related items
        }

    except Exception as e:
        raise HTTPException(status_code=400, detail=f"Invalid query: {e!s}")

@app.post("/api/rag/submissions")
async def create_rag_submission(request: Request):
    """Create a new RAG submission"""
    return {
        "id": f"RAG-{datetime.now().strftime('%Y%m%d%H%M%S')}",
        "status": "submitted",
        "message": "Document submitted for processing"
    }

# Debug endpoint to test query logic
@app.post("/api/debug/test-query")
async def test_query(request: Request):
    """Test query logic"""
    try:
        body = await request.json()
        query = body.get("query", "")
        query_lower = query.lower()
        
        ai_keywords = ["ai document", "document processing", "ai processing", "rag submission", "upload document"]
        is_ai_query = any(keyword in query_lower for keyword in ai_keywords)
        is_submit_query = "submit" in query_lower and ("sample" in query_lower or "document" in query_lower)
        
        return {
            "query": query,
            "query_lower": query_lower,
            "is_ai_query": is_ai_query,
            "is_submit_query": is_submit_query,
            "should_match": is_ai_query or is_submit_query,
            "ai_keyword_matches": [kw for kw in ai_keywords if kw in query_lower]
        }
    except Exception as e:
        return {"error": str(e)}

# Debug endpoint to see all registered routes
@app.get("/api/debug/routes")
async def debug_routes():
    """Debug endpoint to see all registered routes"""
    from fastapi.routing import APIRoute
    routes = []
    for route in app.routes:
        if isinstance(route, APIRoute):
            routes.append({
                "path": route.path,
                "methods": list(route.methods) if route.methods else ["GET"]
            })
    return {"routes": sorted(routes, key=lambda x: x["path"])}

# Note: Removed catch-all endpoint to prevent route conflicts

@app.get("/api/v1/status")
async def api_status():
    """API status endpoint"""
    services = {
        "api-gateway": "healthy",
        "rag-service": "unknown"
    }

    # Check RAG service health
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{RAG_SERVICE_URL}/api/v1/health", timeout=5.0)
            if response.status_code == 200:
                services["rag-service"] = "healthy"
            else:
                services["rag-service"] = "unhealthy"
    except Exception:
        services["rag-service"] = "unreachable"

    return {
        "services": services,
        "overall": "healthy" if all(s in ["healthy", "unknown"] for s in services.values()) else "degraded"
    }

@app.api_route("/api/v1/rag/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_rag(path: str, request: Request):
    """Proxy requests to RAG service"""
    try:
        async with httpx.AsyncClient() as client:
            # Forward the request to RAG service
            url = f"{RAG_SERVICE_URL}/{path}"

            # Get request body if present
            body = None
            if request.method in ["POST", "PUT", "PATCH"]:
                body = await request.body()

            response = await client.request(
                method=request.method,
                url=url,
                headers=dict(request.headers),
                params=request.query_params,
                content=body,
                timeout=30.0
            )

            return response.json() if response.headers.get("content-type", "").startswith("application/json") else response.text

    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Service unavailable: {e!s}")

# Proxy routes for other services
@app.api_route("/api/auth/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_auth(path: str, request: Request):
    """Proxy requests to Auth service"""
    try:
        async with httpx.AsyncClient() as client:
            url = f"{AUTH_SERVICE_URL}/auth/{path}"
            body = None
            if request.method in ["POST", "PUT", "PATCH"]:
                body = await request.body()

            response = await client.request(
                method=request.method,
                url=url,
                headers=dict(request.headers),
                params=request.query_params,
                content=body,
                timeout=30.0
            )
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Auth service unavailable: {e!s}")

@app.api_route("/api/sequencing/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_sequencing(path: str, request: Request):
    """Proxy requests to Sequencing service"""
    try:
        async with httpx.AsyncClient() as client:
            # For health checks, use the direct health endpoint
            if path == "health":
                url = f"{SEQUENCING_SERVICE_URL}/health"
            else:
                url = f"{SEQUENCING_SERVICE_URL}/api/v1/{path}"
            body = None
            if request.method in ["POST", "PUT", "PATCH"]:
                body = await request.body()

            response = await client.request(
                method=request.method,
                url=url,
                headers=dict(request.headers),
                params=request.query_params,
                content=body,
                timeout=30.0
            )
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Sequencing service unavailable: {e!s}")

@app.api_route("/api/notification/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_notification(path: str, request: Request):
    """Proxy requests to Notification service"""
    try:
        async with httpx.AsyncClient() as client:
            url = f"{NOTIFICATION_SERVICE_URL}/api/v1/{path}"
            body = None
            if request.method in ["POST", "PUT", "PATCH"]:
                body = await request.body()

            response = await client.request(
                method=request.method,
                url=url,
                headers=dict(request.headers),
                params=request.query_params,
                content=body,
                timeout=30.0
            )
            return response.json()
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Notification service unavailable: {e!s}")

# Enhanced services proxy routes
@app.api_route("/api/notifications/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_notifications(path: str, request: Request):
    """Proxy requests to Notifications service (plural)"""
    notification_url = "http://lims-notification:8000"
    try:
        async with httpx.AsyncClient() as client:
            # For health checks, use the direct health endpoint
            if path == "health":
                url = f"{notification_url}/health"
            else:
                url = f"{notification_url}/api/v1/{path}"
            
            body = None
            if request.method in ["POST", "PUT", "PATCH"]:
                body = await request.body()

            response = await client.request(
                method=request.method,
                url=url,
                headers=dict(request.headers),
                params=request.query_params,
                content=body,
                timeout=30.0
            )
            
            # Return the response content and status
            return Response(
                content=response.content,
                status_code=response.status_code,
                headers=dict(response.headers)
            )
    except httpx.ConnectError:
        raise HTTPException(status_code=503, detail="Notifications service unavailable")
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Notifications service error: {e!s}")

@app.api_route("/api/events/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_events(path: str, request: Request):
    """Proxy requests to Events service"""
    events_url = EVENT_SERVICE_URL
    try:
        async with httpx.AsyncClient() as client:
            # For health checks, use the direct health endpoint
            if path == "health":
                url = f"{events_url}/health"
            else:
                url = f"{events_url}/api/v1/{path}"
            
            body = None
            if request.method in ["POST", "PUT", "PATCH"]:
                body = await request.body()

            response = await client.request(
                method=request.method,
                url=url,
                headers=dict(request.headers),
                params=request.query_params,
                content=body,
                timeout=30.0
            )
            
            # Return the response content and status
            return Response(
                content=response.content,
                status_code=response.status_code,
                headers=dict(response.headers)
            )
    except httpx.ConnectError:
        raise HTTPException(status_code=503, detail="Events service unavailable")
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Events service error: {e!s}")

@app.api_route("/api/transactions/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_transactions(path: str, request: Request):
    """Proxy requests to Transactions service"""
    try:
        async with httpx.AsyncClient() as client:
            # For health checks, use the direct health endpoint
            if path == "health":
                url = f"{TRANSACTION_SERVICE_URL}/health"
            else:
                url = f"{TRANSACTION_SERVICE_URL}/api/v1/{path}"
            
            body = None
            if request.method in ["POST", "PUT", "PATCH"]:
                body = await request.body()

            response = await client.request(
                method=request.method,
                url=url,
                headers=dict(request.headers),
                params=request.query_params,
                content=body,
                timeout=30.0
            )
            
            # Return the response content and status
            return Response(
                content=response.content,
                status_code=response.status_code,
                headers=dict(response.headers)
            )
    except httpx.ConnectError:
        raise HTTPException(status_code=503, detail="Transactions service unavailable")
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Transactions service error: {e!s}")

@app.api_route("/api/qaqc/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_qaqc(path: str, request: Request):
    """Proxy requests to QA/QC service"""
    # Mock health check for QA/QC while the service is being fixed
    if path == "health" and request.method == "GET":
        return JSONResponse({
            "service": "qaqc-service",
            "status": "unavailable",
            "message": "Service binary issue - being fixed",
            "timestamp": datetime.now().isoformat()
        }, status_code=503)
    
    try:
        async with httpx.AsyncClient() as client:
            # For health checks, use the direct health endpoint
            if path == "health":
                url = f"{QAQC_SERVICE_URL}/health"
            else:
                url = f"{QAQC_SERVICE_URL}/api/v1/{path}"
            body = None
            if request.method in ["POST", "PUT", "PATCH"]:
                body = await request.body()

            response = await client.request(
                method=request.method,
                url=url,
                headers=dict(request.headers),
                params=request.query_params,
                content=body,
                timeout=30.0
            )
            return Response(
                content=response.content,
                status_code=response.status_code,
                headers=dict(response.headers),
                media_type=response.headers.get("content-type", "application/json")
            )
    except Exception as e:
        print(f"Error proxying to QA/QC service: {e}")
        return JSONResponse({"error": "Service unavailable"}, status_code=503)

@app.api_route("/api/templates/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_templates(path: str, request: Request):
    """Proxy requests to Templates service"""
    try:
        print(f"Template proxy: path={path}, method={request.method}")
        print(f"Headers: {dict(request.headers)}")
        
        async with httpx.AsyncClient() as client:
            # For health checks, use the direct health endpoint
            if path == "health":
                url = f"{TEMPLATE_SERVICE_URL}/health"
            else:
                # Template service expects routes without /api/v1 prefix
                url = f"{TEMPLATE_SERVICE_URL}/templates/{path}"
            
            print(f"Proxying to: {url}")
            
            # Handle multipart form data for uploads
            if path == "upload" and request.method == "POST":
                # For file uploads, we need to handle multipart/form-data
                content_type = request.headers.get("content-type", "")
                print(f"Content-Type: {content_type}")
                
                if content_type.startswith("multipart/form-data"):
                    # Read the raw body and forward it with the same content-type
                    body = await request.body()
                    print(f"Body length: {len(body)}")
                    
                    headers = dict(request.headers)
                    # Remove host header to avoid conflicts
                    headers.pop("host", None)
                    headers.pop("content-length", None)
                    
                    print("Forwarding multipart request...")
                    response = await client.request(
                        method=request.method,
                        url=url,
                        headers=headers,
                        content=body,
                        timeout=30.0
                    )
                    print(f"Response status: {response.status_code}")
                else:
                    # Non-multipart POST request
                    print("Not multipart, handling as regular POST")
                    body = await request.body()
                    response = await client.request(
                        method=request.method,
                        url=url,
                        headers=dict(request.headers),
                        params=request.query_params,
                        content=body,
                        timeout=30.0
                    )
            else:
                # For non-upload requests, handle normally
                body = None
                if request.method in ["POST", "PUT", "PATCH"]:
                    body = await request.body()

                response = await client.request(
                    method=request.method,
                    url=url,
                    headers=dict(request.headers),
                    params=request.query_params,
                    content=body,
                    timeout=30.0
                )
            
            return Response(
                content=response.content,
                status_code=response.status_code,
                headers=dict(response.headers),
                media_type=response.headers.get("content-type", "application/json")
            )
    except Exception as e:
        print(f"Error proxying to Templates service: {e}")
        import traceback
        traceback.print_exc()
        return JSONResponse({"error": "Service unavailable"}, status_code=503)

# NOTE: Removed proxy endpoints - using direct endpoints below
# Project endpoints
@app.get("/api/projects")
async def get_projects():
    """Get all projects from database"""
    try:
        async with get_database_connection() as conn:
            # Query all projects from the database
            rows = await conn.fetch("""
                SELECT 
                    id, project_code, name, description, project_type, 
                    status, priority, start_date, target_end_date, 
                    principal_investigator_id, project_manager_id, 
                    department, budget_approved, budget_used, metadata,
                    created_at, updated_at
                FROM projects
                ORDER BY created_at DESC
            """)
            
            # Convert rows to list of dicts
            projects = []
            for row in rows:
                project = dict(row)
                # Convert UUID to string for JSON serialization
                project['id'] = str(project['id'])
                if project['principal_investigator_id']:
                    project['principal_investigator_id'] = str(project['principal_investigator_id'])
                if project['project_manager_id']:
                    project['project_manager_id'] = str(project['project_manager_id'])
                # Convert dates to ISO format
                if project['start_date']:
                    project['start_date'] = project['start_date'].isoformat()
                if project['target_end_date']:
                    project['target_end_date'] = project['target_end_date'].isoformat()
                if project['created_at']:
                    project['created_at'] = project['created_at'].isoformat()
                if project['updated_at']:
                    project['updated_at'] = project['updated_at'].isoformat()
                projects.append(project)
                
            return projects
    except Exception as e:
        print(f"Error fetching projects: {e}")
        return []

@app.get("/api/projects/batches")
async def get_batches():
    """Get all batches"""
    # Return empty list for now
    return []

@app.post("/api/projects")
async def create_project(project: dict):
    """Create a new project"""
    # In production, this would save to database
    return {"id": str(uuid.uuid4()), **project}

@app.get("/api/projects/{project_id}")
async def get_project(project_id: str):
    """Get a specific project"""
    # Return mock project
    return {
        "id": project_id,
        "project_code": "PROJ-2024-001",
        "name": "Sample Project",
        "status": "active",
        "priority": "high",
        "created_at": datetime.now().isoformat()
    }

@app.get("/api/projects/{project_id}/files")
async def get_project_files(project_id: str):
    """Get project files"""
    return []

@app.get("/api/projects/{project_id}/signoffs")
async def get_project_signoffs(project_id: str):
    """Get project signoffs"""
    return []

# NOTE: Removed proxy endpoints for library prep - using direct endpoints below
# Library Prep endpoints
@app.get("/api/library-prep/preparations")
async def get_library_preparations():
    """Get library preparations"""
    return []

@app.get("/api/library-prep/protocols/active")
async def get_active_protocols():
    """Get active library prep protocols"""
    return []

@app.post("/api/library-prep/preparations")
async def create_library_prep(prep: dict):
    """Create a new library preparation"""
    return {"id": str(uuid.uuid4()), **prep}

@app.get("/api/library-prep/protocols")
async def get_protocols():
    """Get all library prep protocols"""
    return []

# Proxy endpoints for QC
# QC endpoints
@app.get("/api/qc/reviews")
async def get_qc_reviews():
    """Get QC reviews"""
    return []

@app.get("/api/qc/metrics")
async def get_qc_metrics():
    """Get QC metrics"""
    return []

@app.get("/api/qc/control-samples")
async def get_control_samples():
    """Get control samples"""
    return []

@app.post("/api/qc/reviews")
async def create_qc_review(review: dict):
    """Create a new QC review"""
    return {"id": str(uuid.uuid4()), **review}

@app.get("/api/qc/dashboard/stats")
async def get_qc_dashboard_stats():
    """Get QC dashboard statistics"""
    return {
        "totalSamples": 1247,
        "passedQC": 1156,
        "failedQC": 91,
        "pendingQC": 0,
        "passRate": 92.7,
        "recentActivity": [
            {
                "id": "QC-001",
                "sampleId": "SMPL-001",
                "status": "passed",
                "timestamp": (datetime.now() - timedelta(hours=1)).isoformat(),
                "reviewer": "Dr. Smith"
            },
            {
                "id": "QC-002", 
                "sampleId": "SMPL-002",
                "status": "failed",
                "timestamp": (datetime.now() - timedelta(hours=2)).isoformat(),
                "reviewer": "Lab Tech Johnson"
            }
        ]
    }

@app.get("/api/qc/metrics/recent")
async def get_qc_metrics_recent():
    """Get recent QC metrics"""
    return {
        "metrics": [
            {
                "id": "QCM-001",
                "sampleId": "SMPL-001",
                "concentration": 50.2,
                "purity": 1.85,
                "qualityScore": 95,
                "timestamp": (datetime.now() - timedelta(hours=1)).isoformat()
            },
            {
                "id": "QCM-002",
                "sampleId": "SMPL-002", 
                "concentration": 32.1,
                "purity": 1.72,
                "qualityScore": 78,
                "timestamp": (datetime.now() - timedelta(hours=3)).isoformat()
            }
        ],
        "summary": {
            "avgConcentration": 41.15,
            "avgPurity": 1.785,
            "avgQualityScore": 86.5
        }
    }

# Flow Cell endpoints
@app.get("/api/flow-cells/types")
async def get_flow_cell_types():
    """Get flow cell types"""
    return [
        {"id": str(uuid.uuid4()), "name": "NovaSeq S4", "lane_count": 4, "reads_per_lane": 3200000000},
        {"id": str(uuid.uuid4()), "name": "NovaSeq S2", "lane_count": 2, "reads_per_lane": 1650000000},
        {"id": str(uuid.uuid4()), "name": "MiSeq v3", "lane_count": 1, "reads_per_lane": 25000000}
    ]

@app.post("/api/flow-cells/designs")
async def create_flow_cell_design(design: dict):
    """Create a new flow cell design"""
    return {"id": str(uuid.uuid4()), **design}

@app.post("/api/flow-cells/optimize")
async def optimize_flow_cell(optimization_request: dict):
    """Optimize flow cell design"""
    # Mock optimization result
    return {
        "optimized": True,
        "lane_assignments": [],
        "balance_score": 0.95,
        "estimated_reads": 3200000000
    }

# Add dedicated endpoint for RAG samples search
@app.get("/api/rag/samples")
async def get_rag_samples():
    """Get RAG-processed samples"""
    rag_samples_data = [
        {
            "id": "SMPL-RAG-001",
            "originalId": "SMPL-001",
            "name": "Sample 001 (RAG Processed)",
            "type": "DNA",
            "status": "RAG_Analyzed",
            "submittedBy": "Dr. Smith",
            "submittedDate": (datetime.now() - timedelta(days=1)).isoformat(),
            "ragProcessingDate": (datetime.now() - timedelta(hours=2)).isoformat(),
            "extractedMetadata": {
                "concentration": "50 ng/μL",
                "volume": "100 μL",
                "quality": "High",
                "extractionMethod": "Qiagen DNeasy"
            },
            "confidenceScore": 0.94,
            "ragStatus": "Completed"
        },
        {
            "id": "SMPL-RAG-002",
            "originalId": "SMPL-002",
            "name": "Sample 002 (RAG Processed)",
            "type": "RNA",
            "status": "RAG_Processing",
            "submittedBy": "Dr. Johnson",
            "submittedDate": (datetime.now() - timedelta(days=3)).isoformat(),
            "ragProcessingDate": (datetime.now() - timedelta(minutes=30)).isoformat(),
            "extractedMetadata": {
                "concentration": "75 ng/μL",
                "volume": "50 μL",
                "quality": "Medium",
                "extractionMethod": "TRIzol"
            },
            "confidenceScore": 0.87,
            "ragStatus": "Processing"
        },
        {
            "id": "SMPL-RAG-003",
            "originalId": "SMPL-003",
            "name": "Sample 003 (RAG Pending)",
            "type": "Protein",
            "status": "RAG_Pending",
            "submittedBy": "Dr. Williams",
            "submittedDate": datetime.now().isoformat(),
            "extractedMetadata": {},
            "confidenceScore": 0.0,
            "ragStatus": "Pending"
        }
    ]

    # Return both formats for compatibility
    return {
        "data": rag_samples_data,  # For frontend expecting .data.filter()
        "ragSamples": rag_samples_data,  # For other consumers
        "totalCount": len(rag_samples_data),
        "processing": 1,
        "completed": 1,
        "pending": 1
    }

@app.post("/api/rag/samples/search")
async def search_rag_samples(request: Request):
    """Search RAG-processed samples"""
    try:
        body = await request.json()
        search_term = body.get("searchTerm", "")

        # Mock search results
        search_results = [
            {
                "id": "SMPL-RAG-001",
                "name": "Sample 001 (RAG Processed)",
                "type": "DNA",
                "status": "RAG_Analyzed",
                "relevance": 0.95,
                "matchedFields": ["name", "type"],
                "extractedMetadata": {
                    "concentration": "50 ng/μL",
                    "volume": "100 μL",
                    "quality": "High"
                }
            }
        ] if search_term else []

        return {
            "data": search_results,  # For frontend expecting .data.filter()
            "searchResults": search_results,  # For other consumers
            "query": search_term,
            "totalResults": len(search_results),
            "timestamp": datetime.now().isoformat()
        }

    except Exception as e:
        raise HTTPException(status_code=400, detail=f"Invalid search request: {e!s}")

# Spreadsheet endpoints for dataset management
@app.get("/api/spreadsheets/filters")
async def get_spreadsheet_filters():
    """Get available filters for spreadsheet search"""
    return {
        "data": {
            "datasets": [
                {"id": "DS-001", "name": "Sample Tracking Dataset"},
                {"id": "DS-002", "name": "Sequencing Results Dataset"},
                {"id": "DS-003", "name": "Storage Inventory Dataset"}
            ],
            "file_types": ["xlsx", "csv", "xls"],
            "columns": [
                "Sample_ID", "Type", "Concentration", "Volume", "Quality_Score",
                "Storage_Location", "Submitted_Date", "Status", "Job_ID", "Platform",
                "Coverage", "Location_ID", "Temperature", "Capacity"
            ],
            "sample_types": ["DNA", "RNA", "Protein"],
            "statuses": ["Pending", "Processing", "Completed", "Failed"],
            "platforms": ["Illumina NovaSeq", "Illumina MiSeq", "Ion Torrent"],
            # Add the missing arrays that the frontend expects
            "pools": ["Pool-A", "Pool-B", "Pool-C", "Pool-D"],
            "samples": ["SMPL-001", "SMPL-002", "SMPL-003", "SMPL-004", "SMPL-005"],
            "projects": ["Project-Alpha", "Project-Beta", "Project-Gamma", "Project-Delta"],
            "all_columns": [
                "Sample_ID", "Type", "Concentration", "Volume", "Quality_Score",
                "Storage_Location", "Submitted_Date", "Status", "Job_ID", "Platform",
                "Coverage", "Location_ID", "Temperature", "Capacity"
            ],
            "column_values": {
                "Type": ["DNA", "RNA", "Protein"],
                "Status": ["Pending", "Processing", "Completed", "Failed"],
                "Platform": ["Illumina NovaSeq", "Illumina MiSeq", "Ion Torrent"]
            }
        }
    }

@app.get("/api/spreadsheets/search")
async def search_spreadsheet_data(
    dataset_id: str = Query(None),
    limit: int = Query(50, ge=1, le=1000),
    offset: int = Query(0, ge=0),
    search_term: str = Query(None),
    column: str = Query(None)
):
    """Search spreadsheet data within a dataset or across all datasets"""
    # If no dataset_id provided, search across all datasets
    if dataset_id is None:
        # Global search across all mock datasets
        all_sample_data = []
        
        # Add data from DS-001 (Sample tracking)
        ds001_data = [
            {
                "Sample_ID": "SMPL-001",
                "Type": "DNA",
                "Concentration": 50.2,
                "Volume": 100.0,
                "Quality_Score": 95,
                "Storage_Location": "Freezer A1-B2",
                "Submitted_Date": "2024-01-15",
                "Status": "Completed",
                "dataset_id": "DS-001",
                "dataset_name": "Sample Tracking Dataset"
            },
            {
                "Sample_ID": "SMPL-002", 
                "Type": "RNA",
                "Concentration": 75.0,
                "Volume": 50.0,
                "Quality_Score": 88,
                "Storage_Location": "Freezer A2-C1",
                "Submitted_Date": "2024-01-16",
                "Status": "Processing",
                "dataset_id": "DS-001",
                "dataset_name": "Sample Tracking Dataset"
            },
            {
                "Sample_ID": "SMPL-004",
                "Type": "DNA",
                "Concentration": 68.5,
                "Volume": 80.0,
                "Quality_Score": 97,
                "Storage_Location": "Freezer A1-C4",
                "Submitted_Date": "2024-01-18",
                "Status": "Completed",
                "dataset_id": "DS-001",
                "dataset_name": "Sample Tracking Dataset"
            }
        ]
        
        # Add data from DS-002 (Sequencing results)
        ds002_data = [
            {
                "Job_ID": "SEQ-001",
                "Sample_Count": 24,
                "Platform": "Illumina NovaSeq",
                "Coverage": 30.5,
                "Quality_Score": 35.2,
                "Completion_Date": "2024-01-20",
                "Output_Files": ["file1.fastq", "file2.fastq"],
                "dataset_id": "DS-002",
                "dataset_name": "Sequencing Results Dataset"
            }
        ]
        
        # Add data from DS-003 (Storage inventory)
        ds003_data = [
            {
                "Location_ID": "LOC-001",
                "Temperature": -80,
                "Capacity": 1000,
                "Occupied": 750,
                "Utilization": 75.0,
                "Last_Check": "2024-01-22T10:30:00Z",
                "dataset_id": "DS-003",
                "dataset_name": "Storage Inventory Dataset"
            }
        ]
        
        all_sample_data = ds001_data + ds002_data + ds003_data
        sample_data = all_sample_data
        
    # Mock search results based on dataset
    elif dataset_id == "DS-001":
        # Sample tracking dataset
        sample_data = [
            {
                "Sample_ID": "SMPL-001",
                "Type": "DNA",
                "Concentration": 50.2,
                "Volume": 100.0,
                "Quality_Score": 95,
                "Storage_Location": "Freezer A1-B2",
                "Submitted_Date": "2024-01-15",
                "Status": "Completed"
            },
            {
                "Sample_ID": "SMPL-002", 
                "Type": "RNA",
                "Concentration": 75.0,
                "Volume": 50.0,
                "Quality_Score": 88,
                "Storage_Location": "Freezer A2-C1",
                "Submitted_Date": "2024-01-16",
                "Status": "Processing"
            },
            {
                "Sample_ID": "SMPL-003",
                "Type": "Protein",
                "Concentration": 32.1,
                "Volume": 75.0,
                "Quality_Score": 92,
                "Storage_Location": "Refrigerator B1-A3",
                "Submitted_Date": "2024-01-17",
                "Status": "Pending"
            },
            {
                "Sample_ID": "SMPL-004",
                "Type": "DNA",
                "Concentration": 68.5,
                "Volume": 80.0,
                "Quality_Score": 97,
                "Storage_Location": "Freezer A1-C4",
                "Submitted_Date": "2024-01-18",
                "Status": "Completed"
            },
            {
                "Sample_ID": "SMPL-005",
                "Type": "RNA",
                "Concentration": 45.3,
                "Volume": 60.0,
                "Quality_Score": 85,
                "Storage_Location": "Freezer A3-B2",
                "Submitted_Date": "2024-01-19",
                "Status": "Failed"
            }
        ]
    elif dataset_id == "DS-002":
        # Sequencing results dataset
        sample_data = [
            {
                "Job_ID": "SEQ-001",
                "Sample_Count": 24,
                "Platform": "Illumina NovaSeq",
                "Coverage": 30.5,
                "Quality_Score": 35.2,
                "Completion_Date": "2024-01-20",
                "Output_Files": ["file1.fastq", "file2.fastq"]
            },
            {
                "Job_ID": "SEQ-002",
                "Sample_Count": 48,
                "Platform": "Illumina MiSeq", 
                "Coverage": 25.0,
                "Quality_Score": 32.8,
                "Completion_Date": "2024-01-21",
                "Output_Files": ["file3.fastq", "file4.fastq"]
            }
        ]
    elif dataset_id == "DS-003":
        # Storage inventory dataset
        sample_data = [
            {
                "Location_ID": "LOC-001",
                "Temperature": -80,
                "Capacity": 1000,
                "Occupied": 750,
                "Utilization": 75.0,
                "Last_Check": "2024-01-22T10:30:00Z"
            },
            {
                "Location_ID": "LOC-002",
                "Temperature": 4,
                "Capacity": 500,
                "Occupied": 320,
                "Utilization": 64.0,
                "Last_Check": "2024-01-22T11:15:00Z"
            }
        ]
    else:
        # Check if this is an uploaded dataset in the database
        try:
            async with get_database_connection() as conn:
                # Check if dataset exists in database
                dataset_exists = await conn.fetchrow("""
                    SELECT id, total_rows FROM spreadsheet_datasets 
                    WHERE id = $1 OR name LIKE $2
                """, dataset_id, f"%{dataset_id}%")
                
                if dataset_exists:
                    # Fetch actual data from spreadsheet_records table
                    query = """
                        SELECT row_number, row_data 
                        FROM spreadsheet_records 
                        WHERE dataset_id = $1
                    """
                    params = [str(dataset_exists['id'])]
                    
                    # Apply search filter if provided
                    if search_term:
                        query += " AND search_text ILIKE $2"
                        params.append(f"%{search_term}%")
                    
                    # Apply column filter if provided
                    if column:
                        query += f" AND row_data ->> '{column}' IS NOT NULL"
                    
                    query += " ORDER BY row_number"
                    
                    # Apply pagination
                    query += f" LIMIT {limit} OFFSET {offset}"
                    
                    rows = await conn.fetch(query, *params)
                    
                    # Convert database rows to sample_data format
                    sample_data = []
                    for row in rows:
                        row_data = json.loads(row['row_data']) if isinstance(row['row_data'], str) else row['row_data']
                        
                        # Apply column filter if provided
                        if column and column in row_data:
                            # Filter to show only the specified column (plus ID field)
                            id_field = next((key for key in row_data.keys() if 'ID' in key), None)
                            if id_field:
                                filtered_data = {id_field: row_data[id_field], column: row_data[column]}
                            else:
                                filtered_data = {column: row_data[column]}
                            sample_data.append(filtered_data)
                        else:
                            sample_data.append(row_data)
                else:
                    sample_data = []
        except Exception as e:
            print(f"Database error fetching spreadsheet data: {e}")
            sample_data = []
    
    # Apply search filter if provided
    if search_term:
        search_lower = search_term.lower()
        filtered_data = []
        for row in sample_data:
            # Search across all string values in the row
            if any(search_lower in str(value).lower() for value in row.values()):
                filtered_data.append(row)
        sample_data = filtered_data
    
    # Apply column filter if provided
    if column and sample_data:
        # Filter to show only the specified column (plus ID field)
        id_field = next((key for key in sample_data[0].keys() if 'ID' in key), None)
        if id_field and column in sample_data[0]:
            sample_data = [{id_field: row[id_field], column: row[column]} for row in sample_data]
    
    # Apply pagination
    total_count = len(sample_data)
    paginated_data = sample_data[offset:offset + limit]
    
    # Convert data to the format expected by frontend
    records = []
    for i, row in enumerate(paginated_data):
        # For global search, use the dataset_id from the row data if available
        row_dataset_id = row.get("dataset_id", dataset_id) if dataset_id is None else dataset_id
        records.append({
            "id": f"{row_dataset_id or 'GLOBAL'}-{offset + i + 1}",
            "row_number": offset + i + 1,
            "row_data": row,
            "dataset_id": row_dataset_id,
            "created_at": "2024-01-15T10:30:00Z"
        })
    
    return {
        "success": True,
        "data": {
            "records": records,
            "total_count": total_count,
            "pagination": {
                "limit": limit,
                "offset": offset,
                "has_more": offset + limit < total_count
            }
        },
        "dataset_id": dataset_id,
        "search_term": search_term,
        "column_filter": column,
        "global_search": dataset_id is None
    }

@app.get("/api/spreadsheets/datasets")
async def get_spreadsheet_datasets():
    """Get all spreadsheet datasets including uploaded ones from database"""
    # Static mock datasets
    static_datasets = [
        {
            "id": "DS-001",
            "name": "Sample Tracking Dataset",
            "description": "Main sample tracking spreadsheet with QC data",
            "fileName": "sample_tracking_2024.xlsx",
            "version": "1.3",
            "lastModified": (datetime.now() - timedelta(hours=2)).isoformat(),
            "createdBy": "Dr. Smith",
            "status": "Active",
            "recordCount": 1247,
            "columns": [
                {"name": "Sample_ID", "type": "string", "required": True},
                {"name": "Type", "type": "enum", "values": ["DNA", "RNA", "Protein"]},
                {"name": "Concentration", "type": "number", "unit": "ng/μL"},
                {"name": "Volume", "type": "number", "unit": "μL"},
                {"name": "Quality_Score", "type": "number", "range": [0, 100]},
                {"name": "Storage_Location", "type": "string"},
                {"name": "Submitted_Date", "type": "date"},
                {"name": "Status", "type": "enum", "values": ["Pending", "Processing", "Completed", "Failed"]}
            ],
            "column_headers": ["Sample_ID", "Type", "Concentration", "Volume", "Quality_Score", "Storage_Location", "Submitted_Date", "Status"]
        },
        {
            "id": "DS-002",
            "name": "Sequencing Results Dataset",
            "description": "Sequencing job results and quality metrics",
            "fileName": "sequencing_results_2024.xlsx",
            "version": "2.1",
            "lastModified": (datetime.now() - timedelta(days=1)).isoformat(),
            "createdBy": "Lab Tech Johnson",
            "status": "Active",
            "recordCount": 456,
            "columns": [
                {"name": "Job_ID", "type": "string", "required": True},
                {"name": "Sample_Count", "type": "number"},
                {"name": "Platform", "type": "enum", "values": ["Illumina NovaSeq", "Illumina MiSeq", "Ion Torrent"]},
                {"name": "Coverage", "type": "number", "unit": "X"},
                {"name": "Quality_Score", "type": "number", "range": [0, 40]},
                {"name": "Completion_Date", "type": "date"},
                {"name": "Output_Files", "type": "array"}
            ]
        },
        {
            "id": "DS-003",
            "name": "Storage Inventory Dataset",
            "description": "Current storage locations and capacity tracking",
            "fileName": "storage_inventory_2024.xlsx",
            "version": "1.0",
            "lastModified": (datetime.now() - timedelta(hours=6)).isoformat(),
            "createdBy": "Dr. Williams",
            "status": "Active",
            "recordCount": 2500,
            "columns": [
                {"name": "Location_ID", "type": "string", "required": True},
                {"name": "Temperature", "type": "number", "unit": "°C"},
                {"name": "Capacity", "type": "number"},
                {"name": "Occupied", "type": "number"},
                {"name": "Utilization", "type": "number", "unit": "%"},
                {"name": "Last_Check", "type": "datetime"}
            ]
        }
    ]
    
    # Combine static datasets with uploaded datasets from database
    all_datasets = static_datasets.copy()
    
    try:
        async with get_database_connection() as conn:
            # Fetch uploaded datasets from database
            rows = await conn.fetch("""
                SELECT 
                    id, name, filename, original_filename, sheet_name, file_type, file_size,
                    total_rows, total_columns, upload_status, metadata, column_headers,
                    uploaded_by, created_at, updated_at
                FROM spreadsheet_datasets
                ORDER BY created_at DESC
            """)
            
            for row in rows:
                # Parse metadata and column_headers from JSON
                metadata = json.loads(row['metadata']) if row['metadata'] else {}
                column_headers = json.loads(row['column_headers']) if row['column_headers'] else []
                
                dataset = {
                    "id": str(row['id']),
                    "name": row['name'],
                    "original_filename": row['original_filename'],
                    "fileName": row['filename'],
                    "sheet_name": row['sheet_name'],
                    "file_type": row['file_type'],
                    "file_size": row['file_size'],
                    "upload_status": row['upload_status'],
                    "status": metadata.get("status", "Active"),
                    "uploaded_by": row['uploaded_by'],
                    "createdBy": row['uploaded_by'],
                    "created_at": row['created_at'].isoformat(),
                    "lastModified": row['updated_at'].isoformat(),
                    "total_rows": row['total_rows'],
                    "recordCount": row['total_rows'],
                    "total_columns": row['total_columns'],
                    "column_headers": column_headers,
                    "columns": [
                        {"name": "Sample_ID", "type": "string", "required": True},
                        {"name": "Type", "type": "enum", "values": ["DNA", "RNA", "Protein"]},
                        {"name": "Concentration", "type": "number", "unit": "ng/μL"},
                        {"name": "Volume", "type": "number", "unit": "μL"},
                        {"name": "Quality_Score", "type": "number", "range": [0, 100]},
                        {"name": "Storage_Location", "type": "string"},
                        {"name": "Submitted_Date", "type": "date"},
                        {"name": "Status", "type": "enum", "values": ["Pending", "Processing", "Completed", "Failed"]}
                    ],
                    "description": metadata.get("description", f"Uploaded spreadsheet: {row['filename']}"),
                    "version": metadata.get("version", "1.0")
                }
                all_datasets.append(dataset)
                
    except Exception as e:
        print(f"Error fetching datasets from database: {e}")
        # Continue with static datasets only

    # Return both formats for compatibility
    return {
        "data": all_datasets,  # For frontend expecting .data.filter()
        "datasets": all_datasets,  # For other consumers
        "totalCount": len(all_datasets),
        "activeDatasets": len([ds for ds in all_datasets if ds.get("status") == "Active"]),
        "totalRecords": sum(ds.get("recordCount", 0) for ds in all_datasets)
    }

@app.post("/api/spreadsheets/datasets")
async def create_spreadsheet_dataset(request: Request):
    """Create a new spreadsheet dataset"""
    try:
        body = await request.json()
        dataset_name = body.get("name", f"Dataset-{datetime.now().strftime('%Y%m%d%H%M%S')}")

        return {
            "id": f"DS-{datetime.now().strftime('%Y%m%d%H%M%S')}",
            "name": dataset_name,
            "status": "created",
            "message": "Spreadsheet dataset created successfully",
            "version": "1.0",
            "createdDate": datetime.now().isoformat()
        }

    except Exception as e:
        raise HTTPException(status_code=400, detail=f"Invalid dataset creation request: {e!s}")

@app.get("/api/spreadsheets/datasets/{dataset_id}")
async def get_spreadsheet_dataset(dataset_id: str):
    """Get specific spreadsheet dataset"""
    # Mock dataset details
    return {
        "data": {
            "id": dataset_id,
            "name": f"Dataset {dataset_id}",
            "description": "Detailed dataset information",
            "version": "1.0",
            "records": [
                {
                    "Sample_ID": "SMPL-001",
                    "Type": "DNA",
                    "Concentration": 50.0,
                    "Volume": 100.0,
                    "Quality_Score": 95,
                    "Storage_Location": "Freezer A1-B2",
                    "Status": "Completed"
                },
                {
                    "Sample_ID": "SMPL-002",
                    "Type": "RNA",
                    "Concentration": 75.0,
                    "Volume": 50.0,
                    "Quality_Score": 88,
                    "Storage_Location": "Freezer A2-C1",
                    "Status": "Processing"
                }
            ]
        },
        "totalRecords": 2,
        "lastModified": datetime.now().isoformat()
    }

@app.get("/api/spreadsheets/preview-sheets")
async def preview_spreadsheet_sheets():
    """Get sheet names from a spreadsheet for preview"""
    # Mock sheet names for now
    return {
        "sheets": [
            {"name": "Samples", "index": 0, "rowCount": 1247, "columnCount": 15},
            {"name": "QC_Results", "index": 1, "rowCount": 456, "columnCount": 12},
            {"name": "Storage_Inventory", "index": 2, "rowCount": 2500, "columnCount": 8},
            {"name": "Metadata", "index": 3, "rowCount": 50, "columnCount": 4}
        ],
        "fileName": "current_dataset.xlsx",
        "fileSize": 2048576,  # 2MB
        "lastModified": datetime.now().isoformat()
    }

@app.post("/api/spreadsheets/preview-sheets")
async def preview_spreadsheet_sheets_upload(file: UploadFile = File(...)):
    """Get sheet names from an uploaded spreadsheet file for preview"""
    try:
        # Read the uploaded file
        file_content = await file.read()
        file_size = len(file_content)
        
        # For now, return mock sheet names based on file type
        # In production, this would parse the actual spreadsheet
        file_extension = file.filename.split('.')[-1].lower() if file.filename else 'unknown'
        
        if file_extension in ['xlsx', 'xls']:
            sheet_names = ["Sheet1", "Data", "Summary"]
        elif file_extension == 'csv':
            sheet_names = ["CSV_Data"]
        else:
            sheet_names = ["Unknown"]
        
        return {
            "success": True,
            "data": sheet_names,
            "message": f"Found {len(sheet_names)} sheet(s) in {file.filename}"
        }
        
    except Exception as e:
        print(f"Error processing uploaded file: {e}")
        raise HTTPException(status_code=400, detail=f"Failed to process file: {str(e)}")

@app.post("/api/spreadsheets/upload-multiple")
async def upload_spreadsheet_multiple(
    file: UploadFile = File(...),
    uploaded_by: str = Form(""),
    selected_sheets: str = Form("[]")
):
    """Upload spreadsheet file and create datasets with database persistence"""
    try:
        # Read the uploaded file
        file_content = await file.read()
        file_size = len(file_content)
        file_extension = file.filename.split('.')[-1].lower() if file.filename else 'unknown'
        
        # Parse selected sheets
        try:
            sheets = json.loads(selected_sheets) if selected_sheets else []
        except:
            sheets = []
        
        # Generate dataset IDs for each sheet
        datasets = []
        timestamp = datetime.now().strftime('%Y%m%d%H%M%S')
        
        if not sheets:  # Default to single dataset for CSV or if no sheets selected
            sheets = ["Sheet1"] if file_extension in ['xlsx', 'xls'] else ["Data"]
        
        async with get_database_connection() as conn:
            for i, sheet_name in enumerate(sheets):
                dataset_id = str(uuid.uuid4())  # Use proper UUID
                record_count = 100 + (i * 50)  # Mock row counts
                column_count = 8 + (i * 2)     # Mock column counts
                
                # Insert into spreadsheet_datasets table
                await conn.execute("""
                    INSERT INTO spreadsheet_datasets (
                        id, name, filename, original_filename, sheet_name, file_type, file_size,
                        total_rows, total_columns, upload_status, metadata, 
                        column_headers, uploaded_by, created_at, updated_at
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $14)
                """, 
                dataset_id,
                f"{file.filename} - {sheet_name}",
                file.filename,
                file.filename,
                sheet_name,
                file_extension,
                file_size,
                record_count,
                column_count,
                "completed",
                json.dumps({
                    "description": f"Uploaded spreadsheet: {file.filename}",
                    "version": "1.0",
                    "status": "Active"
                }),
                json.dumps([
                    "Sample_ID", "Type", "Concentration", "Volume", 
                    "Quality_Score", "Storage_Location", "Submitted_Date", "Status"
                ]),
                uploaded_by or "anonymous",
                datetime.now(),
                )
                
                # Generate and insert sample records into spreadsheet_records table
                for row_num in range(1, min(record_count + 1, 21)):  # Limit to 20 rows for demo
                    row_data = {
                        "Sample_ID": f"SMPL-{row_num:03d}",
                        "Type": ["DNA", "RNA", "Protein"][(row_num - 1) % 3],
                        "Concentration": round(20.0 + ((row_num - 1) * 5.5) + ((row_num - 1) % 7) * 3.2, 1),
                        "Volume": round(50.0 + ((row_num - 1) * 10) + ((row_num - 1) % 5) * 8.5, 1),
                        "Quality_Score": 85 + ((row_num - 1) % 15),
                        "Storage_Location": f"Freezer {chr(65 + ((row_num - 1) % 3))}{((row_num - 1) % 5) + 1}-{chr(65 + ((row_num - 1) % 4))}{((row_num - 1) % 3) + 1}",
                        "Submitted_Date": f"2024-{(((row_num - 1) % 12) + 1):02d}-{(((row_num - 1) % 28) + 1):02d}",
                        "Status": ["Completed", "Processing", "Pending", "Failed"][(row_num - 1) % 4]
                    }
                    
                    # Create search text for full-text search
                    search_text = " ".join(str(value) for value in row_data.values())
                    
                    await conn.execute("""
                        INSERT INTO spreadsheet_records (
                            dataset_id, row_number, row_data, search_text, created_at
                        ) VALUES ($1, $2, $3, $4, $5)
                    """,
                    dataset_id,
                    row_num,
                    json.dumps(row_data),
                    search_text,
                    datetime.now()
                    )
                
                # Build dataset response
                dataset = {
                    "id": dataset_id,
                    "name": f"{file.filename} - {sheet_name}",
                    "original_filename": file.filename,
                    "fileName": file.filename,
                    "sheet_name": sheet_name,
                    "file_type": file_extension,
                    "file_size": file_size,
                    "upload_status": "completed",
                    "status": "Active",
                    "uploaded_by": uploaded_by or "anonymous",
                    "createdBy": uploaded_by or "anonymous",
                    "created_at": datetime.now().isoformat(),
                    "lastModified": datetime.now().isoformat(),
                    "total_rows": record_count,
                    "recordCount": record_count,
                    "total_columns": column_count,
                    "column_headers": [
                        "Sample_ID", "Type", "Concentration", "Volume", 
                        "Quality_Score", "Storage_Location", "Submitted_Date", "Status"
                    ],
                    "columns": [
                        {"name": "Sample_ID", "type": "string", "required": True},
                        {"name": "Type", "type": "enum", "values": ["DNA", "RNA", "Protein"]},
                        {"name": "Concentration", "type": "number", "unit": "ng/μL"},
                        {"name": "Volume", "type": "number", "unit": "μL"},
                        {"name": "Quality_Score", "type": "number", "range": [0, 100]},
                        {"name": "Storage_Location", "type": "string"},
                        {"name": "Submitted_Date", "type": "date"},
                        {"name": "Status", "type": "enum", "values": ["Pending", "Processing", "Completed", "Failed"]}
                    ],
                    "description": f"Uploaded spreadsheet: {file.filename}",
                    "version": "1.0"
                }
                datasets.append(dataset)
        
        return {
            "success": True,
            "data": datasets,
            "message": f"Successfully uploaded {file.filename} with {len(datasets)} dataset(s)"
        }
        
    except Exception as e:
        print(f"Error uploading file: {e}")
        import traceback
        traceback.print_exc()
        raise HTTPException(status_code=400, detail=f"Failed to upload file: {str(e)}")

# Redirect handlers for double /api URLs (frontend routing issue)
@app.get("/api/api/storage/locations")
async def redirect_storage_locations():
    """Redirect handler for double /api prefix - frontend routing issue"""
    # Just redirect to the correct endpoint internally
    return await get_storage_locations()

# CONTAINERS REDIRECT: Remove duplicate endpoint - using the main one at line 2310

@app.get("/api/api/samples")
async def redirect_samples(request: Request):
    """Redirect handler for double /api prefix - frontend routing issue"""
    return await get_samples(request)

@app.get("/api/api/templates")
async def redirect_templates():
    """Redirect handler for double /api prefix - frontend routing issue"""
    return await get_templates()

@app.get("/api/api/sequencing/jobs")
async def redirect_sequencing_jobs():
    """Redirect handler for double /api prefix - frontend routing issue"""
    return await get_sequencing_jobs()

# Additional debug endpoint to help identify which endpoint is causing filter issues
@app.get("/api/debug/problematic-endpoints")
async def debug_problematic_endpoints():
    """Debug endpoint to test all data formats"""
    test_results = {}

    # Test each endpoint's data format
    endpoints_to_test = [
        ("/api/samples", "samples"),
        ("/api/templates", "templates"),
        ("/api/sequencing/jobs", "sequencing jobs"),
        ("/api/storage/locations", "storage locations"),
        ("/api/rag/submissions", "rag submissions"),
        ("/api/rag/samples", "rag samples"),
        ("/api/spreadsheets/datasets", "spreadsheet datasets")
    ]

    for endpoint, name in endpoints_to_test:
        try:
            # Simulate a request to test data format
            if endpoint == "/api/samples":
                # Create a proper request object for testing
                from starlette.requests import Request
                from starlette.datastructures import QueryParams, Headers
                scope = {
                    "type": "http",
                    "query_string": b"",
                    "headers": [],
                }
                response = await get_samples(Request(scope))
            elif endpoint == "/api/templates":
                response = await get_templates()
            elif endpoint == "/api/sequencing/jobs":
                response = await get_sequencing_jobs()
            elif endpoint == "/api/storage/locations":
                response = await get_storage_locations()
            elif endpoint == "/api/rag/submissions":
                response = await get_rag_submissions()
            elif endpoint == "/api/rag/samples":
                response = await get_rag_samples()
            elif endpoint == "/api/spreadsheets/datasets":
                response = await get_spreadsheet_datasets()

            # Handle both list and dict responses
            if isinstance(response, list):
                test_results[name] = {
                    "endpoint": endpoint,
                    "has_data_field": False,
                    "data_is_array": True,
                    "data_length": len(response),
                    "data_type": "list (direct)",
                    "status": "❌ ISSUE - Returns list directly, should return {data: [...]}"
                }
            else:
                test_results[name] = {
                    "endpoint": endpoint,
                    "has_data_field": "data" in response,
                    "data_is_array": isinstance(response.get("data"), list) if "data" in response else False,
                    "data_length": len(response.get("data", [])) if isinstance(response.get("data"), list) else 0,
                    "data_type": str(type(response.get("data", None))),
                    "status": "✅ OK" if (isinstance(response.get("data"), list) and len(response.get("data", [])) > 0) else "❌ ISSUE"
                }
        except Exception as e:
            test_results[name] = {
                "endpoint": endpoint,
                "error": str(e),
                "status": "❌ ERROR"
            }

    return {
        "debug_info": "Testing all endpoints for proper array data format",
        "test_results": test_results,
        "timestamp": datetime.now().isoformat()
    }

# ============================================================================
# Storage Management Endpoints - REMOVED DUPLICATES
# ============================================================================
# NOTE: Storage endpoints are now defined earlier in the file (lines ~187-244)

# NOTE: All storage endpoints removed from here - they are defined earlier in the file

# File opening endpoints for Finder
@app.get("/api/files/{file_id}/open")
async def open_file(file_id: str):
    """Open a file by ID"""
    return {"message": f"Opening file {file_id}", "action": "open"}

@app.get("/api/files/{file_id}/download")
async def download_file(file_id: str):
    """Download a file by ID"""
    # For now, return a simple response
    # In production, this would serve the actual file
    return {"message": f"Downloading file {file_id}", "action": "download"}



if __name__ == "__main__":
    # Get configuration from environment
    host = os.getenv("HOST", "0.0.0.0")
    port = int(os.getenv("PORT", "8000"))
    log_level = os.getenv("LOG_LEVEL", "info").lower()

    print(f"🚀 Starting TracSeq 2.0 API Gateway on {host}:{port}")
    print(f"📊 RAG Service URL: {RAG_SERVICE_URL}")

    # Run the application
    uvicorn.run(
        app,
        host=host,
        port=port,
        log_level=log_level,
        access_log=True
    )
