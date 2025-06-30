# 🧪 LIMS Microservice System – Full Stack AI-Driven Application

A modern, modular **Laboratory Information Management System (LIMS)** built with cutting-edge technologies for efficient laboratory operations and intelligent data processing.

## 🚀 Technology Stack

- 🦀 **Rust** - High-performance microservices (Actix/Axum)
- 🐍 **Python** - AI and data analysis (FastAPI, PyTorch, Pandas) 
- 💻 **TypeScript + React** - Interactive frontend (Tailwind CSS)
- 🐘 **PostgreSQL** - Robust data persistence
- 🐳 **Docker** - Containerized deployment
- 🤖 **AI/ML** - RAG, document processing, predictive analytics

## 🧠 Purpose

Build an intelligent LIMS platform that empowers lab technicians and researchers to:
- Track samples and batches through complete workflows
- Run AI-powered analyses (e.g., flagging abnormal results)
- Manage storage with IoT sensor integration
- Process laboratory documents with AI extraction
- Interface through a modern web dashboard
- Integrate securely with existing laboratory systems

## 📁 Project Structure

```
lims-microservices/
├── lims-core/              # Rust microservices
│   ├── auth_service/       # Authentication & authorization
│   ├── sample_service/     # Sample management
│   ├── storage_service/    # Storage tracking & IoT
│   ├── lab_manager/        # Core laboratory workflows
│   └── ...                 # Other services
│
├── lims-ai/                # Python AI services
│   ├── enhanced_rag_service/   # Document processing
│   ├── lab_submission_rag/     # Submission analysis
│   ├── ml-models/              # Trained models
│   └── ml-platform/            # MLOps infrastructure
│
├── lims-ui/                # React + TypeScript frontend
│   ├── src/                # Source code
│   ├── public/             # Static assets
│   └── ...                 # Frontend configs
│
├── db/                     # Database resources
│   ├── migrations/         # SQL migrations by service
│   ├── seeds/              # Test data
│   └── docs/               # ERD and schemas
│
└── docker/                 # Docker configurations
    ├── docker-compose.yml  # Main compose file
    ├── postgres/           # DB initialization
    └── ...                 # Service Dockerfiles
```

## 🛠️ Quick Start

### Prerequisites
- Docker & Docker Compose
- Rust (1.90+)
- Node.js (20+) & pnpm
- Python (3.11+)

### Development Setup

1. **Clone and navigate:**
   ```bash
   git clone https://github.com/your-org/lims-microservices.git
   cd lims-microservices
   ```

2. **Start all services:**
   ```bash
   cd docker
   docker-compose up -d
   ```

3. **Access the application:**
   - Frontend: http://localhost:3000
   - API Gateway: http://localhost:8080
   - Auth Service: http://localhost:8001
   - Sample Service: http://localhost:8002

### Development Commands

```bash
# Rust services (from lims-core/)
cargo build --workspace
cargo test --workspace
cargo run --bin auth_service

# Frontend (from lims-ui/)
pnpm install
pnpm dev
pnpm build

# Python AI services (from lims-ai/)
pip install -r requirements.txt
python -m enhanced_rag_service
```

## 🏗️ Architecture

### Microservices Overview

| Service | Port | Description |
|---------|------|-------------|
| API Gateway | 8080 | Request routing & authentication |
| Auth Service | 8001 | JWT authentication & RBAC |
| Sample Service | 8002 | Sample tracking & workflows |
| Storage Service | 8003 | Storage management & IoT |
| RAG Service | 8100 | AI document processing |
| Frontend | 3000 | React web interface |

### Data Flow
```
Frontend → API Gateway → Microservices → Database
                      ↓
                 AI Services → Vector DB
```

## 🔧 Configuration

### Environment Variables
Create `.env` files in each service directory:

```env
# Database
DATABASE_URL=postgres://user:pass@localhost:5432/lims_db

# Redis
REDIS_URL=redis://localhost:6379

# JWT
JWT_SECRET=your-secret-key

# AI Services
OLLAMA_HOST=http://localhost:11434
```

## 🧪 Testing

```bash
# Run all tests
./scripts/test-all.sh

# Service-specific tests
cd lims-core/sample_service && cargo test
cd lims-ui && pnpm test
cd lims-ai && pytest
```

## 📊 Key Features

### Laboratory Management
- Sample registration and tracking
- Batch processing workflows
- Chain of custody documentation
- Quality control checkpoints

### Storage System
- Multi-temperature zone management (-80°C to 37°C)
- Real-time IoT sensor monitoring
- Automated alerts for excursions
- Capacity optimization

### AI-Powered Processing
- Document extraction from PDFs
- Intelligent form parsing
- Anomaly detection in results
- Predictive maintenance

### Security
- JWT-based authentication
- Role-based access control (RBAC)
- Audit logging
- Data encryption at rest

## 🚀 Deployment

### Production Deployment
```bash
# Build and deploy all services
./scripts/deploy-production.sh

# Using Kubernetes
kubectl apply -f k8s/
```

### Monitoring
- Prometheus metrics: http://localhost:9090
- Grafana dashboards: http://localhost:3001
- Health checks: http://localhost:8080/health

## 📖 Documentation

- [API Documentation](./docs/api/)
- [Architecture Guide](./docs/architecture.md)
- [Development Guide](./docs/development.md)
- [Deployment Guide](./docs/deployment.md)

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

Built with ❤️ by the Laboratory Systems Team

---

For more information, visit our [documentation site](https://docs.lims-system.com) or contact the team. 