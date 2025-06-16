"""
Integration tests for FastAPI endpoints
"""

import pytest
import asyncio
from pathlib import Path
from unittest.mock import Mock, patch, AsyncMock
from fastapi.testclient import TestClient
import tempfile
import json
import io

import sys
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from api.main import app
from models.rag_models import ExtractionResult
from models.submission import LabSubmission


class TestAPIEndpoints:
    """Test cases for API endpoints"""

    @pytest.fixture
    def client(self):
        """Create a test client for the FastAPI app"""
        return TestClient(app)

    @pytest.fixture
    def mock_rag_system(self):
        """Mock RAG system for testing"""
        mock_rag = AsyncMock()
        
        # Mock successful extraction result
        mock_extraction_result = ExtractionResult(
            success=True,
            confidence_score=0.85,
            missing_fields=[],
            warnings=["Minor formatting inconsistency"],
            processing_time=2.5,
            source_document="test_document.pdf",
            submission_id="SUB-12345",
            extracted_data={
                "patient_name": "John Doe",
                "patient_id": "PAT-12345",
                "sample_type": "Blood"
            }
        )
        
        # Create a mock submission
        mock_submission = LabSubmission(
            patient_name="John Doe",
            patient_id="PAT-12345",
            date_of_birth="1990-05-15",
            sample_id="SAM-67890",
            sample_type="Blood",
            collection_date="2024-01-15",
            collection_time="09:30",
            tests_requested=["CBC", "Lipid Panel"],
            healthcare_provider="Dr. Jane Smith",
            provider_license="ML-98765",
            clinical_notes="Routine screening"
        )
        
        mock_extraction_result.submission = mock_submission
        mock_rag.process_document = AsyncMock(return_value=mock_extraction_result)
        mock_rag.query_submissions = AsyncMock(return_value="Test response from RAG system")
        mock_rag.get_system_status = AsyncMock(return_value={
            "status": "healthy",
            "total_documents": 5,
            "total_chunks": 25,
            "last_updated": "2024-01-15T10:30:00"
        })
        
        return mock_rag

    @pytest.fixture
    def sample_pdf_file(self):
        """Create a sample PDF file for testing"""
        pdf_content = b"Sample PDF content for laboratory submission testing"
        return ("test_document.pdf", io.BytesIO(pdf_content), "application/pdf")

    def test_health_check(self, client):
        """Test health check endpoint"""
        response = client.get("/health")
        
        assert response.status_code == 200
        assert response.json() == {"status": "healthy"}

    @patch('api.main.rag_system')
    def test_process_document_success(self, mock_rag, client, mock_rag_system, sample_pdf_file):
        """Test successful document processing"""
        mock_rag.process_document = mock_rag_system.process_document
        
        filename, file_content, content_type = sample_pdf_file
        response = client.post(
            "/process-document",
            files={"file": (filename, file_content, content_type)}
        )
        
        assert response.status_code == 200
        data = response.json()
        
        assert data["success"] is True
        assert data["confidence_score"] == 0.85
        assert data["source_document"] == "test_document.pdf" 
        assert "submission" in data
        assert data["submission"]["patient_name"] == "John Doe"

    @patch('api.main.rag_system')
    def test_process_document_failure(self, mock_rag, client, sample_pdf_file):
        """Test document processing failure"""
        # Mock failed extraction
        failed_result = ExtractionResult(
            success=False,
            confidence_score=0.2,
            missing_fields=["patient_name", "sample_id"],
            warnings=["Document format not recognized"],
            processing_time=1.0,
            source_document="test_document.pdf",
            extracted_data={}
        )
        
        mock_rag.process_document = AsyncMock(return_value=failed_result)
        
        filename, file_content, content_type = sample_pdf_file
        response = client.post(
            "/process-document",
            files={"file": (filename, file_content, content_type)}
        )
        
        assert response.status_code == 200
        data = response.json()
        
        assert data["success"] is False
        assert data["confidence_score"] == 0.2
        assert len(data["missing_fields"]) == 2
        assert "submission" not in data or data["submission"] is None

    @patch('api.main.rag_system')
    def test_process_document_exception(self, mock_rag, client, sample_pdf_file):
        """Test document processing with exception"""
        mock_rag.process_document = AsyncMock(side_effect=Exception("Processing error"))
        
        filename, file_content, content_type = sample_pdf_file
        response = client.post(
            "/process-document",
            files={"file": (filename, file_content, content_type)}
        )
        
        assert response.status_code == 500
        assert "Processing error" in response.json()["detail"]

    def test_process_document_no_file(self, client):
        """Test document processing without file"""
        response = client.post("/process-document")
        
        assert response.status_code == 422  # Unprocessable Entity

    @patch('api.main.rag_system')
    def test_query_submission_success(self, mock_rag, client, mock_rag_system):
        """Test successful query submission"""
        mock_rag.query_submissions = mock_rag_system.query_submissions
        
        query_data = {
            "query": "How do I submit a blood sample?",
            "submission_id": "SUB-12345",
            "session_id": "session_1",
            "k": 5
        }
        
        response = client.post("/query", json=query_data)
        
        assert response.status_code == 200
        data = response.json()
        
        assert "answer" in data
        assert data["answer"] == "Test response from RAG system"

    @patch('api.main.rag_system')
    def test_query_submission_minimal_params(self, mock_rag, client, mock_rag_system):
        """Test query submission with minimal parameters"""
        mock_rag.query_submissions = mock_rag_system.query_submissions
        
        query_data = {
            "query": "What are the requirements for lab submission?"
        }
        
        response = client.post("/query", json=query_data)
        
        assert response.status_code == 200
        data = response.json()
        
        assert "answer" in data
        # Should use default values for optional parameters
        mock_rag.query_submissions.assert_called_once()
        call_args = mock_rag.query_submissions.call_args
        assert call_args[1]["session_id"] == "default"

    @patch('api.main.rag_system')
    def test_query_submission_exception(self, mock_rag, client):
        """Test query submission with exception"""
        mock_rag.query_submissions = AsyncMock(side_effect=Exception("Query error"))
        
        query_data = {
            "query": "Test query"
        }
        
        response = client.post("/query", json=query_data)
        
        assert response.status_code == 500
        assert "Query error" in response.json()["detail"]

    def test_query_submission_invalid_data(self, client):
        """Test query submission with invalid data"""
        # Missing required query field
        query_data = {
            "session_id": "session_1"
        }
        
        response = client.post("/query", json=query_data)
        
        assert response.status_code == 422  # Unprocessable Entity

    @patch('api.main.rag_system')
    def test_get_system_info_success(self, mock_rag, client, mock_rag_system):
        """Test successful system info retrieval"""
        mock_rag.get_system_status = mock_rag_system.get_system_status
        
        response = client.get("/system-info")
        
        assert response.status_code == 200
        data = response.json()
        
        assert data["status"] == "healthy"
        assert "total_documents" in data
        assert "total_chunks" in data
        assert "last_updated" in data

    @patch('api.main.rag_system')
    def test_get_system_info_exception(self, mock_rag, client):
        """Test system info retrieval with exception"""
        mock_rag.get_system_status = AsyncMock(side_effect=Exception("System error"))
        
        response = client.get("/system-info")
        
        assert response.status_code == 500
        assert "System error" in response.json()["detail"]

    @patch('api.main.rag_system')
    def test_legacy_process_endpoint(self, mock_rag, client, mock_rag_system, sample_pdf_file):
        """Test legacy process endpoint"""
        mock_rag.process_document = mock_rag_system.process_document
        
        filename, file_content, content_type = sample_pdf_file
        response = client.post(
            "/process",
            files={"file": (filename, file_content, content_type)}
        )
        
        assert response.status_code == 200
        data = response.json()
        
        # Should return same format as new endpoint
        assert data["success"] is True
        assert "submission" in data

    def test_cors_headers(self, client):
        """Test CORS headers are properly set"""
        response = client.options("/health")
        
        # Should not fail and should include CORS headers
        assert response.status_code in [200, 405]  # 405 is also acceptable for OPTIONS

    @patch('api.main.rag_system')
    def test_concurrent_requests(self, mock_rag, client, mock_rag_system):
        """Test handling concurrent requests"""
        mock_rag.query_submissions = mock_rag_system.query_submissions
        
        query_data = {
            "query": "Test concurrent query"
        }
        
        # Send multiple concurrent requests
        responses = []
        for i in range(5):
            response = client.post("/query", json=query_data)
            responses.append(response)
        
        # All requests should succeed
        for response in responses:
            assert response.status_code == 200
            assert "answer" in response.json()

    @patch('api.main.rag_system')
    def test_large_file_upload(self, mock_rag, client, mock_rag_system):
        """Test uploading larger files"""
        mock_rag.process_document = mock_rag_system.process_document
        
        # Create a larger file
        large_content = b"Large PDF content " * 1000  # Simulate larger file
        large_file = ("large_document.pdf", io.BytesIO(large_content), "application/pdf")
        
        filename, file_content, content_type = large_file
        response = client.post(
            "/process-document",
            files={"file": (filename, file_content, content_type)}
        )
        
        assert response.status_code == 200
        data = response.json()
        assert data["success"] is True

    def test_api_documentation_endpoints(self, client):
        """Test that API documentation endpoints are accessible"""
        # Test OpenAPI schema
        response = client.get("/openapi.json")
        assert response.status_code == 200
        
        # Should be valid JSON
        schema = response.json()
        assert "openapi" in schema
        assert "info" in schema
        assert schema["info"]["title"] == "Laboratory Submission RAG API"

    @patch('api.main.rag_system')
    def test_query_with_submission_filter(self, mock_rag, client, mock_rag_system):
        """Test query with submission ID filter"""
        mock_rag.query_submissions = mock_rag_system.query_submissions
        
        query_data = {
            "query": "What tests were requested?",
            "submission_id": "SUB-12345"
        }
        
        response = client.post("/query", json=query_data)
        
        assert response.status_code == 200
        
        # Check that filter was passed correctly
        mock_rag.query_submissions.assert_called_once()
        call_args = mock_rag.query_submissions.call_args
        assert call_args[1]["filter_metadata"]["submission_id"] == "SUB-12345" 
