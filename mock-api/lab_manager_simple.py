#!/usr/bin/env python3
import http.server
import socketserver
import json
from urllib.parse import urlparse

class TracSeqHandler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/health':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            response = {
                "status": "healthy",
                "service": "TracSeq Lab Manager",
                "version": "2.0.0-mock"
            }
            self.wfile.write(json.dumps(response).encode())
        else:
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            response = {"message": "TracSeq Lab Manager Mock API", "endpoints": ["/health"]}
            self.wfile.write(json.dumps(response).encode())

if __name__ == "__main__":
    PORT = 3000
    with socketserver.TCPServer(("", PORT), TracSeqHandler) as httpd:
        print(f"Lab Manager Mock API serving at port {PORT}")
        httpd.serve_forever()
