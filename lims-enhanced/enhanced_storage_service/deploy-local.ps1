# ========================================
# Enhanced Storage Service - Local Deployment Script (PowerShell)
# ========================================

param(
    [Parameter(Position=0)]
    [string]$Command = "deploy"
)

# Configuration
$ProjectName = "enhanced-storage-service"
$DockerComposeFile = "docker-compose.local.yml"
$EnvFile = "local.env"

# Helper functions
function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# Check prerequisites
function Test-Prerequisites {
    Write-Info "Checking prerequisites..."
    
    # Check Docker
    try {
        docker --version | Out-Null
    }
    catch {
        Write-Error "Docker is not installed. Please install Docker Desktop first."
        exit 1
    }
    
    # Check Docker Compose
    try {
        docker-compose --version | Out-Null
    }
    catch {
        try {
            docker compose version | Out-Null
        }
        catch {
            Write-Error "Docker Compose is not available. Please install Docker Desktop with Compose."
            exit 1
        }
    }
    
    # Check if Docker is running
    try {
        docker info | Out-Null
    }
    catch {
        Write-Error "Docker is not running. Please start Docker Desktop first."
        exit 1
    }
    
    Write-Success "Prerequisites check completed"
}

# Create necessary directories
function New-Directories {
    Write-Info "Creating necessary directories..."
    
    $directories = @(
        "config/mosquitto",
        "config/prometheus", 
        "config/grafana/provisioning/dashboards",
        "config/grafana/provisioning/datasources",
        "config/grafana/dashboards",
        "config/nginx",
        "mocks/lims/mappings",
        "mocks/erp/mappings",
        "scripts",
        "frontend/src",
        "frontend/public"
    )
    
    foreach ($dir in $directories) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
    }
    
    Write-Success "Directories created"
}

# Create configuration files
function New-ConfigFiles {
    Write-Info "Creating configuration files..."
    
    # Mosquitto MQTT configuration
    @"
# Mosquitto MQTT Broker Configuration
listener 1883
allow_anonymous true
persistence true
persistence_location /mosquitto/data/
log_dest file /mosquitto/log/mosquitto.log
log_type error
log_type warning
log_type notice
log_type information
"@ | Out-File -FilePath "config/mosquitto/mosquitto.conf" -Encoding UTF8

    # Prometheus configuration
    @"
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'enhanced-storage-service'
    static_configs:
      - targets: ['enhanced-storage-service:9100']
    scrape_interval: 5s
    metrics_path: /metrics

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres:5432']

  - job_name: 'redis'
    static_configs:
      - targets: ['redis:6379']
"@ | Out-File -FilePath "config/prometheus/prometheus.yml" -Encoding UTF8

    # Mock LIMS service mappings
    @"
{
  "request": {
    "method": "GET",
    "urlPath": "/api/samples"
  },
  "response": {
    "status": 200,
    "headers": {
      "Content-Type": "application/json"
    },
    "jsonBody": {
      "samples": [
        {
          "id": "LIMS-001",
          "barcode": "SAMPLE-20241201-001",
          "status": "processed",
          "type": "blood",
          "created_at": "2024-12-01T10:00:00Z"
        }
      ],
      "total": 1
    }
  }
}
"@ | Out-File -FilePath "mocks/lims/mappings/samples.json" -Encoding UTF8

    Write-Success "Configuration files created"
}

# Build the application
function Build-Application {
    Write-Info "Building Enhanced Storage Service..."
    
    try {
        docker-compose -f $DockerComposeFile build
    }
    catch {
        docker compose -f $DockerComposeFile build
    }
    
    Write-Success "Application built successfully"
}

# Start services
function Start-Services {
    Write-Info "Starting services..."
    
    try {
        docker-compose -f $DockerComposeFile up -d
    }
    catch {
        docker compose -f $DockerComposeFile up -d
    }
    
    Write-Success "Services started"
}

# Wait for services to be ready
function Wait-ForServices {
    Write-Info "Waiting for services to be ready..."
    
    # Wait for main service
    Write-Info "Waiting for Enhanced Storage Service..."
    $count = 0
    do {
        if ($count -gt 30) {
            Write-Error "Service health check timeout"
            return $false
        }
        Write-Host "." -NoNewline
        Start-Sleep -Seconds 5
        $count++
        try {
            $response = Invoke-WebRequest -Uri "http://localhost:8082/health" -Method GET -TimeoutSec 5
            if ($response.StatusCode -eq 200) {
                break
            }
        }
        catch {
            # Service not ready yet
        }
    } while ($true)
    
    Write-Host ""
    Write-Success "All services are ready"
    return $true
}

# Display service URLs
function Show-ServiceUrls {
    Write-Info "Deployment completed! Access your services:"
    Write-Host ""
    Write-Host "🚀 Enhanced Storage Service API: http://localhost:8082" -ForegroundColor Cyan
    Write-Host "📊 Grafana Dashboard: http://localhost:3000 (admin/admin)" -ForegroundColor Cyan
    Write-Host "📈 Prometheus Metrics: http://localhost:9090" -ForegroundColor Cyan
    Write-Host "🔍 Jaeger Tracing: http://localhost:16686" -ForegroundColor Cyan
    Write-Host "💾 MinIO Console: http://localhost:9001 (minioadmin/minioadmin)" -ForegroundColor Cyan
    Write-Host "🌐 Main Application: http://localhost:80" -ForegroundColor Cyan
    Write-Host "🔧 Mock LIMS: http://localhost:8090" -ForegroundColor Cyan
    Write-Host "💼 Mock ERP: http://localhost:8091" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "🔗 Key API Endpoints:" -ForegroundColor Yellow
    Write-Host "  - Health Check: http://localhost:8082/health" -ForegroundColor White
    Write-Host "  - Storage Overview: http://localhost:8082/storage/overview" -ForegroundColor White
    Write-Host "  - IoT Sensors: http://localhost:8082/iot/sensors" -ForegroundColor White  
    Write-Host "  - AI Platform: http://localhost:8082/ai/overview" -ForegroundColor White
    Write-Host "  - Integrations: http://localhost:8082/integrations/overview" -ForegroundColor White
    Write-Host ""
    Write-Host "📝 To check logs: docker logs enhanced-storage-service_enhanced-storage-service_1" -ForegroundColor Gray
    Write-Host "🛑 To stop services: .\deploy-local.ps1 stop" -ForegroundColor Gray
}

# Stop services
function Stop-Services {
    Write-Info "Stopping services..."
    
    try {
        docker-compose -f $DockerComposeFile down
    }
    catch {
        docker compose -f $DockerComposeFile down
    }
    
    Write-Success "Services stopped"
}

# Clean up everything
function Remove-Everything {
    Write-Info "Cleaning up..."
    
    try {
        docker-compose -f $DockerComposeFile down -v --remove-orphans
    }
    catch {
        docker compose -f $DockerComposeFile down -v --remove-orphans
    }
    
    docker system prune -f
    
    Write-Success "Cleanup completed"
}

# Test the deployment
function Test-Deployment {
    Write-Info "Running deployment tests..."
    
    # Test health endpoint
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:8082/health" -Method GET -TimeoutSec 10
        if ($response.StatusCode -eq 200) {
            Write-Success "✅ Health check passed"
        }
    }
    catch {
        Write-Error "❌ Health check failed"
        return $false
    }
    
    # Test API endpoints
    $endpoints = @(
        @{Url="http://localhost:8082/storage/overview"; Name="Storage API"},
        @{Url="http://localhost:8082/ai/overview"; Name="AI Platform API"},
        @{Url="http://localhost:8082/integrations/overview"; Name="Integrations API"}
    )
    
    foreach ($endpoint in $endpoints) {
        try {
            $response = Invoke-WebRequest -Uri $endpoint.Url -Method GET -TimeoutSec 5
            if ($response.StatusCode -eq 200) {
                Write-Success "✅ $($endpoint.Name) accessible"
            }
        }
        catch {
            Write-Warning "⚠️  $($endpoint.Name) not accessible yet"
        }
    }
    
    Write-Success "Deployment tests completed"
    return $true
}

# Main execution
switch ($Command.ToLower()) {
    "deploy" {
        Write-Info "🚀 Starting Enhanced Storage Service local deployment..."
        Test-Prerequisites
        New-Directories
        New-ConfigFiles
        Build-Application
        Start-Services
        if (Wait-ForServices) {
            Test-Deployment
            Show-ServiceUrls
        }
    }
    "start" {
        Start-Services
        if (Wait-ForServices) {
            Show-ServiceUrls
        }
    }
    "stop" {
        Stop-Services
    }
    "restart" {
        Stop-Services
        Start-Services
        if (Wait-ForServices) {
            Show-ServiceUrls
        }
    }
    "test" {
        Test-Deployment
    }
    "clean" {
        Remove-Everything
    }
    "logs" {
        try {
            docker-compose -f $DockerComposeFile logs -f enhanced-storage-service
        }
        catch {
            docker compose -f $DockerComposeFile logs -f enhanced-storage-service
        }
    }
    default {
        Write-Host "Usage: .\deploy-local.ps1 {deploy|start|stop|restart|test|clean|logs}"
        Write-Host ""
        Write-Host "Commands:"
        Write-Host "  deploy   - Full deployment (default)"
        Write-Host "  start    - Start services"
        Write-Host "  stop     - Stop services"
        Write-Host "  restart  - Restart services"
        Write-Host "  test     - Test deployment"
        Write-Host "  clean    - Clean up everything"
        Write-Host "  logs     - Show service logs"
        exit 1
    }
} 
