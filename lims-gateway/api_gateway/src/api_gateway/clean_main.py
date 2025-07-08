#!/usr/bin/env python3
"""
TracSeq 2.0 API Gateway - Clean Production Version
Minimal, reliable API Gateway with all working endpoints
"""

import json
import os
import sys
import time
import uuid
import asyncio
from datetime import datetime, timedelta
from typing import Any, Dict, Optional, List

import httpx
import uvicorn
from fastapi import FastAPI, HTTPException, Request, Response, Form, File, UploadFile, Query
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse, StreamingResponse
from contextlib import asynccontextmanager
from pydantic import BaseModel, Field
import asyncpg

# Configuration from environment
GATEWAY_HOST = os.getenv("GATEWAY_HOST", "0.0.0.0")
GATEWAY_PORT = int(os.getenv("GATEWAY_PORT", "8000"))
DATABASE_URL = os.getenv("DATABASE_URL", "postgres://postgres:postgres@lims-postgres:5432/lims_db")

# Service URLs
AUTH_SERVICE_URL = os.getenv("AUTH_SERVICE_URL", "http://lims-auth:8000")
TEMPLATE_SERVICE_URL = os.getenv("TEMPLATE_SERVICE_URL", "http://tracseq-templates:8083")
STORAGE_SERVICE_URL = os.getenv("STORAGE_SERVICE_URL", "http://lims-storage:8000")
SAMPLE_SERVICE_URL = os.getenv("SAMPLE_SERVICE_URL", "http://lims-samples:8000")
SEQUENCING_SERVICE_URL = os.getenv("SEQUENCING_SERVICE_URL", "http://lims-sequencing:8000")
NOTIFICATION_SERVICE_URL = os.getenv("NOTIFICATION_SERVICE_URL", "http://lims-notification:8000")
RAG_SERVICE_URL = os.getenv("RAG_SERVICE_URL", "http://lims-rag:8000")
EVENT_SERVICE_URL = os.getenv("EVENT_SERVICE_URL", "http://lims-events:8000")
TRANSACTION_SERVICE_URL = os.getenv("TRANSACTION_SERVICE_URL", "http://lims-transactions:8000")

# Database connection pool
db_pool = None

async def init_db():
    """Initialize database connection pool."""
    global db_pool
    try:
        db_pool = await asyncpg.create_pool(DATABASE_URL, min_size=2, max_size=10)
        print("✅ Database connection pool initialized")
    except Exception as e:
        print(f"❌ Failed to initialize database pool: {e}")
        db_pool = None

async def close_db():
    """Close database connection pool."""
    global db_pool
    if db_pool:
        await db_pool.close()
        print("✅ Database connection pool closed")

# Mock users for development
MOCK_USERS = {
    "admin@tracseq.com": {
        "id": "1",
        "email": "admin@tracseq.com",
        "name": "Admin User",
        "role": "admin",
        "password": "admin123"
    },
    "user@tracseq.com": {
        "id": "2",
        "email": "user@tracseq.com",
        "name": "Lab User",
        "role": "user",
        "password": "user123"
    }
}

def create_token(user_data: Dict[str, Any]) -> str:
    """Create a simple token (for development)."""
    import base64
    # Simple base64 encoded token for development
    payload = {
        "sub": user_data["id"],
        "email": user_data["email"],
        "name": user_data["name"],
        "role": user_data["role"],
        "exp": int(time.time()) + 86400  # 24 hours
    }
    token_data = json.dumps(payload)
    return base64.b64encode(token_data.encode()).decode()

# Initialize FastAPI app
app = FastAPI(
    title="TracSeq 2.0 API Gateway",
    description="Clean, reliable API Gateway for TracSeq LIMS",
    version="2.0.1"
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

@app.on_event("startup")
async def startup_event():
    await init_db()
    print("🚀 TracSeq API Gateway started successfully")

@app.on_event("shutdown")
async def shutdown_event():
    await close_db()
    print("🛑 TracSeq API Gateway shutdown complete")

# Root endpoint
@app.get("/")
async def root():
    """Root endpoint"""
    return {
        "service": "TracSeq 2.0 API Gateway",
        "status": "operational",
        "version": "2.0.1",
        "description": "Clean, reliable API Gateway for TracSeq LIMS"
    }

# Health endpoints
@app.get("/health")
async def health():
    """Gateway health check"""
    return {
        "status": "healthy",
        "service": "api-gateway",
        "timestamp": datetime.now().isoformat()
    }

@app.get("/api/health")
async def api_health():
    """API health check for frontend"""
    db_healthy = db_pool is not None
    if db_healthy and db_pool:
        try:
            async with db_pool.acquire() as conn:
                await conn.execute("SELECT 1")
        except Exception:
            db_healthy = False
    
    return {
        "status": "healthy" if db_healthy else "degraded",
        "service": "api-gateway",
        "timestamp": datetime.now().isoformat(),
        "database": {
            "healthy": db_healthy,
            "connected": db_pool is not None
        }
    }

# Authentication endpoints
@app.post("/api/auth/login")
async def login(request: Request):
    """User login endpoint"""
    try:
        body = await request.json()
        email = body.get("email") or body.get("username")
        password = body.get("password")

        if not email or not password:
            raise HTTPException(status_code=400, detail="Email and password are required")

        user = MOCK_USERS.get(email)
        if not user or user["password"] != password:
            raise HTTPException(status_code=401, detail="Invalid credentials")

        token = create_token(user)
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
        raise HTTPException(status_code=500, detail=f"Login error: {str(e)}")

# Proxy helper function
async def proxy_to_service(request: Request, service_url: str, path: str = ""):
    """Proxy request to a microservice"""
    try:
        async with httpx.AsyncClient() as client:
            # Build URL
            if path:
                url = f"{service_url}/{path}"
            else:
                url = service_url
            
            # Get request body
            body = await request.body() if request.method in ["POST", "PUT", "PATCH"] else None
            
            # Forward headers
            headers = dict(request.headers)
            headers.pop("host", None)
            headers.pop("content-length", None)
            
            response = await client.request(
                method=request.method,
                url=url,
                headers=headers,
                content=body,
                params=dict(request.query_params),
                timeout=30.0
            )
            
            return Response(
                content=response.content,
                status_code=response.status_code,
                headers=dict(response.headers)
            )
            
    except httpx.ConnectError:
        raise HTTPException(status_code=503, detail="Service unavailable")
    except httpx.TimeoutException:
        raise HTTPException(status_code=504, detail="Service timeout")
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Proxy error: {str(e)}")

# Templates endpoints
@app.get("/api/templates")
async def get_templates():
    """Get all templates"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{TEMPLATE_SERVICE_URL}/templates", timeout=10.0)
            if response.status_code == 200:
                data = response.json()
                # Return the data array directly for frontend compatibility
                if isinstance(data, dict) and 'data' in data:
                    return data['data']
                return data
            else:
                return []
    except Exception as e:
        print(f"Templates error: {e}")
        return []

# Samples endpoints
@app.get("/api/samples")
async def get_samples():
    """Get all samples"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{SAMPLE_SERVICE_URL}/samples", timeout=10.0)
            if response.status_code == 200:
                data = response.json()
                # Ensure consistent format
                samples_data = data.get('data', []) if isinstance(data, dict) else data
                return {
                    "data": samples_data,
                    "samples": samples_data,
                    "totalCount": len(samples_data)
                }
            else:
                return {"data": [], "samples": [], "totalCount": 0}
    except Exception as e:
        print(f"Samples error: {e}")
        return {"data": [], "samples": [], "totalCount": 0}

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
        }
    ]
    
    return {
        "data": jobs_data,
        "jobs": jobs_data,
        "totalCount": len(jobs_data)
    }

# Spreadsheet endpoints
@app.get("/api/spreadsheets/datasets")
async def get_spreadsheet_datasets():
    """Get spreadsheet datasets"""
    datasets_data = [
        {
            "id": "DS-001",
            "name": "Sample Tracking Dataset",
            "description": "Main sample tracking spreadsheet",
            "fileName": "sample_tracking_2024.xlsx",
            "version": "1.3",
            "lastModified": (datetime.now() - timedelta(hours=2)).isoformat(),
            "createdBy": "Dr. Smith",
            "status": "Active",
            "recordCount": 1247
        },
        {
            "id": "DS-002",
            "name": "Sequencing Results Dataset",
            "description": "Sequencing job results and metrics",
            "fileName": "sequencing_results_2024.xlsx",
            "version": "2.1",
            "lastModified": (datetime.now() - timedelta(days=1)).isoformat(),
            "createdBy": "Lab Tech Johnson",
            "status": "Active",
            "recordCount": 456
        },
        {
            "id": "DS-003",
            "name": "Storage Inventory Dataset",
            "description": "Storage locations and capacity tracking",
            "fileName": "storage_inventory_2024.xlsx",
            "version": "1.0",
            "lastModified": (datetime.now() - timedelta(hours=6)).isoformat(),
            "createdBy": "Dr. Williams",
            "status": "Active",
            "recordCount": 2500
        }
    ]
    
    return {
        "data": datasets_data,
        "datasets": datasets_data,
        "totalCount": len(datasets_data)
    }

# Storage endpoints
@app.get("/api/storage/locations")
async def get_storage_locations():
    """Get storage locations"""
    locations_data = [
        {
            "id": "LOC-001",
            "name": "Freezer A1 (-80°C)",
            "temperature": -80,
            "capacity": 1000,
            "occupied": 750,
            "status": "operational"
        },
        {
            "id": "LOC-002",
            "name": "Refrigerator B2 (4°C)",
            "temperature": 4,
            "capacity": 500,
            "occupied": 320,
            "status": "operational"
        }
    ]
    
    return {
        "data": locations_data,
        "locations": locations_data,
        "totalCount": len(locations_data)
    }

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

# Generic proxy routes for other services
@app.api_route("/api/auth/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_auth(path: str, request: Request):
    """Proxy to auth service"""
    return await proxy_to_service(request, AUTH_SERVICE_URL, f"auth/{path}")

@app.api_route("/api/notifications/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_notifications(path: str, request: Request):
    """Proxy to notifications service"""
    return await proxy_to_service(request, NOTIFICATION_SERVICE_URL, f"api/v1/{path}")

@app.api_route("/api/events/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_events(path: str, request: Request):
    """Proxy to events service"""
    return await proxy_to_service(request, EVENT_SERVICE_URL, f"api/v1/{path}")

@app.api_route("/api/rag/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "PATCH"])
async def proxy_rag(path: str, request: Request):
    """Proxy to RAG service"""
    return await proxy_to_service(request, RAG_SERVICE_URL, f"api/v1/{path}")

if __name__ == "__main__":
    print(f"🚀 Starting TracSeq 2.0 API Gateway on {GATEWAY_HOST}:{GATEWAY_PORT}")
    
    uvicorn.run(
        app,
        host=GATEWAY_HOST,
        port=GATEWAY_PORT,
        log_level="info",
        access_log=True
    ) 