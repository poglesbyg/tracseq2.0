"""
Workflow Integration with Existing RAG System

This module demonstrates how to integrate LlamaIndex Workflows 1.0
with the existing TracSeq 2.0 laboratory management system.
"""

import asyncio
import logging
from pathlib import Path
from typing import Any, Dict, List, Optional

from llama_index.workflows import Context, Workflow

from .document_processing import DocumentProcessingWorkflow, process_document_with_workflow
from .quality_control import QualityControlWorkflow, extract_with_quality_control
from .experiment_tracking import ExperimentTrackingWorkflow, run_mlops_experiment
from .multi_agent import MultiAgentLabWorkflow, process_laboratory_submission

from ..rag_orchestrator import LabSubmissionRAG
from ..models.submission import ExtractionResult

logger = logging.getLogger(__name__)


class WorkflowAdapter:
    """
    Adapter class to integrate new workflows with existing RAG system.
    
    Provides compatibility layer between workflow-based and traditional approaches.
    """
    
    def __init__(self, use_workflows: bool = True):
        self.use_workflows = use_workflows
        self.rag_system = LabSubmissionRAG()
        
        # Initialize workflows
        self.doc_workflow = DocumentProcessingWorkflow()
        self.qc_workflow = QualityControlWorkflow()
        self.mlops_workflow = ExperimentTrackingWorkflow()
        self.multi_agent_workflow = MultiAgentLabWorkflow()
    
    async def process_document(
        self, file_path: str | Path, use_quality_control: bool = True
    ) -> ExtractionResult:
        """
        Process document using either workflow or traditional approach.
        
        Args:
            file_path: Path to document
            use_quality_control: Whether to apply quality control workflow
            
        Returns:
            ExtractionResult with extracted data
        """
        if self.use_workflows:
            logger.info("Using workflow-based document processing")
            
            if use_quality_control:
                # First run document processing workflow
                result = await process_document_with_workflow(file_path)
                
                if result.success and result.submission:
                    # Apply quality control workflow
                    qc_result = await extract_with_quality_control(
                        result.submission.raw_text or "",
                        require_human_review=False
                    )
                    
                    # Merge results
                    if qc_result.success:
                        result.submission = qc_result.submission
                        result.confidence_score = qc_result.confidence_score
                        result.warnings.extend(qc_result.warnings)
                
                return result
            else:
                # Direct document processing
                return await process_document_with_workflow(file_path)
        else:
            # Use traditional RAG orchestrator
            logger.info("Using traditional RAG orchestrator")
            await self.rag_system.initialize_database()
            return await self.rag_system.process_document(file_path)
    
    async def run_experiment(
        self,
        experiment_name: str,
        test_documents: List[str],
        model_configs: Optional[List[Dict[str, Any]]] = None
    ) -> Dict[str, Any]:
        """
        Run MLOps experiment comparing different extraction models.
        
        Args:
            experiment_name: Name of the experiment
            test_documents: List of document paths to test
            model_configs: List of model configurations to compare
            
        Returns:
            Experiment results with winning model
        """
        if not model_configs:
            # Default model configurations
            model_configs = [
                {
                    "model_id": "ollama_llama3",
                    "model_type": "local",
                    "provider": "ollama",
                    "model_name": "llama3.2:3b"
                },
                {
                    "model_id": "openai_gpt35",
                    "model_type": "cloud",
                    "provider": "openai",
                    "model_name": "gpt-3.5-turbo"
                }
            ]
        
        # Prepare test data
        test_data = []
        for doc_path in test_documents:
            test_data.append({
                "id": Path(doc_path).stem,
                "path": doc_path,
                "type": Path(doc_path).suffix
            })
        
        # Run experiment
        return await run_mlops_experiment(
            experiment_name=experiment_name,
            model_configs=model_configs,
            test_data=test_data
        )
    
    async def process_with_multi_agent(
        self,
        document_path: str,
        submission_type: str = "standard",
        priority: str = "normal"
    ) -> Dict[str, Any]:
        """
        Process submission using multi-agent workflow.
        
        Args:
            document_path: Path to document
            submission_type: Type of submission
            priority: Processing priority
            
        Returns:
            Processing summary with agent logs
        """
        return await process_laboratory_submission(
            document_path=document_path,
            submission_type=submission_type,
            priority=priority
        )
    
    async def query_with_context(
        self, query: str, session_id: str = "default"
    ) -> str:
        """
        Query the system using existing RAG with workflow context.
        
        Args:
            query: Natural language query
            session_id: Session identifier
            
        Returns:
            Query response
        """
        await self.rag_system.initialize_database()
        return await self.rag_system.query_submissions(
            query=query,
            session_id=session_id
        )


# Example usage functions
async def demo_workflow_integration():
    """Demonstrate workflow integration with existing system"""
    
    adapter = WorkflowAdapter(use_workflows=True)
    
    # Example 1: Process document with quality control
    print("=== Document Processing with Quality Control ===")
    result = await adapter.process_document(
        "/path/to/sample_submission.pdf",
        use_quality_control=True
    )
    print(f"Success: {result.success}")
    print(f"Confidence: {result.confidence_score:.2f}")
    
    # Example 2: Run MLOps experiment
    print("\n=== MLOps Experiment ===")
    experiment_result = await adapter.run_experiment(
        experiment_name="Model Comparison 2024",
        test_documents=[
            "/path/to/test1.pdf",
            "/path/to/test2.docx"
        ]
    )
    print(f"Deployed model: {experiment_result.get('deployed_model', 'None')}")
    
    # Example 3: Multi-agent processing
    print("\n=== Multi-Agent Processing ===")
    agent_result = await adapter.process_with_multi_agent(
        document_path="/path/to/urgent_submission.pdf",
        submission_type="urgent",
        priority="high"
    )
    print(f"Submission ID: {agent_result['submission_id']}")
    print(f"Agents involved: {', '.join(agent_result['agents_involved'])}")
    
    # Example 4: Query with context
    print("\n=== Query Processing ===")
    response = await adapter.query_with_context(
        "How many DNA samples are stored at -80°C?"
    )
    print(f"Response: {response}")


async def migrate_to_workflows():
    """
    Example migration script from traditional to workflow-based processing.
    
    This demonstrates how to gradually migrate existing functionality.
    """
    
    # Phase 1: Test workflows alongside existing system
    print("Phase 1: Testing workflows in parallel")
    
    test_document = "/path/to/test_document.pdf"
    
    # Traditional processing
    traditional_adapter = WorkflowAdapter(use_workflows=False)
    traditional_result = await traditional_adapter.process_document(test_document)
    
    # Workflow processing
    workflow_adapter = WorkflowAdapter(use_workflows=True)
    workflow_result = await workflow_adapter.process_document(test_document)
    
    # Compare results
    print(f"Traditional confidence: {traditional_result.confidence_score:.2f}")
    print(f"Workflow confidence: {workflow_result.confidence_score:.2f}")
    
    # Phase 2: Monitor performance
    print("\nPhase 2: Performance monitoring")
    
    # Run experiment to compare approaches
    experiment_result = await workflow_adapter.run_experiment(
        experiment_name="Workflow vs Traditional",
        test_documents=[test_document],
        model_configs=[
            {"model_id": "traditional", "approach": "rag_orchestrator"},
            {"model_id": "workflow", "approach": "document_processing_workflow"}
        ]
    )
    
    # Phase 3: Gradual rollout
    print("\nPhase 3: Gradual rollout based on document type")
    
    # Use workflows for specific document types
    async def smart_process(doc_path: str) -> ExtractionResult:
        doc_type = Path(doc_path).suffix.lower()
        
        if doc_type in ['.pdf', '.docx']:  # Well-supported formats
            return await workflow_adapter.process_document(doc_path)
        else:  # Fallback for other formats
            return await traditional_adapter.process_document(doc_path)
    
    # Process various documents
    for doc in ["/path/doc.pdf", "/path/doc.txt", "/path/doc.docx"]:
        result = await smart_process(doc)
        print(f"Processed {doc}: Success={result.success}")


# Integration with existing FastAPI endpoints
def create_workflow_routes(app):
    """
    Create FastAPI routes that use workflows.
    
    This can be integrated into the existing API gateway.
    """
    from fastapi import APIRouter, UploadFile, File
    from fastapi.responses import JSONResponse
    
    router = APIRouter(prefix="/api/v2/workflows", tags=["workflows"])
    adapter = WorkflowAdapter(use_workflows=True)
    
    @router.post("/process")
    async def process_document_endpoint(
        file: UploadFile = File(...),
        use_quality_control: bool = True
    ):
        """Process document using workflow"""
        # Save uploaded file
        file_path = f"/tmp/{file.filename}"
        with open(file_path, "wb") as f:
            content = await file.read()
            f.write(content)
        
        # Process with workflow
        result = await adapter.process_document(
            file_path,
            use_quality_control=use_quality_control
        )
        
        return JSONResponse({
            "success": result.success,
            "confidence_score": result.confidence_score,
            "submission_id": result.submission.submission_id if result.submission else None,
            "warnings": result.warnings
        })
    
    @router.post("/multi-agent")
    async def multi_agent_endpoint(
        file: UploadFile = File(...),
        submission_type: str = "standard",
        priority: str = "normal"
    ):
        """Process using multi-agent workflow"""
        file_path = f"/tmp/{file.filename}"
        with open(file_path, "wb") as f:
            content = await file.read()
            f.write(content)
        
        result = await adapter.process_with_multi_agent(
            document_path=file_path,
            submission_type=submission_type,
            priority=priority
        )
        
        return JSONResponse(result)
    
    return router


if __name__ == "__main__":
    # Run demo
    asyncio.run(demo_workflow_integration())