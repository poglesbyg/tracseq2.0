# 🌐 Azure Deployment Guide for TracSeq 2.0

This guide covers deploying TracSeq 2.0 to Microsoft Azure using modern cloud-native services optimized for your multi-service architecture.

## 🏗️ Architecture Overview

TracSeq 2.0 on Azure will use:
- **Azure Container Apps** - For microservices (Frontend, Backend, RAG Service)
- **Azure Database for PostgreSQL** - Managed database service
- **Azure Container Registry** - Private container registry
- **Azure Key Vault** - Secrets management
- **Azure Application Insights** - Monitoring and logging
- **Azure GitHub Actions** - CI/CD pipeline

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   React SPA     │    │  Rust Backend   │    │  Python RAG     │
│ (Container App) │◄──►│ (Container App) │◄──►│ (Container App) │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 ▼
                    ┌─────────────────┐
                    │ Azure Database  │
                    │ for PostgreSQL  │
                    └─────────────────┘
```

## 📋 Prerequisites

- Azure account with active subscription
- Azure CLI installed
- Docker Desktop
- Git repository access
- Domain name (optional, for custom domains)

## 🚀 Quick Deployment (One-Click)

### Option 1: Deploy Button (Recommended)
[![Deploy to Azure](https://aka.ms/deploytoazurebutton)](https://portal.azure.com/#create/Microsoft.Template)

### Option 2: Azure CLI Deployment
```bash
# Clone and deploy
git clone https://github.com/your-username/tracseq2.0.git
cd tracseq2.0
./deploy/azure/deploy.sh
```

## 🛠️ Manual Setup Guide

### Step 1: Azure Resource Setup

```bash
# Login to Azure
az login

# Set subscription (replace with your subscription ID)
az account set --subscription "your-subscription-id"

# Create resource group
az group create \
  --name tracseq-rg \
  --location eastus

# Create Container Apps environment
az containerapp env create \
  --name tracseq-env \
  --resource-group tracseq-rg \
  --location eastus
```

### Step 2: Database Setup

```bash
# Create Azure Database for PostgreSQL
az postgres flexible-server create \
  --resource-group tracseq-rg \
  --name tracseq-db-server \
  --location eastus \
  --admin-user tracseqadmin \
  --admin-password "SecurePassword123!" \
  --sku-name Standard_B1ms \
  --tier Burstable \
  --version 15

# Create database
az postgres flexible-server db create \
  --resource-group tracseq-rg \
  --server-name tracseq-db-server \
  --database-name lab_manager

# Configure firewall (allow Azure services)
az postgres flexible-server firewall-rule create \
  --resource-group tracseq-rg \
  --name tracseq-db-server \
  --rule-name AllowAzureServices \
  --start-ip-address 0.0.0.0 \
  --end-ip-address 0.0.0.0
```

### Step 3: Container Registry

```bash
# Create container registry
az acr create \
  --resource-group tracseq-rg \
  --name tracseqregistry \
  --sku Basic \
  --admin-enabled true

# Login to registry
az acr login --name tracseqregistry
```

### Step 4: Build and Push Images

```bash
# Get registry login server
REGISTRY_NAME=$(az acr show --name tracseqregistry --resource-group tracseq-rg --query loginServer --output tsv)

# Build and push frontend
docker build -t $REGISTRY_NAME/tracseq-frontend:latest ./lab_manager/frontend
docker push $REGISTRY_NAME/tracseq-frontend:latest

# Build and push backend
docker build -t $REGISTRY_NAME/tracseq-backend:latest ./lab_manager
docker push $REGISTRY_NAME/tracseq-backend:latest

# Build and push RAG service
docker build -t $REGISTRY_NAME/tracseq-rag:latest ./lab_submission_rag
docker push $REGISTRY_NAME/tracseq-rag:latest
```

### Step 5: Deploy Container Apps

```bash
# Get registry credentials
REGISTRY_USERNAME=$(az acr credential show --name tracseqregistry --resource-group tracseq-rg --query username --output tsv)
REGISTRY_PASSWORD=$(az acr credential show --name tracseqregistry --resource-group tracseq-rg --query passwords[0].value --output tsv)

# Deploy RAG service first (backend dependency)
az containerapp create \
  --name tracseq-rag \
  --resource-group tracseq-rg \
  --environment tracseq-env \
  --image $REGISTRY_NAME/tracseq-rag:latest \
  --registry-server $REGISTRY_NAME \
  --registry-username $REGISTRY_USERNAME \
  --registry-password $REGISTRY_PASSWORD \
  --target-port 8000 \
  --ingress external \
  --cpu 1.0 \
  --memory 2.0Gi \
  --min-replicas 1 \
  --max-replicas 5 \
  --env-vars \
    POSTGRES_URL="postgresql://tracseqadmin:SecurePassword123!@tracseq-db-server.postgres.database.azure.com:5432/lab_manager?sslmode=require" \
    RAG_LOG_LEVEL=INFO

# Deploy backend service
az containerapp create \
  --name tracseq-backend \
  --resource-group tracseq-rg \
  --environment tracseq-env \
  --image $REGISTRY_NAME/tracseq-backend:latest \
  --registry-server $REGISTRY_NAME \
  --registry-username $REGISTRY_USERNAME \
  --registry-password $REGISTRY_PASSWORD \
  --target-port 3000 \
  --ingress external \
  --cpu 0.5 \
  --memory 1.0Gi \
  --min-replicas 1 \
  --max-replicas 10 \
  --env-vars \
    DATABASE_URL="postgresql://tracseqadmin:SecurePassword123!@tracseq-db-server.postgres.database.azure.com:5432/lab_manager?sslmode=require" \
    RUST_LOG=info \
    RAG_SERVICE_URL="https://tracseq-rag.internal.azurecontainerapps.io" \
    JWT_SECRET="your-super-secret-jwt-key-change-in-production"

# Deploy frontend
az containerapp create \
  --name tracseq-frontend \
  --resource-group tracseq-rg \
  --environment tracseq-env \
  --image $REGISTRY_NAME/tracseq-frontend:latest \
  --registry-server $REGISTRY_NAME \
  --registry-username $REGISTRY_USERNAME \
  --registry-password $REGISTRY_PASSWORD \
  --target-port 80 \
  --ingress external \
  --cpu 0.25 \
  --memory 0.5Gi \
  --min-replicas 1 \
  --max-replicas 5 \
  --env-vars \
    NODE_ENV=production \
    BACKEND_URL="https://tracseq-backend.internal.azurecontainerapps.io"
```

### Step 6: Configure Secrets (Security)

```bash
# Create Key Vault
az keyvault create \
  --name tracseq-kv \
  --resource-group tracseq-rg \
  --location eastus

# Store secrets
az keyvault secret set --vault-name tracseq-kv --name "database-url" --value "postgresql://tracseqadmin:SecurePassword123!@tracseq-db-server.postgres.database.azure.com:5432/lab_manager?sslmode=require"
az keyvault secret set --vault-name tracseq-kv --name "jwt-secret" --value "your-super-secret-jwt-key-change-in-production"
```

## 🔄 CI/CD Pipeline Setup

### GitHub Actions Workflow

Create `.github/workflows/azure-deploy.yml`:

```yaml
name: Deploy to Azure Container Apps

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  REGISTRY_NAME: tracseqregistry
  RESOURCE_GROUP: tracseq-rg

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Login to Azure
      uses: azure/login@v1
      with:
        creds: ${{ secrets.AZURE_CREDENTIALS }}
    
    - name: Login to Container Registry
      run: az acr login --name ${{ env.REGISTRY_NAME }}
    
    - name: Build and push frontend
      run: |
        docker build -t ${{ env.REGISTRY_NAME }}.azurecr.io/tracseq-frontend:${{ github.sha }} ./lab_manager/frontend
        docker push ${{ env.REGISTRY_NAME }}.azurecr.io/tracseq-frontend:${{ github.sha }}
    
    - name: Build and push backend
      run: |
        docker build -t ${{ env.REGISTRY_NAME }}.azurecr.io/tracseq-backend:${{ github.sha }} ./lab_manager
        docker push ${{ env.REGISTRY_NAME }}.azurecr.io/tracseq-backend:${{ github.sha }}
    
    - name: Build and push RAG service
      run: |
        docker build -t ${{ env.REGISTRY_NAME }}.azurecr.io/tracseq-rag:${{ github.sha }} ./lab_submission_rag
        docker push ${{ env.REGISTRY_NAME }}.azurecr.io/tracseq-rag:${{ github.sha }}
    
    - name: Deploy to Container Apps
      run: |
        # Update RAG service
        az containerapp update \
          --name tracseq-rag \
          --resource-group ${{ env.RESOURCE_GROUP }} \
          --image ${{ env.REGISTRY_NAME }}.azurecr.io/tracseq-rag:${{ github.sha }}
        
        # Update backend
        az containerapp update \
          --name tracseq-backend \
          --resource-group ${{ env.RESOURCE_GROUP }} \
          --image ${{ env.REGISTRY_NAME }}.azurecr.io/tracseq-backend:${{ github.sha }}
        
        # Update frontend
        az containerapp update \
          --name tracseq-frontend \
          --resource-group ${{ env.RESOURCE_GROUP }} \
          --image ${{ env.REGISTRY_NAME }}.azurecr.io/tracseq-frontend:${{ github.sha }}
```

### GitHub Secrets Setup

In your GitHub repository, add these secrets:

1. **AZURE_CREDENTIALS**: Service principal JSON
```bash
# Create service principal
az ad sp create-for-rbac --name "tracseq-deploy" --role contributor --scopes /subscriptions/{subscription-id}/resourceGroups/tracseq-rg --sdk-auth
```

## 🔧 Configuration Files

### Environment Variables for Azure

Create `deploy/azure/azure.env`:

```env
# Azure-specific environment configuration
AZURE_REGION=eastus
RESOURCE_GROUP=tracseq-rg
CONTAINER_ENVIRONMENT=tracseq-env
REGISTRY_NAME=tracseqregistry

# Database Configuration
DATABASE_URL=postgresql://tracseqadmin:SecurePassword123!@tracseq-db-server.postgres.database.azure.com:5432/lab_manager?sslmode=require

# Service URLs (Internal Azure Container Apps communication)
RAG_SERVICE_URL=https://tracseq-rag.internal.azurecontainerapps.io
BACKEND_URL=https://tracseq-backend.internal.azurecontainerapps.io

# Security
JWT_SECRET=your-super-secret-jwt-key-change-in-production
RUST_LOG=info
NODE_ENV=production

# Application Settings
STORAGE_PATH=/app/storage
RAG_ENABLED=true
RAG_TIMEOUT_SECONDS=300
```

## 📊 Monitoring and Logging

### Application Insights Setup

```bash
# Create Application Insights
az monitor app-insights component create \
  --app tracseq-insights \
  --location eastus \
  --resource-group tracseq-rg \
  --kind web

# Get instrumentation key
INSTRUMENTATION_KEY=$(az monitor app-insights component show --app tracseq-insights --resource-group tracseq-rg --query instrumentationKey --output tsv)

# Update container apps with monitoring
az containerapp update \
  --name tracseq-backend \
  --resource-group tracseq-rg \
  --set-env-vars APPINSIGHTS_INSTRUMENTATIONKEY=$INSTRUMENTATION_KEY
```

## 💰 Cost Optimization

### Resource Sizing Recommendations

| Service | CPU | Memory | Replicas | Estimated Cost/Month |
|---------|-----|--------|----------|---------------------|
| Frontend | 0.25 | 0.5Gi | 1-3 | $15-45 |
| Backend | 0.5 | 1.0Gi | 1-5 | $30-150 |
| RAG Service | 1.0 | 2.0Gi | 1-3 | $60-180 |
| Ollama | 1.0 | 4.0Gi | 1-2 | $120-240 |
| PostgreSQL | Standard_B1ms | - | - | $25 |
| **Total** | - | - | - | **$250-640/month** |

### Cost-Saving Tips

1. **Use Consumption Tier**: Container Apps Consumption for dev/test
2. **Auto-scaling**: Configure appropriate min/max replicas
3. **Reserved Instances**: For PostgreSQL in production
4. **Resource Tags**: For cost tracking and management

## 🔒 Security Best Practices

### Network Security
```bash
# Restrict database access to Container Apps only
az postgres flexible-server firewall-rule create \
  --resource-group tracseq-rg \
  --name tracseq-db-server \
  --rule-name AllowContainerApps \
  --start-ip-address 10.0.0.0 \
  --end-ip-address 10.255.255.255
```

### Key Vault Integration
```bash
# Enable managed identity for container apps
az containerapp identity assign \
  --name tracseq-backend \
  --resource-group tracseq-rg \
  --system-assigned

# Grant Key Vault access
az keyvault set-policy \
  --name tracseq-kv \
  --object-id $(az containerapp identity show --name tracseq-backend --resource-group tracseq-rg --query principalId --output tsv) \
  --secret-permissions get list
```

## 🌐 Custom Domain Setup

```bash
# Add custom domain to frontend container app
az containerapp hostname add \
  --hostname tracseq.yourdomain.com \
  --name tracseq-frontend \
  --resource-group tracseq-rg

# Configure SSL certificate (managed certificate)
az containerapp ssl upload \
  --hostname tracseq.yourdomain.com \
  --name tracseq-frontend \
  --resource-group tracseq-rg \
  --certificate-file path/to/certificate.pfx \
  --certificate-password password
```

## 🚨 Troubleshooting

### Common Issues

**Container App Won't Start**
```bash
# Check logs
az containerapp logs show \
  --name tracseq-backend \
  --resource-group tracseq-rg \
  --follow

# Check resource allocation
az containerapp show \
  --name tracseq-backend \
  --resource-group tracseq-rg \
  --query properties.template.containers[0].resources
```

**Database Connection Issues**
```bash
# Test database connectivity
az postgres flexible-server connect \
  --name tracseq-db-server \
  --admin-user tracseqadmin \
  --admin-password SecurePassword123!
```

**Registry Authentication Issues**
```bash
# Refresh registry credentials
az acr login --name tracseqregistry
az acr credential show --name tracseqregistry --resource-group tracseq-rg
```

## 📈 Scaling and Performance

### Auto-scaling Configuration
```bash
# Configure CPU-based scaling
az containerapp update \
  --name tracseq-backend \
  --resource-group tracseq-rg \
  --min-replicas 1 \
  --max-replicas 10 \
  --scale-rule-name cpu-scale \
  --scale-rule-type cpu \
  --scale-rule-metadata cpuThreshold=70
```

### Performance Monitoring
```bash
# Set up alerts
az monitor metrics alert create \
  --name "High CPU Usage" \
  --resource-group tracseq-rg \
  --condition "avg Percentage CPU > 80" \
  --description "Alert when CPU usage is high"
```

## 🔄 Backup and Disaster Recovery

### Database Backups
```bash
# Configure automated backups
az postgres flexible-server parameter set \
  --resource-group tracseq-rg \
  --server-name tracseq-db-server \
  --name backup_retention_days \
  --value 30

# Manual backup
az postgres flexible-server backup create \
  --resource-group tracseq-rg \
  --server-name tracseq-db-server \
  --backup-name manual-backup-$(date +%Y%m%d)
```

## 🎯 Next Steps

1. **Deploy to Development**: Use staging slots for testing
2. **Set up Monitoring**: Configure alerts and dashboards
3. **Performance Testing**: Load test your deployment
4. **Security Review**: Implement additional security measures
5. **Documentation**: Update team documentation with Azure URLs

## 📞 Support

- 📧 **Azure Support**: Create support ticket in Azure Portal
- 📚 **Documentation**: [Azure Container Apps Docs](https://docs.microsoft.com/en-us/azure/container-apps/)
- 💬 **Community**: [Azure Community Forums](https://techcommunity.microsoft.com/t5/azure/ct-p/Azure)

---

**Ready to go live with TracSeq 2.0 on Azure! 🚀** 
