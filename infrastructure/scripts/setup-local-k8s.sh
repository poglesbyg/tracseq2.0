#!/bin/bash
# TracSeq 2.0 Local Kubernetes Setup Script
# Sets up a local Kubernetes environment using kind or minikube

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CLUSTER_NAME="tracseq-local"
K8S_VERSION="v1.28.0"
KUBERNETES_PROVIDER="${KUBERNETES_PROVIDER:-kind}"

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
    
    # Check Docker
    if ! command -v docker >/dev/null 2>&1; then
        missing_tools+=("docker")
    else
        # Check if Docker is running
        if ! docker info >/dev/null 2>&1; then
            print_message "$RED" "❌ Docker is not running"
            print_message "$YELLOW" "Please start Docker and try again"
            exit 1
        fi
    fi
    
    # Check kubectl
    command -v kubectl >/dev/null 2>&1 || missing_tools+=("kubectl")
    command -v helm >/dev/null 2>&1 || missing_tools+=("helm")
    
    # Check Kubernetes provider
    if [ "$KUBERNETES_PROVIDER" == "kind" ]; then
        command -v kind >/dev/null 2>&1 || missing_tools+=("kind")
    elif [ "$KUBERNETES_PROVIDER" == "minikube" ]; then
        command -v minikube >/dev/null 2>&1 || missing_tools+=("minikube")
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        print_message "$RED" "❌ Missing required tools: ${missing_tools[*]}"
        print_message "$YELLOW" "Installation instructions:"
        for tool in "${missing_tools[@]}"; do
            case $tool in
                docker)
                    print_message "$YELLOW" "  Docker: https://docs.docker.com/get-docker/"
                    ;;
                kubectl)
                    print_message "$YELLOW" "  kubectl: https://kubernetes.io/docs/tasks/tools/"
                    ;;
                helm)
                    print_message "$YELLOW" "  helm: https://helm.sh/docs/intro/install/"
                    ;;
                kind)
                    print_message "$YELLOW" "  kind: https://kind.sigs.k8s.io/docs/user/quick-start/#installation"
                    ;;
                minikube)
                    print_message "$YELLOW" "  minikube: https://minikube.sigs.k8s.io/docs/start/"
                    ;;
            esac
        done
        exit 1
    fi
    
    print_message "$GREEN" "✅ All prerequisites met"
}

# Function to create kind cluster
create_kind_cluster() {
    print_message "$BLUE" "🚀 Creating kind cluster..."
    
    # Check if cluster already exists
    if kind get clusters | grep -q "^${CLUSTER_NAME}$"; then
        print_message "$YELLOW" "Cluster ${CLUSTER_NAME} already exists. Deleting..."
        kind delete cluster --name "${CLUSTER_NAME}"
    fi
    
    # Create kind config
    cat > /tmp/kind-config.yaml <<EOF
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
name: ${CLUSTER_NAME}
nodes:
  - role: control-plane
    kubeadmConfigPatches:
      - |
        kind: InitConfiguration
        nodeRegistration:
          kubeletExtraArgs:
            node-labels: "ingress-ready=true"
    extraPortMappings:
      - containerPort: 80
        hostPort: 80
        protocol: TCP
      - containerPort: 443
        hostPort: 443
        protocol: TCP
  - role: worker
  - role: worker
EOF
    
    # Create cluster
    kind create cluster --config /tmp/kind-config.yaml --image kindest/node:${K8S_VERSION}
    
    # Set kubectl context
    kubectl cluster-info --context kind-${CLUSTER_NAME}
    
    print_message "$GREEN" "✅ Kind cluster created"
}

# Function to create minikube cluster
create_minikube_cluster() {
    print_message "$BLUE" "🚀 Creating minikube cluster..."
    
    # Check if cluster already exists
    if minikube status -p "${CLUSTER_NAME}" >/dev/null 2>&1; then
        print_message "$YELLOW" "Cluster ${CLUSTER_NAME} already exists. Deleting..."
        minikube delete -p "${CLUSTER_NAME}"
    fi
    
    # Start minikube
    minikube start \
        -p "${CLUSTER_NAME}" \
        --kubernetes-version="${K8S_VERSION}" \
        --cpus=4 \
        --memory=8192 \
        --disk-size=50g \
        --driver=docker
    
    # Enable necessary addons
    minikube addons enable ingress -p "${CLUSTER_NAME}"
    minikube addons enable metrics-server -p "${CLUSTER_NAME}"
    minikube addons enable dashboard -p "${CLUSTER_NAME}"
    
    print_message "$GREEN" "✅ Minikube cluster created"
}

# Function to install local storage
install_local_storage() {
    print_message "$BLUE" "💾 Installing local storage..."
    
    # Install local-path-provisioner for kind
    if [ "$KUBERNETES_PROVIDER" == "kind" ]; then
        kubectl apply -f https://raw.githubusercontent.com/rancher/local-path-provisioner/v0.0.24/deploy/local-path-storage.yaml
        
        # Set as default storage class
        kubectl patch storageclass local-path -p '{"metadata": {"annotations":{"storageclass.kubernetes.io/is-default-class":"true"}}}'
    fi
    
    print_message "$GREEN" "✅ Local storage installed"
}

# Function to install MetalLB (for kind)
install_metallb() {
    if [ "$KUBERNETES_PROVIDER" != "kind" ]; then
        return
    fi
    
    print_message "$BLUE" "🔧 Installing MetalLB for LoadBalancer support..."
    
    # Install MetalLB
    kubectl apply -f https://raw.githubusercontent.com/metallb/metallb/v0.13.10/config/manifests/metallb-native.yaml
    
    # Wait for MetalLB to be ready
    kubectl wait --namespace metallb-system \
        --for=condition=ready pod \
        --selector=app=metallb \
        --timeout=90s
    
    # Configure IP address pool
    # Use a static IP range that works with most kind clusters
    cat <<EOF | kubectl apply -f -
apiVersion: metallb.io/v1beta1
kind: IPAddressPool
metadata:
  name: example
  namespace: metallb-system
spec:
  addresses:
  - 172.18.255.200-172.18.255.250
---
apiVersion: metallb.io/v1beta1
kind: L2Advertisement
metadata:
  name: empty
  namespace: metallb-system
EOF
    
    print_message "$GREEN" "✅ MetalLB installed"
}

# Function to install ingress controller
install_ingress() {
    print_message "$BLUE" "🌐 Installing NGINX ingress controller..."
    
    if [ "$KUBERNETES_PROVIDER" == "kind" ]; then
        # Install NGINX ingress for kind
        kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/main/deploy/static/provider/kind/deploy.yaml
        
        # Wait for ingress to be ready
        kubectl wait --namespace ingress-nginx \
            --for=condition=ready pod \
            --selector=app.kubernetes.io/component=controller \
            --timeout=90s
    fi
    
    print_message "$GREEN" "✅ Ingress controller installed"
}

# Function to create local environment file
create_local_env() {
    print_message "$BLUE" "📝 Creating local environment configuration..."
    
    cat > "${INFRA_DIR}/kubernetes/helm/tracseq/values.development.yaml" <<EOF
# Local development values for TracSeq
global:
  environment: development
  domain: localhost
  
frontend:
  replicaCount: 1
  resources:
    limits:
      cpu: 200m
      memory: 256Mi
    requests:
      cpu: 50m
      memory: 64Mi
  ingress:
    hosts:
      - host: localhost
        paths:
          - path: /
            pathType: Prefix
    tls: []

apiGateway:
  replicaCount: 1
  resources:
    limits:
      cpu: 500m
      memory: 512Mi
    requests:
      cpu: 100m
      memory: 128Mi
  ingress:
    hosts:
      - host: localhost
        paths:
          - path: /api
            pathType: Prefix
    tls: []

services:
  authService:
    replicaCount: 1
  sampleService:
    replicaCount: 1
  storageService:
    replicaCount: 1
  ragService:
    replicaCount: 1
  reportsService:
    replicaCount: 1

postgresql:
  persistence:
    size: 10Gi
  
redis:
  master:
    persistence:
      size: 1Gi
  replica:
    replicaCount: 0

ollama:
  persistence:
    size: 20Gi
  resources:
    limits:
      cpu: 2000m
      memory: 4Gi
    requests:
      cpu: 500m
      memory: 1Gi

monitoring:
  enabled: false
  
backup:
  enabled: false
EOF
    
    print_message "$GREEN" "✅ Local environment configuration created"
}

# Function to display next steps
show_next_steps() {
    print_message "$GREEN" "========================================="
    print_message "$GREEN" "✅ Local Kubernetes environment ready!"
    print_message "$GREEN" "========================================="
    
    print_message "$BLUE" "Next steps:"
    print_message "$YELLOW" "1. Deploy TracSeq to local cluster:"
    print_message "$YELLOW" "   cd ${INFRA_DIR}/kubernetes"
    print_message "$YELLOW" "   helm install tracseq ./helm/tracseq -f helm/tracseq/values.yaml -f helm/tracseq/values.development.yaml"
    
    print_message "$YELLOW" "\n2. Access the application:"
    print_message "$YELLOW" "   Frontend: http://localhost"
    print_message "$YELLOW" "   API: http://localhost/api"
    
    if [ "$KUBERNETES_PROVIDER" == "minikube" ]; then
        print_message "$YELLOW" "\n3. Access Kubernetes dashboard:"
        print_message "$YELLOW" "   minikube dashboard -p ${CLUSTER_NAME}"
    fi
    
    print_message "$YELLOW" "\n4. Monitor pods:"
    print_message "$YELLOW" "   kubectl get pods -n tracseq-dev -w"
    
    print_message "$YELLOW" "\n5. View logs:"
    print_message "$YELLOW" "   kubectl logs -n tracseq-dev -l app.kubernetes.io/name=tracseq -f"
    
    print_message "$YELLOW" "\n6. Clean up when done:"
    if [ "$KUBERNETES_PROVIDER" == "kind" ]; then
        print_message "$YELLOW" "   kind delete cluster --name ${CLUSTER_NAME}"
    else
        print_message "$YELLOW" "   minikube delete -p ${CLUSTER_NAME}"
    fi
}

# Main execution
main() {
    print_message "$BLUE" "========================================="
    print_message "$BLUE" "TracSeq 2.0 Local Kubernetes Setup"
    print_message "$BLUE" "Provider: ${KUBERNETES_PROVIDER}"
    print_message "$BLUE" "========================================="
    
    check_prerequisites
    
    # Create cluster based on provider
    if [ "$KUBERNETES_PROVIDER" == "kind" ]; then
        create_kind_cluster
        install_metallb
    else
        create_minikube_cluster
    fi
    
    # Install common components
    install_local_storage
    install_ingress
    create_local_env
    
    # Show next steps
    show_next_steps
}

# Run main function
main 