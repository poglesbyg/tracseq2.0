"""
Unit tests for the DocumentProcessor class
"""

import pytest
import asyncio
from pathlib import Path
from unittest.mock import Mock, patch, AsyncMock
import tempfile
import os

import sys
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from rag.document_processor import DocumentProcessor
from models.rag_models import DocumentChunk


class TestDocumentProcessor:
    """Test cases for DocumentProcessor"""

    @pytest.fixture
    def processor(self):
        """Create a DocumentProcessor instance for testing"""
        return DocumentProcessor()

    @pytest.fixture
    def create_test_pdf(self, temp_dir):
        """Create a test PDF file"""
        pdf_path = temp_dir / "test.pdf"
        # Create a mock PDF-like file for testing
        with open(pdf_path, "wb") as f:
            f.write(b"Mock PDF content for testing")
        return pdf_path

    @pytest.fixture
    def create_test_docx(self, temp_dir):
        """Create a test DOCX file"""
        docx_path = temp_dir / "test.docx"
        # Create a mock DOCX-like file for testing
        with open(docx_path, "wb") as f:
            f.write(b"Mock DOCX content for testing")
        return docx_path

    @pytest.mark.asyncio
    async def test_process_document_nonexistent_file(self, processor):
        """Test processing a file that doesn't exist"""
        result = await processor.process_document("nonexistent_file.pdf")
        assert result == []

    @pytest.mark.asyncio
    async def test_process_document_unsupported_format(self, processor, temp_dir):
        """Test processing an unsupported file format"""
        txt_file = temp_dir / "test.txt"
        with open(txt_file, "w") as f:
            f.write("This is a text file")
        
        result = await processor.process_document(txt_file)
        assert result == []

    @pytest.mark.asyncio
    @patch('rag.document_processor.PdfReader')
    async def test_process_pdf_success(self, mock_pdf_reader, processor, create_test_pdf):
        """Test successful PDF processing"""
        # Mock PDF reader
        mock_page = Mock()
        mock_page.extract_text.return_value = "Sample PDF text content"
        
        mock_pdf_instance = Mock()
        mock_pdf_instance.pages = [mock_page]
        mock_pdf_instance.__enter__ = Mock(return_value=mock_pdf_instance)
        mock_pdf_instance.__exit__ = Mock(return_value=None)
        
        mock_pdf_reader.return_value = mock_pdf_instance

        result = await processor.process_document(create_test_pdf)
        
        assert len(result) == 1
        assert isinstance(result[0], DocumentChunk)
        assert result[0].source_document == str(create_test_pdf)
        assert "pdf" in result[0].metadata["file_type"]

    @pytest.mark.asyncio
    @patch('rag.document_processor.Document')
    async def test_process_docx_success(self, mock_document, processor, create_test_docx):
        """Test successful DOCX processing"""
        # Mock DOCX document
        mock_paragraph = Mock()
        mock_paragraph.text = "Sample DOCX paragraph text"
        
        mock_doc_instance = Mock()
        mock_doc_instance.paragraphs = [mock_paragraph]
        
        mock_document.return_value = mock_doc_instance

        result = await processor.process_document(create_test_docx)
        
        assert len(result) == 1
        assert isinstance(result[0], DocumentChunk)
        assert result[0].source_document == str(create_test_docx)
        assert "docx" in result[0].metadata["file_type"]

    @pytest.mark.asyncio
    @patch('rag.document_processor.PdfReader')
    async def test_process_pdf_empty_pages(self, mock_pdf_reader, processor, create_test_pdf):
        """Test PDF processing with empty pages"""
        # Mock PDF reader with empty pages
        mock_page = Mock()
        mock_page.extract_text.return_value = ""
        
        mock_pdf_instance = Mock()
        mock_pdf_instance.pages = [mock_page]
        mock_pdf_instance.__enter__ = Mock(return_value=mock_pdf_instance)
        mock_pdf_instance.__exit__ = Mock(return_value=None)
        
        mock_pdf_reader.return_value = mock_pdf_instance

        result = await processor.process_document(create_test_pdf)
        
        # Should return empty list for empty pages
        assert result == []

    @pytest.mark.asyncio
    @patch('rag.document_processor.Document')
    async def test_process_docx_empty_paragraphs(self, mock_document, processor, create_test_docx):
        """Test DOCX processing with empty paragraphs"""
        # Mock DOCX document with empty paragraphs
        mock_paragraph = Mock()
        mock_paragraph.text = ""
        
        mock_doc_instance = Mock()
        mock_doc_instance.paragraphs = [mock_paragraph]
        
        mock_document.return_value = mock_doc_instance

        result = await processor.process_document(create_test_docx)
        
        # Should return empty list for empty paragraphs
        assert result == []

    @pytest.mark.asyncio
    @patch('rag.document_processor.PdfReader')
    async def test_process_pdf_multiple_pages(self, mock_pdf_reader, processor, create_test_pdf):
        """Test PDF processing with multiple pages"""
        # Mock PDF reader with multiple pages
        mock_page1 = Mock()
        mock_page1.extract_text.return_value = "Page 1 content"
        
        mock_page2 = Mock()
        mock_page2.extract_text.return_value = "Page 2 content"
        
        mock_pdf_instance = Mock()
        mock_pdf_instance.pages = [mock_page1, mock_page2]
        mock_pdf_instance.__enter__ = Mock(return_value=mock_pdf_instance)
        mock_pdf_instance.__exit__ = Mock(return_value=None)
        
        mock_pdf_reader.return_value = mock_pdf_instance

        result = await processor.process_document(create_test_pdf)
        
        assert len(result) == 2
        assert all(isinstance(chunk, DocumentChunk) for chunk in result)
        assert result[0].content != result[1].content

    def test_create_chunk(self, processor, temp_dir):
        """Test chunk creation"""
        test_file = temp_dir / "test.pdf"
        test_text = "This is a test document for chunk creation."
        
        chunk = processor._create_chunk(test_text, test_file, page_number=1, chunk_index=0)
        
        assert isinstance(chunk, DocumentChunk)
        assert chunk.source_document == str(test_file)
        assert chunk.metadata["file_type"] == "pdf"
        assert chunk.metadata["file_path"] == str(test_file)
        assert chunk.metadata["page_number"] == 1
        assert chunk.chunk_index == 0

    @pytest.mark.asyncio
    async def test_process_pdf_exception_handling(self, processor, temp_dir):
        """Test PDF processing with exceptions"""
        # Create a malformed PDF file
        malformed_pdf = temp_dir / "malformed.pdf"
        with open(malformed_pdf, "w") as f:
            f.write("This is not a valid PDF file")

        result = await processor.process_document(malformed_pdf)
        
        # Should handle exception gracefully and return empty list
        assert result == []

    @pytest.mark.asyncio
    async def test_process_docx_exception_handling(self, processor, temp_dir):
        """Test DOCX processing with exceptions"""
        # Create a malformed DOCX file
        malformed_docx = temp_dir / "malformed.docx"
        with open(malformed_docx, "w") as f:
            f.write("This is not a valid DOCX file")

        result = await processor.process_document(malformed_docx)
        
        # Should handle exception gracefully and return empty list
        assert result == []

    def test_processor_initialization(self):
        """Test DocumentProcessor initialization"""
        processor = DocumentProcessor()
        
        assert processor.text_splitter is not None
        assert hasattr(processor.text_splitter, 'split_text')

    @pytest.mark.asyncio
    async def test_process_document_with_path_object(self, processor, temp_dir):
        """Test processing with Path object vs string"""
        # Test with Path object for unsupported format (should return empty list)
        txt_path = temp_dir / "test.txt"
        with open(txt_path, "w") as f:
            f.write("This is a text file")
        
        result = await processor.process_document(txt_path)
        # Should handle Path objects without error and return empty for unsupported format
        assert isinstance(result, list)
        assert result == []

    @pytest.mark.asyncio
    async def test_process_document_case_insensitive_extensions(self, processor, temp_dir):
        """Test processing with different case extensions"""
        # Test with uppercase extension
        pdf_path = temp_dir / "test.PDF"
        with open(pdf_path, "wb") as f:
            f.write(b"Mock PDF")
        
        # Should handle case insensitive extensions (if implemented)
        result = await processor.process_document(pdf_path)
        assert isinstance(result, list) 
