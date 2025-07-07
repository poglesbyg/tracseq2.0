#!/bin/bash

# TracSeq 2.0 Development Environment Setup Script
# This script sets up the database, starts services, and ensures upload functionality works

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running in Docker environment
check_environment() {
    log_info "Checking development environment..."
    
    # Check for required tools
    MISSING_TOOLS=()
    
    if ! command -v node &> /dev/null; then
        MISSING_TOOLS+=("node")
    fi
    
    if ! command -v npm &> /dev/null; then
        MISSING_TOOLS+=("npm")
    fi
    
    if ! command -v python3 &> /dev/null; then
        MISSING_TOOLS+=("python3")
    fi
    
    if [ ${#MISSING_TOOLS[@]} -ne 0 ]; then
        log_error "Missing required tools: ${MISSING_TOOLS[*]}"
        log_info "Please install the missing tools and run this script again"
        exit 1
    fi
    
    log_success "Development tools check passed"
}

# Setup environment variables
setup_environment() {
    log_info "Setting up environment variables..."
    
    # Create .env file for development
    cat > .env << EOF
# TracSeq 2.0 Development Environment Configuration
NODE_ENV=development
RUST_LOG=debug

# Database Configuration (using SQLite for development without Docker)
DATABASE_URL=sqlite:./dev_database.db
SQLITE_DATABASE_PATH=./dev_database.db

# API Configuration
API_GATEWAY_PORT=8089
BACKEND_PORT=3000
FRONTEND_PORT=5173

# Frontend Configuration
VITE_API_URL=http://localhost:8089
VITE_API_BASE_URL=http://localhost:8089
VITE_WS_URL=ws://localhost:8089/ws

# Service URLs
RAG_SERVICE_URL=http://localhost:8000
AUTH_SERVICE_URL=http://localhost:8001
STORAGE_SERVICE_URL=http://localhost:8002

# JWT Secret (development only)
JWT_SECRET=development-jwt-secret-change-in-production

# Upload Configuration
MAX_UPLOAD_SIZE=100MB
UPLOAD_DIR=./uploads

# Debug Configuration
DEBUG=true
VERBOSE_LOGGING=true
EOF

    log_success "Environment variables configured"
}

# Initialize SQLite database for development
init_database() {
    log_info "Initializing development database..."
    
    # Install sqlite3 if not available
    if ! command -v sqlite3 &> /dev/null; then
        log_warning "SQLite3 not found. Installing..."
        if command -v apt-get &> /dev/null; then
            sudo apt-get update && sudo apt-get install -y sqlite3
        elif command -v yum &> /dev/null; then
            sudo yum install -y sqlite3
        else
            log_error "Cannot install SQLite3 automatically. Please install it manually."
            exit 1
        fi
    fi
    
    # Create database directory
    mkdir -p ./data
    
    # Create SQLite database with basic schema
    sqlite3 ./dev_database.db << 'EOF'
-- Basic schema for development
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'guest',
    status TEXT NOT NULL DEFAULT 'active',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS samples (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name TEXT NOT NULL,
    barcode TEXT UNIQUE NOT NULL,
    sample_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    metadata TEXT DEFAULT '{}',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS templates (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name TEXT NOT NULL,
    description TEXT,
    template_data TEXT NOT NULL DEFAULT '{}',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS spreadsheet_datasets (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    filename TEXT NOT NULL,
    original_filename TEXT NOT NULL,
    file_type TEXT NOT NULL,
    file_size INTEGER,
    total_rows INTEGER DEFAULT 0,
    total_columns INTEGER DEFAULT 0,
    column_headers TEXT DEFAULT '[]',
    upload_status TEXT DEFAULT 'completed',
    uploaded_by TEXT,
    metadata TEXT DEFAULT '{}',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS spreadsheet_records (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    dataset_id TEXT NOT NULL,
    row_number INTEGER NOT NULL,
    row_data TEXT NOT NULL DEFAULT '{}',
    search_text TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (dataset_id) REFERENCES spreadsheet_datasets(id)
);

CREATE TABLE IF NOT EXISTS rag_submissions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    submission_id TEXT UNIQUE NOT NULL,
    filename TEXT NOT NULL,
    file_path TEXT,
    extracted_data TEXT DEFAULT '{}',
    confidence_score REAL DEFAULT 0.0,
    status TEXT DEFAULT 'completed',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Insert a test user
INSERT OR IGNORE INTO users (email, password_hash, first_name, last_name, role) 
VALUES ('admin@tracseq.com', 'test-hash', 'Admin', 'User', 'lab_administrator');

-- Insert sample data
INSERT OR IGNORE INTO samples (name, barcode, sample_type) 
VALUES 
    ('Sample 001', 'TSQ001', 'DNA'),
    ('Sample 002', 'TSQ002', 'RNA'),
    ('Sample 003', 'TSQ003', 'Protein');

-- Insert a sample template
INSERT OR IGNORE INTO templates (name, description, template_data)
VALUES ('Basic Sample Template', 'Basic template for sample submission', '{"fields": ["sample_id", "sample_type", "concentration"]}');

EOF

    log_success "Development database initialized"
}

# Create upload directories
setup_upload_directories() {
    log_info "Setting up upload directories..."
    
    mkdir -p ./uploads
    mkdir -p ./uploads/spreadsheets
    mkdir -p ./uploads/documents
    mkdir -p ./uploads/templates
    
    log_success "Upload directories created"
}

# Install frontend dependencies
setup_frontend() {
    log_info "Setting up frontend..."
    
    cd lims-ui
    
    # Install dependencies if node_modules doesn't exist
    if [ ! -d "node_modules" ]; then
        log_info "Installing frontend dependencies..."
        npm install
    fi
    
    # Create .env.local for frontend
    cat > .env.local << EOF
VITE_API_URL=http://localhost:8089
VITE_API_BASE_URL=http://localhost:8089
VITE_WS_URL=ws://localhost:8089/ws
VITE_DEV_MODE=true
VITE_DEBUG_MODE=true
EOF
    
    cd ..
    log_success "Frontend setup complete"
}

# Create a simple Python API Gateway for development
create_api_gateway() {
    log_info "Creating development API gateway..."
    
    mkdir -p ./dev-services
    
    cat > ./dev-services/api_gateway.py << 'EOF'
#!/usr/bin/env python3
"""
Development API Gateway for TracSeq 2.0
Provides basic API endpoints for testing upload functionality
"""

import json
import os
import sqlite3
import uuid
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List

from fastapi import FastAPI, File, Form, HTTPException, UploadFile
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from pydantic import BaseModel

app = FastAPI(title="TracSeq 2.0 Dev API Gateway", version="1.0.0")

# Enable CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:5173", "http://localhost:3000"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Database connection
DATABASE_PATH = "./dev_database.db"
UPLOAD_DIR = Path("./uploads")

def get_db_connection():
    return sqlite3.connect(DATABASE_PATH)

def dict_factory(cursor, row):
    d = {}
    for idx, col in enumerate(cursor.description):
        d[col[0]] = row[idx]
    return d

class UploadResponse(BaseModel):
    success: bool
    data: List[Dict[str, Any]] = None
    message: str

@app.get("/health")
async def health_check():
    return {"status": "healthy", "service": "dev-api-gateway"}

@app.get("/api/dashboard/stats")
async def get_dashboard_stats():
    conn = get_db_connection()
    conn.row_factory = dict_factory
    cursor = conn.cursor()
    
    # Get counts
    cursor.execute("SELECT COUNT(*) as count FROM samples")
    samples_count = cursor.fetchone()["count"]
    
    cursor.execute("SELECT COUNT(*) as count FROM templates")
    templates_count = cursor.fetchone()["count"]
    
    cursor.execute("SELECT COUNT(*) as count FROM spreadsheet_datasets")
    datasets_count = cursor.fetchone()["count"]
    
    cursor.execute("SELECT COUNT(*) as count FROM rag_submissions")
    rag_count = cursor.fetchone()["count"]
    
    conn.close()
    
    return {
        "samples": {"total": samples_count, "active": samples_count},
        "templates": {"total": templates_count},
        "datasets": {"total": datasets_count},
        "rag_submissions": {"total": rag_count},
        "storage": {"locations": 5, "capacity": 85.5}
    }

@app.get("/api/samples")
async def get_samples():
    conn = get_db_connection()
    conn.row_factory = dict_factory
    cursor = conn.cursor()
    
    cursor.execute("SELECT * FROM samples ORDER BY created_at DESC")
    samples = cursor.fetchall()
    conn.close()
    
    return samples

@app.get("/api/templates")
async def get_templates():
    conn = get_db_connection()
    conn.row_factory = dict_factory
    cursor = conn.cursor()
    
    cursor.execute("SELECT * FROM templates ORDER BY created_at DESC")
    templates = cursor.fetchall()
    conn.close()
    
    return templates

@app.get("/api/spreadsheets/datasets")
async def get_datasets():
    conn = get_db_connection()
    conn.row_factory = dict_factory
    cursor = conn.cursor()
    
    cursor.execute("SELECT * FROM spreadsheet_datasets ORDER BY created_at DESC")
    datasets = cursor.fetchall()
    conn.close()
    
    return datasets

@app.get("/api/rag/submissions")
async def get_rag_submissions():
    conn = get_db_connection()
    conn.row_factory = dict_factory
    cursor = conn.cursor()
    
    cursor.execute("SELECT * FROM rag_submissions ORDER BY created_at DESC")
    submissions = cursor.fetchall()
    conn.close()
    
    return submissions

@app.post("/api/spreadsheets/upload-multiple")
async def upload_spreadsheet(
    file: UploadFile = File(...),
    uploaded_by: str = Form(None),
    selected_sheets: str = Form(None)
):
    """Upload and process spreadsheet files"""
    try:
        # Validate file type
        allowed_types = ['.csv', '.xlsx', '.xls']
        file_ext = '.' + file.filename.split('.')[-1].lower()
        
        if file_ext not in allowed_types:
            raise HTTPException(status_code=400, detail=f"Unsupported file type: {file_ext}")
        
        # Create upload directory
        UPLOAD_DIR.mkdir(exist_ok=True)
        spreadsheet_dir = UPLOAD_DIR / "spreadsheets"
        spreadsheet_dir.mkdir(exist_ok=True)
        
        # Save file
        file_id = str(uuid.uuid4())
        saved_filename = f"{file_id}_{file.filename}"
        file_path = spreadsheet_dir / saved_filename
        
        content = await file.read()
        with open(file_path, "wb") as f:
            f.write(content)
        
        # Parse selected sheets
        sheets = []
        if selected_sheets:
            try:
                sheets = json.loads(selected_sheets)
            except:
                sheets = [selected_sheets]
        
        # Create database entries
        conn = get_db_connection()
        cursor = conn.cursor()
        
        datasets = []
        for i, sheet in enumerate(sheets if sheets else [""]):
            dataset_id = str(uuid.uuid4())
            
            cursor.execute("""
                INSERT INTO spreadsheet_datasets 
                (id, filename, original_filename, file_type, file_size, uploaded_by, metadata)
                VALUES (?, ?, ?, ?, ?, ?, ?)
            """, (
                dataset_id,
                saved_filename,
                file.filename,
                file_ext.replace('.', ''),
                len(content),
                uploaded_by or "anonymous",
                json.dumps({"sheet_name": sheet} if sheet else {})
            ))
            
            datasets.append({
                "id": dataset_id,
                "filename": saved_filename,
                "original_filename": file.filename,
                "file_type": file_ext.replace('.', ''),
                "sheet_name": sheet
            })
        
        conn.commit()
        conn.close()
        
        return UploadResponse(
            success=True,
            data=datasets,
            message=f"Successfully uploaded {file.filename}"
        )
        
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/api/rag/process")
async def process_rag_document(file: UploadFile = File(...)):
    """Process RAG documents"""
    try:
        # Create upload directory
        UPLOAD_DIR.mkdir(exist_ok=True)
        docs_dir = UPLOAD_DIR / "documents"
        docs_dir.mkdir(exist_ok=True)
        
        # Save file
        file_id = str(uuid.uuid4())
        saved_filename = f"{file_id}_{file.filename}"
        file_path = docs_dir / saved_filename
        
        content = await file.read()
        with open(file_path, "wb") as f:
            f.write(content)
        
        # Create database entry
        conn = get_db_connection()
        cursor = conn.cursor()
        
        submission_id = f"RAG-{str(uuid.uuid4())[:8]}"
        
        cursor.execute("""
            INSERT INTO rag_submissions 
            (submission_id, filename, file_path, extracted_data, confidence_score)
            VALUES (?, ?, ?, ?, ?)
        """, (
            submission_id,
            file.filename,
            str(file_path),
            json.dumps({"status": "processed", "samples_found": 1}),
            0.85
        ))
        
        conn.commit()
        conn.close()
        
        return {
            "success": True,
            "submission_id": submission_id,
            "confidence_score": 0.85,
            "samples_found": 1,
            "message": "Document processed successfully"
        }
        
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/api/templates/upload")
async def upload_template(file: UploadFile = File(...)):
    """Upload template files"""
    try:
        # Create upload directory
        UPLOAD_DIR.mkdir(exist_ok=True)
        templates_dir = UPLOAD_DIR / "templates"
        templates_dir.mkdir(exist_ok=True)
        
        # Save file
        file_id = str(uuid.uuid4())
        saved_filename = f"{file_id}_{file.filename}"
        file_path = templates_dir / saved_filename
        
        content = await file.read()
        with open(file_path, "wb") as f:
            f.write(content)
        
        # Create database entry
        conn = get_db_connection()
        cursor = conn.cursor()
        
        template_id = str(uuid.uuid4())
        
        cursor.execute("""
            INSERT INTO templates 
            (id, name, description, template_data)
            VALUES (?, ?, ?, ?)
        """, (
            template_id,
            file.filename.replace('.xlsx', '').replace('.csv', ''),
            f"Uploaded template: {file.filename}",
            json.dumps({"file_path": str(file_path), "original_name": file.filename})
        ))
        
        conn.commit()
        conn.close()
        
        return {
            "success": True,
            "id": template_id,
            "name": file.filename,
            "message": "Template uploaded successfully"
        }
        
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8089)
EOF

    # Make it executable
    chmod +x ./dev-services/api_gateway.py
    
    log_success "Development API gateway created"
}

# Create startup script
create_startup_script() {
    log_info "Creating startup script..."
    
    cat > start_dev_services.sh << 'EOF'
#!/bin/bash

# Start TracSeq 2.0 Development Services

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Kill any existing services
pkill -f "api_gateway.py" 2>/dev/null || true
pkill -f "vite" 2>/dev/null || true

log_info "Starting TracSeq 2.0 Development Environment..."

# Start API Gateway
log_info "Starting API Gateway on port 8089..."
cd dev-services
python3 api_gateway.py &
API_GATEWAY_PID=$!
cd ..

# Wait for API Gateway to start
sleep 3

# Start Frontend
log_info "Starting Frontend on port 5173..."
cd lims-ui
npm run dev &
FRONTEND_PID=$!
cd ..

log_success "Services started successfully!"
echo ""
echo "🌐 Frontend: http://localhost:5173"
echo "🔧 API Gateway: http://localhost:8089"
echo "📊 Health Check: http://localhost:8089/health"
echo ""
echo "To stop services, run: ./stop_dev_services.sh"

# Save PIDs for cleanup
echo "$API_GATEWAY_PID" > .api_gateway.pid
echo "$FRONTEND_PID" > .frontend.pid

# Wait for user input to keep script running
echo "Press Ctrl+C to stop all services..."
trap 'kill $API_GATEWAY_PID $FRONTEND_PID 2>/dev/null; exit' INT
wait
EOF

    chmod +x start_dev_services.sh
    
    # Create stop script
    cat > stop_dev_services.sh << 'EOF'
#!/bin/bash

echo "Stopping TracSeq 2.0 Development Services..."

# Kill by PID files
if [ -f .api_gateway.pid ]; then
    kill $(cat .api_gateway.pid) 2>/dev/null || true
    rm .api_gateway.pid
fi

if [ -f .frontend.pid ]; then
    kill $(cat .frontend.pid) 2>/dev/null || true
    rm .frontend.pid
fi

# Kill by process name as backup
pkill -f "api_gateway.py" 2>/dev/null || true
pkill -f "vite" 2>/dev/null || true

echo "Services stopped."
EOF

    chmod +x stop_dev_services.sh
    
    log_success "Startup scripts created"
}

# Install Python dependencies
install_python_deps() {
    log_info "Installing Python dependencies..."
    
    # Check if pip is available
    if ! command -v pip3 &> /dev/null; then
        log_warning "pip3 not found. Installing..."
        if command -v apt-get &> /dev/null; then
            sudo apt-get update && sudo apt-get install -y python3-pip
        fi
    fi
    
    # Install required packages
    pip3 install fastapi uvicorn python-multipart || {
        log_warning "Failed to install with pip3, trying user install..."
        pip3 install --user fastapi uvicorn python-multipart
    }
    
    log_success "Python dependencies installed"
}

# Main execution
main() {
    echo "🚀 TracSeq 2.0 Development Environment Setup"
    echo "============================================="
    echo ""
    
    check_environment
    setup_environment
    init_database
    setup_upload_directories
    setup_frontend
    install_python_deps
    create_api_gateway
    create_startup_script
    
    echo ""
    log_success "🎉 Development environment setup complete!"
    echo ""
    echo "Next steps:"
    echo "1. Run './start_dev_services.sh' to start all services"
    echo "2. Open http://localhost:5173 in your browser"
    echo "3. Test upload functionality:"
    echo "   - Go to Templates page and try uploading a spreadsheet"
    echo "   - Go to RAG Submissions and try uploading a document"
    echo "   - Check the database with: sqlite3 dev_database.db '.tables'"
    echo ""
    echo "To stop services: ./stop_dev_services.sh"
}

main "$@"