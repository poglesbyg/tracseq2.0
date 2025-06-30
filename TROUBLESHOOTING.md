# 🔧 LIMS System Troubleshooting Guide

## Common Issues After Restructuring

### 1. Docker Build Failures

#### PostgreSQL pgvector Error
**Problem**: `make: clang-19: No such file or directory`

**Solution**: 
- Use the simplified Dockerfile: `Dockerfile.simple` (already configured)
- Or install pgvector manually after the container starts:
  ```bash
  docker exec -it lims-postgres psql -U postgres -c "CREATE EXTENSION IF NOT EXISTS vector;"
  ```

#### Service Not Found
**Problem**: Docker can't find service directories

**Solution**: Update paths in docker-compose.yml:
- Auth service: `../lims-core/auth_service`
- Sample service: `../lims-core/sample_service`
- API Gateway: `../lims-core/api_gateway`
- Frontend: `../lims-ui`

### 2. Service Connection Issues

#### Database Connection Failed
**Problem**: Services can't connect to PostgreSQL

**Solution**:
```bash
# Check if postgres is running
docker-compose ps postgres

# Check logs
docker-compose logs postgres

# Ensure DATABASE_URL is correct
# Should be: postgres://postgres:postgres@postgres:5432/lims_db
```

#### API Gateway Can't Find Services
**Problem**: 502 Bad Gateway errors

**Solution**:
- Ensure all services are running: `docker-compose ps`
- Check service URLs in API Gateway environment
- Verify network connectivity: `docker-compose exec api-gateway ping auth-service`

### 3. Frontend Issues

#### API Connection Failed
**Problem**: Frontend can't reach backend

**Solution**:
- Check VITE_API_URL in docker-compose.yml
- Should be: `http://localhost:8080` for API Gateway
- Rebuild frontend: `docker-compose build frontend`

### 4. Quick Fixes

#### Complete Reset
```bash
cd docker
docker-compose down -v  # Remove all containers and volumes
docker-compose build    # Rebuild all images
docker-compose up -d    # Start fresh
```

#### Check Service Health
```bash
# Check all services
docker-compose ps

# Check specific service logs
docker-compose logs -f service-name

# Check service health endpoint
curl http://localhost:8001/health  # Auth service
curl http://localhost:8002/health  # Sample service
```

#### Update Dependencies
```bash
# Rust services
cd lims-core
cargo update

# Frontend
cd lims-ui
pnpm update

# Python services
cd lims-ai
pip install -r requirements.txt --upgrade
```

### 5. Development Mode Issues

#### Hot Reload Not Working
**Solution**: Use development docker-compose or run locally:
```bash
# Rust service
cd lims-core/auth_service
cargo watch -x run

# Frontend
cd lims-ui
pnpm dev

# Python service
cd lims-ai/enhanced_rag_service
python -m src.main --reload
```

### 6. Performance Issues

#### Slow Container Startup
**Solution**:
- Increase Docker Desktop resources (CPU, Memory)
- Use `.dockerignore` to exclude unnecessary files
- Enable Docker BuildKit: `export DOCKER_BUILDKIT=1`

#### Database Performance
**Solution**:
- Check connection pool settings
- Monitor with: `docker-compose exec postgres pg_stat_activity`
- Tune PostgreSQL settings in docker-compose.yml

### 7. Getting Help

1. **Check Logs First**:
   ```bash
   docker-compose logs service-name
   ```

2. **Verify Configuration**:
   - Environment variables
   - Port mappings
   - Network settings

3. **Use Debug Mode**:
   ```bash
   # Set debug logging
   export RUST_LOG=debug
   docker-compose up service-name
   ```

4. **Common Commands**:
   ```bash
   # Restart a service
   docker-compose restart service-name
   
   # Rebuild a service
   docker-compose build service-name
   
   # View real-time logs
   docker-compose logs -f service-name
   
   # Execute commands in container
   docker-compose exec service-name sh
   ```

## Still Having Issues?

1. Check the service-specific README files in each directory
2. Review the `RESTRUCTURING_SUMMARY.md` for what changed
3. Ensure all dependencies are installed
4. Try the `quick-start.sh` script for automated setup

Remember: After restructuring, most issues are related to:
- Updated file paths
- Service discovery URLs
- Docker build contexts
- Environment variables

When in doubt, a clean restart often helps:
```bash
./quick-start.sh
``` 