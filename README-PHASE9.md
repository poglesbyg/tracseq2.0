# TracSeq 2.0 - Phase 9: DevOps & CI/CD Excellence

## Quick Start

### Prerequisites
- AWS Account with appropriate permissions
- Tools installed: terraform, kubectl, helm, aws-cli, docker, git, jq
- GitHub repository with Actions enabled
- Domain name for the application

### Deployment

1. **Clone the repository**
   ```bash
   git clone https://github.com/tracseq/tracseq-2.0.git
   cd tracseq-2.0
   ```

2. **Set environment variables**
   ```bash
   export ENVIRONMENT=development
   export AWS_REGION=us-east-1
   export GITHUB_ORG=tracseq
   export GITHUB_REPO=tracseq-2.0
   ```

3. **Run the deployment script**
   ```bash
   ./deploy-phase9.sh
   ```

   This will:
   - Set up Terraform backend
   - Deploy AWS infrastructure
   - Install ArgoCD
   - Deploy all microservices
   - Set up monitoring

4. **Access the services**
   - ArgoCD: `kubectl get svc -n argocd argocd-server`
   - Grafana: `kubectl get svc -n monitoring prometheus-grafana`
   - API Gateway: `kubectl get svc -n tracseq api-gateway`

### Running Tests

**Performance Tests**
```bash
# Load test
k6 run testing/performance/load-test.js

# Stress test
artillery run testing/performance/stress-test.yml
```

**Contract Tests**
```bash
cd testing/contract
cargo test --features pact_consumer
```

**Check SLOs**
```bash
python scripts/ci-cd/check-slos.py --environment production --time-range 1h
```

### CI/CD Pipeline

The pipeline is triggered automatically on:
- Push to `main` or `develop` branches
- Pull requests
- Manual workflow dispatch

Pipeline stages:
1. Code quality checks
2. Unit tests
3. Integration tests
4. Contract tests
5. Performance tests
6. Build Docker images
7. Security scanning
8. Deploy to staging
9. Deploy to production
10. Post-deployment validation

### Key Files

- `.github/workflows/microservices-ci-cd.yml` - Main CI/CD pipeline
- `infrastructure/terraform/main.tf` - Infrastructure as Code
- `infrastructure/argocd/applications/` - GitOps configurations
- `testing/performance/load-test.js` - k6 performance tests
- `testing/contract/src/consumer_tests.rs` - Pact contract tests

### Monitoring

Access Grafana dashboards:
1. Get Grafana URL: `kubectl get svc -n monitoring prometheus-grafana`
2. Default credentials: admin / prom-operator
3. Available dashboards:
   - TracSeq Overview
   - Microservices Health
   - Infrastructure Metrics
   - SLO Compliance

### Troubleshooting

**Check pod status**
```bash
kubectl get pods -n tracseq
kubectl logs -n tracseq <pod-name>
```

**View ArgoCD application status**
```bash
kubectl get applications -n argocd
argocd app get tracseq-microservices
```

**Check performance regression**
```bash
python scripts/ci-cd/check-performance-regression.py \
  --current load-test-summary.json \
  --baseline https://baseline-storage.com/baseline.json
```

## Next Steps

1. Configure production DNS records
2. Set up SSL certificates
3. Configure production secrets
4. Run full test suite
5. Set up backup procedures

## Support

- Documentation: [docs/PHASE_9_IMPLEMENTATION_SUMMARY.md](docs/PHASE_9_IMPLEMENTATION_SUMMARY.md)
- Issues: https://github.com/tracseq/tracseq-2.0/issues
- Slack: #tracseq-devops

---

Phase 9 establishes enterprise-grade DevOps practices for TracSeq 2.0, enabling rapid, reliable deployments with comprehensive quality gates.