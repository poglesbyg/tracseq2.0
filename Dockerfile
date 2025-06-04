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

# Prepare SQLx cache
RUN cargo install sqlx-cli --version 0.7.4 --no-default-features --features postgres
RUN cargo sqlx prepare -- --lib

# Set SQLx to offline mode during build
ENV SQLX_OFFLINE=true

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

# Copy migrations
COPY migrations /usr/local/bin/migrations

# Copy SQLx cache
COPY --from=builder /usr/src/app/.sqlx /usr/local/bin/.sqlx

# Set environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=postgres://postgres:postgres@db:5432/lab_manager

EXPOSE 3000

CMD ["./lab_manager"] 
