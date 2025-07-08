#!/usr/bin/env python3
"""
Simple script to run the API Gateway without database dependencies for testing
"""
import os
import sys
import uvicorn

# Add the src directory to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'src'))

# Temporarily disable database import
os.environ['DISABLE_STANDARDIZED_DB'] = 'true'

if __name__ == "__main__":
    print("Starting API Gateway without database dependencies...")
    print("This is for testing purposes only!")
    
    # Import the app
    from api_gateway.simple_main import app
    
    # Run the server
    uvicorn.run(app, host="0.0.0.0", port=8000) 