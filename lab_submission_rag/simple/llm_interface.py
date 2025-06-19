#!/usr/bin/env python3
"""
Simple LLM Interface
Extracted from simple_lab_rag.py for better modularity
"""

import os
import json
import logging
from typing import Dict, Any, Optional

# Core dependencies
try:
    import ollama
    OLLAMA_AVAILABLE = True
    # Optional OpenAI fallback
    try:
        import openai
        OPENAI_AVAILABLE = True
    except ImportError:
        OPENAI_AVAILABLE = False
except ImportError as e:
    print(f"Missing dependency: {e}")
    print("⚠️ Ollama not available, will use fallback or demo mode")
    OLLAMA_AVAILABLE = False
    # Still try to import OpenAI as fallback
    try:
        import openai
        OPENAI_AVAILABLE = True
        print("✅ OpenAI available as fallback")
    except ImportError:
        OPENAI_AVAILABLE = False
        print("⚠️ Neither Ollama nor OpenAI available, using demo mode")

logger = logging.getLogger(__name__)


class SimpleLLMInterface:
    """Simplified LLM interface using Ollama (local) with OpenAI fallback"""
    
    def __init__(self, 
                 model: str = "llama3.2:3b", 
                 use_openai_fallback: bool = False,
                 openai_api_key: Optional[str] = None):
        
        self.model = model
        self.use_openai_fallback = use_openai_fallback
        
        # Check if ollama module is available
        if not OLLAMA_AVAILABLE:
            logger.warning("Ollama module not available, skipping Ollama checks")
            self.ollama_available = False
        else:
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
        
        # Check if we have any working LLM - if not, use demo mode
        if not self.ollama_available and not self.openai_client:
            logger.warning("No working LLM found! Using demo mode.")
            logger.warning(f"Ollama available: {self.ollama_available}")
            logger.warning(f"OpenAI client: {self.openai_client}")
            logger.warning("Will return demo responses for extraction and queries")
            self.demo_mode = True
        else:
            self.demo_mode = False
        
        # Log which LLM will be used
        if self.demo_mode:
            logger.warning("⚠️ Using DEMO mode - no actual LLM processing")
        elif self.ollama_available:
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
        if not OLLAMA_AVAILABLE:
            logger.info("Ollama module not available")
            return False
            
        try:
            # Try to list models to check if Ollama is running
            response = ollama.list()
            logger.info(f"Ollama list response type: {type(response)}")
            
            # Check if we got a proper response with models attribute
            if hasattr(response, 'models'):
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
        if not OLLAMA_AVAILABLE:
            logger.warning("Ollama module not available, cannot check model")
            return False
            
        try:
            models_response = ollama.list()
            models = models_response.models if hasattr(models_response, 'models') else []
            logger.info(f"Checking model availability. Found {len(models)} models")
            
            # Extract model names from Model objects
            model_names = []
            for model in models:
                if hasattr(model, 'model'):  # Model object with 'model' attribute
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
        
        # Use demo mode if no LLM is available
        if self.demo_mode:
            logger.info("Using demo mode for extraction")
            return {
                "administrative": {
                    "submitter_name": "Demo User",
                    "submitter_email": "demo@lab.local",
                    "submitter_phone": "(555) 123-4567",
                    "project_name": "Demo Project",
                    "institution": "Demo Laboratory"
                },
                "sample": {
                    "sample_id": "DEMO_001",
                    "sample_type": "DNA",
                    "concentration": "50 ng/uL",
                    "volume": "100 uL",
                    "storage_conditions": "-80C"
                },
                "sequencing": {
                    "platform": "Illumina",
                    "analysis_type": "WGS",
                    "coverage": "30x",
                    "read_length": "150bp"
                }
            }
        
        # Try Ollama first if available
        if self.ollama_available and OLLAMA_AVAILABLE:
            try:
                if self._ensure_model_available():
                    logger.info(f"Generating response with Ollama model: {self.model}")
                    try:
                        response = ollama.generate(
                            model=self.model,
                            prompt=self.extraction_prompt.format(text=text),
                            options={'temperature': 0.1, 'num_predict': 1000}
                        )
                        logger.info(f"✅ Ollama generate succeeded")
                    except Exception as ollama_error:
                        logger.error(f"Ollama generate failed: {ollama_error}")
                        raise
                    
                    # Parse JSON response - handle different response structures
                    if hasattr(response, 'response'):
                        result_text = response.response.strip()
                    elif isinstance(response, dict) and 'response' in response:
                        result_text = response['response'].strip()
                    else:
                        logger.error(f"Unexpected response structure: {type(response)}")
                        raise ValueError(f"Unexpected response structure: {type(response)}")
                        
                    logger.info(f"Raw Ollama response: {result_text[:200]}...")
                    
                    # Clean up common Ollama response formatting
                    if result_text.startswith('```json'):
                        result_text = result_text[7:-3].strip()
                    elif result_text.startswith('```'):
                        result_text = result_text[3:-3].strip()
                    
                    # Additional cleaning for common formatting issues
                    result_text = result_text.strip()
                    if not result_text.startswith('{'):
                        # Find the first { and last }
                        start = result_text.find('{')
                        end = result_text.rfind('}')
                        if start != -1 and end != -1:
                            result_text = result_text[start:end+1]
                        else:
                            logger.warning(f"No valid JSON structure found. Raw response: {result_text}")
                            if '"administrative"' in result_text:
                                result_text = '{\n' + result_text.strip() + '\n}'
                    
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
                    max_tokens=1000
                )
                
                result_text = response.choices[0].message.content.strip()
                
                # Clean up the response before parsing JSON
                if result_text.startswith('```json'):
                    result_text = result_text[7:-3].strip()
                elif result_text.startswith('```'):
                    result_text = result_text[3:-3].strip()
                
                result_text = result_text.strip()
                if not result_text.startswith('{'):
                    start = result_text.find('{')
                    end = result_text.rfind('}')
                    if start != -1 and end != -1:
                        result_text = result_text[start:end+1]
                
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
                        options={'temperature': 0.1, 'num_predict': 500}
                    )
                    return response['response'].strip()
            except Exception as e:
                logger.warning(f"Ollama query failed: {e}")
        
        # Fallback to OpenAI if available
        if self.openai_client:
            try:
                response = self.openai_client.chat.completions.create(
                    model="gpt-3.5-turbo",
                    messages=[{"role": "user", "content": prompt}],
                    temperature=0.1,
                    max_tokens=500
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
                "institution": "University Research Lab"
            },
            "sample": {
                "sample_id": "DNA_SAMPLE_001",
                "sample_type": "Genomic DNA",
                "concentration": "50 ng/uL",
                "volume": "100 uL",
                "storage_conditions": "Frozen at -80C"
            },
            "sequencing": {
                "platform": "Illumina NovaSeq 6000",
                "analysis_type": "Whole Genome Sequencing",
                "coverage": "30x",
                "read_length": "150bp paired-end"
            }
        }
    
    def extract_submission_info(self, text: str) -> Dict[str, Any]:
        """Demo extraction that returns simulated data"""
        return self.demo_response
    
    def answer_query(self, query: str, context: str) -> str:
        """Demo query answering"""
        query_lower = query.lower()
        
        if "submitter" in query_lower:
            return "The submitter is Dr. Sarah Johnson from University Research Lab (sarah.johnson@university.edu)."
        elif "sample" in query_lower:
            return "Sample DNA_SAMPLE_001 is genomic DNA with 50 ng/uL concentration and 100 uL volume, stored frozen at -80C."
        elif "sequencing" in query_lower:
            return "Sequencing will be performed on Illumina NovaSeq 6000 platform for whole genome sequencing with 30x coverage and 150bp paired-end reads."
        else:
            return "This is a demo response. The system contains sample information for genomic DNA sequencing project." 
