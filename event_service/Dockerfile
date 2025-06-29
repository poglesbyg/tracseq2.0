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
