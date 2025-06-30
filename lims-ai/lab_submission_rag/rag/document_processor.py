"""
Document processing component for the RAG system
"""

import logging
from pathlib import Path

import aiofiles
from docx import Document
from langchain.text_splitter import RecursiveCharacterTextSplitter
from pypdf import PdfReader

from config import settings
from models.rag_models import DocumentChunk
from typing import Any

logger = logging.getLogger(__name__)


class DocumentProcessor:
    """Processes various document types for RAG pipeline"""

    def __init__(self) -> None:
        self.text_splitter = RecursiveCharacterTextSplitter(
            chunk_size=settings.chunk_size,
            chunk_overlap=settings.chunk_overlap,
            length_function=len,
        )

    async def process_document(self, file_path: str | Path) -> list[DocumentChunk]:
        """Process a single document and return chunks"""
        file_path = Path(file_path)

        if not file_path.exists():
            logger.error(f"File {file_path} does not exist")
            return []

        if file_path.suffix == ".pdf":
            return await self._process_pdf(file_path)
        elif file_path.suffix == ".docx":
            return await self._process_docx(file_path)
        elif file_path.suffix == ".txt":
            return await self._process_txt(file_path)
        else:
            logger.error(f"Unsupported file type: {file_path.suffix}")
            return []

    async def _process_txt(self, file_path: Path) -> list[DocumentChunk]:
        """Process a text document and return chunks"""
        chunks = []
        try:
            logger.info(f"Starting to process TXT file: {file_path}")
            async with aiofiles.open(file_path, encoding="utf-8") as file:
                text = await file.read()
                logger.info(f"Read {len(text)} characters from file")
                logger.debug(f"First 200 characters: {text[:200]}")

                if text.strip():
                    logger.info("Text content found, splitting into chunks")
                    # Split text into smaller chunks
                    text_chunks = self.text_splitter.split_text(text)
                    logger.info(f"Text splitter produced {len(text_chunks)} chunks")

                    for chunk_idx, chunk_text in enumerate(text_chunks):
                        logger.debug(f"Processing chunk {chunk_idx}, length: {len(chunk_text)}")
                        if chunk_text.strip():
                            chunk = self._create_chunk(
                                chunk_text, file_path, page_number=1, chunk_index=chunk_idx
                            )
                            chunks.append(chunk)
                            logger.debug(f"Created chunk {chunk_idx} with ID: {chunk.chunk_id}")
                        else:
                            logger.warning(f"Chunk {chunk_idx} is empty after stripping")
                else:
                    logger.warning("No text content found in file after stripping whitespace")

            logger.info(f"Successfully processed TXT file, created {len(chunks)} chunks")
        except Exception as e:
            logger.error(f"Error processing TXT {file_path}: {str(e)}")
            return []
        return chunks

    async def _process_pdf(self, file_path: Path) -> list[DocumentChunk]:
        """Process a PDF document and return chunks"""
        chunks = []
        try:
            with open(file_path, "rb") as file:
                pdf_reader = PdfReader(file)
                for page_num, page in enumerate(pdf_reader.pages, 1):
                    text = page.extract_text()
                    if text.strip():  # Only create chunks for non-empty text
                        # Split text into smaller chunks if needed
                        text_chunks = self.text_splitter.split_text(text)
                        for chunk_idx, chunk_text in enumerate(text_chunks):
                            if chunk_text.strip():
                                chunks.append(
                                    self._create_chunk(
                                        chunk_text,
                                        file_path,
                                        page_number=page_num,
                                        chunk_index=chunk_idx,
                                    )
                                )
        except Exception as e:
            logger.error(f"Error processing PDF {file_path}: {str(e)}")
            return []
        return chunks

    async def _process_docx(self, file_path: Path) -> list[DocumentChunk]:
        """Process a DOCX document and return chunks"""
        chunks = []
        try:
            doc = Document(str(file_path))
            # Combine all paragraphs into pages (every 10 paragraphs = 1 page)
            page_size = 10
            all_text = []

            for paragraph in doc.paragraphs:
                if paragraph.text.strip():
                    all_text.append(paragraph.text)

            # Group paragraphs into pages
            for page_num, start_idx in enumerate(range(0, len(all_text), page_size), 1):
                page_text = "\n".join(all_text[start_idx : start_idx + page_size])
                if page_text.strip():
                    # Split page text into smaller chunks if needed
                    text_chunks = self.text_splitter.split_text(page_text)
                    for chunk_idx, chunk_text in enumerate(text_chunks):
                        if chunk_text.strip():
                            chunks.append(
                                self._create_chunk(
                                    chunk_text,
                                    file_path,
                                    page_number=page_num,
                                    chunk_index=chunk_idx,
                                )
                            )
        except Exception as e:
            logger.error(f"Error processing DOCX {file_path}: {str(e)}")
            return []
        return chunks

    def _create_chunk(
        self, text: str, file_path: Path, page_number: int = 1, chunk_index: int = 0
    ) -> DocumentChunk:
        """Create a DocumentChunk from a text and file path"""
        chunk_id = f"{file_path.stem}_page{page_number}_chunk{chunk_index}"
        chunk_content = (
            self.text_splitter.split_text(text)[0] if self.text_splitter.split_text(text) else text
        )
        metadata = {
            "file_path": str(file_path),
            "file_type": file_path.suffix[1:],
            "page_number": page_number,
            "chunk_index": chunk_index,
        }
        return DocumentChunk(
            chunk_id=chunk_id,
            content=chunk_content,
            metadata=metadata,
            embedding=None,
            source_document=str(file_path),
            chunk_index=chunk_index,
        )
