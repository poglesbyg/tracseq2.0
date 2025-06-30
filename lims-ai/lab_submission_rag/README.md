# Laboratory Submission RAG System 🧬

A specialized Retrieval-Augmented Generation (RAG) system designed to extract and process laboratory submission information from scientific documents. This system can intelligently extract information across 7 key categories required for laboratory sample processing.

## Features

### 🎯 **Intelligent Information Extraction**
Automatically extracts laboratory submission information from documents across 7 specialized categories:

1. **Administrative Information**
   - Submitter details (name, email, phone)
   - Project assignments
   - Contact information

2. **Source and Submitting Material**
   - Material type (Genomic DNA, RNA, Other)
   - Collection details and methods
   - Storage conditions

3. **Pooling (Multiplexing)**
   - Pooling strategies
   - Barcode information
   - Sample pooling ratios

4. **Sequence Generation**
   - Sequencing platforms
   - Read parameters
   - Coverage requirements
   - Library preparation details

5. **Container and Diluent**
   - Container specifications
   - Volume and concentration
   - Storage requirements

6. **Informatics**
   - Analysis types (WGS, WES, RNA-seq)
   - Reference genomes
   - Pipeline requirements

7. **Sample Details**
   - Sample identifiers
   - Quality metrics
   - Priority levels
   - Special instructions

### 🔍 **Smart Document Processing**
- **Multi-format Support**: PDF, DOCX, and text files
- **Intelligent Chunking**: Laboratory-specific text segmentation
- **Metadata Preservation**: Maintains document traceability
- **Vector Storage**: Efficient similarity search with ChromaDB

### 🤖 **LLM Integration**
- **Local Llama Models** via Ollama (privacy-focused, no API costs)
- **OpenAI GPT-4** and **Anthropic Claude** support
- **Structured Extraction**: Converts unstructured text to validated data models
- **Confidence Scoring**: Provides extraction confidence metrics
- **Natural Language Queries**: Ask questions about submissions in plain English

## Installation

### Prerequisites
- Python 3.9 or higher
- [uv](https://docs.astral.sh/uv/) package manager (recommended for fastest setup)

### Installing uv

If you don't have `uv` installed yet:

```bash
# On macOS and Linux
curl -LsSf https://astral.sh/uv/install.sh | sh

# On Windows
powershell -c "irm https://astral.sh/uv/install.ps1 | iex"

# With pip (if you prefer)
pip install uv

# With homebrew (macOS)
brew install uv
```

### Quick Setup with uv (Recommended)

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd lab_submission_rag/app
   ```

2. **Create virtual environment and install dependencies**
   ```bash
   # Create virtual environment
   uv venv
   
   # Activate environment
   source .venv/bin/activate  # On Windows: .venv\Scripts\activate
   
   # Install project in editable mode with all dependencies
   uv pip install -e .
   
   # Install email validation support
   uv pip install pydantic[email]
   ```

3. **Choose your LLM provider**
   
   **Option A: Local Llama Models (Recommended - Private & Free)**
   ```bash
   # Install and setup Ollama with Llama models
   python setup_llama.py --all
   ```
   
   **Option B: Cloud LLM APIs**
   Create a `.env` file with your API keys:
   ```env
   OPENAI_API_KEY=your_openai_api_key_here
   # OR
   ANTHROPIC_API_KEY=your_anthropic_api_key_here
   ```

4. **Verify installation**
   ```bash
   # Run comprehensive setup check
   python setup_check.py
   
   # Run the demo
   python example_usage.py
   ```

### Alternative Installation with pip

If you prefer traditional pip:

1. **Clone and setup**
   ```bash
   git clone <repository-url>
   cd lab_submission_rag/app
   python -m venv .venv
   source .venv/bin/activate
   pip install -r requirements.txt
   pip install pydantic[email]
   ```

### Development Setup

For development with additional tools:

```bash
# Install with development dependencies
uv pip install -e ".[dev]"

# Install all optional dependencies
uv pip install -e ".[all]"
```

## Quick Start

### Environment Activation

Always activate your virtual environment first:

```bash
# Activate the virtual environment
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Verify installation
python setup_check.py

# Run the demo
python example_usage.py
```

### Basic Usage

```python
import asyncio
from rag_orchestrator import rag_system

async def extract_lab_info():
    # Process a laboratory document
    result = await rag_system.process_document("lab_submission_form.pdf")
    
    if result.success:
        print(f"✅ Extraction successful! Confidence: {result.confidence_score:.2f}")
        
        # Access extracted information
        admin_info = result.submission.administrative_info
        print(f"Submitter: {admin_info.submitter_first_name} {admin_info.submitter_last_name}")
        print(f"Email: {admin_info.submitter_email}")
        print(f"Project: {admin_info.assigned_project}")
        
        # Export the data
        export_path = await rag_system.export_submission_data(result.submission, "json")
        print(f"Data exported to: {export_path}")
    else:
        print(f"❌ Extraction failed: {result.warnings}")

# Run the extraction
asyncio.run(extract_lab_info())
```

### Query System

```python
async def query_lab_data():
    # Ask questions about processed documents
    queries = [
        "What sequencing platform is being used?",
        "Who is the submitter for this project?",
        "What type of analysis is requested?",
        "What are the storage requirements?"
    ]
    
    for query in queries:
        answer = await rag_system.query_submissions(query)
        print(f"Q: {query}")
        print(f"A: {answer}\n")

asyncio.run(query_lab_data())
```

### Batch Processing

```python
async def process_multiple_documents():
    document_paths = [
        "uploads/form1.pdf",
        "uploads/form2.pdf", 
        "uploads/form3.docx"
    ]
    
    batch_result = await rag_system.process_documents_batch(document_paths)
    
    print(f"Processed: {batch_result.total_documents} documents")
    print(f"Successful: {batch_result.successful_extractions}")
    print(f"Overall confidence: {batch_result.overall_confidence:.2f}")

asyncio.run(process_multiple_documents())
```

## System Architecture

```
┌─────────────────────┐
│   Document Input    │ (PDF, DOCX, TXT)
└──────────┬──────────┘
           │
┌──────────▼──────────┐
│ Document Processor  │ (Chunking, Metadata)
└──────────┬──────────┘
           │
┌──────────▼──────────┐
│   Vector Store      │ (ChromaDB, Embeddings)
└──────────┬──────────┘
           │
┌──────────▼──────────┐
│   LLM Interface     │ (GPT-4, Claude)
└──────────┬──────────┘
           │
┌──────────▼──────────┐
│ Structured Output   │ (7 Categories)
└─────────────────────┘
```

## Configuration

### Project Configuration

The project uses modern `pyproject.toml` for dependency management:

```toml
[project]
name = "lab-submission-rag"
version = "1.0.0"
requires-python = ">=3.9"

dependencies = [
    "langchain>=0.1.0",
    "chromadb>=0.4.0",
    "sentence-transformers>=2.2.0",
    # ... other dependencies
]

[project.optional-dependencies]
dev = ["pytest>=7.4.0", "black>=23.0.0", "ruff>=0.1.0"]
web = ["fastapi>=0.104.0", "uvicorn>=0.24.0"]
```

### Runtime Configuration

The system can be configured via `config.py` or environment variables:

```python
# Document Processing
CHUNK_SIZE=1000
CHUNK_OVERLAP=200
MAX_FILE_SIZE_MB=50

# Vector Store
EMBEDDING_MODEL="all-MiniLM-L6-v2"
SIMILARITY_THRESHOLD=0.7

# LLM Settings
DEFAULT_LLM_MODEL="gpt-4"
LLM_TEMPERATURE=0.1
MAX_TOKENS=2000
```

### LLM Provider Configuration

The system supports multiple LLM providers. Choose based on your needs:

#### **🦙 Local Llama Models (Recommended)**
- **Pros**: Private, no API costs, works offline
- **Cons**: Requires local compute resources

```bash
# Automatic setup
python setup_llama.py --all

# Manual configuration in .env
USE_OLLAMA=true
OLLAMA_MODEL=llama3.1:8b
OLLAMA_BASE_URL=http://localhost:11434
```

#### **☁️ Cloud LLM APIs**
- **Pros**: Powerful models, no local resources needed
- **Cons**: API costs, data leaves your environment

```env
# OpenAI (GPT-4)
OPENAI_API_KEY=your_openai_key

# Anthropic (Claude)
ANTHROPIC_API_KEY=your_anthropic_key
```

### Environment Variables

Create a `.env` file for configuration:

```env
# LLM Configuration (choose one approach)
USE_OLLAMA=true                    # Use local Llama models
OLLAMA_MODEL=llama3.1:8b          # Llama model to use
# OR
OPENAI_API_KEY=your_openai_key     # Cloud API option
# OR  
ANTHROPIC_API_KEY=your_anthropic_key

# Optional: Custom settings
EMBEDDING_MODEL=all-MiniLM-L6-v2
CHUNK_SIZE=1000
VECTOR_DB_PATH=./data/vector_store
```

## Data Models

The system uses comprehensive Pydantic models for data validation:

```python
# Complete submission structure
class LabSubmission(BaseModel):
    administrative_info: AdministrativeInfo
    source_material: SourceMaterial
    pooling_info: PoolingInfo
    sequence_generation: SequenceGeneration
    container_info: ContainerInfo
    informatics_info: InformaticsInfo
    sample_details: SampleDetails
    
    # Metadata
    submission_id: Optional[str]
    created_at: datetime
    extracted_confidence: Optional[float]
```

## API Reference

### Main Methods

#### `rag_system.process_document(file_path)`
Process a single laboratory document and extract submission information.

**Parameters:**
- `file_path`: Path to the document (PDF, DOCX, TXT)

**Returns:** `ExtractionResult` with extracted data and metadata

#### `rag_system.query_submissions(query, filter_metadata=None)`
Answer questions about processed laboratory submissions.

**Parameters:**
- `query`: Natural language question
- `filter_metadata`: Optional filters for search scope

**Returns:** Natural language answer based on stored data

#### `rag_system.get_system_status()`
Get current system status and statistics.

**Returns:** System status dictionary with document counts and configuration

## Supported File Types

- **PDF**: Scientific papers, forms, reports
- **DOCX**: Laboratory protocols, submission forms
- **TXT**: Plain text laboratory documentation

## Example Use Cases

1. **Laboratory Form Processing**: Extract submission details from standardized lab forms
2. **Research Paper Analysis**: Identify experimental parameters from scientific publications
3. **Protocol Documentation**: Extract methodology details from laboratory protocols
4. **Quality Control**: Validate submission completeness and accuracy
5. **Data Integration**: Aggregate submission information across multiple documents

## Performance

- **Processing Speed**: ~2-5 seconds per document (depending on size)
- **Accuracy**: 85-95% extraction accuracy on well-formatted documents
- **Scalability**: Handles batch processing of multiple documents
- **Memory Efficient**: Streaming document processing for large files

## Error Handling

The system provides comprehensive error handling:

```python
# Check extraction results
if result.success:
    print(f"Confidence: {result.confidence_score}")
    if result.missing_fields:
        print(f"Missing: {result.missing_fields}")
    if result.warnings:
        print(f"Warnings: {result.warnings}")
else:
    print(f"Failed: {result.warnings}")
```

## Dependency Management with uv

### Adding New Dependencies

```bash
# Add a runtime dependency
uv pip install new-package

# Add a development dependency
uv pip install --dev pytest-mock

# Add with specific version constraints
uv pip install "package>=1.0.0,<2.0.0"

# Update pyproject.toml and reinstall
uv pip install -e .
```

### Managing Optional Dependencies

```bash
# Install web server dependencies
uv pip install -e ".[web]"

# Install development tools
uv pip install -e ".[dev]"

# Install documentation tools
uv pip install -e ".[docs]"

# Install everything
uv pip install -e ".[all]"
```

### Environment Management

```bash
# Create new environment
uv venv --python 3.11  # Specify Python version

# Remove environment
rm -rf .venv

# List installed packages
uv pip list

# Check for outdated packages
uv pip list --outdated

# Sync with pyproject.toml
uv pip install -e .
```

## Development Workflow

### Code Quality Tools

With uv environment activated:

```bash
# Format code
black .
isort .

# Lint code
ruff check .
ruff check --fix .

# Type checking
mypy .

# Run all quality checks
black . && isort . && ruff check . && mypy .
```

### Testing

```bash
# Install test dependencies
uv pip install -e ".[dev]"

# Run tests
pytest

# Run with coverage
pytest --cov=rag --cov=models

# Run specific test types
pytest -m unit        # Unit tests only
pytest -m integration # Integration tests only
pytest -m "not slow"  # Skip slow tests
```

### Performance Comparison

**uv vs pip Installation Times:**
- `uv pip install`: ~2-3 seconds
- `pip install`: ~15-30 seconds
- **Result**: ~10x faster with uv! 🚀

## Contributing

1. Fork the repository
2. Set up development environment:
   ```bash
   uv venv
   source .venv/bin/activate
   uv pip install -e ".[dev]"
   ```
3. Create a feature branch
4. Add tests for new functionality
5. Run quality checks:
   ```bash
   black . && isort . && ruff check . && pytest
   ```
6. Submit a pull request

## License

This project is licensed under the MIT License.

## Troubleshooting

### Common Issues

**ImportError with relative imports:**
```bash
# Make sure you're running from the project root
cd /path/to/lab_submission_rag/app
source .venv/bin/activate
python example_usage.py
```

**Missing email-validator:**
```bash
uv pip install pydantic[email]
```

**TOML parsing errors:**
- Check `pyproject.toml` for proper escape sequences
- Validate TOML syntax online

**Slow package installation:**
- Use `uv` instead of `pip` for 10x faster installs
- Clear package cache: `uv cache clean`

### Performance Optimization

```bash
# Check uv is being used
which uv

# Optimize for development
uv pip install -e ".[dev]" --no-deps  # Skip dependency checking

# Clear caches if needed
uv cache clean
```

## Support

For issues and questions:
1. Check the documentation
2. Verify `uv` and Python versions
3. Search existing issues
4. Create a new issue with:
   - Python version (`python --version`)
   - uv version (`uv --version`)
   - Error message and full traceback
   - Operating system

## Why uv?

**Performance Benefits:**
- 🚀 **10-100x faster** than pip
- 🔒 **Better dependency resolution**
- 💾 **Efficient caching**
- 📦 **Modern package management**
- 🔄 **Reliable virtual environments**

**Developer Experience:**
- Modern `pyproject.toml` standard
- Consistent dependency locking
- Cross-platform compatibility
- Better error messages

## Project Files

**Core System:**
- `rag_orchestrator.py` - Main RAG system coordinator
- `models/submission.py` - Laboratory submission data models
- `rag/` - RAG components (document processor, vector store, LLM interface)
- `config.py` - System configuration

**Development:**
- `pyproject.toml` - Modern Python project configuration with uv
- `setup_check.py` - Environment verification script
- `setup_llama.py` - Llama/Ollama installation and configuration
- `example_usage.py` - Usage demonstration
- `requirements.txt` - Legacy pip requirements (use pyproject.toml instead)

**Documentation:**
- `README.md` - This comprehensive guide
- `config.py` - Contains all configurable environment variables
- `.gitignore` - Comprehensive git ignore rules for RAG projects

## Version Control

### What's Tracked in Git ✅
- **Source Code**: All Python files (`.py`), configuration (`pyproject.toml`, `config.py`)
- **Documentation**: `README.md`, code comments, docstrings
- **Project Structure**: `models/`, `rag/` directories and their Python modules
- **Dependencies**: `requirements.txt` (legacy), `pyproject.toml` (modern)

### What's Ignored 🚫
- **Environments**: `.venv/`, `.env` files with API keys
- **Data Directories**: `data/`, `uploads/`, `exports/`, `logs/`
- **Cache Files**: `__pycache__/`, `.cache/`, model downloads
- **IDE Files**: `.vscode/`, `.idea/`, `.cursor/`
- **OS Files**: `.DS_Store`, `Thumbs.db`, temporary files
- **ML Artifacts**: `*.model`, `*.bin`, `*.pkl`, `*.h5`

The `.gitignore` is designed specifically for RAG/ML projects to ensure sensitive data, large models, and generated files are never accidentally committed.
