# 🧬 TracSeq 2.0 - Advanced Laboratory Information Management System

[![Build Status](https://github.com/poglesbyg/tracseq2.0/actions/workflows/ci.yml/badge.svg)](https://github.com/poglesbyg/tracseq2.0/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75+-blue.svg)](https://www.rust-lang.org)
[![React](https://img.shields.io/badge/react-18.3+-61dafb.svg)](https://reactjs.org)
[![Python](https://img.shields.io/badge/python-3.9+-blue.svg)](https://www.python.org)

> **Modern laboratory information management system with AI-powered document processing, intelligent storage management, and comprehensive sample tracking for biological research workflows.**

## 🚀 Quick Start

### One-Click Windows Start
```cmd
./scripts/start-tracseq.cmd
```

### Cross-Platform Development
```bash
# Clone and start the entire system
git clone https://github.com/poglesbyg/tracseq2.0.git
cd tracseq2.0
./scripts/run_full_app.sh
```

### Docker Compose (Recommended)
```bash
# Start all services
docker-compose up -d

# Start only specific services
docker-compose up -d db rag-service  # Infrastructure only
docker-compose up -d app frontend    # Application layer
```

**Access URLs:**
- 🌐 **Frontend**: http://localhost:5173 (dev) / http://localhost:8080 (prod)
- 🔧 **Lab Manager API**: http://localhost:3000  
- 📊 **RAG Service**: http://localhost:8000
- 🗄️ **Database**: localhost:5433

## 🏗️ Repository Structure

This is a **multi-component workspace** with clean separation of concerns:

```
tracseq2.0/                          # 🏠 Workspace Root
├── 📋 README.md                     # This file - main documentation
├── ⚙️ Cargo.toml                     # Rust workspace configuration
├── 🐳 docker-compose.yml            # Main orchestration
├── 📄 LICENSE                       # MIT license
├── 🙈 .gitignore                    # Git ignore patterns
│
├── 🧪 lab_manager/                  # Rust Backend + React Frontend
│   ├── 🦀 src/                     # Rust backend source
│   ├── ⚛️ frontend/                # React frontend application
│   ├── 🗃️ migrations/              # Database migrations
│   ├── 📋 Cargo.toml               # Component configuration
│   ├── 🐳 Dockerfile               # Production container
│   └── 📊 examples/                # Usage examples
│
├── 🤖 lab_submission_rag/          # Python RAG Processing Service
│   ├── 🌐 api/                     # FastAPI service
│   ├── 🧠 rag/                     # Document processing engine
│   ├── 📦 models/                  # Data models
│   ├── 🧪 tests/                   # Python tests
│   ├── 📋 pyproject.toml           # Python configuration
│   ├── 🐳 Dockerfile               # Service container
│   └── 📋 requirements.txt         # Dependencies
│
├── 📚 docs/                        # 📖 Workspace Documentation
│   ├── api/                        # API documentation
│   ├── user-guide/                 # User guides
│   ├── DOCKER_INTEGRATION_GUIDE.md # Docker setup guide
│   ├── README-Windows.md           # Windows-specific instructions
│   └── [other documentation]
│
├── 🚀 deploy/                      # 🏭 Deployment Configurations
│   ├── production/                 # Production configs
│   │   └── docker-compose.production.yml
│   ├── development/                # Development configs
│   │   └── docker-compose.unified.yml
│   ├── tracseq.env                 # Main environment file
│   └── tracseq.env.example         # Environment template
│
├── 📝 scripts/                     # 🛠️ Workspace Scripts
│   ├── run_full_app.sh            # Main startup script
│   ├── start-tracseq.cmd          # Windows startup
│   ├── run.ps1                    # PowerShell runner
│   ├── demo-integration.ps1       # Demo scripts
│   └── [other utility scripts]
│
└── 💾 uploads/                     # 📁 Runtime Data Storage
```

## ✨ Key Features

### 🧪 **Sample Management**
- **AI-Powered Document Processing** - RAG integration extracts structured data from unstructured lab documents
- **Intelligent Sample Validation** - Multi-stage validation with confidence scoring (0.5-1.0 threshold)
- **Automated Barcode Generation** - Laboratory naming conventions with format `{TYPE}-{TIMESTAMP}-{RANDOM}`
- **State-Based Workflow** - `Pending → Validated → InStorage → InSequencing → Completed`

### 🏪 **Storage Management**
- **Temperature Zone Control** - Five zones: -80°C, -20°C, 4°C, RT, 37°C
- **Intelligent Capacity Management** - Real-time tracking with threshold alerts (80% warning, 95% critical)
- **Chain of Custody** - Complete sample movement tracking with audit trail
- **Hierarchical Organization** - Building/Room/Freezer/Shelf with container type support

### 📊 **Data Processing**
- **Multi-Format Spreadsheet Support** - CSV, XLS, XLSX with template-based processing
- **RAG Document Analysis** - Confidence scoring for data extraction quality
- **Advanced Search & Filtering** - Semantic search across all sample data
- **Real-time Data Validation** - Laboratory-specific validation rules

### 🔐 **Security & Access Control**
- **Role-Based Access** - Lab Admin, PI, Technician, Scientist, Analyst, Guest
- **JWT Authentication** - Secure session management with refresh tokens
- **Audit Logging** - Complete activity tracking for compliance
- **Multi-tenant Support** - Department and lab-specific access controls

## 🏗️ Architecture

### **Technology Stack**
```
Frontend:  React 18 + TypeScript + Vite + TailwindCSS
Backend:   Rust + Axum + SQLx + PostgreSQL  
AI/RAG:    Python + FastAPI + Ollama/LLM
Deploy:    Docker + Compose + GitHub Actions
```

### **Service Architecture**
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   React SPA     │    │  Rust Backend   │    │  Python RAG     │
│  (Frontend)     │◄──►│ (Lab Manager)   │◄──►│   (AI Service)  │
│                 │    │                 │    │                 │
│ • Sample UI     │    │ • REST API      │    │ • Doc Analysis  │
│ • Dashboard     │    │ • Auth Service  │    │ • AI Models     │
│ • Storage Mgmt  │    │ • Sample Logic  │    │ • Confidence    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 ▼
                    ┌─────────────────┐
                    │   PostgreSQL    │
                    │   (Database)    │
                    │                 │
                    │ • Sample Data   │
                    │ • User Records  │
                    │ • Storage Info  │
                    └─────────────────┘
```

## 📋 Prerequisites

### **Required**
- 🐳 **Docker Desktop** 20.10+
- 💾 **8GB RAM** minimum (16GB recommended)
- 💿 **5GB free disk space**

### **Operating System Support**
- ✅ **Windows 10/11** with WSL2
- ✅ **macOS** 10.15+
- ✅ **Linux** (Ubuntu 20.04+, RHEL 8+)

### **Optional Development Tools**
- 🦀 **Rust** 1.75+ (for backend development)
- 📦 **Node.js** 20+ (for frontend development)
- 🐍 **Python** 3.9+ (for RAG development)
- 🔧 **Git** (for version control)

## 🛠️ Installation & Setup

### **Production Deployment**
```bash
# Use production configuration
docker-compose -f deploy/production/docker-compose.production.yml up -d

# Or copy environment template
cp deploy/tracseq.env.example .env
# Edit .env with your settings
docker-compose up -d
```

### **Development Setup**
```bash
# Full development environment (all services)
./scripts/run_full_app.sh

# Unified development (streamlined)
docker-compose -f deploy/development/docker-compose.unified.yml up -d

# Individual service development
docker-compose up -d db              # Database only
docker-compose up dev frontend-dev   # Development servers
docker-compose up rag-service        # RAG service only
```

### **Component Development**
```bash
# Lab Manager (Rust + React)
cd lab_manager
cargo run                           # Backend development
cd frontend && npm run dev          # Frontend development

# RAG Service (Python)
cd lab_submission_rag
python -m uvicorn api.main:app --reload
```

### **Windows-Specific Setup**
See [📖 Windows Setup Guide](docs/README-Windows.md) for detailed Windows instructions.

## 🎯 Usage Guide

### **Quick Commands**
```bash
# Start everything
docker-compose up -d

# View logs
docker-compose logs -f [service-name]

# Stop everything
docker-compose down

# Reset everything (⚠️ destroys data)
docker-compose down -v
```

### **Service Management**
```bash
# Scale services
docker-compose up -d --scale rag-service=2

# Update a single service
docker-compose up -d --build app

# Access service shells
docker-compose exec app bash
docker-compose exec rag-service bash
```

## 📚 Documentation

### **Getting Started**
- 📖 [Windows Setup Guide](docs/README-Windows.md)
- 🛠️ [Development Setup](docs/DEVELOPMENT_SETUP.md)
- 🐳 [Docker Integration Guide](docs/DOCKER_INTEGRATION_GUIDE.md)

### **Feature Guides**
- 🧪 [Sample Management](docs/SAMPLE_EDITING_FEATURE.md)
- 🏪 [Storage Management](lab_manager/docs/storage-management-flows.md)
- 📊 [Spreadsheet Processing](docs/SPREADSHEET_SERVICE.md)
- 🤖 [RAG Integration](docs/RAG_INTEGRATION.md)

### **Technical Documentation**
- 🏗️ [Architecture Overview](docs/MODULAR_ARCHITECTURE.md)
- 🔧 [API Documentation](docs/api/)
- 🧪 [Testing Guide](docs/EXPANDED_TEST_COVERAGE.md)
- 🚀 [CI/CD Guide](docs/CI_CD_GUIDE.md)

### **User Guides**
- 👥 [User Management](docs/user-guide/)
- 📋 [Template Creation](docs/TEMPLATE_EDITING_FEATURE.md)
- 📊 [Reports & Analytics](docs/SQL_REPORTS_FEATURE.md)

## 🔧 Development

### **Workspace Commands**
```bash
# Run all tests
./scripts/run_tests.sh

# Start development environment
./scripts/run_full_app.sh

# Windows development
./scripts/run.ps1

# Demo integration
./scripts/demo-integration.ps1
```

### **Component Development**
```bash
# Lab Manager (Rust Backend)
cd lab_manager
cargo test                        # Run backend tests
cargo clippy                      # Rust linting
cargo build --release             # Production build

# Frontend (React)
cd lab_manager/frontend
npm test                          # Run frontend tests
npm run lint                      # Frontend linting
npm run build                     # Production build

# RAG Service (Python)
cd lab_submission_rag
pytest                            # Run Python tests
flake8                            # Python linting
python -m build                   # Build package
```

### **Contributing**
1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes in the appropriate component
4. Add tests for your changes
5. Ensure all tests pass: `./scripts/run_tests.sh`
6. Commit: `git commit -m 'Add amazing feature'`
7. Push: `git push origin feature/amazing-feature`
8. Open a Pull Request

See [CONTRIBUTING.md](docs/CONTRIBUTING.md) for detailed guidelines.

## 🚀 Deployment

### **Environment Configuration**
```bash
# Copy template and configure
cp deploy/tracseq.env.example .env

# Required variables
DATABASE_URL=postgres://user:pass@host:port/db
RUST_LOG=info
RAG_SERVICE_URL=http://rag-service:8000

# Optional configurations
JWT_SECRET=your-secret-key
STORAGE_PATH=/app/storage
OLLAMA_HOST=http://localhost:11434
```

### **Production Deployment**
```bash
# Using production configuration
docker-compose -f deploy/production/docker-compose.production.yml up -d

# Using GitHub Actions (automatic)
git push origin main  # Triggers CI/CD pipeline
```

### **Monitoring & Maintenance**
- 📊 Health checks at `/health` endpoint
- 📝 Structured logging with configurable levels
- 🔄 Automatic database migrations
- 📈 Performance metrics and monitoring

## 🔍 Troubleshooting

### **Common Issues**

**🐳 Docker Issues**
```bash
# Reset Docker environment
docker-compose down -v
docker system prune -f
docker-compose up -d
```

**🗄️ Database Connection**
```bash
# Check database status
docker-compose ps db
docker-compose logs db

# Test connection
docker-compose exec db psql -U postgres -d lab_manager -c "SELECT 1;"
```

**🌐 Port Conflicts**
```bash
# Find and kill processes using ports
lsof -i :3000 && kill -9 $(lsof -t -i:3000)
lsof -i :5173 && kill -9 $(lsof -t -i:5173)
lsof -i :8000 && kill -9 $(lsof -t -i:8000)
```

**🤖 RAG Service Issues**
```bash
# Check RAG service status
docker-compose logs rag-service

# Restart RAG service
docker-compose restart rag-service
```

See [Docker Integration Guide](docs/DOCKER_INTEGRATION_GUIDE.md) for more solutions.

## 📊 Performance & Scaling

### **System Requirements**
- **Minimum**: 4GB RAM, 2 CPU cores, 10GB storage
- **Recommended**: 16GB RAM, 4 CPU cores, 50GB storage
- **Production**: 32GB RAM, 8 CPU cores, 100GB+ storage

### **Performance Optimization**
- 🔄 Connection pooling for database
- 📦 Asset bundling and compression
- 🗄️ Database indexing on search fields
- 🚀 Rust's zero-cost abstractions for speed

## 🔐 Security

### **Security Features**
- 🔑 JWT-based authentication with refresh tokens
- 🛡️ Role-based access control (RBAC)
- 🔒 Password hashing with Argon2
- 📝 Comprehensive audit logging
- 🌐 CORS protection and security headers

### **Security Best Practices**
- Regular security updates via Dependabot
- Secrets management with environment variables
- Database access controls and encryption
- Input validation and sanitization

## 📈 Roadmap

### **Current Version (v0.1.0)**
- ✅ Core sample management
- ✅ Storage tracking system
- ✅ RAG document processing
- ✅ User authentication and roles
- ✅ Clean repository structure

### **Planned Features**
- 🔬 Enhanced sequencing workflow integration
- 📱 Mobile app for barcode scanning
- 🤖 Advanced AI models for data extraction
- 📊 Advanced analytics and reporting
- 🔗 Laboratory equipment integration
- 🌍 Multi-laboratory support

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](docs/CONTRIBUTING.md) for details.

### **Development Team**
- 👨‍💻 **Core Contributors**: Backend, Frontend, DevOps
- 🧪 **Laboratory Consultants**: Domain expertise and validation
- 🔬 **Research Partners**: Feature requirements and testing

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

- 📧 **Email**: support@tracseq2.dev
- 💬 **Discussions**: [GitHub Discussions](https://github.com/poglesbyg/tracseq2.0/discussions)
- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/poglesbyg/tracseq2.0/issues)
- 📖 **Documentation**: [docs/](docs/)

## 🙏 Acknowledgments

- 🦀 **Rust Community** for excellent tooling and libraries
- ⚛️ **React Team** for the robust frontend framework
- 🤖 **Ollama** for local AI model support
- 🧪 **Laboratory Partners** for domain expertise and testing

---

**Built with ❤️ for the scientific community**

*Context improved by Giga AI*
