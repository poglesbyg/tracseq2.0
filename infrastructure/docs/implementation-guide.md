# TracSeq 2.0 Infrastructure Implementation Guide

This guide provides step-by-step instructions for implementing the TracSeq 2.0 infrastructure from development to production.

## Table of Contents

1. [Overview](#overview)
2. [Local Development](#local-development)
3. [Cloud Infrastructure](#cloud-infrastructure)
4. [Kubernetes Deployment](#kubernetes-deployment)
5. [Monitoring & Observability](#monitoring--observability)
6. [Security Implementation](#security-implementation)
7. [CI/CD Pipeline](#cicd-pipeline)
8. [Disaster Recovery](#disaster-recovery)

## Overview

TracSeq 2.0 uses a microservices architecture deployed on Kubernetes with the following components:

- **Frontend**: React/TypeScript application
- **API Gateway**: FastAPI-based gateway
- **Microservices**: Auth, Sample, Storage, Reports, RAG (AI)
- **Data Layer**: PostgreSQL, Redis, Ollama (LLM)
- **Infrastructure**: Kubernetes, Terraform, ArgoCD

## Local Development

### Prerequisites

Install the following tools:
- Docker Desktop
- kubectl
- helm
- kind or minikube

### Quick Start

```bash
# 1. Set up local Kubernetes
./infrastructure/scripts/setup-local-k8s.sh

# 2. Deploy monitoring stack (optional)
cd infrastructure/monitoring
docker-compose -f docker-compose.monitoring.yml up -d

# 3. Build and deploy services
cd ../..
docker-compose -f docker/docker-compose.yml build
./infrastructure/scripts/deploy.sh development deploy

# 4. Access the application
open http://localhost
```

### Development Workflow

1. **Code Changes**: Make changes to services
2. **Build Images**: `docker-compose build <service>`
3. **Deploy**: `kubectl rollout restart deployment/<service> -n tracseq-dev`
4. **Monitor**: `kubectl logs -f deployment/<service> -n tracseq-dev`

## Cloud Infrastructure

### AWS Setup

1. **Configure AWS CLI**:
```bash
aws configure
# Enter your AWS Access Key ID, Secret Access Key, and region
```

2. **Create S3 Backend** (one-time setup):
```bash
aws s3 mb s3://tracseq-terraform-state
aws dynamodb create-table \
  --table-name tracseq-terraform-locks \
  --attribute-definitions AttributeName=LockID,AttributeType=S \
  --key-schema AttributeName=LockID,KeyType=HASH \
  --provisioned-throughput ReadCapacityUnits=1,WriteCapacityUnits=1
```

3. **Deploy Infrastructure**:
```bash
cd infrastructure/terraform/environments/production
terraform init
terraform plan
terraform apply
```

### Infrastructure Components

#### VPC Configuration
- CIDR: 10.0.0.0/16
- Public Subnets: 10.0.1.0/24, 10.0.2.0/24, 10.0.3.0/24
- Private Subnets: 10.0.101.0/24, 10.0.102.0/24, 10.0.103.0/24
- NAT Gateways in each AZ

#### EKS Cluster
- Kubernetes version: 1.28
- Node groups:
  - General: t3.large (min: 2, max: 10)
  - Compute: c5.2xlarge (min: 1, max: 5)
- Add-ons: EBS CSI, VPC CNI, CoreDNS

#### RDS Aurora PostgreSQL
- Engine: Aurora PostgreSQL 15.4
- Instance class: db.r6g.large
- Multi-AZ deployment
- Automated backups: 30 days

#### ElastiCache Redis
- Engine: Redis 7.0
- Node type: cache.r6g.large
- Cluster mode: enabled (3 nodes)
- Encryption at rest and in transit

## Kubernetes Deployment

### Namespace Strategy

```yaml
Production: tracseq
Staging: tracseq-staging
Development: tracseq-dev
```

### Deploy Application

```bash
# 1. Configure kubectl
aws eks update-kubeconfig --name tracseq-production --region us-east-1

# 2. Create namespace and secrets
kubectl create namespace tracseq
kubectl create secret generic tracseq-secrets \
  --namespace tracseq \
  --from-literal=database-url="postgres://..." \
  --from-literal=redis-password="..." \
  --from-literal=jwt-secret="..."

# 3. Deploy with Helm
cd infrastructure/kubernetes/helm
helm dependency update tracseq
helm install tracseq ./tracseq \
  --namespace tracseq \
  --values tracseq/values.yaml \
  --values tracseq/values.production.yaml
```

### Service Mesh (Optional)

Deploy Istio for advanced traffic management:

```bash
# Install Istio
curl -L https://istio.io/downloadIstio | sh -
cd istio-*
export PATH=$PWD/bin:$PATH
istioctl install --set profile=demo -y

# Enable sidecar injection
kubectl label namespace tracseq istio-injection=enabled
```

## Monitoring & Observability

### Prometheus & Grafana

```bash
# Deploy monitoring stack
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm install monitoring prometheus-community/kube-prometheus-stack \
  --namespace monitoring \
  --create-namespace \
  --values infrastructure/monitoring/prometheus-values.yaml
```

### Access Dashboards

```bash
# Port-forward Grafana
kubectl port-forward -n monitoring svc/monitoring-grafana 3000:80

# Default credentials: admin / prom-operator
open http://localhost:3000
```

### Configure Alerts

Create alert rules in `infrastructure/monitoring/prometheus/rules/`:

```yaml
groups:
  - name: tracseq
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: High error rate detected
```

## Security Implementation

### Network Policies

```yaml
# Deny all ingress by default
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-ingress
  namespace: tracseq
spec:
  podSelector: {}
  policyTypes:
  - Ingress
```

### Pod Security Standards

```yaml
# Enforce restricted security standard
kubectl label namespace tracseq \
  pod-security.kubernetes.io/enforce=restricted \
  pod-security.kubernetes.io/audit=restricted \
  pod-security.kubernetes.io/warn=restricted
```

### Secrets Management

Use Sealed Secrets for GitOps:

```bash
# Install Sealed Secrets controller
kubectl apply -f https://github.com/bitnami-labs/sealed-secrets/releases/download/v0.18.0/controller.yaml

# Create sealed secret
echo -n mypassword | kubectl create secret generic mysecret \
  --dry-run=client \
  --from-file=password=/dev/stdin \
  -o yaml | kubeseal -o yaml > mysealedsecret.yaml
```

## CI/CD Pipeline

### GitHub Actions Workflow

Create `.github/workflows/deploy.yml`:

```yaml
name: Deploy to Production

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v2
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: us-east-1
      
      - name: Build and push images
        run: |
          aws ecr get-login-password | docker login --username AWS --password-stdin $ECR_REGISTRY
          docker build -t $ECR_REGISTRY/tracseq/frontend:$GITHUB_SHA .
          docker push $ECR_REGISTRY/tracseq/frontend:$GITHUB_SHA
      
      - name: Deploy to Kubernetes
        run: |
          aws eks update-kubeconfig --name tracseq-production
          helm upgrade tracseq ./infrastructure/kubernetes/helm/tracseq \
            --namespace tracseq \
            --set frontend.image.tag=$GITHUB_SHA
```

### ArgoCD Setup

```bash
# Install ArgoCD
kubectl create namespace argocd
kubectl apply -n argocd -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml

# Apply app-of-apps pattern
kubectl apply -f infrastructure/argocd/applications/tracseq-app-of-apps.yaml

# Access ArgoCD UI
kubectl port-forward svc/argocd-server -n argocd 8080:443
```

## Disaster Recovery

### Backup Strategy

1. **Database Backups**:
   - Automated RDS snapshots (daily)
   - Point-in-time recovery enabled
   - Cross-region snapshot copies

2. **Application Data**:
   - S3 bucket versioning enabled
   - Cross-region replication
   - Lifecycle policies for cost optimization

3. **Kubernetes Resources**:
   ```bash
   # Backup all resources
   kubectl get all --all-namespaces -o yaml > k8s-backup.yaml
   
   # Backup with Velero
   velero backup create tracseq-backup --include-namespaces tracseq
   ```

### Recovery Procedures

1. **Database Recovery**:
   ```bash
   # Restore from snapshot
   aws rds restore-db-cluster-from-snapshot \
     --db-cluster-identifier tracseq-restore \
     --snapshot-identifier <snapshot-id>
   ```

2. **Application Recovery**:
   ```bash
   # Restore Kubernetes resources
   kubectl apply -f k8s-backup.yaml
   
   # Or restore with Velero
   velero restore create --from-backup tracseq-backup
   ```

### Testing Recovery

Run recovery drills quarterly:

1. Create test environment
2. Simulate failure scenarios
3. Execute recovery procedures
4. Validate data integrity
5. Document lessons learned

## Troubleshooting

### Common Issues

1. **Pods not starting**:
   ```bash
   kubectl describe pod <pod-name> -n tracseq
   kubectl logs <pod-name> -n tracseq --previous
   ```

2. **Database connection issues**:
   ```bash
   # Check secret
   kubectl get secret tracseq-secrets -n tracseq -o yaml
   
   # Test connection
   kubectl run -it --rm debug --image=postgres:15 --restart=Never -- psql <connection-string>
   ```

3. **Service mesh issues**:
   ```bash
   istioctl analyze -n tracseq
   istioctl proxy-config cluster <pod-name> -n tracseq
   ```

### Performance Tuning

1. **Enable HPA**:
   ```yaml
   apiVersion: autoscaling/v2
   kind: HorizontalPodAutoscaler
   metadata:
     name: api-gateway-hpa
   spec:
     scaleTargetRef:
       apiVersion: apps/v1
       kind: Deployment
       name: api-gateway
     minReplicas: 2
     maxReplicas: 10
     metrics:
     - type: Resource
       resource:
         name: cpu
         target:
           type: Utilization
           averageUtilization: 70
   ```

2. **Database optimization**:
   - Enable query performance insights
   - Adjust connection pooling
   - Implement read replicas

3. **Caching strategy**:
   - Redis for session management
   - CDN for static assets
   - Application-level caching

## Next Steps

1. **Implement GitOps**: Set up ArgoCD for continuous deployment
2. **Add Service Mesh**: Deploy Istio for advanced traffic management
3. **Enhance Security**: Implement OPA policies and admission webhooks
4. **Cost Optimization**: Set up Kubecost and implement autoscaling
5. **Multi-Region**: Expand to multiple regions for high availability

For questions or support, contact the infrastructure team. 