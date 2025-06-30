#!/bin/bash
# TracSeq 2.0 Infrastructure Deployment Script
# This script automates the deployment of TracSeq infrastructure

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
ENVIRONMENT="${1:-development}"
ACTION="${2:-deploy}"
REGION="${AWS_REGION:-us-east-1}"
CLUSTER_NAME="tracseq-${ENVIRONMENT}"

# Script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
INFRA_DIR="$(dirname "$SCRIPT_DIR")"

# Function to print colored output
print_message() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Function to check prerequisites
check_prerequisites() {
    print_message "$BLUE" "🔍 Checking prerequisites..."
    
    local missing_tools=()
    
    # Check required tools
    command -v terraform >/dev/null 2>&1 || missing_tools+=("terraform")
    command -v kubectl >/dev/null 2>&1 || missing_tools+=("kubectl")
    command -v helm >/dev/null 2>&1 || missing_tools+=("helm")
    command -v aws >/dev/null 2>&1 || missing_tools+=("aws-cli")
    command -v jq >/dev/null 2>&1 || missing_tools+=("jq")
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        print_message "$RED" "❌ Missing required tools: ${missing_tools[*]}"
        print_message "$YELLOW" "Please install missing tools and try again."
        exit 1
    fi
    
    # Check AWS credentials
    if ! aws sts get-caller-identity >/dev/null 2>&1; then
        print_message "$RED" "❌ AWS credentials not configured"
        print_message "$YELLOW" "Please run 'aws configure' or set AWS credentials"
        exit 1
    fi
    
    print_message "$GREEN" "✅ All prerequisites met"
}

# Function to deploy Terraform infrastructure
deploy_terraform() {
    print_message "$BLUE" "🚀 Deploying Terraform infrastructure for ${ENVIRONMENT}..."
    
    cd "${INFRA_DIR}/terraform/environments/${ENVIRONMENT}"
    
    # Initialize Terraform
    print_message "$YELLOW" "Initializing Terraform..."
    terraform init -upgrade
    
    # Create workspace if it doesn't exist
    terraform workspace select ${ENVIRONMENT} 2>/dev/null || terraform workspace new ${ENVIRONMENT}
    
    # Plan deployment
    print_message "$YELLOW" "Planning Terraform deployment..."
    terraform plan -out=tfplan
    
    # Apply deployment
    print_message "$YELLOW" "Applying Terraform deployment..."
    terraform apply tfplan
    
    # Save outputs
    terraform output -json > "${INFRA_DIR}/terraform/outputs/${ENVIRONMENT}.json"
    
    print_message "$GREEN" "✅ Terraform infrastructure deployed"
}

# Function to configure kubectl
configure_kubectl() {
    print_message "$BLUE" "🔧 Configuring kubectl..."
    
    # Update kubeconfig
    aws eks update-kubeconfig --name "${CLUSTER_NAME}" --region "${REGION}"
    
    # Verify connection
    if kubectl cluster-info >/dev/null 2>&1; then
        print_message "$GREEN" "✅ kubectl configured successfully"
    else
        print_message "$RED" "❌ Failed to connect to cluster"
        exit 1
    fi
}

# Function to deploy Kubernetes resources
deploy_kubernetes() {
    print_message "$BLUE" "🚀 Deploying Kubernetes resources..."
    
    cd "${INFRA_DIR}/kubernetes"
    
    # Create namespaces
    print_message "$YELLOW" "Creating namespaces..."
    kubectl apply -f namespace.yaml
    
    # Install cert-manager
    print_message "$YELLOW" "Installing cert-manager..."
    helm repo add jetstack https://charts.jetstack.io
    helm repo update
    helm upgrade --install cert-manager jetstack/cert-manager \
        --namespace cert-manager \
        --create-namespace \
        --version v1.12.0 \
        --set installCRDs=true \
        --wait
    
    # Install ingress controller
    print_message "$YELLOW" "Installing NGINX ingress controller..."
    helm upgrade --install ingress-nginx ingress-nginx \
        --repo https://kubernetes.github.io/ingress-nginx \
        --namespace ingress-nginx \
        --create-namespace \
        --set controller.service.type=LoadBalancer \
        --wait
    
    # Create secrets
    print_message "$YELLOW" "Creating secrets..."
    kubectl create secret generic tracseq-secrets \
        --namespace "tracseq-${ENVIRONMENT}" \
        --from-literal=database-url="postgres://tracseq:password@postgresql:5432/tracseq" \
        --from-literal=redis-password="changeme" \
        --from-literal=jwt-secret="$(openssl rand -base64 32)" \
        --dry-run=client -o yaml | kubectl apply -f -
    
    print_message "$GREEN" "✅ Kubernetes resources deployed"
}

# Function to deploy TracSeq application
deploy_tracseq() {
    print_message "$BLUE" "🚀 Deploying TracSeq application..."
    
    cd "${INFRA_DIR}/kubernetes/helm"
    
    # Update Helm dependencies
    print_message "$YELLOW" "Updating Helm dependencies..."
    helm dependency update tracseq
    
    # Deploy TracSeq
    print_message "$YELLOW" "Installing TracSeq Helm chart..."
    helm upgrade --install tracseq ./tracseq \
        --namespace "tracseq-${ENVIRONMENT}" \
        --values "tracseq/values.yaml" \
        --values "tracseq/values.${ENVIRONMENT}.yaml" \
        --wait \
        --timeout 15m
    
    print_message "$GREEN" "✅ TracSeq application deployed"
}

# Function to deploy monitoring stack
deploy_monitoring() {
    print_message "$BLUE" "📊 Deploying monitoring stack..."
    
    cd "${INFRA_DIR}/monitoring"
    
    # Add Prometheus Helm repo
    helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
    helm repo update
    
    # Deploy kube-prometheus-stack
    helm upgrade --install monitoring prometheus-community/kube-prometheus-stack \
        --namespace monitoring \
        --create-namespace \
        --values prometheus-values.yaml \
        --wait
    
    print_message "$GREEN" "✅ Monitoring stack deployed"
}

# Function to deploy ArgoCD
deploy_argocd() {
    print_message "$BLUE" "🔄 Deploying ArgoCD..."
    
    # Add ArgoCD Helm repo
    helm repo add argo https://argoproj.github.io/argo-helm
    helm repo update
    
    # Deploy ArgoCD
    helm upgrade --install argocd argo/argo-cd \
        --namespace argocd \
        --create-namespace \
        --set server.service.type=LoadBalancer \
        --wait
    
    # Apply app-of-apps
    kubectl apply -f "${INFRA_DIR}/argocd/applications/tracseq-app-of-apps.yaml"
    
    # Get initial admin password
    local argocd_password=$(kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" | base64 -d)
    print_message "$YELLOW" "ArgoCD admin password: ${argocd_password}"
    
    print_message "$GREEN" "✅ ArgoCD deployed"
}

# Function to verify deployment
verify_deployment() {
    print_message "$BLUE" "🔍 Verifying deployment..."
    
    # Check pods
    print_message "$YELLOW" "Checking pod status..."
    kubectl get pods -n "tracseq-${ENVIRONMENT}"
    
    # Check services
    print_message "$YELLOW" "Checking service endpoints..."
    kubectl get svc -n "tracseq-${ENVIRONMENT}"
    
    # Get ingress URLs
    print_message "$YELLOW" "Getting application URLs..."
    local ingress_ip=$(kubectl get svc -n ingress-nginx ingress-nginx-controller -o jsonpath='{.status.loadBalancer.ingress[0].hostname}')
    
    print_message "$GREEN" "Application URLs:"
    print_message "$GREEN" "  Frontend: http://${ingress_ip}"
    print_message "$GREEN" "  API: http://${ingress_ip}/api"
    print_message "$GREEN" "  Grafana: http://${ingress_ip}/grafana"
    
    print_message "$GREEN" "✅ Deployment verification complete"
}

# Function to destroy infrastructure
destroy_infrastructure() {
    print_message "$RED" "⚠️  WARNING: This will destroy all infrastructure for ${ENVIRONMENT}"
    read -p "Are you sure? (yes/no): " confirm
    
    if [ "$confirm" != "yes" ]; then
        print_message "$YELLOW" "Destroy cancelled"
        exit 0
    fi
    
    # Delete Kubernetes resources first
    print_message "$YELLOW" "Deleting Kubernetes resources..."
    helm uninstall tracseq -n "tracseq-${ENVIRONMENT}" || true
    helm uninstall monitoring -n monitoring || true
    helm uninstall argocd -n argocd || true
    helm uninstall cert-manager -n cert-manager || true
    helm uninstall ingress-nginx -n ingress-nginx || true
    
    # Destroy Terraform infrastructure
    print_message "$YELLOW" "Destroying Terraform infrastructure..."
    cd "${INFRA_DIR}/terraform/environments/${ENVIRONMENT}"
    terraform destroy -auto-approve
    
    print_message "$GREEN" "✅ Infrastructure destroyed"
}

# Main execution
main() {
    print_message "$BLUE" "========================================="
    print_message "$BLUE" "TracSeq 2.0 Infrastructure Deployment"
    print_message "$BLUE" "Environment: ${ENVIRONMENT}"
    print_message "$BLUE" "Action: ${ACTION}"
    print_message "$BLUE" "========================================="
    
    check_prerequisites
    
    case "$ACTION" in
        deploy)
            deploy_terraform
            configure_kubectl
            deploy_kubernetes
            deploy_tracseq
            deploy_monitoring
            deploy_argocd
            verify_deployment
            ;;
        destroy)
            destroy_infrastructure
            ;;
        verify)
            configure_kubectl
            verify_deployment
            ;;
        *)
            print_message "$RED" "Unknown action: ${ACTION}"
            print_message "$YELLOW" "Usage: $0 [environment] [deploy|destroy|verify]"
            exit 1
            ;;
    esac
    
    print_message "$GREEN" "🎉 Operation completed successfully!"
}

# Run main function
main 