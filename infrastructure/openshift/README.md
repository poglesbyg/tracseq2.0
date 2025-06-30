# TracSeq 2.0 OpenShift Deployment

This directory contains all the necessary artifacts for deploying TracSeq 2.0 on OpenShift.

## Directory Structure

```
openshift/
├── base/                    # Base configurations (kustomization)
├── overlays/               # Environment-specific configurations
│   ├── dev/               # Development environment
│   ├── staging/           # Staging environment
│   └── prod/              # Production environment
├── templates/              # OpenShift templates for services
├── pipelines/              # Tekton pipeline definitions
└── scripts/                # Deployment and utility scripts
```

## Prerequisites

- OpenShift 4.12+ cluster
- oc CLI tool installed and configured
- Cluster admin access for initial setup
- Container registry access

## Quick Start

1. Login to OpenShift:
   ```bash
   oc login <your-openshift-cluster>
   ```

2. Create the project:
   ```bash
   ./scripts/create-project.sh prod
   ```

3. Deploy the platform:
   ```bash
   ./scripts/deploy.sh prod
   ```

## Components

- **Core Services**: Auth, Sample, Template, Notification, Sequencing, Transaction
- **Data Layer**: PostgreSQL, Redis, ChromaDB
- **API Gateway**: FastAPI-based gateway service
- **AI Services**: Enhanced RAG service with LLM integration
- **Monitoring**: Prometheus, Grafana, Jaeger integration

## Security

- All services run with restricted SCCs
- Network policies for service isolation
- Secrets managed via OpenShift secrets
- RBAC configured per service