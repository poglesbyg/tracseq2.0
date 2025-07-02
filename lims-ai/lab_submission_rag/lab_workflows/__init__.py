"""
Laboratory Submission Workflows using LlamaIndex Workflows 1.0

This module contains event-driven workflows for:
- Document processing and information extraction
- Quality control and validation
- MLOps experiment tracking
- Multi-service orchestration
"""

from .document_processing import DocumentProcessingWorkflow
from .quality_control import QualityControlWorkflow
from .experiment_tracking import ExperimentTrackingWorkflow
from .multi_agent import MultiAgentLabWorkflow

__all__ = [
    "DocumentProcessingWorkflow",
    "QualityControlWorkflow", 
    "ExperimentTrackingWorkflow",
    "MultiAgentLabWorkflow",
]