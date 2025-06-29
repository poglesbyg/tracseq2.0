#!/usr/bin/env python3
"""
Mock RAG Service for testing DevOps tools
Responds to health checks and AI requests
"""

from flask import Flask, jsonify
import logging

app = Flask(__name__)
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@app.route('/health', methods=['GET'])
def health():
    """Health check endpoint"""
    return jsonify({
        "status": "healthy",
        "service": "TracSeq RAG Service",
        "version": "2.0.0-mock",
        "timestamp": "2024-12-29T16:30:00Z"
    }), 200

@app.route('/api/rag/status', methods=['GET']) 
def rag_status():
    """RAG status endpoint"""
    return jsonify({
        "service": "rag-service",
        "status": "running",
        "models": ["llama3.2:3b"],
        "vector_store": "connected",
        "document_count": 125
    }), 200

@app.route('/api/extract', methods=['POST'])
def extract():
    """Mock document extraction endpoint"""
    return jsonify({
        "status": "success",
        "extracted": {
            "submitter": "Dr. Smith",
            "sample_type": "DNA",
            "volume": "50μL"
        },
        "confidence": 0.92
    }), 200

@app.route('/', methods=['GET'])
def root():
    """Root endpoint"""
    return jsonify({
        "message": "TracSeq 2.0 RAG Service Mock",
        "version": "2.0.0-mock",
        "endpoints": ["/health", "/api/rag/status", "/api/extract"]
    }), 200

if __name__ == '__main__':
    logger.info("Starting TracSeq RAG Service Mock API on port 8000")
    app.run(host='0.0.0.0', port=8000, debug=True)
