# TracSeq 2.0 Microservices Migration Script
# This script helps migrate from monolithic to microservices architecture

param(
    [Parameter(Mandatory=$true)]
    [ValidateSet("prepare", "phase1", "phase2", "phase3", "rollback", "status")]
    [string]$Action,
    
    [Parameter(Mandatory=$false)]
    [switch]$Force,
    
    [Parameter(Mandatory=$false)]
    [switch]$Backup = $true
)

$ErrorActionPreference = "Stop"

# Configuration
$MIGRATION_DIR = "deploy/migration"
$DOCKER_COMPOSE_FILE = "$MIGRATION_DIR/docker-compose.migration.yml"
$ENV_FILE = "$MIGRATION_DIR/migration.env"
$BACKUP_DIR = "backups/migration"

# Logging
function Write-MigrationLog {
    param($Message, $Level = "INFO")
    $Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Host "[$Timestamp] [$Level] $Message" -ForegroundColor $(
        switch ($Level) {
            "ERROR" { "Red" }
            "WARN"  { "Yellow" }
            "SUCCESS" { "Green" }
            default { "White" }
        }
    )
}

function Test-Prerequisites {
    Write-MigrationLog "Checking prerequisites..."
    
    # Check Docker
    if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
        throw "Docker is not installed or not in PATH"
    }
    
    # Check Docker Compose
    if (-not (Get-Command docker-compose -ErrorAction SilentlyContinue)) {
        throw "Docker Compose is not installed or not in PATH"
    }
    
    # Check if migration files exist
    if (-not (Test-Path $DOCKER_COMPOSE_FILE)) {
        throw "Migration Docker Compose file not found: $DOCKER_COMPOSE_FILE"
    }
    
    if (-not (Test-Path $ENV_FILE)) {
        throw "Migration environment file not found: $ENV_FILE"
    }
    
    Write-MigrationLog "Prerequisites check passed" "SUCCESS"
}

function Backup-CurrentState {
    if (-not $Backup) {
        Write-MigrationLog "Backup skipped (--no-backup flag)" "WARN"
        return
    }
    
    Write-MigrationLog "Creating backup of current state..."
    
    $BackupTimestamp = Get-Date -Format "yyyyMMdd_HHmmss"
    $BackupPath = "$BACKUP_DIR/$BackupTimestamp"
    
    New-Item -ItemType Directory -Path $BackupPath -Force | Out-Null
    
    # Backup databases
    Write-MigrationLog "Backing up databases..."
    
    # Export current monolith database
    $MonolithBackup = "$BackupPath/monolith_db.sql"
    docker exec lab_manager_db_1 pg_dump -U postgres -d lab_manager > $MonolithBackup
    
    # Export volumes
    Write-MigrationLog "Backing up volumes..."
    docker run --rm -v lab_manager_postgres_data:/data -v "${PWD}/${BackupPath}:/backup" busybox tar czf /backup/postgres_data.tar.gz -C /data .
    docker run --rm -v lab_manager_app_storage:/data -v "${PWD}/${BackupPath}:/backup" busybox tar czf /backup/app_storage.tar.gz -C /data .
    
    Write-MigrationLog "Backup completed: $BackupPath" "SUCCESS"
}

function Start-MigrationPhase1 {
    Write-MigrationLog "Starting Migration Phase 1: Hybrid Architecture"
    
    # Update environment for Phase 1
    (Get-Content $ENV_FILE) -replace 'MIGRATION_PHASE=.*', 'MIGRATION_PHASE=1' | Set-Content $ENV_FILE
    
    # Start services with Phase 1 configuration
    Write-MigrationLog "Starting hybrid services..."
    
    Push-Location $MIGRATION_DIR
    try {
        # Start core infrastructure
        docker-compose --env-file migration.env up -d `
            legacy-postgres `
            auth-postgres `
            storage-postgres `
            notification-postgres `
            rag-postgres `
            event-postgres `
            gateway-redis `
            event-redis `
            ollama
        
        Start-Sleep 10
        
        # Start microservices
        docker-compose --env-file migration.env up -d `
            auth-service `
            enhanced-storage-service `
            notification-service `
            enhanced-rag-service `
            event-service
        
        Start-Sleep 5
        
        # Start legacy monolith
        docker-compose --env-file migration.env up -d lab-manager
        
        Start-Sleep 5
        
        # Start API Gateway
        docker-compose --env-file migration.env up -d api-gateway
        
        Start-Sleep 5
        
        # Start Frontend
        docker-compose --env-file migration.env up -d frontend
        
        Write-MigrationLog "Phase 1 deployment completed" "SUCCESS"
    }
    finally {
        Pop-Location
    }
}

function Start-MigrationPhase2 {
    Write-MigrationLog "Starting Migration Phase 2: Partial Migration"
    
    # Update environment for Phase 2
    (Get-Content $ENV_FILE) -replace 'MIGRATION_PHASE=.*', 'MIGRATION_PHASE=2' | Set-Content $ENV_FILE
    
    Push-Location $MIGRATION_DIR
    try {
        # Add more microservices
        docker-compose --env-file migration.env up -d `
            template-postgres `
            sequencing-postgres `
            template-service `
            sequencing-service
        
        # Update API Gateway routing
        docker-compose --env-file migration.env restart api-gateway
        
        Write-MigrationLog "Phase 2 deployment completed" "SUCCESS"
    }
    finally {
        Pop-Location
    }
}

function Start-MigrationPhase3 {
    Write-MigrationLog "Starting Migration Phase 3: Full Microservices"
    
    # Update environment for Phase 3
    (Get-Content $ENV_FILE) -replace 'MIGRATION_PHASE=.*', 'MIGRATION_PHASE=3' | Set-Content $ENV_FILE
    
    Push-Location $MIGRATION_DIR
    try {
        # Add remaining microservices
        docker-compose --env-file migration.env up -d `
            sample-postgres `
            transaction-postgres `
            sample-service `
            transaction-service
        
        # Update API Gateway to route everything to microservices
        docker-compose --env-file migration.env restart api-gateway
        
        # Gracefully shut down legacy monolith
        Write-MigrationLog "Shutting down legacy monolith..."
        docker-compose --env-file migration.env stop lab-manager
        
        Write-MigrationLog "Phase 3 deployment completed - Full microservices!" "SUCCESS"
    }
    finally {
        Pop-Location
    }
}

function Get-MigrationStatus {
    Write-MigrationLog "Checking migration status..."
    
    Push-Location $MIGRATION_DIR
    try {
        $Services = docker-compose --env-file migration.env ps --services
        $RunningServices = docker-compose --env-file migration.env ps --filter "status=running" --services
        
        Write-Host "`n=== Migration Status ===" -ForegroundColor Cyan
        Write-Host "Total Services: $($Services.Count)"
        Write-Host "Running Services: $($RunningServices.Count)"
        
        Write-Host "`n=== Service Health ===" -ForegroundColor Cyan
        foreach ($service in $Services) {
            $status = docker-compose --env-file migration.env ps $service --format "table {{.State}}" | Select-Object -Last 1
            $color = if ($status -eq "running") { "Green" } else { "Red" }
            Write-Host "$service : $status" -ForegroundColor $color
        }
        
        # Check API Gateway health
        Write-Host "`n=== API Gateway Health ===" -ForegroundColor Cyan
        try {
            $response = Invoke-WebRequest -Uri "http://localhost:8000/health" -TimeoutSec 5
            Write-Host "API Gateway: Healthy" -ForegroundColor Green
        }
        catch {
            Write-Host "API Gateway: Unhealthy" -ForegroundColor Red
        }
        
        # Check Frontend
        Write-Host "`n=== Frontend Health ===" -ForegroundColor Cyan
        try {
            $response = Invoke-WebRequest -Uri "http://localhost:8080" -TimeoutSec 5
            Write-Host "Frontend: Accessible" -ForegroundColor Green
        }
        catch {
            Write-Host "Frontend: Inaccessible" -ForegroundColor Red
        }
    }
    finally {
        Pop-Location
    }
}

function Rollback-Migration {
    Write-MigrationLog "Rolling back migration..." "WARN"
    
    if (-not $Force) {
        $confirmation = Read-Host "This will stop all microservices and restore monolith. Continue? (y/N)"
        if ($confirmation -ne 'y' -and $confirmation -ne 'Y') {
            Write-MigrationLog "Rollback cancelled" "INFO"
            return
        }
    }
    
    Push-Location $MIGRATION_DIR
    try {
        # Stop all migration services
        docker-compose --env-file migration.env down
        
        # Restart original monolith
        Push-Location "../.."
        docker-compose up -d
        Pop-Location
        
        Write-MigrationLog "Rollback completed" "SUCCESS"
    }
    finally {
        Pop-Location
    }
}

function Prepare-Migration {
    Write-MigrationLog "Preparing migration environment..."
    
    # Create migration directories
    New-Item -ItemType Directory -Path $MIGRATION_DIR -Force | Out-Null
    New-Item -ItemType Directory -Path $BACKUP_DIR -Force | Out-Null
    
    # Stop current monolith
    Write-MigrationLog "Stopping current monolith services..."
    docker-compose down
    
    Write-MigrationLog "Migration preparation completed" "SUCCESS"
}

# Main execution
try {
    Write-MigrationLog "TracSeq 2.0 Microservices Migration Tool" "SUCCESS"
    Write-MigrationLog "Action: $Action"
    
    Test-Prerequisites
    
    switch ($Action) {
        "prepare" {
            Prepare-Migration
        }
        "phase1" {
            if ($Backup) { Backup-CurrentState }
            Start-MigrationPhase1
        }
        "phase2" {
            Start-MigrationPhase2
        }
        "phase3" {
            Start-MigrationPhase3
        }
        "rollback" {
            Rollback-Migration
        }
        "status" {
            Get-MigrationStatus
        }
    }
    
    if ($Action -ne "status" -and $Action -ne "rollback") {
        Write-MigrationLog "Waiting for services to stabilize..."
        Start-Sleep 10
        Get-MigrationStatus
    }
    
    Write-MigrationLog "Migration action '$Action' completed successfully!" "SUCCESS"
    
    # Display next steps
    switch ($Action) {
        "phase1" {
            Write-Host "`n=== Next Steps ===" -ForegroundColor Cyan
            Write-Host "1. Test the hybrid setup at http://localhost:8080"
            Write-Host "2. Verify API Gateway at http://localhost:8000/health"
            Write-Host "3. Run 'migrate-to-microservices.ps1 phase2' when ready"
        }
        "phase2" {
            Write-Host "`n=== Next Steps ===" -ForegroundColor Cyan
            Write-Host "1. Test additional services (Templates, Sequencing)"
            Write-Host "2. Run 'migrate-to-microservices.ps1 phase3' for full migration"
        }
        "phase3" {
            Write-Host "`n=== Congratulations! ===" -ForegroundColor Green
            Write-Host "Full microservices migration completed!"
            Write-Host "Your application is now running on distributed architecture."
            Write-Host "Frontend: http://localhost:8080"
            Write-Host "API Gateway: http://localhost:8000"
        }
    }
}
catch {
    Write-MigrationLog "Migration failed: $($_.Exception.Message)" "ERROR"
    exit 1
} 
