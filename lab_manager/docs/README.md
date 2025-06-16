# 📚 Documentation Index - Lab Manager

Welcome to the Lab Manager documentation! This index provides easy navigation to all available documentation for users, developers, and administrators.

## 🚀 Getting Started

### **New Users**
- 🌟 [**README**](../README.md) - Start here for project overview and quick setup
- 🪟 [**Windows Setup Guide**](../README-Windows.md) - Windows-specific installation instructions
- 🐳 [**Docker Quick Start**](DOCKER_QUICK_START.md) - Get running with Docker in minutes
- 🛠️ [**Development Setup**](DEVELOPMENT_SETUP.md) - Full development environment setup

### **System Overview**
- 🔬 [**System Overview**](SYSTEM_OVERVIEW.md) - Comprehensive platform overview
- 🏗️ [**Architecture Guide**](MODULAR_ARCHITECTURE.md) - System architecture and design patterns
- 🔧 [**API Overview**](API_OVERVIEW.md) - Complete API documentation

## 🧪 Feature Documentation

### **Core Features**
- 🧬 [**Sample Management**](SAMPLE_EDITING_FEATURE.md) - Sample lifecycle and management
- 🏪 [**Storage Management**](storage-management-flows.md) - Temperature-controlled storage system
- 📊 [**Spreadsheet Processing**](SPREADSHEET_SERVICE.md) - Data upload and processing
- 📋 [**Template System**](TEMPLATE_EDITING_FEATURE.md) - Template creation and management

### **Advanced Features**
- 🤖 [**RAG Integration**](RAG_INTEGRATION.md) - AI-powered document processing
- 📈 [**SQL Reports**](SQL_REPORTS_FEATURE.md) - Custom reporting and analytics
- 🔍 [**Spreadsheet Data Viewer**](FRONTEND_SPREADSHEET_GUIDE.md) - Data visualization interface

## 👨‍💻 Developer Documentation

### **Development Guides**
- 🛠️ [**Development Setup**](DEVELOPMENT_SETUP.md) - Complete dev environment setup
- 🏗️ [**Modular Architecture**](MODULAR_ARCHITECTURE.md) - System design principles
- 🧩 [**Component Guide**](COMPONENT_GUIDE.md) - Frontend component architecture
- 🧪 [**Testing Guide**](EXPANDED_TEST_COVERAGE.md) - Testing strategy and coverage

### **Technical Specifications**
- 🔧 [**API Documentation**](API_OVERVIEW.md) - RESTful API reference
- 🌐 [**Frontend Architecture**](FRONTEND_ARCHITECTURE.md) - React/TypeScript frontend
- 📊 [**System Overview**](SYSTEM_OVERVIEW.md) - Technical architecture overview

### **AI & Processing**
- 🤖 [**RAG Integration**](RAG_INTEGRATION.md) - AI document processing system
- 🔬 [**RAG Testing Guide**](RAG_TESTING_GUIDE.md) - Testing AI components
- 💬 [**Chatbot Setup**](RAG_CHATBOT_SETUP.md) - AI chatbot configuration

## 🚀 Deployment & Operations

### **Deployment**
- 🐳 [**Docker Quick Start**](DOCKER_QUICK_START.md) - Container deployment
- 🔧 [**Docker Troubleshooting**](DOCKER_TROUBLESHOOTING.md) - Common deployment issues
- 🚀 [**CI/CD Guide**](CI_CD_GUIDE.md) - Continuous integration and deployment

### **Administration**
- 👥 [**User Guide**](user-guide/) - End-user documentation
- 🔒 [**Security & Compliance**](SYSTEM_OVERVIEW.md#security--compliance) - Security features
- 📊 [**Monitoring & Observability**](SYSTEM_OVERVIEW.md#monitoring--observability) - System monitoring

## 🔄 Process Documentation

### **Workflows**
- 📋 [**Sample Submission Workflow**](SYSTEM_OVERVIEW.md#workflow-examples) - End-to-end sample processing
- 🏪 [**Storage Assignment Workflow**](SYSTEM_OVERVIEW.md#workflow-examples) - Storage management process
- 🤖 [**RAG Processing Workflow**](RAG_INTEGRATION.md) - AI document processing

### **Quality Assurance**
- 🧪 [**Testing Strategy**](EXPANDED_TEST_COVERAGE.md) - Comprehensive testing approach
- 🔍 [**Bug Fixes & Issues**](BUG_FIXES_SAMPLE_MODAL_BATCH_API.md) - Issue resolution documentation
- 📈 [**Performance Testing**](TEST_SUMMARY.md) - Performance validation

## 📊 Feature Summaries

### **Implementation Summaries**
- 📈 [**Modular Achievements**](MODULAR_ACHIEVEMENTS.md) - Architecture accomplishments
- 🔄 [**Workflow Upgrades**](WORKFLOW_UPGRADES_SUMMARY.md) - Process improvements
- 🧩 [**Modular Summary**](MODULAR_SUMMARY.md) - Component organization

### **Test Coverage**
- 📊 [**Test Summary**](TEST_SUMMARY.md) - Overall testing status
- 🔧 [**Test Fixes Summary**](TEST_FIXES_SUMMARY.md) - Issue resolution tracking
- 📈 [**Expanded Test Coverage**](EXPANDED_TEST_COVERAGE.md) - Comprehensive testing

## 🤝 Community & Support

### **Contributing**
- 📝 [**Contributing Guide**](CONTRIBUTING.md) - How to contribute to the project
- 🔧 [**Development Guidelines**](../README.md#development) - Code standards and practices
- 🐛 [**Bug Reports**](https://github.com/poglesbyg/lab_manager/issues) - Report issues

### **Support Resources**
- 💬 [**Discussions**](https://github.com/poglesbyg/lab_manager/discussions) - Community discussions
- 📧 **Email**: support@lab-manager.dev
- 📖 **Documentation**: This documentation site

## 🔍 Quick Reference

### **Common Commands**
```bash
# Start development environment
./run_full_app.sh

# Run tests
cargo test                    # Backend tests
cd frontend && npm test       # Frontend tests

# Database operations
docker-compose exec db psql -U postgres -d lab_manager

# View logs
docker-compose logs -f backend
docker-compose logs -f rag
```

### **Access URLs (Development)**
- 🌐 **Frontend**: http://localhost:5173
- 🔧 **Backend API**: http://localhost:3000
- 📊 **RAG Service**: http://localhost:8000
- 🗄️ **Database**: localhost:5432

### **Key Directories**
```
lab_manager/
├── 📁 src/                  # Rust backend source
├── 📁 frontend/src/         # React frontend source
├── 📁 docs/                 # Documentation (you are here)
├── 📁 migrations/           # Database migrations
├── 📁 scripts/              # Utility scripts
└── 📁 config/               # Configuration files
```

## 📋 Documentation Status

### **✅ Complete Documentation**
- System overview and architecture
- Development setup and workflow
- Core feature documentation
- API reference and examples
- Testing and quality assurance

### **🚧 In Progress**
- User guides and tutorials
- Advanced configuration options
- Performance optimization guides
- Troubleshooting procedures

### **📋 Planned Documentation**
- Video tutorials and walkthroughs
- Migration guides from other LIMS
- Advanced integration examples
- Performance benchmarking results

## 🔄 Document Updates

This documentation is actively maintained and updated with each release. For the most current information:

1. **Check Git History**: `git log --oneline docs/` for recent changes
2. **Version Tags**: Documentation versions align with software releases
3. **Issue Tracking**: Documentation improvements tracked in GitHub issues
4. **Community Input**: Feedback welcome through discussions and issues

---

**📚 Happy Learning!** If you can't find what you're looking for, please [open an issue](https://github.com/poglesbyg/lab_manager/issues) or start a [discussion](https://github.com/poglesbyg/lab_manager/discussions).

*Context improved by Giga AI* 
