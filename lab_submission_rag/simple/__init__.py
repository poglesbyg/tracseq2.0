"""
Simple Laboratory RAG System Components

Modular components extracted from the original large simple_lab_rag.py file
for better maintainability and organization.

Components:
- models: Data structures and Pydantic models
- document_processor: Document text extraction utilities
- llm_interface: LLM interaction interfaces (Ollama + OpenAI)
"""

__version__ = "1.0.0"

# Import main classes for convenience
from .document_processor import SimpleDocumentProcessor
from .llm_interface import DemoLLMInterface, SimpleLLMInterface
from .models import AdministrativeInfo, ExtractionResult, LabSubmission, SampleInfo, SequencingInfo

__all__ = [
    "LabSubmission",
    "ExtractionResult",
    "AdministrativeInfo",
    "SampleInfo",
    "SequencingInfo",
    "SimpleDocumentProcessor",
    "SimpleLLMInterface",
    "DemoLLMInterface",
]
