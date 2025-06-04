# 🛠️ Development Setup Guide

## Quick Start

### **🚀 One-Command Setup**
```bash
./scripts/run.sh
```

This will start all services including:
- **Frontend Development Server** (http://localhost:5173)
- **Backend API** (http://localhost:3000)
- **PostgreSQL Database** (localhost:5432)

### **🔧 Manual Setup**

If you prefer manual control:

```bash
# 1. Start the database
docker-compose up -d db

# 2. Start the development backend
docker-compose up -d dev

# 3. Start the frontend (optional)
docker-compose up -d frontend-dev
```

## 🧱 **Modular Development Approach**

Thanks to our IKEA-style modular architecture, you can work on components independently:

### **Component-Specific Development**
```bash
# Test specific components
cargo test handlers --verbose
cargo test storage --verbose  
cargo test config --verbose
cargo test assembly --verbose

# Run component-specific code
cargo run --bin component_name  # (when available)
```

### **Hot Reloading**
The development setup includes `cargo-watch` for automatic rebuilding:
- **Backend**: Automatically rebuilds on Rust code changes
- **Frontend**: Hot module replacement for instant feedback

## 🗄️ **Database Setup**

### **SQLx Online Mode**
We use SQLx in **online mode** for development to avoid version compatibility issues:

- ✅ **Pros**: No build-time dependency conflicts
- ✅ **Pros**: Always uses latest database schema
- ✅ **Pros**: Easier to set up and maintain
- ⚠️ **Note**: Requires database connection during compilation

### **Environment Variables**
```bash
# Core settings (automatically set by docker-compose)
DATABASE_URL=postgres://postgres:postgres@db:5432/lab_manager
STORAGE_PATH=/usr/local/bin/storage
SQLX_OFFLINE=false  # Use online mode
RUST_LOG=info       # Enable logging
```

## 🔧 **Troubleshooting**

### **Common Issues**

#### **Port Already in Use**
```bash
# Check what's using the ports
lsof -i :3000  # Backend
lsof -i :5173  # Frontend
lsof -i :5432  # Database

# Stop existing services
docker-compose down
```

#### **Database Connection Issues**
```bash
# Check database status
docker-compose ps db

# View database logs
docker-compose logs db

# Reset database
docker-compose down -v  # This removes volumes!
docker-compose up -d db
```

#### **Build Issues**
```bash
# Clean build
cargo clean
docker-compose build --no-cache dev

# Check for dependency conflicts
cargo check
```

### **SQLx-Specific Issues**

If you see SQLx compilation errors:

1. **Check database connection**:
   ```bash
   docker-compose ps db  # Should show "Up"
   ```

2. **Verify environment variables**:
   ```bash
   echo $DATABASE_URL  # Should be set
   echo $SQLX_OFFLINE  # Should be "false"
   ```

3. **Test database connection**:
   ```bash
   docker-compose exec db psql -U postgres -d lab_manager -c "SELECT 1;"
   ```

## 🧪 **Testing**

### **Component Testing**
```bash
# Test all components in parallel (fast!)
cargo test

# Test specific component
cargo test handlers
cargo test storage
cargo test config
cargo test assembly

# Integration tests (requires database)
cargo test --test integration
```

### **End-to-End Testing**
```bash
# Start full environment
./scripts/run.sh

# Run E2E tests (if available)
npm run test:e2e  # Frontend
cargo test --test e2e  # Backend
```

## 📊 **Development Tools**

### **Logging**
```bash
# Adjust log levels
export RUST_LOG=debug  # More verbose
export RUST_LOG=error  # Less verbose

# Component-specific logging
export RUST_LOG=lab_manager::handlers=debug
```

### **Database Tools**
```bash
# Connect to database
docker-compose exec db psql -U postgres -d lab_manager

# View database logs
docker-compose logs -f db

# Database backup/restore
docker-compose exec db pg_dump -U postgres lab_manager > backup.sql
docker-compose exec -T db psql -U postgres lab_manager < backup.sql
```

### **Performance Monitoring**
```bash
# Build performance
time cargo build

# Runtime performance
cargo build --release
time ./target/release/lab_manager

# Memory usage
valgrind ./target/release/lab_manager  # (requires valgrind)
```

## 🔄 **Development Workflow**

### **Daily Development**
1. **Start environment**: `./scripts/run.sh`
2. **Make changes**: Edit code in your IDE
3. **Test changes**: Automatic rebuilding with cargo-watch
4. **Test components**: `cargo test component_name`
5. **Check CI/CD**: Push commits trigger automated testing

### **Adding New Features**
1. **Create component**: Follow modular architecture patterns
2. **Add tests**: Component-specific test files
3. **Update documentation**: Keep docs current
4. **Test integration**: Ensure components work together
5. **Create PR**: CI/CD will validate everything

### **Component Development**
```bash
# Create new component
mkdir src/my_component
touch src/my_component/mod.rs

# Add to main modules
echo "pub mod my_component;" >> src/main.rs

# Create tests
mkdir src/my_component/tests
touch src/my_component/tests/mod.rs
```

## 🚀 **Production Build**

### **Local Production Build**
```bash
# Build production Docker image
docker build -t lab_manager:latest .

# Run production container
docker run -p 3000:3000 \
  -e DATABASE_URL=your_production_db \
  lab_manager:latest
```

### **CI/CD Pipeline**
- **Automatic**: Triggered on push to master
- **Manual**: Use GitHub Actions workflow dispatch
- **Environments**: Staging → Production pipeline

## 📚 **Additional Resources**

- [Modular Architecture Guide](./MODULAR_ARCHITECTURE.md)
- [CI/CD Guide](./CI_CD_GUIDE.md)
- [Component Development](./COMPONENT_GUIDE.md)
- [API Documentation](./API.md)

## 🎯 **Quick Commands Reference**

```bash
# Start everything
./scripts/run.sh

# Stop everything  
docker-compose down

# Restart specific service
docker-compose restart dev

# View logs
docker-compose logs -f dev

# Clean rebuild
docker-compose down -v
docker-compose build --no-cache
docker-compose up -d

# Test specific component
cargo test handlers --verbose

# Check all components
cargo check
```

---

*🧱 This development setup leverages your modular architecture for maximum development efficiency and minimal setup friction!* 
