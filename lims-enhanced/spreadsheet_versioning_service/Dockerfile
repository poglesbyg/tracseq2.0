# Build stage
FROM rust:1.80-alpine as builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig

# Create app directory
WORKDIR /usr/src/app

# Copy manifests
COPY Cargo.toml ./

# Create src directory with a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies
RUN cargo build --release
RUN rm src/main.rs

# Copy source code
COPY src ./src

# Build application
RUN cargo build --release

# Runtime stage
FROM alpine:3.18

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    openssl

# Create app user
RUN addgroup -g 1001 -S appuser && \
    adduser -S appuser -u 1001 -G appuser

# Create app directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /usr/src/app/target/release/spreadsheet_versioning_service /app/

# Change ownership to app user
RUN chown -R appuser:appuser /app
USER appuser

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8088/health || exit 1

# Expose port
EXPOSE 8088

# Run the application
CMD ["./spreadsheet_versioning_service"] 
