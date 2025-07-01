# TracSeq 2.0 GitHub Workflows

This directory contains the CI/CD workflows for the TracSeq 2.0 Laboratory Management System.

## 🔄 Workflows Overview

### 1. **Main CI Pipeline** (`ci.yml`)
The primary continuous integration workflow that runs on every push and pull request.

**Triggers:**
- Push to `main`, `master`, or `dev` branches
- Pull requests to these branches
- Manual workflow dispatch

**Features:**
- 🔍 Automatic change detection for different components
- 🎨 Frontend checks (TypeScript, ESLint, tests, build)
- 🦀 Rust service checks (format, clippy, tests) with matrix builds
- 🐍 Python service checks (black, isort, flake8, mypy, tests)
- 🧪 Integration tests
- 📊 Test result summaries

### 2. **Deployment Pipeline** (`deploy.yml`)
Handles deployment to different environments.

**Triggers:**
- Push to `main` or `master` branches
- Version tags (`v*`)
- Manual workflow dispatch with environment selection

**Features:**
- 🏗️ Multi-platform Docker image builds (amd64, arm64)
- 🔍 Automatic Dockerfile generation if missing
- 🚀 Environment-specific deployments (development, staging, production)
- 📊 Post-deployment validation
- 🏷️ Proper image tagging and versioning

### 3. **Microservices CI/CD** (`microservices-ci-cd.yml`)
Specialized workflow for microservice changes with optimized builds.

**Triggers:**
- Push/PR with changes to `lims-core/**` or `lims-ai/**`

**Features:**
- 🔍 Smart change detection per service
- 🎯 Targeted builds only for changed services
- 🐳 Automatic Docker image building and pushing
- 🔒 Security scanning with Trivy
- 📈 Test coverage tracking per service

### 4. **Playwright E2E Tests** (`playwright.yml`)
End-to-end testing for the frontend application.

**Triggers:**
- Push/PR with changes to `lims-ui/**`
- Manual workflow dispatch

**Features:**
- 🎭 Full browser automation tests
- 📸 Test report artifacts
- 🔄 Automatic browser installation
- 💾 Efficient pnpm caching

### 5. **Security Scanning** (`security.yml`)
Comprehensive security analysis across all components.

**Triggers:**
- Push to main branches
- Weekly scheduled scans (Sundays)
- Manual workflow dispatch

**Features:**
- 🔍 Dependency vulnerability scanning (Rust, Python, Node.js)
- 🐍 Python code security with Bandit
- 🦀 Rust security with cargo-deny
- 🔐 Secret scanning with Gitleaks
- 🐳 Container security with Trivy
- 📊 CodeQL analysis for JavaScript and Python

### 6. **Performance Testing** (`performance.yml`)
Performance benchmarking and load testing.

**Triggers:**
- Push to `main` branch
- PRs with `performance` label
- Weekly scheduled runs (Mondays)
- Manual workflow dispatch with custom parameters

**Features:**
- 🚀 k6 load testing for Rust services
- � Locust testing for Python services
- 🎨 Lighthouse CI for frontend performance
- 📊 Consolidated performance reports
- ⚡ Configurable test duration and concurrent users

## 🛠️ Workflow Configuration

### Environment Variables
All workflows use consistent environment variables:
```yaml
env:
  RUST_VERSION: '1.75'
  PYTHON_VERSION: '3.11'
  NODE_VERSION: '20'
  PNPM_VERSION: '10.12.2'
```

### Service Matrix
The workflows automatically detect and build these services:

**Rust Services** (in `lims-core/`):
- auth_service
- sample_service
- enhanced_storage_service
- event_service
- notification_service
- transaction_service
- sequencing_service
- qaqc_service
- template_service
- spreadsheet_versioning_service
- library_details_service
- dashboard_service
- reports_service
- cognitive_assistant_service
- barcode_service

**Python Services**:
- api_gateway (in `lims-core/`)
- lab_submission_rag (in `lims-ai/`)
- enhanced_rag_service (in `lims-ai/`)

**Frontend**:
- lims-ui (React/TypeScript application)

## 🚀 Usage

### Running Workflows Manually

Most workflows support manual triggering via GitHub's UI:

1. Go to Actions tab
2. Select the workflow
3. Click "Run workflow"
4. Configure parameters (if available)
5. Click "Run workflow" button

### Workflow Badges

Add these badges to your README:

```markdown
![CI](https://github.com/YOUR_ORG/tracseq2.0/workflows/🧬%20TracSeq%202.0%20CI%20Pipeline/badge.svg)
![Security](https://github.com/YOUR_ORG/tracseq2.0/workflows/🔒%20Security%20Scan/badge.svg)
![Deploy](https://github.com/YOUR_ORG/tracseq2.0/workflows/🚀%20TracSeq%202.0%20Deploy/badge.svg)
```

## 🔧 Maintenance

### Updating Dependencies

To update tool versions, modify the environment variables at the top of each workflow:

```yaml
env:
  RUST_VERSION: '1.75'  # Update to latest stable
  PYTHON_VERSION: '3.11'  # Update to latest stable
  NODE_VERSION: '20'  # Update to latest LTS
```

### Adding New Services

1. Add the service to the appropriate matrix in workflows
2. Ensure the service has proper test commands
3. Create a Dockerfile (or let workflows auto-generate)
4. Update this README

### Debugging Failed Workflows

1. Check the workflow run logs in the Actions tab
2. Look for specific error messages in failed steps
3. Use `workflow_dispatch` to run with different parameters
4. Add debug logging with `echo` statements
5. Use action artifacts to save debug information

## � Best Practices

1. **Keep workflows DRY**: Use matrix builds and reusable patterns
2. **Cache aggressively**: Cache dependencies to speed up builds
3. **Fail fast**: Use `fail-fast: false` only when needed
4. **Security first**: Run security scans on every PR
5. **Monitor performance**: Regular performance testing prevents degradation
6. **Document changes**: Update this README when modifying workflows

## 🆘 Troubleshooting

### Common Issues

1. **PostgreSQL connection failures**: Ensure service health checks pass
2. **Cache misses**: Check if lock files have changed
3. **Docker build failures**: Verify Dockerfile syntax and base images
4. **Test timeouts**: Increase timeout values for slower tests
5. **Permission errors**: Check GITHUB_TOKEN permissions

### Getting Help

1. Check workflow logs for detailed error messages
2. Review recent changes that might have broken workflows
3. Consult GitHub Actions documentation
4. Open an issue with workflow logs attached

---

**📝 Note**: These workflows are designed for the TracSeq 2.0 monorepo structure. Ensure your local development environment matches the expected project layout before running workflows.

*Context improved by Giga AI* 
