"""
Advanced RAG Algorithm Integration Tests
*Context added by Giga rag-algorithms*

Tests for laboratory-specific RAG algorithms including:
- Document processing pipeline optimization
- Vector indexing and similarity search
- Laboratory terminology extraction
- Cross-reference verification
- Confidence scoring validation
"""

import pytest
import asyncio
import json
from unittest.mock import AsyncMock, patch
from uuid import uuid4
from datetime import datetime, timezone

from lab_submission_rag.rag.document_processor import DocumentProcessor
from lab_submission_rag.rag.vector_store import VectorStore
from lab_submission_rag.rag.llm_interface import LLMInterface
from lab_submission_rag.models.rag_models import (
    ExtractionResult,
    DocumentMetadata,
    VectorQueryResult,
    ConfidenceScore
)


class TestAdvancedRagAlgorithms:
    """Test suite for advanced RAG algorithms and laboratory-specific processing"""

    @pytest.fixture
    async def rag_processor(self):
        """Create RAG processor with test configuration"""
        processor = DocumentProcessor(
            confidence_threshold=0.85,
            extraction_categories=7,
            chunk_size=512,
            chunk_overlap=50
        )
        await processor.initialize()
        return processor

    @pytest.fixture
    async def vector_store(self):
        """Create vector store for testing"""
        store = VectorStore(
            similarity_threshold=0.7,
            index_type="laboratory_documents",
            dimension=768
        )
        await store.initialize()
        return store

    @pytest.fixture
    def laboratory_documents(self):
        """Sample laboratory documents for testing"""
        return [
            {
                "content": """
                Laboratory Submission Form
                
                Sample Information:
                - Sample ID: COVID-001
                - Sample Type: RNA
                - Collection Date: 2024-01-15
                - Storage Temperature: -80°C
                - Sequencing Platform: Illumina NovaSeq
                
                Quality Control:
                - RNA Integrity: 8.5/10
                - Concentration: 250 ng/μL
                - Purity (260/280): 2.1
                
                Processing Requirements:
                - PCR Amplification: 35 cycles
                - Library Prep: TruSeq Stranded
                - Read Length: 150bp paired-end
                """,
                "metadata": {
                    "document_type": "lab_submission",
                    "submitter": "Dr. Smith",
                    "lab_id": "LAB-001",
                    "submission_date": "2024-01-15"
                }
            },
            {
                "content": """
                Equipment Calibration Report
                
                Equipment: Thermal Cycler PCR-2000
                Calibration Date: 2024-01-10
                Next Calibration: 2024-04-10
                
                Temperature Verification:
                - Target: 95°C, Actual: 94.8°C (±0.2°C)
                - Target: 60°C, Actual: 59.9°C (±0.1°C)
                - Target: 72°C, Actual: 72.1°C (±0.1°C)
                
                Performance: PASS
                Certified by: Tech-001
                """,
                "metadata": {
                    "document_type": "calibration_report",
                    "equipment_id": "PCR-2000",
                    "technician": "Tech-001"
                }
            }
        ]

    @pytest.mark.asyncio
    async def test_laboratory_terminology_extraction(self, rag_processor, laboratory_documents):
        """Test extraction of laboratory-specific terminology and concepts"""
        document = laboratory_documents[0]
        
        # Process document with laboratory context
        result = await rag_processor.process_document(
            content=document["content"],
            metadata=document["metadata"],
            extraction_focus="laboratory_terminology"
        )
        
        # Verify extraction categories
        assert result.confidence_score >= 0.85
        assert len(result.extracted_categories) == 7
        
        # Verify laboratory-specific extractions
        extracted_terms = result.extracted_data.get("terminology", [])
        expected_terms = [
            "RNA", "PCR", "Illumina NovaSeq", "TruSeq", 
            "ng/μL", "260/280", "150bp paired-end"
        ]
        
        for term in expected_terms:
            assert any(term in ext_term for ext_term in extracted_terms), \
                f"Expected term '{term}' not found in extractions"

    @pytest.mark.asyncio
    async def test_sample_specification_extraction(self, rag_processor, laboratory_documents):
        """Test extraction of sample specifications and requirements"""
        document = laboratory_documents[0]
        
        result = await rag_processor.process_document(
            content=document["content"],
            metadata=document["metadata"],
            extraction_focus="sample_specifications"
        )
        
        # Verify sample data extraction
        sample_data = result.extracted_data.get("samples", [])
        assert len(sample_data) >= 1
        
        sample = sample_data[0]
        assert sample["sample_id"] == "COVID-001"
        assert sample["sample_type"] == "RNA"
        assert sample["storage_temperature"] == "-80°C"
        assert sample["concentration"] == "250 ng/μL"
        
        # Verify quality metrics
        quality_data = result.extracted_data.get("quality_control", {})
        assert quality_data["rna_integrity"] == "8.5/10"
        assert quality_data["purity_ratio"] == "2.1"

    @pytest.mark.asyncio
    async def test_equipment_parameter_extraction(self, rag_processor, laboratory_documents):
        """Test extraction of equipment parameters and calibration data"""
        document = laboratory_documents[1]
        
        result = await rag_processor.process_document(
            content=document["content"],
            metadata=document["metadata"],
            extraction_focus="equipment_parameters"
        )
        
        # Verify equipment data extraction
        equipment_data = result.extracted_data.get("equipment", {})
        assert equipment_data["name"] == "Thermal Cycler PCR-2000"
        assert equipment_data["calibration_status"] == "PASS"
        
        # Verify temperature calibration data
        temp_data = result.extracted_data.get("temperature_verification", [])
        assert len(temp_data) == 3
        
        # Check specific temperature points
        temp_95 = next(t for t in temp_data if t["target"] == "95°C")
        assert temp_95["actual"] == "94.8°C"
        assert temp_95["tolerance"] == "±0.2°C"

    @pytest.mark.asyncio
    async def test_confidence_scoring_validation(self, rag_processor, laboratory_documents):
        """Test confidence scoring for extraction quality assessment"""
        high_quality_doc = laboratory_documents[0]  # Well-structured document
        
        # Create ambiguous document for comparison
        ambiguous_doc = {
            "content": "Some sample data... maybe DNA or RNA... temperature unknown...",
            "metadata": {"document_type": "unclear"}
        }
        
        # Process high-quality document
        high_quality_result = await rag_processor.process_document(
            content=high_quality_doc["content"],
            metadata=high_quality_doc["metadata"]
        )
        
        # Process ambiguous document
        ambiguous_result = await rag_processor.process_document(
            content=ambiguous_doc["content"],
            metadata=ambiguous_doc["metadata"]
        )
        
        # Verify confidence scoring
        assert high_quality_result.confidence_score >= 0.85
        assert ambiguous_result.confidence_score < 0.85
        
        # Verify extraction completeness correlation
        assert len(high_quality_result.extracted_categories) > len(ambiguous_result.extracted_categories)

    @pytest.mark.asyncio
    async def test_vector_indexing_and_search(self, vector_store, laboratory_documents):
        """Test vector indexing with laboratory-optimized search"""
        # Index laboratory documents
        document_ids = []
        for i, doc in enumerate(laboratory_documents):
            doc_id = str(uuid4())
            document_ids.append(doc_id)
            
            await vector_store.index_document(
                document_id=doc_id,
                content=doc["content"],
                metadata=doc["metadata"]
            )
        
        # Test laboratory-specific queries
        test_queries = [
            "RNA sequencing requirements",
            "PCR thermal cycler calibration",
            "sample storage temperature",
            "quality control metrics"
        ]
        
        for query in test_queries:
            results = await vector_store.search(
                query=query,
                top_k=5,
                filter_metadata={"document_type": "lab_submission"}
            )
            
            assert len(results) > 0
            assert all(r.similarity_score >= 0.7 for r in results)
            
            # Verify laboratory context relevance
            for result in results:
                assert any(term in result.content.lower() for term in 
                          ["rna", "dna", "pcr", "temperature", "sample"])

    @pytest.mark.asyncio
    async def test_multi_model_fallback_mechanism(self, rag_processor):
        """Test LLM fallback mechanism for extraction reliability"""
        test_document = {
            "content": "Complex laboratory protocol with ambiguous terminology...",
            "metadata": {"document_type": "protocol", "complexity": "high"}
        }
        
        # Mock primary model failure
        with patch.object(rag_processor.llm_interface, 'extract_information') as mock_extract:
            mock_extract.side_effect = [
                Exception("Primary model timeout"),  # Primary fails
                ExtractionResult(  # Fallback succeeds
                    confidence_score=0.75,
                    extracted_categories=["protocols", "equipment"],
                    extracted_data={"protocols": ["centrifugation", "pcr_amplification"]},
                    processing_time_ms=3500
                )
            ]
            
            # Process with fallback
            result = await rag_processor.process_document_with_fallback(
                content=test_document["content"],
                metadata=test_document["metadata"]
            )
            
            # Verify fallback was used
            assert result.confidence_score == 0.75
            assert "protocols" in result.extracted_categories
            assert mock_extract.call_count == 2  # Primary + fallback

    @pytest.mark.asyncio
    async def test_batch_document_processing(self, rag_processor, laboratory_documents):
        """Test batch processing of multiple laboratory documents"""
        # Create batch processing request
        batch_size = len(laboratory_documents)
        
        start_time = datetime.now(timezone.utc)
        results = await rag_processor.process_batch(
            documents=laboratory_documents,
            batch_size=batch_size,
            parallel_processing=True
        )
        processing_time = (datetime.now(timezone.utc) - start_time).total_seconds()
        
        # Verify batch processing results
        assert len(results) == batch_size
        assert all(r.confidence_score > 0.0 for r in results)
        
        # Verify processing efficiency
        assert processing_time < 10.0  # Should complete within 10 seconds
        
        # Verify individual document quality
        for result in results:
            assert len(result.extracted_categories) > 0
            assert result.processing_time_ms > 0

    @pytest.mark.asyncio
    async def test_laboratory_workflow_integration(self, rag_processor, vector_store):
        """Test complete RAG workflow for laboratory document processing"""
        # Simulate laboratory submission workflow
        submission_document = {
            "content": """
            Laboratory Submission Request
            
            Project: COVID-19 Variant Analysis
            Samples: 
            - COVID-001: RNA, -80°C storage
            - COVID-002: DNA, -20°C storage
            
            Sequencing Requirements:
            - Platform: Illumina NovaSeq 6000
            - Read Type: 150bp paired-end
            - Depth: 30X coverage
            
            Analysis Pipeline:
            - Quality Control: FastQC
            - Alignment: BWA-MEM
            - Variant Calling: GATK
            """,
            "metadata": {
                "document_type": "lab_submission",
                "project_id": "COVID-VAR-2024",
                "submitter": "Dr. Johnson",
                "priority": "high"
            }
        }
        
        # Step 1: Process document
        extraction_result = await rag_processor.process_document(
            content=submission_document["content"],
            metadata=submission_document["metadata"]
        )
        
        # Step 2: Index for future retrieval
        doc_id = str(uuid4())
        await vector_store.index_document(
            document_id=doc_id,
            content=submission_document["content"],
            metadata=submission_document["metadata"]
        )
        
        # Step 3: Query for similar submissions
        similar_docs = await vector_store.search(
            query="COVID sequencing Illumina NovaSeq",
            top_k=3
        )
        
        # Verify complete workflow
        assert extraction_result.confidence_score >= 0.85
        assert "COVID-001" in str(extraction_result.extracted_data)
        assert "COVID-002" in str(extraction_result.extracted_data)
        assert len(similar_docs) > 0
        
        # Verify laboratory-specific processing
        samples = extraction_result.extracted_data.get("samples", [])
        assert len(samples) == 2
        assert any(s["sample_id"] == "COVID-001" for s in samples)
        assert any(s["sample_id"] == "COVID-002" for s in samples)

    @pytest.mark.asyncio
    async def test_rag_algorithm_performance_benchmarks(self, rag_processor):
        """Test RAG algorithm performance under various conditions"""
        # Create documents of varying complexity
        test_documents = [
            {"content": "Simple RNA sample", "complexity": "low"},
            {"content": "Complex multi-sample submission with detailed protocols...", "complexity": "medium"},
            {"content": "Extremely detailed laboratory protocol with multiple samples, equipment specifications, quality control requirements, and complex sequencing parameters..." * 10, "complexity": "high"}
        ]
        
        performance_results = []
        
        for doc in test_documents:
            start_time = datetime.now(timezone.utc)
            
            result = await rag_processor.process_document(
                content=doc["content"],
                metadata={"complexity": doc["complexity"]}
            )
            
            processing_time = (datetime.now(timezone.utc) - start_time).total_seconds() * 1000
            
            performance_results.append({
                "complexity": doc["complexity"],
                "processing_time_ms": processing_time,
                "confidence_score": result.confidence_score,
                "extracted_categories": len(result.extracted_categories)
            })
        
        # Verify performance scaling
        low_complexity = next(r for r in performance_results if r["complexity"] == "low")
        high_complexity = next(r for r in performance_results if r["complexity"] == "high")
        
        # Processing time should scale reasonably
        assert high_complexity["processing_time_ms"] < low_complexity["processing_time_ms"] * 10
        
        # Confidence should remain stable
        assert all(r["confidence_score"] > 0.5 for r in performance_results)
        
        # Extraction quality should improve with more content
        assert high_complexity["extracted_categories"] >= low_complexity["extracted_categories"] 
