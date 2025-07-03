# 🧬 Lab Manager - Advanced Laboratory Information Management System

[![Build Status](https://github.com/poglesbyg/lab_manager/actions/workflows/ci.yml/badge.svg)](https://github.com/poglesbyg/lab_manager/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75+-blue.svg)](https://www.rust-lang.org)
[![React](https://img.shields.io/badge/react-18.3+-61dafb.svg)](https://reactjs.org)

> **Modern laboratory information management system with AI-powered document processing, intelligent storage management, and comprehensive sample tracking for biological research workflows.**

## 🚀 Quick Start

### One-Click Windows Start
```cmd
start-tracseq.cmd
```

### Cross-Platform Development
```bash
# Clone and start
git clone https://github.com/poglesbyg/lab_manager.git
cd lab_manager
./run_full_app.sh
```

**Access URLs:**
- 🌐 **Frontend**: http://localhost:5173
- 🔧 **API**: http://localhost:3000  
- 📊 **RAG Service**: http://localhost:8000
- 🗄️ **Database**: localhost:5432

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
AI/RAG:    Python + FastAPI + Ollama
Deploy:    Docker + GitHub Actions
```

### **Core Components**
```
lab_manager/
├── 🌐 Frontend (React + TypeScript)
│   ├── Sample Management UI
│   ├── Storage Dashboard  
│   ├── Data Visualization
│   └── Authentication
├── ⚙️ Backend (Rust + Axum)
│   ├── REST API Handlers
│   ├── Database Layer (SQLx)
│   ├── Authentication Service
│   └── Storage Management
├── 🤖 RAG Service (Python + FastAPI)
│   ├── Document Processing
│   ├── AI Model Integration
│   └── Confidence Scoring
└── 🗄️ Database (PostgreSQL)
    ├── Sample Records
    ├── Storage Locations
    ├── User Management
    └── Audit Logs
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
- 🔧 **Git** (for version control)

## 🛠️ Installation & Setup

### **Production Deployment**
```bash
# Quick production setup
docker-compose -f docker-compose.prod.yml up -d

# With custom configuration
cp .env.example .env
# Edit .env with your settings
docker-compose up -d
```

### **Development Setup**
```bash
# Full development environment
./run_full_app.sh

# Individual services
docker-compose up -d db          # Database only
docker-compose up -d backend     # Backend + DB
docker-compose up -d frontend    # Frontend dev server
```

### **Windows-Specific Setup**
See [📖 Windows Setup Guide](README-Windows.md) for detailed Windows instructions.

## 🎯 Usage Guide

### **Sample Submission Workflow**

1. **📄 Upload Documents**
   ```
   Upload → RAG Processing → Data Extraction → Validation → Sample Creation
   ```

2. **🧪 Sample Management**
   - Create samples via template upload or manual entry
   - Automatic barcode generation and validation
   - State transition management with approvals

3. **🏪 Storage Operations**
   - Assign samples to temperature-controlled locations
   - Track capacity utilization and movements
   - Generate storage reports and alerts

4. **🔍 Data Analysis**
   - Search samples across all metadata
   - Filter by storage location, temperature, status
   - Export data in multiple formats

### **Key User Workflows**

#### **Lab Administrator**
- Manage users and permissions
- Configure storage locations and templates
- Monitor system health and capacity

#### **Principal Investigator**  
- Oversee project samples and data
- Approve sample state transitions
- Generate compliance reports

#### **Lab Technician**
- Process sample submissions
- Manage storage operations
- Perform quality control checks

#### **Research Scientist**
- Submit samples via document upload
- Track sample processing status
- Access research data and results

## 📚 Documentation

### **Getting Started**
- 📖 [Windows Setup Guide](README-Windows.md)
- 🛠️ [Development Setup](docs/DEVELOPMENT_SETUP.md)
- 🐳 [Docker Quick Start](docs/DOCKER_QUICK_START.md)

### **Feature Guides**
- 🧪 [Sample Management](docs/SAMPLE_EDITING_FEATURE.md)
- 🏪 [Storage Management](docs/storage-management-flows.md)
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

### **Quick Development Commands**
```bash
# Start development environment
./run_full_app.sh

# Run tests
cargo test                    # Backend tests
cd frontend && npm test       # Frontend tests

# Code quality
cargo clippy                  # Rust linting
cd frontend && npm run lint   # Frontend linting

# Database operations
./scripts/migrate.sh          # Run migrations
./scripts/seed.sh            # Seed test data
```

### **Contributing**
1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and add tests
4. Ensure all tests pass: `cargo test && cd frontend && npm test`
5. Commit: `git commit -m 'Add amazing feature'`
6. Push: `git push origin feature/amazing-feature`
7. Open a Pull Request

See [CONTRIBUTING.md](docs/CONTRIBUTING.md) for detailed guidelines.

## 🚀 Deployment

### **Production Deployment**
```bash
# Using Docker Compose
docker-compose -f docker-compose.prod.yml up -d

# Using GitHub Actions (automatic)
git push origin main  # Triggers CI/CD pipeline
```

### **Environment Configuration**
```bash
# Required environment variables
DATABASE_URL=postgres://user:pass@host:port/db
RUST_LOG=info
STORAGE_PATH=/app/storage

# Optional configurations
RAG_SERVICE_URL=http://localhost:8000
OLLAMA_HOST=http://localhost:11434
JWT_SECRET=your-secret-key
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
```

**🤖 RAG Service Issues**
```bash
# Check Ollama status
ollama list
ollama serve

# Restart RAG service
docker-compose restart rag
```

See [DOCKER_TROUBLESHOOTING.md](docs/DOCKER_TROUBLESHOOTING.md) for more solutions.

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

### **Planned Features**
- 🔬 Sequencing workflow integration
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

- 📧 **Email**: support@lab-manager.dev
- 💬 **Discussions**: [GitHub Discussions](https://github.com/poglesbyg/lab_manager/discussions)
- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/poglesbyg/lab_manager/issues)
- 📖 **Documentation**: [docs/](docs/)

## 🙏 Acknowledgments

- 🦀 **Rust Community** for excellent tooling and libraries
- ⚛️ **React Team** for the robust frontend framework
- 🤖 **Ollama** for local AI model support
- 🧪 **Laboratory Partners** for domain expertise and testing

---

**Built with ❤️ for the scientific community**

*Context added by Giga data-models-relationships* 
