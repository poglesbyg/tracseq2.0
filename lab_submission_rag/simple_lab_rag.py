#!/usr/bin/env python3
"""
Simple Laboratory Submission RAG System

DEPRECATED: This file has been refactored for better modularity.

New usage:
- Use simple_lab_rag_refactored.py for the main system
- Individual components are now in the simple/ directory:
  - simple.models: Data structures
  - simple.document_processor: Document handling
  - simple.llm_interface: LLM interactions

This legacy file is maintained for backward compatibility but will be removed in future versions.
"""

import warnings

warnings.warn(
    "simple_lab_rag.py is deprecated. Use simple_lab_rag_refactored.py instead.",
    DeprecationWarning,
    stacklevel=2,
)

# For backward compatibility, import from the refactored version
try:
    from simple_lab_rag_refactored import LightweightLabRAG, create_demo_document, main

    # Legacy aliases
    SimpleLabRAG = LightweightLabRAG

except ImportError as e:
    print(f"Error importing refactored version: {e}")
    print("Please ensure simple_lab_rag_refactored.py and the simple/ module are available.")
    raise

if __name__ == "__main__":
    print("⚠️ Warning: Using deprecated simple_lab_rag.py")
    print(
        "Please switch to simple_lab_rag_refactored.py for better performance and maintainability."
    )
    print()
    main()

import json
import logging
import os
import uuid
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional, Union

# Core dependencies
try:
    import chromadb
    import ollama
    import pypdf
    from docx import Document as DocxDocument
    from dotenv import load_dotenv
    from pydantic import BaseModel, Field
    from sentence_transformers import SentenceTransformer

    # Optional OpenAI fallback
    try:
        import openai

        OPENAI_AVAILABLE = True
    except ImportError:
        OPENAI_AVAILABLE = False
except ImportError as e:
    print(f"Missing dependency: {e}")
    print("Please install: pip install -r requirements-lite.txt")
    print("For Ollama setup: python setup_simple.py --ollama")
    exit(1)

# Load environment variables
load_dotenv()

# Configure logging
logging.basicConfig(level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s")
logger = logging.getLogger(__name__)


# =============================================================================
# Data Models - Simplified versions of the complex models
# =============================================================================


class AdministrativeInfo(BaseModel):
    """Administrative information from lab submissions"""

    submitter_name: Optional[str] = None
    submitter_email: Optional[str] = None
    submitter_phone: Optional[str] = None
    project_name: Optional[str] = None
    institution: Optional[str] = None


class SampleInfo(BaseModel):
    """Sample information from lab submissions"""

    sample_id: Optional[str] = None
    sample_type: Optional[str] = None  # DNA, RNA, etc.
    concentration: Optional[str] = None
    volume: Optional[str] = None
    storage_conditions: Optional[str] = None


class SequencingInfo(BaseModel):
    """Sequencing information from lab submissions"""

    platform: Optional[str] = None  # Illumina, PacBio, etc.
    analysis_type: Optional[str] = None  # WGS, RNA-seq, etc.
    coverage: Optional[str] = None
    read_length: Optional[str] = None


class LabSubmission(BaseModel):
    """Simplified lab submission model"""

    submission_id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    created_at: datetime = Field(default_factory=datetime.now)

    # Core information categories
    administrative: AdministrativeInfo = Field(default_factory=AdministrativeInfo)
    sample: SampleInfo = Field(default_factory=SampleInfo)
    sequencing: SequencingInfo = Field(default_factory=SequencingInfo)

    # Raw extracted text and metadata
    raw_text: Optional[str] = None
    confidence_score: Optional[float] = None
    source_document: Optional[str] = None


class ExtractionResult(BaseModel):
    """Result of document processing"""

    success: bool
    submission: Optional[LabSubmission] = None
    submission_id: Optional[str] = None
    extracted_data: Optional[Dict[str, Any]] = None
    confidence_score: Optional[float] = None
    error: Optional[str] = None
    warnings: List[str] = Field(default_factory=list)


# =============================================================================
# Document Processor - Simplified document handling
# =============================================================================


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


# =============================================================================
# LLM Interface - Ollama (local) with OpenAI fallback
# =============================================================================


class SimpleLLMInterface:
    """Simplified LLM interface using Ollama (local) with OpenAI fallback"""

    def __init__(
        self,
        model: str = "llama3.2:3b",
        use_openai_fallback: bool = False,
        openai_api_key: Optional[str] = None,
    ):

        self.model = model
        self.use_openai_fallback = use_openai_fallback

        logger.info("Checking Ollama availability...")
        self.ollama_available = self._check_ollama()
        logger.info(f"Ollama available: {self.ollama_available}")

        # Initialize OpenAI client if available and requested
        self.openai_client = None
        if OPENAI_AVAILABLE and (use_openai_fallback or not self.ollama_available):
            api_key = openai_api_key or os.getenv("OPENAI_API_KEY")
            logger.info(f"Checking OpenAI API key... (length: {len(api_key) if api_key else 0})")
            # Check if API key is valid (not a placeholder)
            if api_key and not api_key.startswith("your_") and len(api_key) > 20:
                try:
                    self.openai_client = openai.OpenAI(api_key=api_key)
                    logger.info("OpenAI client initialized successfully")
                except Exception as e:
                    logger.warning(f"OpenAI client initialization failed: {e}")
            else:
                logger.info("OpenAI API key appears to be placeholder or invalid")

        # Check if we have any working LLM
        if not self.ollama_available and not self.openai_client:
            logger.error("No working LLM found!")
            logger.error(f"Ollama available: {self.ollama_available}")
            logger.error(f"OpenAI client: {self.openai_client}")
            raise RuntimeError(
                "No LLM available. Please install Ollama or configure OpenAI API key."
            )

        # Log which LLM will be used
        if self.ollama_available:
            logger.info("✅ Using Ollama (local LLM)")
        elif self.openai_client:
            logger.info("✅ Using OpenAI (cloud LLM) - Ollama not available")

        # Extraction prompt template
        self.extraction_prompt = """
You are an expert at extracting laboratory submission information from documents.
Extract the following information from the text below. If information is not available, leave it as null.

Extract these fields:
- Administrative: submitter_name, submitter_email, submitter_phone, project_name, institution
- Sample: sample_id, sample_type, concentration, volume, storage_conditions  
- Sequencing: platform, analysis_type, coverage, read_length

Respond with valid JSON only, no other text:

{{
  "administrative": {{
    "submitter_name": "value or null",
    "submitter_email": "value or null", 
    "submitter_phone": "value or null",
    "project_name": "value or null",
    "institution": "value or null"
  }},
  "sample": {{
    "sample_id": "value or null",
    "sample_type": "value or null",
    "concentration": "value or null", 
    "volume": "value or null",
    "storage_conditions": "value or null"
  }},
  "sequencing": {{
    "platform": "value or null",
    "analysis_type": "value or null",
    "coverage": "value or null",
    "read_length": "value or null"
  }}
}}

Text to analyze:
{text}
"""

    def _check_ollama(self) -> bool:
        """Check if Ollama is available and running"""
        try:
            # Try to list models to check if Ollama is running
            response = ollama.list()
            logger.info(f"Ollama list response type: {type(response)}")

            # Check if we got a proper response with models attribute
            if hasattr(response, "models"):
                models = response.models
                logger.info(f"Found {len(models)} models in Ollama")
                return True
            else:
                logger.warning(f"Unexpected response structure: {response}")
                return False
        except Exception as e:
            logger.info(f"Ollama not available: {e}")
            return False

    def _ensure_model_available(self) -> bool:
        """Ensure the specified model is available in Ollama"""
        try:
            models_response = ollama.list()
            models = models_response.models if hasattr(models_response, "models") else []
            logger.info(f"Checking model availability. Found {len(models)} models")

            # Extract model names from Model objects
            model_names = []
            for model in models:
                if hasattr(model, "model"):  # Model object with 'model' attribute
                    model_names.append(model.model)
                elif isinstance(model, str):
                    model_names.append(model)

            logger.info(f"Available models: {model_names}")
            logger.info(f"Looking for model: {self.model}")

            if self.model not in model_names:
                logger.warning(f"Model {self.model} not found in available models")
                logger.info(f"Attempting to pull model {self.model}...")
                try:
                    ollama.pull(self.model)
                    logger.info(f"✅ Model {self.model} pulled successfully")
                except Exception as pull_error:
                    logger.error(f"Failed to pull model {self.model}: {pull_error}")
                    return False
            else:
                logger.info(f"✅ Model {self.model} is available")

            return True
        except Exception as e:
            logger.error(f"Failed to ensure model availability: {e}")
            logger.warning(f"Will try to use model {self.model} anyway")
            return True  # Try anyway - Ollama might still work

    def extract_submission_info(self, text: str) -> Dict[str, Any]:
        """Extract structured information from text"""

        # Try Ollama first if available
        if self.ollama_available:
            try:
                if self._ensure_model_available():
                    logger.info(f"Generating response with Ollama model: {self.model}")
                    try:
                        response = ollama.generate(
                            model=self.model,
                            prompt=self.extraction_prompt.format(text=text),
                            options={"temperature": 0.1, "num_predict": 1000},
                        )
                        logger.info("✅ Ollama generate succeeded")
                    except Exception as ollama_error:
                        logger.error(f"Ollama generate failed: {ollama_error}")
                        raise

                    # Parse JSON response - handle different response structures
                    if hasattr(response, "response"):
                        result_text = response.response.strip()
                    elif isinstance(response, dict) and "response" in response:
                        result_text = response["response"].strip()
                    else:
                        logger.error(f"Unexpected response structure: {type(response)}")
                        raise ValueError(f"Unexpected response structure: {type(response)}")

                    logger.info(f"Raw Ollama response: {result_text[:200]}...")

                    # Clean up common Ollama response formatting
                    if result_text.startswith("```json"):
                        result_text = result_text[7:-3].strip()
                    elif result_text.startswith("```"):
                        result_text = result_text[3:-3].strip()

                    # Additional cleaning for common formatting issues
                    # Remove any leading/trailing whitespace and ensure it starts with {
                    result_text = result_text.strip()
                    if not result_text.startswith("{"):
                        # Find the first { and last }
                        start = result_text.find("{")
                        end = result_text.rfind("}")
                        if start != -1 and end != -1:
                            result_text = result_text[start : end + 1]
                        else:
                            # If no valid JSON structure found, try to construct it
                            logger.warning(
                                f"No valid JSON structure found. Raw response: {result_text}"
                            )
                            if '"administrative"' in result_text:
                                # Fallback: Try to wrap in proper JSON structure
                                result_text = "{\n" + result_text.strip() + "\n}"

                    logger.info(f"Cleaned response: {result_text[:200]}...")
                    return json.loads(result_text)
            except json.JSONDecodeError as e:
                logger.warning(f"Ollama JSON parsing failed: {e}")
                logger.warning(f"Response text: {result_text[:500]}...")
                if not self.openai_client:
                    return {"error": f"Ollama JSON parsing failed: {str(e)}"}
            except Exception as e:
                logger.warning(f"Ollama extraction failed: {e}")
                if not self.openai_client:
                    return {"error": f"Ollama failed and no OpenAI fallback: {str(e)}"}

        # Fallback to OpenAI if available
        if self.openai_client:
            try:
                response = self.openai_client.chat.completions.create(
                    model="gpt-3.5-turbo",
                    messages=[
                        {"role": "user", "content": self.extraction_prompt.format(text=text)}
                    ],
                    temperature=0.1,
                    max_tokens=1000,
                )

                result_text = response.choices[0].message.content.strip()

                # Clean up the response before parsing JSON
                if result_text.startswith("```json"):
                    result_text = result_text[7:-3].strip()
                elif result_text.startswith("```"):
                    result_text = result_text[3:-3].strip()

                # Additional cleaning for common formatting issues
                result_text = result_text.strip()
                if not result_text.startswith("{"):
                    # Find the first { and last }
                    start = result_text.find("{")
                    end = result_text.rfind("}")
                    if start != -1 and end != -1:
                        result_text = result_text[start : end + 1]

                # Parse JSON
                return json.loads(result_text)

            except json.JSONDecodeError as e:
                logger.error(f"OpenAI JSON parsing failed: {e}")
                logger.error(f"Response text: {result_text[:500]}...")
                return {"error": f"Failed to parse OpenAI response as JSON: {str(e)}"}
            except Exception as e:
                logger.error(f"OpenAI fallback failed: {e}")
                return {"error": f"Both Ollama and OpenAI failed: {str(e)}"}

        return {"error": "No LLM available for extraction"}

    def answer_query(self, query: str, context: str) -> str:
        """Answer a query using the provided context"""
        prompt = f"""
Based on the laboratory submission information below, answer the following question.
If the information is not available in the context, say "Information not available".

Context:
{context}

Question: {query}

Answer:"""

        # Try Ollama first if available
        if self.ollama_available:
            try:
                if self._ensure_model_available():
                    response = ollama.generate(
                        model=self.model,
                        prompt=prompt,
                        options={"temperature": 0.1, "num_predict": 500},
                    )
                    return response["response"].strip()
            except Exception as e:
                logger.warning(f"Ollama query failed: {e}")

        # Fallback to OpenAI if available
        if self.openai_client:
            try:
                response = self.openai_client.chat.completions.create(
                    model="gpt-3.5-turbo",
                    messages=[{"role": "user", "content": prompt}],
                    temperature=0.1,
                    max_tokens=500,
                )
                return response.choices[0].message.content.strip()
            except Exception as e:
                logger.error(f"OpenAI fallback failed: {e}")
                return f"Error: {str(e)}"

        return "No LLM available to answer query"


class DemoLLMInterface:
    """Demo LLM interface that simulates responses for demonstration"""

    def __init__(self):
        self.demo_response = {
            "administrative": {
                "submitter_name": "Dr. Sarah Johnson",
                "submitter_email": "sarah.johnson@university.edu",
                "submitter_phone": "(555) 123-4567",
                "project_name": "Genomic Diversity Study 2024",
                "institution": "University Research Lab",
            },
            "sample": {
                "sample_id": "DNA_SAMPLE_001",
                "sample_type": "Genomic DNA",
                "concentration": "50 ng/uL",
                "volume": "100 uL",
                "storage_conditions": "Frozen at -80C",
            },
            "sequencing": {
                "platform": "Illumina NovaSeq 6000",
                "analysis_type": "Whole Genome Sequencing",
                "coverage": "30x",
                "read_length": "150bp paired-end",
            },
        }

    def extract_submission_info(self, text: str) -> Dict[str, Any]:
        """Demo extraction that returns simulated data"""
        return self.demo_response

    def answer_query(self, query: str, context: str) -> str:
        """Demo query answering"""
        query_lower = query.lower()

        if "submitter" in query_lower:
            return "The submitter is Dr. Sarah Johnson from University Research Lab."
        elif "sample" in query_lower and "type" in query_lower:
            return "The sample type is Genomic DNA."
        elif "platform" in query_lower or "sequencing" in query_lower:
            return "The sequencing platform is Illumina NovaSeq 6000."
        elif "concentration" in query_lower:
            return "The sample concentration is 50 ng/uL."
        else:
            return f"This is a demo response for the query: '{query}'"


# =============================================================================
# Vector Store - Simplified ChromaDB integration
# =============================================================================


class SimpleVectorStore:
    """Simplified vector store using ChromaDB"""

    def __init__(self, persist_directory: str = "data/vector_store"):
        self.persist_directory = persist_directory
        Path(persist_directory).mkdir(parents=True, exist_ok=True)

        # Initialize ChromaDB
        self.client = chromadb.PersistentClient(path=persist_directory)
        self.collection = self.client.get_or_create_collection(
            name="lab_submissions", metadata={"hnsw:space": "cosine"}
        )

        # Initialize embeddings model
        print("Loading embeddings model...")
        self.embeddings_model = SentenceTransformer("all-MiniLM-L6-v2")

    def add_document(self, submission_id: str, text: str, metadata: Dict[str, Any]):
        """Add document to vector store"""
        try:
            # Create embeddings
            embeddings = self.embeddings_model.encode([text]).tolist()

            # Add to collection
            self.collection.add(
                documents=[text], embeddings=embeddings, metadatas=[metadata], ids=[submission_id]
            )

        except Exception as e:
            logger.error(f"Failed to add document to vector store: {e}")
            raise

    def search(self, query: str, n_results: int = 5) -> List[Dict[str, Any]]:
        """Search for similar documents"""
        try:
            # Create query embedding
            query_embedding = self.embeddings_model.encode([query]).tolist()

            # Search collection
            results = self.collection.query(query_embeddings=query_embedding, n_results=n_results)

            # Format results
            formatted_results = []
            for i in range(len(results["ids"][0])):
                formatted_results.append(
                    {
                        "id": results["ids"][0][i],
                        "document": results["documents"][0][i],
                        "metadata": results["metadatas"][0][i],
                        "distance": results["distances"][0][i],
                    }
                )

            return formatted_results

        except Exception as e:
            logger.error(f"Search failed: {e}")
            return []


# =============================================================================
# Lightweight RAG System for Docker/Web Interface
# =============================================================================


class LightweightLabRAG:
    """Lightweight Laboratory Submission RAG System for Docker deployment"""

    def __init__(self, use_ollama=True, use_openai=True, data_dir="./data"):
        """Initialize the RAG system"""
        # Check for Docker environment variables
        self.use_ollama = use_ollama or os.getenv("USE_OLLAMA", "false").lower() == "true"
        self.use_openai = use_openai
        self.ollama_base_url = os.getenv("OLLAMA_BASE_URL", "http://localhost:11434")

        # Debug environment variable reading
        env_model = os.getenv("OLLAMA_MODEL")
        logger.info(f"🔧 DEBUG: Environment OLLAMA_MODEL = '{env_model}'")
        self.ollama_model = os.getenv("OLLAMA_MODEL", "llama3.2:3b")
        logger.info(f"🔧 DEBUG: Set ollama_model to: '{self.ollama_model}'")

        data_dir = os.getenv("DATA_DIRECTORY", data_dir)
        self.data_dir = Path(data_dir)
        self.data_dir.mkdir(exist_ok=True)

        # Initialize components
        self.document_processor = SimpleDocumentProcessor()
        self.vector_store = None
        self.llm = None

        # Storage for submissions
        self.submissions_file = self.data_dir / "submissions.json"
        self.submissions: Dict[str, Dict] = self._load_submissions()

    async def initialize(self):
        """Initialize the RAG system (async version for web interface)"""
        try:
            # Initialize vector store
            self.vector_store = SimpleVectorStore(str(self.data_dir / "vector_store"))

            # Initialize LLM interface
            try:
                logger.info("Attempting to initialize SimpleLLMInterface...")
                # Try to initialize the real LLM interface
                self.llm = SimpleLLMInterface(
                    model=self.ollama_model,
                    use_openai_fallback=self.use_openai,
                    openai_api_key=os.getenv("OPENAI_API_KEY"),
                )
                logger.info("✅ SimpleLLMInterface initialized successfully!")
            except RuntimeError as e:
                # Fall back to demo mode
                logger.warning(f"LLM initialization failed: {e}")
                logger.info("⚠️  No LLM available - using demo mode")
                self.llm = DemoLLMInterface()
            except Exception as e:
                # Unexpected error, log details
                logger.error(f"Unexpected error initializing LLM: {e}")
                logger.info("⚠️  Falling back to demo mode")
                self.llm = DemoLLMInterface()

        except Exception as e:
            logger.error(f"Failed to initialize RAG system: {e}")
            # Initialize with demo components
            self.llm = DemoLLMInterface()

    async def _check_ollama_connection(self) -> bool:
        """Check if Ollama is available"""
        try:
            import requests

            response = requests.get(f"{self.ollama_base_url}/api/tags", timeout=5)
            return response.status_code == 200
        except Exception:
            return False

    def _load_submissions(self) -> Dict[str, Dict]:
        """Load submissions from file"""
        if self.submissions_file.exists():
            try:
                with open(self.submissions_file) as f:
                    return json.load(f)
            except Exception as e:
                logger.warning(f"Failed to load submissions: {e}")
        return {}

    def _save_submissions(self):
        """Save submissions to file"""
        try:
            with open(self.submissions_file, "w") as f:
                json.dump(self.submissions, f, indent=2, default=str)
        except Exception as e:
            logger.error(f"Failed to save submissions: {e}")

    async def process_document(self, file_path: Union[str, Path]) -> ExtractionResult:
        """Process a single document and extract information"""
        try:
            file_path = Path(file_path)
            logger.info(f"Processing document: {file_path}")

            # Extract text
            text = self.document_processor.extract_text(file_path)
            if not text.strip():
                return ExtractionResult(success=False, error="No text extracted from document")

            # Extract structured information using LLM
            extracted_data = self.llm.extract_submission_info(text)

            if "error" in extracted_data:
                return ExtractionResult(success=False, error=extracted_data["error"])

            # Create submission object
            submission = LabSubmission(
                administrative=AdministrativeInfo(**extracted_data.get("administrative", {})),
                sample=SampleInfo(**extracted_data.get("sample", {})),
                sequencing=SequencingInfo(**extracted_data.get("sequencing", {})),
                raw_text=text,
                confidence_score=0.8,  # Simple confidence score
                source_document=str(file_path),
            )

            # Store submission
            self.submissions[submission.submission_id] = submission.model_dump()
            self._save_submissions()

            # Add to vector store
            if self.vector_store:
                metadata = {
                    "submission_id": submission.submission_id,
                    "source_document": str(file_path),
                    "created_at": submission.created_at.isoformat(),
                }
                self.vector_store.add_document(submission.submission_id, text, metadata)

            logger.info(f"Successfully processed document: {submission.submission_id}")
            return ExtractionResult(
                success=True,
                submission=submission,
                extracted_data=extracted_data,
                confidence_score=0.8,
                submission_id=submission.submission_id,
            )

        except Exception as e:
            logger.error(f"Document processing failed: {e}")
            return ExtractionResult(success=False, error=str(e))

    async def query(self, question: str) -> str:
        """Query the system about processed documents"""
        try:
            # Search for relevant documents
            if self.vector_store:
                search_results = self.vector_store.search(question, n_results=3)
            else:
                search_results = []

            if not search_results:
                return "No relevant information found. Please process some documents first."

            # Combine context from search results
            context = "\n\n".join([result["document"] for result in search_results])

            # Get answer from LLM
            answer = self.llm.answer_query(question, context)
            return answer

        except Exception as e:
            logger.error(f"Query failed: {e}")
            return f"Error processing query: {str(e)}"

    async def export_submissions(self, format: str = "json") -> str:
        """Export all submissions to file"""
        export_dir = self.data_dir / "exports"
        export_dir.mkdir(exist_ok=True)

        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

        if format.lower() == "json":
            export_file = export_dir / f"submissions_{timestamp}.json"
            with open(export_file, "w") as f:
                json.dump(self.submissions, f, indent=2, default=str)
        else:
            raise ValueError(f"Unsupported export format: {format}")

        return str(export_file)


# =============================================================================
# Example Usage and Demo
# =============================================================================


def create_demo_document():
    """Create a demo document for testing"""
    demo_dir = Path("demo")
    demo_dir.mkdir(exist_ok=True)

    demo_content = """
Laboratory Submission Form

Submitter Information:
Name: Dr. Sarah Johnson
Email: sarah.johnson@university.edu
Phone: (555) 123-4567
Institution: University Research Lab
Project: Genomic Diversity Study 2024

Sample Information:
Sample ID: DNA_SAMPLE_001
Sample Type: Genomic DNA
Concentration: 50 ng/uL
Volume: 100 uL
Storage: Frozen at -80C

Sequencing Information:
Platform: Illumina NovaSeq 6000
Analysis Type: Whole Genome Sequencing
Coverage: 30x
Read Length: 150bp paired-end

Additional Notes:
High priority sample for genomic diversity analysis.
Please ensure quality control metrics are included.
"""

    demo_file = demo_dir / "demo_submission.txt"
    demo_file.write_text(demo_content)
    return demo_file


# =============================================================================
# Synchronous Wrapper for Simple Usage
# =============================================================================


class SimpleLabRAG:
    """Synchronous wrapper around LightweightLabRAG"""

    def __init__(self, use_ollama=True, use_openai=True, data_dir="./data"):
        """Initialize the simple RAG system"""
        self.rag = LightweightLabRAG(use_ollama=use_ollama, use_openai=use_openai, data_dir=data_dir)

        # Task management - store references to prevent garbage collection
        self.background_tasks: set = set()

        # Initialize synchronously
        import asyncio

        try:
            loop = asyncio.get_running_loop()
            # If loop is running, schedule the initialization with proper task management
            task = asyncio.create_task(self.rag.initialize())
            self.background_tasks.add(task)
            task.add_done_callback(self.background_tasks.discard)
        except RuntimeError:
            # No loop running, create one
            asyncio.run(self.rag.initialize())

    def process_document(self, file_path):
        """Process a document synchronously"""
        import asyncio

        try:
            return asyncio.run(self.rag.process_document(file_path))
        except RuntimeError:
            # If there's already a loop running, use it
            import concurrent.futures

            loop = asyncio.get_running_loop()
            with concurrent.futures.ThreadPoolExecutor() as executor:
                future = executor.submit(asyncio.run, self.rag.process_document(file_path))
                return future.result()

    def query(self, question: str) -> str:
        """Query the system synchronously"""
        import asyncio

        try:
            return asyncio.run(self.rag.query(question))
        except RuntimeError:
            # If there's already a loop running, use it
            import concurrent.futures

            loop = asyncio.get_running_loop()
            with concurrent.futures.ThreadPoolExecutor() as executor:
                future = executor.submit(asyncio.run, self.rag.query(question))
                return future.result()

    def export_submissions(self, format: str = "json") -> str:
        """Export submissions synchronously"""
        import asyncio

        try:
            return asyncio.run(self.rag.export_submissions(format))
        except RuntimeError:
            # If there's already a loop running, use it
            import concurrent.futures

            loop = asyncio.get_running_loop()
            with concurrent.futures.ThreadPoolExecutor() as executor:
                future = executor.submit(asyncio.run, self.rag.export_submissions(format))
                return future.result()

    @property
    def submissions(self):
        """Get submissions"""
        return self.rag.submissions

    def get_stats(self):
        """Get system statistics"""
        return {
            "total_submissions": len(self.rag.submissions),
            "data_directory": str(self.rag.data_dir),
            "supported_formats": list(self.rag.document_processor.supported_extensions),
        }


def main():
    """Demo of the ultra-lightweight RAG system with Ollama"""
    print("🧬 Ultra-Lightweight Laboratory Submission RAG System")
    print("🦙 Powered by Ollama (Local LLM)")
    print("=" * 60)

    try:
        # Initialize system (will auto-detect Ollama or fallback to OpenAI)
        print("Initializing RAG system...")

        # Check what's available
        has_openai = bool(os.getenv("OPENAI_API_KEY"))

        if has_openai:
            print("🔄 OpenAI API key detected - will use as backup if Ollama unavailable")
        else:
            print("🦙 Running in Ollama-only mode (no API costs!)")

        try:
            rag = SimpleLabRAG(use_ollama=True, use_openai=has_openai)
        except RuntimeError:
            print("⚠️  No working LLM found - running in DEMO MODE")
            print("   This shows how the system works with simulated responses")

            # Create system with demo interface
            rag = SimpleLabRAG.__new__(SimpleLabRAG)  # Create without __init__
            rag.rag = LightweightLabRAG.__new__(LightweightLabRAG)  # Create without __init__
            rag.rag.data_dir = Path("data")
            rag.rag.data_dir.mkdir(parents=True, exist_ok=True)
            rag.rag.document_processor = SimpleDocumentProcessor()
            rag.rag.llm = DemoLLMInterface()
            rag.rag.vector_store = SimpleVectorStore(str(rag.rag.data_dir / "vector_store"))
            rag.rag.submissions_file = rag.rag.data_dir / "submissions.json"
            # Initialize submissions storage
            try:
                if rag.rag.submissions_file.exists():
                    with open(rag.rag.submissions_file) as f:
                        rag.rag.submissions = json.load(f)
                else:
                    rag.rag.submissions = {}
            except:
                rag.rag.submissions = {}

        # Create demo document
        demo_file = create_demo_document()
        print(f"Created demo document: {demo_file}")

        # Process the document
        print("\nProcessing document with local LLM...")
        result = rag.process_document(demo_file)

        if result.success:
            print("✅ Document processed successfully!")
            submission = result.submission

            print("\nExtracted Information:")
            print(f"  Submission ID: {submission.submission_id}")
            print(f"  Submitter: {submission.administrative.submitter_name}")
            print(f"  Email: {submission.administrative.submitter_email}")
            print(f"  Sample Type: {submission.sample.sample_type}")
            print(f"  Platform: {submission.sequencing.platform}")

            # Test queries
            print("\n" + "=" * 60)
            print("Testing Local LLM Queries:")

            queries = [
                "Who is the submitter?",
                "What type of sample is this?",
                "What sequencing platform is being used?",
                "What is the sample concentration?",
            ]

            for query in queries:
                answer = rag.query(query)
                print(f"\nQ: {query}")
                print(f"A: {answer}")

            # System stats
            print("\n" + "=" * 60)
            stats = rag.get_stats()
            print("System Statistics:")
            for key, value in stats.items():
                print(f"  {key}: {value}")

            # Export data
            export_file = rag.export_submissions()
            print(f"\nData exported to: {export_file}")

        else:
            print(f"❌ Processing failed: {result.error}")

    except Exception as e:
        print(f"❌ Error: {e}")
        print("\nTroubleshooting:")
        print("1. Install Ollama: https://ollama.ai/download")
        print("2. Run: ollama pull llama3.2:3b")
        print("3. Install dependencies: pip install -r requirements-lite.txt")
        print("4. Or set OPENAI_API_KEY for fallback mode")


if __name__ == "__main__":
    main()
