# 🚀 Quick Start: Deploy TracSeq 2.0 to Azure

## ⚡ Fastest Deployment Options

### Option 1: Automated Script (Recommended)

**For Windows (PowerShell):**
```powershell
# Open PowerShell as Administrator
cd tracseq2.0
./deploy/azure/deploy.ps1
```

**For Linux/macOS (Bash):**
```bash
cd tracseq2.0
./deploy/azure/deploy.sh
```

### Option 2: Manual Azure CLI

Follow the step-by-step guide in [README.md](README.md) for complete control over the deployment process.

## 📋 Prerequisites Checklist

- [ ] Azure account with active subscription
- [ ] Azure CLI installed ([Download](https://docs.microsoft.com/en-us/cli/azure/install-azure-cli))
- [ ] Docker Desktop running ([Download](https://www.docker.com/products/docker-desktop))
- [ ] Logged into Azure: `az login`
- [ ] Git repository access

## 🎯 Quick Commands

### Login and Setup
```bash
# Login to Azure
az login

# Set your subscription
az account set --subscription "your-subscription-id"

# Verify login
az account show
```

### Run Deployment
```bash
# Clone repository (if needed)
git clone https://github.com/your-username/tracseq2.0.git
cd tracseq2.0

# Run deployment script
./deploy/azure/deploy.sh    # Linux/macOS
# OR
./deploy/azure/deploy.ps1   # Windows PowerShell
```

## 📊 What Gets Deployed

| Service | Type | Purpose |
|---------|------|---------|
| **Frontend** | Container App | React SPA for user interface |
| **Backend** | Container App | Rust API server |
| **RAG Service** | Container App | Python AI document processing |
| **Ollama** | Container App | Local LLM inference (llama3.2:3b) |
| **Database** | PostgreSQL | Managed database service |
| **Registry** | Container Registry | Private Docker images |
| **Key Vault** | Secrets | Secure configuration storage |

## 🔑 Required Information

During deployment, you'll be prompted for:

1. **Resource Group Name** (default: `tracseq-rg`)
2. **Azure Region** (default: `eastus`)
3. **Container Registry Name** (default: `tracseqregistry`)
4. **Database Admin Password** (secure input)
5. **JWT Secret** (secure input)

## ⏱️ Deployment Timeline

| Phase | Duration | Description |
|-------|----------|-------------|
| Resource Creation | 5-10 min | Azure infrastructure setup |
| Image Building | 10-15 min | Docker container builds (includes Ollama) |
| Service Deployment | 5-10 min | Container Apps deployment |
| Model Download | 5-10 min | Ollama downloads llama3.2:3b (~2GB) |
| **Total** | **25-45 min** | Complete deployment |

## 🌐 Access Your Application

After deployment completes, you'll receive:

```
📊 Service URLs:
   Frontend:    https://tracseq-frontend.xyz.azurecontainerapps.io
   Backend:     https://tracseq-backend.xyz.azurecontainerapps.io  
   RAG Service: https://tracseq-rag.xyz.azurecontainerapps.io
   Ollama:      https://tracseq-ollama.xyz.azurecontainerapps.io (internal)

🤖 LLM Configuration:
   Provider:    Ollama (local inference)
   Model:       llama3.2:3b
   Cost:        No per-token charges!
```

Visit the **Frontend URL** to access TracSeq 2.0!

## 💰 Cost Estimation

**Monthly costs (USD):**
- Development: $200-300
- Production: $400-600
- Enterprise: $600-1000+

**💡 Cost Benefit:** Ollama eliminates per-token API charges - you pay only for compute!

*Costs vary by usage, region, and scaling requirements.*

## 🔄 CI/CD Setup (Optional)

### 1. Add GitHub Secrets

The deployment script will provide a service principal JSON. Add it as `AZURE_CREDENTIALS` in your GitHub repository:

1. Go to your GitHub repository
2. Settings → Secrets and variables → Actions
3. Add secret: `AZURE_CREDENTIALS`
4. Paste the JSON from deployment script

### 2. Enable GitHub Actions

The workflow file at `.github/workflows/azure-deploy.yml` will automatically:
- Run tests on pull requests
- Build and deploy on main branch pushes
- Provide deployment summaries

## 🚨 Troubleshooting

### Common Issues

**❌ "Registry name not available"**
- Solution: Choose a different registry name (must be globally unique)

**❌ "Location not available"**
- Solution: Try different Azure region (e.g., `westus2`, `centralus`)

**❌ "Docker build failed"**
- Solution: Ensure Docker Desktop is running and you have internet access

**❌ "Access denied"**
- Solution: Run `az login` again or check Azure subscription access

### Get Help

```bash
# Check deployment status
az containerapp list --resource-group tracseq-rg --output table

# View service logs
az containerapp logs show --name tracseq-frontend --resource-group tracseq-rg --follow

# Check resource health
az resource list --resource-group tracseq-rg --output table
```

## 🎯 Next Steps

### Immediate
1. **Visit your frontend URL** to access the application
2. **Create admin user** through the backend API
3. **Test sample upload** to verify RAG processing

### Configuration
1. **Set up custom domain** (optional)
2. **Configure SSL certificates** 
3. **Set up monitoring alerts**
4. **Review security settings**

### Development
1. **Enable GitHub Actions** for CI/CD
2. **Set up staging environment**
3. **Configure backups**
4. **Monitor costs and usage**

## 📞 Support

- **Azure Issues**: Create support ticket in Azure Portal
- **TracSeq Issues**: [GitHub Issues](https://github.com/your-username/tracseq2.0/issues)
- **Documentation**: [Full Guide](README.md)

---

**🎉 Ready to deploy? Run the script and you'll have TracSeq 2.0 running on Azure in minutes!**

```bash
./deploy/azure/deploy.sh    # Let's go! 🚀
``` 
