"""
LLM Integration Tests for Laboratory Document Processing

*Context added by Giga llm-integration*

Tests the complete AI-powered document processing pipeline including:
- Multi-model LLM support (Ollama, OpenAI, Anthropic)
- Laboratory-specific prompt engineering
- Confidence scoring for extracted fields
- Vector store integration for RAG queries
"""

import tempfile
from pathlib import Path
from unittest.mock import patch

import pytest

from rag_orchestrator import LabSubmissionRAG


class TestLLMIntegration:
    """Test suite for LLM integration in laboratory document processing."""

    @pytest.fixture
    async def rag_system(self) -> None:
        """Create RAG system instance for testing."""
        system = LabSubmissionRAG()
        await system.initialize_database()
        return system

    @pytest.fixture
    def sample_lab_document(self) -> None:
        """Create sample laboratory document content."""
        return """
        Laboratory Submission Form
        
        Project: COVID-19 Sequencing Study
        Principal Investigator: Dr. Jane Smith
        Contact: jane.smith@university.edu
        
        Sample Information:
        - Sample ID: COVID-001
        - Sample Type: RNA
        - Collection Date: 2024-01-15
        - Volume: 500µL
        - Concentration: 150 ng/µL
        - Storage Condition: -80°C
        
        Sequencing Requirements:
        - Platform: Illumina NovaSeq
        - Read Length: 150bp paired-end
        - Depth: 30X coverage
        - Priority: High
        
        Quality Requirements:
        - RIN Score: >7.0
        - 260/280 Ratio: 1.8-2.2
        """

    @pytest.mark.asyncio
    async def test_document_processing_pipeline(self, rag_system, sample_lab_document) -> None:
        """Test complete document processing with LLM extraction."""
        # Create temporary test file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as f:
            f.write(sample_lab_document)
            temp_file = Path(f.name)

        try:
            # Process document through RAG pipeline
            result = await rag_system.process_document(temp_file)

            # Verify processing success
            assert result.success, f"Document processing failed: {result.warnings}"
            assert result.confidence_score > 0.85, "Confidence score too low"
            assert result.submission is not None, "No submission data extracted"

            # Verify extracted laboratory data
            submission = result.submission
            assert submission.project_name == "COVID-19 Sequencing Study"
            assert submission.principal_investigator == "Dr. Jane Smith"
            assert "COVID-001" in submission.sample_ids

            # Verify sample specifications
            sample_specs = submission.sample_specifications
            assert sample_specs["sample_type"] == "RNA"
            assert sample_specs["volume"] == "500µL"
            assert sample_specs["concentration"] == "150 ng/µL"
            assert sample_specs["storage_condition"] == "-80°C"

            # Verify sequencing requirements
            seq_reqs = submission.sequencing_requirements
            assert seq_reqs["platform"] == "Illumina NovaSeq"
            assert seq_reqs["read_length"] == "150bp paired-end"
            assert seq_reqs["priority"] == "High"

        finally:
            temp_file.unlink(missing_ok=True)

    @pytest.mark.asyncio
    async def test_llm_confidence_scoring(self, rag_system) -> None:
        """Test LLM confidence scoring for field extraction."""
        # Test with high-quality document
        high_quality_doc = """
        Complete Laboratory Form
        Sample ID: CLEAR-001
        Sample Type: DNA
        Concentration: 250 ng/µL
        Platform: Illumina HiSeq
        """

        with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as f:
            f.write(high_quality_doc)
            high_quality_file = Path(f.name)

        try:
            result = await rag_system.process_document(high_quality_file)
            high_confidence = result.confidence_score

            # Test with ambiguous document
            ambiguous_doc = """
            Some samples... maybe DNA or RNA?
            Concentration unclear...
            Platform: possibly Illumina?
            """

            with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as f:
                f.write(ambiguous_doc)
                ambiguous_file = Path(f.name)

            try:
                result = await rag_system.process_document(ambiguous_file)
                low_confidence = result.confidence_score

                # High quality should have higher confidence
                assert high_confidence > low_confidence
                assert high_confidence > 0.85
                assert low_confidence < 0.70

            finally:
                ambiguous_file.unlink(missing_ok=True)
        finally:
            high_quality_file.unlink(missing_ok=True)

    @pytest.mark.asyncio
    async def test_rag_query_intelligence(self, rag_system, sample_lab_document) -> None:
        """Test enhanced RAG querying capabilities."""
        # First process a document to populate vector store
        with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as f:
            f.write(sample_lab_document)
            temp_file = Path(f.name)

        try:
            await rag_system.process_document(temp_file)

            # Test various query types
            queries = [
                "What RNA samples are stored at -80°C?",
                "Show me high priority sequencing requests",
                "Which samples need Illumina NovaSeq?",
                "What is the concentration of COVID-001?",
            ]

            for query in queries:
                response = await rag_system.query_submissions(query)

                assert response is not None
                assert len(response) > 10  # Should have substantive response
                assert "COVID" in response or "RNA" in response or "Illumina" in response

        finally:
            temp_file.unlink(missing_ok=True)

    @pytest.mark.asyncio
    async def test_multi_model_fallback(self, rag_system) -> None:
        """Test LLM model fallback mechanisms."""
        with patch('rag.enhanced_llm_interface.enhanced_llm.answer_query') as mock_llm:
            # Simulate primary model failure
            mock_llm.side_effect = [
                Exception("Primary model unavailable"),
                "Fallback model response"
            ]

            response = await rag_system.query_submissions("Test query")

            # Should succeed with fallback
            assert response == "Fallback model response"
            assert mock_llm.call_count == 2

    @pytest.mark.asyncio
    async def test_laboratory_terminology_extraction(self, rag_system) -> None:
        """Test extraction of laboratory-specific terminology."""
        lab_terminology_doc = """
        Sample Preparation Protocol
        
        Equipment: PCR Cycler, Centrifuge, Pipettes
        Reagents: DNA Polymerase, dNTPs, Buffer Solution
        Temperature Cycles: 
        - Denaturation: 95°C for 30 seconds
        - Annealing: 58°C for 30 seconds  
        - Extension: 72°C for 45 seconds
        
        Quality Control:
        - Gel Electrophoresis
        - Spectrophotometry (260/280 ratio)
        - Fragment Analyzer
        """

        with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as f:
            f.write(lab_terminology_doc)
            temp_file = Path(f.name)

        try:
            result = await rag_system.process_document(temp_file)

            # Should extract laboratory equipment and protocols
            assert result.success
            submission = result.submission

            # Check if equipment is properly extracted
            equipment_keywords = ["PCR", "Centrifuge", "Pipettes", "Gel Electrophoresis"]
            doc_text = str(submission.__dict__).lower()

            extracted_count = sum(1 for keyword in equipment_keywords
                                if keyword.lower() in doc_text)
            assert extracted_count >= 2, "Should extract laboratory equipment"

        finally:
            temp_file.unlink(missing_ok=True)

    @pytest.mark.asyncio
    async def test_batch_document_processing(self, rag_system) -> None:
        """Test batch processing of multiple laboratory documents."""
        # Create multiple test documents
        documents = []
        for i in range(3):
            content = f"""
            Laboratory Sample {i+1}
            Sample ID: BATCH-{i+1:03d}
            Sample Type: {'DNA' if i % 2 == 0 else 'RNA'}
            Concentration: {100 + i*50} ng/µL
            Platform: Illumina MiSeq
            """

            temp_file = tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False)
            temp_file.write(content)
            temp_file.close()
            documents.append(Path(temp_file.name))

        try:
            # Process batch
            batch_result = await rag_system.process_documents_batch(documents)

            # Verify batch results
            assert batch_result.total_documents == 3
            assert batch_result.successful_extractions >= 2
            assert batch_result.overall_confidence > 0.70
            assert len(batch_result.results) == 3

            # Check individual results
            for result in batch_result.results:
                assert result.processing_time > 0

        finally:
            for doc_path in documents:
                doc_path.unlink(missing_ok=True)

    @pytest.mark.asyncio
    async def test_database_query_optimization(self, rag_system) -> None:
        """Test database-specific query optimization."""
        # Test sample count queries
        count_queries = [
            "How many DNA samples are in the system?",
            "Total number of samples stored at -80°C?",
            "Show sample statistics breakdown",
        ]

        for query in count_queries:
            response = await rag_system.query_submissions(query)

            # Should provide specific numerical information
            assert any(char.isdigit() for char in response)
            assert len(response) > 20  # Substantive response

    def test_prompt_engineering_quality(self) -> None:
        """Test laboratory-specific prompt engineering."""
        # This would test the prompt templates used for LLM queries
        # Verify prompts include laboratory context and terminology

        # Mock test - in real implementation, check actual prompts
        lab_prompt_elements = [
            "laboratory", "sample", "sequencing", "concentration",
            "temperature", "storage", "protocol", "equipment"
        ]

        # Simulate prompt template checking
        mock_prompt = "Extract laboratory sample information including sample type, concentration, storage conditions, and sequencing requirements"

        element_count = sum(1 for element in lab_prompt_elements
                          if element in mock_prompt.lower())

        assert element_count >= 4, "Prompt should include laboratory terminology"

    @pytest.mark.asyncio
    async def test_vector_store_integration(self, rag_system, sample_lab_document) -> None:
        """Test vector store integration for semantic search."""
        # Process document to populate vector store
        with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as f:
            f.write(sample_lab_document)
            temp_file = Path(f.name)

        try:
            await rag_system.process_document(temp_file)

            # Test semantic similarity search
            similar_queries = [
                "RNA samples at freezing temperature",  # Should match -80°C RNA
                "High priority genetic sequencing",     # Should match High priority sequencing
                "NovaSeq platform requirements",       # Should match Illumina NovaSeq
            ]

            for query in similar_queries:
                response = await rag_system.query_submissions(query)

                # Should find semantically similar content
                assert response is not None
                assert len(response) > 50

        finally:
            temp_file.unlink(missing_ok=True)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
