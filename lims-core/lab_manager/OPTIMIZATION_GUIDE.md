# Container Optimization Guide

## Current vs Optimized Memory Usage

### Current State
- **Development Container**: 475MB memory / 2.35GB image
- **Production Container**: ~80MB memory / 147MB image  
- **PostgreSQL**: 45MB memory / 608MB image
- **Total**: ~520MB memory usage

### Optimized Targets
- **Development Container**: ~200MB memory / ~800MB image
- **Production Container**: ~40MB memory / ~15MB image
- **PostgreSQL**: ~30MB memory / ~238MB image (Alpine)
- **Total Expected**: ~250MB memory usage (**52% reduction**)

## Optimization Strategies

### 1. Ultra-Light Production (Dockerfile.alpine)
```bash
# Build ultra-minimal production image (~15MB)
docker build -f Dockerfile.alpine -t lab_manager:alpine .
```

**Features:**
- `FROM scratch` (no base OS)
- Static musl binary
- Size-optimized Rust flags
- Stripped symbols
- **Expected**: ~15MB image, ~40MB memory

### 2. Light Production with Better Compatibility (Dockerfile.alpine-light)
```bash
# Build lightweight production image (~25MB)
docker build -f Dockerfile.alpine-light -t lab_manager:alpine-light .
```

**Features:**
- Alpine Linux base (~5MB)
- Non-root user security
- Basic system utilities
- **Expected**: ~25MB image, ~45MB memory

### 3. Optimized Development (Dockerfile.dev-alpine)
```bash
# Build lightweight development image (~800MB vs 2.35GB)
docker build -f Dockerfile.dev-alpine -t lab_manager:dev-alpine .
```

**Features:**
- Alpine-based Rust environment
- Cached dependency builds
- Efficient cargo-watch
- **Expected**: ~800MB image, ~200MB memory

### 4. Memory-Limited Compose (docker-compose.lightweight.yml)
```bash
# Run with memory limits
docker-compose -f docker-compose.lightweight.yml up
```

**Resource Limits:**
- Dev container: 512MB limit
- Production: 128MB limit
- PostgreSQL: 256MB limit
- Frontend: 64MB limit

## Quick Start Commands

### Switch to Lightweight Setup
```bash
# Stop current containers
docker-compose down

# Build and run optimized containers
docker-compose -f docker-compose.lightweight.yml up --build
```

### Ultra-Minimal Production
```bash
# Build the smallest possible image
docker build -f Dockerfile.alpine -t lab_manager:minimal .

# Run with minimal resources
docker run -p 3000:3000 \
  -e DATABASE_URL="postgresql://user:pass@host:5432/db" \
  --memory=64m \
  lab_manager:minimal
```

### Custom Size-Optimized Build
```bash
# Build with ultra-small profile
docker build -f Dockerfile.alpine-light \
  --build-arg CARGO_PROFILE=release-small \
  -t lab_manager:ultra-small .
```

## Advanced Optimizations

### Binary Size Reduction
The Cargo.toml includes optimized profiles:

```toml
[profile.release]
opt-level = "s"          # Size optimization
lto = true               # Link-time optimization
codegen-units = 1        # Smaller binaries
panic = "abort"          # Remove unwinding
strip = true             # Strip symbols

[profile.release-small]
opt-level = "z"          # Maximum size optimization
lto = "fat"              # Full LTO
```

### PostgreSQL Optimization
The lightweight compose includes tuned PostgreSQL settings:

- `shared_buffers=64MB` (reduced from default 128MB)
- `effective_cache_size=128MB` (reduced from default 4GB)
- `maintenance_work_mem=16MB` (reduced from default 64MB)
- Alpine-based PostgreSQL image (238MB vs 608MB)

### Build Cache Optimization
```bash
# Pre-build dependencies for faster rebuilds
docker build --target=builder -f Dockerfile.alpine -t lab_manager:deps .

# Use cache from previous build
docker build --cache-from=lab_manager:deps -f Dockerfile.alpine -t lab_manager:alpine .
```

## Memory Monitoring

### Check Current Usage
```bash
# Monitor container memory usage
docker stats --no-stream

# Check image sizes
docker images | grep lab_manager
```

### Memory Profiling
```bash
# Profile memory usage in development
docker exec -it lab_dev sh -c "ps aux | grep lab_manager"

# Check binary size
docker exec -it lab_dev sh -c "ls -lh target/*/release/lab_manager"
```

## Troubleshooting

### If Alpine Build Fails
```bash
# Add musl target
docker exec -it lab_dev cargo install --target x86_64-unknown-linux-musl

# Or use glibc version
docker build -f Dockerfile.alpine-light -t lab_manager:light .
```

### Database Connection Issues
```bash
# Ensure PostgreSQL is accessible
docker exec -it lab_manager-db-1 psql -U postgres -c "SELECT version();"
```

### Performance Concerns
The optimized builds prioritize size over speed. For performance-critical production:

```bash
# Use balanced optimization
docker build -f Dockerfile.alpine-light \
  --build-arg CARGO_PROFILE=release \
  -t lab_manager:balanced .
```

## Expected Results

| Configuration | Image Size | Memory Usage | Build Time |
|---------------|------------|--------------|------------|
| Current Dev   | 2.35GB     | 475MB        | 3-5 min    |
| Alpine Dev    | ~800MB     | ~200MB       | 2-3 min    |
| Current Prod  | 147MB      | ~80MB        | 3-5 min    |
| Alpine Light  | ~25MB      | ~45MB        | 2-3 min    |
| Alpine Minimal| ~15MB      | ~40MB        | 2-3 min    |

**Total Memory Reduction: ~52% (520MB → 250MB)** 
