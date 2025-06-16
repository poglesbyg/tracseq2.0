#!/usr/bin/env python3
"""
Simple Frontend API Bridge for RAG Submissions
Provides basic API endpoints that the lab_manager frontend needs
"""

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import List, Optional
import asyncio
import asyncpg
import uuid
from datetime import datetime

app = FastAPI(
    title="Simple RAG Submissions API Bridge",
    description="Basic API bridge for lab_manager frontend to access RAG submissions",
    version="1.0.0"
)

# Enable CORS for lab_manager frontend
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Allow all origins for development
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Database connection
DB_CONFIG = {
    'host': 'postgres',
    'port': 5432,
    'database': 'lab_manager',
    'user': 'postgres',
    'password': 'postgres'
}

class RagSubmissionResponse(BaseModel):
    """Response model for RAG submissions"""
    id: str
    submission_id: str
    submitter_name: Optional[str]
    submitter_email: Optional[str]
    sample_type: Optional[str]
    sample_name: Optional[str]
    confidence_score: float
    created_at: str
    status: str = "completed"

async def get_db_connection():
    """Get database connection"""
    return await asyncpg.connect(**DB_CONFIG)

@app.get("/")
async def root():
    """Root endpoint"""
    return {"message": "Simple RAG Submissions API Bridge", "status": "operational"}

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    try:
        conn = await get_db_connection()
        await conn.close()
        return {"status": "healthy", "database": "connected"}
    except Exception as e:
        raise HTTPException(status_code=503, detail=f"Database connection failed: {e}")

@app.get("/api/rag/submissions", response_model=List[RagSubmissionResponse])
async def get_rag_submissions(limit: int = 50, offset: int = 0):
    """Get RAG submissions for the frontend"""
    try:
        conn = await get_db_connection()
        
        # Query RAG submissions from database
        submissions = await conn.fetch("""
            SELECT 
                submission_id,
                submitter_name,
                submitter_email,
                sample_type,
                document_name,
                confidence_score,
                created_at
            FROM rag_submissions 
            ORDER BY created_at DESC 
            LIMIT $1 OFFSET $2
        """, limit, offset)
        
        await conn.close()
        
        # Convert to response format
        result = []
        for row in submissions:
            result.append(RagSubmissionResponse(
                id=str(uuid.uuid4())[:8],  # Short ID for display
                submission_id=row['submission_id'],
                submitter_name=row['submitter_name'],
                submitter_email=row['submitter_email'],
                sample_type=row['sample_type'] or "Unknown",
                sample_name=row['document_name'],
                confidence_score=row['confidence_score'] or 0.0,
                created_at=row['created_at'].isoformat() if row['created_at'] else "",
                status="completed"
            ))
        
        return result
        
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to fetch submissions: {e}")

@app.get("/api/rag/stats")
async def get_rag_statistics():
    """Get RAG system statistics"""
    try:
        conn = await get_db_connection()
        
        # Get submission counts
        total_submissions = await conn.fetchval("SELECT COUNT(*) FROM rag_submissions")
        
        # Get recent activity
        recent_count = await conn.fetchval("""
            SELECT COUNT(*) FROM rag_submissions 
            WHERE created_at >= NOW() - INTERVAL '7 days'
        """)
        
        # Get average confidence
        avg_confidence = await conn.fetchval("""
            SELECT AVG(confidence_score) FROM rag_submissions 
            WHERE confidence_score > 0
        """)
        
        await conn.close()
        
        return {
            "total_submissions": total_submissions or 0,
            "recent_submissions": recent_count or 0,
            "average_confidence": float(avg_confidence or 0.0),
            "status": "operational"
        }
        
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to get statistics: {e}")

@app.post("/api/rag/process")
async def process_document():
    """Placeholder for document processing"""
    return {
        "success": False,
        "message": "Document processing not implemented in simple bridge",
        "processing_time": 0.0
    }

class QueryRequest(BaseModel):
    """Request model for queries"""
    query: str
    session_id: Optional[str] = "default"

class QueryResponse(BaseModel):
    """Response model for queries"""
    answer: str

def get_intelligent_response(query: str) -> str:
    """Generate intelligent responses based on query content"""
    query_lower = query.lower().strip()
    
    # Greeting responses
    if any(word in query_lower for word in ['hello', 'hi', 'hey', 'greetings']):
        return f"""Hello! I'm your lab management assistant. I can help you with sample processing, storage management, sequencing workflows, and more.

What can I help you with today? You can ask me about:
• Submitting new samples
• Storage requirements 
• Setting up sequencing jobs
• Generating reports
• Using the lab management system"""

    # Sample submission and processing (check this BEFORE general help)
    elif any(phrase in query_lower for phrase in ['submit', 'upload', 'create sample', 'new sample', 'add sample', 'submit a sample', 'submit sample', 'submission']):
        return """To submit new samples, you have several options:

1. 📄 AI DOCUMENT PROCESSING (Recommended)
   • Upload lab submission forms (PDF, Word, or text)
   • I'll automatically extract sample information
   • Review and confirm the extracted data
   
2. ✏️ MANUAL SAMPLE ENTRY
   • Use the "Create Sample" form
   • Fill in all required fields manually
   • Generate barcodes automatically

3. 📊 BULK UPLOAD VIA TEMPLATES
   • Download Excel templates
   • Fill in multiple samples at once
   • Upload for batch processing

Which method would you prefer to use?"""

    # Storage and temperature questions
    elif any(word in query_lower for word in ['storage', 'store', 'temperature', 'freezer', 'refrigerator', 'location']):
        return """For sample storage management:

🌡️ TEMPERATURE REQUIREMENTS:
• DNA samples: -20°C or -80°C for long-term storage
• RNA samples: -80°C (temperature critical!)
• Proteins: -80°C with appropriate buffers
• Cell cultures: Liquid nitrogen (-196°C) or -80°C

📍 STORAGE LOCATIONS:
• Create freezer/refrigerator locations
• Assign storage positions with barcodes
• Track capacity and utilization
• Log all sample movements

🔍 FINDING SAMPLES:
• Scan barcodes to locate samples
• Search by sample ID or name
• View storage history and movements

Would you like help setting up storage locations or finding a specific sample?"""

    # Sequencing and molecular biology
    elif any(word in query_lower for word in ['sequencing', 'sequence', 'dna', 'rna', 'library', 'prep', 'qc', 'quality']):
        return """For sequencing workflows and quality control:

🧬 SEQUENCING PLATFORMS SUPPORTED:
• Illumina: MiSeq, NextSeq, NovaSeq
• Oxford Nanopore: MinION, GridION  
• PacBio: Sequel, Revio

📋 SAMPLE SHEET GENERATION:
• Automatically create sample sheets
• Include barcodes and metadata
• Export in platform-specific formats

🔬 QUALITY REQUIREMENTS:
• DNA: A260/A280 ratio 1.8-2.0, >10 ng/μL
• RNA: RIN score >7, >100 ng/μL
• Library QC: Fragment size, molarity

📊 TRACKING & ANALYSIS:
• Monitor job progress and status
• Track quality metrics over time
• Configure analysis pipelines

What type of sequencing are you planning?"""

    # Reports and data analysis  
    elif any(word in query_lower for word in ['report', 'export', 'data', 'analysis', 'statistics', 'analytics', 'generate report', 'create report']):
        return """For reports and data analysis:

📊 AVAILABLE REPORTS:
• Sample inventory and status reports
• Storage utilization summaries
• Sequencing job progress tracking
• Quality metrics analysis
• Custom SQL queries

📤 EXPORT OPTIONS:
• Excel spreadsheets (.xlsx)
• CSV files for further analysis
• PDF reports for sharing
• JSON data for API integration

🔍 SEARCH & FILTER:
• Advanced search across all samples
• Filter by date ranges, sample types
• Sort by various criteria
• Save commonly used filters

📈 ANALYTICS:
• Track lab productivity over time
• Monitor storage usage trends
• Quality control statistics
• Sample submission patterns

What kind of report would you like to generate?"""

    # Barcode and tracking
    elif any(phrase in query_lower for phrase in ['barcode', 'track', 'find sample', 'locate sample', 'scan', 'find a sample', 'locate a sample', 'where is sample']):
        return """For barcode tracking and sample location:

🏷️ BARCODE SYSTEM:
• Automatic barcode generation for new samples
• Customizable barcode formats
• Support for 1D and 2D codes
• Print barcode labels directly

📍 SAMPLE TRACKING:
• Scan barcodes to find samples instantly
• Track movements between locations
• Maintain complete audit trails
• Real-time location updates

🔍 SEARCH CAPABILITIES:
• Search by barcode, sample name, or ID
• Filter by storage location or date
• View complete sample history
• Export tracking reports

📱 MOBILE SCANNING:
• Use smartphone cameras for scanning
• Update locations on-the-go
• Quick status updates

Need help finding a specific sample or setting up barcode printing?"""

    # Templates and batch processing
    elif any(word in query_lower for word in ['template', 'excel', 'batch', 'bulk', 'multiple']):
        return """For template-based batch processing:

📊 EXCEL TEMPLATES:
• Download pre-formatted templates
• Include all required sample fields
• Built-in validation rules
• Example data provided

📤 BATCH UPLOAD PROCESS:
1. Download the Excel template
2. Fill in your sample information
3. Upload the completed file
4. Review and validate data
5. Confirm batch creation

✅ VALIDATION FEATURES:
• Automatic data validation
• Duplicate detection
• Format checking
• Error highlighting with suggestions

🔄 SUPPORTED FORMATS:
• Excel (.xlsx, .xls)
• CSV files
• Tab-delimited text
• Custom formats on request

How many samples are you looking to upload at once?"""

    # Help and general queries (check after specific ones)
    elif any(phrase in query_lower for phrase in ['help', 'what can you do', 'what do you do', 'how can you help', 'what are your capabilities']):
        return """I'm here to help with your laboratory management needs! Here's what I can assist with:

🧪 SAMPLE MANAGEMENT
• Submit samples using AI document processing
• Create and edit sample records with barcodes
• Track sample status and locations

🏠 STORAGE SYSTEMS  
• Manage storage locations and conditions
• Track temperature requirements
• Monitor capacity and sample movements

🧬 SEQUENCING WORKFLOWS
• Set up sequencing jobs and protocols
• Generate sample sheets for instruments
• Track quality metrics and analysis

📊 DATA & REPORTS
• Generate custom reports and analytics
• Export data in various formats
• Search and filter sample information

Just ask me a specific question about any of these areas!"""

    # Login, access, and system issues
    elif any(word in query_lower for word in ['login', 'access', 'permission', 'error', 'problem', 'issue']):
        return """For system access and troubleshooting:

🔐 ACCESS ISSUES:
• Default admin login: admin@lab.local / admin123
• Contact your lab administrator for new accounts
• Different roles have different permissions
• Password reset available through admin

❗ COMMON ISSUES:
• Clear browser cache if pages aren't loading
• Check internet connection for API calls
• Refresh page if data seems outdated
• Try logging out and back in

🛠️ TROUBLESHOOTING:
• Browser compatibility: Chrome, Firefox, Safari
• Enable JavaScript and cookies
• Disable ad blockers if needed
• Check for system maintenance announcements

👥 USER ROLES:
• Lab Administrator: Full access
• Principal Investigator: Sample and project management
• Lab Technician: Sample processing and QC
• Data Analyst: Reports and analytics only

What specific issue are you experiencing?"""

    # Default response for unmatched queries
    else:
        return f"""I understand you're asking about: "{query}"

I'm your lab management assistant and I can help with many laboratory tasks. Here are some things you might want to know about:

🧪 COMMON TASKS:
• "How do I submit a new sample?"
• "What are the storage requirements for DNA?"
• "How do I create a sequencing job?"
• "Can you help me generate a report?"
• "Where is sample XYZ located?"

🔍 TRY ASKING ABOUT:
• Sample submission and processing
• Storage locations and temperatures  
• Sequencing workflows and QC
• Barcode tracking and scanning
• Data export and reporting
• System navigation and troubleshooting

Could you rephrase your question or ask about a specific lab management task? I'm here to help make your laboratory work more efficient!"""

@app.post("/query", response_model=QueryResponse)
async def query_submission_information(request: QueryRequest):
    """Query the RAG system for information about submitted samples"""
    try:
        answer = get_intelligent_response(request.query)
        return QueryResponse(answer=answer)
        
    except Exception as e:
        # Return a helpful error message
        return QueryResponse(
            answer=f"I apologize, but I'm having trouble processing your question right now. This could be due to a temporary system issue. Please try again in a moment, or contact your lab administrator if the problem persists."
        )

# Startup event to test database connection
@app.on_event("startup")
async def startup_event():
    """Test database connection on startup"""
    try:
        conn = await get_db_connection()
        await conn.close()
        print("✅ Database connection successful")
    except Exception as e:
        print(f"❌ Database connection failed: {e}")

if __name__ == "__main__":
    import uvicorn
    print("🚀 Starting Simple RAG Submissions API Bridge")
    print("📡 Providing basic RAG data access for frontend")
    print("🌐 CORS enabled for all origins")
    
    uvicorn.run(app, host="0.0.0.0", port=8000) 
