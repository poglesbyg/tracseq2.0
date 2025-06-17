# 🚀 TracSeq 2.0 Azure Deployment Script (PowerShell)
# Automated deployment to Azure Container Apps

param(
    [string]$ResourceGroup = "tracseq-rg",
    [string]$Location = "eastus",
    [string]$ContainerEnv = "tracseq-env",
    [string]$RegistryName = "tracseqregistry",
    [string]$DBServerName = "tracseq-db-server",
    [string]$DBName = "lab_manager",
    [string]$DBUser = "tracseqadmin",
    [string]$KeyVaultName = "tracseq-kv",
    [switch]$SkipPrerequisiteCheck
)

# Error handling
$ErrorActionPreference = "Stop"

# Function to write colored output
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

# Function to check prerequisites
function Test-Prerequisites {
    Write-Info "Checking prerequisites..."
    
    # Check if Azure CLI is installed
    try {
        $null = Get-Command az -ErrorAction Stop
    }
    catch {
        Write-Error "Azure CLI is not installed. Please install it first."
        Write-Host "Download from: https://aka.ms/installazurecliwindows" -ForegroundColor Cyan
        exit 1
    }
    
    # Check if Docker is installed
    try {
        $null = Get-Command docker -ErrorAction Stop
    }
    catch {
        Write-Error "Docker is not installed. Please install Docker Desktop first."
        Write-Host "Download from: https://desktop.docker.com/win/main/amd64/Docker%20Desktop%20Installer.exe" -ForegroundColor Cyan
        exit 1
    }
    
    # Check if user is logged in to Azure
    try {
        $account = az account show --output json 2>$null | ConvertFrom-Json
        if (-not $account) {
            throw "Not logged in"
        }
        Write-Info "Logged in as: $($account.user.name)"
    }
    catch {
        Write-Error "You are not logged in to Azure. Please run 'az login' first."
        exit 1
    }
    
    Write-Success "Prerequisites check passed!"
}

# Function to get user input
function Get-UserInput {
    Write-Info "Configuring deployment parameters..."
    
    $script:ResourceGroup = Read-Host "Enter resource group name (default: $ResourceGroup)" 
    if ([string]::IsNullOrWhiteSpace($script:ResourceGroup)) { $script:ResourceGroup = $ResourceGroup }
    
    $inputLocation = Read-Host "Enter Azure region (default: $Location)"
    if (-not [string]::IsNullOrWhiteSpace($inputLocation)) { $script:Location = $inputLocation }
    
    $inputRegistry = Read-Host "Enter container registry name (default: $RegistryName)"
    if (-not [string]::IsNullOrWhiteSpace($inputRegistry)) { $script:RegistryName = $inputRegistry }
    
    do {
        $script:DBPassword = Read-Host "Enter database admin password" -AsSecureString
        $plaintextPassword = [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR($script:DBPassword))
        if ([string]::IsNullOrWhiteSpace($plaintextPassword)) {
            Write-Error "Database password cannot be empty!"
        }
    } while ([string]::IsNullOrWhiteSpace($plaintextPassword))
    
    do {
        $script:JWTSecret = Read-Host "Enter JWT secret" -AsSecureString
        $plaintextJWT = [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR($script:JWTSecret))
        if ([string]::IsNullOrWhiteSpace($plaintextJWT)) {
            Write-Error "JWT secret cannot be empty!"
        }
    } while ([string]::IsNullOrWhiteSpace($plaintextJWT))
}

# Function to create Azure resources
function New-AzureResources {
    Write-Info "Creating Azure resources..."
    
    try {
        # Create resource group
        Write-Info "Creating resource group: $ResourceGroup"
        az group create --name $ResourceGroup --location $Location --output none
        
        # Create Container Apps environment
        Write-Info "Creating Container Apps environment: $ContainerEnv"
        az containerapp env create --name $ContainerEnv --resource-group $ResourceGroup --location $Location --output none
        
        # Create container registry
        Write-Info "Creating container registry: $RegistryName"
        az acr create --resource-group $ResourceGroup --name $RegistryName --sku Basic --admin-enabled true --output none
        
        # Convert secure strings to plain text for Azure CLI
        $dbPasswordPlain = [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR($DBPassword))
        
        # Create PostgreSQL database
        Write-Info "Creating PostgreSQL database: $DBServerName"
        az postgres flexible-server create `
            --resource-group $ResourceGroup `
            --name $DBServerName `
            --location $Location `
            --admin-user $DBUser `
            --admin-password $dbPasswordPlain `
            --sku-name Standard_B1ms `
            --tier Burstable `
            --version 15 `
            --output none
        
        # Create database
        Write-Info "Creating database: $DBName"
        az postgres flexible-server db create `
            --resource-group $ResourceGroup `
            --server-name $DBServerName `
            --database-name $DBName `
            --output none
        
        # Configure firewall
        Write-Info "Configuring database firewall rules"
        az postgres flexible-server firewall-rule create `
            --resource-group $ResourceGroup `
            --name $DBServerName `
            --rule-name AllowAzureServices `
            --start-ip-address 0.0.0.0 `
            --end-ip-address 0.0.0.0 `
            --output none
        
        # Create Key Vault
        Write-Info "Creating Key Vault: $KeyVaultName"
        az keyvault create --name $KeyVaultName --resource-group $ResourceGroup --location $Location --output none
        
        Write-Success "Azure resources created successfully!"
    }
    catch {
        Write-Error "Failed to create Azure resources: $($_.Exception.Message)"
        throw
    }
}

# Function to build and push Docker images
function Build-AndPushImages {
    Write-Info "Building and pushing Docker images..."
    
    try {
        # Login to container registry
        az acr login --name $RegistryName
        
        # Get registry login server
        $registryLoginServer = az acr show --name $RegistryName --resource-group $ResourceGroup --query loginServer --output tsv
        
        # Build and push frontend
        Write-Info "Building frontend image..."
        docker build -t "$registryLoginServer/tracseq-frontend:latest" ./lab_manager/frontend
        Write-Info "Pushing frontend image..."
        docker push "$registryLoginServer/tracseq-frontend:latest"
        
        # Build and push backend
        Write-Info "Building backend image..."
        docker build -t "$registryLoginServer/tracseq-backend:latest" ./lab_manager
        Write-Info "Pushing backend image..."
        docker push "$registryLoginServer/tracseq-backend:latest"
        
        # Build and push RAG service
        Write-Info "Building RAG service image..."
        docker build -t "$registryLoginServer/tracseq-rag:latest" ./lab_submission_rag
        Write-Info "Pushing RAG service image..."
        docker push "$registryLoginServer/tracseq-rag:latest"
        
        # Pull and push Ollama image
        Write-Info "Pulling and pushing Ollama image..."
        docker pull ollama/ollama:latest
        docker tag ollama/ollama:latest "$registryLoginServer/tracseq-ollama:latest"
        docker push "$registryLoginServer/tracseq-ollama:latest"
        
        Write-Success "Docker images built and pushed successfully!"
    }
    catch {
        Write-Error "Failed to build and push images: $($_.Exception.Message)"
        throw
    }
}

# Function to deploy container apps
function Deploy-ContainerApps {
    Write-Info "Deploying container apps..."
    
    try {
        # Get registry credentials
        $registryLoginServer = az acr show --name $RegistryName --resource-group $ResourceGroup --query loginServer --output tsv
        $registryUsername = az acr credential show --name $RegistryName --resource-group $ResourceGroup --query username --output tsv
        $registryPassword = az acr credential show --name $RegistryName --resource-group $ResourceGroup --query passwords[0].value --output tsv
        
        # Database connection string
        $dbPasswordPlain = [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR($DBPassword))
        $dbConnectionString = "postgresql://$DBUser`:$dbPasswordPlain@$DBServerName.postgres.database.azure.com:5432/$DBName`?sslmode=require"
        
        # Deploy Ollama service first
        Write-Info "Deploying Ollama service..."
        az containerapp create `
            --name tracseq-ollama `
            --resource-group $ResourceGroup `
            --environment $ContainerEnv `
            --image "$registryLoginServer/tracseq-ollama:latest" `
            --registry-server $registryLoginServer `
            --registry-username $registryUsername `
            --registry-password $registryPassword `
            --target-port 11434 `
            --ingress internal `
            --cpu 1.0 `
            --memory 4.0Gi `
            --min-replicas 1 `
            --max-replicas 2 `
            --startup-probe-initial-delay 30 `
            --startup-probe-period 10 `
            --startup-probe-timeout 5 `
            --startup-probe-failure-threshold 10 `
            --output none
        
        # Get Ollama service URL
        $ollamaServiceFqdn = az containerapp show --name tracseq-ollama --resource-group $ResourceGroup --query properties.configuration.ingress.fqdn --output tsv
        $ollamaServiceUrl = "https://$ollamaServiceFqdn"
        
        # Deploy RAG service
        Write-Info "Deploying RAG service..."
        az containerapp create `
            --name tracseq-rag `
            --resource-group $ResourceGroup `
            --environment $ContainerEnv `
            --image "$registryLoginServer/tracseq-rag:latest" `
            --registry-server $registryLoginServer `
            --registry-username $registryUsername `
            --registry-password $registryPassword `
            --target-port 8000 `
            --ingress external `
            --cpu 1.0 `
            --memory 2.0Gi `
            --min-replicas 1 `
            --max-replicas 3 `
            --env-vars "POSTGRES_URL=$dbConnectionString" "RAG_LOG_LEVEL=INFO" "USE_OLLAMA=true" "LLM_PROVIDER=ollama" "OLLAMA_BASE_URL=$ollamaServiceUrl" "OLLAMA_MODEL=llama3.2:3b" `
            --output none
        
        # Get RAG service URL
        $ragServiceFqdn = az containerapp show --name tracseq-rag --resource-group $ResourceGroup --query properties.configuration.ingress.fqdn --output tsv
        $ragServiceUrl = "https://$ragServiceFqdn"
        
        # Deploy backend service
        Write-Info "Deploying backend service..."
        $jwtSecretPlain = [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR($JWTSecret))
        
        az containerapp create `
            --name tracseq-backend `
            --resource-group $ResourceGroup `
            --environment $ContainerEnv `
            --image "$registryLoginServer/tracseq-backend:latest" `
            --registry-server $registryLoginServer `
            --registry-username $registryUsername `
            --registry-password $registryPassword `
            --target-port 3000 `
            --ingress external `
            --cpu 0.5 `
            --memory 1.0Gi `
            --min-replicas 1 `
            --max-replicas 5 `
            --env-vars "DATABASE_URL=$dbConnectionString" "RUST_LOG=info" "RAG_SERVICE_URL=$ragServiceUrl" "JWT_SECRET=$jwtSecretPlain" "STORAGE_PATH=/app/storage" `
            --output none
        
        # Get backend service URL
        $backendServiceFqdn = az containerapp show --name tracseq-backend --resource-group $ResourceGroup --query properties.configuration.ingress.fqdn --output tsv
        $backendServiceUrl = "https://$backendServiceFqdn"
        
        # Deploy frontend
        Write-Info "Deploying frontend..."
        az containerapp create `
            --name tracseq-frontend `
            --resource-group $ResourceGroup `
            --environment $ContainerEnv `
            --image "$registryLoginServer/tracseq-frontend:latest" `
            --registry-server $registryLoginServer `
            --registry-username $registryUsername `
            --registry-password $registryPassword `
            --target-port 80 `
            --ingress external `
            --cpu 0.25 `
            --memory 0.5Gi `
            --min-replicas 1 `
            --max-replicas 3 `
            --env-vars "NODE_ENV=production" "BACKEND_URL=$backendServiceUrl" `
            --output none
        
        # Get frontend service URL
        $frontendServiceFqdn = az containerapp show --name tracseq-frontend --resource-group $ResourceGroup --query properties.configuration.ingress.fqdn --output tsv
        $frontendServiceUrl = "https://$frontendServiceFqdn"
        
        Write-Success "Container apps deployed successfully!"
        
        # Store secrets in Key Vault
        Write-Info "Storing secrets in Key Vault..."
        az keyvault secret set --vault-name $KeyVaultName --name "database-url" --value $dbConnectionString --output none
        az keyvault secret set --vault-name $KeyVaultName --name "jwt-secret" --value $jwtSecretPlain --output none
        
        # Output deployment information
        Write-Host ""
        Write-Success "🎉 Deployment completed successfully!"
        Write-Host ""
        Write-Host "📊 Service URLs:" -ForegroundColor Cyan
        Write-Host "   Frontend:    $frontendServiceUrl" -ForegroundColor White
        Write-Host "   Backend:     $backendServiceUrl" -ForegroundColor White
        Write-Host "   RAG Service: $ragServiceUrl" -ForegroundColor White
        Write-Host "   Ollama:      $ollamaServiceUrl (internal)" -ForegroundColor White
        Write-Host ""
        Write-Host "🔧 Resource Details:" -ForegroundColor Cyan
        Write-Host "   Resource Group: $ResourceGroup" -ForegroundColor White
        Write-Host "   Database:       $DBServerName.postgres.database.azure.com" -ForegroundColor White
        Write-Host "   Registry:       $registryLoginServer" -ForegroundColor White
        Write-Host "   Key Vault:      $KeyVaultName" -ForegroundColor White
        Write-Host ""
        Write-Host "🤖 LLM Configuration:" -ForegroundColor Cyan
        Write-Host "   Provider:       Ollama (local inference)" -ForegroundColor White
        Write-Host "   Model:          llama3.2:3b" -ForegroundColor White
        Write-Host "   Cost:           No per-token charges!" -ForegroundColor White
        Write-Host ""
        Write-Host "🔍 Next Steps:" -ForegroundColor Cyan
        Write-Host "   1. Wait 5-10 minutes for Ollama to download the model" -ForegroundColor White
        Write-Host "   2. Visit the frontend URL to access TracSeq 2.0" -ForegroundColor White
        Write-Host "   3. Test RAG functionality with document upload" -ForegroundColor White
        Write-Host "   4. Monitor services: az containerapp logs show --name [service-name] --resource-group $ResourceGroup --follow" -ForegroundColor White
        Write-Host "   5. Set up custom domain (optional)" -ForegroundColor White
        Write-Host ""
        
        return @{
            Frontend = $frontendServiceUrl
            Backend = $backendServiceUrl
            RAG = $ragServiceUrl
            Ollama = $ollamaServiceUrl
        }
    }
    catch {
        Write-Error "Failed to deploy container apps: $($_.Exception.Message)"
        throw
    }
}

# Function to create service principal for GitHub Actions
function New-ServicePrincipal {
    Write-Info "Creating service principal for GitHub Actions CI/CD..."
    
    try {
        $subscriptionId = az account show --query id --output tsv
        
        # Create service principal
        $spOutput = az ad sp create-for-rbac `
            --name "tracseq-deploy-sp" `
            --role contributor `
            --scopes "/subscriptions/$subscriptionId/resourceGroups/$ResourceGroup" `
            --sdk-auth
        
        Write-Success "Service principal created!"
        Write-Info "Add this JSON as 'AZURE_CREDENTIALS' secret in your GitHub repository:"
        Write-Host ""
        Write-Host $spOutput -ForegroundColor Yellow
        Write-Host ""
    }
    catch {
        Write-Error "Failed to create service principal: $($_.Exception.Message)"
        throw
    }
}

# Function to cleanup on error
function Invoke-Cleanup {
    Write-Error "Deployment failed. Cleaning up resources..."
    try {
        az group delete --name $ResourceGroup --yes --no-wait 2>$null
    }
    catch {
        # Ignore cleanup errors
    }
}

# Main deployment flow
function Start-Deployment {
    Write-Host "🚀 TracSeq 2.0 Azure Deployment" -ForegroundColor Magenta
    Write-Host "================================" -ForegroundColor Magenta
    Write-Host ""
    
    try {
        if (-not $SkipPrerequisiteCheck) {
            Test-Prerequisites
        }
        
        Get-UserInput
        New-AzureResources
        Build-AndPushImages
        $deploymentUrls = Deploy-ContainerApps
        
        $createSP = Read-Host "Would you like to create a service principal for GitHub Actions? (y/n)"
        if ($createSP -eq 'y' -or $createSP -eq 'Y') {
            New-ServicePrincipal
        }
        
        Write-Success "🎉 TracSeq 2.0 is now running on Azure!"
        
        return $deploymentUrls
    }
    catch {
        Write-Error "Deployment failed: $($_.Exception.Message)"
        Invoke-Cleanup
        exit 1
    }
}

# Run the deployment
if ($MyInvocation.InvocationName -ne '.') {
    Start-Deployment
} 
