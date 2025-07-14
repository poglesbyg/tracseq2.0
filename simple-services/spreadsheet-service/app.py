from fastapi import FastAPI, HTTPException, UploadFile, File
from datetime import datetime
import uvicorn
import os

app = FastAPI(title="Spreadsheet Service", version="1.0.0")

# Mock data for spreadsheet datasets
MOCK_DATASETS = [
    {
        "id": "dataset-001",
        "name": "Lab Results Q4 2024",
        "description": "Quarterly laboratory results compilation",
        "file_type": "xlsx",
        "size_mb": 2.5,
        "last_modified": "2024-12-15T10:30:00Z",
        "created_by": "Dr. Smith",
        "status": "active",
        "row_count": 1250,
        "column_count": 15
    },
    {
        "id": "dataset-002", 
        "name": "Sample Tracking Database",
        "description": "Complete sample tracking and storage data",
        "file_type": "csv",
        "size_mb": 5.8,
        "last_modified": "2024-12-14T16:45:00Z",
        "created_by": "Lab Manager",
        "status": "active",
        "row_count": 3200,
        "column_count": 22
    },
    {
        "id": "dataset-003",
        "name": "Equipment Maintenance Log",
        "description": "Equipment maintenance and calibration records",
        "file_type": "xlsx",
        "size_mb": 1.2,
        "last_modified": "2024-12-13T09:15:00Z", 
        "created_by": "Maintenance Team",
        "status": "archived",
        "row_count": 580,
        "column_count": 8
    }
]

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "service": "spreadsheet-service",
        "timestamp": datetime.utcnow().isoformat(),
        "version": "1.0.0"
    }

@app.get("/api/v1/spreadsheets")
async def get_spreadsheets():
    """Get all spreadsheet datasets"""
    return {
        "success": True,
        "data": {
            "datasets": MOCK_DATASETS,
            "total_count": len(MOCK_DATASETS),
            "active_count": len([d for d in MOCK_DATASETS if d["status"] == "active"]),
            "total_size_mb": sum(d["size_mb"] for d in MOCK_DATASETS)
        }
    }

@app.post("/api/v1/spreadsheets")
async def create_spreadsheet():
    """Create a new spreadsheet dataset"""
    return {
        "success": True,
        "data": {
            "message": "Dataset created successfully",
            "dataset_id": "dataset-004",
            "status": "processing"
        }
    }

@app.get("/api/v1/spreadsheets/{dataset_id}")
async def get_spreadsheet_dataset(dataset_id: str):
    """Get a specific spreadsheet dataset"""
    # Find the dataset
    dataset = next((d for d in MOCK_DATASETS if d["id"] == dataset_id), None)
    if not dataset:
        raise HTTPException(status_code=404, detail="Dataset not found")
    
    # Return detailed dataset information
    return {
        "success": True,
        "data": {
            **dataset,
            "columns": [
                {"name": "sample_id", "type": "string"},
                {"name": "date_collected", "type": "date"},
                {"name": "sample_type", "type": "string"},
                {"name": "concentration", "type": "number"},
                {"name": "volume", "type": "number"}
            ],
            "preview_data": [
                ["SMPL-001", "2024-12-01", "DNA", 150.5, 50.0],
                ["SMPL-002", "2024-12-01", "RNA", 200.3, 25.0],
                ["SMPL-003", "2024-12-02", "Protein", 75.8, 100.0]
            ]
        }
    }

@app.get("/api/v1/versions")
async def get_spreadsheet_versions():
    """Get spreadsheet versions"""
    return {
        "success": True,
        "data": {
            "versions": [
                {
                    "id": "v1.0",
                    "created_at": "2024-12-01T10:00:00Z",
                    "created_by": "Dr. Smith",
                    "changes": "Initial version"
                },
                {
                    "id": "v1.1", 
                    "created_at": "2024-12-10T14:30:00Z",
                    "created_by": "Lab Manager",
                    "changes": "Added new sample types"
                }
            ]
        }
    }

@app.post("/api/v1/versions")
async def upload_spreadsheet(file: UploadFile = File(...)):
    """Upload a spreadsheet"""
    return {
        "success": True,
        "data": {
            "message": "Spreadsheet uploaded successfully",
            "file_id": "file-12345",
            "filename": file.filename,
            "processing_status": "queued"
        }
    }

@app.post("/api/v1/spreadsheets/preview-sheets")
async def preview_spreadsheet_sheets(request_data: dict):
    """Get sheet names for spreadsheet preview"""
    dataset_id = request_data.get("dataset_id")
    
    if not dataset_id:
        raise HTTPException(status_code=400, detail="dataset_id is required")
    
    # Find the dataset
    dataset = next((d for d in MOCK_DATASETS if d["id"] == dataset_id), None)
    if not dataset:
        raise HTTPException(status_code=404, detail="Dataset not found")
    
    # Return mock sheet names based on file type
    if dataset["file_type"] == "xlsx":
        sheets = ["Lab Results", "Summary", "Quality Control"]
    else:
        sheets = ["Main Data"]
    
    return {
        "success": True,
        "data": {
            "dataset_id": dataset_id,
            "sheets": sheets,
            "default_sheet": sheets[0] if sheets else None
        }
    }

if __name__ == "__main__":
    port = int(os.getenv("PORT", 8080))
    uvicorn.run(app, host="0.0.0.0", port=port) 