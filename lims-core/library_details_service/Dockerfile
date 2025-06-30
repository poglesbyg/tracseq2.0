# Multi-stage build for library_details_service
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
COPY library_details_service/ ./library_details_service/

# Build the service
WORKDIR /usr/src/app/library_details_service
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
COPY --from=builder /usr/src/app/target/release/library-details-service ./service

# Expose default port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info

CMD ["./service"]
