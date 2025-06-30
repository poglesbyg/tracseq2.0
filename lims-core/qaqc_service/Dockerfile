# Multi-stage build for qaqc_service
FROM rustlang/rust:nightly-bookworm as builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml ./
COPY qaqc_service/ ./qaqc_service/

# Build the service
WORKDIR /usr/src/app/qaqc_service
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary
COPY --from=builder /usr/src/app/target/release/qaqc-service ./service

# Expose default port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info

CMD ["./service"]
