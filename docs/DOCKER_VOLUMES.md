# Docker Volume Persistence Guide

This document describes the Docker volume persistence setup for TracSeq 2.0, ensuring data survives container restarts and system reboots.

## Overview

TracSeq 2.0 uses Docker volumes to persist critical data across container lifecycle events. This ensures that:

- Database data is preserved
- Application files and uploads are retained
- Log files are maintained for debugging
- Configuration and templates persist

## Volume Configuration

### Database and Cache Volumes

| Volume | Purpose | Mount Point | Service |
|--------|---------|-------------|---------|
| `postgres_data` | PostgreSQL database files | `/var/lib/postgresql/data` | postgres |
| `redis_data` | Redis persistence files | `/data` | redis |

### Application Data Volumes

| Volume | Purpose | Mount Point | Services |
|--------|---------|-------------|----------|
| `app_storage` | Application storage files | `/app/storage` | dashboard-service, samples-service, sequencing-service |
| `app_uploads` | File uploads | `/app/uploads` | dashboard-service, samples-service, sequencing-service |
| `app_logs` | Application logs | `/app/logs` | All services |
| `templates_data` | Template files | `/app/templates` | spreadsheet-service |

### Development Cache Volumes

| Volume | Purpose | Mount Point | Services |
|--------|---------|-------------|----------|
| `node_modules` | Node.js dependencies | `/app/node_modules` | Frontend services |
| `cargo_cache` | Rust build cache | `/usr/local/cargo/registry` | Rust services |

## Volume Lifecycle

### Automatic Creation

Volumes are automatically created when services start:

```bash
docker-compose -f docker-compose.development.yml up -d
```

### Manual Volume Management

```bash
# List all TracSeq volumes
docker volume ls | grep tracseq20

# Inspect a specific volume
docker volume inspect tracseq20_postgres_data

# Remove all volumes (⚠️ This will delete all data!)
docker-compose -f docker-compose.development.yml down -v
```

## Data Persistence Verification

### Quick Test

```bash
# Test database persistence
docker-compose -f docker-compose.development.yml exec postgres psql -U postgres -d lab_manager -c "SELECT version();"

# Test Redis persistence
docker-compose -f docker-compose.development.yml exec redis redis-cli ping

# Test application storage
docker-compose -f docker-compose.development.yml exec dashboard-service ls -la /app/storage/
```

### Comprehensive Test

Run the automated persistence test:

```bash
./scripts/test-persistence.sh
```

This script:
1. Creates test data in all persistent volumes
2. Restarts all services
3. Verifies data persistence
4. Cleans up test data

## Backup and Restore

### Database Backup

```bash
# Create PostgreSQL backup
docker-compose -f docker-compose.development.yml exec postgres pg_dump -U postgres lab_manager > backup.sql

# Restore from backup
docker-compose -f docker-compose.development.yml exec -T postgres psql -U postgres lab_manager < backup.sql
```

### Volume Backup

```bash
# Backup all volumes
docker run --rm -v tracseq20_postgres_data:/data -v $(pwd):/backup ubuntu tar czf /backup/postgres_backup.tar.gz /data

# Restore volume
docker run --rm -v tracseq20_postgres_data:/data -v $(pwd):/backup ubuntu tar xzf /backup/postgres_backup.tar.gz -C /
```

### Application Data Backup

```bash
# Backup application storage
docker-compose -f docker-compose.development.yml exec dashboard-service tar czf /tmp/storage_backup.tar.gz /app/storage/
docker cp $(docker-compose -f docker-compose.development.yml ps -q dashboard-service):/tmp/storage_backup.tar.gz ./storage_backup.tar.gz
```

## Production Considerations

### Volume Drivers

For production, consider using:

- **Local driver with bind mounts** for single-node deployments
- **NFS driver** for multi-node deployments
- **Cloud storage drivers** (AWS EBS, Azure Disk, GCP Persistent Disk)

Example production volume configuration:

```yaml
volumes:
  postgres_data:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /opt/tracseq/data/postgres
  
  app_storage:
    driver: nfs
    driver_opts:
      share: nfs-server.example.com:/opt/tracseq/storage
```

### Backup Strategy

1. **Automated daily backups** of database
2. **Weekly full volume backups**
3. **Real-time replication** for critical data
4. **Offsite backup storage**
5. **Regular restore testing**

### Security

- **Encrypt volumes** at rest
- **Restrict volume access** to necessary services
- **Monitor volume usage** and access patterns
- **Implement backup encryption**

## Troubleshooting

### Common Issues

#### Volume Mount Errors

```bash
# Error: volume is in use
docker-compose -f docker-compose.development.yml down
docker volume rm tracseq20_postgres_data

# Error: permission denied
docker-compose -f docker-compose.development.yml exec postgres chown -R postgres:postgres /var/lib/postgresql/data
```

#### Data Corruption

```bash
# Check volume integrity
docker volume inspect tracseq20_postgres_data

# Repair PostgreSQL database
docker-compose -f docker-compose.development.yml exec postgres pg_resetwal /var/lib/postgresql/data
```

#### Performance Issues

```bash
# Monitor volume usage
docker system df -v

# Check I/O performance
docker-compose -f docker-compose.development.yml exec postgres iostat -x 1
```

### Volume Cleanup

```bash
# Remove unused volumes
docker volume prune

# Remove specific volume (⚠️ Data will be lost!)
docker volume rm tracseq20_postgres_data

# Complete system cleanup (⚠️ All data will be lost!)
docker system prune -a --volumes
```

## Monitoring

### Volume Usage

```bash
# Check volume sizes
docker system df -v

# Monitor volume growth
watch -n 60 'docker system df -v'

# Check specific volume details
docker volume inspect tracseq20_postgres_data
```

### Health Checks

```bash
# Database health
docker-compose -f docker-compose.development.yml exec postgres pg_isready -U postgres

# Redis health
docker-compose -f docker-compose.development.yml exec redis redis-cli ping

# Application storage health
docker-compose -f docker-compose.development.yml exec dashboard-service df -h /app/storage/
```

## Best Practices

### Development

1. **Use named volumes** for all persistent data
2. **Mount source code** as bind mounts for hot reload
3. **Separate volumes** for different data types
4. **Regular backup testing**
5. **Document volume purposes**

### Production

1. **Use production-grade volume drivers**
2. **Implement automated backups**
3. **Monitor volume performance**
4. **Set up alerting** for volume issues
5. **Regular disaster recovery testing**

### Security

1. **Encrypt sensitive volumes**
2. **Implement access controls**
3. **Audit volume access**
4. **Secure backup storage**
5. **Regular security assessments**

## Migration Guide

### From Bind Mounts to Volumes

```bash
# 1. Stop services
docker-compose -f docker-compose.development.yml down

# 2. Create volumes
docker volume create tracseq20_postgres_data

# 3. Copy data from bind mount
docker run --rm -v /host/path:/source -v tracseq20_postgres_data:/target ubuntu cp -r /source/* /target/

# 4. Update docker-compose.yml
# Replace bind mounts with volume mounts

# 5. Start services
docker-compose -f docker-compose.development.yml up -d
```

### Volume Upgrades

```bash
# 1. Backup current volumes
./scripts/backup-volumes.sh

# 2. Stop services
docker-compose -f docker-compose.development.yml down

# 3. Create new volumes with updated configuration
docker-compose -f docker-compose.development.yml up -d

# 4. Verify data integrity
./scripts/test-persistence.sh
```

## Support

For volume-related issues:

1. Check the [troubleshooting section](#troubleshooting)
2. Review Docker logs: `docker-compose logs`
3. Inspect volume configuration: `docker volume inspect <volume_name>`
4. Test persistence: `./scripts/test-persistence.sh`

## References

- [Docker Volume Documentation](https://docs.docker.com/storage/volumes/)
- [Docker Compose Volume Configuration](https://docs.docker.com/compose/compose-file/#volumes)
- [PostgreSQL Docker Documentation](https://hub.docker.com/_/postgres)
- [Redis Docker Documentation](https://hub.docker.com/_/redis) 