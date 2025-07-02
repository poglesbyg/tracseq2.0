"""
LLM interface for information extraction and generation
"""

import asyncio
import json
import logging
from typing import Any, Dict, List, Optional, Tuple
from enum import Enum
import time
import random

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


class LLMProvider(Enum):
    """Supported LLM providers"""
    OLLAMA = "ollama"
    OPENAI = "openai"
    ANTHROPIC = "anthropic"
    MOCK = "mock"


class LLMException(Exception):
    """Base exception for LLM operations"""
    pass


class LLMConnectionError(LLMException):
    """LLM connection error"""
    pass


class LLMResponseError(LLMException):
    """LLM response parsing error"""
    pass


class LLMInterface:
    """Interface for LLM-based information extraction and query processing"""

    def __init__(self) -> None:
        self.client = None
        self.client_type: LLMProvider = LLMProvider.MOCK
        self._initialize_client()
        self._mock_responses = self._load_mock_responses()

    def _initialize_client(self) -> None:
        """Initialize the LLM client based on configuration"""
        try:
            # Check for Ollama first if enabled
            if hasattr(settings, "use_ollama") and settings.use_ollama:
                try:
                    import ollama
                    # Test Ollama connection
                    ollama.list()  # This will fail if Ollama is not running
                    self.client = ollama
                    self.client_type = LLMProvider.OLLAMA
                    logger.info(f"Using Ollama with model: {settings.ollama_model}")
                    return
                except Exception as e:
                    logger.warning(
                        f"Ollama not available: {str(e)}. Falling back to other providers."
                    )

            # Fall back to cloud providers
            if hasattr(settings, "openai_api_key") and settings.openai_api_key:
                import openai
                openai.api_key = settings.openai_api_key
                self.client_type = LLMProvider.OPENAI
                logger.info("Using OpenAI API")
            elif hasattr(settings, "anthropic_api_key") and settings.anthropic_api_key:
                import anthropic
                self.client = anthropic.Anthropic(api_key=settings.anthropic_api_key)
                self.client_type = LLMProvider.ANTHROPIC
                logger.info("Using Anthropic API")
            else:
                logger.warning("No LLM providers available. Using mock responses.")
                self.client_type = LLMProvider.MOCK
        except Exception as e:
            logger.error(f"Failed to initialize LLM client: {str(e)}")
            self.client_type = LLMProvider.MOCK

    def _load_mock_responses(self) -> Dict[str, Any]:
        """Load configurable mock responses"""
        return {
            "default": {
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
                "missing_fields": [],
                "warnings": []
            }
        }

    async def _retry_with_backoff(self, func, max_attempts: int = 3):
        """Simple retry with exponential backoff"""
        for attempt in range(max_attempts):
            try:
                return await func()
            except (LLMConnectionError, asyncio.TimeoutError) as e:
                if attempt == max_attempts - 1:
                    raise
                wait_time = 2 ** attempt  # Exponential backoff
                logger.warning(f"Attempt {attempt + 1} failed: {e}. Retrying in {wait_time}s...")
                await asyncio.sleep(wait_time)

    async def extract_submission_info(
        self, 
        document_chunks: List[Tuple[str, float]], 
        source_document: str, 
        custom_prompt: Optional[str] = None
    ) -> ExtractionResult:
        """Extract laboratory submission information from document chunks
        
        Args:
            document_chunks: List of (chunk_content, similarity_score) tuples
            source_document: Source document path/identifier
            custom_prompt: Optional custom prompt to use instead of default
            
        Returns:
            ExtractionResult with extracted submission information
        """
        start_time = time.time()
        
        try:
            if custom_prompt:
                # Use custom prompt directly
                prompt = custom_prompt
            else:
                # Combine relevant chunks into context
                context = self._prepare_context(document_chunks)
                # Create extraction prompt
                prompt = self._create_extraction_prompt(context)

            # Get LLM response with retry
            response = await self._retry_with_backoff(
                lambda: self._get_llm_response(prompt)
            )

            # Parse response into structured data
            result = await self._parse_extraction_response(response or "", source_document)
            
            # Update processing time
            result.processing_time = time.time() - start_time
            
            return result

        except LLMException as e:
            logger.error(f"LLM error in submission extraction: {str(e)}")
            return ExtractionResult(
                success=False,
                confidence_score=0.0,
                missing_fields=[],
                warnings=[f"LLM extraction failed: {str(e)}"],
                processing_time=time.time() - start_time,
                source_document=source_document,
            )
        except Exception as e:
            logger.error(f"Unexpected error in submission extraction: {str(e)}")
            return ExtractionResult(
                success=False,
                confidence_score=0.0,
                missing_fields=[],
                warnings=[f"Extraction failed: {str(e)}"],
                processing_time=time.time() - start_time,
                source_document=source_document,
            )

    def _prepare_context(self, document_chunks: List[Tuple[str, float]]) -> str:
        """Prepare context from document chunks"""
        # Sort chunks by relevance score
        sorted_chunks = sorted(document_chunks, key=lambda x: x[1], reverse=True)
        
        context_parts = []
        total_length = 0
        max_context_length = getattr(settings, "max_context_length", 8000)
        
        for chunk_content, similarity_score in sorted_chunks:
            chunk_text = f"[Relevance: {similarity_score:.2f}]\n{chunk_content}\n"
            chunk_length = len(chunk_text)
            
            if total_length + chunk_length > max_context_length:
                break
                
            context_parts.append(chunk_text)
            total_length += chunk_length

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
        """Get response from LLM with timeout and error handling"""
        try:
            timeout = getattr(settings, "llm_timeout", 30)
            
            if self.client_type == LLMProvider.OLLAMA:
                import ollama
                # Use Ollama for local Llama models
                response = await asyncio.wait_for(
                    asyncio.to_thread(
                        ollama.generate,
                        model=settings.ollama_model,
                        prompt=f"You are a specialized laboratory document processing assistant.\n\n{prompt}",
                        options={
                            "temperature": getattr(settings, "llm_temperature", 0.3),
                            "num_predict": getattr(settings, "max_tokens", 2000),
                        },
                    ),
                    timeout=timeout
                )
                return response["response"]

            elif self.client_type == LLMProvider.OPENAI:
                import openai
                # Use the v1.0+ API
                client = openai.AsyncOpenAI(api_key=settings.openai_api_key)
                response = await asyncio.wait_for(
                    client.chat.completions.create(
                        model="gpt-4",
                        messages=[
                            {
                                "role": "system",
                                "content": "You are a specialized laboratory document processing assistant.",
                            },
                            {"role": "user", "content": prompt},
                        ],
                        temperature=getattr(settings, "llm_temperature", 0.3),
                        max_tokens=getattr(settings, "max_tokens", 2000),
                    ),
                    timeout=timeout
                )
                return response.choices[0].message.content or ""

            elif self.client_type == LLMProvider.ANTHROPIC:
                # Anthropic's create method is not async, so we use asyncio.to_thread
                response = await asyncio.wait_for(
                    asyncio.to_thread(
                        self.client.messages.create,
                        model="claude-3-sonnet-20240229",
                        max_tokens=getattr(settings, "max_tokens", 2000),
                        temperature=getattr(settings, "llm_temperature", 0.3),
                        messages=[{"role": "user", "content": prompt}],
                    ),
                    timeout=timeout
                )
                # Handle different content types
                if hasattr(response.content[0], 'text'):
                    return response.content[0].text
                else:
                    return str(response.content[0])

            else:
                # Mock response for testing
                return self._mock_extraction_response()

        except asyncio.TimeoutError:
            raise LLMConnectionError(f"LLM request timed out after {timeout} seconds")
        except Exception as e:
            logger.error(f"Error getting LLM response: {str(e)}")
            raise LLMConnectionError(f"Failed to get LLM response: {str(e)}")

    def _mock_extraction_response(self) -> str:
        """Mock response for testing purposes with configurable responses"""
        mock_data = self._mock_responses.get("default", {}).copy()
        
        # Add some variability to mock responses
        mock_data["confidence_score"] = round(random.uniform(0.7, 0.95), 2)
        
        # Randomly add missing fields
        if random.random() < 0.3:
            mock_data["missing_fields"] = ["submitter_phone"]
            mock_data["warnings"] = ["Some fields extracted with low confidence"]
            
        return json.dumps(mock_data)

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
            
            # Store the extracted data
            extracted_data = {
                "administrative_info": data.get("administrative_info", {}),
                "source_material": data.get("source_material", {}),
                "pooling_info": data.get("pooling_info", {}),
                "sequence_generation": data.get("sequence_generation", {}),
                "container_info": data.get("container_info", {}),
                "informatics_info": data.get("informatics_info", {}),
                "sample_details": data.get("sample_details", {})
            }

            # Note: We're not creating a LabSubmission object here because
            # the model expects different fields than what we extract
            # Instead, we'll return the extracted data directly
            
            return ExtractionResult(
                success=True,
                submission=None,  # Skip LabSubmission creation due to model mismatch
                extracted_data=extracted_data,
                confidence_score=confidence_score,
                missing_fields=missing_fields,
                warnings=warnings,
                processing_time=0.0,  # Will be set by caller
                source_document=source_document,
            )

        except json.JSONDecodeError as e:
            logger.error(f"Failed to parse JSON response: {str(e)}")
            raise LLMResponseError(f"Invalid JSON response: {str(e)}")
        except Exception as e:
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
        relevant_chunks: List[Tuple[str, float]],
        submission_data: Optional[LabSubmission] = None,
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

            response = await self._retry_with_backoff(
                lambda: self._get_llm_response(prompt)
            )
            return response

        except LLMException as e:
            logger.error(f"LLM error in query processing: {str(e)}")
            return f"I apologize, but I encountered an error while processing your query: {str(e)}"
        except Exception as e:
            logger.error(f"Unexpected error in query processing: {str(e)}")
            return f"I apologize, but I encountered an unexpected error: {str(e)}"

    async def extract_submission_info_with_prompt(self, prompt: str) -> Dict[str, Any]:
        """Extract submission information using a custom prompt"""
        try:
            response = await self._retry_with_backoff(
                lambda: self._get_llm_response(prompt)
            )
            # Try to parse as JSON, otherwise return as dict with content
            try:
                return json.loads(response)
            except json.JSONDecodeError:
                return {"content": response, "raw_response": True}
        except LLMException as e:
            logger.error(f"LLM error in extraction with prompt: {str(e)}")
            return {"error": str(e), "error_type": "llm_error"}
        except Exception as e:
            logger.error(f"Unexpected error in extraction with prompt: {str(e)}")
            return {"error": str(e), "error_type": "unexpected_error"}
