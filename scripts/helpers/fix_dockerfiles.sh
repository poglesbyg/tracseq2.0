#!/bin/bash

echo "🔧 Fixing Dockerfiles for microservices..."
echo "========================================"

# List of services to fix
services=(
    "enhanced_storage_service"
    "sample_service"
    "notification_service"
    "sequencing_service"
    "qaqc_service"
    "library_details_service"
    "cognitive_assistant_service"
    "dashboard_service"
    "reports_service"
)

for service in "${services[@]}"; do
    dockerfile="$service/Dockerfile"
    
    if [ -f "$dockerfile" ]; then
        echo "📝 Fixing $dockerfile..."
        
        # Create a temporary fixed Dockerfile
        cat > "${dockerfile}.fixed" << 'DOCKEREOF'
FROM rustlang/rust:nightly-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the service's Cargo.toml (not root workspace)
COPY SERVICE_NAME/Cargo.toml ./

# Create dummy source for dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (cached layer)
RUN cargo build --release
RUN rm src/main.rs

# Copy actual source code
COPY SERVICE_NAME/src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false -m -d /app appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/SERVICE_NAME /app/SERVICE_NAME

# Change ownership
RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./SERVICE_NAME"]
DOCKEREOF
        
        # Replace SERVICE_NAME with actual service name
        sed -i.bak "s/SERVICE_NAME/$service/g" "${dockerfile}.fixed"
        
        # Move the fixed file to replace the original
        mv "${dockerfile}.fixed" "$dockerfile"
        rm -f "${dockerfile}.bak"
        
        echo "✅ Fixed $dockerfile"
    else
        echo "⚠️  $dockerfile not found, skipping..."
    fi
done

echo ""
echo "🎉 Dockerfile fixes complete!"
