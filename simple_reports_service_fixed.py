#!/usr/bin/env python3
import http.server
import socketserver
import json
import os
from urllib.parse import urlparse, parse_qs

class ReportsHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        self.handle_request()
    
    def do_POST(self):
        self.handle_request()
    
    def handle_request(self):
        path = self.path
        
        # Health check
        if path == '/health':
            self.send_json_response({
                "status": "healthy",
                "service": "reports-service",
                "version": "0.2.0-python"
            })
        
        # Reports endpoints - handle both /api/reports/* and /reports/* paths
        elif path in ['/api/reports/health', '/reports/health']:
            self.send_json_response({
                "status": "healthy",
                "service": "reports",
                "version": "0.2.0-python",
                "features": {
                    "templates": True,
                    "analytics": True,
                    "export": True,
                    "scheduling": True
                }
            })
        
        elif path in ['/api/reports/templates', '/reports/templates']:
            self.send_json_response({
                "templates": [
                    {
                        "id": "TPL-001",
                        "name": "Sample Summary Report",
                        "description": "Comprehensive summary of sample processing",
                        "category": "samples",
                        "format": "pdf"
                    },
                    {
                        "id": "TPL-002",
                        "name": "Storage Utilization Report",
                        "description": "Storage capacity and usage analysis",
                        "category": "storage",
                        "format": "excel"
                    },
                    {
                        "id": "TPL-003",
                        "name": "Sequencing Metrics Report",
                        "description": "Detailed sequencing performance metrics",
                        "category": "sequencing",
                        "format": "pdf"
                    },
                    {
                        "id": "TPL-004",
                        "name": "Financial Summary Report",
                        "description": "Cost analysis and billing summary",
                        "category": "financial",
                        "format": "excel"
                    },
                    {
                        "id": "TPL-005",
                        "name": "Performance Analytics Report",
                        "description": "Laboratory performance and efficiency metrics",
                        "category": "performance",
                        "format": "pdf"
                    }
                ],
                "total": 5,
                "success": True
            })
        
        elif path in ['/api/reports/analytics/samples', '/reports/analytics/samples']:
            self.send_json_response({
                "analytics": {
                    "total_samples": 1247,
                    "samples_by_type": {
                        "DNA": 623,
                        "RNA": 401,
                        "Protein": 156,
                        "Tissue": 67
                    },
                    "samples_by_status": {
                        "pending": 89,
                        "validated": 156,
                        "in_storage": 834,
                        "in_sequencing": 123,
                        "completed": 45
                    },
                    "processing_time_avg": "2.3 hours",
                    "success_rate": 98.7
                },
                "success": True
            })
        
        elif path in ['/api/reports/analytics/sequencing', '/reports/analytics/sequencing']:
            self.send_json_response({
                "analytics": {
                    "total_runs": 234,
                    "successful_runs": 228,
                    "failed_runs": 6,
                    "success_rate": 97.4,
                    "average_quality_score": 38.2,
                    "platforms": {
                        "NovaSeq": 156,
                        "MiSeq": 78
                    },
                    "throughput_gb": 15420.5
                },
                "success": True
            })
        
        elif path in ['/api/reports/analytics/storage', '/reports/analytics/storage']:
            self.send_json_response({
                "analytics": {
                    "total_capacity": "95.2TB",
                    "used_capacity": "67.8TB",
                    "utilization_percent": 71.2,
                    "zones": {
                        "-80C": {"capacity": "25TB", "used": "18.2TB", "utilization": 72.8},
                        "-20C": {"capacity": "30TB", "used": "21.5TB", "utilization": 71.7},
                        "4C": {"capacity": "25TB", "used": "17.8TB", "utilization": 71.2},
                        "RT": {"capacity": "15.2TB", "used": "10.3TB", "utilization": 67.8}
                    },
                    "access_frequency": {
                        "daily": 1247,
                        "weekly": 8934,
                        "monthly": 2156
                    }
                },
                "success": True
            })
        
        elif path in ['/api/reports/analytics/financial', '/reports/analytics/financial']:
            self.send_json_response({
                "analytics": {
                    "total_revenue": 1247832.50,
                    "total_costs": 892156.75,
                    "profit_margin": 28.5,
                    "cost_breakdown": {
                        "reagents": 345678.90,
                        "equipment": 123456.78,
                        "personnel": 234567.89,
                        "utilities": 87653.21,
                        "maintenance": 100799.97
                    },
                    "revenue_by_service": {
                        "sequencing": 756234.50,
                        "storage": 234567.89,
                        "sample_prep": 156789.23,
                        "analysis": 100240.88
                    }
                },
                "success": True
            })
        
        elif path in ['/api/reports/analytics/performance', '/reports/analytics/performance']:
            self.send_json_response({
                "analytics": {
                    "throughput": {
                        "samples_per_day": 127.5,
                        "samples_per_week": 892.5,
                        "samples_per_month": 3847.2
                    },
                    "efficiency": {
                        "processing_time_avg": "2.3 hours",
                        "queue_time_avg": "0.8 hours",
                        "total_turnaround": "3.1 hours"
                    },
                    "quality_metrics": {
                        "error_rate": 1.3,
                        "rework_rate": 2.1,
                        "customer_satisfaction": 4.7
                    },
                    "resource_utilization": {
                        "equipment": 78.5,
                        "personnel": 82.3,
                        "storage": 71.2
                    }
                },
                "success": True
            })
        
        elif path in ['/api/reports', '/reports']:
            self.send_json_response({
                "reports": [
                    {
                        "id": "RPT-2024-001",
                        "title": "Sample Processing Summary",
                        "status": "completed",
                        "created_at": "2024-01-15T10:30:00Z",
                        "format": "pdf"
                    },
                    {
                        "id": "RPT-2024-002",
                        "title": "Storage Utilization Report",
                        "status": "generating",
                        "created_at": "2024-01-15T11:00:00Z",
                        "format": "excel"
                    }
                ],
                "total": 2,
                "success": True
            })
        
        elif (path in ['/api/reports/export/pdf', '/reports/export/pdf'] and self.command == 'POST') or \
             (path in ['/api/reports/generate', '/reports/generate'] and self.command == 'POST'):
            self.send_json_response({
                "export_id": "EXP-PDF-001",
                "format": "pdf",
                "status": "generating",
                "download_url": "/downloads/report.pdf",
                "success": True
            })
        
        elif path in ['/api/reports/export/excel', '/reports/export/excel'] and self.command == 'POST':
            self.send_json_response({
                "export_id": "EXP-XLS-001",
                "format": "excel",
                "status": "generating",
                "download_url": "/downloads/report.xlsx",
                "success": True
            })
        
        elif path in ['/api/reports/export/csv', '/reports/export/csv'] and self.command == 'POST':
            self.send_json_response({
                "export_id": "EXP-CSV-001",
                "format": "csv",
                "status": "completed",
                "download_url": "/downloads/report.csv",
                "file_size": "1.2MB",
                "success": True
            })
        
        elif path == '/':
            self.send_response(200)
            self.send_header('Content-type', 'text/plain')
            self.end_headers()
            self.wfile.write(b'Enhanced Reports Service - Python Version')
        
        else:
            self.send_json_response({
                "error": "Not Found",
                "message": f"Endpoint not available: {path}"
            }, status=404)
    
    def send_json_response(self, data, status=200):
        self.send_response(status)
        self.send_header('Content-type', 'application/json')
        self.end_headers()
        self.wfile.write(json.dumps(data).encode())

if __name__ == "__main__":
    PORT = int(os.environ.get('REPORTS_PORT', 8000))
    
    print(f"📊 Starting Enhanced Reports Service - Python Version (Fixed)")
    print(f"🚀 Enhanced Reports Service listening on 0.0.0.0:{PORT}")
    
    with socketserver.TCPServer(("", PORT), ReportsHandler) as httpd:
        httpd.serve_forever()
