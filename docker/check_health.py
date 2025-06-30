#!/usr/bin/env python3
import subprocess
import requests
import json
import time

def check_service_health(name, port, path="/health"):
    """Check if a service is healthy"""
    try:
        url = f"http://localhost:{port}{path}"
        response = requests.get(url, timeout=2)
        if response.status_code == 200:
            try:
                data = response.json()
                return True, data
            except:
                return True, response.text
        else:
            return False, f"HTTP {response.status_code}"
    except requests.exceptions.RequestException as e:
        return False, str(e)

def main():
    print("Checking all services...")
    
    services = [
        ("Auth Service", 8011, "/health"),
        ("Sample Service", 8012, "/health"),
        ("Storage Service", 8013, "/health"),
        ("Reports Service", 8014, "/health"),
        ("RAG Service", 8100, "/api/v1/health"),
        ("API Gateway", 8089, "/health"),
    ]
    
    for name, port, path in services:
        healthy, data = check_service_health(name, port, path)
        if healthy:
            print(f"✅ {name}: Healthy - {data}")
        else:
            print(f"❌ {name}: Failed - {data}")
    
    # Check reports service logs
    print("\nChecking reports service logs...")
    try:
        result = subprocess.run(['docker', 'logs', 'lims-reports', '--tail=10'], 
                              capture_output=True, text=True)
        print(f"Reports logs: {repr(result.stdout)}")
        print(f"Reports stderr: {repr(result.stderr)}")
    except Exception as e:
        print(f"Error getting reports logs: {e}")

if __name__ == "__main__":
    main() 