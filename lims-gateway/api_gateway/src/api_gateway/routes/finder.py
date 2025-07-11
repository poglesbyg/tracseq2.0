"""
Finder routes for the TracSeq 2.0 API Gateway.

This module provides comprehensive finder functionality for searching
and browsing all laboratory data across the LIMS system.
"""

import json
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any
from fastapi import APIRouter, Query, HTTPException, Depends

from ..core.logging import get_logger
from ..core.exceptions import DatabaseException, ValidationException


router = APIRouter()
logger = get_logger("api_gateway.routes.finder")


# Dependency to get database connection
async def get_database_connection():
    """Get database connection - placeholder for actual implementation."""
    # This would be replaced with actual database connection logic
    # For now, we'll use a mock connection
    class MockConnection:
        async def fetch(self, query: str):
            return []
        
        async def fetchval(self, query: str):
            return 0
        
        async def __aenter__(self):
            return self
        
        async def __aexit__(self, exc_type, exc_val, exc_tb):
            pass
    
    return MockConnection()


@router.get("/all-data")
async def get_all_finder_data(
    search: Optional[str] = Query(None, description="Search term to filter across all data"),
    category: Optional[str] = Query(None, description="Filter by category: samples, templates, projects, reports, storage, sequencing, qc, library"),
    limit: int = Query(1000, ge=1, le=5000, description="Maximum number of items to return"),
    offset: int = Query(0, ge=0, description="Number of items to skip"),
    conn = Depends(get_database_connection)
):
    """
    Get all laboratory data for the finder interface.
    
    This endpoint aggregates data from all services and presents it in a unified format
    that the finder can use to display all laboratory records.
    """
    try:
        all_data = []
        
        # Get samples data
        try:
            samples_query = """
                SELECT id, name, barcode, sample_type, status, created_at, updated_at, 
                       metadata, concentration, volume, quality_score, notes, template_id
                FROM samples 
                ORDER BY created_at DESC
            """
            samples_rows = await conn.fetch(samples_query)
            
            for row in samples_rows:
                metadata = row.get('metadata', {}) if row.get('metadata') else {}
                # Handle metadata - it could be a string or dict
                if isinstance(metadata, str):
                    try:
                        metadata = json.loads(metadata)
                    except:
                        metadata = {}
                
                template_data = metadata.get('template_data', {}) if isinstance(metadata, dict) else {}
                
                sample_item = {
                    "id": f"sample-{row.get('id')}",
                    "name": row.get('name'),
                    "type": "sample",
                    "category": "samples",
                    "barcode": row.get('barcode'),
                    "status": row.get('status'),
                    "sample_type": row.get('sample_type'),
                    "created_at": row.get('created_at').isoformat() if row.get('created_at') else None,
                    "updated_at": row.get('updated_at').isoformat() if row.get('updated_at') else None,
                    "concentration": template_data.get('Concentration_ng_uL') or row.get('concentration'),
                    "volume": template_data.get('Volume_mL') or row.get('volume'),
                    "quality_score": template_data.get('Quality_Score') or row.get('quality_score'),
                    "department": template_data.get('Department'),
                    "submitter": template_data.get('Submitter'),
                    "analysis_type": template_data.get('Analysis_Type'),
                    "priority": template_data.get('Priority'),
                    "notes": template_data.get('Notes') or row.get('notes'),
                    "patient_id": template_data.get('Patient_ID'),
                    "collection_date": template_data.get('Collection_Date'),
                    "storage_temp": template_data.get('Storage_Temp'),
                    "metadata": metadata
                }
                
                # Add search relevance
                if search:
                    search_text = f"{sample_item['name']} {sample_item['barcode']} {sample_item['sample_type']} {sample_item.get('department', '')} {sample_item.get('submitter', '')}".lower()
                    if search.lower() in search_text:
                        all_data.append(sample_item)
                else:
                    all_data.append(sample_item)
        except Exception as e:
            logger.error(f"Error fetching samples: {e}")
        
        # Get templates data
        try:
            templates_query = """
                SELECT id, name, description, category, status, created_at, updated_at,
                       tags, is_public, usage_count, version
                FROM templates 
                ORDER BY created_at DESC
            """
            templates_rows = await conn.fetch(templates_query)
            
            for row in templates_rows:
                template_item = {
                    "id": f"template-{row.get('id')}",
                    "name": row.get('name'),
                    "type": "template",
                    "category": "templates",
                    "description": row.get('description'),
                    "template_category": row.get('category'),
                    "status": row.get('status'),
                    "template_type": "unknown",
                    "created_at": row.get('created_at').isoformat() if row.get('created_at') else None,
                    "updated_at": row.get('updated_at').isoformat() if row.get('updated_at') else None,
                    "tags": row.get('tags', []),
                    "is_public": row.get('is_public'),
                    "usage_count": row.get('usage_count'),
                    "version": row.get('version')
                }
                
                if search:
                    search_text = f"{template_item['name']} {template_item['description']} {template_item['template_category']}".lower()
                    if search.lower() in search_text:
                        all_data.append(template_item)
                else:
                    all_data.append(template_item)
        except Exception as e:
            logger.error(f"Error fetching templates: {e}")
        
        # Get storage locations data
        try:
            storage_query = """
                SELECT id, name, zone_type, temperature_celsius, capacity, current_usage,
                       status, location_code, description, created_at, updated_at
                FROM storage_locations 
                ORDER BY created_at DESC
            """
            storage_rows = await conn.fetch(storage_query)
            
            for row in storage_rows:
                storage_item = {
                    "id": f"storage-{row.get('id')}",
                    "name": row.get('name'),
                    "type": "storage",
                    "category": "storage",
                    "zone_type": row.get('zone_type'),
                    "temperature": row.get('temperature_celsius'),
                    "capacity": row.get('capacity'),
                    "current_usage": row.get('current_usage'),
                    "utilization": round((row.get('current_usage', 0) / row.get('capacity', 1)) * 100, 1) if row.get('capacity', 0) > 0 else 0,
                    "status": row.get('status'),
                    "location_code": row.get('location_code'),
                    "description": row.get('description'),
                    "created_at": row.get('created_at').isoformat() if row.get('created_at') else None,
                    "updated_at": row.get('updated_at').isoformat() if row.get('updated_at') else None
                }
                
                if search:
                    search_text = f"{storage_item['name']} {storage_item['zone_type']} {storage_item['location_code']}".lower()
                    if search.lower() in search_text:
                        all_data.append(storage_item)
                else:
                    all_data.append(storage_item)
        except Exception as e:
            logger.error(f"Error fetching storage locations: {e}")
        
        # Get sequencing jobs data
        try:
            sequencing_query = """
                SELECT id, job_id, job_name, description, status, priority, run_type,
                       platform, sample_count, submission_date, scheduled_start,
                       completion_date, estimated_cost, actual_cost, notes
                FROM sequencing_jobs 
                ORDER BY submission_date DESC
            """
            sequencing_rows = await conn.fetch(sequencing_query)
            
            for row in sequencing_rows:
                sequencing_item = {
                    "id": f"sequencing-{row.get('id')}",
                    "name": row.get('job_name'),
                    "type": "sequencing",
                    "category": "sequencing",
                    "job_id": row.get('job_id'),
                    "description": row.get('description'),
                    "status": row.get('status'),
                    "priority": row.get('priority'),
                    "run_type": row.get('run_type'),
                    "platform": row.get('platform'),
                    "sample_count": row.get('sample_count'),
                    "submission_date": row.get('submission_date').isoformat() if row.get('submission_date') else None,
                    "scheduled_start": row.get('scheduled_start').isoformat() if row.get('scheduled_start') else None,
                    "completion_date": row.get('completion_date').isoformat() if row.get('completion_date') else None,
                    "estimated_cost": row.get('estimated_cost'),
                    "actual_cost": row.get('actual_cost'),
                    "notes": row.get('notes')
                }
                
                if search:
                    search_text = f"{sequencing_item['name']} {sequencing_item['job_id']} {sequencing_item['platform']}".lower()
                    if search.lower() in search_text:
                        all_data.append(sequencing_item)
                else:
                    all_data.append(sequencing_item)
        except Exception as e:
            logger.error(f"Error fetching sequencing jobs: {e}")
        
        # Get QC reviews data
        try:
            qc_query = """
                SELECT id, sample_id, assessment_date, overall_status, overall_score,
                       acceptance_decision, rejection_reason, assessed_by, comments
                FROM sample_quality_assessments 
                ORDER BY assessment_date DESC
            """
            qc_rows = await conn.fetch(qc_query)
            
            for row in qc_rows:
                qc_item = {
                    "id": f"qc-{row.get('id')}",
                    "name": f"QC Assessment {row.get('sample_id')}",
                    "type": "qc",
                    "category": "qc",
                    "sample_id": row.get('sample_id'),
                    "assessment_date": row.get('assessment_date').isoformat() if row.get('assessment_date') else None,
                    "status": row.get('overall_status'),
                    "score": row.get('overall_score'),
                    "decision": row.get('acceptance_decision'),
                    "rejection_reason": row.get('rejection_reason'),
                    "assessed_by": row.get('assessed_by'),
                    "comments": row.get('comments')
                }
                
                if search:
                    search_text = f"{qc_item['name']} {qc_item['sample_id']} {qc_item['decision']}".lower()
                    if search.lower() in search_text:
                        all_data.append(qc_item)
                else:
                    all_data.append(qc_item)
        except Exception as e:
            logger.error(f"Error fetching QC reviews: {e}")
        
        # Get library preparations data
        try:
            library_query = """
                SELECT id, library_id, sample_id, library_name, library_type, prep_status,
                       concentration, volume, fragment_size_bp, prep_date, prepared_by, notes
                FROM libraries 
                ORDER BY prep_date DESC
            """
            library_rows = await conn.fetch(library_query)
            
            for row in library_rows:
                library_item = {
                    "id": f"library-{row.get('id')}",
                    "name": row.get('library_name') or f"Library {row.get('library_id')}",
                    "type": "library",
                    "category": "library",
                    "library_id": row.get('library_id'),
                    "sample_id": row.get('sample_id'),
                    "library_type": row.get('library_type'),
                    "status": row.get('prep_status'),
                    "concentration": row.get('concentration'),
                    "volume": row.get('volume'),
                    "fragment_size": row.get('fragment_size_bp'),
                    "prep_date": row.get('prep_date').isoformat() if row.get('prep_date') else None,
                    "prepared_by": row.get('prepared_by'),
                    "notes": row.get('notes')
                }
                
                if search:
                    search_text = f"{library_item['name']} {library_item['library_id']} {library_item['library_type']}".lower()
                    if search.lower() in search_text:
                        all_data.append(library_item)
                else:
                    all_data.append(library_item)
        except Exception as e:
            logger.error(f"Error fetching library preparations: {e}")
        
        # Get projects data
        try:
            projects_query = """
                SELECT id, project_code, name, description, project_type, status, priority,
                       department, budget_approved, budget_used, created_at, updated_at
                FROM projects 
                ORDER BY created_at DESC
            """
            projects_rows = await conn.fetch(projects_query)
            
            for row in projects_rows:
                project_item = {
                    "id": f"project-{row.get('id')}",
                    "name": row.get('name'),
                    "type": "project",
                    "category": "projects",
                    "project_code": row.get('project_code'),
                    "description": row.get('description'),
                    "project_type": row.get('project_type'),
                    "status": row.get('status'),
                    "priority": row.get('priority'),
                    "department": row.get('department'),
                    "budget_approved": row.get('budget_approved'),
                    "budget_used": row.get('budget_used'),
                    "budget_remaining": (row.get('budget_approved', 0) - row.get('budget_used', 0)) if row.get('budget_approved') and row.get('budget_used') else None,
                    "created_at": row.get('created_at').isoformat() if row.get('created_at') else None,
                    "updated_at": row.get('updated_at').isoformat() if row.get('updated_at') else None
                }
                
                if search:
                    search_text = f"{project_item['name']} {project_item['project_code']} {project_item['department']}".lower()
                    if search.lower() in search_text:
                        all_data.append(project_item)
                else:
                    all_data.append(project_item)
        except Exception as e:
            logger.error(f"Error fetching projects: {e}")
        
        # Get reports data
        try:
            reports_query = """
                SELECT id, name, description, status, format, file_path, file_size,
                       generated_by, started_at, completed_at, error_message
                FROM generated_reports 
                ORDER BY started_at DESC
            """
            reports_rows = await conn.fetch(reports_query)
            
            for row in reports_rows:
                report_item = {
                    "id": f"report-{row.get('id')}",
                    "name": row.get('name'),
                    "type": "report",
                    "category": "reports",
                    "description": row.get('description'),
                    "status": row.get('status'),
                    "format": row.get('format'),
                    "file_path": row.get('file_path'),
                    "file_size": row.get('file_size'),
                    "generated_by": row.get('generated_by'),
                    "started_at": row.get('started_at').isoformat() if row.get('started_at') else None,
                    "completed_at": row.get('completed_at').isoformat() if row.get('completed_at') else None,
                    "error_message": row.get('error_message')
                }
                
                if search:
                    search_text = f"{report_item['name']} {report_item['description']} {report_item['format']}".lower()
                    if search.lower() in search_text:
                        all_data.append(report_item)
                else:
                    all_data.append(report_item)
        except Exception as e:
            logger.error(f"Error fetching reports: {e}")
        
        # Filter by category if specified
        if category:
            all_data = [item for item in all_data if item.get('category') == category]
        
        # Sort by most recent first
        all_data.sort(key=lambda x: x.get('created_at') or x.get('updated_at') or x.get('submission_date') or x.get('assessment_date') or '1970-01-01', reverse=True)
        
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
async def get_finder_categories(conn = Depends(get_database_connection)):
    """Get available categories and their counts for the finder."""
    try:
        categories = {}
        
        # Count samples
        samples_count = await conn.fetchval("SELECT COUNT(*) FROM samples")
        categories["samples"] = {"count": samples_count, "label": "Samples", "icon": "beaker"}
        
        # Count templates
        templates_count = await conn.fetchval("SELECT COUNT(*) FROM templates")
        categories["templates"] = {"count": templates_count, "label": "Templates", "icon": "document"}
        
        # Count storage locations
        try:
            storage_count = await conn.fetchval("SELECT COUNT(*) FROM storage_locations")
            categories["storage"] = {"count": storage_count, "label": "Storage", "icon": "archive"}
        except:
            categories["storage"] = {"count": 0, "label": "Storage", "icon": "archive"}
        
        # Count sequencing jobs
        try:
            sequencing_count = await conn.fetchval("SELECT COUNT(*) FROM sequencing_jobs")
            categories["sequencing"] = {"count": sequencing_count, "label": "Sequencing", "icon": "chart-bar"}
        except:
            categories["sequencing"] = {"count": 0, "label": "Sequencing", "icon": "chart-bar"}
        
        # Count QC assessments
        try:
            qc_count = await conn.fetchval("SELECT COUNT(*) FROM sample_quality_assessments")
            categories["qc"] = {"count": qc_count, "label": "Quality Control", "icon": "shield-check"}
        except:
            categories["qc"] = {"count": 0, "label": "Quality Control", "icon": "shield-check"}
        
        # Count library preparations
        try:
            library_count = await conn.fetchval("SELECT COUNT(*) FROM libraries")
            categories["library"] = {"count": library_count, "label": "Libraries", "icon": "test-tube"}
        except:
            categories["library"] = {"count": 0, "label": "Libraries", "icon": "test-tube"}
        
        # Count projects
        try:
            projects_count = await conn.fetchval("SELECT COUNT(*) FROM projects")
            categories["projects"] = {"count": projects_count, "label": "Projects", "icon": "folder"}
        except:
            categories["projects"] = {"count": 0, "label": "Projects", "icon": "folder"}
        
        # Count reports
        try:
            reports_count = await conn.fetchval("SELECT COUNT(*) FROM generated_reports")
            categories["reports"] = {"count": reports_count, "label": "Reports", "icon": "document-text"}
        except:
            categories["reports"] = {"count": 0, "label": "Reports", "icon": "document-text"}
        
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
    conn = Depends(get_database_connection)
):
    """Advanced search across all laboratory data."""
    try:
        # Use the existing all-data endpoint with search
        return await get_all_finder_data(search=q, category=category, limit=limit, offset=offset, conn=conn)
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
    conn = Depends(get_database_connection)
):
    """Get recently created or updated items."""
    try:
        cutoff_date = datetime.now() - timedelta(days=days)
        
        # Get recent items from all categories
        all_data = []
        
        # Recent samples
        try:
            samples_query = """
                SELECT id, name, barcode, sample_type, status, created_at, updated_at
                FROM samples 
                WHERE created_at >= %s OR updated_at >= %s
                ORDER BY GREATEST(created_at, updated_at) DESC
                LIMIT %s
            """
            samples_rows = await conn.fetch(samples_query, cutoff_date, cutoff_date, limit)
            
            for row in samples_rows:
                all_data.append({
                    "id": f"sample-{row.get('id')}",
                    "name": row.get('name'),
                    "type": "sample",
                    "category": "samples",
                    "barcode": row.get('barcode'),
                    "status": row.get('status'),
                    "created_at": row.get('created_at').isoformat() if row.get('created_at') else None,
                    "updated_at": row.get('updated_at').isoformat() if row.get('updated_at') else None
                })
        except Exception as e:
            logger.error(f"Error fetching recent samples: {e}")
        
        # Sort by most recent
        all_data.sort(key=lambda x: x.get('updated_at') or x.get('created_at') or '1970-01-01', reverse=True)
        
        return {
            "success": True,
            "data": all_data[:limit],
            "days": days,
            "total_items": len(all_data)
        }
        
    except Exception as e:
        logger.error(f"Error in get_recent_finder_data: {e}")
        raise DatabaseException(
            f"Failed to retrieve recent finder data: {str(e)}",
            operation="get_recent_finder_data"
        )


@router.get("/stats")
async def get_finder_stats(conn = Depends(get_database_connection)):
    """Get finder statistics and overview."""
    try:
        stats = {}
        
        # Get category counts
        categories = await get_finder_categories(conn)
        stats["categories"] = categories["categories"]
        stats["total_items"] = categories["total_items"]
        
        # Get recent activity (last 7 days)
        recent_data = await get_recent_finder_data(days=7, limit=100, conn=conn)
        stats["recent_activity"] = {
            "items_last_7_days": len(recent_data["data"]),
            "categories_with_activity": len(set(item["category"] for item in recent_data["data"]))
        }
        
        # Get system health indicators
        stats["system_health"] = {
            "database_responsive": True,
            "last_updated": datetime.now().isoformat(),
            "data_freshness": "current"
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