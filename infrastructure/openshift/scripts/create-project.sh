#!/bin/bash
set -euo pipefail

# TracSeq 2.0 OpenShift Project Creation Script

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
ENVIRONMENT=${1:-prod}
PROJECT_NAME="tracseq-${ENVIRONMENT}"
DISPLAY_NAME="TracSeq ${ENVIRONMENT^}"

echo -e "${BLUE}=== TracSeq 2.0 OpenShift Project Setup ===${NC}"
echo -e "${BLUE}Environment: ${ENVIRONMENT}${NC}"

# Check if logged in to OpenShift
if ! oc whoami &> /dev/null; then
    echo -e "${RED}Error: Not logged in to OpenShift${NC}"
    echo "Please run: oc login <cluster-url>"
    exit 1
fi

# Create project if it doesn't exist
if oc get project ${PROJECT_NAME} &> /dev/null; then
    echo -e "${YELLOW}Project ${PROJECT_NAME} already exists${NC}"
else
    echo -e "${GREEN}Creating project ${PROJECT_NAME}...${NC}"
    oc new-project ${PROJECT_NAME} \
        --display-name="${DISPLAY_NAME}" \
        --description="TracSeq 2.0 Laboratory Management System - ${ENVIRONMENT}"
fi

# Switch to the project
oc project ${PROJECT_NAME}

# Create pull secret for Red Hat registry
echo -e "${GREEN}Creating pull secrets...${NC}"
if ! oc get secret redhat-pull-secret &> /dev/null; then
    echo -e "${YELLOW}Please create Red Hat registry pull secret:${NC}"
    echo "1. Get your pull secret from: https://access.redhat.com/terms-based-registry/"
    echo "2. Save it to a file named 'pull-secret.json'"
    echo "3. Run: oc create secret generic redhat-pull-secret --from-file=.dockerconfigjson=pull-secret.json --type=kubernetes.io/dockerconfigjson"
    echo ""
fi

# Install OpenShift Pipelines Operator
echo -e "${GREEN}Checking OpenShift Pipelines Operator...${NC}"
if ! oc get csv -n openshift-operators | grep openshift-pipelines-operator &> /dev/null; then
    echo -e "${YELLOW}OpenShift Pipelines Operator not installed${NC}"
    echo "Please install it from the OperatorHub in the OpenShift Console"
fi

# Create service accounts and permissions
echo -e "${GREEN}Setting up RBAC...${NC}"
oc apply -f ../base/rbac.yaml || true

# Grant necessary permissions
echo -e "${GREEN}Granting permissions...${NC}"
oc adm policy add-scc-to-user anyuid -z default || true
oc adm policy add-scc-to-user tracseq-scc -z tracseq-auth-service || true
oc adm policy add-scc-to-user tracseq-scc -z tracseq-sample-service || true
oc adm policy add-scc-to-user tracseq-scc -z tracseq-template-service || true
oc adm policy add-scc-to-user tracseq-scc -z tracseq-notification-service || true
oc adm policy add-scc-to-user tracseq-scc -z tracseq-sequencing-service || true
oc adm policy add-scc-to-user tracseq-scc -z tracseq-transaction-service || true
oc adm policy add-scc-to-user tracseq-scc -z tracseq-api-gateway || true
oc adm policy add-scc-to-user tracseq-scc -z tracseq-rag-service || true

# Create namespace labels for network policies
echo -e "${GREEN}Setting up network policies...${NC}"
oc label namespace ${PROJECT_NAME} name=${PROJECT_NAME} --overwrite

# Create ConfigMaps and Secrets templates
echo -e "${GREEN}Creating configuration templates...${NC}"
cat > /tmp/secrets-patch.yaml <<EOF
data:
  POSTGRES_PASSWORD: $(openssl rand -base64 32 | base64)
  REDIS_PASSWORD: $(openssl rand -base64 32 | base64)
  JWT_SECRET_KEY: $(openssl rand -base64 32 | base64)
EOF

echo -e "${YELLOW}Generated random passwords for secrets${NC}"
echo -e "${YELLOW}Please update these with your actual values before deploying${NC}"

# Summary
echo -e "${GREEN}=== Project Setup Complete ===${NC}"
echo -e "Project: ${PROJECT_NAME}"
echo -e "Next steps:"
echo -e "1. Update secrets in ../base/secrets/ with actual values"
echo -e "2. Configure your git repository URL in buildconfigs"
echo -e "3. Run ./deploy.sh ${ENVIRONMENT} to deploy the platform"