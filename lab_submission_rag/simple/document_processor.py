#!/usr/bin/env python3
"""
Simple Document Processor
Extracted from simple_lab_rag.py for better modularity
"""

from pathlib import Path
from typing import Union

import pypdf
from docx import Document as DocxDocument


class SimpleDocumentProcessor:
    """Simplified document processor for basic file types"""

    def __init__(self):
        self.supported_extensions = {".pdf", ".docx", ".txt"}

    def can_process(self, file_path: Union[str, Path]) -> bool:
        """Check if file can be processed"""
        return Path(file_path).suffix.lower() in self.supported_extensions

    def extract_text(self, file_path: Union[str, Path]) -> str:
        """Extract text from document"""
        file_path = Path(file_path)

        if not file_path.exists():
            raise FileNotFoundError(f"File not found: {file_path}")

        if not self.can_process(file_path):
            raise ValueError(f"Unsupported file type: {file_path.suffix}")

        try:
            if file_path.suffix.lower() == ".pdf":
                return self._extract_from_pdf(file_path)
            elif file_path.suffix.lower() == ".docx":
                return self._extract_from_docx(file_path)
            else:  # .txt
                return self._extract_from_txt(file_path)
        except Exception as e:
            raise RuntimeError(f"Failed to extract text from {file_path}: {str(e)}")

    def _extract_from_pdf(self, file_path: Path) -> str:
        """Extract text from PDF"""
        text = ""
        with open(file_path, "rb") as file:
            pdf_reader = pypdf.PdfReader(file)
            for page in pdf_reader.pages:
                text += page.extract_text() + "\n"
        return text.strip()

    def _extract_from_docx(self, file_path: Path) -> str:
        """Extract text from DOCX"""
        doc = DocxDocument(file_path)
        text = "\n".join([paragraph.text for paragraph in doc.paragraphs])
        return text.strip()

    def _extract_from_txt(self, file_path: Path) -> str:
        """Extract text from TXT"""
        # Try multiple encodings to handle various text files
        encodings = ["utf-8", "utf-8-sig", "latin1", "cp1252"]

        for encoding in encodings:
            try:
                with open(file_path, encoding=encoding) as file:
                    return file.read().strip()
            except UnicodeDecodeError:
                continue

        # If all encodings fail, read as binary and decode with errors='ignore'
        with open(file_path, "rb") as file:
            return file.read().decode("utf-8", errors="ignore").strip()
