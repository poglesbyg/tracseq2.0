# 🚀 GitHub Workflows for TracSeq 2.0

This directory contains GitHub Actions workflows for the TracSeq 2.0 laboratory management platform. These workflows implement modern DevOps practices including continuous integration, security scanning, performance monitoring, and deployment automation.

## 📁 Workflow Overview

### Core Workflows

| Workflow | Purpose | Triggers | Key Features |
|----------|---------|----------|--------------|
| [**ci.yml**](./ci.yml) | Continuous Integration | Push, PR | Lint, test, build validation |
| [**microservices-ci-cd.yml**](./microservices-ci-cd.yml) | Microservices CI/CD | Push, PR | Service-specific testing and deployment |
| [**security.yml**](./security.yml) | Security & Compliance | Push, PR, Schedule | Dependency scanning, secret detection, SBOM |
| [**performance.yml**](./performance.yml) | Performance Testing | Push, PR, Schedule | Benchmarks, load testing, frontend metrics |
| [**deploy.yml**](./deploy.yml) | Deployment Pipeline | Push, Tags, Manual | Multi-environment deployment |

## �️ Project Structure

The workflows are designed to work with the following project structure:

```
tracseq2.0/
├── lims-core/          # Rust microservices
│   ├── auth_service/
│   ├── sample_service/
│   ├── enhanced_storage_service/
│   ├── api_gateway/    # Python API Gateway
│   └── ...
├── lims-ai/            # AI/ML services
│   ├── lab_submission_rag/
│   └── enhanced_rag_service/
├── lims-ui/            # Frontend (React/TypeScript)
└── docker/             # Docker configurations
```

## 🎯 Workflow Features

### 🧪 Continuous Integration (ci.yml)

**Features:**
- Multi-language support (Rust, Python, TypeScript)
- Parallel linting and type checking
- Comprehensive test suites
- Service architecture validation
- Coverage reporting

**Key Jobs:**
- `lint-and-typecheck`: Code quality checks
- `test-suite`: Unit and integration tests
- `microservices-validation`: Service structure validation
- `build-validation`: Docker and deployment validation

### 🚀 Microservices CI/CD (microservices-ci-cd.yml)

**Features:**
- Service-specific pipelines
- Matrix strategy for parallel builds
- Container security scanning
- Automated deployment to staging/production

**Services Covered:**
- Rust services in `lims-core/`
- Python services in `lims-ai/` and `lims-core/api_gateway/`
- Frontend in `lims-ui/`

### 🔒 Security & Compliance (security.yml)

**Features:**
- Dependency vulnerability scanning
- Secret detection (TruffleHog, Gitleaks)
- Container security analysis
- SBOM generation
- License compliance checks

**Security Checks:**
- Rust: cargo-audit
- Python: safety, bandit
- Node.js: npm audit
- Containers: Trivy scanning

### ⚡ Performance Testing (performance.yml)

**Features:**
- Rust service benchmarks
- Load testing with k6
- Frontend performance with Lighthouse
- Performance regression detection

**Metrics Tracked:**
- API response times (p95 < 500ms)
- Throughput and error rates
- Frontend Core Web Vitals
- Laboratory operation performance

### 🚀 Deployment Pipeline (deploy.yml)

**Features:**
- Multi-environment support
- Service-specific deployment modes
- Dockerfile generation for missing files
- Health check validation
- Deployment summaries

**Environments:**
- Development
- Staging
- Production
- Preview (for PRs)

## 🛠️ Usage Guide

### Running Workflows Manually

```bash
# Trigger CI workflow
gh workflow run "TracSeq 2.0 CI/CD Pipeline" --ref main

# Trigger deployment with options
gh workflow run "TracSeq 2.0 Deployment Pipeline" \
  --ref main \
  -f environment=staging \
  -f deployment_mode=full-stack \
  -f version_tag=v1.0.0

# Trigger security scan
gh workflow run "Security & Compliance" --ref main

# Trigger performance tests
gh workflow run "Performance Testing" \
  --ref main \
  -f test_type=comprehensive
```

### Environment Variables

**Common Variables:**
```yaml
env:
  RUST_VERSION: '1.82'
  PYTHON_VERSION: '3.11'
  NODE_VERSION: '20'
  PNPM_VERSION: '10.12.2'
```

**Laboratory-Specific Variables:**
```yaml
env:
  LAB_TEMPERATURE_ZONES: "-80,-20,4,22,37"
  RAG_CONFIDENCE_THRESHOLD: "0.85"
  SAMPLE_LIFECYCLE_STATES: "Pending,Validated,InStorage,InSequencing,Completed"
```

### Required Secrets

- `GITHUB_TOKEN`: Automatically provided
- `POSTGRES_PASSWORD`: Database password (optional)
- Additional deployment-specific secrets as needed

## 📊 Workflow Artifacts

### Generated Artifacts

**CI Artifacts:**
- `coverage-reports/`: Test coverage reports
- `performance-test-artifacts/`: Built binaries for testing

**Security Artifacts:**
- `bandit-reports/`: Python security analysis
- `software-bill-of-materials/`: SBOM files

**Performance Artifacts:**
- `rust-benchmark-results/`: Benchmark outputs
- `load-test-results/`: k6 test results
- `lighthouse-results/`: Frontend performance metrics

### Artifact Retention

- Build artifacts: 1 day
- Test results: 7 days
- Security reports: 30 days

## 🔧 Configuration

### Service Matrix Configuration

The workflows use matrix strategies to test services in parallel:

```yaml
matrix:
  include:
    - service: auth_service
      path: lims-core/auth_service
      type: rust
    - service: lab_submission_rag
      path: lims-ai/lab_submission_rag
      type: python
    - service: frontend
      path: lims-ui
      type: frontend
```

### Performance Thresholds

```yaml
thresholds:
  http_req_duration: ['p(95)<500']  # 95% under 500ms
  http_req_failed: ['rate<0.1']     # <10% error rate
  'categories:performance': ['error', {minScore: 0.8}]  # Lighthouse
```

## 🚨 Monitoring & Notifications

### Workflow Status

All workflows generate summary reports in the GitHub Actions UI:
- Job status tables
- Key metrics and results
- Actionable recommendations

### Failure Handling

- Security vulnerabilities: Warnings (non-blocking)
- Test failures: Blocking
- Performance regressions: Warnings with details
- Build failures: Immediate notification

## 📚 Best Practices

### Workflow Development

1. **Use Matrix Strategies**: Parallelize similar jobs
2. **Cache Dependencies**: Improve build times
3. **Generate Summaries**: Use `$GITHUB_STEP_SUMMARY`
4. **Handle Failures Gracefully**: Use `continue-on-error` where appropriate
5. **Artifact Management**: Clean up old artifacts

### Security

1. **Scan Early**: Run security checks on every PR
2. **Multiple Tools**: Use complementary security scanners
3. **SBOM Generation**: Track dependencies
4. **Secret Management**: Never hardcode secrets

### Performance

1. **Baseline Tracking**: Compare against historical data
2. **Multiple Metrics**: Measure various aspects
3. **Regular Testing**: Schedule periodic runs
4. **Actionable Results**: Provide clear recommendations

## 🔮 Future Enhancements

### Planned Improvements

- **GitOps Integration**: ArgoCD deployment workflows
- **Advanced Monitoring**: Prometheus/Grafana integration
- **Chaos Engineering**: Reliability testing
- **Cost Optimization**: Resource usage tracking

### Roadmap

**Q1 2024:**
- ✅ Simplified workflow structure
- ✅ Service-aware testing
- ✅ Basic security scanning
- ✅ Performance baselines

**Q2 2024:**
- 🔄 Advanced deployment strategies
- � Enhanced monitoring
- 🔄 ML model testing workflows
- 🔄 Compliance automation

## 📞 Support

**Issues**: GitHub Issues for bug reports
**Discussions**: GitHub Discussions for questions
**Documentation**: See `/docs` for detailed guides

---

**📝 Note**: These workflows are designed for the TracSeq 2.0 monorepo structure. Ensure your local development environment matches the expected project layout before running workflows.

*Context improved by Giga AI* 
