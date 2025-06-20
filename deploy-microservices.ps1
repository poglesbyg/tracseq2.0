# ========================================
# TracSeq 2.0 Complete Microservices Deployment
# ========================================

param(
    [Parameter(Position=0)]
    [string]$Action = "deploy",
    [switch]$QuickStart,
    [switch]$SkipBuild
)

# Colors for output
$Red = "Red"
$Green = "Green"
$Yellow = "Yellow"
$Blue = "Blue"
$Cyan = "Cyan"

function Write-Phase {
    param([string]$Message)
    Write-Host "🚀 $Message" -ForegroundColor $Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "✅ $Message" -ForegroundColor $Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "⚠️  $Message" -ForegroundColor $Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "❌ $Message" -ForegroundColor $Red
}

function Test-ServiceHealth {
    param([string]$Url, [string]$ServiceName)
    
    try {
        $response = Invoke-WebRequest -Uri $Url -Method GET -TimeoutSec 10 -ErrorAction Stop
        if ($response.StatusCode -eq 200) {
            Write-Success "$ServiceName is healthy"
            return $true
        }
    }
    catch {
        Write-Warning "$ServiceName not responding yet"
        return $false
    }
    return $false
}

function Wait-ForServices {
    param([array]$Services, [int]$TimeoutMinutes = 5)
    
    Write-Host "⏳ Waiting for services to become healthy..." -ForegroundColor $Yellow
    $timeout = (Get-Date).AddMinutes($TimeoutMinutes)
    
    do {
        $healthyCount = 0
        foreach ($service in $Services) {
            if (Test-ServiceHealth -Url $service.Url -ServiceName $service.Name) {
                $healthyCount++
            }
        }
        
        Write-Host "Health Status: $healthyCount/$($Services.Count) services healthy" -ForegroundColor $Cyan
        
        if ($healthyCount -eq $Services.Count) {
            Write-Success "All services are healthy!"
            return $true
        }
        
        Start-Sleep -Seconds 15
    } while ((Get-Date) -lt $timeout)
    
    Write-Warning "Timeout reached. Some services may still be starting."
    return $false
}

# Main deployment function
function Deploy-Microservices {
    Write-Phase "Starting TracSeq 2.0 Complete Microservices Deployment"
    
    # Check Docker
    try {
        docker --version | Out-Null
        docker info | Out-Null
        Write-Success "Docker is running"
    }
    catch {
        Write-Error "Docker is not running. Please start Docker Desktop first."
        exit 1
    }
    
    # Check existing infrastructure
    Write-Host "Checking existing infrastructure..." -ForegroundColor $Yellow
    $existingContainers = docker ps --format "{{.Names}}"
    if ($existingContainers -match "enhanced_storage_service-postgres-1") {
        Write-Success "Using existing PostgreSQL and Redis infrastructure"
        $useExistingInfra = $true
    } else {
        Write-Phase "Phase 1: Deploying Infrastructure Services"
        docker-compose -f docker-compose.microservices.yml up -d postgres-main redis-main
        Start-Sleep -Seconds 20
        Write-Success "Infrastructure services started"
        $useExistingInfra = $false
    }
    
    # Phase 2: Core Services (Auth first, then others)
    Write-Phase "Phase 2: Deploying Core Services"
    docker-compose -f docker-compose.microservices.yml up -d auth-service
    Start-Sleep -Seconds 30
    
    docker-compose -f docker-compose.microservices.yml up -d `
        sample-service `
        template-service `
        notification-service `
        event-service
    
    $coreServices = @(
        @{Name="Auth Service"; Url="http://localhost:8080/health"},
        @{Name="Sample Service"; Url="http://localhost:8081/health"},
        @{Name="Template Service"; Url="http://localhost:8083/health"},
        @{Name="Notification Service"; Url="http://localhost:8085/health"},
        @{Name="Event Service"; Url="http://localhost:8087/health"}
    )
    
    Wait-ForServices -Services $coreServices -TimeoutMinutes 3
    
    # Phase 3: Business Services
    Write-Phase "Phase 3: Deploying Business Services"
    docker-compose -f docker-compose.microservices.yml up -d `
        enhanced-storage-service `
        sequencing-service `
        transaction-service
    
    $businessServices = @(
        @{Name="Enhanced Storage Service"; Url="http://localhost:8082/health"},
        @{Name="Sequencing Service"; Url="http://localhost:8084/health"},
        @{Name="Transaction Service"; Url="http://localhost:8088/health"}
    )
    
    Wait-ForServices -Services $businessServices -TimeoutMinutes 3
    
    # Phase 4: AI & Integration Services
    Write-Phase "Phase 4: Deploying AI and Integration Services"
    docker-compose -f docker-compose.microservices.yml up -d `
        enhanced-rag-service `
        api-gateway
    
    $aiServices = @(
        @{Name="Enhanced RAG Service"; Url="http://localhost:8086/health"},
        @{Name="API Gateway"; Url="http://localhost:8089/health"}
    )
    
    Wait-ForServices -Services $aiServices -TimeoutMinutes 3
    
    # Phase 5: Monitoring & Observability
    Write-Phase "Phase 5: Deploying Monitoring Stack"
    docker-compose -f docker-compose.microservices.yml up -d `
        prometheus `
        jaeger
    
    Start-Sleep -Seconds 20
    
    # Final Health Check
    Write-Phase "Running Comprehensive Health Check"
    Test-AllServices
    
    # Show Service URLs
    Show-ServiceUrls
}

function Test-AllServices {
    Write-Host "🏥 Comprehensive Health Check" -ForegroundColor $Blue
    
    $allServices = @(
        @{Name="Auth Service"; Url="http://localhost:8080/health"; Port="8080"},
        @{Name="Sample Service"; Url="http://localhost:8081/health"; Port="8081"},
        @{Name="Enhanced Storage Service"; Url="http://localhost:8082/health"; Port="8082"},
        @{Name="Template Service"; Url="http://localhost:8083/health"; Port="8083"},
        @{Name="Sequencing Service"; Url="http://localhost:8084/health"; Port="8084"},
        @{Name="Notification Service"; Url="http://localhost:8085/health"; Port="8085"},
        @{Name="Enhanced RAG Service"; Url="http://localhost:8086/health"; Port="8086"},
        @{Name="Event Service"; Url="http://localhost:8087/health"; Port="8087"},
        @{Name="Transaction Service"; Url="http://localhost:8088/health"; Port="8088"},
        @{Name="API Gateway"; Url="http://localhost:8089/health"; Port="8089"}
    )
    
    $healthyCount = 0
    foreach ($service in $allServices) {
        if (Test-ServiceHealth -Url $service.Url -ServiceName $service.Name) {
            $healthyCount++
        }
    }
    
    Write-Host ""
    Write-Host "🎯 Final Results: $healthyCount/$($allServices.Count) services healthy" -ForegroundColor $Cyan
    
    if ($healthyCount -eq $allServices.Count) {
        Write-Success "🎉 ALL MICROSERVICES DEPLOYED SUCCESSFULLY!"
    } elseif ($healthyCount -gt ($allServices.Count * 0.8)) {
        Write-Warning "Most services are healthy. Some may still be starting up."
    } else {
        Write-Warning "Several services need attention. Check logs with: docker-compose -f docker-compose.microservices.yml logs"
    }
}

function Show-ServiceUrls {
    Write-Host ""
    Write-Host "🌐 TracSeq 2.0 Microservices Access Points" -ForegroundColor $Green
    Write-Host "=" * 60
    
    Write-Host "🔐 Authentication and Authorization:" -ForegroundColor $Yellow
    Write-Host "   Auth Service: http://localhost:8080" -ForegroundColor $Cyan
    
    Write-Host ""
    Write-Host "🧪 Laboratory Management:" -ForegroundColor $Yellow
    Write-Host "   Sample Service: http://localhost:8081" -ForegroundColor $Cyan
    Write-Host "   Enhanced Storage: http://localhost:8082" -ForegroundColor $Cyan
    Write-Host "   Template Service: http://localhost:8083" -ForegroundColor $Cyan
    Write-Host "   Sequencing Service: http://localhost:8084" -ForegroundColor $Cyan
    
    Write-Host ""
    Write-Host "🔔 Communication and Events:" -ForegroundColor $Yellow
    Write-Host "   Notification Service: http://localhost:8085" -ForegroundColor $Cyan
    Write-Host "   Event Service: http://localhost:8087" -ForegroundColor $Cyan
    
    Write-Host ""
    Write-Host "🤖 AI and Integration:" -ForegroundColor $Yellow
    Write-Host "   Enhanced RAG Service: http://localhost:8086" -ForegroundColor $Cyan
    Write-Host "   API Gateway (Unified): http://localhost:8089" -ForegroundColor $Cyan
    
    Write-Host ""
    Write-Host "🔄 Transaction Management:" -ForegroundColor $Yellow
    Write-Host "   Transaction Service: http://localhost:8088" -ForegroundColor $Cyan
    
    Write-Host ""
    Write-Host "📊 Monitoring and Observability:" -ForegroundColor $Yellow
    Write-Host "   Prometheus: http://localhost:9090" -ForegroundColor $Cyan
    Write-Host "   Jaeger Tracing: http://localhost:16686" -ForegroundColor $Cyan
    Write-Host "   Grafana (existing): http://localhost:3000" -ForegroundColor $Cyan
    
    Write-Host ""
    Write-Host "🎯 Integration with Existing TracSeq:" -ForegroundColor $Green
    Write-Host "   Frontend: http://localhost:5173" -ForegroundColor $Cyan
    Write-Host "   Backend: http://localhost:3003" -ForegroundColor $Cyan
    Write-Host "   RAG Service: http://localhost:8000" -ForegroundColor $Cyan
    
    Write-Host ""
    Write-Host "📋 Next Steps:" -ForegroundColor $Green
    Write-Host "  1. Test API Gateway: curl http://localhost:8089/health" -ForegroundColor White
    Write-Host "  2. Explore Enhanced Storage: http://localhost:8082/storage/overview" -ForegroundColor White
    Write-Host "  3. Monitor services: http://localhost:9090" -ForegroundColor White
    Write-Host "  4. View distributed traces: http://localhost:16686" -ForegroundColor White
}

function Stop-Microservices {
    Write-Phase "Stopping TracSeq 2.0 Microservices"
    docker-compose -f docker-compose.microservices.yml down
    Write-Success "All microservices stopped"
}

function Get-ServiceLogs {
    Write-Phase "Showing Recent Service Logs"
    docker-compose -f docker-compose.microservices.yml logs --tail=50
}

# Main execution
switch ($Action.ToLower()) {
    "deploy" {
        Deploy-Microservices
    }
    "stop" {
        Stop-Microservices
    }
    "test" {
        Test-AllServices
    }
    "urls" {
        Show-ServiceUrls
    }
    "logs" {
        Get-ServiceLogs
    }
    "restart" {
        Stop-Microservices
        Start-Sleep -Seconds 10
        Deploy-Microservices
    }
    default {
        Write-Host "TracSeq 2.0 Microservices Deployment Script" -ForegroundColor $Green
        Write-Host ""
        Write-Host "Usage: .\deploy-microservices.ps1 [action]" -ForegroundColor $Yellow
        Write-Host ""
        Write-Host "Actions:" -ForegroundColor $Cyan
        Write-Host "  deploy    - Deploy all microservices (default)" -ForegroundColor White
        Write-Host "  stop      - Stop all microservices" -ForegroundColor White
        Write-Host "  test      - Run health checks" -ForegroundColor White
        Write-Host "  urls      - Show service access points" -ForegroundColor White
        Write-Host "  logs      - Show service logs" -ForegroundColor White
        Write-Host "  restart   - Stop and redeploy services" -ForegroundColor White
        Write-Host ""
        Write-Host "Examples:" -ForegroundColor $Yellow
        Write-Host "  .\deploy-microservices.ps1 deploy" -ForegroundColor White
        Write-Host "  .\deploy-microservices.ps1 test" -ForegroundColor White
        Write-Host "  .\deploy-microservices.ps1 urls" -ForegroundColor White
    }
} 
