FROM python:3.9-slim

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    docker.io \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application
COPY mlops_pipeline.py .

# Create model directory
RUN mkdir -p /models

EXPOSE 8097

CMD ["python", "mlops_pipeline.py"]
