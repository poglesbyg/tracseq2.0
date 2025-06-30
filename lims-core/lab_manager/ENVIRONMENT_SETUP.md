# Environment Configuration Guide

This project uses environment variables to configure different deployment scenarios. All hardcoded values have been moved to configurable environment variables.

## Environment Files

- **env.example**: Template with all configuration options
- **env.development**: Current development configuration  
- **env.production**: Production template with secure defaults

## Quick Setup

### Development (Current Setup)
```bash
cp env.development .env
docker-compose -f docker-compose.windows.yml up -d
```

### Production
```bash
cp env.production .env
# Edit .env and change passwords/secrets!
docker-compose -f deploy/docker-compose.production.yml up -d
```

## Security Notes

### Critical for Production:
- Change `JWT_SECRET` (min 32 chars)
- Change `POSTGRES_PASSWORD` 
- Use dedicated database user
- Set `RUST_LOG=warn`

## Environment Variables

- `POSTGRES_USER/PASSWORD/DB`: Database config
- `JWT_SECRET`: JWT signing key ⚠️
- `STORAGE_PATH`: File storage location
- `RAG_SERVICE_URL`: External service URL
- Port mappings: `DB_EXTERNAL_PORT`, `BACKEND_DEV_PORT`, etc.

## Testing
```bash
curl http://localhost:3000/health
docker ps
``` 
