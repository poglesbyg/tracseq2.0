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
COPY mcp_monitoring_dashboard.py .

# Create static directory for dashboard assets
RUN mkdir -p static/css static/js

# Expose ports
EXPOSE 7890

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:7890/health || exit 1

# Run the server
CMD ["python", "-u", "mcp_monitoring_dashboard.py"] 