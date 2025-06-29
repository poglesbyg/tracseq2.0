#!/usr/bin/env python3
"""
Mock API Gateway for testing DevOps tools
Responds to health checks and routes requests
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
        "service": "TracSeq API Gateway",
        "version": "2.0.0-mock",
        "timestamp": "2024-12-29T16:30:00Z"
    }), 200

@app.route('/api/gateway/status', methods=['GET'])
def gateway_status():
    """Gateway status endpoint"""
    return jsonify({
        "service": "api-gateway", 
        "status": "running",
        "routes": [
            {"path": "/api/samples", "target": "sample-service", "status": "healthy"},
            {"path": "/api/storage", "target": "storage-service", "status": "healthy"},
            {"path": "/api/auth", "target": "auth-service", "status": "healthy"}
        ]
    }), 200

@app.route('/', methods=['GET'])
def root():
    """Root endpoint"""
    return jsonify({
        "message": "TracSeq 2.0 API Gateway Mock",
        "version": "2.0.0-mock",
        "endpoints": ["/health", "/api/gateway/status"]
    }), 200

if __name__ == '__main__':
    logger.info("Starting TracSeq API Gateway Mock API on port 8089")
    app.run(host='0.0.0.0', port=8089, debug=True)
