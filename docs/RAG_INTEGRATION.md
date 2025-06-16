# RAG LLM Integration Guide

## Overview

The Laboratory Management System now integrates with a sophisticated RAG (Retrieval-Augmented Generation) LLM system for intelligent document processing and data extraction. This integration enables automatic extraction of laboratory submission information from documents like PDFs and DOCX files, significantly reducing manual data entry and improving accuracy.

## Architecture

### Components

1. **RAG LLM System** (`/home/paul/Dev/lab_submission_rag/`)
   - Python-based document processing system
   - Extracts structured data from laboratory documents
   - Provides query interface for submission information
   - Runs on port 8000 by default

2. **Lab Manager Integration Service** (`src/services/rag_integration_service.rs`)
   - Rust service that bridges the lab manager and RAG system
   - Handles HTTP communication with the RAG system
   - Converts RAG results to lab manager data models
   - Provides health monitoring and error handling

3. **Enhanced Sample Handlers** (`src/handlers/samples/rag_enhanced_handlers.rs`)
   - New API endpoints for document processing
   - Intelligent sample creation from extracted data
   - Preview and validation workflows

## Features

### 🤖 AI-Powered Document Processing

**Automatic Data Extraction**:
- Processes PDF, DOCX, and TXT laboratory documents
- Extracts information across 7 categories:
  1. Administrative Information
  2. Source and Submitting Material
  3. Pooling (Multiplexing)
  4. Sequence Generation
  5. Container and Diluent
  6. Informatics
  7. Sample Details

**Confidence Scoring**:
- Provides confidence scores for extracted data
- Configurable confidence thresholds
- Validation warnings for low-confidence extractions

### 🔍 Intelligent Query System

**Natural Language Queries**:
```rust
// Query the RAG system about submissions
let answer = rag_service.query_submissions(
    "What sequencing platform is being used for project PROJ-2024-001?"
).await?;
```

**Example Queries**:
- "Who is the submitter for this project?"
- "What type of analysis is requested?"
- "What are the storage requirements?"
- "What is the sample priority level?"

### 📋 Enhanced Sample Workflow

**Document-to-Sample Pipeline**:
1. Upload laboratory document
2. RAG system extracts structured data
3. Convert to lab manager sample format
4. Validate and create samples
5. Store with rich metadata

## API Endpoints

### Document Processing

#### Process Document and Create Samples
```http
POST /api/samples/rag/process-document
Content-Type: multipart/form-data

Form Data:
- document: [laboratory document file]
- confidence_threshold: 0.7 (optional)
```

**Response**:
```json
{
  "samples": [
    {
      "name": "PROJ-2024-WGS-001-dna-1",
      "barcode": "DNA-240115123456-ABC",
      "location": "Storage-minus80",
      "metadata": {
        "rag_extraction": {
          "confidence_score": 0.92,
          "administrative_info": {...},
          "source_material": {...}
        }
      }
    }
  ],
  "extraction_result": {
    "success": true,
    "confidence_score": 0.92,
    "missing_fields": [],
    "warnings": []
  },
  "confidence_score": 0.92,
  "validation_warnings": [],
  "processing_time": 2.34
}
```

#### Preview Document Extraction
```http
POST /api/samples/rag/preview
Content-Type: multipart/form-data

Form Data:
- document: [laboratory document file]
```

**Use Case**: Preview extracted data before creating samples.

#### Create Samples from RAG Data
```http
POST /api/samples/rag/create-from-data
Content-Type: application/json

{
  "samples": [...],
  "extraction_result": {...},
  "confidence_score": 0.92,
  "validation_warnings": []
}
```

### Query Interface

#### Query Submission Information
```http
POST /api/samples/rag/query
Content-Type: application/json

{
  "query": "What sequencing platform is being used?"
}
```

**Response**:
```json
{
  "query": "What sequencing platform is being used?",
  "answer": "Based on the submitted documents, the Illumina NovaSeq 6000 platform is being used for whole genome sequencing with paired-end 150bp reads.",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

#### RAG System Status
```http
GET /api/samples/rag/status
```

**Response**:
```json
{
  "status": "operational",
  "vector_store": {
    "total_documents": 5,
    "total_chunks": 127,
    "embedding_model": "all-MiniLM-L6-v2"
  },
  "supported_categories": [
    "Administrative Information",
    "Source and Submitting Material",
    "Pooling (Multiplexing)",
    "Sequence Generation",
    "Container and Diluent",
    "Informatics",
    "Sample Details"
  ]
}
```

## Configuration

### Environment Variables

**Lab Manager** (`.env`):
```env
# RAG Integration
RAG_ENABLED=true
RAG_BASE_URL=http://localhost:8000
RAG_TIMEOUT_SECONDS=300
RAG_MAX_FILE_SIZE_MB=50
RAG_CONFIDENCE_THRESHOLD=0.7
RAG_AUTO_CREATE_SAMPLES=false
```

**RAG System** (`/home/paul/Dev/lab_submission_rag/app/.env`):
```env
# LLM API Keys (choose one)
OPENAI_API_KEY=your_openai_key
ANTHROPIC_API_KEY=your_anthropic_key

# Server Configuration
HOST=0.0.0.0
PORT=8000
DEBUG=false

# Processing Configuration
CHUNK_SIZE=1000
CHUNK_OVERLAP=200
SIMILARITY_THRESHOLD=0.7
```

### Rust Configuration

```rust
use crate::services::rag_integration_service::{RagConfig, RagIntegrationService};

let rag_config = RagConfig {
    base_url: "http://localhost:8000".to_string(),
    timeout_seconds: 300,
    max_file_size_mb: 50,
    supported_formats: vec!["pdf".to_string(), "docx".to_string()],
};

let rag_service = RagIntegrationService::new(rag_config);
```

## Usage Examples

### Basic Document Processing

```rust
use lab_manager::services::rag_integration_service::RagIntegrationService;

// Process a laboratory document
let extraction_result = rag_service
    .process_document("lab_submission_form.pdf")
    .await?;

if extraction_result.success {
    println!("Confidence: {:.2}", extraction_result.confidence_score);
    
    // Convert to lab manager samples
    let samples = rag_service.convert_to_samples(&extraction_result)?;
    
    // Create samples in the system
    for sample in samples {
        let created_sample = sample_manager.create_sample(sample).await?;
        println!("Created sample: {} ({})", created_sample.name, created_sample.barcode);
    }
}
```

### Query Processing

```rust
// Query for specific information
let answer = rag_service
    .query_submissions("What is the target sequencing coverage?")
    .await?;

println!("Answer: {}", answer);
```

### Integration with Existing Workflow

```rust
// Enhanced sample creation with RAG
pub async fn create_enhanced_sample(
    document_path: Option<String>,
    manual_data: Option<CreateSample>,
    confidence_threshold: f64,
) -> Result<Vec<Sample>, Error> {
    
    if let Some(doc_path) = document_path {
        // Use RAG for extraction
        let extraction = rag_service.process_document(&doc_path).await?;
        
        if extraction.confidence_score >= confidence_threshold {
            let samples = rag_service.convert_to_samples(&extraction)?;
            create_samples_batch(samples).await
        } else {
            // Fall back to manual entry with RAG suggestions
            combine_rag_and_manual_data(extraction, manual_data).await
        }
    } else {
        // Standard manual entry
        create_sample_manual(manual_data.unwrap()).await
    }
}
```

## Data Models

### RAG Extraction Result

```rust
pub struct RagExtractionResult {
    pub success: bool,
    pub submission: Option<RagSubmission>,
    pub confidence_score: f64,
    pub missing_fields: Vec<String>,
    pub warnings: Vec<String>,
    pub processing_time: f64,
    pub source_document: String,
}
```

### Enhanced Sample Request

```rust
pub struct RagEnhancedSampleRequest {
    pub document_path: Option<String>,
    pub manual_data: Option<CreateSample>,
    pub use_rag_extraction: bool,
    pub confidence_threshold: Option<f64>,
}
```

## Barcode Generation

The integration includes intelligent barcode generation based on sample type:

```rust
fn generate_barcode(source_type: &str) -> String {
    let prefix = match source_type.to_lowercase().as_str() {
        "blood" => "BLD",
        "saliva" => "SAL", 
        "tissue" => "TSU",
        "dna" => "DNA",
        "rna" => "RNA",
        _ => "UNK",
    };
    
    let timestamp = chrono::Utc::now().format("%y%m%d%H%M");
    let random_suffix = generate_random_suffix();
    
    format!("{}-{}-{}", prefix, timestamp, random_suffix)
}
```

**Examples**:
- `DNA-240115123456-ABC` (DNA sample)
- `BLD-240115123500-XYZ` (Blood sample)
- `TSU-240115123600-DEF` (Tissue sample)

## Error Handling

### Common Error Scenarios

1. **RAG System Unavailable**:
   ```rust
   match rag_service.process_document(path).await {
       Err(ApiError::ServiceUnavailable(_)) => {
           // Fall back to manual entry
           handle_manual_submission().await
       }
       Ok(result) => process_rag_result(result).await,
       Err(e) => return Err(e),
   }
   ```

2. **Low Confidence Extraction**:
   ```rust
   if extraction_result.confidence_score < confidence_threshold {
       // Require manual review
       return RequiresManualReview {
           extraction: extraction_result,
           suggested_edits: generate_suggestions(),
       };
   }
   ```

3. **File Format Issues**:
   ```rust
   if !supported_formats.contains(&file_extension) {
       return Err(ApiError::BadRequest(
           format!("Unsupported format: {}", file_extension)
       ));
   }
   ```

## Performance Considerations

### Processing Times
- **PDF Processing**: 2-5 seconds (typical)
- **DOCX Processing**: 1-3 seconds (typical)
- **Query Response**: 0.5-2 seconds (typical)

### Optimization Strategies

1. **Async Processing**: All RAG operations are async
2. **Timeout Management**: Configurable timeouts prevent hanging
3. **File Size Limits**: Prevent resource exhaustion
4. **Caching**: RAG system caches processed documents
5. **Batch Processing**: Support for multiple documents

## Security Considerations

### Data Privacy
- Documents processed locally or in controlled environments
- No data sent to external APIs unless explicitly configured
- Temporary files cleaned up after processing
- Audit logs for all document processing activities

### Access Control
- RAG endpoints require same authentication as other lab manager APIs
- Document upload size limits prevent DoS attacks
- File type validation prevents malicious uploads

## Troubleshooting

### Common Issues

1. **RAG System Connection Failed**
   ```bash
   # Check if RAG system is running
   curl http://localhost:8000/health
   
   # Start RAG system
   cd /home/paul/Dev/lab_submission_rag/app
   python -m uvicorn main:app --host 0.0.0.0 --port 8000
   ```

2. **Low Confidence Scores**
   - Check document quality and format
   - Verify all required information is present
   - Adjust confidence threshold in configuration
   - Review RAG system prompt engineering

3. **File Upload Issues**
   - Verify file size limits
   - Check supported file formats
   - Ensure upload directory permissions
   - Review multipart form encoding

### Debug Mode

Enable debug logging for detailed information:

```rust
// Enable debug logging
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

## Future Enhancements

### Planned Features
- [ ] **Batch Document Processing**: Process multiple documents simultaneously
- [ ] **Template Learning**: Improve extraction based on successful submissions
- [ ] **Custom Field Extraction**: Configure additional laboratory-specific fields
- [ ] **Integration Webhooks**: Real-time notifications for processing events
- [ ] **Advanced Validation**: Cross-reference extracted data with existing samples
- [ ] **Multi-language Support**: Process documents in multiple languages

### Integration Opportunities
- [ ] **LIMS Integration**: Connect with existing laboratory information systems
- [ ] **Workflow Automation**: Trigger downstream processes automatically
- [ ] **Quality Control**: Automated QC checks on extracted data
- [ ] **Reporting**: Enhanced analytics on submission patterns

## Support

For issues related to:
- **RAG System**: Check `/home/paul/Dev/lab_submission_rag/app/README.md`
- **Lab Manager Integration**: Review logs in `src/services/rag_integration_service.rs`
- **API Endpoints**: Test with provided examples in `examples/rag_integration_demo.rs`

---

*This integration represents a significant advancement in laboratory automation, combining the power of modern LLMs with practical laboratory management workflows.* 
