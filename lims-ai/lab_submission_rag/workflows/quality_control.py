"""
Quality Control Workflow with Self-Correction

This workflow implements quality control for laboratory data extraction with:
1. Confidence scoring and validation
2. Retry logic for failed extractions
3. Self-correction loops for improving extraction quality
4. Human-in-the-loop approval when needed
"""

import asyncio
import json
import logging
import re
from datetime import datetime
from typing import Any, Dict, List, Optional, Tuple

from llama_index.workflows import (
    Context,
    Event,
    StartEvent,
    StopEvent,
    Workflow,
    step,
)
from pydantic import BaseModel, Field

from ..models.submission import ExtractionResult, LabSubmission
from ..rag.llm_interface import LLMInterface

logger = logging.getLogger(__name__)


# Quality control thresholds
CONFIDENCE_THRESHOLD_HIGH = 0.85
CONFIDENCE_THRESHOLD_MEDIUM = 0.70
CONFIDENCE_THRESHOLD_LOW = 0.50
MAX_RETRY_ATTEMPTS = 3

# Field validation patterns
EMAIL_PATTERN = re.compile(r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$')
PHONE_PATTERN = re.compile(r'^\+?1?\d{10,14}$')
SAMPLE_ID_PATTERN = re.compile(r'^[A-Z0-9\-_]+$')


# Define workflow events
class ExtractionAttemptEvent(Event):
    """Request for extraction attempt"""
    text: str
    attempt_number: int = 1
    previous_result: Optional[Dict[str, Any]] = None
    feedback: Optional[str] = None


class ValidationResultEvent(Event):
    """Result of validation check"""
    extraction_result: Dict[str, Any]
    confidence_score: float
    validation_errors: List[str]
    missing_fields: List[str]
    attempt_number: int
    field_scores: Dict[str, float]


class RetryExtractionEvent(Event):
    """Request to retry extraction with feedback"""
    text: str
    attempt_number: int
    validation_errors: List[str]
    previous_result: Dict[str, Any]
    improvement_suggestions: List[str]


class HumanReviewRequestEvent(Event):
    """Request for human review"""
    extraction_result: Dict[str, Any]
    confidence_score: float
    validation_errors: List[str]
    missing_fields: List[str]
    attempt_number: int
    field_scores: Dict[str, float]


class HumanReviewResponseEvent(Event):
    """Human review response"""
    approved: bool
    corrections: Optional[Dict[str, Any]] = None
    feedback: Optional[str] = None


class QualityControlWorkflow(Workflow):
    """
    Quality control workflow for laboratory data extraction.
    
    Implements self-correcting loops with confidence scoring,
    validation, and optional human review.
    """
    
    def __init__(self, timeout: int = 600, verbose: bool = True):
        super().__init__(timeout=timeout, verbose=verbose)
        self.llm_interface = LLMInterface()
        
    @step(pass_context=True)
    async def start_extraction(
        self, ctx: Context, ev: StartEvent
    ) -> ExtractionAttemptEvent:
        """Initialize extraction process"""
        text = getattr(ev, "text", None)
        require_human_review = getattr(ev, "require_human_review", False)
        extraction_mode = getattr(ev, "extraction_mode", "standard")
        
        ctx.data["text"] = text
        ctx.data["require_human_review"] = require_human_review
        ctx.data["extraction_mode"] = extraction_mode
        ctx.data["extraction_history"] = []
        ctx.data["start_time"] = datetime.utcnow()
        
        logger.info(f"Starting quality-controlled extraction (mode: {extraction_mode})")
        
        return ExtractionAttemptEvent(
            text=text,
            attempt_number=1
        )
    
    @step(pass_context=True)
    async def extract_with_feedback(
        self, ctx: Context, ev: ExtractionAttemptEvent | RetryExtractionEvent
    ) -> ValidationResultEvent:
        """Perform extraction with optional feedback from previous attempts"""
        
        logger.info(f"Extraction attempt #{ev.attempt_number}")
        
        # Build enhanced prompt with feedback
        prompt = self._build_extraction_prompt(
            ev.text,
            getattr(ev, "previous_result", None),
            feedback=getattr(ev, "feedback", None),
            validation_errors=getattr(ev, "validation_errors", []),
            improvement_suggestions=getattr(ev, "improvement_suggestions", [])
        )
        
        # Perform extraction
        try:
            extraction_result = await self.llm_interface.extract_submission_info_with_prompt(
                prompt
            )
            
            # Handle extraction errors
            if "error" in extraction_result:
                logger.error(f"Extraction error: {extraction_result['error']}")
                extraction_result = self._get_empty_extraction()
            
            # Calculate confidence score and field scores
            confidence_score, field_scores = self._calculate_detailed_confidence(
                extraction_result
            )
            
            # Validate extraction
            validation_errors, missing_fields = self._validate_extraction(
                extraction_result
            )
            
            # Store attempt in history
            ctx.data["extraction_history"].append({
                "attempt": ev.attempt_number,
                "result": extraction_result,
                "confidence": confidence_score,
                "field_scores": field_scores,
                "errors": validation_errors,
                "timestamp": datetime.utcnow().isoformat()
            })
            
            return ValidationResultEvent(
                extraction_result=extraction_result,
                confidence_score=confidence_score,
                validation_errors=validation_errors,
                missing_fields=missing_fields,
                attempt_number=ev.attempt_number,
                field_scores=field_scores
            )
            
        except Exception as e:
            logger.error(f"Extraction failed: {e}")
            return ValidationResultEvent(
                extraction_result=self._get_empty_extraction(),
                confidence_score=0.0,
                validation_errors=[f"Extraction error: {str(e)}"],
                missing_fields=[],
                attempt_number=ev.attempt_number,
                field_scores={}
            )
    
    @step(pass_context=True)
    async def evaluate_quality(
        self, ctx: Context, ev: ValidationResultEvent
    ) -> RetryExtractionEvent | HumanReviewRequestEvent | StopEvent:
        """Evaluate extraction quality and decide next action"""
        
        logger.info(
            f"Evaluating quality - Confidence: {ev.confidence_score:.2f}, "
            f"Errors: {len(ev.validation_errors)}, Missing: {len(ev.missing_fields)}"
        )
        
        # High confidence - accept result
        if ev.confidence_score >= CONFIDENCE_THRESHOLD_HIGH and not ev.validation_errors:
            logger.info("High confidence extraction accepted")
            return StopEvent(
                result=self._create_extraction_result(
                    ev.extraction_result,
                    ev.confidence_score,
                    True,
                    ctx.data["text"],
                    missing_fields=ev.missing_fields,
                    metadata={
                        "attempts": ev.attempt_number,
                        "field_scores": ev.field_scores,
                        "extraction_mode": ctx.data.get("extraction_mode", "standard"),
                        "processing_time": (datetime.utcnow() - ctx.data["start_time"]).total_seconds()
                    }
                )
            )
        
        # Low confidence or errors - retry if attempts remaining
        if ev.attempt_number < MAX_RETRY_ATTEMPTS:
            if ev.confidence_score < CONFIDENCE_THRESHOLD_MEDIUM or ev.validation_errors:
                logger.info(f"Retrying extraction (attempt {ev.attempt_number + 1})")
                
                # Generate improvement suggestions
                improvement_suggestions = self._generate_improvement_suggestions(
                    ev.validation_errors,
                    ev.missing_fields,
                    ev.field_scores
                )
                
                return RetryExtractionEvent(
                    text=ctx.data["text"],
                    attempt_number=ev.attempt_number + 1,
                    validation_errors=ev.validation_errors,
                    previous_result=ev.extraction_result,
                    improvement_suggestions=improvement_suggestions
                )
        
        # Medium confidence or max attempts reached - request human review if enabled
        if ctx.data.get("require_human_review", False):
            logger.info("Requesting human review")
            return HumanReviewRequestEvent(
                extraction_result=ev.extraction_result,
                confidence_score=ev.confidence_score,
                validation_errors=ev.validation_errors,
                missing_fields=ev.missing_fields,
                attempt_number=ev.attempt_number,
                field_scores=ev.field_scores
            )
        
        # Otherwise, accept the best result we have
        logger.info("Accepting best available result")
        return StopEvent(
            result=self._create_extraction_result(
                ev.extraction_result,
                ev.confidence_score,
                ev.confidence_score >= CONFIDENCE_THRESHOLD_LOW,
                ctx.data["text"],
                warnings=ev.validation_errors,
                missing_fields=ev.missing_fields,
                metadata={
                    "attempts": ev.attempt_number,
                    "field_scores": ev.field_scores,
                    "extraction_mode": ctx.data.get("extraction_mode", "standard"),
                    "processing_time": (datetime.utcnow() - ctx.data["start_time"]).total_seconds(),
                    "quality_threshold_met": False
                }
            )
        )
    
    @step(pass_context=True)
    async def handle_human_review(
        self, ctx: Context, ev: HumanReviewRequestEvent
    ) -> StopEvent:
        """Handle human review process"""
        
        # In a real implementation, this would integrate with a UI
        # For now, we'll simulate or wait for external input
        
        logger.info("Awaiting human review...")
        
        # Write event to stream for external handling
        ctx.write_event_to_stream(ev)
        
        # Wait for human response with timeout
        try:
            human_response = await asyncio.wait_for(
                ctx.wait_for_event(HumanReviewResponseEvent),
                timeout=300  # 5 minute timeout
            )
        except asyncio.TimeoutError:
            logger.warning("Human review timed out")
            return StopEvent(
                result=self._create_extraction_result(
                    ev.extraction_result,
                    ev.confidence_score,
                    False,
                    ctx.data["text"],
                    warnings=["Human review timed out"] + ev.validation_errors,
                    missing_fields=ev.missing_fields
                )
            )
        
        if human_response.approved:
            # Apply any corrections from human review
            final_result = ev.extraction_result.copy()
            if human_response.corrections:
                final_result = self._merge_corrections(final_result, human_response.corrections)
            
            return StopEvent(
                result=self._create_extraction_result(
                    final_result,
                    1.0,  # Human-reviewed results have perfect confidence
                    True,
                    ctx.data["text"],
                    warnings=["Human-reviewed and approved"],
                    missing_fields=[],  # Human review resolves missing fields
                    metadata={
                        "human_reviewed": True,
                        "human_feedback": human_response.feedback,
                        "original_confidence": ev.confidence_score,
                        "processing_time": (datetime.utcnow() - ctx.data["start_time"]).total_seconds()
                    }
                )
            )
        else:
            # Human rejected the extraction
            return StopEvent(
                result=self._create_extraction_result(
                    {},
                    0.0,
                    False,
                    ctx.data["text"],
                    warnings=["Human review rejected extraction"],
                    metadata={
                        "human_reviewed": True,
                        "human_rejected": True,
                        "human_feedback": human_response.feedback
                    }
                )
            )
    
    def _build_extraction_prompt(
        self,
        text: str,
        previous_result: Optional[Dict[str, Any]] = None,
        feedback: Optional[str] = None,
        validation_errors: Optional[List[str]] = None,
        improvement_suggestions: Optional[List[str]] = None
    ) -> str:
        """Build extraction prompt with feedback from previous attempts"""
        
        base_prompt = f"""
Extract laboratory submission information from the following text.
Focus on accuracy and completeness. All extracted values should be from the text, do not make up information.

Text: {text}

Required format:
{{
    "administrative": {{
        "submitter_first_name": "...",
        "submitter_last_name": "...",
        "submitter_email": "...",
        "submitter_phone": "...",
        "institution": "...",
        "department": "..."
    }},
    "sample": {{
        "sample_id": "...",
        "sample_type": "...",
        "volume": "...",
        "concentration": "...",
        "collection_date": "..."
    }},
    "sequencing": {{
        "platform": "...",
        "read_length": "...",
        "coverage": "...",
        "library_prep": "..."
    }},
    "storage": {{
        "temperature": "...",
        "container_type": "...",
        "location": "..."
    }}
}}
"""
        
        if previous_result and (validation_errors or improvement_suggestions):
            feedback_prompt = f"""

Previous extraction had these issues:
{json.dumps(validation_errors or [], indent=2)}

Improvement suggestions:
{json.dumps(improvement_suggestions or [], indent=2)}

Previous result:
{json.dumps(previous_result, indent=2)}

Please correct these issues in your extraction. Pay special attention to:
1. Email addresses must be valid (contain @ and domain)
2. Phone numbers should include country code if available
3. Sample IDs should use standard format (uppercase letters, numbers, hyphens)
4. Dates should be in ISO format (YYYY-MM-DD)
5. Only extract information that is explicitly stated in the text
"""
            base_prompt += feedback_prompt
        
        if feedback:
            base_prompt += f"\n\nAdditional feedback: {feedback}"
        
        return base_prompt
    
    def _calculate_detailed_confidence(
        self, extraction_result: Dict[str, Any]
    ) -> Tuple[float, Dict[str, float]]:
        """Calculate confidence score with detailed field-level scores"""
        
        if not extraction_result or "error" in extraction_result:
            return 0.0, {}
        
        field_scores = {}
        category_scores = {}
        
        # Define field weights
        field_weights = {
            "administrative": {
                "submitter_email": 0.3,
                "submitter_first_name": 0.2,
                "submitter_last_name": 0.2,
                "institution": 0.15,
                "submitter_phone": 0.1,
                "department": 0.05
            },
            "sample": {
                "sample_id": 0.3,
                "sample_type": 0.25,
                "volume": 0.15,
                "concentration": 0.15,
                "collection_date": 0.15
            },
            "sequencing": {
                "platform": 0.4,
                "coverage": 0.3,
                "read_length": 0.2,
                "library_prep": 0.1
            },
            "storage": {
                "temperature": 0.5,
                "container_type": 0.3,
                "location": 0.2
            }
        }
        
        # Calculate scores for each category
        for category, fields in field_weights.items():
            if category in extraction_result:
                category_data = extraction_result[category]
                category_score = 0.0
                
                for field, weight in fields.items():
                    field_value = category_data.get(field)
                    field_score = 0.0
                    
                    if field_value and str(field_value).strip() and str(field_value).lower() != "null":
                        # Base score for having a value
                        field_score = 0.7
                        
                        # Additional validation
                        if field == "submitter_email" and EMAIL_PATTERN.match(str(field_value)):
                            field_score = 1.0
                        elif field == "submitter_phone" and PHONE_PATTERN.match(str(field_value).replace("-", "").replace(" ", "")):
                            field_score = 1.0
                        elif field == "sample_id" and SAMPLE_ID_PATTERN.match(str(field_value)):
                            field_score = 1.0
                        elif field in ["submitter_first_name", "submitter_last_name", "institution"]:
                            field_score = 1.0 if len(str(field_value)) > 1 else 0.5
                        
                    field_scores[f"{category}.{field}"] = field_score
                    category_score += field_score * weight
                
                category_scores[category] = category_score
        
        # Calculate overall confidence
        category_weights = {
            "administrative": 0.3,
            "sample": 0.3,
            "sequencing": 0.25,
            "storage": 0.15
        }
        
        overall_confidence = sum(
            category_scores.get(cat, 0) * weight 
            for cat, weight in category_weights.items()
        )
        
        return min(overall_confidence, 1.0), field_scores
    
    def _validate_extraction(
        self, extraction_result: Dict[str, Any]
    ) -> Tuple[List[str], List[str]]:
        """Validate extraction result and return errors and missing fields"""
        
        validation_errors = []
        missing_fields = []
        
        if not extraction_result or "error" in extraction_result:
            validation_errors.append("Extraction failed or returned empty result")
            return validation_errors, missing_fields
        
        # Check required structure
        required_categories = ["administrative", "sample", "sequencing"]
        for category in required_categories:
            if category not in extraction_result:
                validation_errors.append(f"Missing category: {category}")
        
        # Validate administrative info
        if "administrative" in extraction_result:
            admin = extraction_result["administrative"]
            
            # Email validation
            email = admin.get("submitter_email")
            if email:
                if not EMAIL_PATTERN.match(str(email)):
                    validation_errors.append(f"Invalid email format: {email}")
            else:
                missing_fields.append("submitter_email")
            
            # Name validation
            if not admin.get("submitter_first_name"):
                missing_fields.append("submitter_first_name")
            if not admin.get("submitter_last_name"):
                missing_fields.append("submitter_last_name")
            
            # Phone validation
            phone = admin.get("submitter_phone")
            if phone:
                cleaned_phone = str(phone).replace("-", "").replace(" ", "").replace("(", "").replace(")", "")
                if not PHONE_PATTERN.match(cleaned_phone):
                    validation_errors.append(f"Invalid phone format: {phone}")
        
        # Validate sample info
        if "sample" in extraction_result:
            sample = extraction_result["sample"]
            
            # Sample ID validation
            sample_id = sample.get("sample_id")
            if sample_id:
                if not SAMPLE_ID_PATTERN.match(str(sample_id)):
                    validation_errors.append(f"Invalid sample ID format: {sample_id}")
            else:
                missing_fields.append("sample_id")
            
            # Sample type validation
            if not sample.get("sample_type"):
                missing_fields.append("sample_type")
            else:
                valid_types = ["blood", "saliva", "tissue", "dna", "rna", "plasma", "serum", "urine", "swab"]
                if str(sample["sample_type"]).lower() not in valid_types:
                    validation_errors.append(f"Unknown sample type: {sample['sample_type']}")
            
            # Numeric validations
            if sample.get("volume"):
                try:
                    volume = float(str(sample["volume"]).replace("µL", "").replace("ml", "").strip())
                    if volume <= 0:
                        validation_errors.append(f"Invalid volume: {sample['volume']}")
                except ValueError:
                    validation_errors.append(f"Volume must be numeric: {sample['volume']}")
        
        # Validate sequencing info
        if "sequencing" in extraction_result:
            seq = extraction_result["sequencing"]
            
            # Platform validation
            if seq.get("platform"):
                valid_platforms = ["illumina", "pacbio", "nanopore", "ion torrent", "element", "ultima"]
                if str(seq["platform"]).lower() not in valid_platforms:
                    validation_errors.append(f"Unknown sequencing platform: {seq['platform']}")
            else:
                missing_fields.append("sequencing_platform")
        
        return validation_errors, missing_fields
    
    def _generate_improvement_suggestions(
        self,
        validation_errors: List[str],
        missing_fields: List[str],
        field_scores: Dict[str, float]
    ) -> List[str]:
        """Generate specific suggestions for improving extraction"""
        
        suggestions = []
        
        # Suggestions for missing fields
        if missing_fields:
            suggestions.append(f"Look more carefully for these missing fields: {', '.join(missing_fields)}")
        
        # Suggestions for low-scoring fields
        low_score_fields = [
            field for field, score in field_scores.items() 
            if score < 0.7
        ]
        if low_score_fields:
            suggestions.append(f"Re-examine these fields for better values: {', '.join(low_score_fields)}")
        
        # Specific error suggestions
        for error in validation_errors:
            if "email" in error.lower():
                suggestions.append("Ensure email addresses include @ symbol and valid domain")
            elif "phone" in error.lower():
                suggestions.append("Format phone numbers with country code (e.g., +1-555-123-4567)")
            elif "sample ID" in error.lower():
                suggestions.append("Sample IDs should use uppercase letters, numbers, and hyphens only")
        
        return suggestions
    
    def _merge_corrections(
        self, original: Dict[str, Any], corrections: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Merge human corrections into extraction result"""
        
        result = original.copy()
        
        for key, value in corrections.items():
            if "." in key:  # Handle nested keys like "administrative.submitter_email"
                parts = key.split(".")
                current = result
                for part in parts[:-1]:
                    if part not in current:
                        current[part] = {}
                    current = current[part]
                current[parts[-1]] = value
            else:
                result[key] = value
        
        return result
    
    def _get_empty_extraction(self) -> Dict[str, Any]:
        """Return empty extraction structure"""
        return {
            "administrative": {},
            "sample": {},
            "sequencing": {},
            "storage": {}
        }
    
    def _create_extraction_result(
        self,
        extraction_data: Dict[str, Any],
        confidence_score: float,
        success: bool,
        source_text: str,
        warnings: Optional[List[str]] = None,
        missing_fields: Optional[List[str]] = None,
        metadata: Optional[Dict[str, Any]] = None
    ) -> ExtractionResult:
        """Create extraction result from validated data"""
        
        # Create result with extracted data
        result = ExtractionResult(
            success=success,
            submission=None,  # Still skip LabSubmission creation due to model mismatch
            extracted_data=extraction_data,
            confidence_score=confidence_score,
            missing_fields=missing_fields or [],
            warnings=warnings or [],
            processing_time=0.0,  # Will be set by caller
            source_document="quality_control_workflow"
        )
        
        # Add metadata if available
        if metadata:
            result.metadata = metadata
        
        return result


# Example usage function
async def extract_with_quality_control(
    text: str,
    require_human_review: bool = False,
    extraction_mode: str = "standard"
) -> ExtractionResult:
    """Extract laboratory submission with quality control
    
    Args:
        text: Text to extract from
        require_human_review: Whether to require human review for low confidence
        extraction_mode: Mode of extraction ("standard", "strict", "lenient")
    """
    
    workflow = QualityControlWorkflow()
    result = await workflow.run(
        text=text,
        require_human_review=require_human_review,
        extraction_mode=extraction_mode
    )
    
    return result