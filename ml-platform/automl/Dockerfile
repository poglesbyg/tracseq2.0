FROM python:3.9-slim

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application
COPY automl_framework.py .

# Create model directory
RUN mkdir -p /models/automl

EXPOSE 8096

CMD ["python", "automl_framework.py"]
