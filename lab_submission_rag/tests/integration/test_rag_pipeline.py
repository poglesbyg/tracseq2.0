"""
End-to-end integration tests for the RAG pipeline
"""

import asyncio
import sys
from pathlib import Path
from unittest.mock import AsyncMock, Mock, patch

import pytest

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from models.rag_models import DocumentChunk
from rag.document_processor import DocumentProcessor
from rag.enhanced_llm_interface import EnhancedLLMInterface
from rag.llm_interface import LLMInterface
from rag.vector_store import VectorStore
from rag_orchestrator import LabSubmissionRAG


class TestRAGPipeline:
    """Integration tests for the complete RAG pipeline"""

    @pytest.fixture
    def mock_dependencies(self):
        """Mock external dependencies for integration testing"""
        with (
            patch("rag.vector_store.chromadb.PersistentClient") as mock_chromadb,
            patch("rag.vector_store.SentenceTransformer") as mock_transformer,
            patch("rag.llm_interface.AsyncOpenAI") as mock_openai,
            patch("rag.enhanced_llm_interface.AsyncOpenAI") as mock_enhanced_openai,
        ):

            # Mock ChromaDB
            mock_collection = Mock()
            mock_collection.add = Mock()
            mock_collection.query = Mock(
                return_value={
                    "documents": [["Test document content"]],
                    "metadatas": [[{"source_document": "test.pdf", "chunk_index": 0}]],
                    "ids": [["test_chunk_001"]],
                    "distances": [[0.1]],
                    "embeddings": [[[0.1, 0.2, 0.3, 0.4, 0.5]]],
                }
            )
            mock_collection.count = Mock(return_value=5)

            mock_client = Mock()
            mock_client.get_or_create_collection = Mock(return_value=mock_collection)
            mock_chromadb.return_value = mock_client

            # Mock embedding model
            mock_model = Mock()
            mock_model.encode = Mock(return_value=[[0.1, 0.2, 0.3, 0.4, 0.5]])
            mock_transformer.return_value = mock_model

            # Mock LLM
            mock_llm_response = Mock()
            mock_llm_response.choices = [Mock(message=Mock(content="Test LLM response"))]
            mock_llm_client = AsyncMock()
            mock_llm_client.chat.completions.create = AsyncMock(return_value=mock_llm_response)
            mock_openai.return_value = mock_llm_client
            mock_enhanced_openai.return_value = mock_llm_client

            yield {
                "chromadb": mock_chromadb,
                "transformer": mock_transformer,
                "openai": mock_openai,
                "enhanced_openai": mock_enhanced_openai,
                "collection": mock_collection,
                "llm_client": mock_llm_client,
            }

    @pytest.mark.asyncio
    async def test_document_to_vector_store_pipeline(
        self, mock_dependencies, temp_dir, sample_text_content
    ):
        """Test the pipeline from document processing to vector store"""
        # Create a test document
        test_doc = temp_dir / "test.pdf"
        with open(test_doc, "w") as f:
            f.write(sample_text_content)

        # Initialize components
        processor = DocumentProcessor()
        vector_store = VectorStore()

        # Mock PDF processing
        with patch.object(processor, "_process_pdf") as mock_process_pdf:
            mock_chunk = DocumentChunk(
                chunk_id="test_chunk_001",
                content=sample_text_content,
                metadata={"file_type": "pdf", "page_number": 1},
                source_document=str(test_doc),
                chunk_index=0,
            )
            mock_process_pdf.return_value = [mock_chunk]

            # Process document
            chunks = await processor.process_document(test_doc)

            # Add to vector store
            result = await vector_store.add_chunks(chunks)

            assert len(chunks) == 1
            assert result is True
            assert chunks[0].content == sample_text_content

    @pytest.mark.asyncio
    async def test_query_to_response_pipeline(self, mock_dependencies, sample_document_chunks):
        """Test the pipeline from query to response generation"""
        # Initialize components
        vector_store = VectorStore()
        llm = LLMInterface()

        # Mock similarity search
        with patch.object(vector_store, "similarity_search") as mock_search:
            mock_search.return_value = [(chunk, 0.9) for chunk in sample_document_chunks[:3]]

            # Perform similarity search
            query = "How do I submit a laboratory sample?"
            relevant_chunks = await vector_store.similarity_search(query, k=3)

            # Generate response
            chunks_only = [chunk for chunk, score in relevant_chunks]
            response = await llm.generate_response(query, chunks_only)

            assert len(relevant_chunks) == 3
            assert response == "Test LLM response"
            mock_search.assert_called_once_with(query, k=3)

    @pytest.mark.asyncio
    async def test_enhanced_llm_conversation_flow(self, mock_dependencies):
        """Test enhanced LLM conversation flow with memory"""
        enhanced_llm = EnhancedLLMInterface()
        session_id = "test_session"

        # First query
        response1 = await enhanced_llm.answer_query(
            "What are the requirements for blood sample collection?", [], session_id
        )

        # Follow-up query
        response2 = await enhanced_llm.answer_query(
            "How long does it take to process?", [], session_id
        )

        assert response1 == "Test LLM response"
        assert response2 == "Test LLM response"
        assert session_id in enhanced_llm.conversation_memory
        assert len(enhanced_llm.conversation_memory[session_id]) == 4  # 2 queries + 2 responses

    @pytest.mark.asyncio
    async def test_full_rag_orchestrator_pipeline(
        self, mock_dependencies, temp_dir, sample_text_content
    ):
        """Test the complete RAG orchestrator pipeline"""
        # Create test document
        test_doc = temp_dir / "test_submission.pdf"
        with open(test_doc, "w") as f:
            f.write(sample_text_content)

        # Initialize RAG orchestrator
        rag = LabSubmissionRAG()

        # Mock document processing
        with patch.object(rag.document_processor, "process_document") as mock_process:
            mock_chunk = DocumentChunk(
                chunk_id="test_chunk_001",
                content=sample_text_content,
                metadata={"file_type": "pdf"},
                source_document=str(test_doc),
                chunk_index=0,
            )
            mock_process.return_value = [mock_chunk]

            # Process document
            result = await rag.process_document(str(test_doc))

            assert result.success is True
            assert result.source_document == str(test_doc)
            assert result.processing_time > 0

    @pytest.mark.asyncio
    async def test_rag_query_with_context(self, mock_dependencies, sample_document_chunks):
        """Test RAG query with document context"""
        rag = LabSubmissionRAG()

        # Mock vector store search
        with patch.object(rag.vector_store, "similarity_search") as mock_search:
            mock_search.return_value = [(chunk, 0.9) for chunk in sample_document_chunks[:2]]

            # Query with context
            response = await rag.query_submissions(
                "What tests are available?", session_id="test_session"
            )

            assert isinstance(response, str)
            assert len(response) > 0
            mock_search.assert_called_once()

    @pytest.mark.asyncio
    async def test_error_handling_in_pipeline(self, mock_dependencies, temp_dir):
        """Test error handling throughout the pipeline"""
        # Create invalid document
        invalid_doc = temp_dir / "invalid.pdf"
        with open(invalid_doc, "w") as f:
            f.write("Invalid content")

        rag = LabSubmissionRAG()

        # Mock processor to raise exception
        with patch.object(rag.document_processor, "process_document") as mock_process:
            mock_process.side_effect = Exception("Processing error")

            # Should handle error gracefully
            result = await rag.process_document(str(invalid_doc))

            assert result.success is False
            assert "error" in result.warnings[0].lower()

    @pytest.mark.asyncio
    async def test_system_status_integration(self, mock_dependencies):
        """Test system status reporting"""
        rag = LabSubmissionRAG()

        status = await rag.get_system_status()

        assert "status" in status
        assert "vector_store" in status or "total_chunks" in status

    @pytest.mark.asyncio
    async def test_concurrent_operations(self, mock_dependencies, sample_document_chunks):
        """Test concurrent operations on the RAG system"""
        rag = LabSubmissionRAG()

        # Mock vector store operations
        with patch.object(rag.vector_store, "similarity_search") as mock_search:
            mock_search.return_value = [(chunk, 0.9) for chunk in sample_document_chunks[:2]]

            # Perform concurrent queries
            tasks = []
            for i in range(5):
                task = rag.query_submissions(f"Query {i}", session_id=f"session_{i}")
                tasks.append(task)

            # Wait for all tasks to complete
            responses = await asyncio.gather(*tasks)

            # All should succeed
            assert len(responses) == 5
            assert all(isinstance(response, str) and len(response) > 0 for response in responses)

    @pytest.mark.asyncio
    async def test_memory_management_under_load(self, mock_dependencies):
        """Test memory management under load"""
        enhanced_llm = EnhancedLLMInterface()

        # Create many sessions
        for session_id in range(10):
            for query_num in range(5):
                await enhanced_llm.answer_query(f"Query {query_num}", [], f"session_{session_id}")

        # Should have created 10 sessions
        assert len(enhanced_llm.conversation_memory) == 10

        # Each session should have reasonable memory usage
        for session_id, memory in enhanced_llm.conversation_memory.items():
            assert len(memory) <= 20  # Should limit memory per session
