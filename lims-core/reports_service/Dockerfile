# Build stage
FROM rustlang/rust:nightly-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests (use standalone version)
COPY Cargo.toml.standalone ./Cargo.toml

# Build dependencies (this is cached if Cargo.toml doesn't change)
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code (exclude files that might conflict)
COPY src ./src
RUN rm -f src/lib.rs  # Remove lib.rs as it references non-existent modules

# Use the standalone main.rs for binary-only build
COPY src/main_standalone.rs ./src/main.rs

COPY migrations ./migrations
COPY templates ./templates

# Build application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies including wkhtmltopdf for PDF generation
RUN apt-get update && \
    apt-get install -y \
        ca-certificates \
        libssl3 \
        libpq5 \
        wkhtmltopdf \
        fonts-liberation \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/reports-service /app/

# Copy migrations and templates
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/templates /app/templates

# Create data directories
RUN mkdir -p /data/reports /data/templates

# Create non-root user
RUN useradd -m -u 1001 appuser && \
    chown -R appuser:appuser /app /data

USER appuser

# Expose port
EXPOSE 3026

# Set environment variables
ENV RUST_LOG=info
ENV REPORTS_PORT=3026

# Run the binary
CMD ["/app/reports-service"]