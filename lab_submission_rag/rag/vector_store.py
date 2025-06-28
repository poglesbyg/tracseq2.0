"""
Vector store implementation for the RAG system
"""

import asyncio
import logging
from datetime import datetime
from pathlib import Path
from typing import Any

import chromadb
import numpy as np
from chromadb.config import Settings as ChromaSettings
from sentence_transformers import SentenceTransformer

from config import settings
from models.rag_models import DocumentChunk, VectorStoreInfo

logger = logging.getLogger(__name__)


class VectorStore:
    """Manages vector embeddings and similarity search"""

    def __init__(self) -> None:
        self.client = None
        self.collection = None
        self.embedding_model = SentenceTransformer(settings.embedding_model)
        self._initialize_store()

    def _initialize_store(self) -> None:
        """Initialize ChromaDB client and collection"""
        try:
            # Create vector database directory if it doesn't exist
            Path(settings.vector_store_path).mkdir(parents=True, exist_ok=True)

            # Initialize ChromaDB client
            self.client = chromadb.PersistentClient(
                path=str(settings.vector_store_path),
                settings=ChromaSettings(anonymized_telemetry=False, allow_reset=True),
            )

            # Get or create collection
            self.collection = self.client.get_or_create_collection(
                name="lab_submissions", metadata={"description": "Laboratory submission documents"}
            )

            logger.info("Vector store initialized successfully")

        except Exception as e:
            logger.error(f"Failed to initialize vector store: {str(e)}")
            raise

    async def add_chunks(self, chunks: list[DocumentChunk]) -> bool:
        """Add document chunks to the vector store"""
        try:
            if not chunks:
                logger.warning("No chunks to add to vector store")
                return True

            # Generate embeddings for all chunks
            texts = [chunk.content for chunk in chunks]
            embeddings = await self._generate_embeddings(texts)

            # Prepare data for ChromaDB
            ids = [chunk.chunk_id for chunk in chunks]
            metadatas = []

            for chunk in chunks:
                metadata = {
                    "source_document": chunk.source_document,
                    "chunk_index": chunk.chunk_index,
                    **chunk.metadata,
                }
                metadatas.append(metadata)

            # Add to collection
            self.collection.add(
                embeddings=embeddings.tolist(), documents=texts, metadatas=metadatas, ids=ids
            )

            logger.info(f"Added {len(chunks)} chunks to vector store")
            return True

        except Exception as e:
            logger.error(f"Error adding chunks to vector store: {str(e)}")
            return False

    async def similarity_search(
        self, query: str, k: int = 5, filter_metadata: dict[str, Any] | None = None
    ) -> list[tuple[DocumentChunk, float]]:
        """Perform similarity search and return relevant chunks with scores"""
        try:
            # Generate query embedding
            query_embedding = await self._generate_embeddings([query])

            # Perform search
            results = self.collection.query(
                query_embeddings=query_embedding.tolist(), n_results=k, where=filter_metadata
            )

            # Convert results to DocumentChunk objects with scores
            chunks_with_scores = []

            if results["documents"]:
                for i in range(len(results["documents"][0])):
                    chunk = DocumentChunk(
                        chunk_id=results["ids"][0][i],
                        content=results["documents"][0][i],
                        metadata=results["metadatas"][0][i],
                        source_document=results["metadatas"][0][i].get("source_document", ""),
                        chunk_index=results["metadatas"][0][i].get("chunk_index", 0),
                        embedding=results["embeddings"][0][i] if results["embeddings"] else None,
                    )

                    # ChromaDB returns distances, convert to similarity scores
                    distance = results["distances"][0][i]
                    similarity_score = 1 - distance  # Convert distance to similarity

                    chunks_with_scores.append((chunk, similarity_score))

            logger.info(f"Found {len(chunks_with_scores)} relevant chunks for query")
            return chunks_with_scores

        except Exception as e:
            logger.error(f"Error in similarity search: {str(e)}")
            return []

    async def _generate_embeddings(self, texts: list[str]) -> np.ndarray:
        """Generate embeddings for a list of texts"""
        try:
            # Run embedding generation in thread pool to avoid blocking
            loop = asyncio.get_event_loop()
            embeddings = await loop.run_in_executor(None, self.embedding_model.encode, texts)
            return embeddings

        except Exception as e:
            logger.error(f"Error generating embeddings: {str(e)}")
            raise

    def get_store_info(self) -> VectorStoreInfo:
        """Get information about the vector store"""
        try:
            count = self.collection.count()

            return VectorStoreInfo(
                total_documents=len(
                    set(
                        item.get("source_document", "")
                        for item in self.collection.get()["metadatas"]
                    )
                ),
                total_chunks=count,
                embedding_model=settings.embedding_model,
                last_updated=datetime.now(),
                storage_size=self._get_storage_size(),
            )

        except Exception as e:
            logger.error(f"Error getting store info: {str(e)}")
            return VectorStoreInfo(
                total_documents=0,
                total_chunks=0,
                embedding_model=settings.embedding_model,
                last_updated=datetime.now(),
                storage_size=0,
            )

    def _get_storage_size(self) -> int:
        """Calculate storage size of the vector database"""
        try:
            db_path = Path(settings.vector_store_path)
            if db_path.exists():
                return sum(f.stat().st_size for f in db_path.rglob("*") if f.is_file())
            return 0
        except Exception:
            return 0

    async def delete_by_source(self, source_document: str) -> bool:
        """Delete all chunks from a specific source document"""
        try:
            # Find all chunks from the source document
            results = self.collection.get(where={"source_document": source_document})

            if results["ids"]:
                self.collection.delete(ids=results["ids"])
                logger.info(f"Deleted {len(results['ids'])} chunks from {source_document}")
                return True

            logger.info(f"No chunks found for source document: {source_document}")
            return True

        except Exception as e:
            logger.error(f"Error deleting chunks from {source_document}: {str(e)}")
            return False

    def reset_store(self) -> bool:
        """Reset the entire vector store (use with caution)"""
        try:
            self.client.reset()
            self._initialize_store()
            logger.info("Vector store reset successfully")
            return True

        except Exception as e:
            logger.error(f"Error resetting vector store: {str(e)}")
            return False
