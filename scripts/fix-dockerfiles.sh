#!/bin/bash

# Fix Dockerfiles to use service-specific Cargo.toml files

set -e

echo "🔄 Fixing Dockerfiles to use service-specific Cargo.toml files..."

# Auth Service
echo "🔧 Fixing auth_service Dockerfile..."
cat > auth_service/Dockerfile << 'EOF'
FROM rustlang/rust:nightly-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy service-specific Cargo.toml
COPY auth_service/Cargo.toml ./

# Create dummy source for dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Set offline mode for SQLx
ENV DATABASE_URL=postgres://postgres:postgres@localhost:5432/lab_manager
ENV SQLX_OFFLINE=true

# Build dependencies (this will be cached)
RUN cargo build --release
RUN rm src/main.rs

# Copy actual source code
COPY auth_service/src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false -m -d /app appuser

WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/auth_service /app/auth_service

# Change ownership
RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./auth_service"]
EOF

# Sample Service
echo "🔧 Fixing sample_service Dockerfile..."
cat > sample_service/Dockerfile << 'EOF'
FROM rustlang/rust:nightly-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy service-specific Cargo.toml
COPY sample_service/Cargo.toml ./

# Create dummy source for dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Set offline mode for SQLx
ENV DATABASE_URL=postgres://postgres:postgres@localhost:5432/lab_manager
ENV SQLX_OFFLINE=true

# Build dependencies (cached layer)
RUN cargo build --release
RUN rm src/main.rs

# Copy actual source code
COPY sample_service/src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false -m -d /app appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/sample_service /app/sample_service

# Change ownership
RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./sample_service"]
EOF

# Sequencing Service
echo "🔧 Fixing sequencing_service Dockerfile..."
cat > sequencing_service/Dockerfile << 'EOF'
FROM rustlang/rust:nightly-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy service-specific Cargo.toml
COPY sequencing_service/Cargo.toml ./

# Create dummy source for dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Set offline mode for SQLx
ENV DATABASE_URL=postgres://postgres:postgres@localhost:5432/lab_manager
ENV SQLX_OFFLINE=true

# Build dependencies (cached layer)
RUN cargo build --release
RUN rm src/main.rs

# Copy actual source code
COPY sequencing_service/src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false -m -d /app appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/sequencing_service /app/sequencing_service

# Change ownership
RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./sequencing_service"]
EOF

echo "✅ All Dockerfiles fixed!"
echo "📋 Fixed services: auth_service, sample_service, sequencing_service"
echo "🚀 Ready to test with fixed configurations!"

# Fix Dockerfile paths in lims-core services

echo "🔧 Fixing Dockerfile paths in lims-core services..."

# Find all Dockerfiles in lims-core
for dockerfile in lims-core/*/Dockerfile; do
    if [ -f "$dockerfile" ]; then
        service_dir=$(dirname "$dockerfile")
        service_name=$(basename "$service_dir")
        
        echo "  Fixing $service_name..."
        
        # Fix COPY commands that reference the service directory
        sed -i.bak -E "s|COPY $service_name/Cargo.toml|COPY Cargo.toml|g" "$dockerfile"
        sed -i.bak -E "s|COPY $service_name/src|COPY src|g" "$dockerfile"
        sed -i.bak -E "s|COPY $service_name/migrations|COPY migrations|g" "$dockerfile"
        
        # Clean up backup files
        rm -f "${dockerfile}.bak"
    fi
done

echo "✅ Dockerfile paths fixed!" 