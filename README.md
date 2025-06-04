# Lab Manager

A comprehensive scientific sample management system that integrates spreadsheet template handling, storage management with barcode tracking, and sequencing job orchestration.

## Documentation

For detailed documentation, please visit our [Documentation Center](docs/README.md). The documentation includes:
- User guides
- API documentation
- Development guides
- Deployment instructions
- Troubleshooting guides

## Features

- **Template Management**
  - Spreadsheet template upload and processing
  - Metadata extraction and tracking
  - Relational database mapping
  - Template validation and verification

- **Sample Processing**
  - Sample submission wizard
  - Status tracking through submission lifecycle
  - Barcode generation and association
  - Validation rules enforcement

- **Sequencing Management**
  - Sample sheet creation for sequencing jobs
  - Job status management
  - Template-to-sample relationships
  - Sequencing workflow validation

- **Storage Control**
  - Barcode-based storage tracking
  - Location management
  - Sample storage state transitions
  - Storage validation rules

## Tech Stack

### Backend
- Rust
- PostgreSQL
- SQLx for database operations
- Actix-web for API endpoints

### Frontend
- React with TypeScript
- Vite for build tooling
- Tailwind CSS for styling
- React Router for navigation

### Infrastructure
- Docker and Docker Compose for containerization
- Nginx for production serving
- PostgreSQL for data persistence

## Prerequisites

- Docker and Docker Compose
- Git
- Ports 80, 3000, 3001, 5173, and 5432 available

## Quick Start

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd lab_manager
   ```

2. Run the project:
   ```bash
   ./scripts/run.sh
   ```

   This will:
   - Set up the environment
   - Build and start all services
   - Show access points and logs

## Access Points

- Production Frontend: http://localhost
- Production API: http://localhost/api
- Development Frontend: http://localhost:5173
- Development API: http://localhost:3000

## Development

### Running in Development Mode

```bash
docker-compose up frontend-dev dev db
```

This will start:
- Frontend with hot reloading
- Backend with auto-rebuild
- PostgreSQL database

### Project Structure

```
lab_manager/
├── frontend/           # React frontend application
├── src/               # Rust backend source code
│   ├── config/       # Configuration
│   ├── handlers/     # API handlers
│   ├── models/       # Data models
│   ├── sample_submission/  # Sample processing
│   ├── sequencing/   # Sequencing management
│   └── storage/      # Storage management
├── migrations/        # Database migrations
├── scripts/          # Utility scripts
└── docker-compose.yml # Docker configuration
```

### Database Migrations

Migrations are managed using SQLx. To create a new migration:

```bash
cargo sqlx migrate add <migration_name>
```

To run migrations:

```bash
cargo sqlx migrate run
```

## Docker Commands

```bash
# View logs
docker-compose logs -f

# Stop all services
docker-compose down

# Rebuild and restart services
docker-compose up --build -d

# View running containers
docker-compose ps
```

## Environment Variables

The following environment variables are used:

- `DATABASE_URL`: PostgreSQL connection string
- `STORAGE_PATH`: Path for file storage
- `RUST_LOG`: Logging level for Rust application

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

[Add your license information here]

## Support

[Add support information here] 
