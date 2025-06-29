#!/bin/bash

# Fix remaining Dockerfiles to use service-specific Cargo.toml files

set -e

echo "🔄 Fixing remaining Dockerfiles..."

# Enhanced Storage Service
echo "🔧 Fixing enhanced_storage_service Dockerfile..."
cat > enhanced_storage_service/Dockerfile << 'EOF'
FROM rustlang/rust:nightly-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy service-specific Cargo.toml
COPY enhanced_storage_service/Cargo.toml ./

# Create dummy source for dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Set offline mode for SQLx
ENV DATABASE_URL=postgres://postgres:postgres@localhost:5432/lab_manager
ENV SQLX_OFFLINE=true

# Build dependencies (cached layer)
RUN cargo build --release
RUN rm src/main.rs

# Copy actual source code
COPY enhanced_storage_service/src ./src

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
COPY --from=builder /app/target/release/enhanced_storage_service /app/enhanced_storage_service

# Change ownership
RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./enhanced_storage_service"]
EOF

# Notification Service
echo "🔧 Fixing notification_service Dockerfile..."
cat > notification_service/Dockerfile << 'EOF'
FROM rustlang/rust:nightly-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy service-specific Cargo.toml
COPY notification_service/Cargo.toml ./

# Create dummy source for dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Set offline mode for SQLx
ENV DATABASE_URL=postgres://postgres:postgres@localhost:5432/lab_manager
ENV SQLX_OFFLINE=true

# Build dependencies (cached layer)
RUN cargo build --release
RUN rm src/main.rs

# Copy actual source code
COPY notification_service/src ./src

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
COPY --from=builder /app/target/release/notification_service /app/notification_service

# Change ownership
RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./notification_service"]
EOF

# Event Service
echo "🔧 Fixing event_service Dockerfile..."
cat > event_service/Dockerfile << 'EOF'
FROM rustlang/rust:nightly-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy service-specific Cargo.toml
COPY event_service/Cargo.toml ./

# Create dummy source for dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Set offline mode for SQLx
ENV DATABASE_URL=postgres://postgres:postgres@localhost:5432/lab_manager
ENV SQLX_OFFLINE=true

# Build dependencies (cached layer)
RUN cargo build --release
RUN rm src/main.rs

# Copy actual source code
COPY event_service/src ./src

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
COPY --from=builder /app/target/release/event_service /app/event_service

# Change ownership
RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./event_service"]
EOF

echo "✅ All remaining Dockerfiles fixed!"
echo "📋 Fixed services: enhanced_storage_service, notification_service, event_service"
echo "🚀 Ready to test migration with all fixes!" 