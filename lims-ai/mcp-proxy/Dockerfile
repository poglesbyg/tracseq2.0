FROM python:3.11-slim

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    gcc \
    g++ \
    make \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements
COPY requirements.txt .

# Install Python dependencies
RUN pip install --no-cache-dir -r requirements.txt

# Copy source code
COPY mcp_proxy_server.py .

# Health check script
COPY <<'EOF' /health_check.py
#!/usr/bin/env python3
import sys
import json
import asyncio
import websockets

async def check_health():
    try:
        async with websockets.connect("ws://localhost:9500/health") as ws:
            await ws.send(json.dumps({"type": "ping"}))
            response = await ws.recv()
            data = json.loads(response)
            if data.get("status") == "healthy":
                sys.exit(0)
    except:
        pass
    sys.exit(1)

asyncio.run(check_health())
EOF

RUN chmod +x /health_check.py

# Expose ports
EXPOSE 9500

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD python /health_check.py

# Run the server
CMD ["python", "-u", "mcp_proxy_server.py"] 