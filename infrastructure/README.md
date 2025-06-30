# TracSeq 2.0 Infrastructure

This directory contains all infrastructure as code (IaC) and deployment configurations for TracSeq 2.0 Laboratory Management System.

## 📁 Directory Structure

```
infrastructure/
├── terraform/              # Cloud infrastructure (AWS/Azure/GCP)
├── kubernetes/             # Kubernetes manifests and Helm charts  
├── monitoring/             # Observability stack (Prometheus, Grafana, etc.)
├── service-mesh/           # Service mesh configuration (Istio/Linkerd)
├── argocd/                 # GitOps continuous deployment
├── openshift/              # OpenShift-specific deployments
├── scripts/                # Deployment automation scripts
└── docs/                   # Infrastructure documentation
```

## 🎯 Quick Start

### Prerequisites
- Terraform >= 1.6.0
- kubectl >= 1.28
- Helm >= 3.12
- AWS CLI configured (for AWS deployment)
- Docker >= 24.0

### Deploy to AWS (Production)
```bash
# 1. Initialize Terraform
cd terraform/environments/production
terraform init

# 2. Plan infrastructure
terraform plan -out=tfplan

# 3. Apply infrastructure
terraform apply tfplan

# 4. Configure kubectl
aws eks update-kubeconfig --name tracseq-production --region us-east-1

# 5. Deploy applications
cd ../../../kubernetes
./deploy.sh production
```

### Deploy Locally (Development)
```bash
# 1. Start local Kubernetes (kind/minikube)
./scripts/setup-local-k8s.sh

# 2. Deploy monitoring stack
cd monitoring
docker-compose -f docker-compose.monitoring.yml up -d

# 3. Deploy applications
cd ../kubernetes
./deploy.sh development
```

## 🏗️ Infrastructure Components

### Cloud Infrastructure (Terraform)
- **Compute**: EKS cluster with autoscaling node groups
- **Database**: RDS Aurora PostgreSQL (Multi-AZ)
- **Cache**: ElastiCache Redis cluster
- **Storage**: S3 buckets for data and backups
- **Networking**: VPC with public/private subnets
- **Security**: IAM roles, security groups, KMS encryption

### Kubernetes Platform
- **Ingress**: NGINX Ingress Controller with TLS
- **Service Mesh**: Istio for traffic management
- **Secrets**: Sealed Secrets for secret management
- **Storage**: EBS CSI driver for persistent volumes
- **Autoscaling**: HPA and VPA configured

### Observability Stack
- **Metrics**: Prometheus + Grafana
- **Logging**: Loki + Promtail
- **Tracing**: Jaeger
- **Monitoring**: Uptime Kuma
- **Alerting**: AlertManager

### CI/CD Pipeline
- **GitOps**: ArgoCD for continuous deployment
- **Container Registry**: ECR for Docker images
- **Build Pipeline**: GitHub Actions / Jenkins
- **Security Scanning**: Trivy, SonarQube

## 🔧 Configuration

### Environment Variables
Each environment has its own configuration in `terraform/environments/`:
- `development/` - Local development
- `staging/` - Staging environment
- `production/` - Production environment

### Secrets Management
Secrets are managed using:
- AWS Secrets Manager (for RDS passwords)
- Kubernetes Secrets (for application secrets)
- Sealed Secrets (for GitOps)

## 📊 Monitoring & Alerts

### Dashboards
Access Grafana dashboards:
- Development: http://localhost:3001
- Production: https://monitoring.tracseq.com

Default credentials: `admin` / `tracseq-admin`

### Alert Rules
- CPU usage > 80%
- Memory usage > 85%
- Database connections > 90%
- API response time > 2s
- Error rate > 5%

## 🔐 Security

### Network Policies
- Service-to-service communication restricted
- Ingress only through API Gateway
- Egress restricted to required services

### Pod Security
- Non-root containers
- Read-only root filesystem
- Security contexts enforced

### Compliance
- HIPAA compliance for healthcare data
- SOC 2 Type II controls
- Regular security audits

## 🚨 Disaster Recovery

### Backup Strategy
- RDS automated backups (30-day retention)
- S3 cross-region replication
- Kubernetes etcd backups daily
- Application data exports

### Recovery Procedures
See [Disaster Recovery Guide](docs/disaster-recovery.md)

## 📝 Maintenance

### Upgrade Procedures
1. Test in staging environment
2. Create backup of production
3. Apply rolling updates
4. Verify health checks
5. Monitor for 24 hours

### Troubleshooting
See [Troubleshooting Guide](docs/troubleshooting.md)

## 📞 Support

- **Slack**: #tracseq-infrastructure
- **Email**: infrastructure@tracseq.com
- **On-Call**: See PagerDuty rotation 