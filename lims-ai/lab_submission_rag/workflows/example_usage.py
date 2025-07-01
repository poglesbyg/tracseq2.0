#!/usr/bin/env python3
"""
Example Usage of LlamaIndex Workflows in TracSeq 2.0

This script demonstrates various workflow implementations and usage patterns.
Run this file to see workflows in action.
"""

import asyncio
import logging
from pathlib import Path
from typing import Dict, Any

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


async def example_document_processing():
    """Example: Process a laboratory document using the document processing workflow"""
    print("\n=== Document Processing Workflow Example ===")
    
    try:
        from lab_submission_rag.workflows.document_processing import process_document_with_workflow
        
        # Process a sample document
        test_doc = Path("./test_data/sample_submission.pdf")
        
        if test_doc.exists():
            result = await process_document_with_workflow(test_doc)
            
            print(f"Processing success: {result.success}")
            print(f"Confidence score: {result.confidence_score:.2f}")
            
            if result.submission:
                print(f"Submission ID: {result.submission.submission_id}")
                print(f"Sample Type: {result.submission.sample.sample_type}")
                print(f"Submitter: {result.submission.administrative.submitter_name}")
            
            if result.warnings:
                print(f"Warnings: {', '.join(result.warnings)}")
        else:
            print(f"Test document not found: {test_doc}")
            print("Creating mock result for demonstration...")
            
            # Mock result for demonstration
            from lab_submission_rag.models.submission import ExtractionResult, LabSubmission
            
            mock_submission = LabSubmission(
                administrative={"submitter_name": "Dr. Jane Smith"},
                sample={"sample_type": "DNA", "sample_id": "TEST-001"},
                sequencing={"platform": "Illumina"}
            )
            
            result = ExtractionResult(
                success=True,
                submission=mock_submission,
                confidence_score=0.92,
                warnings=[]
            )
            
            print(f"Mock result created - Success: {result.success}")
            
    except ImportError as e:
        print(f"Import error: {e}")
        print("Make sure to install dependencies: pip install llama-index-workflows")


async def example_quality_control():
    """Example: Extract data with quality control and retry logic"""
    print("\n=== Quality Control Workflow Example ===")
    
    try:
        from lab_submission_rag.workflows.quality_control import extract_with_quality_control
        
        # Sample text with some issues
        sample_text = """
        Laboratory Submission Form
        
        Submitter: Dr. John Doe
        Email: johndoe.lab.com (missing @)
        
        Sample Information:
        Type: RNA
        Volume: 100 µL
        
        Sequencing Platform: Unknown Platform XYZ
        """
        
        # Run extraction with quality control
        result = await extract_with_quality_control(
            text=sample_text,
            require_human_review=False
        )
        
        print(f"Extraction success: {result.success}")
        print(f"Final confidence: {result.confidence_score:.2f}")
        print(f"Warnings: {result.warnings}")
        
        # The workflow should detect issues and retry with corrections
        
    except ImportError as e:
        print(f"Import error: {e}")
        print("Quality control workflow demonstration skipped")


async def example_mlops_experiment():
    """Example: Run an A/B test comparing extraction models"""
    print("\n=== MLOps Experiment Tracking Workflow Example ===")
    
    try:
        from lab_submission_rag.workflows.experiment_tracking import run_mlops_experiment
        
        # Define model configurations to compare
        model_configs = [
            {
                "model_id": "fast_model",
                "model_type": "lightweight",
                "extraction_speed": "fast",
                "accuracy": 0.85
            },
            {
                "model_id": "accurate_model",
                "model_type": "heavyweight",
                "extraction_speed": "slow",
                "accuracy": 0.95
            }
        ]
        
        # Create test data
        test_data = [
            {"id": "test1", "path": "/path/to/test1.pdf", "type": ".pdf"},
            {"id": "test2", "path": "/path/to/test2.docx", "type": ".docx"},
            {"id": "test3", "path": "/path/to/test3.txt", "type": ".txt"}
        ]
        
        # Run experiment
        result = await run_mlops_experiment(
            experiment_name="Extraction Model Comparison",
            model_configs=model_configs,
            test_data=test_data
        )
        
        print(f"Experiment completed: {result.get('status', 'unknown')}")
        print(f"Winning model: {result.get('deployed_model', 'none')}")
        print(f"Deployment decision: {result.get('deployed', False)}")
        
    except ImportError as e:
        print(f"Import error: {e}")
        print("MLOps workflow demonstration skipped")


async def example_multi_agent():
    """Example: Process submission using multiple coordinated agents"""
    print("\n=== Multi-Agent Workflow Example ===")
    
    try:
        from lab_submission_rag.workflows.multi_agent import (
            MultiAgentLabWorkflow,
            coordinate_agents_example
        )
        
        # Create workflow
        workflow = MultiAgentLabWorkflow(timeout=300, verbose=True)
        
        # Process with event streaming
        print("Starting multi-agent processing with event streaming...")
        
        # Mock document path for demonstration
        mock_doc = "/path/to/urgent_submission.pdf"
        
        # Run workflow
        result = await workflow.run(
            document_path=mock_doc,
            submission_type="urgent",
            priority="high"
        )
        
        print(f"\nWorkflow completed!")
        print(f"Submission ID: {result['submission_id']}")
        print(f"Status: {result['status']}")
        print(f"Processing time: {result['total_processing_time']:.2f}s")
        print(f"Quality score: {result['quality_score']:.2f}")
        
        # Show agent activity log
        print("\nAgent Activity Log:")
        for log_entry in result['workflow_log']:
            print(f"  - {log_entry['agent']}: {log_entry['action']} at {log_entry['timestamp']}")
        
    except ImportError as e:
        print(f"Import error: {e}")
        print("Multi-agent workflow demonstration skipped")


async def example_workflow_integration():
    """Example: Using the workflow adapter for gradual migration"""
    print("\n=== Workflow Integration Example ===")
    
    try:
        from lab_submission_rag.workflows.workflow_integration import WorkflowAdapter
        
        # Create adapters for both approaches
        workflow_adapter = WorkflowAdapter(use_workflows=True)
        traditional_adapter = WorkflowAdapter(use_workflows=False)
        
        print("Workflow adapter created successfully")
        
        # Example: Smart routing based on document type
        async def smart_process(doc_path: str) -> Dict[str, Any]:
            """Route to appropriate processor based on document type"""
            doc_type = Path(doc_path).suffix.lower()
            
            if doc_type in ['.pdf', '.docx']:
                print(f"Using workflow for {doc_type} file")
                # Use workflow approach for well-supported formats
                return {"approach": "workflow", "doc_type": doc_type}
            else:
                print(f"Using traditional approach for {doc_type} file")
                # Use traditional approach for other formats
                return {"approach": "traditional", "doc_type": doc_type}
        
        # Test routing
        test_files = [
            "/path/to/document.pdf",
            "/path/to/document.txt",
            "/path/to/document.docx",
            "/path/to/document.csv"
        ]
        
        print("\nDocument routing decisions:")
        for file_path in test_files:
            result = await smart_process(file_path)
            print(f"  {Path(file_path).name} -> {result['approach']}")
        
    except ImportError as e:
        print(f"Import error: {e}")
        print("Integration demonstration skipped")


async def example_workflow_debugging():
    """Example: Debugging and visualization techniques"""
    print("\n=== Workflow Debugging Example ===")
    
    try:
        from llama_index.workflows import Context
        from lab_submission_rag.workflows.document_processing import DocumentProcessingWorkflow
        
        # Create workflow
        workflow = DocumentProcessingWorkflow(verbose=True)
        
        # Create context for step-by-step execution
        ctx = Context(workflow)
        
        print("Workflow created for debugging")
        print("In production, you can:")
        print("  1. Use draw_all_possible_flows() to visualize the workflow")
        print("  2. Stream events for real-time monitoring")
        print("  3. Execute steps manually with run_step()")
        print("  4. Inspect context data between steps")
        
        # Example of what you can inspect
        print("\nWorkflow information:")
        print(f"  Timeout: {workflow.timeout} seconds")
        print(f"  Verbose mode: {workflow.verbose}")
        
    except ImportError as e:
        print(f"Import error: {e}")
        print("Debugging demonstration skipped")


async def main():
    """Run all examples"""
    print("=" * 60)
    print("LlamaIndex Workflows Examples for TracSeq 2.0")
    print("=" * 60)
    
    # Run examples
    await example_document_processing()
    await example_quality_control()
    await example_mlops_experiment()
    await example_multi_agent()
    await example_workflow_integration()
    await example_workflow_debugging()
    
    print("\n" + "=" * 60)
    print("Examples completed!")
    print("\nNext steps:")
    print("1. Install dependencies: pip install llama-index-workflows")
    print("2. Update imports to match your project structure")
    print("3. Create test documents in ./test_data/")
    print("4. Run individual workflows with real data")
    print("5. Monitor execution with logging and event streaming")
    print("=" * 60)


if __name__ == "__main__":
    # Run the examples
    asyncio.run(main())