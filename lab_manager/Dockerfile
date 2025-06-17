# syntax=docker/dockerfile:1

FROM rust:1.82-bookworm as builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency files first for better caching
COPY Cargo.toml ./

# Create dummy source files to build dependencies
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/main.rs && \
    echo "// lib.rs" > src/lib.rs && \
    echo "fn main() {}" > src/bin/create_admin.rs && \
    echo "fn main() {}" > src/bin/test_password.rs && \
    echo "fn main() {}" > src/bin/test_rag_client.rs

# Update dependencies to compatible versions and build
RUN cargo update && cargo build --release && rm -rf src/

# Copy the actual source code
COPY . .

# Skip SQLx offline preparation to avoid dependency conflicts
# For production, consider using a .sqlx directory committed to repo
# or building in an environment where database is available
ENV SQLX_OFFLINE=false

# Build the application (dependencies already cached)
RUN cargo build --release

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
RUN if [ -d "/usr/src/app/migrations" ]; then \
        cp -r /usr/src/app/migrations/* /usr/local/bin/migrations/ 2>/dev/null || true; \
    fi

# Set default environment variables (can be overridden at runtime)
ENV RUST_LOG=info
ENV STORAGE_PATH=/usr/local/bin/storage

EXPOSE 3000

CMD ["./lab_manager"] 
