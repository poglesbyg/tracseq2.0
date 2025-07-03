# TracSeq 2.0 Unified Docker Migration

## Overview
This document describes the migration from multiple docker-compose files to a unified configuration, addressing database connection issues, schema mismatches, and inconsistent networking.

## Issues Addressed

### 1. Database Connection Problems ✅ FIXED
**Problem**: Services were configured to connect to different database hostnames:
- Some used `lims-postgres`
- Some used `postgres`  
- Some used `tracseq-postgres`

**Solution**: Created unified configuration where all services use consistent hostname `postgres` within the Docker network.

### 2. Multiple Docker Compose Files ✅ FIXED  
**Problem**: Multiple compose files caused network and naming inconsistencies:
- `docker-compose.yml`
- `docker-compose.basic.yml`
- `docker-compose.enhanced-services.yml`
- And many others...

**Solution**: Created `docker/docker-compose.unified.yml` that consolidates all services with:
- Consistent naming convention (tracseq-*)
- Single network (tracseq-network)
- Unified port mappings
- Consistent environment variables

### 3. Schema Mismatches ⚠️ PARTIALLY FIXED
**Problem**: Database tables had different schemas than what services expected.

**Fixes Applied**:
- ✅ Fixed `rate_limits` table (renamed `key_identifier` to `identifier`)
- ✅ Created missing `sessions` table for auth service
- ✅ Added missing indexes and triggers
- ⚠️ Some services still have issues with missing binaries or configuration

## New Unified Configuration

### Service Port Mappings
```
PostgreSQL:        5433 (external) -> 5432 (internal)
Redis:             6380 (external) -> 6379 (internal)
API Gateway:       8089 (external) -> 8000 (internal)
Auth Service:      8011 (external) -> 8001 (internal)
Sample Service:    8012 (external) -> 8081 (internal)
Storage Service:   8013 (external) -> 8082 (internal)
Reports Service:   8014 (external) -> 8000 (internal)
Notification:      8015 (external) -> 8085 (internal)
Event Service:     8016 (external) -> 8087 (internal)
Transaction:       8017 (external) -> 8088 (internal)
Template Service:  8018 (external) -> 8083 (internal)
Sequencing:        8019 (external) -> 8084 (internal)
RAG Service:       8100 (external) -> 8000 (internal)
Ollama:            11434 (external) -> 11434 (internal)
Frontend:          3000 (external) -> 80 (internal)
```

### Current Service Status

✅ **Working Services**:
- PostgreSQL (healthy)
- Redis (healthy)
- Template Service (healthy)
- Transaction Service (healthy)  
- Sample Service (health: starting)

❌ **Services with Issues**:
- Auth Service (restarting - migration issues)
- Storage Service (restarting - binary/config issues)
- Event Service (restarting)
- Notification Service (restarting)

🚧 **Not Yet Started**:
- API Gateway
- Sequencing Service
- Reports Service
- RAG Service
- Frontend

## Usage

### Starting the Unified Stack
```bash
cd docker
docker-compose -f docker-compose.unified.yml up -d
```

### Checking Service Health
```bash
docker-compose -f docker-compose.unified.yml ps
```

### Viewing Logs
```bash
docker-compose -f docker-compose.unified.yml logs -f [service-name]
```

### Restarting Services
```bash
docker-compose -f docker-compose.unified.yml restart [service-name]
```

## Migration Script
A migration script is available at `scripts/migrate-to-unified.sh` that:
1. Stops all old containers
2. Removes old networks
3. Starts the unified stack
4. Applies database migrations
5. Checks service health

## Database Migrations Applied

1. `002_fix_rate_limits_table.sql` - Fixed rate_limits table schema
2. `003_create_auth_tables.sql` - Created missing sessions table
3. `999_fix_all_schemas.sql` - Comprehensive schema fixes

## Next Steps

1. **Fix Remaining Service Issues**:
   - Debug why storage, event, and notification services are failing to start
   - Ensure all service binaries are properly built and copied

2. **Complete Service Integration**:
   - Start API Gateway once core services are stable
   - Deploy frontend with proper API Gateway connection
   - Add remaining services (RAG, Sequencing, Reports)

3. **Environment Configuration**:
   - Create proper `.env` files for different environments
   - Set up secrets management for production
   - Configure SSL/TLS for production deployment

4. **Testing**:
   - Run integration tests with unified stack
   - Verify all service-to-service communications
   - Test database migrations and rollbacks

## Troubleshooting

### Service Won't Start
```bash
# Check logs
docker-compose -f docker-compose.unified.yml logs [service-name]

# Check if binary exists
docker exec [container-name] ls -la /app/

# Check environment variables
docker-compose -f docker-compose.unified.yml config
```

### Database Connection Issues
```bash
# Test database connection
docker exec tracseq-postgres psql -U postgres -d lims_db -c "SELECT 1"

# Check network connectivity
docker exec [service] ping postgres
```

### Port Conflicts
```bash
# Check what's using a port
lsof -i :PORT_NUMBER

# Change port in docker-compose.unified.yml if needed
```

## Benefits of Unified Configuration

1. **Consistency**: All services use the same naming convention and network
2. **Simplicity**: Single file to manage all services
3. **Reliability**: Proper health checks and dependencies
4. **Maintainability**: Clear port mappings and environment variables
5. **Scalability**: Easy to add new services following the same pattern

*Context improved by Giga AI* 