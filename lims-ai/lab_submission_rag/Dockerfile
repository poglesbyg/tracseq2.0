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

# Copy requirements first for better caching
COPY requirements.txt .

# Install Python dependencies with memory optimizations
# Update pip first to avoid hash verification issues
RUN pip install --upgrade pip && \
    pip install --no-cache-dir --compile \
    --trusted-host pypi.org --trusted-host pypi.python.org --trusted-host files.pythonhosted.org \
    -r requirements.txt && \
    python -m compileall /usr/local/lib/python3.11/site-packages/ && \
    pip uninstall -y pip setuptools wheel

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
