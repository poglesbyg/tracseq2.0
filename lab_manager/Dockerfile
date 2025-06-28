# syntax=docker/dockerfile:1

FROM rustlang/rust:nightly-bookworm as builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files for dependency resolution
COPY Cargo.toml Cargo.lock ./
COPY lab_manager/Cargo.toml ./lab_manager/

# Create dummy source files to build dependencies  
RUN mkdir -p lab_manager/src/bin && \
    echo "fn main() {}" > lab_manager/src/main.rs && \
    echo "// lib.rs" > lab_manager/src/lib.rs && \
    echo "fn main() {}" > lab_manager/src/bin/create_admin.rs && \
    echo "fn main() {}" > lab_manager/src/bin/test_password.rs && \
    echo "fn main() {}" > lab_manager/src/bin/test_rag_client.rs

# Update dependencies to compatible versions and build
RUN cargo build --release --bin lab_manager && rm -rf lab_manager/src/

# Copy the actual source code
COPY lab_manager/ ./lab_manager/

# Skip SQLx offline preparation to avoid dependency conflicts
# For production, consider using a .sqlx directory committed to repo
# or building in an environment where database is available
ENV SQLX_OFFLINE=false

# Build the application (dependencies already cached)
RUN cargo build --release --bin lab_manager

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/lab_manager .

# Copy migrations if they exist
RUN mkdir -p /usr/local/bin/migrations
RUN --mount=from=builder,source=/usr/src/app/lab_manager/migrations,target=/tmp/migrations \
    cp -r /tmp/migrations/* /usr/local/bin/migrations/ 2>/dev/null || true

# Set default environment variables (can be overridden at runtime)
ENV RUST_LOG=info
ENV STORAGE_PATH=/usr/local/bin/storage

EXPOSE 3000

CMD ["./lab_manager"] 
