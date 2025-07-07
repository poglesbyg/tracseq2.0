# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

TracSeq 2.0 is a modern, AI-powered Laboratory Information Management System (LIMS) built with microservices architecture. It combines Rust backend services, React frontend, and Python AI services to manage laboratory workflows, sample tracking, and intelligent data processing.

## Development Commands

### Frontend (React + TypeScript)
```bash
# From lims-ui/ directory
pnpm install              # Install dependencies
pnpm dev                  # Start development server on localhost:5173
pnpm build               # Build for production
pnpm typecheck           # Run TypeScript type checking
pnpm lint                # Run ESLint
pnpm fix                 # Fix ESLint issues
pnpm test                # Run Jest unit tests
pnpm test:e2e            # Run Playwright E2E tests
```

### Backend (Rust Services)
```bash
# From project root
cargo build --workspace                    # Build all services
cargo test --workspace                     # Run all tests
cargo fmt --all                           # Format code
cargo clippy --workspace --all-targets    # Run linting
cargo run --bin auth_service              # Run specific service

# Run individual service tests
cd lims-core/sample_service && cargo test
```

### AI Services (Python)
```bash
# From lims-ai/ directory
pip install -r requirements.txt
python -m enhanced_rag_service
```

### Development Workflow (Makefile)
```bash
make help                # Show all available commands
make check              # Run all checks (frontend + Rust)
make lint               # Run ESLint on frontend
make test-rust          # Run Rust tests
make test-e2e           # Run E2E tests
make fmt                # Format Rust code
make clean              # Clean build artifacts
```

### Docker Development
```bash
# From docker/ directory
docker-compose up -d        # Start all services
docker-compose down         # Stop all services
docker-compose logs -f      # View logs
```

## Architecture Overview

### Service Structure
The codebase is organized into four main service categories:

**Core Services (`lims-core/`)**
- `auth_service` (Port 8001): JWT authentication & RBAC
- `sample_service` (Port 8002): Sample tracking & workflows
- `project_service` (Port 8101): Project management
- `reports_service` (Port 8014): Reporting & analytics
- `template_service` (Port 8083): Template management
- `transaction_service`: Distributed transactions
- `barcode_service`: Barcode generation/scanning
- `dashboard_service`: Dashboard aggregation

**Enhanced Services (`lims-enhanced/`)**
- `enhanced_storage_service` (Port 8013): AI-powered storage with IoT integration
- `cognitive_assistant_service`: AI assistant for lab workflows
- `event_service`: Event-driven architecture
- `notification_service` (Port 8085): Real-time notifications
- `spreadsheet_versioning_service`: Version control for lab data

**Laboratory Services (`lims-laboratory/`)**
- `lab_manager`: Core laboratory workflow management
- `sequencing_service`: DNA/RNA sequencing workflows
- `qaqc_service` (Port 8103): Quality assurance
- `library_prep_service` (Port 8102): Library preparation
- `flow_cell_service` (Port 8104): Flow cell management
- `library_details_service`: Library metadata

**Gateway (`lims-gateway/`)**
- `api_gateway` (Port 8089): Request routing & authentication

### Technology Stack
- **Backend**: Rust with Axum/Actix frameworks, PostgreSQL, Redis
- **Frontend**: React 18 + TypeScript, Vite, Tailwind CSS, React Query
- **AI Services**: Python with FastAPI, ChromaDB, LlamaIndex
- **Database**: PostgreSQL with SQLx for async operations
- **Deployment**: Docker, Kubernetes, Helm charts

## Key Development Patterns

### Rust Service Structure
Each Rust service follows this pattern:
```
service_name/
├── src/
│   ├── main.rs          # Service entry point
│   ├── handlers/        # HTTP request handlers
│   ├── models/          # Data models
│   ├── services/        # Business logic
│   └── config.rs        # Configuration
├── Cargo.toml
└── Dockerfile
```

### Frontend Structure
```
lims-ui/
├── src/
│   ├── components/      # React components
│   ├── pages/          # Page components
│   ├── services/       # API client code
│   ├── hooks/          # Custom React hooks
│   └── utils/          # Utility functions
```

### Database Migrations
- Located in `db/migrations/` organized by service
- Use `make db-migrate` to run all migrations
- Each service has its own schema namespace

## Important Conventions

### Code Quality
- Always run `make check` before committing
- Use meaningful names for functions and variables
- Follow existing patterns in the codebase
- Keep functions small and focused
- Add proper error handling

### Testing
- Write tests for new functionality
- Use `cargo test --workspace` for Rust tests
- Use `pnpm test` for frontend unit tests
- Use `pnpm test:e2e` for E2E tests
- Test edge cases and error conditions

### Security
- Validate all inputs, especially from external sources
- Use parameterized queries to prevent SQL injection
- Implement proper authentication and authorization
- Never commit secrets or API keys
- Follow the principle of least privilege

### Laboratory Domain
- Always consider data integrity and compliance requirements
- Maintain audit trails for critical operations
- Support both manual and automated workflows
- Validate scientific data formats and constraints
- Consider sample tracking and chain of custody

## Development Environment Setup

1. **Prerequisites**: Docker, Rust 1.90+, Node.js 20+, Python 3.11+
2. **Database**: PostgreSQL runs on port 5433 (Docker)
3. **Frontend**: Runs on port 3000 (development) or 5173 (Vite dev server)
4. **API Gateway**: Runs on port 8089
5. **Individual services**: Use ports 8001-8104 as defined in docker-compose

## AI Integration Notes

- RAG services use ChromaDB for document embeddings
- FastMCP integration for multi-modal AI capabilities
- Support for multiple LLM providers (OpenAI, Anthropic, Ollama)
- Document processing for laboratory submissions
- Predictive analytics for storage optimization

## Debugging Tips

- Use `RUST_LOG=debug` environment variable for detailed logging
- Check service logs with `docker-compose logs -f [service-name]`
- Use `make docker-logs` to view all service logs
- Each service has health check endpoints
- Monitor metrics at http://localhost:9090 (Prometheus)

## Common Issues

- **Port conflicts**: Services use non-standard ports to avoid conflicts
- **Database migrations**: Run `make db-migrate` after pulling changes
- **Dependencies**: Use `cargo clean` and rebuild if facing compilation issues
- **Frontend builds**: Clear node_modules and reinstall if needed