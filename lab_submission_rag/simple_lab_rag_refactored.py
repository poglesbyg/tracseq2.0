#!/usr/bin/env python3
"""
Refactored Simple Laboratory Submission RAG System

This is a cleaned-up version of simple_lab_rag.py with better modularity.
Uses the new modular components:
- simple.models for data structures
- simple.document_processor for document handling  
- simple.llm_interface for LLM interactions
"""

import os
import json
import asyncio
from datetime import datetime
from pathlib import Path
from typing import List, Optional, Dict, Any, Union
import logging

# Core dependencies
try:
    import chromadb
    from sentence_transformers import SentenceTransformer
    from dotenv import load_dotenv
except ImportError as e:
    print(f"Missing dependency: {e}")
    print("Please install: pip install -r requirements-lite.txt")
    exit(1)

# Import our modular components
try:
    from simple.models import LabSubmission, ExtractionResult, AdministrativeInfo, SampleInfo, SequencingInfo
    from simple.document_processor import SimpleDocumentProcessor
    from simple.llm_interface import SimpleLLMInterface, DemoLLMInterface
except ImportError as e:
    print(f"Missing modular components: {e}")
    print("Please ensure the simple/ directory contains the required modules")
    exit(1)

# Load environment variables
load_dotenv()

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)


class SimpleVectorStore:
    """Simple vector storage using ChromaDB"""
    
    def __init__(self, persist_directory: str = "data/vector_store"):
        self.persist_directory = persist_directory
        Path(persist_directory).mkdir(parents=True, exist_ok=True)
        
        # Initialize ChromaDB client
        self.client = chromadb.PersistentClient(path=persist_directory)
        self.collection = self.client.get_or_create_collection(
            name="lab_submissions",
            metadata={"description": "Laboratory submission documents"}
        )
        
        # Initialize sentence transformer
        self.embedding_model = SentenceTransformer('all-MiniLM-L6-v2')
        logger.info("Vector store initialized")
    
    def add_document(self, submission_id: str, text: str, metadata: Dict[str, Any]):
        """Add document to vector store"""
        try:
            # Create embedding
            embedding = self.embedding_model.encode(text).tolist()
            
            # Add to collection
            self.collection.add(
                embeddings=[embedding],
                documents=[text],
                metadatas=[metadata],
                ids=[submission_id]
            )
            logger.info(f"Added document {submission_id} to vector store")
        except Exception as e:
            logger.error(f"Failed to add document to vector store: {e}")
    
    def search(self, query: str, n_results: int = 5) -> List[Dict[str, Any]]:
        """Search for similar documents"""
        try:
            # Create query embedding
            query_embedding = self.embedding_model.encode(query).tolist()
            
            # Search
            results = self.collection.query(
                query_embeddings=[query_embedding],
                n_results=n_results
            )
            
            # Format results
            formatted_results = []
            for i in range(len(results['ids'][0])):
                formatted_results.append({
                    'id': results['ids'][0][i],
                    'text': results['documents'][0][i],
                    'metadata': results['metadatas'][0][i],
                    'distance': results['distances'][0][i]
                })
            
            return formatted_results
        except Exception as e:
            logger.error(f"Search failed: {e}")
            return []


class LightweightLabRAG:
    """Lightweight Laboratory RAG System - main class"""
    
    def __init__(self, use_ollama=True, use_openai=True, data_dir="./data"):
        self.use_ollama = use_ollama
        self.use_openai = use_openai
        self.data_dir = Path(data_dir)
        self.data_dir.mkdir(exist_ok=True)
        
        # Initialize components
        self.document_processor = SimpleDocumentProcessor()
        self.vector_store = SimpleVectorStore(str(self.data_dir / "vector_store"))
        
        # Initialize LLM interface
        try:
            self.llm = SimpleLLMInterface(
                model=os.getenv("OLLAMA_MODEL", "llama3.2:3b"),
                use_openai_fallback=use_openai
            )
        except RuntimeError:
            logger.warning("No LLM available, using demo interface")
            self.llm = DemoLLMInterface()
        
        # Storage for submissions
        self.submissions_file = self.data_dir / "submissions.json"
        self.submissions = self._load_submissions()
        
        logger.info("LightweightLabRAG initialized successfully")
    
    def _load_submissions(self) -> Dict[str, Dict]:
        """Load submissions from file"""
        try:
            if self.submissions_file.exists():
                with open(self.submissions_file, 'r') as f:
                    return json.load(f)
        except Exception as e:
            logger.warning(f"Could not load submissions: {e}")
        return {}
    
    def _save_submissions(self):
        """Save submissions to file"""
        try:
            with open(self.submissions_file, 'w') as f:
                json.dump(self.submissions, f, indent=2, default=str)
        except Exception as e:
            logger.error(f"Could not save submissions: {e}")
    
    def process_document(self, file_path: Union[str, Path]) -> ExtractionResult:
        """Process a laboratory document"""
        start_time = datetime.now()
        file_path = Path(file_path)
        
        try:
            # Check if file exists and is supported
            if not file_path.exists():
                return ExtractionResult(
                    success=False,
                    error=f"File not found: {file_path}"
                )
            
            if not self.document_processor.can_process(file_path):
                return ExtractionResult(
                    success=False,
                    error=f"Unsupported file type: {file_path.suffix}"
                )
            
            # Extract text
            logger.info(f"Processing document: {file_path}")
            text = self.document_processor.extract_text(file_path)
            
            if not text.strip():
                return ExtractionResult(
                    success=False,
                    error="No text extracted from document"
                )
            
            # Extract structured information using LLM
            extracted_data = self.llm.extract_submission_info(text)
            
            if "error" in extracted_data:
                return ExtractionResult(
                    success=False,
                    error=extracted_data["error"]
                )
            
            # Create submission object
            submission = LabSubmission(
                administrative=AdministrativeInfo(**extracted_data.get("administrative", {})),
                sample=SampleInfo(**extracted_data.get("sample", {})),
                sequencing=SequencingInfo(**extracted_data.get("sequencing", {})),
                raw_text=text,
                confidence_score=0.85,  # Could be calculated based on completeness
                source_document=str(file_path)
            )
            
            # Store submission
            self.submissions[submission.submission_id] = submission.dict()
            self._save_submissions()
            
            # Add to vector store
            self.vector_store.add_document(
                submission.submission_id,
                text,
                {
                    "source_document": str(file_path),
                    "submitter": submission.administrative.submitter_name,
                    "sample_type": submission.sample.sample_type,
                    "created_at": submission.created_at.isoformat()
                }
            )
            
            processing_time = (datetime.now() - start_time).total_seconds()
            
            return ExtractionResult(
                success=True,
                submission=submission,
                submission_id=submission.submission_id,
                extracted_data=extracted_data,
                confidence_score=submission.confidence_score
            )
            
        except Exception as e:
            logger.error(f"Document processing failed: {e}")
            return ExtractionResult(
                success=False,
                error=str(e)
            )
    
    def query(self, question: str) -> str:
        """Query the RAG system"""
        try:
            # Search for relevant documents
            search_results = self.vector_store.search(question, n_results=3)
            
            # Prepare context
            context_parts = []
            for result in search_results:
                context_parts.append(f"Document: {result['metadata'].get('source_document', 'Unknown')}")
                context_parts.append(f"Content: {result['text'][:500]}...")
                context_parts.append("")
            
            context = "\n".join(context_parts)
            
            # Get answer from LLM
            answer = self.llm.answer_query(question, context)
            return answer
            
        except Exception as e:
            logger.error(f"Query failed: {e}")
            return f"Sorry, I couldn't process your query: {str(e)}"
    
    def export_submissions(self, format: str = "json") -> str:
        """Export submissions to specified format"""
        try:
            if format.lower() == "json":
                export_file = self.data_dir / f"submissions_export_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
                with open(export_file, 'w') as f:
                    json.dump(self.submissions, f, indent=2, default=str)
                return str(export_file)
            else:
                raise ValueError(f"Unsupported export format: {format}")
        except Exception as e:
            logger.error(f"Export failed: {e}")
            return f"Export failed: {str(e)}"
    
    def get_stats(self) -> Dict[str, Any]:
        """Get system statistics"""
        return {
            "total_submissions": len(self.submissions),
            "data_directory": str(self.data_dir),
            "vector_store_items": len(self.vector_store.collection.get()['ids']),
            "last_updated": datetime.now().isoformat()
        }


# Convenience functions for backward compatibility
def create_demo_document():
    """Create a demo document for testing"""
    demo_content = """
Laboratory Sample Submission Form

Submitter Information:
Name: Dr. Jane Smith
Email: jane.smith@university.edu
Phone: (555) 123-4567
Institution: University Research Lab
Project: Cancer Genomics Study 2024

Sample Information:
Sample ID: CANCER_001
Sample Type: Tumor Tissue DNA
Concentration: 75 ng/μL
Volume: 200 μL
Storage: -80°C freezer

Sequencing Requirements:
Platform: Illumina HiSeq 4000
Analysis: Whole Exome Sequencing
Coverage: 100x
Read Length: 150bp paired-end

Priority: High
Quality Score: 9.2/10
Special Instructions: Handle with care, fragile sample
"""
    
    demo_file = Path("demo_submission.txt")
    demo_file.write_text(demo_content)
    return demo_file


def main():
    """Main demonstration function"""
    print("🧬 Simple Laboratory RAG System (Refactored)")
    print("=" * 50)
    
    # Initialize system
    rag = LightweightLabRAG()
    
    # Create demo document
    demo_file = create_demo_document()
    print(f"📄 Created demo document: {demo_file}")
    
    # Process document
    print("\n🔄 Processing document...")
    result = rag.process_document(demo_file)
    
    if result.success:
        print("✅ Processing successful!")
        print(f"   Submission ID: {result.submission_id}")
        print(f"   Confidence: {result.confidence_score:.2f}")
        
        # Show extracted info
        submission = result.submission
        print(f"\n📋 Extracted Information:")
        print(f"   Submitter: {submission.administrative.submitter_name}")
        print(f"   Email: {submission.administrative.submitter_email}")
        print(f"   Sample: {submission.sample.sample_id} ({submission.sample.sample_type})")
        print(f"   Platform: {submission.sequencing.platform}")
        
        # Test query
        print(f"\n❓ Testing query functionality...")
        query_result = rag.query("Who is the submitter?")
        print(f"   Query: 'Who is the submitter?'")
        print(f"   Answer: {query_result}")
        
        # Show stats
        stats = rag.get_stats()
        print(f"\n📊 System Statistics:")
        for key, value in stats.items():
            print(f"   {key}: {value}")
    else:
        print(f"❌ Processing failed: {result.error}")
    
    # Cleanup
    demo_file.unlink()
    print(f"\n🧹 Cleaned up demo file")
    print("🎉 Demo completed!")


if __name__ == "__main__":
    main() 
