#!/usr/bin/env python3

import socket
import json
import time

def send_log_to_logstash(host='localhost', port=5000):
    """Send a test log message to Logstash via TCP"""
    
    # Create test log message
    log_message = {
        "timestamp": "2025-07-14T19:15:00.000Z",
        "level": "INFO",
        "logger": "python_test",
        "message": "Python TCP test message",
        "request_id": "python-001",
        "processing_time_ms": 50
    }
    
    try:
        # Create socket connection
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.connect((host, port))
        
        # Send JSON message with newline
        message = json.dumps(log_message) + '\n'
        sock.send(message.encode('utf-8'))
        
        print(f"✅ Successfully sent log message to {host}:{port}")
        print(f"Message: {message.strip()}")
        
        sock.close()
        
    except Exception as e:
        print(f"❌ Error sending log message: {e}")

if __name__ == "__main__":
    print("🧪 Testing TCP log sending to Logstash...")
    send_log_to_logstash()
    
    # Wait a bit then check if it was indexed
    print("⏳ Waiting 3 seconds for processing...")
    time.sleep(3)
    
    # Check if index was created
    import subprocess
    try:
        result = subprocess.run(['curl', '-s', 'http://localhost:9200/_cat/indices?v'], 
                              capture_output=True, text=True)
        print("📊 Current indices:")
        print(result.stdout)
        
        # Search for the test message
        search_result = subprocess.run(['curl', '-s', 'http://localhost:9200/tracseq-debug-*/_search?q=python_test&pretty'], 
                                     capture_output=True, text=True)
        print("🔍 Search results:")
        print(search_result.stdout)
        
    except Exception as e:
        print(f"❌ Error checking results: {e}") 