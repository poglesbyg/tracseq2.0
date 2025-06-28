#!/usr/bin/env python3
"""
Simple API Gateway for TracSeq 2.0 Development
Provides basic endpoints for frontend development without full microservices.
"""

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from datetime import datetime
import uvicorn
import json

app = FastAPI(title="TracSeq 2.0 Simple API Gateway", version="1.0.0")

# Enable CORS for frontend development
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:5173", "http://localhost:3000"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

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
• Samples In Storage: 1
• Samples Pending: 1

**Storage Zones Available**:
• -80°C (Ultra-low freezer)
• -20°C (Standard freezer) 
• 4°C (Refrigerated)
• Room Temperature
• 37°C (Incubator)

Storage locations are automatically assigned based on sample requirements extracted from your documents or manually specified during sample creation."""
        
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

if __name__ == "__main__":
    print("🚀 Starting TracSeq 2.0 Simple API Gateway...")
    print("📍 API Gateway: http://localhost:8089")
    print("📊 Health Check: http://localhost:8089/health")
    print("🔗 API Docs: http://localhost:8089/docs")
    
    uvicorn.run(app, host="0.0.0.0", port=8089) 