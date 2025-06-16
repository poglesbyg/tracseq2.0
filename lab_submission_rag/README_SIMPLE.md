# Ultra-Lightweight Laboratory Submission RAG System 🧬🦙

A **ultra-lightweight, easy-to-run** version powered by **Ollama (local LLM)** that focuses on core functionality without complex dependencies or API costs.

## Why This Ultra-Lightweight Version?

✅ **8 dependencies** instead of 49+ (removed OpenAI requirement)  
✅ **Local LLM** - runs entirely on your machine with Ollama  
✅ **No API costs** - completely free to run  
✅ **Privacy-first** - your data never leaves your computer  
✅ **No database** - uses simple file storage  
✅ **No web framework** - focuses on core RAG functionality  
✅ **Single file** - easy to understand and modify  
✅ **2-minute setup** - get running instantly  

## Quick Start

### 1. One-Command Setup (Recommended)

```bash
# This installs everything including Ollama and the LLM model
python setup_simple.py --ollama
```

That's it! This will:
- Install all Python dependencies
- Download and install Ollama
- Download the lightweight Llama 3.2 3B model
- Set up the RAG system

### 2. Alternative: Manual Setup

```bash
# Install dependencies first
pip install -r requirements-lite.txt

# Install Ollama manually
# Visit: https://ollama.ai/download
# Then download a model: ollama pull llama3.2:3b

# Run setup to verify
python setup_simple.py
```

### 3. Run the System

```bash
# Run the demo
python simple_lab_rag.py

# Or use it in your own code
python -c "
import simple_lab_rag
rag = simple_lab_rag.SimpleLabRAG()
result = rag.process_document('your_document.pdf')
print(result)
"
```

## Features

### 📄 Document Processing
- **PDF, DOCX, TXT** support
- Simple text extraction
- No complex preprocessing

### 🧠 Local Information Extraction
- **Llama 3.2 3B** for structured extraction (runs locally!)
- Laboratory-specific prompts
- JSON output format
- **OpenAI fallback** available if needed

### 🔍 Query System
- **Natural language queries** about your documents
- Vector similarity search with local embeddings
- Context-aware answers using local LLM
- **No data sent to external servers**

### 💾 Simple Storage
- **File-based storage** (no database needed)
- JSON format for easy inspection
- Automatic backup and export
- **Everything stays on your machine**

### 🦙 Ollama Integration
- **Automatic model downloading**
- **Smart fallback system**
- **Multiple model options** (1B, 3B, 8B)
- **Cross-platform support**

## Usage Examples

### Process a Document

```python
from simple_lab_rag import SimpleLabRAG

# Initialize system
rag = SimpleLabRAG()

# Process a document
result = rag.process_document("lab_submission.pdf")

if result.success:
    submission = result.submission
    print(f"Submitter: {submission.administrative.submitter_name}")
    print(f"Sample Type: {submission.sample.sample_type}")
    print(f"Platform: {submission.sequencing.platform}")
else:
    print(f"Error: {result.error}")
```

### Query the System

```python
# Ask questions about processed documents
questions = [
    "Who is the submitter?",
    "What sequencing platform is being used?",
    "What type of sample is this?",
    "What is the sample concentration?"
]

for question in questions:
    answer = rag.query(question)
    print(f"Q: {question}")
    print(f"A: {answer}\n")
```

### Export Data

```python
# Export all processed submissions
export_file = rag.export_submissions(format="json")
print(f"Data exported to: {export_file}")

# Get system statistics
stats = rag.get_stats()
print(f"Total submissions: {stats['total_submissions']}")
```

## Data Models

The system extracts information into three main categories:

```python
class LabSubmission:
    administrative: AdministrativeInfo  # Name, email, phone, project
    sample: SampleInfo                  # ID, type, concentration, volume
    sequencing: SequencingInfo          # Platform, analysis type, coverage
```

## File Structure

```
your_project/
├── simple_lab_rag.py          # Main system (single file)
├── requirements-lite.txt       # Minimal dependencies
├── setup_simple.py            # Setup script
├── .env                       # API key configuration
├── data/                      # System data
│   ├── submissions.json       # Processed submissions
│   ├── vector_store/          # Vector database
│   └── exports/               # Exported data
├── demo/                      # Demo documents
└── uploads/                   # Your documents
```

## Dependencies

Only **8 essential packages** (reduced from 49+):

```
ollama>=0.1.7                  # Local LLM API
sentence-transformers>=2.2.0   # Embeddings (local)
chromadb>=0.4.0                # Vector database (local)
numpy>=1.24.0                  # Numerical operations
pypdf>=4.0.0                   # PDF processing
python-docx>=0.8.11           # Word documents
pydantic>=2.5.0                # Data validation
python-dotenv>=1.0.0           # Environment variables

# Optional fallback (not required):
# openai>=1.10.0               # Only if you want OpenAI fallback
```

**Local Models Used:**
- **Llama 3.2 3B** (~2GB) - Main model (lightweight & capable)
- **all-MiniLM-L6-v2** (~80MB) - Embeddings model
- **Total download**: ~2.1GB (one-time download)

## Troubleshooting

### Common Issues

**1. ImportError: Missing dependency**
```bash
pip install -r requirements-lite.txt
```

**2. Ollama Not Running**
```bash
# Check if Ollama is installed
ollama --version

# Start Ollama service
ollama serve

# Download the model
ollama pull llama3.2:3b
```

**3. Model Download Issues**
```bash
# Try a smaller model first
ollama pull llama3.2:1b

# Or use OpenAI fallback
echo "OPENAI_API_KEY=your_key_here" > .env
```

**4. Document Processing Failed**
- Check file format (PDF, DOCX, TXT only)
- Ensure file is not corrupted
- Try with a simple text file first

**5. No Query Results**
- Process some documents first
- Check if documents contain relevant information
- Try simpler queries

### Getting Help

1. **Run the setup script**: `python setup_simple.py --ollama`
2. **Test installation**: `python test_simple.py`
3. **Check the demo**: Look at the generated demo document
4. **Verify Ollama**: `ollama list` should show llama3.2:3b

## Comparison with Full Version

| Feature | Ultra-Light Version | Full Version |
|---------|-------------------|--------------|
| Dependencies | 8 packages | 49+ packages |
| Database | File-based | PostgreSQL |
| Web Interface | No | FastAPI |
| Architecture | Single file | Service layers |
| Setup Time | **< 2 minutes** | 20+ minutes |
| LLM Providers | **Ollama (local)** + OpenAI fallback | OpenAI, Anthropic, Ollama |
| API Costs | **$0 (local)** | Variable |
| Privacy | **100% local** | Depends on LLM choice |
| Internet Required | **No (after setup)** | Yes (for cloud LLMs) |
| Deployment | **Single command** | Docker, complex |
| Resource Usage | **~2GB storage** | 5GB+ |

## When to Use Which Version

**Use Ultra-Lightweight Version when:**
- ✅ You want **instant setup** (< 2 minutes)
- ✅ You prefer **no API costs** (100% free)
- ✅ You need **privacy** (everything local)
- ✅ You have **basic-medium document processing** needs
- ✅ You're **prototyping or learning**
- ✅ You want **simplicity over complexity**
- ✅ You work **offline frequently**

**Use Full Version when:**
- ⚙️ You need **web interface**
- ⚙️ You want **cloud LLM options**
- ⚙️ You need **database integration**
- ⚙️ You're building **production systems**
- ⚙️ You need **enterprise features**

## License

Same as the full version - check the main LICENSE file.

---

*This lightweight version focuses on core functionality while maintaining the power of the Laboratory Submission RAG System.* 
