#!/usr/bin/env python3
"""
Simple API Gateway for TracSeq 2.0 Development
Provides basic endpoints for frontend development without full microservices.
"""

from fastapi import FastAPI, HTTPException, UploadFile, File
from fastapi.middleware.cors import CORSMiddleware
from datetime import datetime
import uvicorn
import json
import uuid
import asyncpg
import os
from typing import Optional, List, Dict, Any

app = FastAPI(title="TracSeq 2.0 Simple API Gateway", version="1.0.0")

# Enable CORS for frontend development
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:5173", "http://localhost:3000"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Database configuration for PostgreSQL connection
DATABASE_URL = "postgresql://tracseq_admin:tracseq_secure_password@localhost:5433/tracseq_main"
db_pool: Optional[asyncpg.Pool] = None

async def get_db_pool() -> asyncpg.Pool:
    """Get database connection pool"""
    global db_pool
    if db_pool is None:
        try:
            db_pool = await asyncpg.create_pool(
                DATABASE_URL,
                min_size=5,
                max_size=20,
                command_timeout=60,
                server_settings={'application_name': 'tracseq_api_gateway'}
            )
            print(f"✅ Connected to PostgreSQL database")
        except Exception as e:
            print(f"❌ Failed to connect to database: {e}")
            raise
    return db_pool

async def init_database():
    """Initialize database connection on startup"""
    await get_db_pool()

async def close_database():
    """Close database connection on shutdown"""
    global db_pool
    if db_pool:
        await db_pool.close()
        print("✅ Database connection closed")

# In-memory cache for better performance (optional fallback)
CACHE_ENABLED = True

# Mock data for development
MOCK_SAMPLES = [
    {
        "id": "smp-001",
        "name": "Sample Alpha-001",
        "barcode": "LAB-20240101-001",
        "location": "Zone A-1",
        "status": "Pending",
        "created_at": datetime.now().isoformat(),
        "metadata": {
            "confidence_score": 0.95,
            "processing_time": 2.1,
            "source_document": "lab_submission_form_001.pdf",
            "submitter_name": "Dr. Sarah Wilson",
            "submitter_email": "sarah.wilson@lab.edu",
            "extraction_method": "manual",
            "sample_type": "DNA",
            "volume": 150,
            "concentration": 25.7
        }
    },
    {
        "id": "smp-002",
        "name": "AI-Generated Sample Beta",
        "barcode": "LAB-20240101-002",
        "location": "Zone B-2",
        "status": "InStorage",
        "created_at": datetime.now().isoformat(),
        "metadata": {
            "confidence_score": 0.92,
            "processing_time": 3.5,
            "source_document": "lab_submission_form_001.pdf",
            "submitter_name": "Dr. Jane Smith",
            "submitter_email": "jane.smith@lab.edu",
            "rag_submission_id": "rag-001",
            "extraction_method": "ai_rag",
            "sample_type": "RNA",
            "volume": 100,
            "concentration": 22.3,
            "validation_warnings": [],
            "extraction_warnings": ["Low confidence in volume measurement"]
        }
    }
]

MOCK_RAG_SUBMISSIONS = [
    {
        "id": "rag-001",
        "filename": "lab_submission_form_001.pdf",
        "status": "completed",
        "uploadedAt": datetime.now().isoformat(),
        "processedAt": datetime.now().isoformat(),
        "confidence": 0.92,
        "extractedSamples": 1,
        "errors": [],
        "metadata": {
            "submission_id": "SUB-20240101-001",
            "source_document": "lab_submission_form_001.pdf",
            "submitter_name": "Dr. Jane Smith",
            "submitter_email": "jane.smith@lab.edu",
            "confidence_score": 0.92,
            "processing_time": 3.5,
            "samples_created": 1
        }
    },
    {
        "id": "rag-002",
        "filename": "research_protocol_v2.pdf",
        "status": "processing",
        "uploadedAt": datetime.now().isoformat(),
        "confidence": 0.78,
        "extractedSamples": 0,
        "errors": [],
        "metadata": {
            "submission_id": "SUB-20240101-002",
            "source_document": "research_protocol_v2.pdf",
            "submitter_name": "Dr. Bob Johnson",
            "submitter_email": "bob.johnson@research.org",
            "confidence_score": 0.78,
            "processing_time": 5.2,
            "samples_created": 0
        }
    }
]

MOCK_TEMPLATES = [
    {
        "id": "tpl-001",
        "name": "Standard Sample Submission Form",
        "description": "Standard laboratory sample submission template",
        "file_type": "PDF",
        "version": "1.0",
        "created_at": datetime.now().isoformat(),
        "updated_at": datetime.now().isoformat()
    },
    {
        "id": "tpl-002",
        "name": "Research Protocol Template",
        "description": "Template for research protocol documentation",
        "file_type": "DOCX",
        "version": "2.1",
        "created_at": datetime.now().isoformat(),
        "updated_at": datetime.now().isoformat()
    },
    {
        "id": "tpl-003",
        "name": "Quality Control Checklist",
        "description": "Quality control and validation checklist",
        "file_type": "XLSX",
        "version": "1.5",
        "created_at": datetime.now().isoformat(),
        "updated_at": datetime.now().isoformat()
    }
]

MOCK_SEQUENCING_JOBS = [
    {
        "id": "job-001",
        "name": "Genomic Analysis Batch #42",
        "status": "in_progress",
        "platform": "Illumina NovaSeq",
        "samples_count": 24,
        "priority": "high",
        "estimated_completion": "2024-01-15T14:30:00Z",
        "created_at": datetime.now().isoformat()
    },
    {
        "id": "job-002",
        "name": "RNA-Seq Project Delta",
        "status": "completed",
        "platform": "Oxford Nanopore",
        "samples_count": 12,
        "priority": "normal",
        "completed_at": "2024-01-10T09:15:00Z",
        "created_at": datetime.now().isoformat()
    }
]

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {"status": "healthy", "timestamp": datetime.now().isoformat(), "service": "api-gateway"}

@app.get("/api/samples")
async def get_samples(extraction_method: str = ""):
    """Get all samples, optionally filtered by extraction method"""
    if extraction_method == "ai_rag":
        return [s for s in MOCK_SAMPLES if s["metadata"].get("extraction_method") == "ai_rag"]
    return MOCK_SAMPLES

@app.get("/api/rag/samples")
async def get_rag_samples():
    """Get AI-generated samples"""
    return [s for s in MOCK_SAMPLES if s["metadata"].get("extraction_method") == "ai_rag"]

@app.get("/api/rag/submissions")
async def get_rag_submissions():
    """Get RAG submissions"""
    return MOCK_RAG_SUBMISSIONS

@app.get("/api/rag/submissions/{submission_id}")
async def get_rag_submission(submission_id: str):
    """Get a specific RAG submission"""
    submission = next((s for s in MOCK_RAG_SUBMISSIONS if s["id"] == submission_id), None)
    if not submission:
        raise HTTPException(status_code=404, detail="Submission not found")
    return submission

@app.post("/api/rag/process")
async def process_document():
    """Process uploaded document"""
    return {
        "success": True,
        "samples": [
            {
                "name": f"AI-Extracted Sample {datetime.now().strftime('%H%M%S')}",
                "barcode": f"AUTO-{datetime.now().strftime('%H%M%S')}",
                "location": "Zone-AI-1",
                "metadata": {
                    "extraction_method": "ai_rag",
                    "confidence_score": 0.88,
                    "source_document": "uploaded_document.pdf",
                    "sample_type": "RNA",
                    "volume": 120,
                    "concentration": 18.5
                }
            }
        ],
        "confidence_score": 0.88,
        "validation_warnings": ["Sample type inference has medium confidence"],
        "processing_time": 2.3,
        "source_document": "uploaded_document.pdf"
    }

@app.post("/api/samples/rag/query")
async def rag_query(request: dict):
    """Handle RAG queries with intelligent, helpful responses"""
    query = request.get("query", "").lower()
    
    # Provide detailed, helpful responses based on query intent
    if any(word in query for word in ["submit", "upload", "new sample", "document processing", "ai document"]):
        response = """To submit a new sample using AI document processing:

1. **Navigate to AI-Powered Document Submissions** - Click on the "AI-Powered Document Submissions" page from the main navigation
2. **Upload Your Document** - Drag and drop or click to upload your laboratory document (PDF, DOCX, or TXT)
3. **Adjust Settings** - Set your confidence threshold (default 0.8) and choose whether to auto-create samples
4. **Process Document** - Click "Process & Extract" to let AI extract sample information
5. **Review Results** - Check the extracted data and confidence scores
6. **Create Samples** - Confirm to automatically create sample records

You can also use "Preview" mode to see what would be extracted without creating samples first. The AI will extract details like sample names, types, concentrations, and storage requirements from your documents."""
        
    elif any(word in query for word in ["rag", "ai samples", "view samples", "ai-generated"]):
        response = """To view AI-generated samples:

1. **Go to RAG Samples** - Navigate to "AI-Generated Sample Records" page
2. **Browse Samples** - View all samples created through AI document processing
3. **Filter & Search** - Use filters for status, confidence level, or search by name/barcode
4. **View Details** - Click the eye icon to see detailed extraction information
5. **Check Source** - See which document each sample was extracted from

AI-generated samples show confidence scores, source documents, and any extraction warnings. You can edit these samples just like manually created ones."""
        
    elif any(word in query for word in ["template", "form", "document format"]):
        response = f"""Available templates for sample submission:

• **Standard Sample Submission Form** (PDF) - Most common laboratory submission template
• **Research Protocol Template** (DOCX) - For research protocol documentation  
• **Quality Control Checklist** (XLSX) - Quality control and validation checklist

You can download these templates from the Templates page. The AI works best with structured documents that include sample details like names, types, volumes, concentrations, and storage requirements."""
        
    elif any(word in query for word in ["sequencing", "jobs", "platform"]):
        response = f"""Current sequencing status:

• **Active Jobs**: {len([j for j in MOCK_SEQUENCING_JOBS if j['status'] == 'in_progress'])} job(s) in progress
• **Completed Jobs**: {len([j for j in MOCK_SEQUENCING_JOBS if j['status'] == 'completed'])} job(s) completed

To create a new sequencing job:
1. Go to the Sequencing page
2. Click "Create New Job"
3. Select your platform (Illumina NovaSeq, Oxford Nanopore, etc.)
4. Add samples and set priority
5. Generate sample sheets when ready"""
        
    elif any(word in query for word in ["help", "how to", "guide", "tutorial"]):
        response = """TracSeq 2.0 Quick Start Guide:

**AI Document Processing**:
• Upload lab documents to automatically extract sample information
• Use confidence thresholds to control extraction quality
• Preview before creating samples

**Sample Management**:
• Track samples through their entire lifecycle
• View storage locations and environmental conditions
• Monitor quality control status

**Sequencing Workflows**:
• Create and manage sequencing jobs
• Generate sample sheets for instruments
• Track job progress and completion

**Templates & Standards**:
• Download standard forms for consistent submissions
• Use structured documents for best AI extraction results

Need specific help? Ask about "submitting samples", "viewing AI samples", "sequencing jobs", or "templates"."""
        
    elif any(word in query for word in ["confidence", "accuracy", "quality"]):
        response = f"""AI Extraction Quality Information:

**Current System Stats**:
• Total Samples: {len(MOCK_SAMPLES)} (1 AI-generated, 1 manual)
• Average AI Confidence: 92% (high quality)
• Processing Time: ~3.5 seconds per document

**Confidence Levels**:
• **High (≥80%)**: Auto-create samples safely
• **Medium (60-79%)**: Review before creating
• **Low (<60%)**: Manual verification required

**Quality Tips**:
• Use clear, structured documents
• Include complete sample information
• Check extraction warnings before confirming
• Adjust confidence threshold based on document quality"""
        
    elif any(word in query for word in ["storage", "location", "temperature"]):
        response = """Sample Storage Information:

**Current Storage Status**:
• Total Storage Zones: 5 active locations
• Ultra-Low Freezer (-80°C): 287/500 occupied (57%)
• Standard Freezer (-20°C): 156/300 occupied (52%)
• Refrigerated Storage (4°C): 89/200 occupied (45%)
• Room Temperature: 34/150 occupied (23%)
• Incubator (37°C): 23/100 occupied (23%)

**Storage Management**:
• Real-time capacity monitoring
• Temperature alerts and notifications
• Automated sample location assignment
• Equipment status tracking (all operational)

Storage locations are automatically assigned based on sample requirements extracted from your documents or manually specified during sample creation."""
        
    elif any(word in query for word in ["reports", "export", "analytics", "dashboard"]):
        response = """Reports & Analytics:

**Available Reports**:
• Sample Summary Report - All samples with key metrics
• Storage Audit Report - Detailed utilization and audit trail
• Sequencing Performance Metrics - Platform analysis

**Report Formats**:
• CSV - Data exports for analysis
• XLSX - Formatted spreadsheets 
• PDF - Formatted reports for sharing

**Dashboard Features**:
• Real-time system statistics
• Sample processing metrics
• Storage utilization charts
• Performance monitoring

Access the Reports section or Dashboard to generate custom reports and view analytics."""
        
    elif any(word in query for word in ["user", "account", "team", "permissions"]):
        response = """User Management:

**Current Users**:
• Lab Administrator (admin@lab.local) - Full access
• Dr. Sarah Wilson (scientist@lab.local) - Principal Investigator  
• John Smith (tech@lab.local) - Lab Technician

**User Roles Available**:
• Lab Administrator - Full system access
• Principal Investigator - Research project management
• Lab Technician - Sample processing and data entry
• Data Analyst - Read-only access to reports
• Research Scientist - Sample management and analysis

**Permissions**:
• Role-based access control
• Departmental restrictions
• Activity logging and audit trails

Contact your administrator to manage user accounts and permissions."""
        
    elif any(word in query for word in ["spreadsheet", "dataset", "data export", "excel"]):
        response = """Spreadsheet & Data Management:

**Available Datasets**:
• Sample Tracking Database (2.4 MB, 1,247 rows)
• Sequencing Results Archive (856 KB, 892 rows)  
• Template Usage Statistics (1.1 MB, 456 rows)

**Data Export Options**:
• Download datasets in XLSX or CSV format
• Filter and export specific sample subsets
• Generate custom reports from templates

**Spreadsheet Features**:
• Import/export laboratory data
• Version control for dataset changes
• Real-time collaboration on shared spreadsheets
• Automated data validation and quality checks

Access the Spreadsheets page to view, download, or create new datasets."""
        
    elif any(word in query for word in ["login", "access", "permission", "account"]):
        response = """Access & Authentication:

**Test Login Credentials**:
• Email: admin@lab.local (or any email)
• Password: password

**User Roles Available**:
• Lab Administrator (full access)
• Principal Investigator
• Research Scientist
• Lab Technician
• Data Analyst

Each role has different permissions for creating, viewing, and modifying samples, templates, and sequencing jobs."""
        
    else:
        # Default helpful response
        response = f"""TracSeq 2.0 Laboratory Management System

**System Overview**:
• Total Samples: {len(MOCK_SAMPLES)}
• AI Submissions: {len(MOCK_RAG_SUBMISSIONS)}
• Sequencing Jobs: {len(MOCK_SEQUENCING_JOBS)}
• Templates Available: {len(MOCK_TEMPLATES)}

**What you can ask about**:
• "How to submit new samples" - AI document processing guide
• "View AI samples" - Managing AI-generated samples  
• "Sequencing jobs" - Creating and managing sequencing
• "Templates" - Available document templates
• "Storage locations" - Sample storage information
• "Login help" - Access and authentication

Try asking: "How do I submit a new sample?" or "Show me AI-generated samples" for specific guidance."""
    
    return {
        "success": True,
        "response": response,
        "confidence": 0.95,
        "sources": ["tracseq-lab-assistant"],
        "query_type": "instructional_help"
    }

@app.get("/api/templates")
async def get_templates():
    """Get all templates"""
    return MOCK_TEMPLATES

@app.get("/api/templates/{template_id}/data")
async def get_template_data(template_id: str):
    """Get template data/structure for viewing"""
    template = next((t for t in MOCK_TEMPLATES if t["id"] == template_id), None)
    if not template:
        raise HTTPException(status_code=404, detail="Template not found")
    
    # Return mock spreadsheet data structure
    return {
        "template": template,
        "data": {
            "sheet_names": ["Sample Information", "Storage Requirements"],
            "sheets": [
                {
                    "name": "Sample Information",
                    "headers": ["Sample Name", "Sample Type", "Volume (µL)", "Concentration (ng/µL)", "Submitter"],
                    "rows": [
                        ["Sample-001", "DNA", "150", "25.7", "Dr. Sarah Wilson"],
                        ["Sample-002", "RNA", "100", "22.3", "Dr. Jane Smith"],
                        ["Sample-003", "Protein", "200", "15.8", "Dr. Bob Johnson"]
                    ],
                    "total_rows": 3,
                    "total_columns": 5
                },
                {
                    "name": "Storage Requirements",
                    "headers": ["Sample Name", "Temperature", "Container Type", "Special Requirements"],
                    "rows": [
                        ["Sample-001", "-80°C", "Cryotube", "Avoid freeze-thaw"],
                        ["Sample-002", "-20°C", "PCR Tube", "RNase-free environment"],
                        ["Sample-003", "4°C", "Protein LoBind", "Keep on ice"]
                    ],
                    "total_rows": 3,
                    "total_columns": 4
                }
            ]
        }
    }

@app.post("/api/templates/upload")
async def upload_template():
    """Upload and process a new template"""
    # Mock template upload processing
    new_template = {
        "id": f"tpl-{len(MOCK_TEMPLATES) + 1:03d}",
        "name": f"Uploaded Template {len(MOCK_TEMPLATES) + 1}",
        "description": "User uploaded template",
        "file_type": "XLSX",
        "version": "1.0",
        "created_at": datetime.now().isoformat(),
        "updated_at": datetime.now().isoformat()
    }
    MOCK_TEMPLATES.append(new_template)
    
    return {
        "success": True,
        "template": new_template,
        "message": "Template uploaded and processed successfully"
    }

@app.delete("/api/templates/{template_id}")
async def delete_template(template_id: str):
    """Delete a template"""
    global MOCK_TEMPLATES
    template = next((t for t in MOCK_TEMPLATES if t["id"] == template_id), None)
    if not template:
        raise HTTPException(status_code=404, detail="Template not found")
    
    MOCK_TEMPLATES = [t for t in MOCK_TEMPLATES if t["id"] != template_id]
    return {"success": True, "message": "Template deleted successfully"}

@app.get("/api/users/me")
async def get_current_user():
    """Get current user information"""
    # In a real app, this would validate the JWT token
    return {
        "id": "user-001",
        "email": "admin@lab.local",
        "first_name": "Lab",
        "last_name": "Administrator", 
        "name": "Lab Administrator",
        "role": "lab_administrator",
        "status": "active",
        "department": "Laboratory Management",
        "lab_affiliation": "TracSeq Laboratory",
        "position": "Administrator",
        "email_verified": True,
        "created_at": datetime.now().isoformat(),
        "permissions": ["read", "write", "admin"]
    }

@app.get("/api/dashboard/stats")
async def get_dashboard_stats():
    """Get dashboard statistics"""
    return {
        "total_samples": len(MOCK_SAMPLES),
        "samples_in_storage": len([s for s in MOCK_SAMPLES if s["status"] == "InStorage"]),
        "samples_pending": len([s for s in MOCK_SAMPLES if s["status"] == "Pending"]),
        "active_sequencing_jobs": len([j for j in MOCK_SEQUENCING_JOBS if j["status"] == "in_progress"]),
        "completed_sequencing_jobs": len([j for j in MOCK_SEQUENCING_JOBS if j["status"] == "completed"]),
        "templates_available": len(MOCK_TEMPLATES),
        "rag_submissions": len(MOCK_RAG_SUBMISSIONS),
        "processing_time": {
            "average": 2.8,
            "median": 2.9,
            "bottlenecks": ["Document parsing", "Quality validation"]
        },
        "throughput": {
            "samples_per_day": 12,
            "documents_processed": 8,
            "efficiency_score": 0.87
        }
    }

@app.get("/api/sequencing/jobs")
async def get_sequencing_jobs():
    """Get sequencing jobs (compatible with frontend expectations)"""
    return MOCK_SEQUENCING_JOBS

@app.get("/api/sequencing/jobs/{job_id}")
async def get_sequencing_job(job_id: str):
    """Get a specific sequencing job"""
    job = next((j for j in MOCK_SEQUENCING_JOBS if j["id"] == job_id), None)
    if not job:
        raise HTTPException(status_code=404, detail="Sequencing job not found")
    return job

@app.post("/api/sequencing/jobs")
async def create_sequencing_job(request: dict):
    """Create a new sequencing job"""
    new_job = {
        "id": f"job-{len(MOCK_SEQUENCING_JOBS) + 1:03d}",
        "name": request.get("name", "New Sequencing Job"),
        "status": "pending",
        "platform": request.get("platform", "Illumina NovaSeq"),
        "samples_count": request.get("samples_count", 0),
        "priority": request.get("priority", "normal"),
        "created_at": datetime.now().isoformat()
    }
    MOCK_SEQUENCING_JOBS.append(new_job)
    return new_job

@app.post("/api/sequencing/jobs/{job_id}/sample-sheet")
async def generate_sample_sheet(job_id: str):
    """Generate sample sheet for sequencing job"""
    job = next((j for j in MOCK_SEQUENCING_JOBS if j["id"] == job_id), None)
    if not job:
        raise HTTPException(status_code=404, detail="Sequencing job not found")
    
    return {
        "success": True,
        "sample_sheet_url": f"/api/sequencing/jobs/{job_id}/sample-sheet.csv",
        "samples_included": job["samples_count"],
        "generated_at": datetime.now().isoformat()
    }

@app.patch("/api/sequencing/jobs/{job_id}")
async def update_sequencing_job(job_id: str, request: dict):
    """Update sequencing job status"""
    job = next((j for j in MOCK_SEQUENCING_JOBS if j["id"] == job_id), None)
    if not job:
        raise HTTPException(status_code=404, detail="Sequencing job not found")
    
    # Update job with new data
    for key, value in request.items():
        if key in job:
            job[key] = value
    
    job["updated_at"] = datetime.now().isoformat()
    return job

@app.get("/api/spreadsheets/datasets")
async def get_spreadsheet_datasets():
    """Get available spreadsheet datasets from PostgreSQL database"""
    try:
        pool = await get_db_pool()
        async with pool.acquire() as connection:
            # Get spreadsheet datasets from the database
            rows = await connection.fetch("""
                SELECT 
                    id,
                    name,
                    filename,
                    file_type,
                    file_size,
                    total_rows,
                    total_columns,
                    upload_status as status,
                    description,
                    uploaded_by,
                    created_at,
                    updated_at,
                    metadata
                FROM spreadsheet_datasets 
                WHERE upload_status IN ('completed', 'processing', 'pending')
                ORDER BY created_at DESC
            """)
            
            # Convert database rows to API format
            datasets = []
            for row in rows:
                # Format file size
                file_size_bytes = row['file_size'] or 0
                if file_size_bytes > 1024 * 1024:
                    size_str = f"{file_size_bytes / (1024 * 1024):.1f} MB"
                elif file_size_bytes > 1024:
                    size_str = f"{file_size_bytes / 1024:.1f} KB"
                else:
                    size_str = f"{file_size_bytes} B"
                
                # Parse metadata if available
                metadata = {}
                if row['metadata']:
                    try:
                        metadata = json.loads(row['metadata']) if isinstance(row['metadata'], str) else row['metadata']
                    except (json.JSONDecodeError, TypeError):
                        metadata = {}
                
                dataset = {
                    "id": str(row['id']),
                    "name": row['name'],
                    "description": row['description'] or "User uploaded spreadsheet",
                    "filename": row['filename'],
                    "file_type": row['file_type'],
                    "size": size_str,
                    "last_modified": row['updated_at'].isoformat() if row['updated_at'] else row['created_at'].isoformat(),
                    "status": row['status'],
                    "row_count": row['total_rows'] or 0,
                    "column_count": row['total_columns'] or 0,
                    "sheets": metadata.get("sheets", ["Data"]),
                    "created_by": str(row['uploaded_by']) if row['uploaded_by'] else "Unknown",
                    "created_at": row['created_at'].isoformat(),
                    "upload_method": metadata.get("upload_method", "database")
                }
                datasets.append(dataset)
            
            # Add mock datasets for demonstration (can be removed in production)
            mock_datasets = [
                {
                    "id": "dataset-mock-001",
                    "name": "Sample Tracking Database",
                    "description": "Comprehensive sample tracking and metadata",
                    "file_type": "XLSX",
                    "size": "2.4 MB",
                    "last_modified": datetime.now().isoformat(),
                    "status": "active",
                    "row_count": 1247,
                    "column_count": 23,
                    "sheets": ["Samples", "Storage", "QC Results"],
                    "created_by": "Dr. Sarah Wilson",
                    "created_at": datetime.now().isoformat(),
                    "upload_method": "system"
                },
                {
                    "id": "dataset-mock-002", 
                    "name": "Sequencing Results Archive",
                    "description": "Historical sequencing job results and metrics",
                    "file_type": "CSV",
                    "size": "856 KB",
                    "last_modified": datetime.now().isoformat(),
                    "status": "active",
                    "row_count": 892,
                    "column_count": 15,
                    "sheets": ["Results"],
                    "created_by": "Dr. Jane Smith",
                    "created_at": datetime.now().isoformat(),
                    "upload_method": "system"
                },
                {
                    "id": "dataset-mock-003",
                    "name": "Template Usage Statistics",
                    "description": "Analytics on template usage and effectiveness",
                    "file_type": "XLSX",
                    "size": "1.1 MB", 
                    "last_modified": datetime.now().isoformat(),
                    "status": "active",
                    "row_count": 456,
                    "column_count": 12,
                    "sheets": ["Usage Stats", "Templates", "Metrics"],
                    "created_by": "Lab Administrator",
                    "created_at": datetime.now().isoformat(),
                    "upload_method": "system"
                }
            ]
            
            # Combine database datasets with mock datasets
            all_datasets = datasets + mock_datasets
            return all_datasets
            
    except Exception as e:
        print(f"Database error in get_spreadsheet_datasets: {e}")
        # Return mock data if database fails
        return [
            {
                "id": "dataset-error-001",
                "name": "Sample Tracking Database (DB Error)",
                "description": "Mock data - database connection failed",
                "file_type": "XLSX",
                "size": "2.4 MB",
                "last_modified": datetime.now().isoformat(),
                "status": "active",
                "row_count": 1247,
                "column_count": 23,
                "sheets": ["Samples", "Storage", "QC Results"],
                "created_by": "Dr. Sarah Wilson",
                "created_at": datetime.now().isoformat(),
                "upload_method": "fallback"
            }
        ]

@app.get("/api/spreadsheets/datasets/{dataset_id}")
async def get_spreadsheet_dataset(dataset_id: str):
    """Get a specific spreadsheet dataset"""
    datasets = await get_spreadsheet_datasets()
    dataset = next((d for d in datasets if d["id"] == dataset_id), None)
    if not dataset:
        raise HTTPException(status_code=404, detail="Dataset not found")
    return dataset

@app.post("/api/spreadsheets/datasets")
async def create_spreadsheet_dataset(request: dict):
    """Create a new spreadsheet dataset"""
    return {
        "id": f"dataset-{datetime.now().strftime('%H%M%S')}",
        "name": request.get("name", "New Dataset"),
        "description": request.get("description", ""),
        "file_type": "XLSX",
        "size": "0 KB",
        "last_modified": datetime.now().isoformat(),
        "status": "active",
        "row_count": 0,
        "column_count": 0,
        "created_by": "Lab Administrator",
        "created_at": datetime.now().isoformat()
    }

@app.get("/api/spreadsheets/filters")
async def get_spreadsheet_filters():
    """Get available filters for spreadsheet search and filtering"""
    return {
        "file_types": [
            {"value": "all", "label": "All File Types", "count": 8},
            {"value": "xlsx", "label": "Excel (.xlsx)", "count": 5},
            {"value": "csv", "label": "CSV (.csv)", "count": 2},
            {"value": "xls", "label": "Excel Legacy (.xls)", "count": 1}
        ],
        "created_by": [
            {"value": "all", "label": "All Users", "count": 8},
            {"value": "admin", "label": "Lab Administrator", "count": 3},
            {"value": "dr.wilson", "label": "Dr. Sarah Wilson", "count": 2},
            {"value": "dr.smith", "label": "Dr. Jane Smith", "count": 2},
            {"value": "technician", "label": "Lab Technician", "count": 1}
        ],
        "date_ranges": [
            {"value": "all", "label": "All Time", "count": 8},
            {"value": "today", "label": "Today", "count": 2},
            {"value": "week", "label": "This Week", "count": 4},
            {"value": "month", "label": "This Month", "count": 6},
            {"value": "quarter", "label": "This Quarter", "count": 8}
        ],
        "status": [
            {"value": "all", "label": "All Status", "count": 8},
            {"value": "active", "label": "Active", "count": 6},
            {"value": "archived", "label": "Archived", "count": 1},
            {"value": "processing", "label": "Processing", "count": 1}
        ],
        "categories": [
            {"value": "all", "label": "All Categories", "count": 8},
            {"value": "samples", "label": "Sample Data", "count": 3},
            {"value": "results", "label": "Results & Analysis", "count": 2},
            {"value": "templates", "label": "Templates", "count": 1},
            {"value": "reports", "label": "Reports", "count": 1},
            {"value": "tracking", "label": "Tracking & QC", "count": 1}
        ],
        "size_ranges": [
            {"value": "all", "label": "All Sizes", "count": 8},
            {"value": "small", "label": "< 1 MB", "count": 2},
            {"value": "medium", "label": "1-10 MB", "count": 4},
            {"value": "large", "label": "10-100 MB", "count": 2},
            {"value": "xlarge", "label": "> 100 MB", "count": 0}
        ]
    }

@app.get("/api/storage/locations")
async def get_storage_locations():
    """Get available storage locations"""
    return [
        {
            "id": "zone-001",
            "name": "Ultra-Low Freezer Zone A",
            "zone": "Zone-A",
            "temperature": "-80°C",
            "capacity": 500,
            "occupied": 287,
            "available": 213,
            "status": "operational",
            "last_checked": datetime.now().isoformat(),
            "equipment": "Thermo Fisher ULT-2586",
            "location": "Building A, Room 101"
        },
        {
            "id": "zone-002", 
            "name": "Standard Freezer Zone B",
            "zone": "Zone-B",
            "temperature": "-20°C",
            "capacity": 300,
            "occupied": 156,
            "available": 144,
            "status": "operational",
            "last_checked": datetime.now().isoformat(),
            "equipment": "Fisher Scientific Freezer",
            "location": "Building A, Room 102"
        },
        {
            "id": "zone-003",
            "name": "Refrigerated Storage Zone C",
            "zone": "Zone-C", 
            "temperature": "4°C",
            "capacity": 200,
            "occupied": 89,
            "available": 111,
            "status": "operational",
            "last_checked": datetime.now().isoformat(),
            "equipment": "Lab Refrigerator Unit",
            "location": "Building A, Room 103"
        },
        {
            "id": "zone-004",
            "name": "Room Temperature Zone D",
            "zone": "Zone-D",
            "temperature": "RT",
            "capacity": 150,
            "occupied": 34,
            "available": 116,
            "status": "operational", 
            "last_checked": datetime.now().isoformat(),
            "equipment": "Climate Controlled Cabinet",
            "location": "Building A, Room 104"
        },
        {
            "id": "zone-005",
            "name": "Incubator Zone E",
            "zone": "Zone-E",
            "temperature": "37°C",
            "capacity": 100,
            "occupied": 23,
            "available": 77,
            "status": "operational",
            "last_checked": datetime.now().isoformat(), 
            "equipment": "CO2 Incubator",
            "location": "Building A, Room 105"
        }
    ]

@app.get("/api/storage/locations/{location_id}")
async def get_storage_location(location_id: str):
    """Get specific storage location details"""
    locations = await get_storage_locations()
    location = next((l for l in locations if l["id"] == location_id), None)
    if not location:
        raise HTTPException(status_code=404, detail="Storage location not found")
    return location

@app.post("/api/spreadsheets/preview-sheets")
async def preview_spreadsheet_sheets(file: UploadFile = File(...)):
    """Preview spreadsheet sheets from uploaded file"""
    # Mock response for spreadsheet sheet preview - matches frontend expectations
    return {
        "success": True,
        "data": ["Sample Data", "Storage Info", "QC Results"],  # Frontend expects 'data' not 'sheet_names'
        "message": "Sheet names retrieved successfully",
        "total_sheets": 3,
        "file_info": {
            "filename": file.filename,
            "size": file.size if hasattr(file, 'size') else 0,
            "content_type": file.content_type
        },
        "preview_data": {
            "Sample Data": {
                "headers": ["Sample ID", "Type", "Volume", "Concentration", "Date"],
                "row_count": 156,
                "preview_rows": [
                    ["S001", "DNA", "150", "25.7", "2024-01-15"],
                    ["S002", "RNA", "100", "22.3", "2024-01-16"], 
                    ["S003", "Protein", "200", "18.9", "2024-01-17"]
                ]
            },
            "Storage Info": {
                "headers": ["Sample ID", "Location", "Temperature", "Container"],
                "row_count": 156,
                "preview_rows": [
                    ["S001", "Zone-A-01", "-80°C", "Cryotube"],
                    ["S002", "Zone-B-05", "-20°C", "PCR Tube"],
                    ["S003", "Zone-C-12", "4°C", "Eppendorf"]
                ]
            },
            "QC Results": {
                "headers": ["Sample ID", "Quality Score", "Pass/Fail", "Notes"],
                "row_count": 156,
                "preview_rows": [
                    ["S001", "8.5", "Pass", "Good quality"],
                    ["S002", "7.2", "Pass", "Minor degradation"],
                    ["S003", "9.1", "Pass", "Excellent"]
                ]
            }
        }
    }

@app.post("/api/spreadsheets/upload-multiple")
async def upload_multiple_spreadsheets(file: UploadFile = File(...), uploaded_by: str = "Lab Administrator"):
    """Upload spreadsheet files and save to PostgreSQL database"""
    try:
        # Get file extension and determine type
        file_extension = file.filename.split('.')[-1].upper() if file.filename else "UNKNOWN"
        file_size_bytes = file.size if hasattr(file, 'size') and file.size else 2000000
        file_size_mb = round(file_size_bytes / (1024 * 1024), 1)
        
        pool = await get_db_pool()
        async with pool.acquire() as connection:
            # Generate UUID for the spreadsheet
            spreadsheet_id = uuid.uuid4()
            
            # Create metadata
            metadata = {
                "upload_method": "web_interface",
                "original_name": file.filename or "Uploaded Spreadsheet",
                "sheets": ["Sample Data", "Storage Info"] if file_extension in ["XLSX", "XLS"] else ["Data"],
                "preview": {
                    "headers": ["Sample ID", "Type", "Volume", "Concentration"],
                    "sample_rows": [
                        ["S001", "DNA", "150", "25.7"],
                        ["S002", "RNA", "100", "22.3"]
                    ]
                }
            }
            
            # Get a valid user UUID (Lab Administrator) for the uploaded_by field
            user_result = await connection.fetchrow("""
                SELECT id FROM users WHERE email = 'admin@lab.local' LIMIT 1
            """)
            user_id = user_result['id'] if user_result else None
            
            # Insert spreadsheet record into database
            await connection.execute("""
                INSERT INTO spreadsheet_datasets (
                    id, name, filename, file_size, file_type, 
                    upload_status, total_rows, total_columns,
                    description, metadata, uploaded_by, created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW())
            """, 
                spreadsheet_id,
                f"Uploaded: {file.filename or 'Unknown File'}",
                file.filename or "uploaded_file.xlsx", 
                file_size_bytes,
                file_extension,
                "completed",  # Set as completed for now
                245,  # Mock row count
                8,    # Mock column count
                "User uploaded spreadsheet via web interface",
                json.dumps(metadata),
                user_id  # Use UUID instead of string
            )
            
            # Create response with the inserted data
            new_spreadsheet = {
                "id": str(spreadsheet_id),
                "name": f"Uploaded: {file.filename or 'Unknown File'}",
                "description": "User uploaded spreadsheet via web interface",
                "filename": file.filename or "uploaded_file.xlsx",
                "original_name": file.filename or "Uploaded Spreadsheet.xlsx",
                "file_type": file_extension,
                "size": f"{file_size_mb} MB",
                "status": "completed",
                "last_modified": datetime.now().isoformat(),
                "created_at": datetime.now().isoformat(),
                "uploaded_by": uploaded_by,
                "created_by": uploaded_by,
                "row_count": 245,  # Mock row count
                "column_count": 8,  # Mock column count
                "sheets": metadata["sheets"],
                "upload_method": "web_interface",
                "preview": metadata["preview"]
            }
            
            return {
                "success": True,
                "message": f"Spreadsheet '{file.filename}' uploaded successfully and saved to PostgreSQL database",
                "uploaded_count": 1,
                "files": [new_spreadsheet],
                "data": [new_spreadsheet],  # Frontend might expect 'data' key
                "total_size": f"{file_size_mb} MB",
                "processing_time": 1.5,
                "spreadsheet_id": str(spreadsheet_id)
            }
            
    except Exception as e:
        print(f"Database error during upload: {e}")
        # Return error response
        raise HTTPException(
            status_code=500, 
            detail=f"Failed to save spreadsheet to database: {str(e)}"
        )

@app.get("/api/reports/schema")
async def get_reports_schema():
    """Get reports schema information"""
    return {
        "schemas": [
            {
                "id": "sample_report",
                "name": "Sample Report",
                "description": "Comprehensive sample tracking report",
                "fields": ["sample_id", "type", "status", "location", "created_date"],
                "format": "CSV"
            },
            {
                "id": "storage_report", 
                "name": "Storage Utilization Report",
                "description": "Storage capacity and utilization metrics",
                "fields": ["zone", "capacity", "occupied", "utilization_percent", "temperature"],
                "format": "XLSX"
            },
            {
                "id": "sequencing_report",
                "name": "Sequencing Jobs Report", 
                "description": "Sequencing job status and metrics",
                "fields": ["job_id", "status", "platform", "samples_count", "created_date"],
                "format": "PDF"
            }
        ]
    }

@app.get("/api/reports/templates")
async def get_reports_templates():
    """Get available report templates"""
    return [
        {
            "id": "tpl_sample_summary",
            "name": "Sample Summary Report",
            "description": "Summary of all samples with key metrics",
            "category": "samples",
            "format": "XLSX",
            "parameters": ["date_range", "sample_type", "status"]
        },
        {
            "id": "tpl_storage_audit",
            "name": "Storage Audit Report", 
            "description": "Detailed storage utilization and audit trail",
            "category": "storage",
            "format": "PDF",
            "parameters": ["zone", "date_range", "include_movements"]
        },
        {
            "id": "tpl_sequencing_metrics",
            "name": "Sequencing Performance Metrics",
            "description": "Platform performance and throughput analysis", 
            "category": "sequencing",
            "format": "CSV",
            "parameters": ["platform", "date_range", "job_status"]
        }
    ]

@app.get("/api/users")
async def get_users(page: int = 1, per_page: int = 10):
    """Get paginated list of users"""
    mock_users = [
        {
            "id": "user-001",
            "email": "admin@lab.local",
            "first_name": "Lab",
            "last_name": "Administrator",
            "role": "lab_administrator",
            "status": "active",
            "department": "Laboratory Management",
            "created_at": datetime.now().isoformat()
        },
        {
            "id": "user-002", 
            "email": "scientist@lab.local",
            "first_name": "Dr. Sarah",
            "last_name": "Wilson",
            "role": "principal_investigator",
            "status": "active",
            "department": "Research",
            "created_at": datetime.now().isoformat()
        },
        {
            "id": "user-003",
            "email": "tech@lab.local", 
            "first_name": "John",
            "last_name": "Smith",
            "role": "lab_technician",
            "status": "active",
            "department": "Laboratory Operations",
            "created_at": datetime.now().isoformat()
        }
    ]
    
    # Simulate pagination
    total = len(mock_users)
    start = (page - 1) * per_page
    end = start + per_page
    users = mock_users[start:end]
    
    return {
        "users": users,
        "pagination": {
            "page": page,
            "per_page": per_page,
            "total": total,
            "pages": (total + per_page - 1) // per_page
        }
    }

@app.post("/api/auth/login")
async def login(request: dict):
    """Mock login endpoint with correct response format"""
    email = request.get("email", "")
    password = request.get("password", "")
    
    # Accept various test emails with password "password"
    valid_emails = [
        "admin@lab.local", "admin.test@tracseq.com", "admin@tracseq.com", 
        "admin@localhost", "test@test.com", "user@example.com"
    ]
    
    if password == "password" or email in valid_emails:
        return {
            "data": {
                "user": {
                    "id": "user-001",
                    "email": email,
                    "first_name": "Lab",
                    "last_name": "Administrator",
                    "name": "Lab Administrator",
                    "role": "lab_administrator",
                    "status": "active",
                    "department": "Laboratory Management",
                    "lab_affiliation": "TracSeq Laboratory",
                    "position": "Administrator",
                    "email_verified": True,
                    "created_at": datetime.now().isoformat()
                },
                "token": "mock-jwt-token-12345"
            }
        }
    
    raise HTTPException(status_code=401, detail="Invalid credentials. Use password 'password' or any valid email.")

# Redirect handlers for double /api URLs (frontend routing issue) 
@app.get("/api/api/storage/locations")
async def redirect_storage_locations():
    """Redirect handler for double /api prefix - frontend routing issue"""
    return await get_storage_locations()

@app.get("/api/api/samples")
async def redirect_samples():
    """Redirect handler for double /api prefix - frontend routing issue"""
    return await get_samples()

@app.get("/api/api/templates")
async def redirect_templates():
    """Redirect handler for double /api prefix - frontend routing issue"""
    return await get_templates()

@app.get("/api/api/sequencing/jobs")
async def redirect_sequencing_jobs():
    """Redirect handler for double /api prefix - frontend routing issue"""
    return await get_sequencing_jobs()

@app.get("/api/api/spreadsheets/datasets")
async def redirect_spreadsheets_datasets():
    """Redirect handler for double /api prefix - frontend routing issue"""
    return await get_spreadsheet_datasets()

@app.get("/api/api/dashboard/stats")
async def redirect_dashboard_stats():
    """Redirect handler for double /api prefix - frontend routing issue"""
    return await get_dashboard_stats()

@app.get("/api/api/rag/submissions")
async def redirect_rag_submissions():
    """Redirect handler for double /api prefix - frontend routing issue"""
    return await get_rag_submissions()

@app.get("/api/api/rag/samples")
async def redirect_rag_samples():
    """Redirect handler for double /api prefix - frontend routing issue"""
    return await get_rag_samples()

# Add startup and shutdown event handlers
@app.on_event("startup")
async def startup_event():
    """Initialize database connection on startup"""
    await init_database()

@app.on_event("shutdown")
async def shutdown_event():
    """Close database connection on shutdown"""
    await close_database()

if __name__ == "__main__":
    print("🚀 Starting TracSeq 2.0 Simple API Gateway...")
    print("📍 API Gateway: http://localhost:8089")
    print("📊 Health Check: http://localhost:8089/health")
    print("🔗 API Docs: http://localhost:8089/docs")
    print("💾 Database: PostgreSQL (lab_manager)")
    
    uvicorn.run(app, host="0.0.0.0", port=8089) 