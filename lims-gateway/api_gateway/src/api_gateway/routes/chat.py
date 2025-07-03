"""
Chat API Routes for TracSeq ChatBot Integration

Handles chat streaming, document processing, sample creation, and protocol retrieval.
"""

import json
import asyncio
from typing import AsyncGenerator, Dict, Any, List, Optional
from datetime import datetime
import uuid

from fastapi import APIRouter, HTTPException, Request, UploadFile, File, Form, BackgroundTasks
from fastapi.responses import StreamingResponse
from pydantic import BaseModel, Field
import httpx
import structlog

logger = structlog.get_logger(__name__)

# Create router
router = APIRouter(prefix="/api/chat", tags=["chat"])

# Request/Response Models
class ChatMessage(BaseModel):
    message: str
    conversationId: str = Field(default_factory=lambda: f"conv_{uuid.uuid4().hex[:8]}")
    metadata: Optional[Dict[str, Any]] = None

class ChatResponse(BaseModel):
    id: str
    content: str
    type: str = "assistant"
    timestamp: datetime
    confidence: float
    actions: Optional[List[Dict[str, Any]]] = None
    metadata: Optional[Dict[str, Any]] = None

class SampleCreationRequest(BaseModel):
    sample_type: str
    volume: float
    concentration: float
    buffer: str
    storage_temperature: float
    storage_location: Dict[str, str]
    project_id: Optional[str] = None
    principal_investigator: Optional[str] = None
    metadata: Optional[Dict[str, Any]] = None

class ProtocolInfo(BaseModel):
    id: str
    name: str
    version: str
    last_updated: datetime
    category: str
    file_url: str

class DocumentProcessingResult(BaseModel):
    success: bool
    extracted_data: Dict[str, Any]
    confidence: float
    validation_errors: Optional[List[str]] = None


@router.post("/stream")
async def chat_stream(
    request: Request,
    message: str = Form(...),
    conversationId: str = Form(...),
    files: Optional[List[UploadFile]] = File(None)
):
    """
    Stream chat responses with support for file uploads.
    
    This endpoint handles:
    - Real-time streaming of AI responses
    - File upload processing
    - Context-aware responses based on conversation history
    """
    async def generate_stream() -> AsyncGenerator[str, None]:
        # Initial response
        response_id = str(uuid.uuid4())
        
        # Process any uploaded files first
        file_context = ""
        if files:
            for file in files:
                content = await file.read()
                file_context += f"\n[Processing file: {file.filename} ({len(content)} bytes)]"
        
        # Simulate streaming response
        # In production, this would connect to the RAG service
        response_parts = [
            "I understand your request",
            f" regarding '{message}'.",
            file_context if file_context else "",
            "\n\nBased on our laboratory protocols,",
            " I can help you with the following:\n\n",
            "1. **Sample Registration**: Create and track new samples\n",
            "2. **Document Processing**: Extract data from PDFs\n",
            "3. **Protocol Access**: View standard operating procedures\n",
            "4. **Quality Control**: Validate sample metrics\n\n",
            "Would you like me to assist with any of these tasks?"
        ]
        
        # Stream each part with a delay
        for part in response_parts:
            if part:  # Skip empty parts
                chunk = {
                    "id": response_id,
                    "content": part,
                    "type": "chunk",
                    "timestamp": datetime.utcnow().isoformat()
                }
                yield f"data: {json.dumps(chunk)}\n\n"
                await asyncio.sleep(0.05)  # 50ms delay between chunks
        
        # Send completion event
        completion = {
            "id": response_id,
            "type": "completion",
            "timestamp": datetime.utcnow().isoformat(),
            "metadata": {
                "conversationId": conversationId,
                "confidence": 0.95,
                "modelUsed": "gpt-4",
                "processingTime": 1.2
            }
        }
        yield f"data: {json.dumps(completion)}\n\n"
        yield "data: [DONE]\n\n"
    
    return StreamingResponse(
        generate_stream(),
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
            "X-Accel-Buffering": "no"  # Disable nginx buffering
        }
    )


@router.post("/documents/process")
async def process_document(
    file: UploadFile = File(...),
    background_tasks: BackgroundTasks = BackgroundTasks(),
    metadata: Optional[str] = Form(None)
):
    """
    Process uploaded documents (PDFs, Excel, etc.) and extract laboratory data.
    
    Supports:
    - PDF submission forms
    - Excel sample sheets
    - CSV data files
    """
    try:
        # Validate file type
        allowed_types = [
            "application/pdf",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "application/vnd.ms-excel",
            "text/csv"
        ]
        
        if file.content_type not in allowed_types:
            raise HTTPException(
                status_code=400,
                detail=f"Unsupported file type: {file.content_type}"
            )
        
        # Read file content
        content = await file.read()
        
        # In production, this would send to the RAG service for processing
        # For now, return mock extracted data
        extracted_data = {
            "document_type": "laboratory_submission",
            "extracted_fields": {
                "submitter": {
                    "name": "Dr. Jane Smith",
                    "institution": "Central Research Lab",
                    "email": "jane.smith@research.edu"
                },
                "samples": [{
                    "type": "DNA Extract",
                    "volume": 50.0,
                    "volume_unit": "µL",
                    "concentration": 125.0,
                    "concentration_unit": "ng/µL",
                    "quality_metrics": {
                        "A260_280": 1.85,
                        "A260_230": 2.10,
                        "RIN": 9.2
                    }
                }],
                "storage_requirements": {
                    "temperature": -20,
                    "container": "1.5mL Eppendorf tube",
                    "special_handling": "Avoid freeze-thaw cycles"
                },
                "project_info": {
                    "project_id": "PROJ-2024-001",
                    "funding_source": "NIH R01-123456"
                }
            }
        }
        
        result = DocumentProcessingResult(
            success=True,
            extracted_data=extracted_data,
            confidence=0.92,
            validation_errors=None
        )
        
        # Log processing
        logger.info("Document processed successfully", 
                   filename=file.filename,
                   size=len(content),
                   confidence=result.confidence)
        
        return result
        
    except Exception as e:
        logger.error("Document processing failed", error=str(e))
        raise HTTPException(status_code=500, detail=f"Processing failed: {str(e)}")


@router.post("/samples/create")
async def create_sample_from_chat(
    request: Request,
    sample_data: SampleCreationRequest
):
    """
    Create a new sample based on chat interaction data.
    
    This endpoint is called when users confirm sample creation
    through the chat interface.
    """
    try:
        # In production, this would forward to the sample service
        # For now, create a mock response
        sample_id = f"SAMP-{datetime.utcnow().strftime('%Y%m%d')}-{uuid.uuid4().hex[:6].upper()}"
        
        # Mock barcode generation
        barcode = f"TSQ{sample_id.replace('-', '')}"
        
        response = {
            "success": True,
            "sample": {
                "id": sample_id,
                "barcode": barcode,
                "type": sample_data.sample_type,
                "volume": sample_data.volume,
                "concentration": sample_data.concentration,
                "buffer": sample_data.buffer,
                "storage": {
                    "temperature": sample_data.storage_temperature,
                    "location": sample_data.storage_location
                },
                "project_id": sample_data.project_id,
                "principal_investigator": sample_data.principal_investigator,
                "created_at": datetime.utcnow().isoformat(),
                "status": "registered"
            },
            "actions": {
                "print_label": f"/api/samples/{sample_id}/label",
                "view_details": f"/api/samples/{sample_id}",
                "schedule_qc": f"/api/samples/{sample_id}/qc"
            }
        }
        
        logger.info("Sample created from chat", 
                   sample_id=sample_id,
                   sample_type=sample_data.sample_type)
        
        return response
        
    except Exception as e:
        logger.error("Sample creation failed", error=str(e))
        raise HTTPException(status_code=500, detail=f"Sample creation failed: {str(e)}")


@router.get("/protocols/list")
async def list_protocols(
    category: Optional[str] = None,
    search: Optional[str] = None,
    limit: int = 20,
    offset: int = 0
):
    """
    Retrieve available laboratory protocols and SOPs.
    
    Query parameters:
    - category: Filter by protocol category (e.g., 'extraction', 'qc', 'storage')
    - search: Search protocols by name or content
    - limit: Number of results to return
    - offset: Pagination offset
    """
    # Mock protocol data
    # In production, this would query the protocol database
    protocols = [
        ProtocolInfo(
            id="SOP-001",
            name="DNA/RNA Extraction Protocol",
            version="2.3",
            last_updated=datetime(2024, 1, 15),
            category="extraction",
            file_url="/protocols/SOP-001-v2.3.pdf"
        ),
        ProtocolInfo(
            id="SOP-005",
            name="Library Preparation Protocol",
            version="1.8",
            last_updated=datetime(2024, 2, 20),
            category="library_prep",
            file_url="/protocols/SOP-005-v1.8.pdf"
        ),
        ProtocolInfo(
            id="SOP-009",
            name="Quality Control Standards",
            version="3.1",
            last_updated=datetime(2023, 12, 10),
            category="qc",
            file_url="/protocols/SOP-009-v3.1.pdf"
        ),
        ProtocolInfo(
            id="SOP-012",
            name="Sample Storage Guidelines",
            version="2.0",
            last_updated=datetime(2024, 1, 5),
            category="storage",
            file_url="/protocols/SOP-012-v2.0.pdf"
        ),
        ProtocolInfo(
            id="SOP-015",
            name="Sequencing Run Setup",
            version="1.5",
            last_updated=datetime(2024, 3, 1),
            category="sequencing",
            file_url="/protocols/SOP-015-v1.5.pdf"
        )
    ]
    
    # Apply filters
    filtered_protocols = protocols
    
    if category:
        filtered_protocols = [p for p in filtered_protocols if p.category == category]
    
    if search:
        search_lower = search.lower()
        filtered_protocols = [
            p for p in filtered_protocols 
            if search_lower in p.name.lower() or search_lower in p.category.lower()
        ]
    
    # Apply pagination
    total = len(filtered_protocols)
    paginated = filtered_protocols[offset:offset + limit]
    
    return {
        "protocols": paginated,
        "total": total,
        "limit": limit,
        "offset": offset,
        "categories": list(set(p.category for p in protocols))
    }


# Health check for chat service
@router.get("/health")
async def chat_health():
    """Check chat service health status."""
    return {
        "status": "healthy",
        "service": "chat",
        "timestamp": datetime.utcnow().isoformat(),
        "features": {
            "streaming": True,
            "file_upload": True,
            "document_processing": True,
            "sample_creation": True,
            "protocol_access": True
        }
    } 