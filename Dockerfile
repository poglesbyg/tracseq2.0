# syntax=docker/dockerfile:1

FROM rust:1.82-slim as builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Copy the source code
COPY . .

# Skip SQLx offline preparation to avoid dependency conflicts
# For production, consider using a .sqlx directory committed to repo
# or building in an environment where database is available
ENV SQLX_OFFLINE=false

# Build the application
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

# Copy migrations (if they exist)
COPY migrations /usr/local/bin/migrations || echo "No migrations directory found"

# Set environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=postgres://postgres:postgres@db:5432/lab_manager

EXPOSE 3000

CMD ["./lab_manager"] 
