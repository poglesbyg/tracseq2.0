#!/usr/bin/env python3
"""
Simple API Gateway for TracSeq 2.0
Minimal working implementation for demonstration
"""

import os
import json
from typing import Dict, Any
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
import httpx
import uvicorn

# Initialize FastAPI app
app = FastAPI(
    title="TracSeq 2.0 API Gateway",
    description="Central routing hub for TracSeq microservices",
    version="2.0.0"
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # In production, specify exact origins
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Service URLs from environment
RAG_SERVICE_URL = os.getenv("RAG_SERVICE_URL", "http://rag-service:8000")

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
        "timestamp": "2024-01-01T00:00:00Z"
    }

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
            response = await client.get(f"{RAG_SERVICE_URL}/health", timeout=5.0)
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
async def proxy_rag(path: str, request):
    """Proxy requests to RAG service"""
    try:
        async with httpx.AsyncClient() as client:
            # Forward the request to RAG service
            url = f"{RAG_SERVICE_URL}/{path}"
            response = await client.request(
                method=request.method,
                url=url,
                headers=dict(request.headers),
                params=request.query_params,
                timeout=30.0
            )
            
            return response.json() if response.headers.get("content-type", "").startswith("application/json") else response.text
    
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"Service unavailable: {str(e)}")

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