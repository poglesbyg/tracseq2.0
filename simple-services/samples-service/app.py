from fastapi import FastAPI, HTTPException
from datetime import datetime
import uvicorn
import os

app = FastAPI(title="Samples Service", version="1.0.0")

# Mock data for samples
MOCK_SAMPLES = [
    {
        "id": "sample-001",
        "name": "DNA Sample 001",
        "barcode": "DNA-001-2024",
        "sample_type": "DNA",
        "status": "validated",
        "concentration": 150.5,
        "volume": 2.0,
        "storage_location": "Freezer A1-B2",
        "submitter": "Dr. Smith",
        "department": "Genomics",
        "created_at": "2024-12-15T10:30:00Z",
        "updated_at": "2024-12-15T11:00:00Z",
        "metadata": {
            "patient_id": "P12345",
            "collection_date": "2024-12-14",
            "analysis_type": "WGS",
            "priority": "high"
        }
    },
    {
        "id": "sample-002",
        "name": "RNA Sample 002",
        "barcode": "RNA-002-2024",
        "sample_type": "RNA",
        "status": "in_storage",
        "concentration": 89.3,
        "volume": 1.5,
        "storage_location": "Freezer B2-C3",
        "submitter": "Dr. Johnson",
        "department": "Transcriptomics",
        "created_at": "2024-12-15T09:15:00Z",
        "updated_at": "2024-12-15T09:45:00Z",
        "metadata": {
            "patient_id": "P12346",
            "collection_date": "2024-12-13",
            "analysis_type": "RNA-seq",
            "priority": "medium"
        }
    },
    {
        "id": "sample-003",
        "name": "Protein Sample 003",
        "barcode": "PRT-003-2024",
        "sample_type": "Protein",
        "status": "pending",
        "concentration": 45.7,
        "volume": 0.8,
        "storage_location": "Freezer C1-D2",
        "submitter": "Dr. Brown",
        "department": "Proteomics",
        "created_at": "2024-12-15T08:00:00Z",
        "updated_at": "2024-12-15T08:30:00Z",
        "metadata": {
            "patient_id": "P12347",
            "collection_date": "2024-12-12",
            "analysis_type": "MS/MS",
            "priority": "low"
        }
    }
]

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "service": "samples-service",
        "timestamp": datetime.utcnow().isoformat(),
        "version": "1.0.0"
    }

@app.get("/api/v1/samples")
async def get_samples():
    """Get all samples"""
    return {
        "success": True,
        "data": {
            "samples": MOCK_SAMPLES,
            "page": 1,
            "page_size": 50,
            "total_count": len(MOCK_SAMPLES),
            "total_pages": 1
        },
        "pagination": {
            "total": len(MOCK_SAMPLES),
            "page": 1,
            "per_page": 50,
            "pages": 1
        },
        "filters": {
            "status": ["pending", "validated", "in_storage", "completed"],
            "sample_type": ["DNA", "RNA", "Protein"],
            "department": ["Genomics", "Transcriptomics", "Proteomics"]
        }
    }

@app.get("/api/v1/samples/{sample_id}")
async def get_sample(sample_id: str):
    """Get a specific sample"""
    sample = next((s for s in MOCK_SAMPLES if s["id"] == sample_id), None)
    if not sample:
        raise HTTPException(status_code=404, detail="Sample not found")
    
    return {
        "success": True,
        "data": sample
    }

@app.post("/api/v1/samples")
async def create_sample():
    """Create a new sample"""
    return {
        "success": True,
        "data": {
            "message": "Sample created successfully",
            "sample_id": "sample-004",
            "status": "pending"
        }
    }

@app.post("/api/v1/samples/batch")
async def create_samples_batch():
    """Create multiple samples in batch"""
    return {
        "success": True,
        "data": {
            "message": "Batch processing completed",
            "total_created": 3,
            "total_failed": 0,
            "created_samples": [
                {"id": "sample-004", "name": "Batch Sample 1", "status": "pending"},
                {"id": "sample-005", "name": "Batch Sample 2", "status": "pending"},
                {"id": "sample-006", "name": "Batch Sample 3", "status": "pending"}
            ],
            "failed_samples": []
        }
    }

if __name__ == "__main__":
    port = int(os.getenv("PORT", 8080))
    uvicorn.run(app, host="0.0.0.0", port=port) 