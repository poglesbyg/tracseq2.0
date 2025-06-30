#!/bin/bash

# 🚀 TracSeq 2.0 Azure Deployment Script
# Automated deployment to Azure Container Apps

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
RESOURCE_GROUP="tracseq-rg"
LOCATION="eastus"
CONTAINER_ENV="tracseq-env"
REGISTRY_NAME="tracseqregistry"
DB_SERVER_NAME="tracseq-db-server"
DB_NAME="lab_manager"
DB_USER="tracseqadmin"
KEY_VAULT_NAME="tracseq-kv"

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check prerequisites
check_prerequisites() {
    print_info "Checking prerequisites..."
    
    # Check if Azure CLI is installed
    if ! command -v az &> /dev/null; then
        print_error "Azure CLI is not installed. Please install it first."
        exit 1
    fi
    
    # Check if Docker is installed
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed. Please install it first."
        exit 1
    fi
    
    # Check if user is logged in to Azure
    if ! az account show &> /dev/null; then
        print_error "You are not logged in to Azure. Please run 'az login' first."
        exit 1
    fi
    
    print_success "Prerequisites check passed!"
}

# Function to prompt for user input
get_user_input() {
    print_info "Configuring deployment parameters..."
    
    read -p "Enter resource group name (default: $RESOURCE_GROUP): " input
    RESOURCE_GROUP=${input:-$RESOURCE_GROUP}
    
    read -p "Enter Azure region (default: $LOCATION): " input
    LOCATION=${input:-$LOCATION}
    
    read -p "Enter container registry name (default: $REGISTRY_NAME): " input
    REGISTRY_NAME=${input:-$REGISTRY_NAME}
    
    read -s -p "Enter database admin password: " DB_PASSWORD
    echo
    
    if [[ -z "$DB_PASSWORD" ]]; then
        print_error "Database password cannot be empty!"
        exit 1
    fi
    
    read -s -p "Enter JWT secret: " JWT_SECRET
    echo
    
    if [[ -z "$JWT_SECRET" ]]; then
        print_error "JWT secret cannot be empty!"
        exit 1
    fi
}

# Function to create Azure resources
create_azure_resources() {
    print_info "Creating Azure resources..."
    
    # Create resource group
    print_info "Creating resource group: $RESOURCE_GROUP"
    az group create --name $RESOURCE_GROUP --location $LOCATION > /dev/null
    
    # Create Container Apps environment
    print_info "Creating Container Apps environment: $CONTAINER_ENV"
    az containerapp env create \
        --name $CONTAINER_ENV \
        --resource-group $RESOURCE_GROUP \
        --location $LOCATION > /dev/null
    
    # Create container registry
    print_info "Creating container registry: $REGISTRY_NAME"
    az acr create \
        --resource-group $RESOURCE_GROUP \
        --name $REGISTRY_NAME \
        --sku Basic \
        --admin-enabled true > /dev/null
    
    # Create PostgreSQL database
    print_info "Creating PostgreSQL database: $DB_SERVER_NAME"
    az postgres flexible-server create \
        --resource-group $RESOURCE_GROUP \
        --name $DB_SERVER_NAME \
        --location $LOCATION \
        --admin-user $DB_USER \
        --admin-password "$DB_PASSWORD" \
        --sku-name Standard_B1ms \
        --tier Burstable \
        --version 15 > /dev/null
    
    # Create database
    print_info "Creating database: $DB_NAME"
    az postgres flexible-server db create \
        --resource-group $RESOURCE_GROUP \
        --server-name $DB_SERVER_NAME \
        --database-name $DB_NAME > /dev/null
    
    # Configure firewall
    print_info "Configuring database firewall rules"
    az postgres flexible-server firewall-rule create \
        --resource-group $RESOURCE_GROUP \
        --name $DB_SERVER_NAME \
        --rule-name AllowAzureServices \
        --start-ip-address 0.0.0.0 \
        --end-ip-address 0.0.0.0 > /dev/null
    
    # Create Key Vault
    print_info "Creating Key Vault: $KEY_VAULT_NAME"
    az keyvault create \
        --name $KEY_VAULT_NAME \
        --resource-group $RESOURCE_GROUP \
        --location $LOCATION > /dev/null
    
    print_success "Azure resources created successfully!"
}

# Function to build and push Docker images
build_and_push_images() {
    print_info "Building and pushing Docker images..."
    
    # Login to container registry
    az acr login --name $REGISTRY_NAME
    
    # Get registry login server
    REGISTRY_LOGIN_SERVER=$(az acr show --name $REGISTRY_NAME --resource-group $RESOURCE_GROUP --query loginServer --output tsv)
    
    # Build and push frontend
    print_info "Building frontend image..."
    docker build -t $REGISTRY_LOGIN_SERVER/tracseq-frontend:latest ./lab_manager/frontend
    print_info "Pushing frontend image..."
    docker push $REGISTRY_LOGIN_SERVER/tracseq-frontend:latest
    
    # Build and push backend
    print_info "Building backend image..."
    docker build -t $REGISTRY_LOGIN_SERVER/tracseq-backend:latest ./lab_manager
    print_info "Pushing backend image..."
    docker push $REGISTRY_LOGIN_SERVER/tracseq-backend:latest
    
    # Build and push RAG service
    print_info "Building RAG service image..."
    docker build -t $REGISTRY_LOGIN_SERVER/tracseq-rag:latest ./lab_submission_rag
    print_info "Pushing RAG service image..."
    docker push $REGISTRY_LOGIN_SERVER/tracseq-rag:latest
    
    # Pull and push Ollama image (we don't build this, just use official image)
    print_info "Pulling and pushing Ollama image..."
    docker pull ollama/ollama:latest
    docker tag ollama/ollama:latest $REGISTRY_LOGIN_SERVER/tracseq-ollama:latest
    docker push $REGISTRY_LOGIN_SERVER/tracseq-ollama:latest
    
    print_success "Docker images built and pushed successfully!"
}

# Function to deploy container apps
deploy_container_apps() {
    print_info "Deploying container apps..."
    
    # Get registry credentials
    REGISTRY_LOGIN_SERVER=$(az acr show --name $REGISTRY_NAME --resource-group $RESOURCE_GROUP --query loginServer --output tsv)
    REGISTRY_USERNAME=$(az acr credential show --name $REGISTRY_NAME --resource-group $RESOURCE_GROUP --query username --output tsv)
    REGISTRY_PASSWORD=$(az acr credential show --name $REGISTRY_NAME --resource-group $RESOURCE_GROUP --query passwords[0].value --output tsv)
    
    # Database connection string
    DB_CONNECTION_STRING="postgresql://$DB_USER:$DB_PASSWORD@$DB_SERVER_NAME.postgres.database.azure.com:5432/$DB_NAME?sslmode=require"
    
    # Deploy Ollama service first (RAG service dependency)
    print_info "Deploying Ollama service..."
    az containerapp create \
        --name tracseq-ollama \
        --resource-group $RESOURCE_GROUP \
        --environment $CONTAINER_ENV \
        --image $REGISTRY_LOGIN_SERVER/tracseq-ollama:latest \
        --registry-server $REGISTRY_LOGIN_SERVER \
        --registry-username $REGISTRY_USERNAME \
        --registry-password $REGISTRY_PASSWORD \
        --target-port 11434 \
        --ingress internal \
        --cpu 1.0 \
        --memory 4.0Gi \
        --min-replicas 1 \
        --max-replicas 2 \
        --startup-probe-initial-delay 30 \
        --startup-probe-period 10 \
        --startup-probe-timeout 5 \
        --startup-probe-failure-threshold 10 > /dev/null
    
    # Get Ollama service internal URL
    OLLAMA_SERVICE_FQDN=$(az containerapp show --name tracseq-ollama --resource-group $RESOURCE_GROUP --query properties.configuration.ingress.fqdn --output tsv)
    OLLAMA_SERVICE_URL="https://$OLLAMA_SERVICE_FQDN"
    
    # Deploy RAG service
    print_info "Deploying RAG service..."
    az containerapp create \
        --name tracseq-rag \
        --resource-group $RESOURCE_GROUP \
        --environment $CONTAINER_ENV \
        --image $REGISTRY_LOGIN_SERVER/tracseq-rag:latest \
        --registry-server $REGISTRY_LOGIN_SERVER \
        --registry-username $REGISTRY_USERNAME \
        --registry-password $REGISTRY_PASSWORD \
        --target-port 8000 \
        --ingress external \
        --cpu 1.0 \
        --memory 2.0Gi \
        --min-replicas 1 \
        --max-replicas 3 \
        --env-vars \
            POSTGRES_URL="$DB_CONNECTION_STRING" \
            RAG_LOG_LEVEL=INFO \
            USE_OLLAMA=true \
            LLM_PROVIDER=ollama \
            OLLAMA_BASE_URL="$OLLAMA_SERVICE_URL" \
            OLLAMA_MODEL=llama3.2:3b > /dev/null
    
    # Get RAG service URL
    RAG_SERVICE_URL=$(az containerapp show --name tracseq-rag --resource-group $RESOURCE_GROUP --query properties.configuration.ingress.fqdn --output tsv)
    RAG_SERVICE_URL="https://$RAG_SERVICE_URL"
    
    # Deploy backend service
    print_info "Deploying backend service..."
    az containerapp create \
        --name tracseq-backend \
        --resource-group $RESOURCE_GROUP \
        --environment $CONTAINER_ENV \
        --image $REGISTRY_LOGIN_SERVER/tracseq-backend:latest \
        --registry-server $REGISTRY_LOGIN_SERVER \
        --registry-username $REGISTRY_USERNAME \
        --registry-password $REGISTRY_PASSWORD \
        --target-port 3000 \
        --ingress external \
        --cpu 0.5 \
        --memory 1.0Gi \
        --min-replicas 1 \
        --max-replicas 5 \
        --env-vars \
            DATABASE_URL="$DB_CONNECTION_STRING" \
            RUST_LOG=info \
            RAG_SERVICE_URL="$RAG_SERVICE_URL" \
            JWT_SECRET="$JWT_SECRET" \
            STORAGE_PATH="/app/storage" > /dev/null
    
    # Get backend service URL
    BACKEND_SERVICE_URL=$(az containerapp show --name tracseq-backend --resource-group $RESOURCE_GROUP --query properties.configuration.ingress.fqdn --output tsv)
    BACKEND_SERVICE_URL="https://$BACKEND_SERVICE_URL"
    
    # Deploy frontend
    print_info "Deploying frontend..."
    az containerapp create \
        --name tracseq-frontend \
        --resource-group $RESOURCE_GROUP \
        --environment $CONTAINER_ENV \
        --image $REGISTRY_LOGIN_SERVER/tracseq-frontend:latest \
        --registry-server $REGISTRY_LOGIN_SERVER \
        --registry-username $REGISTRY_USERNAME \
        --registry-password $REGISTRY_PASSWORD \
        --target-port 80 \
        --ingress external \
        --cpu 0.25 \
        --memory 0.5Gi \
        --min-replicas 1 \
        --max-replicas 3 \
        --env-vars \
            NODE_ENV=production \
            BACKEND_URL="$BACKEND_SERVICE_URL" > /dev/null
    
    # Get frontend service URL
    FRONTEND_SERVICE_URL=$(az containerapp show --name tracseq-frontend --resource-group $RESOURCE_GROUP --query properties.configuration.ingress.fqdn --output tsv)
    FRONTEND_SERVICE_URL="https://$FRONTEND_SERVICE_URL"
    
    print_success "Container apps deployed successfully!"
    
    # Store secrets in Key Vault
    print_info "Storing secrets in Key Vault..."
    az keyvault secret set --vault-name $KEY_VAULT_NAME --name "database-url" --value "$DB_CONNECTION_STRING" > /dev/null
    az keyvault secret set --vault-name $KEY_VAULT_NAME --name "jwt-secret" --value "$JWT_SECRET" > /dev/null
    
    # Output deployment information
    echo
    print_success "🎉 Deployment completed successfully!"
    echo
    echo "📊 Service URLs:"
    echo "   Frontend:    $FRONTEND_SERVICE_URL"
    echo "   Backend:     $BACKEND_SERVICE_URL"
    echo "   RAG Service: $RAG_SERVICE_URL"
    echo "   Ollama:      $OLLAMA_SERVICE_URL (internal)"
    echo
    echo "🔧 Resource Details:"
    echo "   Resource Group: $RESOURCE_GROUP"
    echo "   Database:       $DB_SERVER_NAME.postgres.database.azure.com"
    echo "   Registry:       $REGISTRY_LOGIN_SERVER"
    echo "   Key Vault:      $KEY_VAULT_NAME"
    echo
    echo "🤖 LLM Configuration:"
    echo "   Provider:       Ollama (local inference)"
    echo "   Model:          llama3.2:3b"
    echo "   Cost:           No per-token charges!"
    echo
    echo "🔍 Next Steps:"
    echo "   1. Wait 5-10 minutes for Ollama to download the model"
    echo "   2. Visit the frontend URL to access TracSeq 2.0"
    echo "   3. Test RAG functionality with document upload"
    echo "   4. Monitor services: az containerapp logs show --name [service-name] --resource-group $RESOURCE_GROUP --follow"
    echo "   5. Set up custom domain (optional)"
    echo
}

# Function to create Azure service principal for GitHub Actions
create_service_principal() {
    print_info "Creating service principal for GitHub Actions CI/CD..."
    
    SUBSCRIPTION_ID=$(az account show --query id --output tsv)
    
    # Create service principal
    SP_OUTPUT=$(az ad sp create-for-rbac \
        --name "tracseq-deploy-sp" \
        --role contributor \
        --scopes "/subscriptions/$SUBSCRIPTION_ID/resourceGroups/$RESOURCE_GROUP" \
        --sdk-auth)
    
    print_success "Service principal created!"
    print_info "Add this JSON as 'AZURE_CREDENTIALS' secret in your GitHub repository:"
    echo
    echo "$SP_OUTPUT"
    echo
}

# Function to clean up on error
cleanup_on_error() {
    print_error "Deployment failed. Cleaning up resources..."
    az group delete --name $RESOURCE_GROUP --yes --no-wait 2>/dev/null || true
}

# Trap to cleanup on error
trap cleanup_on_error ERR

# Main deployment flow
main() {
    echo "🚀 TracSeq 2.0 Azure Deployment"
    echo "================================"
    echo
    
    check_prerequisites
    get_user_input
    create_azure_resources
    build_and_push_images
    deploy_container_apps
    
    read -p "Would you like to create a service principal for GitHub Actions? (y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        create_service_principal
    fi
    
    print_success "🎉 TracSeq 2.0 is now running on Azure!"
}

# Run main function
main "$@" 
