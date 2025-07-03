# Memory-Optimized RAG Service Dockerfile
FROM python:3.11-slim

# Install system dependencies (minimal set)
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

WORKDIR /app

# Copy requirements and install dependencies
COPY requirements.simple.txt ./requirements.txt
RUN pip install --upgrade pip && \
    pip install --no-cache-dir -r requirements.txt

# Copy application code
COPY . .

# Create necessary directories
RUN mkdir -p uploads exports logs data

# Memory optimization: Use smaller Python allocator
ENV PYTHONMALLOC=malloc
ENV MALLOC_TRIM_THRESHOLD_=100000
ENV MALLOC_TOP_PAD_=100000
ENV MALLOC_MMAP_THRESHOLD_=100000

# Set memory-optimized Python flags
ENV PYTHONOPTIMIZE=1
ENV PYTHONDONTWRITEBYTECODE=1
ENV PYTHONUNBUFFERED=1

# Run the simple frontend bridge (no complex dependencies)
CMD ["python", "simple_frontend_bridge.py"] 
