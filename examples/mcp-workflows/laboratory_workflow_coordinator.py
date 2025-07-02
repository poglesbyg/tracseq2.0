#!/usr/bin/env python3
"""
Laboratory Workflow Coordinator
Implements complex workflows that coordinate multiple MCP services.
"""

import asyncio
import json
import logging
from datetime import datetime
from typing import Dict, Any, List, Optional
from enum import Enum
import httpx

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class WorkflowStatus(Enum):
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    ROLLED_BACK = "rolled_back"

class LaboratoryWorkflow:
    """Base class for laboratory workflows."""
    
    def __init__(self, workflow_id: str, mcp_proxy_url: str = "http://localhost:9500"):
        self.workflow_id = workflow_id
        self.mcp_proxy_url = mcp_proxy_url
        self.status = WorkflowStatus.PENDING
        self.steps_completed = []
        self.transaction_log = []
        
    async def invoke_service(self, service: str, tool: str, params: Dict[str, Any]) -> Dict[str, Any]:
        """Invoke a service through the MCP proxy."""
        async with httpx.AsyncClient() as client:
            response = await client.post(
                f"{self.mcp_proxy_url}/mcp/invoke",
                json={
                    "service": service,
                    "tool": tool,
                    "params": params
                },
                timeout=30.0
            )
            return response.json()
            
    async def log_transaction(self, step: str, data: Dict[str, Any]):
        """Log a transaction step for potential rollback."""
        self.transaction_log.append({
            "step": step,
            "timestamp": datetime.now().isoformat(),
            "data": data
        })
        
    async def rollback(self):
        """Rollback the workflow by undoing completed steps."""
        logger.warning(f"Rolling back workflow {self.workflow_id}")
        # Implementation would undo steps in reverse order
        self.status = WorkflowStatus.ROLLED_BACK

class SampleSubmissionWorkflow(LaboratoryWorkflow):
    """
    Complete sample submission workflow:
    1. Extract data from submission document
    2. Validate sample information
    3. Assign storage locations
    4. Generate barcodes
    5. Create tracking records
    6. Send notifications
    """
    
    def __init__(self, document_path: str):
        super().__init__(f"submission_{datetime.now().strftime('%Y%m%d%H%M%S')}")
        self.document_path = document_path
        self.extracted_data = None
        self.validated_samples = []
        self.storage_assignments = []
        
    async def execute(self) -> Dict[str, Any]:
        """Execute the complete submission workflow."""
        try:
            self.status = WorkflowStatus.RUNNING
            logger.info(f"Starting workflow: {self.workflow_id}")
            
            # Step 1: Extract data from document
            await self.extract_submission_data()
            
            # Step 2: Validate extracted data
            await self.validate_submission()
            
            # Step 3: Get AI recommendations
            await self.get_ai_recommendations()
            
            # Step 4: Assign storage locations
            await self.assign_storage()
            
            # Step 5: Generate barcodes
            await self.generate_barcodes()
            
            # Step 6: Create tracking records
            await self.create_tracking_records()
            
            # Step 7: Send notifications
            await self.send_notifications()
            
            self.status = WorkflowStatus.COMPLETED
            logger.info(f"Workflow {self.workflow_id} completed successfully")
            
            return {
                "workflow_id": self.workflow_id,
                "status": self.status.value,
                "samples_processed": len(self.validated_samples),
                "storage_assignments": self.storage_assignments
            }
            
        except Exception as e:
            logger.error(f"Workflow failed: {e}")
            self.status = WorkflowStatus.FAILED
            await self.rollback()
            raise
            
    async def extract_submission_data(self):
        """Extract data from submission document using RAG service."""
        logger.info("Step 1: Extracting submission data...")
        
        result = await self.invoke_service(
            service="rag_service",
            tool="extract_laboratory_data",
            params={
                "document_path": self.document_path,
                "extraction_schema": {
                    "submitter_info": ["name", "institution", "contact"],
                    "sample_info": ["sample_id", "type", "volume", "concentration"],
                    "storage_requirements": ["temperature", "special_conditions"],
                    "analysis_requested": ["sequencing_type", "coverage", "special_requests"]
                }
            }
        )
        
        if result.get("success"):
            self.extracted_data = result.get("data")
            await self.log_transaction("extract_data", self.extracted_data)
            self.steps_completed.append("data_extraction")
            logger.info(f"Extracted {len(self.extracted_data.get('samples', []))} samples")
        else:
            raise Exception(f"Data extraction failed: {result.get('error')}")
            
    async def validate_submission(self):
        """Validate the extracted submission data."""
        logger.info("Step 2: Validating submission...")
        
        # Use cognitive assistant for validation
        result = await self.invoke_service(
            service="cognitive_assistant",
            tool="ask_laboratory_question",
            params={
                "query": f"Validate this laboratory submission data and identify any issues: {json.dumps(self.extracted_data)}",
                "context": "laboratory_submission_validation"
            }
        )
        
        validation_result = result.get("data", {})
        
        # Process validation results
        if validation_result.get("valid", True):
            self.validated_samples = self.extracted_data.get("samples", [])
            await self.log_transaction("validate_data", validation_result)
            self.steps_completed.append("validation")
            logger.info("Submission validated successfully")
        else:
            issues = validation_result.get("issues", [])
            raise Exception(f"Validation failed: {issues}")
            
    async def get_ai_recommendations(self):
        """Get AI recommendations for sample processing."""
        logger.info("Step 3: Getting AI recommendations...")
        
        # Get proactive suggestions
        result = await self.invoke_service(
            service="cognitive_assistant",
            tool="get_proactive_suggestions",
            params={
                "context": {
                    "samples": self.validated_samples,
                    "workflow": "sample_submission"
                }
            }
        )
        
        recommendations = result.get("data", {}).get("suggestions", [])
        logger.info(f"Received {len(recommendations)} AI recommendations")
        
        # Apply high-confidence recommendations
        for rec in recommendations:
            if rec.get("confidence", 0) > 0.8:
                logger.info(f"Applying recommendation: {rec.get('suggestion')}")
                # Implementation would apply the recommendation
                
        self.steps_completed.append("ai_recommendations")
        
    async def assign_storage(self):
        """Assign optimal storage locations for samples."""
        logger.info("Step 4: Assigning storage locations...")
        
        # Parallel storage assignment for efficiency
        tasks = []
        for sample in self.validated_samples:
            task = self.invoke_service(
                service="storage_optimizer",
                tool="assign_locations",
                params={
                    "sample": sample,
                    "requirements": sample.get("storage_requirements", {})
                }
            )
            tasks.append(task)
            
        results = await asyncio.gather(*tasks)
        
        # Process results
        for sample, result in zip(self.validated_samples, results):
            if result.get("success"):
                assignment = result.get("data", {})
                self.storage_assignments.append({
                    "sample_id": sample.get("sample_id"),
                    "location": assignment.get("location"),
                    "temperature_zone": assignment.get("temperature_zone")
                })
                
        await self.log_transaction("storage_assignment", self.storage_assignments)
        self.steps_completed.append("storage_assignment")
        logger.info(f"Assigned storage for {len(self.storage_assignments)} samples")
        
    async def generate_barcodes(self):
        """Generate barcodes for samples."""
        logger.info("Step 5: Generating barcodes...")
        
        # This would call a barcode service
        # For now, we'll simulate it
        barcodes = []
        for sample in self.validated_samples:
            barcode = f"BAR-{sample.get('sample_id', 'UNKNOWN')}-{datetime.now().strftime('%Y%m%d')}"
            barcodes.append({
                "sample_id": sample.get("sample_id"),
                "barcode": barcode
            })
            
        await self.log_transaction("barcode_generation", barcodes)
        self.steps_completed.append("barcode_generation")
        logger.info(f"Generated {len(barcodes)} barcodes")
        
    async def create_tracking_records(self):
        """Create tracking records in the database."""
        logger.info("Step 6: Creating tracking records...")
        
        # This would create records in the sample tracking system
        tracking_records = []
        for idx, sample in enumerate(self.validated_samples):
            record = {
                "sample_id": sample.get("sample_id"),
                "barcode": f"BAR-{sample.get('sample_id')}-{datetime.now().strftime('%Y%m%d')}",
                "location": self.storage_assignments[idx].get("location") if idx < len(self.storage_assignments) else "TBD",
                "status": "received",
                "workflow_id": self.workflow_id,
                "created_at": datetime.now().isoformat()
            }
            tracking_records.append(record)
            
        await self.log_transaction("tracking_records", tracking_records)
        self.steps_completed.append("tracking_records")
        logger.info(f"Created {len(tracking_records)} tracking records")
        
    async def send_notifications(self):
        """Send notifications about the submission."""
        logger.info("Step 7: Sending notifications...")
        
        # This would use a notification service
        notifications = {
            "submission_received": {
                "recipient": self.extracted_data.get("submitter_info", {}).get("contact"),
                "samples_count": len(self.validated_samples),
                "workflow_id": self.workflow_id
            }
        }
        
        await self.log_transaction("notifications", notifications)
        self.steps_completed.append("notifications")
        logger.info("Notifications sent")

class ParallelAnalysisWorkflow(LaboratoryWorkflow):
    """
    Workflow that performs multiple analyses in parallel.
    Demonstrates efficient parallel processing with MCP.
    """
    
    def __init__(self, sample_ids: List[str]):
        super().__init__(f"parallel_analysis_{datetime.now().strftime('%Y%m%d%H%M%S')}")
        self.sample_ids = sample_ids
        self.analysis_results = {}
        
    async def execute(self) -> Dict[str, Any]:
        """Execute parallel analysis workflow."""
        try:
            self.status = WorkflowStatus.RUNNING
            
            # Create analysis tasks for all samples
            tasks = []
            for sample_id in self.sample_ids:
                task = self.analyze_sample_with_ai(sample_id)
                tasks.append(task)
                
            # Execute all analyses in parallel
            results = await asyncio.gather(*tasks, return_exceptions=True)
            
            # Process results
            successful = 0
            failed = 0
            for sample_id, result in zip(self.sample_ids, results):
                if isinstance(result, Exception):
                    logger.error(f"Analysis failed for {sample_id}: {result}")
                    failed += 1
                else:
                    self.analysis_results[sample_id] = result
                    successful += 1
                    
            self.status = WorkflowStatus.COMPLETED
            
            return {
                "workflow_id": self.workflow_id,
                "status": self.status.value,
                "total_samples": len(self.sample_ids),
                "successful": successful,
                "failed": failed,
                "results": self.analysis_results
            }
            
        except Exception as e:
            logger.error(f"Parallel workflow failed: {e}")
            self.status = WorkflowStatus.FAILED
            raise
            
    async def analyze_sample_with_ai(self, sample_id: str) -> Dict[str, Any]:
        """Analyze a single sample with AI enhancement."""
        # Step 1: Get sample data
        sample_data = await self.invoke_service(
            service="sample_analyzer",
            tool="analyze_sample",
            params={
                "sample_id": sample_id,
                "tests_requested": ["CBC", "glucose", "cholesterol"],
                "ai_interpretation": True
            }
        )
        
        # Step 2: Get storage optimization suggestions
        storage_suggestion = await self.invoke_service(
            service="storage_optimizer",
            tool="optimize_storage",
            params={
                "sample_id": sample_id,
                "current_data": sample_data.get("data", {})
            }
        )
        
        # Step 3: Generate AI insights using Ollama
        ai_insights = await self.get_ollama_insights(sample_data.get("data", {}))
        
        return {
            "sample_id": sample_id,
            "analysis": sample_data.get("data"),
            "storage_optimization": storage_suggestion.get("data"),
            "ai_insights": ai_insights
        }
        
    async def get_ollama_insights(self, analysis_data: Dict[str, Any]) -> str:
        """Get insights from Ollama AI model."""
        try:
            async with httpx.AsyncClient() as client:
                prompt = f"""
                Analyze these laboratory results and provide clinical insights:
                {json.dumps(analysis_data, indent=2)}
                
                Provide:
                1. Key findings
                2. Any concerning values
                3. Recommended follow-up tests
                4. Storage recommendations based on sample stability
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
                    return response.json().get("response", "No insights available")
                    
        except Exception as e:
            logger.error(f"Error getting Ollama insights: {e}")
            
        return "AI insights unavailable"

# Example usage
async def run_sample_submission_workflow():
    """Example: Run a sample submission workflow."""
    workflow = SampleSubmissionWorkflow(
        document_path="/path/to/submission.pdf"
    )
    
    try:
        result = await workflow.execute()
        logger.info(f"Workflow completed: {result}")
    except Exception as e:
        logger.error(f"Workflow failed: {e}")

async def run_parallel_analysis():
    """Example: Run parallel analysis on multiple samples."""
    sample_ids = [
        "SMPL-001", "SMPL-002", "SMPL-003", 
        "SMPL-004", "SMPL-005", "SMPL-006"
    ]
    
    workflow = ParallelAnalysisWorkflow(sample_ids)
    result = await workflow.execute()
    
    logger.info(f"Parallel analysis completed: {result['successful']}/{result['total_samples']} successful")

if __name__ == "__main__":
    # Run example workflows
    asyncio.run(run_sample_submission_workflow()) 