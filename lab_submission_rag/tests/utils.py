"""
Test utilities for the RAG system tests
"""

import asyncio
import tempfile
import json
from pathlib import Path
from typing import List, Dict, Any, Optional
from unittest.mock import Mock, AsyncMock
import io

from models.rag_models import DocumentChunk, ExtractionResult
from models.submission import LabSubmission


class TestDataGenerator:
    """Generate test data for various testing scenarios"""
    
    @staticmethod
    def create_sample_chunks(count: int = 5) -> List[DocumentChunk]:
        """Create sample document chunks for testing"""
        chunks = []
        for i in range(count):
            chunk = DocumentChunk(
                chunk_id=f"test_chunk_{i:03d}",
                content=f"This is test document chunk number {i} with laboratory information about sample processing.",
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
    
    @staticmethod
    def create_lab_submission_data() -> Dict[str, Any]:
        """Create sample lab submission data"""
        return {
            "patient_name": "John Doe",
            "patient_id": "PAT-12345",
            "date_of_birth": "1990-05-15",
            "sample_id": "SAM-67890",
            "sample_type": "Blood",
            "collection_date": "2024-01-15",
            "collection_time": "09:30",
            "tests_requested": ["CBC", "Lipid Panel", "Glucose"],
            "healthcare_provider": "Dr. Jane Smith",
            "provider_license": "ML-98765",
            "clinical_notes": "Routine health screening"
        }
    
    @staticmethod
    def create_extraction_result(success: bool = True) -> ExtractionResult:
        """Create sample extraction result"""
        return ExtractionResult(
            success=success,
            confidence_score=0.85 if success else 0.3,
            missing_fields=[] if success else ["patient_name", "sample_id"],
            warnings=["Minor formatting issue"] if success else ["Document format not recognized"],
            processing_time=2.5,
            source_document="test_document.pdf",
            submission_id="SUB-12345" if success else None,
            extracted_data=TestDataGenerator.create_lab_submission_data() if success else {}
        )
    
    @staticmethod
    def create_pdf_content() -> bytes:
        """Create mock PDF content for testing"""
        return b"""
        %PDF-1.4
        Laboratory Submission Form
        
        Patient Information:
        Name: John Doe
        Patient ID: PAT-12345
        Date of Birth: 1990-05-15
        
        Sample Information:
        Sample ID: SAM-67890
        Sample Type: Blood
        Collection Date: 2024-01-15
        """


class MockFactory:
    """Factory for creating mock objects for testing"""
    
    @staticmethod
    def create_mock_chromadb_client():
        """Create a mock ChromaDB client"""
        mock_client = Mock()
        mock_collection = Mock()
        
        # Configure collection methods
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
        
        return mock_client, mock_collection
    
    @staticmethod
    def create_mock_openai_client():
        """Create a mock OpenAI client"""
        mock_client = AsyncMock()
        mock_response = Mock()
        mock_response.choices = [Mock(message=Mock(content="Test LLM response"))]
        mock_client.chat.completions.create = AsyncMock(return_value=mock_response)
        return mock_client
    
    @staticmethod
    def create_mock_embedding_model():
        """Create a mock sentence transformer model"""
        mock_model = Mock()
        mock_model.encode = Mock(return_value=[[0.1, 0.2, 0.3, 0.4, 0.5]])
        return mock_model


class TestFileManager:
    """Manage test files and cleanup"""
    
    def __init__(self):
        self.temp_files = []
        self.temp_dirs = []
    
    def create_temp_file(self, content: str, suffix: str = ".txt") -> Path:
        """Create a temporary file with content"""
        temp_file = tempfile.NamedTemporaryFile(mode='w', suffix=suffix, delete=False)
        temp_file.write(content)
        temp_file.close()
        
        file_path = Path(temp_file.name)
        self.temp_files.append(file_path)
        return file_path
    
    def create_temp_dir(self) -> Path:
        """Create a temporary directory"""
        temp_dir = Path(tempfile.mkdtemp())
        self.temp_dirs.append(temp_dir)
        return temp_dir
    
    def cleanup(self):
        """Clean up all temporary files and directories"""
        for file_path in self.temp_files:
            if file_path.exists():
                file_path.unlink()
        
        for dir_path in self.temp_dirs:
            if dir_path.exists():
                import shutil
                shutil.rmtree(dir_path)
        
        self.temp_files.clear()
        self.temp_dirs.clear()


class AssertionHelpers:
    """Helper functions for common test assertions"""
    
    @staticmethod
    def assert_valid_document_chunk(chunk: DocumentChunk):
        """Assert that a DocumentChunk is valid"""
        assert isinstance(chunk, DocumentChunk)
        assert chunk.chunk_id is not None
        assert len(chunk.content) > 0
        assert chunk.source_document is not None
        assert isinstance(chunk.metadata, dict)
        assert chunk.chunk_index >= 0
    
    @staticmethod
    def assert_valid_extraction_result(result: ExtractionResult):
        """Assert that an ExtractionResult is valid"""
        assert isinstance(result, ExtractionResult)
        assert isinstance(result.success, bool)
        assert 0 <= result.confidence_score <= 1
        assert isinstance(result.missing_fields, list)
        assert isinstance(result.warnings, list)
        assert result.processing_time >= 0
        assert result.source_document is not None
        assert isinstance(result.extracted_data, dict)
    
    @staticmethod
    def assert_llm_response_valid(response: str):
        """Assert that an LLM response is valid"""
        assert isinstance(response, str)
        assert len(response) > 0
        assert len(response) <= 10000  # Reasonable upper limit
    
    @staticmethod
    def assert_vector_search_results(results: List[tuple]):
        """Assert that vector search results are valid"""
        assert isinstance(results, list)
        for chunk, score in results:
            AssertionHelpers.assert_valid_document_chunk(chunk)
            assert isinstance(score, (int, float))
            assert 0 <= score <= 1


class PerformanceTestHelper:
    """Helper for performance testing"""
    
    @staticmethod
    async def measure_async_execution_time(coro):
        """Measure execution time of an async function"""
        import time
        start_time = time.time()
        result = await coro
        end_time = time.time()
        execution_time = end_time - start_time
        return result, execution_time
    
    @staticmethod
    def assert_execution_time_within_limit(execution_time: float, limit_seconds: float):
        """Assert that execution time is within acceptable limits"""
        assert execution_time <= limit_seconds, f"Execution took {execution_time:.2f}s, limit was {limit_seconds}s"
    
    @staticmethod
    async def run_concurrent_operations(operations: List, max_workers: int = 5):
        """Run multiple operations concurrently and return results"""
        semaphore = asyncio.Semaphore(max_workers)
        
        async def run_with_semaphore(operation):
            async with semaphore:
                return await operation
        
        tasks = [run_with_semaphore(op) for op in operations]
        return await asyncio.gather(*tasks)


def create_test_upload_file(filename: str, content: bytes, content_type: str = "application/pdf"):
    """Create a test upload file for API testing"""
    return (filename, io.BytesIO(content), content_type)


def validate_api_response_structure(response_data: dict, expected_fields: List[str]):
    """Validate that API response has expected structure"""
    for field in expected_fields:
        assert field in response_data, f"Expected field '{field}' not found in response"


def create_mock_rag_system():
    """Create a comprehensive mock RAG system for testing"""
    mock_rag = Mock()
    
    # Mock process_document
    mock_extraction_result = TestDataGenerator.create_extraction_result(success=True)
    mock_rag.process_document = AsyncMock(return_value=mock_extraction_result)
    
    # Mock query_submissions
    mock_rag.query_submissions = AsyncMock(return_value="Test response from mock RAG")
    
    # Mock get_system_status
    mock_rag.get_system_status = AsyncMock(return_value={
        "status": "healthy",
        "total_documents": 10,
        "total_chunks": 50,
        "last_updated": "2024-01-15T10:30:00"
    })
    
    return mock_rag 
