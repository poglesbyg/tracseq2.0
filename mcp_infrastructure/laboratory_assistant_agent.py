#!/usr/bin/env python3
"""
Laboratory Assistant Agent for TracSeq 2.0

This agent uses the Model Context Protocol (MCP) to coordinate
laboratory operations across multiple microservices.
"""

import asyncio
import logging
from typing import Dict, List, Any, Optional, Union
from dataclasses import dataclass
from datetime import datetime, timedelta
import json
import uuid

from mcp_client import McpClient, McpError
from anthropic import AsyncAnthropic
import httpx

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@dataclass
class ProcessingResult:
    """Result of a laboratory processing operation"""
    success: bool
    operation: str
    data: Dict[str, Any]
    errors: List[str] = None
    warnings: List[str] = None
    processing_time: float = 0.0
    
    def __post_init__(self):
        if self.errors is None:
            self.errors = []
        if self.warnings is None:
            self.warnings = []

@dataclass
class AgentConfig:
    """Configuration for the Laboratory Assistant Agent"""
    anthropic_api_key: str
    mcp_endpoints: Dict[str, str]
    confidence_threshold: float = 0.7
    max_retry_attempts: int = 3
    operation_timeout: int = 300  # seconds
    enable_notifications: bool = True

class LaboratoryAssistantAgent:
    """
    Intelligent laboratory assistant agent using MCP to coordinate
    operations across TracSeq 2.0 microservices.
    """
    
    def __init__(self, config: AgentConfig):
        self.config = config
        self.anthropic = AsyncAnthropic(api_key=config.anthropic_api_key)
        self.mcp_clients = {}
        self.session_id = str(uuid.uuid4())
        self.operation_history = []
        
        # Initialize MCP clients for each service
        self._initialize_mcp_clients()
    
    def _initialize_mcp_clients(self):
        """Initialize MCP clients for all configured services"""
        for service_name, endpoint in self.config.mcp_endpoints.items():
            try:
                self.mcp_clients[service_name] = McpClient(
                    endpoint=endpoint,
                    timeout=self.config.operation_timeout
                )
                logger.info(f"Initialized MCP client for {service_name}")
            except Exception as e:
                logger.error(f"Failed to initialize MCP client for {service_name}: {e}")
    
    async def process_laboratory_submission(
        self, 
        document_path: str,
        additional_context: Optional[Dict[str, Any]] = None
    ) -> ProcessingResult:
        """
        Complete laboratory submission processing workflow.
        
        This is the primary workflow that coordinates:
        1. Document processing via RAG service
        2. Sample creation via Sample service
        3. Storage assignment via Storage service
        4. Transaction coordination via Transaction service
        """
        start_time = datetime.now()
        operation_id = str(uuid.uuid4())
        
        logger.info(f"Starting laboratory submission processing: {operation_id}")
        logger.info(f"Document: {document_path}")
        
        try:
            # Step 1: Process document using RAG service
            logger.info("Step 1: Processing document with RAG service")
            rag_result = await self._process_document_with_rag(
                document_path, additional_context
            )
            
            if not rag_result.success:
                return ProcessingResult(
                    success=False,
                    operation="laboratory_submission",
                    data={},
                    errors=[f"Document processing failed: {'; '.join(rag_result.errors)}"],
                    processing_time=(datetime.now() - start_time).total_seconds()
                )
            
            # Step 2: Create samples based on extracted data
            logger.info("Step 2: Creating samples from extracted data")
            samples_result = await self._create_samples_from_extraction(
                rag_result.data
            )
            
            if not samples_result.success:
                return ProcessingResult(
                    success=False,
                    operation="laboratory_submission",
                    data={"rag_result": rag_result.data},
                    errors=[f"Sample creation failed: {'; '.join(samples_result.errors)}"],
                    processing_time=(datetime.now() - start_time).total_seconds()
                )
            
            # Step 3: Assign optimal storage locations
            logger.info("Step 3: Assigning storage locations")
            storage_result = await self._assign_optimal_storage(
                samples_result.data["samples"],
                rag_result.data.get("storage_requirements", {})
            )
            
            # Step 4: Create distributed transaction for consistency
            logger.info("Step 4: Creating distributed transaction")
            transaction_result = await self._create_laboratory_workflow(
                {
                    "rag_result": rag_result.data,
                    "samples": samples_result.data["samples"],
                    "storage_assignments": storage_result.data if storage_result.success else None
                }
            )
            
            # Step 5: Generate summary and recommendations
            summary = await self._generate_submission_summary(
                rag_result, samples_result, storage_result, transaction_result
            )
            
            processing_time = (datetime.now() - start_time).total_seconds()
            
            # Record operation in history
            self.operation_history.append({
                "operation_id": operation_id,
                "operation": "laboratory_submission",
                "timestamp": start_time.isoformat(),
                "processing_time": processing_time,
                "success": True,
                "document_path": document_path
            })
            
            return ProcessingResult(
                success=True,
                operation="laboratory_submission",
                data={
                    "operation_id": operation_id,
                    "document_processed": rag_result.data,
                    "samples_created": samples_result.data,
                    "storage_assigned": storage_result.data if storage_result.success else None,
                    "workflow_initiated": transaction_result.data if transaction_result.success else None,
                    "summary": summary,
                    "recommendations": self._generate_recommendations(
                        rag_result, samples_result, storage_result
                    )
                },
                warnings=self._collect_warnings(rag_result, samples_result, storage_result),
                processing_time=processing_time
            )
            
        except Exception as e:
            logger.error(f"Unexpected error in laboratory submission processing: {e}")
            return ProcessingResult(
                success=False,
                operation="laboratory_submission",
                data={},
                errors=[f"Unexpected error: {str(e)}"],
                processing_time=(datetime.now() - start_time).total_seconds()
            )
    
    async def _process_document_with_rag(
        self, 
        document_path: str, 
        additional_context: Optional[Dict[str, Any]] = None
    ) -> ProcessingResult:
        """Process document using RAG service MCP server"""
        try:
            # Call RAG service to process document
            rag_response = await self.mcp_clients['rag_service'].call_tool(
                'process_document',
                {
                    'file_path': document_path,
                    'confidence_threshold': self.config.confidence_threshold,
                    'additional_context': additional_context or {}
                }
            )
            
            if rag_response.get('success', False):
                return ProcessingResult(
                    success=True,
                    operation="document_processing",
                    data=rag_response
                )
            else:
                return ProcessingResult(
                    success=False,
                    operation="document_processing",
                    data={},
                    errors=[rag_response.get('error', 'Unknown RAG processing error')]
                )
                
        except McpError as e:
            logger.error(f"MCP error during document processing: {e}")
            return ProcessingResult(
                success=False,
                operation="document_processing",
                data={},
                errors=[f"MCP error: {str(e)}"]
            )
    
    async def _create_samples_from_extraction(
        self, 
        rag_data: Dict[str, Any]
    ) -> ProcessingResult:
        """Create samples from RAG extraction data"""
        try:
            # Extract sample information from RAG results
            extracted_samples = rag_data.get('extracted_samples', [])
            
            if not extracted_samples:
                return ProcessingResult(
                    success=False,
                    operation="sample_creation",
                    data={},
                    errors=["No samples found in extracted data"]
                )
            
            # Use Sample service MCP to create samples
            samples_response = await self.mcp_clients['sample_service'].call_tool(
                'batch_create_samples',
                {
                    'samples': extracted_samples,
                    'auto_validate': True,
                    'notify_submitter': self.config.enable_notifications
                }
            )
            
            if samples_response.get('success', False):
                return ProcessingResult(
                    success=True,
                    operation="sample_creation",
                    data=samples_response
                )
            else:
                return ProcessingResult(
                    success=False,
                    operation="sample_creation",
                    data={},
                    errors=samples_response.get('errors', ['Unknown sample creation error'])
                )
                
        except McpError as e:
            logger.error(f"MCP error during sample creation: {e}")
            return ProcessingResult(
                success=False,
                operation="sample_creation",
                data={},
                errors=[f"MCP error: {str(e)}"]
            )
    
    async def _assign_optimal_storage(
        self, 
        samples: List[Dict[str, Any]],
        storage_requirements: Dict[str, Any]
    ) -> ProcessingResult:
        """Assign optimal storage locations for samples"""
        try:
            sample_ids = [sample['id'] for sample in samples]
            
            # Call Storage service MCP to assign storage
            storage_response = await self.mcp_clients['storage_service'].call_tool(
                'optimize_storage_assignment',
                {
                    'sample_ids': sample_ids,
                    'requirements': storage_requirements,
                    'priority': 'efficiency',
                    'consider_capacity': True,
                    'consider_temperature': True
                }
            )
            
            if storage_response.get('success', False):
                return ProcessingResult(
                    success=True,
                    operation="storage_assignment",
                    data=storage_response
                )
            else:
                # Storage assignment failure is not critical - samples can be stored manually
                logger.warning("Storage assignment failed, samples will need manual storage")
                return ProcessingResult(
                    success=False,
                    operation="storage_assignment",
                    data={},
                    warnings=["Automatic storage assignment failed - manual assignment required"],
                    errors=storage_response.get('errors', ['Unknown storage assignment error'])
                )
                
        except McpError as e:
            logger.warning(f"MCP error during storage assignment: {e}")
            return ProcessingResult(
                success=False,
                operation="storage_assignment",
                data={},
                warnings=[f"Storage assignment MCP error: {str(e)}"]
            )
    
    async def _create_laboratory_workflow(
        self, 
        workflow_data: Dict[str, Any]
    ) -> ProcessingResult:
        """Create distributed transaction workflow"""
        try:
            # Call Transaction service MCP to create workflow
            transaction_response = await self.mcp_clients['transaction_service'].call_tool(
                'create_laboratory_workflow',
                {
                    'workflow_type': 'sample_submission',
                    'samples': workflow_data.get('samples', []),
                    'storage_assignments': workflow_data.get('storage_assignments'),
                    'notifications': self.config.enable_notifications,
                    'context': {
                        'agent_session': self.session_id,
                        'processing_timestamp': datetime.now().isoformat()
                    }
                }
            )
            
            if transaction_response.get('success', False):
                return ProcessingResult(
                    success=True,
                    operation="workflow_creation",
                    data=transaction_response
                )
            else:
                return ProcessingResult(
                    success=False,
                    operation="workflow_creation",
                    data={},
                    errors=transaction_response.get('errors', ['Unknown workflow creation error'])
                )
                
        except McpError as e:
            logger.error(f"MCP error during workflow creation: {e}")
            return ProcessingResult(
                success=False,
                operation="workflow_creation",
                data={},
                errors=[f"MCP error: {str(e)}"]
            )
    
    async def automated_quality_control(
        self, 
        sample_ids: List[str],
        assessment_type: str = "comprehensive"
    ) -> ProcessingResult:
        """Run comprehensive quality control checks using AI analysis"""
        start_time = datetime.now()
        logger.info(f"Starting automated QC for {len(sample_ids)} samples")
        
        try:
            # Step 1: Get sample details
            sample_details = await self.mcp_clients['sample_service'].call_tool(
                'get_samples_batch',
                {'sample_ids': sample_ids}
            )
            
            if not sample_details.get('success', False):
                return ProcessingResult(
                    success=False,
                    operation="automated_qc",
                    data={},
                    errors=["Failed to retrieve sample details for QC"]
                )
            
            # Step 2: Run QC checks for each sample
            qc_results = []
            overall_scores = []
            
            for sample in sample_details['samples']:
                # Run QC via QA/QC service MCP
                qc_result = await self.mcp_clients['qaqc_service'].call_tool(
                    'run_quality_assessment',
                    {
                        'sample_id': sample['id'],
                        'assessment_type': assessment_type,
                        'automated': True,
                        'include_ai_analysis': True
                    }
                )
                
                if qc_result.get('success', False):
                    qc_results.append(qc_result)
                    overall_scores.append(qc_result.get('quality_score', 0.0))
                else:
                    logger.warning(f"QC failed for sample {sample['id']}")
                    qc_results.append({
                        'sample_id': sample['id'],
                        'success': False,
                        'error': qc_result.get('error', 'Unknown QC error')
                    })
            
            # Step 3: AI-powered analysis of QC results
            ai_analysis = await self._analyze_qc_results_with_ai(qc_results, sample_details['samples'])
            
            # Step 4: Generate recommendations
            recommendations = await self._generate_qc_recommendations(qc_results, ai_analysis)
            
            processing_time = (datetime.now() - start_time).total_seconds()
            
            return ProcessingResult(
                success=True,
                operation="automated_qc",
                data={
                    'samples_assessed': len(sample_ids),
                    'qc_results': qc_results,
                    'overall_quality_score': sum(overall_scores) / len(overall_scores) if overall_scores else 0.0,
                    'ai_analysis': ai_analysis,
                    'recommendations': recommendations,
                    'assessment_type': assessment_type
                },
                processing_time=processing_time
            )
            
        except Exception as e:
            logger.error(f"Error in automated QC: {e}")
            return ProcessingResult(
                success=False,
                operation="automated_qc",
                data={},
                errors=[f"Automated QC error: {str(e)}"],
                processing_time=(datetime.now() - start_time).total_seconds()
            )
    
    async def _analyze_qc_results_with_ai(
        self, 
        qc_results: List[Dict[str, Any]], 
        sample_details: List[Dict[str, Any]]
    ) -> Dict[str, Any]:
        """Use AI to analyze QC results and identify patterns"""
        try:
            # Prepare context for AI analysis
            context = {
                "qc_results": qc_results,
                "sample_details": sample_details,
                "analysis_request": "comprehensive_quality_analysis"
            }
            
            # Create AI prompt for QC analysis
            prompt = f"""
            You are an expert laboratory quality control analyst. Analyze the following QC results and provide insights:

            QC Results Summary:
            - Total samples assessed: {len(qc_results)}
            - Sample types: {list(set(s.get('sample_type', 'Unknown') for s in sample_details))}
            
            Detailed QC Data:
            {json.dumps(context, indent=2)}
            
            Please provide:
            1. Overall quality assessment
            2. Key issues or concerns identified
            3. Patterns or trends in the data
            4. Risk assessment for downstream processing
            5. Specific recommendations for each failed/borderline sample
            
            Format your response as a structured analysis.
            """
            
            # Get AI analysis
            ai_response = await self.anthropic.messages.create(
                model="claude-3-sonnet-20240229",
                max_tokens=2000,
                messages=[
                    {"role": "user", "content": prompt}
                ]
            )
            
            return {
                "ai_analysis": ai_response.content[0].text,
                "analysis_timestamp": datetime.now().isoformat(),
                "model_used": "claude-3-sonnet-20240229"
            }
            
        except Exception as e:
            logger.error(f"AI analysis error: {e}")
            return {
                "ai_analysis": "AI analysis failed",
                "error": str(e)
            }
    
    async def _generate_qc_recommendations(
        self, 
        qc_results: List[Dict[str, Any]], 
        ai_analysis: Dict[str, Any]
    ) -> List[Dict[str, Any]]:
        """Generate specific recommendations based on QC results"""
        recommendations = []
        
        for qc_result in qc_results:
            if not qc_result.get('success', False):
                recommendations.append({
                    "sample_id": qc_result.get('sample_id'),
                    "type": "critical",
                    "issue": "QC assessment failed",
                    "action": "Manual review required",
                    "priority": "high"
                })
                continue
            
            quality_score = qc_result.get('quality_score', 0.0)
            
            if quality_score < 0.6:
                recommendations.append({
                    "sample_id": qc_result.get('sample_id'),
                    "type": "quality_concern",
                    "issue": f"Low quality score: {quality_score}",
                    "action": "Consider re-processing or additional QC",
                    "priority": "high"
                })
            elif quality_score < 0.8:
                recommendations.append({
                    "sample_id": qc_result.get('sample_id'),
                    "type": "monitoring",
                    "issue": f"Borderline quality score: {quality_score}",
                    "action": "Monitor closely during processing",
                    "priority": "medium"
                })
        
        return recommendations
    
    async def intelligent_sample_search(
        self, 
        query: str, 
        use_ai_interpretation: bool = True
    ) -> ProcessingResult:
        """Intelligent sample search using natural language queries"""
        start_time = datetime.now()
        
        try:
            if use_ai_interpretation:
                # Use AI to interpret the natural language query
                search_params = await self._interpret_search_query_with_ai(query)
            else:
                # Simple keyword-based search
                search_params = {"query": query}
            
            # Search samples using MCP
            search_result = await self.mcp_clients['sample_service'].call_tool(
                'search_samples',
                search_params
            )
            
            if search_result.get('success', False):
                # Enhance results with AI insights if requested
                if use_ai_interpretation and search_result.get('samples'):
                    enhanced_results = await self._enhance_search_results_with_ai(
                        query, search_result['samples']
                    )
                    search_result['ai_insights'] = enhanced_results
                
                return ProcessingResult(
                    success=True,
                    operation="intelligent_search",
                    data=search_result,
                    processing_time=(datetime.now() - start_time).total_seconds()
                )
            else:
                return ProcessingResult(
                    success=False,
                    operation="intelligent_search",
                    data={},
                    errors=[search_result.get('error', 'Search failed')],
                    processing_time=(datetime.now() - start_time).total_seconds()
                )
                
        except Exception as e:
            logger.error(f"Error in intelligent search: {e}")
            return ProcessingResult(
                success=False,
                operation="intelligent_search",
                data={},
                errors=[f"Search error: {str(e)}"],
                processing_time=(datetime.now() - start_time).total_seconds()
            )
    
    async def _interpret_search_query_with_ai(self, query: str) -> Dict[str, Any]:
        """Use AI to interpret natural language search queries"""
        prompt = f"""
        Convert this natural language query into structured search parameters for a laboratory sample database:
        
        Query: "{query}"
        
        Available search parameters:
        - status: Pending, Validated, InStorage, InSequencing, Completed, Failed, Discarded
        - sample_type: DNA, RNA, Protein, etc.
        - created_after, created_before: date filters
        - barcode_prefix: partial barcode matching
        - limit, offset: pagination
        
        Return a JSON object with appropriate search parameters.
        If the query is ambiguous, include the most likely interpretation.
        """
        
        try:
            ai_response = await self.anthropic.messages.create(
                model="claude-3-sonnet-20240229",
                max_tokens=500,
                messages=[
                    {"role": "user", "content": prompt}
                ]
            )
            
            # Extract JSON from AI response
            response_text = ai_response.content[0].text
            # Simple JSON extraction (in production, use more robust parsing)
            import re
            json_match = re.search(r'\{.*\}', response_text, re.DOTALL)
            if json_match:
                return json.loads(json_match.group())
            else:
                return {"query": query}  # Fallback
                
        except Exception as e:
            logger.warning(f"AI query interpretation failed: {e}")
            return {"query": query}  # Fallback to simple search
    
    async def _enhance_search_results_with_ai(
        self, 
        original_query: str, 
        samples: List[Dict[str, Any]]
    ) -> Dict[str, Any]:
        """Enhance search results with AI-generated insights"""
        try:
            prompt = f"""
            Analyze these laboratory sample search results for the query: "{original_query}"
            
            Found {len(samples)} samples. Provide insights about:
            1. Relevance ranking of results
            2. Common patterns or characteristics
            3. Potential quality concerns
            4. Recommended next actions
            
            Sample data: {json.dumps(samples[:5], indent=2)}  # Limit for token efficiency
            """
            
            ai_response = await self.anthropic.messages.create(
                model="claude-3-sonnet-20240229",
                max_tokens=1000,
                messages=[
                    {"role": "user", "content": prompt}
                ]
            )
            
            return {
                "insights": ai_response.content[0].text,
                "query": original_query,
                "sample_count": len(samples)
            }
            
        except Exception as e:
            logger.warning(f"AI result enhancement failed: {e}")
            return {"error": str(e)}
    
    # Helper methods
    
    async def _generate_submission_summary(
        self, 
        rag_result: ProcessingResult,
        samples_result: ProcessingResult, 
        storage_result: ProcessingResult,
        transaction_result: ProcessingResult
    ) -> str:
        """Generate a human-readable summary of the submission processing"""
        prompt = f"""
        Generate a concise summary of this laboratory submission processing:
        
        Document Processing: {'✓ Success' if rag_result.success else '✗ Failed'}
        Samples Created: {samples_result.data.get('total_created', 0) if samples_result.success else 0}
        Storage Assigned: {'✓ Success' if storage_result.success else '✗ Failed/Manual Required'}
        Workflow Created: {'✓ Success' if transaction_result.success else '✗ Failed'}
        
        Create a brief, professional summary suitable for laboratory staff.
        """
        
        try:
            ai_response = await self.anthropic.messages.create(
                model="claude-3-sonnet-20240229",
                max_tokens=300,
                messages=[
                    {"role": "user", "content": prompt}
                ]
            )
            return ai_response.content[0].text
        except Exception:
            return "Laboratory submission processing completed. See detailed results for more information."
    
    def _generate_recommendations(
        self, 
        rag_result: ProcessingResult,
        samples_result: ProcessingResult,
        storage_result: ProcessingResult
    ) -> List[str]:
        """Generate recommendations based on processing results"""
        recommendations = []
        
        if not rag_result.success:
            recommendations.append("Review document format and ensure all required information is present")
        
        if not samples_result.success:
            recommendations.append("Manual sample creation may be required")
        
        if not storage_result.success:
            recommendations.append("Assign storage locations manually for created samples")
        
        if rag_result.success and rag_result.data.get('confidence_score', 0) < self.config.confidence_threshold:
            recommendations.append("Consider manual review of extracted data due to low confidence")
        
        return recommendations
    
    def _collect_warnings(
        self, 
        *results: ProcessingResult
    ) -> List[str]:
        """Collect warnings from multiple processing results"""
        warnings = []
        for result in results:
            if result.warnings:
                warnings.extend(result.warnings)
        return warnings
    
    async def get_agent_status(self) -> Dict[str, Any]:
        """Get current agent status and health"""
        mcp_status = {}
        for service, client in self.mcp_clients.items():
            try:
                # Try to ping the MCP service
                response = await client.call_tool('health_check', {})
                mcp_status[service] = "healthy"
            except Exception:
                mcp_status[service] = "unavailable"
        
        return {
            "agent_id": self.session_id,
            "status": "operational" if all(status == "healthy" for status in mcp_status.values()) else "degraded",
            "mcp_services": mcp_status,
            "operations_completed": len(self.operation_history),
            "uptime": datetime.now().isoformat(),
            "config": {
                "confidence_threshold": self.config.confidence_threshold,
                "max_retry_attempts": self.config.max_retry_attempts,
                "operation_timeout": self.config.operation_timeout
            }
        }

# Example usage and testing
async def main():
    """Example usage of the Laboratory Assistant Agent"""
    config = AgentConfig(
        anthropic_api_key="your-anthropic-api-key",
        mcp_endpoints={
            'sample_service': 'http://localhost:8081/mcp',
            'rag_service': 'http://localhost:8000/mcp',
            'storage_service': 'http://localhost:8082/mcp',
            'transaction_service': 'http://localhost:8088/mcp',
            'qaqc_service': 'http://localhost:8085/mcp'
        },
        confidence_threshold=0.7,
        enable_notifications=True
    )
    
    agent = LaboratoryAssistantAgent(config)
    
    # Example 1: Process a laboratory submission
    logger.info("Testing laboratory submission processing...")
    result = await agent.process_laboratory_submission(
        document_path="/path/to/lab_submission.pdf",
        additional_context={"urgency": "high", "submitter": "Dr. Smith"}
    )
    
    print(f"Submission processing result: {result.success}")
    if result.success:
        print(f"Created {result.data.get('samples_created', {}).get('total_created', 0)} samples")
    
    # Example 2: Run automated QC
    if result.success and result.data.get('samples_created', {}).get('samples'):
        sample_ids = [s['id'] for s in result.data['samples_created']['samples'][:3]]
        qc_result = await agent.automated_quality_control(sample_ids)
        print(f"QC assessment result: {qc_result.success}")
    
    # Example 3: Intelligent search
    search_result = await agent.intelligent_sample_search(
        "Find all DNA samples from last week with high quality scores"
    )
    print(f"Search result: {search_result.success}")
    
    # Get agent status
    status = await agent.get_agent_status()
    print(f"Agent status: {status}")

if __name__ == "__main__":
    asyncio.run(main())