#!/usr/bin/env python3
"""
Mock Lab Manager API for testing DevOps tools
Responds to health checks and basic API calls
"""

from flask import Flask, jsonify
import logging
import sys

app = Flask(__name__)

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@app.route('/health', methods=['GET'])
def health():
    """Health check endpoint"""
    return jsonify({
        "status": "healthy",
        "service": "TracSeq Lab Manager",
        "version": "2.0.0-mock",
        "timestamp": "2024-12-29T16:30:00Z"
    }), 200

@app.route('/api/samples', methods=['GET'])
def list_samples():
    """Mock samples endpoint"""
    return jsonify({
        "samples": [
            {"id": "SMPL-001", "name": "Sample 1", "status": "active"},
            {"id": "SMPL-002", "name": "Sample 2", "status": "processed"}
        ],
        "total": 2
    }), 200

@app.route('/api/status', methods=['GET'])
def status():
    """Mock status endpoint"""
    return jsonify({
        "service": "lab-manager",
        "status": "running",
        "uptime": "2h 30m",
        "database": "connected",
        "storage": "available"
    }), 200

@app.route('/', methods=['GET'])
def root():
    """Root endpoint"""
    return jsonify({
        "message": "TracSeq 2.0 Lab Manager Mock API",
        "version": "2.0.0-mock",
        "endpoints": ["/health", "/api/samples", "/api/status"]
    }), 200

if __name__ == '__main__':
    logger.info("Starting TracSeq Lab Manager Mock API on port 3000")
    app.run(host='0.0.0.0', port=3000, debug=True)
