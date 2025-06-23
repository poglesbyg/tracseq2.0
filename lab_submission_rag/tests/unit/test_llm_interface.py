"""
Unit tests for LLM interface classes
"""

import sys
from pathlib import Path
from unittest.mock import AsyncMock, Mock, patch

import pytest

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from rag.enhanced_llm_interface import EnhancedLLMInterface
from rag.llm_interface import LLMInterface


class TestLLMInterface:
    """Test cases for LLMInterface"""

    @pytest.fixture
    def mock_openai_client(self):
        """Mock OpenAI client"""
        mock_client = AsyncMock()
        mock_response = Mock()
        mock_response.choices = [Mock(message=Mock(content="Test LLM response"))]
        mock_client.chat.completions.create = AsyncMock(return_value=mock_response)
        return mock_client

    @pytest.mark.asyncio
    @patch("rag.llm_interface.openai")
    async def test_llm_interface_initialization(self, mock_openai):
        """Test LLMInterface initialization"""
        llm = LLMInterface()

        assert llm.client_type is not None

    @pytest.mark.asyncio
    @patch("rag.llm_interface.openai")
    async def test_answer_query(self, mock_openai, sample_document_chunks):
        """Test query answering functionality"""
        llm = LLMInterface()

        # Create mock chunks as tuples (content, score)
        chunk_tuples = [(chunk.content, 0.9) for chunk in sample_document_chunks[:2]]

        response = await llm.answer_query("Test query", chunk_tuples)

        assert isinstance(response, str)
        assert len(response) > 0

    @pytest.mark.asyncio
    @patch("rag.llm_interface.openai")
    async def test_answer_query_empty_chunks(self, mock_openai):
        """Test query answering with empty chunks"""
        llm = LLMInterface()

        response = await llm.answer_query("Test query", [])

        assert isinstance(response, str)
        assert len(response) > 0

    @pytest.mark.asyncio
    @patch("rag.llm_interface.openai")
    async def test_answer_query_exception_handling(self, mock_openai):
        """Test query answering with exception handling"""
        # Mock to raise an exception
        mock_openai.ChatCompletion.acreate.side_effect = Exception("API Error")

        llm = LLMInterface()

        response = await llm.answer_query("Test query", [])

        # Should return error message on exception
        assert isinstance(response, str)
        assert len(response) > 0

    @pytest.mark.asyncio
    @patch("rag.llm_interface.openai")
    async def test_prepare_context(self, mock_openai, sample_document_chunks):
        """Test context preparation from chunks"""
        llm = LLMInterface()

        # Create mock chunks as tuples (content, score)
        chunk_tuples = [(chunk.content, 0.9) for chunk in sample_document_chunks[:3]]
        context = llm._prepare_context(chunk_tuples)

        assert isinstance(context, str)
        assert len(context) > 0
        # Should contain content from all chunks
        for chunk_content, _ in chunk_tuples:
            assert chunk_content in context or any(
                word in context for word in chunk_content.split()
            )

    @pytest.mark.asyncio
    @patch("rag.llm_interface.openai")
    async def test_prepare_context_empty_chunks(self, mock_openai):
        """Test context preparation with empty chunks"""
        llm = LLMInterface()

        context = llm._prepare_context([])

        # Should return empty string or minimal context
        assert isinstance(context, str)


class TestEnhancedLLMInterface:
    """Test cases for EnhancedLLMInterface"""

    @pytest.fixture
    def mock_openai_client(self):
        """Mock OpenAI client"""
        mock_client = AsyncMock()
        mock_response = Mock()
        mock_response.choices = [Mock(message=Mock(content="Enhanced LLM response"))]
        mock_client.chat.completions.create = AsyncMock(return_value=mock_response)
        return mock_client

    @pytest.mark.asyncio
    @patch("rag.enhanced_llm_interface.openai")
    async def test_enhanced_llm_initialization(self, mock_openai):
        """Test EnhancedLLMInterface initialization"""
        enhanced_llm = EnhancedLLMInterface()

        assert enhanced_llm.client_type is not None
        assert enhanced_llm.conversation_contexts == {}

    @pytest.mark.asyncio
    @patch("rag.enhanced_llm_interface.openai")
    async def test_answer_query_new_session(self, mock_openai, sample_document_chunks):
        """Test answering query for new session"""
        enhanced_llm = EnhancedLLMInterface()

        # Create mock chunks as tuples (content, score)
        chunk_tuples = [(chunk.content, 0.9) for chunk in sample_document_chunks[:2]]

        response = await enhanced_llm.answer_query("Test query", chunk_tuples, "session_1")

        assert isinstance(response, str)
        assert len(response) > 0
        assert "session_1" in enhanced_llm.conversation_contexts

    @pytest.mark.asyncio
    @patch("rag.enhanced_llm_interface.openai")
    async def test_answer_query_existing_session(self, mock_openai):
        """Test answering query for existing session with conversation history"""
        enhanced_llm = EnhancedLLMInterface()

        # First query to establish session
        await enhanced_llm.answer_query("First query", [], "session_1")

        # Second query should use conversation history
        response = await enhanced_llm.answer_query("Follow-up query", [], "session_1")

        assert isinstance(response, str)
        assert len(response) > 0
        context = enhanced_llm.get_conversation_context("session_1")
        assert len(context.messages) == 4  # 2 queries + 2 responses

    @pytest.mark.asyncio
    @patch("rag.enhanced_llm_interface.openai")
    async def test_conversation_memory_management(self, mock_openai):
        """Test conversation memory management and limits"""
        enhanced_llm = EnhancedLLMInterface()

        # Add many queries to test memory limit
        for i in range(15):  # Test memory management
            await enhanced_llm.answer_query(f"Query {i}", [], "session_1")

        context = enhanced_llm.get_conversation_context("session_1")
        # Memory should be limited to prevent context overflow
        assert len(context.messages) <= 20  # Should have reasonable limit

    @pytest.mark.asyncio
    @patch("rag.enhanced_llm_interface.openai")
    async def test_multiple_sessions(self, mock_openai):
        """Test handling multiple conversation sessions"""
        enhanced_llm = EnhancedLLMInterface()

        # Create multiple sessions
        await enhanced_llm.answer_query("Query for session 1", [], "session_1")
        await enhanced_llm.answer_query("Query for session 2", [], "session_2")
        await enhanced_llm.answer_query("Another query for session 1", [], "session_1")

        assert "session_1" in enhanced_llm.conversation_contexts
        assert "session_2" in enhanced_llm.conversation_contexts

        context1 = enhanced_llm.get_conversation_context("session_1")
        context2 = enhanced_llm.get_conversation_context("session_2")
        assert len(context1.messages) == 4  # 2 queries + 2 responses
        assert len(context2.messages) == 2  # 1 query + 1 response

    @pytest.mark.asyncio
    @patch("rag.enhanced_llm_interface.openai")
    async def test_context_integration(self, mock_openai, sample_document_chunks):
        """Test integration of document context into responses"""
        enhanced_llm = EnhancedLLMInterface()

        # Create mock chunks as tuples (content, score)
        chunk_tuples = [(chunk.content, 0.9) for chunk in sample_document_chunks[:2]]

        response = await enhanced_llm.answer_query(
            "Test query with context", chunk_tuples, "session_1"
        )

        assert isinstance(response, str)
        assert len(response) > 0

    @pytest.mark.asyncio
    @patch("rag.enhanced_llm_interface.openai")
    async def test_error_handling_in_enhanced_llm(self, mock_openai):
        """Test error handling in enhanced LLM interface"""
        # Mock to raise an exception
        mock_openai.ChatCompletion.acreate.side_effect = Exception("Enhanced API Error")

        enhanced_llm = EnhancedLLMInterface()

        response = await enhanced_llm.answer_query("Test query", [], "session_1")

        # Should return error message and not crash
        assert isinstance(response, str)
        assert len(response) > 0
        # Session should still be created even on error
        assert "session_1" in enhanced_llm.conversation_contexts

    @pytest.mark.asyncio
    @patch("rag.enhanced_llm_interface.openai")
    async def test_lab_specific_responses(self, mock_openai):
        """Test that enhanced LLM provides lab-specific responses"""
        enhanced_llm = EnhancedLLMInterface()

        # Test with lab-specific query
        response = await enhanced_llm.answer_query(
            "How do I submit a blood sample?", [], "session_1"
        )

        assert isinstance(response, str)
        assert len(response) > 0
        # Should mention lab-related concepts
        assert any(
            keyword in response.lower() for keyword in ["sample", "lab", "submit", "manager"]
        )
