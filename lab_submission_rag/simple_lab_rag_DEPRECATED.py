#!/usr/bin/env python3
"""
DEPRECATED: Simple Laboratory Submission RAG System

This file has been refactored for better modularity. The original 999-line file
has been split into smaller, more maintainable components.

NEW USAGE:
- Use simple_lab_rag_refactored.py for the main system
- Individual components are now in the simple/ directory:
  - simple.models: Data structures and Pydantic models
  - simple.document_processor: Document text extraction
  - simple.llm_interface: LLM interactions

MIGRATION GUIDE:
Old: from simple_lab_rag import SimpleLabRAG
New: from simple_lab_rag_refactored import LightweightLabRAG

This file is kept for reference but will be removed in future versions.
Please update your imports to use the refactored version.
"""

import warnings

warnings.warn(
    "This file is deprecated. Use simple_lab_rag_refactored.py and the simple/ module instead.",
    DeprecationWarning,
    stacklevel=2,
)

print("⚠️  DEPRECATED FILE - Please use simple_lab_rag_refactored.py instead")
print("📚 See the file header for migration instructions")
