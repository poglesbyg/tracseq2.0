# Multi-stage build for TracSeq Enhanced Storage Service
# Stage 1: Build stage
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user for security
RUN useradd -m -u 1001 appuser

# Set work directory
WORKDIR /usr/src/app

# Copy manifest files
COPY Cargo.toml ./

# Create dummy source to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached unless Cargo.toml changes)
RUN cargo build --release && \
    rm -rf src target/release/deps/enhanced_storage_service*

# Copy actual source code
COPY src ./src

# Build the application
RUN cargo build --release

# Verify the binary was built correctly
RUN ls -la target/release/ && \
    ldd target/release/enhanced-storage-service || true

# Stage 2: Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 appuser

# Create necessary directories
RUN mkdir -p /app/logs /app/config && \
    chown -R appuser:appuser /app

# Copy the binary from builder stage
COPY --from=builder /usr/src/app/target/release/enhanced-storage-service /app/enhanced-storage-service

# Set permissions
RUN chmod +x /app/enhanced-storage-service && \
    chown appuser:appuser /app/enhanced-storage-service

# Switch to non-root user
USER appuser

# Set work directory
WORKDIR /app

# Expose port
EXPOSE 8082

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8082/health || exit 1

# Environment variables
ENV RUST_LOG=info \
    RUST_BACKTRACE=1 \
    PORT=8082

# Run the binary
CMD ["./enhanced-storage-service"] 
