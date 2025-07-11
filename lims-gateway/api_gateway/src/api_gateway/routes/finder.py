"""
Finder routes for the TracSeq 2.0 API Gateway.

This module provides comprehensive finder functionality for searching
and browsing all laboratory data across the LIMS system.
"""

import json
import httpx
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any
from fastapi import APIRouter, Query, HTTPException, Depends

from ..core.logging import get_logger
from ..core.exceptions import DatabaseException, ValidationException


router = APIRouter()
logger = get_logger("api_gateway.routes.finder")

# Service URLs
TEMPLATE_SERVICE_URL = "http://lims-templates:8000"
SAMPLE_SERVICE_URL = "http://lims-gateway-new:8000"  # Use API Gateway for samples
STORAGE_SERVICE_URL = "http://lims-storage:8082"
RAG_SERVICE_URL = "http://lims-rag:8000"


@router.get("/all-data")
async def get_all_finder_data(
    search: Optional[str] = Query(None, description="Search term to filter across all data"),
    category: Optional[str] = Query(None, description="Filter by category: samples, templates, projects, reports, storage, sequencing, qc, library"),
    limit: int = Query(1000, ge=1, le=5000, description="Maximum number of items to return"),
    offset: int = Query(0, ge=0, description="Number of items to skip"),
):
    """
    Get all laboratory data for the finder interface.
    
    This endpoint aggregates data from all services and presents it in a unified format
    that the finder can use to display all laboratory records.
    """
    try:
        all_data = []
        
        # Get templates data from templates service
        try:
            async with httpx.AsyncClient() as client:
                response = await client.get(f"{TEMPLATE_SERVICE_URL}/templates", timeout=10.0)
                if response.status_code == 200:
                    templates_data = response.json()
                    templates_list = templates_data.get('data', []) if isinstance(templates_data, dict) else templates_data
                    
                    for template in templates_list:
                        template_item = {
                            "id": f"template-{template.get('id')}",
                            "name": template.get('name', 'Unknown Template'),
                            "type": "template",
                            "category": "templates",
                            "description": template.get('description', ''),
                            "template_category": template.get('category', 'general'),
                            "status": template.get('status', 'active'),
                            "template_type": template.get('type', 'spreadsheet'),
                            "created_at": template.get('created_at'),
                            "updated_at": template.get('updated_at'),
                            "tags": template.get('tags', []),
                            "is_public": template.get('is_public', True),
                            "usage_count": template.get('usage_count', 0),
                            "version": template.get('version', '1.0'),
                            "file_path": template.get('file_path', ''),
                            "file_type": template.get('file_type', 'xlsx')
                        }
                        
                        if search:
                            search_text = f"{template_item['name']} {template_item['description']} {template_item['template_category']}".lower()
                            if search.lower() in search_text:
                                all_data.append(template_item)
                        else:
                            all_data.append(template_item)
        except Exception as e:
            logger.error(f"Error fetching templates: {e}")
        
        # Get samples data from samples service
        try:
            async with httpx.AsyncClient() as client:
                response = await client.get(f"{SAMPLE_SERVICE_URL}/api/samples", timeout=10.0)
                if response.status_code == 200:
                    samples_data = response.json()
                    samples_list = samples_data.get('data', []) if isinstance(samples_data, dict) else samples_data
                    
                    for sample in samples_list:
                        sample_item = {
                            "id": f"sample-{sample.get('id')}",
                            "name": sample.get('name', 'Unknown Sample'),
                            "type": "sample",
                            "category": "samples",
                            "barcode": sample.get('barcode', ''),
                            "status": sample.get('status', 'pending'),
                            "sample_type": sample.get('sample_type', 'unknown'),
                            "created_at": sample.get('created_at'),
                            "updated_at": sample.get('updated_at'),
                            "concentration": sample.get('concentration'),
                            "volume": sample.get('volume'),
                            "quality_score": sample.get('quality_score'),
                            "department": sample.get('department'),
                            "submitter": sample.get('submitter'),
                            "analysis_type": sample.get('analysis_type'),
                            "priority": sample.get('priority'),
                            "notes": sample.get('notes'),
                            "patient_id": sample.get('patient_id'),
                            "collection_date": sample.get('collection_date'),
                            "storage_temp": sample.get('storage_temp'),
                            "storage_location": sample.get('storage_location'),
                            "metadata": sample.get('metadata', {})
                        }
                        
                        if search:
                            search_text = f"{sample_item['name']} {sample_item['barcode']} {sample_item['sample_type']} {sample_item.get('department', '')} {sample_item.get('submitter', '')}".lower()
                            if search.lower() in search_text:
                                all_data.append(sample_item)
                        else:
                            all_data.append(sample_item)
        except Exception as e:
            logger.error(f"Error fetching samples: {e}")
        
        # Get storage data from storage service
        try:
            async with httpx.AsyncClient() as client:
                response = await client.get(f"{STORAGE_SERVICE_URL}/api/storage/locations", timeout=10.0)
                if response.status_code == 200:
                    storage_data = response.json()
                    storage_list = storage_data.get('data', []) if isinstance(storage_data, dict) else storage_data
                    
                    for storage in storage_list:
                        storage_item = {
                            "id": f"storage-{storage.get('id')}",
                            "name": storage.get('name', 'Unknown Location'),
                            "type": "storage",
                            "category": "storage",
                            "zone_type": storage.get('zone_type', 'general'),
                            "temperature": storage.get('temperature_celsius'),
                            "capacity": storage.get('capacity', 0),
                            "current_usage": storage.get('current_usage', 0),
                            "utilization": round((storage.get('current_usage', 0) / storage.get('capacity', 1)) * 100, 1) if storage.get('capacity', 0) > 0 else 0,
                            "status": storage.get('status', 'active'),
                            "location_code": storage.get('location_code', ''),
                            "description": storage.get('description', ''),
                            "created_at": storage.get('created_at'),
                            "updated_at": storage.get('updated_at')
                        }
                        
                        if search:
                            search_text = f"{storage_item['name']} {storage_item['zone_type']} {storage_item['location_code']}".lower()
                            if search.lower() in search_text:
                                all_data.append(storage_item)
                        else:
                            all_data.append(storage_item)
        except Exception as e:
            logger.error(f"Error fetching storage locations: {e}")
        
        # Get RAG submissions data
        try:
            async with httpx.AsyncClient() as client:
                response = await client.get(f"{RAG_SERVICE_URL}/api/rag/submissions", timeout=10.0)
                if response.status_code == 200:
                    rag_data = response.json()
                    submissions_list = rag_data.get('submissions', []) if isinstance(rag_data, dict) else rag_data
                    
                    for submission in submissions_list:
                        rag_item = {
                            "id": f"rag-{submission.get('id')}",
                            "name": f"RAG Submission {submission.get('submission_id', 'Unknown')}",
                            "type": "document",
                            "category": "documents",
                            "submission_id": submission.get('submission_id'),
                            "source_document": submission.get('source_document'),
                            "submitter_name": submission.get('submitter_name'),
                            "submitter_email": submission.get('submitter_email'),
                            "sample_type": submission.get('sample_type'),
                            "confidence_score": submission.get('confidence_score'),
                            "status": submission.get('status'),
                            "created_at": submission.get('created_at'),
                            "processing_time": submission.get('processing_time'),
                            "extracted_data": submission.get('extracted_data', {})
                        }
                        
                        if search:
                            search_text = f"{rag_item['name']} {rag_item['source_document']} {rag_item['submitter_name']}".lower()
                            if search.lower() in search_text:
                                all_data.append(rag_item)
                        else:
                            all_data.append(rag_item)
        except Exception as e:
            logger.error(f"Error fetching RAG submissions: {e}")
        
        # Filter by category if specified
        if category:
            all_data = [item for item in all_data if item.get('category') == category]
        
        # Sort by most recent first
        all_data.sort(key=lambda x: x.get('created_at') or x.get('updated_at') or '1970-01-01', reverse=True)
        
        # Apply pagination
        total_count = len(all_data)
        paginated_data = all_data[offset:offset + limit]
        
        return {
            "success": True,
            "data": paginated_data,
            "pagination": {
                "total": total_count,
                "offset": offset,
                "limit": limit,
                "has_more": offset + limit < total_count
            },
            "categories": {
                "samples": len([item for item in all_data if item.get('category') == 'samples']),
                "templates": len([item for item in all_data if item.get('category') == 'templates']),
                "storage": len([item for item in all_data if item.get('category') == 'storage']),
                "documents": len([item for item in all_data if item.get('category') == 'documents']),
                "sequencing": len([item for item in all_data if item.get('category') == 'sequencing']),
                "qc": len([item for item in all_data if item.get('category') == 'qc']),
                "library": len([item for item in all_data if item.get('category') == 'library']),
                "projects": len([item for item in all_data if item.get('category') == 'projects']),
                "reports": len([item for item in all_data if item.get('category') == 'reports'])
            }
        }
        
    except Exception as e:
        logger.error(f"Error in get_all_finder_data: {e}")
        raise DatabaseException(
            f"Failed to retrieve finder data: {str(e)}",
            operation="get_all_finder_data"
        )


@router.get("/categories")
async def get_finder_categories():
    """Get available categories and their counts for the finder."""
    try:
        # For now, return static categories since we're fetching from services
        categories = {
            "samples": {"count": 0, "label": "Samples", "icon": "beaker"},
            "templates": {"count": 0, "label": "Templates", "icon": "document"},
            "storage": {"count": 0, "label": "Storage", "icon": "archive"},
            "documents": {"count": 0, "label": "Documents", "icon": "document-text"},
            "sequencing": {"count": 0, "label": "Sequencing", "icon": "chart-bar"},
            "qc": {"count": 0, "label": "Quality Control", "icon": "shield-check"},
            "library": {"count": 0, "label": "Libraries", "icon": "test-tube"},
            "projects": {"count": 0, "label": "Projects", "icon": "folder"},
            "reports": {"count": 0, "label": "Reports", "icon": "document-text"}
        }
        
        # Get actual counts from services
        try:
            async with httpx.AsyncClient() as client:
                # Count templates
                response = await client.get(f"{TEMPLATE_SERVICE_URL}/templates", timeout=5.0)
                if response.status_code == 200:
                    templates_data = response.json()
                    templates_list = templates_data.get('data', []) if isinstance(templates_data, dict) else templates_data
                    categories["templates"]["count"] = len(templates_list)
                
                # Count samples
                response = await client.get(f"{SAMPLE_SERVICE_URL}/api/samples", timeout=5.0)
                if response.status_code == 200:
                    samples_data = response.json()
                    samples_list = samples_data.get('data', []) if isinstance(samples_data, dict) else samples_data
                    categories["samples"]["count"] = len(samples_list)
                
                # Count storage locations
                response = await client.get(f"{STORAGE_SERVICE_URL}/api/storage/locations", timeout=5.0)
                if response.status_code == 200:
                    storage_data = response.json()
                    storage_list = storage_data.get('data', []) if isinstance(storage_data, dict) else storage_data
                    categories["storage"]["count"] = len(storage_list)
                
                # Count RAG submissions
                response = await client.get(f"{RAG_SERVICE_URL}/api/rag/submissions", timeout=5.0)
                if response.status_code == 200:
                    rag_data = response.json()
                    submissions_list = rag_data.get('submissions', []) if isinstance(rag_data, dict) else rag_data
                    categories["documents"]["count"] = len(submissions_list)
        except Exception as e:
            logger.error(f"Error fetching category counts: {e}")

        return {
            "success": True,
            "categories": categories,
            "total_items": sum(cat["count"] for cat in categories.values())
        }
        
    except Exception as e:
        logger.error(f"Error in get_finder_categories: {e}")
        raise DatabaseException(
            f"Failed to retrieve finder categories: {str(e)}",
            operation="get_finder_categories"
        )


@router.get("/search")
async def search_finder_data(
    q: str = Query(..., description="Search query"),
    category: Optional[str] = Query(None, description="Filter by category"),
    limit: int = Query(50, ge=1, le=200),
    offset: int = Query(0, ge=0),
):
    """Advanced search across all laboratory data."""
    try:
        # Use the existing all-data endpoint with search
        return await get_all_finder_data(search=q, category=category, limit=limit, offset=offset)
    except Exception as e:
        logger.error(f"Error in search_finder_data: {e}")
        raise DatabaseException(
            f"Failed to search finder data: {str(e)}",
            operation="search_finder_data"
        )


@router.get("/recent")
async def get_recent_finder_data(
    days: int = Query(30, ge=1, le=365, description="Number of days to look back"),
    limit: int = Query(20, ge=1, le=100),
):
    """Get recently created or updated items."""
    try:
        # For now, just get recent data from the all-data endpoint
        all_data_response = await get_all_finder_data(limit=limit * 2)  # Get more to filter by date
        all_data = all_data_response["data"]
        
        # Filter by recent items (this is a simplified approach)
        cutoff_date = datetime.now() - timedelta(days=days)
        recent_items = []
        
        for item in all_data:
            item_date = None
            if item.get('created_at'):
                try:
                    item_date = datetime.fromisoformat(item['created_at'].replace('Z', '+00:00'))
                except:
                    pass
            elif item.get('updated_at'):
                try:
                    item_date = datetime.fromisoformat(item['updated_at'].replace('Z', '+00:00'))
                except:
                    pass
            
            if item_date and item_date >= cutoff_date:
                recent_items.append(item)
        
        # Sort by most recent and limit
        recent_items.sort(key=lambda x: x.get('updated_at') or x.get('created_at') or '1970-01-01', reverse=True)
        recent_items = recent_items[:limit]

        return {
            "success": True,
            "data": recent_items,
            "days": days,
            "total_items": len(recent_items)
        }
        
    except Exception as e:
        logger.error(f"Error in get_recent_finder_data: {e}")
        raise DatabaseException(
            f"Failed to retrieve recent finder data: {str(e)}",
            operation="get_recent_finder_data"
        )


@router.get("/stats")
async def get_finder_stats():
    """Get finder statistics and overview."""
    try:
        # Get category counts
        categories = await get_finder_categories()
        
        # Get recent activity (last 7 days)
        recent_data = await get_recent_finder_data(days=7, limit=100)
        
        stats = {
            "categories": categories["categories"],
            "total_items": categories["total_items"],
            "recent_activity": {
                "items_last_7_days": len(recent_data["data"]),
                "categories_with_activity": len(set(item["category"] for item in recent_data["data"]))
            },
            "system_health": {
                "database_responsive": True,
                "last_updated": datetime.now().isoformat(),
                "data_freshness": "current"
            }
        }
        
        return {
            "success": True,
            "stats": stats
        }
        
    except Exception as e:
        logger.error(f"Error in get_finder_stats: {e}")
        raise DatabaseException(
            f"Failed to retrieve finder statistics: {str(e)}",
            operation="get_finder_stats"
        )


# Export the router
__all__ = ["router"]