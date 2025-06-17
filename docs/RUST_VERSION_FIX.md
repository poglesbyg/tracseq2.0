# 🦀 Rust Version Compatibility Fix

This document explains the Rust version compatibility issue encountered during Docker builds and how it was resolved.

## 🐛 The Problem

When building the TracSeq 2.0 lab_manager service, users might encounter this error:

```
error: failed to download replaced source registry `crates-io`

Caused by:
  failed to parse manifest at `/usr/local/cargo/registry/src/index.crates.io-6f17d22bba15001f/base64ct-1.8.0/Cargo.toml`

Caused by:
  feature `edition2024` is required

  The package requires the Cargo feature called `edition2024`, but that feature is not stabilized in this version of Cargo (1.75.0).
  Consider trying a newer version of Cargo (this may require the nightly release).
```

## 🔍 Root Cause

This error occurs because:

1. **Outdated Rust Version**: The original Dockerfile used `rustlang/rust:nightly`, which pulled an older nightly version with Cargo 1.75.0 (from November 2023)
2. **Dependency Requirements**: The `base64ct-1.8.0` crate requires the `edition2024` feature, which wasn't available in Cargo 1.75.0
3. **Version Mismatch**: The locked dependency versions in `Cargo.lock` were incompatible with the older Rust version

## ✅ The Solution

### 1. Updated Rust Version

**Before:**
```dockerfile
FROM rustlang/rust:nightly as builder
```

**After:**
```dockerfile
FROM rust:1.80-bookworm as builder
```

**Why this works:**
- Rust 1.80 is stable and recent enough to support modern dependencies
- The `bookworm` variant provides a stable Debian base
- No need for nightly features since the project uses `edition = "2021"`

### 2. Dynamic Dependency Resolution

**Before:**
```dockerfile
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release && rm -rf src/
```

**After:**
```dockerfile
COPY Cargo.toml ./
RUN cargo update && cargo build --release && rm -rf src/
```

**Why this works:**
- Removes the locked `Cargo.lock` file to allow fresh dependency resolution
- `cargo update` ensures compatible versions are selected
- Dependencies are resolved at build time rather than using potentially outdated versions

### 3. Applied to Both Dockerfiles

The fix was applied to both:
- `lab_manager/Dockerfile` (production builds)
- `lab_manager/Dockerfile.dev` (development builds)

## 🧪 Testing the Fix

### Automated Testing

Use the provided test scripts to verify builds work:

```bash
# Linux/macOS
./scripts/test-build.sh

# Windows PowerShell
./scripts/test-build.ps1
```

### Manual Testing

```bash
# Test backend build
cd lab_manager
docker build -f Dockerfile -t test-backend .

# Test development build
docker build -f Dockerfile.dev -t test-backend-dev .

# Clean up
docker rmi test-backend test-backend-dev
```

## 🚀 Building from Scratch

If you encounter this issue in an existing project:

1. **Update Dockerfile:**
   ```dockerfile
   FROM rust:1.80-bookworm as builder
   ```

2. **Remove Cargo.lock from Docker context:**
   ```dockerfile
   COPY Cargo.toml ./
   # Don't copy Cargo.lock
   ```

3. **Add cargo update:**
   ```dockerfile
   RUN cargo update && cargo build --release
   ```

4. **Rebuild images:**
   ```bash
   docker-compose down
   docker-compose build --no-cache
   docker-compose up -d
   ```

## 🔧 For Local Development

If you're developing locally with Rust installed:

```bash
# Update Rust toolchain
rustup update

# Update dependencies
cargo update

# Test build
cargo build
```

## 📈 Version Requirements

### Minimum Versions
- **Rust**: 1.75+ (for basic compatibility)
- **Cargo**: 1.75+ (for most features)
- **Recommended**: Rust 1.80+ (for latest dependency support)

### Docker Images
- ✅ **Recommended**: `rust:1.80-bookworm` 
- ✅ **Alternative**: `rust:1.81-bookworm` (or newer)
- ❌ **Avoid**: `rustlang/rust:nightly` (can be unstable/outdated)

## 🛡️ Prevention

To avoid this issue in the future:

1. **Pin Rust versions** in Dockerfiles rather than using `nightly`
2. **Test builds regularly** with the provided test scripts
3. **Update dependencies periodically** with `cargo update`
4. **Monitor Rust releases** and update versions accordingly

## 🔗 Related Resources

- [Rust Release Notes](https://forge.rust-lang.org/infra/channel-layout.html)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Docker Rust Images](https://hub.docker.com/_/rust)
- [TracSeq Build Test Scripts](../scripts/)

## 📞 Support

If you continue to experience build issues:

1. **Run diagnostics:**
   ```bash
   docker --version
   docker-compose --version
   ./scripts/test-build.sh  # or .ps1 on Windows
   ```

2. **Check system requirements:**
   - Docker Desktop running
   - 5GB+ free disk space
   - 8GB+ RAM
   - Internet connection for downloads

3. **Clean Docker cache:**
   ```bash
   docker system prune -a
   docker-compose build --no-cache
   ```

4. **Create GitHub issue** with:
   - Error logs
   - System info (OS, Docker version)
   - Steps to reproduce

---

**🎉 With these fixes, TracSeq 2.0 builds reliably on modern Rust versions!** 
