# Build stage
FROM rustlang/rust:nightly-bookworm as builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Copy manifest
COPY Cargo.toml.standalone ./Cargo.toml

# Create dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code and migrations
COPY src ./src
COPY migrations ./migrations

# Build the actual application
RUN touch src/main.rs
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

# Copy the binary and migrations
COPY --from=builder /usr/src/app/target/release/qaqc_service ./qaqc_service
COPY --from=builder /usr/src/app/migrations ./migrations

# Expose port
EXPOSE 8103

# Set environment variables
ENV RUST_LOG=info

# Run the binary
CMD ["./qaqc_service"] 