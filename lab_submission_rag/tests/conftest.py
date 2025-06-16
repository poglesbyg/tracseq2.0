"""
Pytest configuration and shared fixtures for the RAG system tests
"""

import pytest
import asyncio
import tempfile
import shutil
from pathlib import Path
from typing import List, Dict, Any
from unittest.mock import Mock, AsyncMock, MagicMock
import os
import sys

# Add the parent directory to the path to import the application modules
sys.path.insert(0, str(Path(__file__).parent.parent))

from models.rag_models import DocumentChunk, DocumentMetadata, ExtractionResult
from models.submission import LabSubmission
from rag.document_processor import DocumentProcessor
from rag.vector_store import VectorStore
from rag.llm_interface import LLMInterface
from rag.enhanced_llm_interface import EnhancedLLMInterface


@pytest.fixture(scope="session")
def event_loop():
    """Create an instance of the default event loop for the test session."""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()


@pytest.fixture
def temp_dir():
    """Create a temporary directory for test files"""
    temp_dir = tempfile.mkdtemp()
    yield Path(temp_dir)
    shutil.rmtree(temp_dir)


@pytest.fixture
def sample_text_content():
    """Sample text content for testing document processing"""
    return """
    Laboratory Submission Form
    
    Patient Information:
    Name: John Doe
    Patient ID: PAT-12345
    Date of Birth: 1990-05-15
    
    Sample Information:
    Sample ID: SAM-67890
    Sample Type: Blood
    Collection Date: 2024-01-15
    Collection Time: 09:30
    
    Tests Requested:
    - Complete Blood Count (CBC)
    - Lipid Panel
    - Glucose Level
    
    Clinical Information:
    Patient presents with fatigue and routine health screening.
    No known allergies.
    Currently taking multivitamin supplements.
    
    Healthcare Provider:
    Dr. Jane Smith
    Medical License: ML-98765
    Contact: jsmith@example.com
    """


@pytest.fixture
def sample_document_chunk():
    """Create a sample DocumentChunk for testing"""
    return DocumentChunk(
        chunk_id="test_chunk_001",
        content="This is a test document chunk with sample laboratory information.",
        metadata={
            "file_type": "pdf",
            "page_number": 1,
            "chunk_index": 0,
            "source_document": "test_document.pdf"
        },
        source_document="test_document.pdf",
        chunk_index=0,
        embedding=[0.1, 0.2, 0.3, 0.4, 0.5]  # Sample embedding vector
    )


@pytest.fixture
def sample_document_chunks():
    """Create multiple sample DocumentChunks for testing"""
    chunks = []
    for i in range(5):
        chunk = DocumentChunk(
            chunk_id=f"test_chunk_{i:03d}",
            content=f"This is test document chunk number {i} with laboratory information.",
            metadata={
                "file_type": "pdf",
                "page_number": (i // 2) + 1,
                "chunk_index": i,
                "source_document": "test_document.pdf"
            },
            source_document="test_document.pdf",
            chunk_index=i,
            embedding=[0.1 * i, 0.2 * i, 0.3 * i, 0.4 * i, 0.5 * i]
        )
        chunks.append(chunk)
    return chunks


@pytest.fixture
def sample_lab_submission():
    """Create a sample LabSubmission for testing"""
    return LabSubmission(
        patient_name="John Doe",
        patient_id="PAT-12345",
        date_of_birth="1990-05-15",
        sample_id="SAM-67890",
        sample_type="Blood",
        collection_date="2024-01-15",
        collection_time="09:30",
        tests_requested=["CBC", "Lipid Panel", "Glucose"],
        healthcare_provider="Dr. Jane Smith",
        provider_license="ML-98765",
        clinical_notes="Routine health screening"
    )


@pytest.fixture
def mock_embedding_model():
    """Mock embedding model for testing"""
    mock_model = Mock()
    mock_model.encode.return_value = [[0.1, 0.2, 0.3, 0.4, 0.5]]
    return mock_model


@pytest.fixture
def mock_chromadb_client():
    """Mock ChromaDB client for testing"""
    mock_client = Mock()
    mock_collection = Mock()
    
    # Mock collection methods
    mock_collection.add = Mock()
    mock_collection.query = Mock(return_value={
        'documents': [['Test document content']],
        'metadatas': [[{'source_document': 'test.pdf', 'chunk_index': 0}]],
        'ids': [['test_chunk_001']],
        'distances': [[0.1]],
        'embeddings': [[[0.1, 0.2, 0.3, 0.4, 0.5]]]
    })
    mock_collection.count = Mock(return_value=5)
    mock_collection.get = Mock(return_value={
        'ids': ['test_chunk_001', 'test_chunk_002'],
        'metadatas': [
            {'source_document': 'test.pdf'},
            {'source_document': 'test.pdf'}
        ]
    })
    mock_collection.delete = Mock()
    
    mock_client.get_or_create_collection = Mock(return_value=mock_collection)
    mock_client.reset = Mock()
    
    return mock_client


@pytest.fixture
def mock_llm_client():
    """Mock LLM client for testing"""
    mock_client = AsyncMock()
    mock_client.chat.completions.create = AsyncMock(return_value=Mock(
        choices=[Mock(message=Mock(content="This is a test response from the LLM."))]
    ))
    return mock_client


@pytest.fixture
def sample_pdf_file(temp_dir):
    """Create a sample PDF file for testing"""
    # For testing purposes, we'll create a simple text file
    # In a real scenario, you'd want to create an actual PDF
    pdf_path = temp_dir / "test_document.pdf"
    with open(pdf_path, "w") as f:
        f.write("This is a test PDF document for laboratory submission testing.")
    return pdf_path


@pytest.fixture
def sample_docx_file(temp_dir):
    """Create a sample DOCX file for testing"""
    # For testing purposes, we'll create a simple text file
    # In a real scenario, you'd want to create an actual DOCX
    docx_path = temp_dir / "test_document.docx"
    with open(docx_path, "w") as f:
        f.write("This is a test DOCX document for laboratory submission testing.")
    return docx_path


@pytest.fixture
def sample_extraction_result():
    """Create a sample ExtractionResult for testing"""
    return ExtractionResult(
        success=True,
        confidence_score=0.85,
        missing_fields=[],
        warnings=["Minor formatting inconsistency detected"],
        processing_time=2.5,
        source_document="test_document.pdf",
        submission_id="SUB-12345",
        extracted_data={
            "patient_name": "John Doe",
            "patient_id": "PAT-12345",
            "sample_type": "Blood"
        }
    )


@pytest.fixture
def mock_document_processor():
    """Mock DocumentProcessor for testing"""
    processor = Mock(spec=DocumentProcessor)
    processor.process_document = AsyncMock(return_value=[])
    return processor


@pytest.fixture
def mock_vector_store():
    """Mock VectorStore for testing"""
    store = Mock(spec=VectorStore)
    store.add_chunks = AsyncMock(return_value=True)
    store.similarity_search = AsyncMock(return_value=[])
    store.get_store_info = Mock()
    store.delete_by_source = AsyncMock(return_value=True)
    store.reset_store = Mock(return_value=True)
    return store


@pytest.fixture
def mock_rag_system():
    """Mock RAG system for integration testing"""
    from rag_orchestrator import LabSubmissionRAG
    
    mock_rag = Mock(spec=LabSubmissionRAG)
    mock_rag.process_document = AsyncMock()
    mock_rag.query_submissions = AsyncMock(return_value="Test response")
    mock_rag.get_system_status = AsyncMock(return_value={"status": "healthy"})
    return mock_rag


# Test data constants
TEST_QUERIES = [
    "How do I submit a laboratory sample?",
    "What are the requirements for blood sample collection?",
    "How long does it take to process a CBC test?",
    "What information is required for a lab submission?",
    "Can you help me understand the test results?"
]

TEST_DOCUMENTS = [
    "Laboratory submission guidelines",
    "Blood collection procedures",
    "Test result interpretation guide",
    "Patient information requirements",
    "Quality control procedures"
] 
