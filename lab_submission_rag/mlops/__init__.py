"""
TracSeq 2.0 MLOps Infrastructure

This package provides machine learning operations infrastructure for the RAG system,
including model management, experiment tracking, A/B testing, and continuous learning.
"""

from .ab_testing import ABTestManager
from .continuous_learning import ContinuousLearningPipeline
from .data_pipeline import DataPipeline
from .deployment_manager import ModelDeploymentManager
from .experiment_tracker import ExperimentTracker
from .model_registry import ModelRegistry
from .monitoring import ModelMonitor

__version__ = "1.0.0"
__author__ = "TracSeq Development Team"

__all__ = [
    "ModelRegistry",
    "ExperimentTracker",
    "ABTestManager",
    "ContinuousLearningPipeline",
    "ModelDeploymentManager",
    "ModelMonitor",
    "DataPipeline",
]
