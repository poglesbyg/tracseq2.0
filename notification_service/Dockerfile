# Multi-stage build for TracSeq Notification Service
# Stage 1: Build stage
FROM rust:1.82-slim as builder

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
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached unless Cargo.toml changes)
RUN cargo build --release && \
    rm -rf src target/release/deps/notification_service*

# Copy actual source code
COPY src ./src

# Build the application
RUN cargo build --release

# Verify the binary was built correctly
RUN ls -la target/release/ && \
    ldd target/release/notification-service || true

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
COPY --from=builder /usr/src/app/target/release/notification-service /app/notification-service

# Set permissions
RUN chmod +x /app/notification-service && \
    chown appuser:appuser /app/notification-service

# Switch to non-root user
USER appuser

# Set work directory
WORKDIR /app

# Expose port
EXPOSE 8085

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8085/health || exit 1

# Environment variables
ENV RUST_LOG=info \
    RUST_BACKTRACE=1 \
    PORT=8085

# Run the binary
CMD ["./notification-service"] 
