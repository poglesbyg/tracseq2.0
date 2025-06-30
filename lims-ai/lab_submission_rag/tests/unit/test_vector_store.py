"""
Unit tests for the VectorStore class
"""

import sys
from datetime import datetime
from pathlib import Path
from unittest.mock import Mock, patch

import numpy as np
import pytest

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from models.rag_models import DocumentChunk, VectorStoreInfo
from rag.vector_store import VectorStore


class TestVectorStore:
    """Test cases for VectorStore"""

    @pytest.fixture
    def mock_chromadb_client(self) -> None:
        """Mock ChromaDB client"""
        mock_client = Mock()
        mock_collection = Mock()

        # Mock collection methods
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
        mock_collection.get = Mock(
            return_value={
                "ids": ["test_chunk_001", "test_chunk_002"],
                "metadatas": [
                    {"source_document": "test.pdf", "chunk_index": 0},
                    {"source_document": "test.pdf", "chunk_index": 1},
                ],
            }
        )
        mock_collection.delete = Mock()

        mock_client.get_or_create_collection = Mock(return_value=mock_collection)
        mock_client.reset = Mock()

        return mock_client, mock_collection

    @pytest.fixture
    def mock_embedding_model(self) -> None:
        """Mock sentence transformer model"""
        mock_model = Mock()
        mock_model.encode.return_value = np.array([[0.1, 0.2, 0.3, 0.4, 0.5]])
        return mock_model

    @pytest.mark.asyncio
    @patch("rag.vector_store.chromadb.PersistentClient")
    @patch("rag.vector_store.SentenceTransformer")
    async def test_vector_store_initialization(
        self, mock_transformer, mock_chromadb, mock_chromadb_client
    ) -> None:
        """Test VectorStore initialization"""
        mock_client, mock_collection = mock_chromadb_client
        mock_chromadb.return_value = mock_client
        mock_transformer.return_value = Mock()

        vector_store = VectorStore()

        assert vector_store.client is not None
        assert vector_store.collection is not None
        assert vector_store.embedding_model is not None

    @pytest.mark.asyncio
    @patch("rag.vector_store.chromadb.PersistentClient")
    @patch("rag.vector_store.SentenceTransformer")
    async def test_add_chunks_success(
        self, mock_transformer, mock_chromadb, mock_chromadb_client, sample_document_chunks
    ) -> None:
        """Test successful chunk addition"""
        mock_client, mock_collection = mock_chromadb_client
        mock_chromadb.return_value = mock_client
        mock_model = Mock()
        mock_model.encode = Mock(
            return_value=np.array([[0.1, 0.2, 0.3, 0.4, 0.5]] * len(sample_document_chunks))
        )
        mock_transformer.return_value = mock_model

        vector_store = VectorStore()
        vector_store.embedding_model = mock_model

        result = await vector_store.add_chunks(sample_document_chunks)

        assert result is True
        mock_collection.add.assert_called_once()

    @pytest.mark.asyncio
    @patch("rag.vector_store.chromadb.PersistentClient")
    @patch("rag.vector_store.SentenceTransformer")
    async def test_add_empty_chunks(self, mock_transformer, mock_chromadb, mock_chromadb_client) -> None:
        """Test adding empty chunks list"""
        mock_client, mock_collection = mock_chromadb_client
        mock_chromadb.return_value = mock_client
        mock_transformer.return_value = Mock()

        vector_store = VectorStore()

        result = await vector_store.add_chunks([])

        assert result is True
        mock_collection.add.assert_not_called()

    @pytest.mark.asyncio
    @patch("rag.vector_store.chromadb.PersistentClient")
    @patch("rag.vector_store.SentenceTransformer")
    async def test_similarity_search_success(
        self, mock_transformer, mock_chromadb, mock_chromadb_client
    ) -> None:
        """Test successful similarity search"""
        mock_client, mock_collection = mock_chromadb_client
        mock_chromadb.return_value = mock_client
        mock_model = Mock()
        mock_model.encode = Mock(return_value=np.array([[0.1, 0.2, 0.3, 0.4, 0.5]]))
        mock_transformer.return_value = mock_model

        vector_store = VectorStore()
        vector_store.embedding_model = mock_model

        result = await vector_store.similarity_search("test query", k=5)

        assert len(result) == 1
        chunk, score = result[0]
        assert isinstance(chunk, DocumentChunk)
        assert isinstance(score, float)
        assert 0 <= score <= 1

    @pytest.mark.asyncio
    @patch("rag.vector_store.chromadb.PersistentClient")
    @patch("rag.vector_store.SentenceTransformer")
    async def test_similarity_search_with_filter(
        self, mock_transformer, mock_chromadb, mock_chromadb_client
    ) -> None:
        """Test similarity search with metadata filter"""
        mock_client, mock_collection = mock_chromadb_client
        mock_chromadb.return_value = mock_client
        mock_model = Mock()
        mock_model.encode = Mock(return_value=np.array([[0.1, 0.2, 0.3, 0.4, 0.5]]))
        mock_transformer.return_value = mock_model

        vector_store = VectorStore()
        vector_store.embedding_model = mock_model

        filter_metadata = {"source_document": "test.pdf"}
        result = await vector_store.similarity_search(
            "test query", k=3, filter_metadata=filter_metadata
        )

        mock_collection.query.assert_called_once()
        call_args = mock_collection.query.call_args
        assert call_args[1]["where"] == filter_metadata
        assert call_args[1]["n_results"] == 3

    @pytest.mark.asyncio
    @patch("rag.vector_store.chromadb.PersistentClient")
    @patch("rag.vector_store.SentenceTransformer")
    async def test_similarity_search_no_results(
        self, mock_transformer, mock_chromadb, mock_chromadb_client
    ) -> None:
        """Test similarity search with no results"""
        mock_client, mock_collection = mock_chromadb_client
        mock_collection.query.return_value = {
            "documents": [],
            "metadatas": [],
            "ids": [],
            "distances": [],
        }
        mock_chromadb.return_value = mock_client
        mock_model = Mock()
        mock_model.encode = Mock(return_value=np.array([[0.1, 0.2, 0.3, 0.4, 0.5]]))
        mock_transformer.return_value = mock_model

        vector_store = VectorStore()
        vector_store.embedding_model = mock_model

        result = await vector_store.similarity_search("test query")

        assert result == []

    @pytest.mark.asyncio
    @patch("rag.vector_store.chromadb.PersistentClient")
    @patch("rag.vector_store.SentenceTransformer")
    async def test_generate_embeddings(self, mock_transformer, mock_chromadb, mock_chromadb_client) -> None:
        """Test embedding generation"""
        mock_client, mock_collection = mock_chromadb_client
        mock_chromadb.return_value = mock_client
        mock_model = Mock()
        mock_model.encode = Mock(return_value=np.array([[0.1, 0.2], [0.3, 0.4]]))
        mock_transformer.return_value = mock_model

        vector_store = VectorStore()
        vector_store.embedding_model = mock_model

        texts = ["text 1", "text 2"]
        embeddings = await vector_store._generate_embeddings(texts)

        assert isinstance(embeddings, np.ndarray)
        assert embeddings.shape == (2, 2)
        mock_model.encode.assert_called_once_with(texts)

    @pytest.mark.asyncio
    @patch("rag.vector_store.chromadb.PersistentClient")
    @patch("rag.vector_store.SentenceTransformer")
    async def test_delete_by_source_success(
        self, mock_transformer, mock_chromadb, mock_chromadb_client
    ) -> None:
        """Test successful deletion by source document"""
        mock_client, mock_collection = mock_chromadb_client
        mock_chromadb.return_value = mock_client
        mock_transformer.return_value = Mock()

        vector_store = VectorStore()

        result = await vector_store.delete_by_source("test.pdf")

        assert result is True
        mock_collection.get.assert_called_once_with(where={"source_document": "test.pdf"})
        mock_collection.delete.assert_called_once()

    @pytest.mark.asyncio
    @patch("rag.vector_store.chromadb.PersistentClient")
    @patch("rag.vector_store.SentenceTransformer")
    async def test_delete_by_source_no_chunks(
        self, mock_transformer, mock_chromadb, mock_chromadb_client
    ) -> None:
        """Test deletion by source when no chunks exist"""
        mock_client, mock_collection = mock_chromadb_client
        mock_collection.get.return_value = {"ids": []}
        mock_chromadb.return_value = mock_client
        mock_transformer.return_value = Mock()

        vector_store = VectorStore()

        result = await vector_store.delete_by_source("nonexistent.pdf")

        assert result is True
        mock_collection.delete.assert_not_called()

    @pytest.mark.asyncio
    @patch("rag.vector_store.chromadb.PersistentClient")
    @patch("rag.vector_store.SentenceTransformer")
    async def test_get_store_info(self, mock_transformer, mock_chromadb, mock_chromadb_client) -> None:
        """Test getting vector store information"""
        mock_client, mock_collection = mock_chromadb_client
        mock_chromadb.return_value = mock_client
        mock_transformer.return_value = Mock()

        vector_store = VectorStore()

        with patch.object(vector_store, "_get_storage_size", return_value=1024):
            info = vector_store.get_store_info()

        assert isinstance(info, VectorStoreInfo)
        assert info.total_chunks == 5
        assert info.storage_size == 1024
        assert isinstance(info.last_updated, datetime)

    @pytest.mark.asyncio
    @patch("rag.vector_store.chromadb.PersistentClient")
    @patch("rag.vector_store.SentenceTransformer")
    async def test_reset_store(self, mock_transformer, mock_chromadb, mock_chromadb_client) -> None:
        """Test resetting the vector store"""
        mock_client, mock_collection = mock_chromadb_client
        mock_chromadb.return_value = mock_client
        mock_transformer.return_value = Mock()

        vector_store = VectorStore()

        result = vector_store.reset_store()

        assert result is True
        mock_client.reset.assert_called_once()

    @pytest.mark.asyncio
    @patch("rag.vector_store.chromadb.PersistentClient")
    @patch("rag.vector_store.SentenceTransformer")
    async def test_add_chunks_exception_handling(
        self, mock_transformer, mock_chromadb, mock_chromadb_client, sample_document_chunks
    ) -> None:
        """Test chunk addition with exception handling"""
        mock_client, mock_collection = mock_chromadb_client
        mock_collection.add.side_effect = Exception("Database error")
        mock_chromadb.return_value = mock_client
        mock_model = Mock()
        mock_model.encode = Mock(
            return_value=np.array([[0.1, 0.2, 0.3, 0.4, 0.5]] * len(sample_document_chunks))
        )
        mock_transformer.return_value = mock_model

        vector_store = VectorStore()
        vector_store.embedding_model = mock_model

        result = await vector_store.add_chunks(sample_document_chunks)

        assert result is False

    @pytest.mark.asyncio
    @patch("rag.vector_store.chromadb.PersistentClient")
    @patch("rag.vector_store.SentenceTransformer")
    async def test_similarity_search_exception_handling(
        self, mock_transformer, mock_chromadb, mock_chromadb_client
    ) -> None:
        """Test similarity search with exception handling"""
        mock_client, mock_collection = mock_chromadb_client
        mock_collection.query.side_effect = Exception("Query error")
        mock_chromadb.return_value = mock_client
        mock_model = Mock()
        mock_model.encode = Mock(return_value=np.array([[0.1, 0.2, 0.3, 0.4, 0.5]]))
        mock_transformer.return_value = mock_model

        vector_store = VectorStore()
        vector_store.embedding_model = mock_model

        result = await vector_store.similarity_search("test query")

        assert result == []

    def test_get_storage_size(self, temp_dir) -> None:
        """Test storage size calculation"""
        with patch("rag.vector_store.settings") as mock_settings:
            mock_settings.vector_store_path = str(temp_dir)

            # Create some test files
            (temp_dir / "file1.db").write_text("test content 1")
            (temp_dir / "file2.db").write_text("test content 2")

            with (
                patch("rag.vector_store.chromadb.PersistentClient"),
                patch("rag.vector_store.SentenceTransformer"),
            ):
                vector_store = VectorStore()
                size = vector_store._get_storage_size()

                assert size > 0
