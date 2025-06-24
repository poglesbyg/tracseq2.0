# TracSeq 2.0 Individual Service Builder (PowerShell)
# Usage: .\build-individual-service.ps1 -ServiceName <service-name> [options]

param(
    [string]$ServiceName,
    [string]$ImageTag = "latest",
    [string]$Dockerfile = "Dockerfile",
    [switch]$NoCache,
    [switch]$Push,
    [string[]]$BuildArgs = @(),
    [switch]$Verbose,
    [switch]$Help
)

# Colors for output
$Red = "Red"
$Green = "Green"
$Yellow = "Yellow"
$Blue = "Blue"

# Print functions
function Write-Info {
    param([string]$Message)
    Write-Host "ℹ️  $Message" -ForegroundColor $Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "✅ $Message" -ForegroundColor $Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "⚠️  $Message" -ForegroundColor $Yellow
}

function Write-Error-Custom {
    param([string]$Message)
    Write-Host "❌ $Message" -ForegroundColor $Red
}

function Write-Header {
    param([string]$Message)
    Write-Host ""
    Write-Host "================================================" -ForegroundColor $Blue
    Write-Host " $Message" -ForegroundColor $Blue
    Write-Host "================================================" -ForegroundColor $Blue
    Write-Host ""
}

# Available services
$RustServices = @(
    "auth_service",
    "sample_service",
    "sequencing_service",
    "notification_service",
    "enhanced_storage_service",
    "template_service",
    "transaction_service",
    "event_service",
    "lab_manager",
    "library_details_service",
    "qaqc_service",
    "spreadsheet_versioning_service",
    "config-service"
)

$PythonServices = @(
    "enhanced_rag_service",
    "api_gateway",
    "lab_submission_rag"
)

$AllServices = $RustServices + $PythonServices

# Help function
function Show-Help {
    Write-Host "TracSeq 2.0 Individual Service Builder (PowerShell)"
    Write-Host ""
    Write-Host "Usage: .\build-individual-service.ps1 -ServiceName <service-name> [options]"
    Write-Host ""
    Write-Host "Available services:"
    Write-Host "  Rust services:"
    foreach ($service in $RustServices) {
        Write-Host "    - $service"
    }
    Write-Host ""
    Write-Host "  Python services:"
    foreach ($service in $PythonServices) {
        Write-Host "    - $service"
    }
    Write-Host ""
    Write-Host "Parameters:"
    Write-Host "  -ServiceName       Name of the service to build"
    Write-Host "  -ImageTag          Image tag (default: latest)"
    Write-Host "  -Dockerfile        Dockerfile name (default: Dockerfile)"
    Write-Host "  -NoCache           Build without cache"
    Write-Host "  -Push              Push image to registry after build"
    Write-Host "  -BuildArgs         Array of build arguments"
    Write-Host "  -Verbose           Verbose output"
    Write-Host "  -Help              Show this help message"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\build-individual-service.ps1 -ServiceName auth_service"
    Write-Host "  .\build-individual-service.ps1 -ServiceName auth_service -ImageTag v1.0.0"
    Write-Host "  .\build-individual-service.ps1 -ServiceName enhanced_storage_service -NoCache"
    Write-Host "  .\build-individual-service.ps1 -ServiceName api_gateway -BuildArgs @('ENV=production')"
}

# Validate service name
function Test-ServiceName {
    param([string]$Name)
    
    if (-not $Name) {
        Write-Error-Custom "Service name is required!"
        Show-Help
        exit 1
    }
    
    if ($Name -notin $AllServices) {
        Write-Error-Custom "Service '$Name' not found!"
        Write-Host ""
        Write-Host "Available services:"
        $AllServices | ForEach-Object { Write-Host "  - $_" }
        exit 1
    }
    
    Write-Success "Service '$Name' is valid"
}

# Check if service directory exists
function Test-ServiceDirectory {
    param([string]$ServiceName, [string]$DockerfileName)
    
    if (-not (Test-Path $ServiceName)) {
        Write-Error-Custom "Service directory '$ServiceName' does not exist!"
        exit 1
    }
    
    $dockerfilePath = Join-Path $ServiceName $DockerfileName
    if (-not (Test-Path $dockerfilePath)) {
        Write-Error-Custom "Dockerfile not found at '$dockerfilePath'!"
        exit 1
    }
    
    Write-Success "Service directory and Dockerfile found"
}

# Build the service
function Build-Service {
    param(
        [string]$ServiceName,
        [string]$Tag,
        [string]$DockerfileName,
        [bool]$NoCacheFlag,
        [bool]$VerboseFlag,
        [string[]]$BuildArguments
    )
    
    $imageName = "tracseq-$ServiceName`:$Tag"
    $buildArgs = @("docker", "build")
    
    # Add build options
    if ($NoCacheFlag) {
        $buildArgs += "--no-cache"
    }
    
    if ($VerboseFlag) {
        $buildArgs += "--progress=plain"
    }
    
    # Add custom build args
    foreach ($arg in $BuildArguments) {
        $buildArgs += "--build-arg"
        $buildArgs += $arg
    }
    
    # Add dockerfile and tag
    $buildArgs += "-f"
    $buildArgs += "$ServiceName\$DockerfileName"
    $buildArgs += "-t"
    $buildArgs += $imageName
    $buildArgs += $ServiceName
    
    Write-Info "Building image: $imageName"
    Write-Info "Build command: $($buildArgs -join ' ')"
    
    try {
        & $buildArgs[0] $buildArgs[1..($buildArgs.Length-1)]
        Write-Success "Successfully built $imageName"
        return $imageName
    }
    catch {
        Write-Error-Custom "Failed to build $imageName"
        Write-Error-Custom $_.Exception.Message
        exit 1
    }
}

# Push image to registry
function Push-Image {
    param([string]$ImageName, [bool]$ShouldPush)
    
    if ($ShouldPush) {
        Write-Info "Pushing image: $ImageName"
        
        try {
            docker push $ImageName
            Write-Success "Successfully pushed $ImageName"
        }
        catch {
            Write-Error-Custom "Failed to push $ImageName"
            Write-Error-Custom $_.Exception.Message
            exit 1
        }
    }
}

# Test the built image
function Test-Image {
    param([string]$ImageName, [bool]$VerboseFlag)
    
    Write-Info "Testing built image..."
    
    try {
        docker image inspect $ImageName | Out-Null
        Write-Success "Image $ImageName exists and is ready"
        
        if ($VerboseFlag) {
            Write-Host ""
            Write-Host "Image details:"
            docker images $ImageName.Split(':')[0]
        }
    }
    catch {
        Write-Error-Custom "Image $ImageName was not created successfully"
        exit 1
    }
}

# Show build summary
function Show-BuildSummary {
    param(
        [string]$ServiceName,
        [string]$ImageName,
        [string]$Tag,
        [string]$DockerfilePath,
        [bool]$NoCacheFlag,
        [bool]$PushedFlag,
        [bool]$VerboseFlag
    )
    
    Write-Header "Build Summary"
    Write-Host "Service: $ServiceName"
    Write-Host "Image: $ImageName"
    Write-Host "Tag: $Tag"
    Write-Host "Dockerfile: $DockerfilePath"
    Write-Host "No Cache: $NoCacheFlag"
    Write-Host "Pushed: $PushedFlag"
    
    if ($VerboseFlag) {
        Write-Host ""
        Write-Host "Available images for this service:"
        $serviceImages = docker images | Select-String "tracseq-$ServiceName"
        if ($serviceImages) {
            $serviceImages
        } else {
            Write-Host "No other images found"
        }
    }
}

# Main execution
function Main {
    Write-Header "TracSeq 2.0 Individual Service Builder"
    
    # Show help if requested
    if ($Help) {
        Show-Help
        exit 0
    }
    
    # Validate inputs
    Test-ServiceName -Name $ServiceName
    Test-ServiceDirectory -ServiceName $ServiceName -DockerfileName $Dockerfile
    
    # Build the service
    $imageName = Build-Service -ServiceName $ServiceName -Tag $ImageTag -DockerfileName $Dockerfile -NoCacheFlag $NoCache -VerboseFlag $Verbose -BuildArguments $BuildArgs
    
    # Test the image
    Test-Image -ImageName $imageName -VerboseFlag $Verbose
    
    # Push if requested
    Push-Image -ImageName $imageName -ShouldPush $Push
    
    # Show summary
    $dockerfilePath = "$ServiceName\$Dockerfile"
    Show-BuildSummary -ServiceName $ServiceName -ImageName $imageName -Tag $ImageTag -DockerfilePath $dockerfilePath -NoCacheFlag $NoCache -PushedFlag $Push -VerboseFlag $Verbose
    
    Write-Success "Build completed successfully! 🎉"
}

# Execute main function
Main 
