from fastapi import FastAPI, HTTPException
from datetime import datetime
import uvicorn
import os
import sys
import logging

from logging_config import (
    setup_logging, 
    RequestLoggingMiddleware, 
    log_performance, 
    log_business_event,
    log_health_check,
    get_logger
)

app = FastAPI(title="Dashboard Service", version="1.0.0")

# Setup structured logging
logger = setup_logging("dashboard-service", os.getenv("LOG_LEVEL", "INFO"))

# Add request logging middleware
app.add_middleware(RequestLoggingMiddleware, service_name="dashboard-service")

# Log service startup
logger.info("Dashboard service starting up", extra={"version": "1.0.0"})

@app.get("/health")
@log_performance("health_check")
async def health_check():
    """Health check endpoint"""
    
    health_status = {
        "status": "healthy",
        "service": "dashboard-service",
        "timestamp": datetime.utcnow().isoformat(),
        "version": "1.0.0"
    }
    
    # Log health check
    log_health_check("dashboard-service", "healthy", {
        "response_time_ms": 0,
        "memory_usage": "normal",
        "database_connection": "ok"
    })
    
    return health_status

@app.get("/api/dashboard")
async def get_dashboard():
    """Get dashboard overview"""
    return {
        "success": True,
        "data": {
            "overview": {
                "totalSamples": 847,
                "totalTemplates": 12,
                "pendingSequencing": 23,
                "completedSequencing": 156
            },
            "recentActivity": [
                {
                    "id": "act-001",
                    "type": "sample_created",
                    "description": "New DNA sample created",
                    "timestamp": "2025-07-11T21:00:00Z",
                    "user": "Dr. Smith"
                },
                {
                    "id": "act-002",
                    "type": "sequencing_completed",
                    "description": "RNA sequencing completed",
                    "timestamp": "2025-07-11T20:30:00Z",
                    "user": "Lab System"
                }
            ]
        }
    }

@app.get("/api/dashboard/metrics")
async def get_dashboard_metrics():
    """Get dashboard metrics"""
    return {
        "success": True,
        "data": {
            "throughput": {
                "last24h": 12,
                "last7d": 89,
                "last30d": 347
            },
            "processing_times": {
                "validation": 2.3,
                "storage": 1.8,
                "sequencing": 24.5,
                "overall": 28.6
            },
            "quality_metrics": {
                "success_rate": 0.95,
                "error_rate": 0.05,
                "average_quality": 0.92
            }
        }
    }

@app.get("/api/dashboard/kpis")
async def get_dashboard_kpis():
    """Get dashboard KPIs"""
    return {
        "success": True,
        "data": {
            "samples": {
                "total": 847,
                "by_status": {
                    "pending": 45,
                    "validated": 123,
                    "in_storage": 234,
                    "in_sequencing": 23,
                    "completed": 422
                },
                "by_type": {
                    "DNA": 456,
                    "RNA": 234,
                    "Protein": 157
                }
            },
            "performance": {
                "bottlenecks": [
                    {
                        "stage": "sequencing",
                        "count": 23,
                        "avg_wait_time": 4.2
                    },
                    {
                        "stage": "validation",
                        "count": 12,
                        "avg_wait_time": 2.1
                    }
                ],
                "capacity_utilization": 0.78
            }
        }
    }

@app.get("/dashboard/stats")
async def get_dashboard_stats():
    """Get dashboard statistics (legacy endpoint)"""
    return {
        "totalTemplates": 12,
        "totalSamples": 847,
        "pendingSequencing": 23,
        "completedSequencing": 156,
        "byStatus": {
            "pending": 45,
            "validated": 123,
            "in_storage": 234,
            "in_sequencing": 23,
            "completed": 422
        },
        "averageProcessingTime": {
            "validation": 2.3,
            "storage": 1.8,
            "sequencing": 24.5,
            "overall": 28.6
        },
        "recentThroughput": {
            "last24h": 12,
            "last7d": 89,
            "last30d": 347
        },
        "bottlenecks": [
            {
                "stage": "sequencing",
                "count": 23,
                "avg_wait_time": 4.2
            },
            {
                "stage": "validation",
                "count": 12,
                "avg_wait_time": 2.1
            }
        ],
        "recentActivity": [
            {
                "id": "act-001",
                "type": "sample_created",
                "description": "New DNA sample created",
                "timestamp": "2025-07-11T21:00:00Z",
                "user": "Dr. Smith"
            },
            {
                "id": "act-002",
                "type": "sequencing_completed",
                "description": "RNA sequencing completed",
                "timestamp": "2025-07-11T20:30:00Z",
                "user": "Lab System"
            },
            {
                "id": "act-003",
                "type": "template_uploaded",
                "description": "New template uploaded",
                "timestamp": "2025-07-11T19:45:00Z",
                "user": "Lab Manager"
            }
        ]
    }

# Additional endpoints for E2E testing
@app.get("/api/v1/users")
@log_performance("get_users")
async def get_users():
    """Get all users"""
    
    users_logger = get_logger("users")
    users_logger.info("Fetching all users")
    
    users_data = [
        {
            "id": "user-001",
            "name": "Dr. Smith",
            "email": "smith@lab.com",
            "department": "Genomics",
            "role": "researcher",
            "created_at": "2024-01-15T10:00:00Z"
        },
        {
            "id": "user-002", 
            "name": "Dr. Johnson",
            "email": "johnson@lab.com",
            "department": "Transcriptomics",
            "role": "researcher",
            "created_at": "2024-02-10T09:30:00Z"
        }
    ]
    
    # Log business event
    log_business_event("users_retrieved", {
        "count": len(users_data),
        "departments": ["Genomics", "Transcriptomics"]
    })
    
    return {
        "success": True,
        "data": {
            "users": users_data
        }
    }

@app.get("/api/v1/storage/locations")
async def get_storage_locations():
    """Get all storage locations"""
    return {
        "success": True,
        "data": {
            "locations": [
                {
                    "id": "loc-001",
                    "name": "Freezer A1-B2",
                    "temperature": -80,
                    "capacity": 100,
                    "current_usage": 45,
                    "status": "operational"
                },
                {
                    "id": "loc-002",
                    "name": "Freezer B2-C3", 
                    "temperature": -20,
                    "capacity": 200,
                    "current_usage": 78,
                    "status": "operational"
                }
            ]
        }
    }

@app.get("/api/v1/samples")
async def get_samples_dashboard():
    """Get samples for dashboard view"""
    return {
        "success": True,
        "data": {
            "samples": [
                {
                    "id": "sample-001",
                    "name": "DNA Sample 001",
                    "status": "validated",
                    "department": "Genomics",
                    "created_at": "2024-12-15T10:30:00Z"
                }
            ]
        }
    }

@app.get("/api/v1/status")
async def get_auth_status():
    """Get authentication status"""
    return {
        "success": True,
        "data": {
            "authenticated": True,
            "user": {
                "id": "user-001",
                "name": "Test User",
                "role": "researcher"
            },
            "session": {
                "expires_at": "2024-12-16T10:00:00Z",
                "token_type": "Bearer"
            }
        }
    }

if __name__ == "__main__":
    port = int(os.getenv("PORT", 8080))
    uvicorn.run(app, host="0.0.0.0", port=port) 