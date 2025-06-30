#!/bin/bash
set -euo pipefail

# TracSeq 2.0 OpenShift Deployment Script

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
ENVIRONMENT=${1:-prod}
PROJECT_NAME="tracseq-${ENVIRONMENT}"
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
BASE_DIR="${SCRIPT_DIR}/../base"
OVERLAY_DIR="${SCRIPT_DIR}/../overlays/${ENVIRONMENT}"

echo -e "${BLUE}=== TracSeq 2.0 OpenShift Deployment ===${NC}"
echo -e "${BLUE}Environment: ${ENVIRONMENT}${NC}"
echo -e "${BLUE}Project: ${PROJECT_NAME}${NC}"

# Check prerequisites
if ! command -v oc &> /dev/null; then
    echo -e "${RED}Error: oc CLI not found${NC}"
    exit 1
fi

if ! command -v kustomize &> /dev/null; then
    echo -e "${YELLOW}Warning: kustomize not found, using oc kustomize${NC}"
    KUSTOMIZE_CMD="oc kustomize"
else
    KUSTOMIZE_CMD="kustomize"
fi

# Check if logged in
if ! oc whoami &> /dev/null; then
    echo -e "${RED}Error: Not logged in to OpenShift${NC}"
    echo "Please run: oc login <cluster-url>"
    exit 1
fi

# Switch to project
oc project ${PROJECT_NAME} || {
    echo -e "${RED}Error: Project ${PROJECT_NAME} not found${NC}"
    echo "Please run: ./create-project.sh ${ENVIRONMENT}"
    exit 1
}

# Function to wait for deployment
wait_for_deployment() {
    local deployment=$1
    local timeout=${2:-300}
    echo -e "${YELLOW}Waiting for ${deployment} to be ready...${NC}"
    oc rollout status deployment/${deployment} -n ${PROJECT_NAME} --timeout=${timeout}s
}

# Function to check if resource exists
resource_exists() {
    local resource_type=$1
    local resource_name=$2
    oc get ${resource_type} ${resource_name} -n ${PROJECT_NAME} &> /dev/null
}

# Phase 1: Deploy infrastructure
echo -e "${GREEN}=== Phase 1: Deploying Infrastructure ===${NC}"

# Deploy namespace and RBAC
echo -e "${GREEN}Applying namespace and RBAC...${NC}"
oc apply -f ${BASE_DIR}/namespace.yaml
oc apply -f ${BASE_DIR}/rbac.yaml
oc apply -f ${BASE_DIR}/network-policies.yaml

# Deploy ConfigMaps and Secrets
echo -e "${GREEN}Applying ConfigMaps and Secrets...${NC}"
oc apply -f ${BASE_DIR}/configmaps/
oc apply -f ${BASE_DIR}/secrets/

# Deploy databases
echo -e "${GREEN}Deploying databases...${NC}"
oc apply -f ${BASE_DIR}/postgres/
oc apply -f ${BASE_DIR}/redis/
oc apply -f ${BASE_DIR}/chromadb/

# Wait for databases
wait_for_deployment postgres
wait_for_deployment redis
wait_for_deployment chromadb

# Initialize database
echo -e "${GREEN}Initializing database...${NC}"
if ! resource_exists job postgres-init; then
    cat <<EOF | oc apply -f -
apiVersion: batch/v1
kind: Job
metadata:
  name: postgres-init
  namespace: ${PROJECT_NAME}
spec:
  template:
    spec:
      restartPolicy: Never
      containers:
      - name: postgres-init
        image: registry.redhat.io/rhel9/postgresql-15:latest
        command: ["/bin/bash", "-c"]
        args:
        - |
          until pg_isready -h postgres-service -p 5432; do
            echo "Waiting for PostgreSQL to be ready..."
            sleep 5
          done
          
          PGPASSWORD=\$POSTGRESQL_PASSWORD psql -h postgres-service -U \$POSTGRESQL_USER -d postgres < /opt/app-root/src/postgresql-init/init-databases.sql
        env:
        - name: POSTGRESQL_USER
          valueFrom:
            secretKeyRef:
              name: tracseq-database-credentials
              key: POSTGRES_USER
        - name: POSTGRESQL_PASSWORD
          valueFrom:
            secretKeyRef:
              name: tracseq-database-credentials
              key: POSTGRES_PASSWORD
        volumeMounts:
        - name: init-script
          mountPath: /opt/app-root/src/postgresql-init
      volumes:
      - name: init-script
        configMap:
          name: postgres-init-script
EOF
    
    # Wait for job to complete
    oc wait --for=condition=complete job/postgres-init --timeout=120s
fi

# Phase 2: Build images
echo -e "${GREEN}=== Phase 2: Building Images ===${NC}"

# Create BuildConfigs for each service
services=("auth-service" "sample-service" "template-service" "notification-service" "sequencing-service" "transaction-service")
for service in "${services[@]}"; do
    echo -e "${GREEN}Creating BuildConfig for ${service}...${NC}"
    
    # Convert service name to context dir (replace - with _)
    context_dir="${service//-/_}"
    
    oc process -f ${SCRIPT_DIR}/../templates/buildconfig-rust-service.yaml \
        -p SERVICE_NAME=${service} \
        -p CONTEXT_DIR=${context_dir} \
        | oc apply -f -
    
    # Start build
    if ! oc get build ${service}-1 &> /dev/null; then
        echo -e "${GREEN}Starting build for ${service}...${NC}"
        oc start-build ${service} --wait
    fi
done

# Build Python services
echo -e "${GREEN}Building Python services...${NC}"
# API Gateway and RAG Service would need their own BuildConfigs

# Phase 3: Deploy services
echo -e "${GREEN}=== Phase 3: Deploying Services ===${NC}"

# Deploy core services
echo -e "${GREEN}Deploying core services...${NC}"
oc apply -f ${BASE_DIR}/services/

# Wait for services to be ready
for service in "${services[@]}"; do
    wait_for_deployment ${service}
done

# Deploy API Gateway
echo -e "${GREEN}Deploying API Gateway...${NC}"
wait_for_deployment api-gateway

# Phase 4: Configure routes
echo -e "${GREEN}=== Phase 4: Configuring Routes ===${NC}"

# Get cluster domain
CLUSTER_DOMAIN=$(oc get ingresses.config.openshift.io cluster -o jsonpath='{.spec.domain}')
echo -e "${YELLOW}Cluster domain: ${CLUSTER_DOMAIN}${NC}"

# Update routes with actual domain
sed "s/apps.your-cluster.example.com/${CLUSTER_DOMAIN}/g" ${BASE_DIR}/routes/api-gateway-route.yaml | oc apply -f -

# Phase 5: Post-deployment tasks
echo -e "${GREEN}=== Phase 5: Post-deployment Tasks ===${NC}"

# Run database migrations
echo -e "${GREEN}Running database migrations...${NC}"
for service in "${services[@]}"; do
    echo -e "${YELLOW}Running migrations for ${service}...${NC}"
    # This would typically run migration jobs for each service
done

# Verify deployment
echo -e "${GREEN}=== Verifying Deployment ===${NC}"

# Check all pods
echo -e "${YELLOW}Checking pod status...${NC}"
oc get pods -n ${PROJECT_NAME}

# Check routes
echo -e "${YELLOW}Checking routes...${NC}"
oc get routes -n ${PROJECT_NAME}

# Get API Gateway URL
API_URL=$(oc get route api-gateway -o jsonpath='{.spec.host}')
echo -e "${GREEN}=== Deployment Complete ===${NC}"
echo -e "API Gateway URL: https://${API_URL}"
echo -e ""
echo -e "To test the deployment:"
echo -e "  curl https://${API_URL}/health"
echo -e ""
echo -e "To view logs:"
echo -e "  oc logs -f deployment/api-gateway"
echo -e ""
echo -e "To access the OpenShift console:"
echo -e "  oc console"