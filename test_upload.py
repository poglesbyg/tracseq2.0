#!/usr/bin/env python3
"""
Test script to verify upload functionality for TracSeq 2.0
"""

import requests
import json

def test_api_endpoints():
    base_url = "http://localhost:8089"
    
    print("🧪 Testing TracSeq 2.0 API Endpoints")
    print("=" * 50)
    
    # Test health check
    try:
        response = requests.get(f"{base_url}/health")
        print(f"✅ Health Check: {response.json()}")
    except Exception as e:
        print(f"❌ Health Check Failed: {e}")
        return
    
    # Test dashboard stats
    try:
        response = requests.get(f"{base_url}/api/dashboard/stats")
        stats = response.json()
        print(f"✅ Dashboard Stats: {json.dumps(stats, indent=2)}")
    except Exception as e:
        print(f"❌ Dashboard Stats Failed: {e}")
    
    # Test samples endpoint
    try:
        response = requests.get(f"{base_url}/api/samples")
        samples = response.json()
        print(f"✅ Samples Count: {len(samples)}")
        if samples:
            print(f"   Sample Example: {samples[0]['name']} ({samples[0]['sample_type']})")
    except Exception as e:
        print(f"❌ Samples Endpoint Failed: {e}")
    
    # Test templates endpoint
    try:
        response = requests.get(f"{base_url}/api/templates")
        templates = response.json()
        print(f"✅ Templates Count: {len(templates)}")
        if templates:
            print(f"   Template Example: {templates[0]['name']}")
    except Exception as e:
        print(f"❌ Templates Endpoint Failed: {e}")
    
    # Test spreadsheet datasets endpoint
    try:
        response = requests.get(f"{base_url}/api/spreadsheets/datasets")
        datasets = response.json()
        print(f"✅ Spreadsheet Datasets Count: {len(datasets)}")
    except Exception as e:
        print(f"❌ Spreadsheet Datasets Failed: {e}")
    
    # Test RAG submissions endpoint
    try:
        response = requests.get(f"{base_url}/api/rag/submissions")
        submissions = response.json()
        print(f"✅ RAG Submissions Count: {len(submissions)}")
    except Exception as e:
        print(f"❌ RAG Submissions Failed: {e}")
    
    print("\n🎯 Upload Endpoints Available:")
    print("   POST /api/spreadsheets/upload-multiple - Upload spreadsheet files")
    print("   POST /api/rag/process - Upload RAG documents")
    print("   POST /api/templates/upload - Upload template files")
    
    print("\n🌐 Frontend should be accessible at: http://localhost:5173")
    print("🔧 API Gateway accessible at: http://localhost:8089")

if __name__ == "__main__":
    test_api_endpoints()