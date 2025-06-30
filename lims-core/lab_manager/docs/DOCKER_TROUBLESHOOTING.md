# Docker Networking Troubleshooting

## Issue: Frontend 500 Errors with Docker

### Problem
When running the frontend and backend in Docker containers, the frontend gets 500 errors when trying to access API endpoints, even though the backend is working correctly.

### Root Cause
The frontend container was trying to connect to `localhost:3000`, but in Docker containers, `localhost` refers to the container itself, not the host machine or other containers.

### Solution
Update the frontend proxy configuration to use the Docker service name instead of `localhost`.

**Before (Broken):**
```typescript
// frontend/vite.config.ts
proxy: {
  '/api': {
    target: 'http://localhost:3000',  // ❌ Won't work in Docker
    changeOrigin: true,
  },
}
```

**After (Fixed):**
```typescript
// frontend/vite.config.ts
proxy: {
  '/api': {
    target: 'http://dev:3000',  // ✅ Uses Docker service name
    changeOrigin: true,
  },
}
```

### Docker Service Names
Based on `docker-compose.yml`:
- **Backend (dev)**: `http://dev:3000`
- **Database**: `http://db:5432`
- **Frontend**: `http://frontend-dev:5173`

### How to Debug
1. **Check if containers are running:**
   ```bash
   docker ps
   ```

2. **Test backend directly:**
   ```bash
   curl http://localhost:3000/health
   ```

3. **Test frontend proxy:**
   ```bash
   curl http://localhost:5173/api/dashboard/stats
   ```

4. **Check Docker logs:**
   ```bash
   docker compose logs frontend-dev
   docker compose logs dev
   ```

### Prevention
- Always use Docker service names for inter-container communication
- Test both direct backend access and proxied frontend access
- Use the included test script: `./scripts/test_dashboard.sh`

### Restart After Changes
When changing Vite configuration:
```bash
docker compose restart frontend-dev
``` 
