"""
LLM interface for information extraction and generation
"""

import asyncio
import json
import logging

import anthropic
import ollama
import openai
from pydantic import ValidationError

from config import settings
from models.submission import (
    AdministrativeInfo,
    ContainerInfo,
    ExtractionResult,
    InformaticsInfo,
    LabSubmission,
    PoolingInfo,
    SampleDetails,
    SequenceGeneration,
    SourceMaterial,
)

logger = logging.getLogger(__name__)


class LLMInterface:
    """Interface for LLM-based information extraction and query processing"""

    def __init__(self) -> None:

        self.client = None
        self._initialize_client()

    def _initialize_client(self) -> None:
        """Initialize the LLM client based on configuration"""
        try:
            # Check for Ollama first if enabled
            if hasattr(settings, "use_ollama") and settings.use_ollama:
                try:
                    # Test Ollama connection
                    ollama.list()  # This will fail if Ollama is not running
                    self.client = ollama
                    self.client_type = "ollama"
                    logger.info(f"Using Ollama with model: {settings.ollama_model}")
                    return
                except Exception as e:
                    logger.warning(
                        f"Ollama not available: {str(e)}. Falling back to other providers."
                    )

            # Fall back to cloud providers
            if hasattr(settings, "openai_api_key") and settings.openai_api_key:
                openai.api_key = settings.openai_api_key
                self.client_type = "openai"
                logger.info("Using OpenAI API")
            elif hasattr(settings, "anthropic_api_key") and settings.anthropic_api_key:
                self.client = anthropic.Anthropic(api_key=settings.anthropic_api_key)
                self.client_type = "anthropic"
                logger.info("Using Anthropic API")
            else:
                logger.warning("No LLM providers available. Using mock responses.")
                self.client_type = "mock"
        except Exception as e:
            logger.error(f"Failed to initialize LLM client: {str(e)}")
            self.client_type = "mock"

    async def extract_submission_info(
        self, document_chunks: list[tuple[str, float]], source_document: str, custom_prompt: str | None = None
    ) -> ExtractionResult:
        """Extract laboratory submission information from document chunks"""
        try:
            if custom_prompt:
                # Use custom prompt directly
                prompt = custom_prompt
            else:
                # Combine relevant chunks into context
                context = self._prepare_context(document_chunks)
                # Create extraction prompt
                prompt = self._create_extraction_prompt(context)

            # Get LLM response
            response = await self._get_llm_response(prompt)

            # Parse response into structured data
            return await self._parse_extraction_response(response, source_document)

        except Exception as e:
            logger.error(f"Error in submission extraction: {str(e)}")
            return ExtractionResult(
                success=False,
                confidence_score=0.0,
                missing_fields=[],
                warnings=[f"Extraction failed: {str(e)}"],
                processing_time=0.0,
                source_document=source_document,
            )

    def _prepare_context(self, document_chunks: list[tuple[str, float]]) -> str:
        """Prepare context from document chunks"""
        context_parts = []
        for chunk_content, similarity_score in document_chunks:
            context_parts.append(f"[Relevance: {similarity_score:.2f}]\n{chunk_content}\n")

        return "\n".join(context_parts)

    def _create_extraction_prompt(self, context: str) -> str:
        """Create prompt for extracting laboratory submission information"""
        return f"""
You are a specialized AI assistant for extracting laboratory submission information from scientific documents.
Your task is to extract information for the following 7 categories:

1. Administrative Information:
   - Submitter First Name (required)
   - Submitter Last Name (required)  
   - Submitter Email (required)
   - Submitter Phone
   - Assigned Project

2. Source and Submitting Material:
   - Material type (genomic DNA, RNA, other)
   - Collection details
   - Storage conditions

3. Pooling (Multiplexing):
   - Pooling strategy
   - Sample pooling details
   - Barcode information

4. Sequence Generation:
   - Sequencing platform
   - Read parameters
   - Library preparation

5. Container and Diluent:
   - Container specifications
   - Volume and concentration
   - Storage conditions

6. Informatics:
   - Analysis type
   - Reference genome
   - Pipeline requirements

7. Sample Details:
   - Sample identifiers
   - Quality metrics
   - Priority level

Document Context:
{context}

Extract the available information and return it in JSON format matching the laboratory submission data model.
Include confidence scores for each extracted field and note any missing required fields.
If information is not available, use null values.

Response format:
{{
  "administrative_info": {{
    "submitter_first_name": "...",
    "submitter_last_name": "...",
    "submitter_email": "...",
    "submitter_phone": "...",
    "assigned_project": "..."
  }},
  "source_material": {{
    "source_type": "...",
    "collection_date": "...",
    "preservation_method": "..."
  }},
  "pooling_info": {{
    "is_pooled": false,
    "pooling_ratio": {{}}
  }},
  "sequence_generation": {{
    "sequencing_platform": "...",
    "read_length": null,
    "target_coverage": null
  }},
  "container_info": {{
    "container_type": "...",
    "volume": null,
    "concentration": null
  }},
  "informatics_info": {{
    "analysis_type": "...",
    "reference_genome": "..."
  }},
  "sample_details": {{
    "sample_id": "...",
    "priority": "medium",
    "quality_score": null
  }},
  "confidence_score": 0.85,
  "missing_fields": ["field1", "field2"],
  "warnings": ["warning1", "warning2"]
}}
"""

    async def _get_llm_response(self, prompt: str) -> str:
        """Get response from LLM"""
        try:
            if self.client_type == "ollama":
                # Use Ollama for local Llama models
                response = await asyncio.to_thread(
                    ollama.generate,
                    model=settings.ollama_model,
                    prompt=f"You are a specialized laboratory document processing assistant.\n\n{prompt}",
                    options={
                        "temperature": settings.llm_temperature,
                        "num_predict": settings.max_tokens,
                    },
                )
                return response["response"]

            elif self.client_type == "openai":
                response = await openai.ChatCompletion.acreate(
                    model="gpt-4",
                    messages=[
                        {
                            "role": "system",
                            "content": "You are a specialized laboratory document processing assistant.",
                        },
                        {"role": "user", "content": prompt},
                    ],
                    temperature=settings.llm_temperature,
                    max_tokens=settings.max_tokens,
                )
                return response.choices[0].message.content

            elif self.client_type == "anthropic":
                response = await self.client.messages.create(
                    model="claude-3-sonnet-20240229",
                    max_tokens=settings.max_tokens,
                    temperature=settings.llm_temperature,
                    messages=[{"role": "user", "content": prompt}],
                )
                return response.content[0].text

            else:
                # Mock response for testing
                return self._mock_extraction_response()

        except Exception as e:
            logger.error(f"Error getting LLM response: {str(e)}")
            return self._mock_extraction_response()

    def _mock_extraction_response(self) -> str:
        """Mock response for testing purposes"""
        return json.dumps(
            {
                "administrative_info": {
                    "submitter_first_name": "John",
                    "submitter_last_name": "Doe",
                    "submitter_email": "john.doe@example.com",
                    "submitter_phone": "555-0123",
                    "assigned_project": "PROJ-2024-001",
                },
                "source_material": {
                    "source_type": "dna",
                    "collection_date": None,
                    "preservation_method": "frozen",
                },
                "pooling_info": {"is_pooled": False, "pooling_ratio": {}},
                "sequence_generation": {
                    "sequencing_platform": "illumina",
                    "read_length": 150,
                    "target_coverage": 30.0,
                },
                "container_info": {"container_type": "tube", "volume": 50.0, "concentration": 25.0},
                "informatics_info": {"analysis_type": "wgs", "reference_genome": "hg38"},
                "sample_details": {
                    "sample_id": "SAMPLE-001",
                    "priority": "medium",
                    "quality_score": 8.5,
                },
                "confidence_score": 0.85,
                "missing_fields": ["submitter_phone"],
                "warnings": ["Some fields extracted with low confidence"],
            }
        )

    async def _parse_extraction_response(
        self, response: str, source_document: str
    ) -> ExtractionResult:
        """Parse LLM response into structured extraction result"""
        try:
            # Parse JSON response
            data = json.loads(response)

            # Extract confidence and metadata
            confidence_score = data.get("confidence_score", 0.0)
            missing_fields = data.get("missing_fields", [])
            warnings = data.get("warnings", [])

            # Create submission object
            submission = LabSubmission(
                administrative_info=AdministrativeInfo(**data["administrative_info"]),
                source_material=SourceMaterial(**data["source_material"]),
                pooling_info=PoolingInfo(**data["pooling_info"]),
                sequence_generation=SequenceGeneration(**data["sequence_generation"]),
                container_info=ContainerInfo(**data["container_info"]),
                informatics_info=InformaticsInfo(**data["informatics_info"]),
                sample_details=SampleDetails(**data["sample_details"]),
                extracted_confidence=confidence_score,
            )

            return ExtractionResult(
                success=True,
                submission=submission,
                confidence_score=confidence_score,
                missing_fields=missing_fields,
                warnings=warnings,
                processing_time=0.0,
                source_document=source_document,
            )

        except (json.JSONDecodeError, ValidationError, KeyError) as e:
            logger.error(f"Error parsing extraction response: {str(e)}")
            return ExtractionResult(
                success=False,
                confidence_score=0.0,
                missing_fields=[],
                warnings=[f"Failed to parse response: {str(e)}"],
                processing_time=0.0,
                source_document=source_document,
            )

    async def answer_query(
        self,
        query: str,
        relevant_chunks: list[tuple[str, float]],
        submission_data: LabSubmission | None = None,
    ) -> str:
        """Answer questions about laboratory submissions using RAG"""
        try:
            # Prepare context from chunks and submission data
            context_parts = []

            # Add document chunks
            for chunk_content, similarity_score in relevant_chunks:
                context_parts.append(
                    f"Document excerpt (relevance: {similarity_score:.2f}):\n{chunk_content}"
                )

            # Add structured submission data if available
            if submission_data:
                context_parts.append(
                    f"Structured submission data:\n{submission_data.json(indent=2)}"
                )

            context = "\n\n".join(context_parts)

            # Create query prompt
            prompt = f"""
You are a helpful assistant specialized in laboratory submissions and scientific document analysis.
Use the following context to answer the user's question accurately and comprehensively.

Context:
{context}

Question: {query}

Please provide a detailed, accurate answer based on the context. If the information is not available in the context, clearly state that.
"""

            response = await self._get_llm_response(prompt)
            return response

        except Exception as e:
            logger.error(f"Error in query processing: {str(e)}")
            return f"I apologize, but I encountered an error while processing your query: {str(e)}"

    async def extract_submission_info_with_prompt(self, prompt: str) -> dict:
        """Extract submission information using a custom prompt"""
        try:
            response = await self._get_llm_response(prompt)
            # Try to parse as JSON, otherwise return as dict with content
            try:
                return json.loads(response)
            except json.JSONDecodeError:
                return {"content": response, "error": "Failed to parse JSON"}
        except Exception as e:
            logger.error(f"Error in extraction with prompt: {str(e)}")
            return {"error": str(e)}
