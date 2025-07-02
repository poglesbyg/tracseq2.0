#!/usr/bin/env python3
"""
Sample Analyzer MCP Service
An example MCP-enabled service that analyzes laboratory samples.
"""

import asyncio
import logging
from datetime import datetime
from typing import Dict, Any, List
import httpx
from aiohttp import web
import json

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class SampleAnalyzerService:
    def __init__(self, service_name="sample_analyzer", port=8020):
        self.service_name = service_name
        self.port = port
        self.mcp_proxy_url = "http://mcp-proxy:9500"
        self.app = web.Application()
        self.setup_routes()
        
    def setup_routes(self):
        """Setup HTTP routes for the service."""
        self.app.router.add_get('/health', self.handle_health)
        self.app.router.add_post('/mcp/tools/analyze_sample', self.analyze_sample)
        self.app.router.add_post('/mcp/tools/batch_analyze', self.batch_analyze)
        self.app.router.add_post('/mcp/tools/get_analysis_report', self.get_analysis_report)
        
    async def register_with_proxy(self):
        """Register this service with the MCP proxy."""
        registration_data = {
            "name": self.service_name,
            "endpoint": f"http://{self.service_name}:{self.port}",
            "transport": "http",
            "status": "online",
            "capabilities": [
                "analyze_sample",
                "batch_analyze",
                "get_analysis_report"
            ]
        }
        
        try:
            async with httpx.AsyncClient() as client:
                response = await client.post(
                    f"{self.mcp_proxy_url}/register",
                    json=registration_data
                )
                if response.status_code == 200:
                    logger.info(f"Successfully registered {self.service_name} with MCP proxy")
                else:
                    logger.error(f"Failed to register: {response.text}")
        except Exception as e:
            logger.error(f"Error registering with proxy: {e}")
            
    async def handle_health(self, request):
        """Health check endpoint."""
        return web.json_response({
            "status": "healthy",
            "service": self.service_name,
            "timestamp": datetime.now().isoformat()
        })
        
    async def analyze_sample(self, request):
        """
        Analyze a single laboratory sample.
        
        Expected input:
        {
            "sample_id": "SMPL-12345",
            "sample_type": "blood",
            "tests_requested": ["CBC", "glucose", "cholesterol"]
        }
        """
        try:
            data = await request.json()
            sample_id = data.get("sample_id")
            sample_type = data.get("sample_type")
            tests = data.get("tests_requested", [])
            
            # Simulate analysis with AI
            analysis_results = {
                "sample_id": sample_id,
                "sample_type": sample_type,
                "analysis_date": datetime.now().isoformat(),
                "results": {}
            }
            
            # Mock results based on test type
            for test in tests:
                if test == "CBC":
                    analysis_results["results"][test] = {
                        "wbc": 7.2,
                        "rbc": 4.8,
                        "hemoglobin": 14.5,
                        "platelets": 250,
                        "status": "normal"
                    }
                elif test == "glucose":
                    analysis_results["results"][test] = {
                        "value": 95,
                        "unit": "mg/dL",
                        "status": "normal",
                        "reference_range": "70-100"
                    }
                elif test == "cholesterol":
                    analysis_results["results"][test] = {
                        "total": 180,
                        "ldl": 100,
                        "hdl": 60,
                        "triglycerides": 100,
                        "unit": "mg/dL",
                        "status": "normal"
                    }
                    
            # Here you could integrate with Ollama for AI analysis
            if data.get("ai_interpretation", False):
                analysis_results["ai_interpretation"] = await self.get_ai_interpretation(analysis_results)
                
            return web.json_response({
                "success": True,
                "data": analysis_results
            })
            
        except Exception as e:
            logger.error(f"Error analyzing sample: {e}")
            return web.json_response({
                "success": False,
                "error": str(e)
            }, status=500)
            
    async def batch_analyze(self, request):
        """Analyze multiple samples in batch."""
        try:
            data = await request.json()
            samples = data.get("samples", [])
            
            results = []
            for sample in samples:
                # Process each sample
                result = {
                    "sample_id": sample.get("sample_id"),
                    "status": "analyzed",
                    "completion_time": datetime.now().isoformat()
                }
                results.append(result)
                
            return web.json_response({
                "success": True,
                "batch_id": f"BATCH-{datetime.now().strftime('%Y%m%d%H%M%S')}",
                "total_samples": len(samples),
                "results": results
            })
            
        except Exception as e:
            logger.error(f"Error in batch analysis: {e}")
            return web.json_response({
                "success": False,
                "error": str(e)
            }, status=500)
            
    async def get_analysis_report(self, request):
        """Generate a comprehensive analysis report."""
        try:
            data = await request.json()
            sample_ids = data.get("sample_ids", [])
            report_format = data.get("format", "json")
            
            report = {
                "report_id": f"RPT-{datetime.now().strftime('%Y%m%d%H%M%S')}",
                "generated_at": datetime.now().isoformat(),
                "sample_count": len(sample_ids),
                "samples": sample_ids,
                "summary": {
                    "total_tests": len(sample_ids) * 3,  # Mock calculation
                    "abnormal_results": 0,
                    "pending_reviews": 0
                }
            }
            
            return web.json_response({
                "success": True,
                "report": report
            })
            
        except Exception as e:
            logger.error(f"Error generating report: {e}")
            return web.json_response({
                "success": False,
                "error": str(e)
            }, status=500)
            
    async def get_ai_interpretation(self, analysis_results):
        """Get AI interpretation using Ollama."""
        try:
            # Connect to Ollama
            async with httpx.AsyncClient() as client:
                prompt = f"""
                Provide a brief medical interpretation of these lab results:
                {json.dumps(analysis_results['results'], indent=2)}
                
                Format: Brief summary with any notable findings.
                """
                
                response = await client.post(
                    "http://ollama:11434/api/generate",
                    json={
                        "model": "llama3.2:3b",
                        "prompt": prompt,
                        "stream": False
                    },
                    timeout=30.0
                )
                
                if response.status_code == 200:
                    return response.json().get("response", "No interpretation available")
                    
        except Exception as e:
            logger.error(f"Error getting AI interpretation: {e}")
            
        return "AI interpretation unavailable"
        
    async def start(self):
        """Start the service."""
        # Register with MCP proxy after startup
        asyncio.create_task(self.register_with_proxy())
        
        # Run the web server
        runner = web.AppRunner(self.app)
        await runner.setup()
        site = web.TCPSite(runner, '0.0.0.0', self.port)
        await site.start()
        
        logger.info(f"{self.service_name} started on port {self.port}")
        
        # Keep running
        while True:
            await asyncio.sleep(3600)

if __name__ == "__main__":
    service = SampleAnalyzerService()
    asyncio.run(service.start()) 