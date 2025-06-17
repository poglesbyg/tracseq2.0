"""
TracSeq 2.0 MLOps Infrastructure

This package provides machine learning operations infrastructure for the RAG system,
including model management, experiment tracking, A/B testing, and continuous learning.
"""

from .model_registry import ModelRegistry
from .experiment_tracker import ExperimentTracker
from .ab_testing import ABTestManager
from .continuous_learning import ContinuousLearningPipeline
from .deployment_manager import ModelDeploymentManager
from .monitoring import ModelMonitor
from .data_pipeline import DataPipeline

__version__ = "1.0.0"
__author__ = "TracSeq Development Team"

__all__ = [
    "ModelRegistry",
    "ExperimentTracker", 
    "ABTestManager",
    "ContinuousLearningPipeline",
    "ModelDeploymentManager",
    "ModelMonitor",
    "DataPipeline"
] 
