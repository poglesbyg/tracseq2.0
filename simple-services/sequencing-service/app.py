from fastapi import FastAPI, HTTPException
from datetime import datetime
import uvicorn
import os

app = FastAPI(title="Sequencing Service", version="1.0.0")

# Mock data for sequencing jobs
MOCK_JOBS = [
    {
        "id": "seq-001",
        "name": "WGS Sample Batch 1",
        "status": "running",
        "sample_count": 24,
        "platform": "NovaSeq 6000",
        "run_type": "Paired-end 150bp",
        "started_at": "2025-07-11T20:00:00Z",
        "estimated_completion": "2025-07-12T08:00:00Z",
        "progress": 65,
        "submitter": "Dr. Smith",
        "priority": "high"
    },
    {
        "id": "seq-002", 
        "name": "RNA-seq Project Alpha",
        "status": "queued",
        "sample_count": 12,
        "platform": "NextSeq 2000",
        "run_type": "Paired-end 100bp",
        "started_at": None,
        "estimated_completion": "2025-07-12T16:00:00Z",
        "progress": 0,
        "submitter": "Dr. Johnson",
        "priority": "medium"
    },
    {
        "id": "seq-003",
        "name": "Exome Sequencing Cohort",
        "status": "completed",
        "sample_count": 48,
        "platform": "HiSeq X",
        "run_type": "Paired-end 150bp",
        "started_at": "2025-07-10T10:00:00Z",
        "estimated_completion": "2025-07-11T18:00:00Z",
        "progress": 100,
        "submitter": "Dr. Brown",
        "priority": "low"
    },
    {
        "id": "seq-004",
        "name": "Single Cell RNA-seq",
        "status": "failed",
        "sample_count": 6,
        "platform": "MiSeq",
        "run_type": "Single-end 75bp",
        "started_at": "2025-07-11T14:00:00Z",
        "estimated_completion": None,
        "progress": 45,
        "submitter": "Dr. Wilson",
        "priority": "high",
        "error_message": "Flow cell failure detected"
    }
]

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "service": "sequencing-service",
        "timestamp": datetime.utcnow().isoformat(),
        "version": "1.0.0"
    }

@app.get("/api/v1/sequencing/jobs")
async def get_sequencing_jobs():
    """Get all sequencing jobs"""
    return {
        "success": True,
        "data": {
            "jobs": MOCK_JOBS,
            "total_count": len(MOCK_JOBS),
            "status_counts": {
                "running": len([j for j in MOCK_JOBS if j["status"] == "running"]),
                "queued": len([j for j in MOCK_JOBS if j["status"] == "queued"]),
                "completed": len([j for j in MOCK_JOBS if j["status"] == "completed"]),
                "failed": len([j for j in MOCK_JOBS if j["status"] == "failed"])
            }
        }
    }

@app.get("/api/v1/sequencing/jobs/{job_id}")
async def get_sequencing_job(job_id: str):
    """Get a specific sequencing job"""
    job = next((j for j in MOCK_JOBS if j["id"] == job_id), None)
    if not job:
        raise HTTPException(status_code=404, detail="Sequencing job not found")
    
    return {
        "success": True,
        "data": job
    }

@app.post("/api/v1/sequencing/jobs")
async def create_sequencing_job():
    """Create a new sequencing job"""
    return {
        "success": True,
        "data": {
            "message": "Sequencing job created successfully",
            "job_id": "seq-005",
            "status": "queued"
        }
    }

@app.get("/api/v1/sequencing/platforms")
async def get_sequencing_platforms():
    """Get available sequencing platforms"""
    return {
        "success": True,
        "data": {
            "platforms": [
                {
                    "id": "novaseq6000",
                    "name": "NovaSeq 6000",
                    "manufacturer": "Illumina",
                    "status": "available",
                    "throughput": "High",
                    "read_lengths": ["50bp", "100bp", "150bp"]
                },
                {
                    "id": "nextseq2000",
                    "name": "NextSeq 2000",
                    "manufacturer": "Illumina", 
                    "status": "available",
                    "throughput": "Medium",
                    "read_lengths": ["50bp", "100bp", "150bp"]
                },
                {
                    "id": "hiseqx",
                    "name": "HiSeq X",
                    "manufacturer": "Illumina",
                    "status": "maintenance",
                    "throughput": "High",
                    "read_lengths": ["150bp"]
                },
                {
                    "id": "miseq",
                    "name": "MiSeq",
                    "manufacturer": "Illumina",
                    "status": "available", 
                    "throughput": "Low",
                    "read_lengths": ["50bp", "75bp", "150bp", "250bp", "300bp"]
                }
            ]
        }
    }

# Additional endpoints for E2E testing
@app.get("/api/v1/jobs")
async def get_jobs():
    """Get all sequencing jobs (simplified endpoint for E2E testing)"""
    return {
        "success": True,
        "data": {
            "jobs": [
                {
                    "id": "seq-001",
                    "name": "WGS Sample Batch 1",
                    "status": "running",
                    "sample_count": 24,
                    "platform": "NovaSeq 6000",
                    "progress": 65,
                    "submitter": "Dr. Smith"
                },
                {
                    "id": "seq-002", 
                    "name": "RNA-seq Project Alpha",
                    "status": "queued",
                    "sample_count": 12,
                    "platform": "NextSeq 2000",
                    "progress": 0,
                    "submitter": "Dr. Johnson"
                }
            ]
        }
    }

if __name__ == "__main__":
    port = int(os.getenv("PORT", 8080))
    uvicorn.run(app, host="0.0.0.0", port=port) 