#!/usr/bin/env python3
"""
Simple RAG Service Runner
Runs the simple frontend bridge on port 8087 to avoid conflict with API Gateway
"""

import uvicorn

from simple_frontend_bridge_working import app

if __name__ == "__main__":
    print("🚀 Starting Simple RAG Service on port 8087")
    print("📡 Available endpoints:")
    print("   GET  /api/rag/submissions")
    print("   POST /api/rag/process")
    print("   GET  /api/rag/stats")
    print("   POST /query")
    print("   GET  /health")
    print("🌐 CORS enabled for all origins")

    uvicorn.run(app, host="0.0.0.0", port=8087)
