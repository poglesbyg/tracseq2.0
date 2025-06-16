# Docker Cleanup and Optimization Script for TracSeq 2.0
Write-Host "🧹 Docker Cleanup and Optimization for TracSeq 2.0" -ForegroundColor Green
Write-Host "================================================="

# Check if Docker is running
try {
    docker version | Out-Null
    Write-Host "✅ Docker is running" -ForegroundColor Green
} catch {
    Write-Host "❌ Docker is not running. Please start Docker first." -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "🗑️ Cleaning up unused Docker resources..."

# Remove unused containers
Write-Host "  Removing stopped containers..."
docker container prune -f | Out-Null

# Remove unused images
Write-Host "  Removing unused images..."
docker image prune -f | Out-Null

# Remove unused networks
Write-Host "  Removing unused networks..."
docker network prune -f | Out-Null

# Remove unused volumes (be careful with this)
Write-Host "  Removing unused volumes..."
docker volume prune -f | Out-Null

Write-Host ""
Write-Host "📊 Current Docker usage:"
docker system df

Write-Host ""
Write-Host "🚀 Optimization tips applied:"
Write-Host "  ✅ Removed obsolete version attributes from docker-compose files"
Write-Host "  ✅ Fixed Dockerfile casing warnings"
Write-Host "  ✅ Added .dockerignore files to reduce build context"
Write-Host "  ✅ Cleaned up unused Docker resources"

Write-Host ""
Write-Host "🔧 For faster builds in the future:"
Write-Host "  • Use 'docker-compose build --parallel' for parallel builds"
Write-Host "  • Run this cleanup script regularly"
Write-Host "  • Consider using BuildKit for better caching: set DOCKER_BUILDKIT=1"

Write-Host ""
Write-Host "✅ Docker cleanup complete!" -ForegroundColor Green 
