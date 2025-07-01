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
from typing import Any, Dict, List, Optional

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


class RetryExtractionEvent(Event):
    """Request to retry extraction with feedback"""
    text: str
    attempt_number: int
    validation_errors: List[str]
    previous_result: Dict[str, Any]


class HumanReviewRequestEvent(Event):
    """Request for human review"""
    extraction_result: Dict[str, Any]
    confidence_score: float
    validation_errors: List[str]
    attempt_number: int


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
        text = ev.get("text")
        require_human_review = ev.get("require_human_review", False)
        
        ctx.data["text"] = text
        ctx.data["require_human_review"] = require_human_review
        ctx.data["extraction_history"] = []
        
        logger.info("Starting quality-controlled extraction")
        
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
            ev.get("previous_result"),
            ev.get("feedback") or ev.get("validation_errors", [])
        )
        
        # Perform extraction
        try:
            extraction_result = await self.llm_interface.extract_submission_info_with_prompt(
                prompt
            )
            
            # Calculate confidence score
            confidence_score = self._calculate_confidence_score(extraction_result)
            
            # Validate extraction
            validation_errors, missing_fields = self._validate_extraction(extraction_result)
            
            # Store attempt in history
            ctx.data["extraction_history"].append({
                "attempt": ev.attempt_number,
                "result": extraction_result,
                "confidence": confidence_score,
                "errors": validation_errors
            })
            
            return ValidationResultEvent(
                extraction_result=extraction_result,
                confidence_score=confidence_score,
                validation_errors=validation_errors,
                missing_fields=missing_fields,
                attempt_number=ev.attempt_number
            )
            
        except Exception as e:
            logger.error(f"Extraction failed: {e}")
            return ValidationResultEvent(
                extraction_result={},
                confidence_score=0.0,
                validation_errors=[f"Extraction error: {str(e)}"],
                missing_fields=[],
                attempt_number=ev.attempt_number
            )
    
    @step(pass_context=True)
    async def evaluate_quality(
        self, ctx: Context, ev: ValidationResultEvent
    ) -> RetryExtractionEvent | HumanReviewRequestEvent | StopEvent:
        """Evaluate extraction quality and decide next action"""
        
        logger.info(
            f"Evaluating quality - Confidence: {ev.confidence_score:.2f}, "
            f"Errors: {len(ev.validation_errors)}"
        )
        
        # High confidence - accept result
        if ev.confidence_score >= CONFIDENCE_THRESHOLD_HIGH and not ev.validation_errors:
            logger.info("High confidence extraction accepted")
            return StopEvent(
                result=self._create_extraction_result(
                    ev.extraction_result,
                    ev.confidence_score,
                    True,
                    ctx.data["text"]
                )
            )
        
        # Low confidence or errors - retry if attempts remaining
        if ev.attempt_number < MAX_RETRY_ATTEMPTS:
            if ev.confidence_score < CONFIDENCE_THRESHOLD_MEDIUM or ev.validation_errors:
                logger.info(f"Retrying extraction (attempt {ev.attempt_number + 1})")
                
                return RetryExtractionEvent(
                    text=ctx.data["text"],
                    attempt_number=ev.attempt_number + 1,
                    validation_errors=ev.validation_errors,
                    previous_result=ev.extraction_result
                )
        
        # Medium confidence or max attempts reached - request human review if enabled
        if ctx.data.get("require_human_review", False):
            logger.info("Requesting human review")
            return HumanReviewRequestEvent(
                extraction_result=ev.extraction_result,
                confidence_score=ev.confidence_score,
                validation_errors=ev.validation_errors,
                attempt_number=ev.attempt_number
            )
        
        # Otherwise, accept the best result we have
        logger.info("Accepting best available result")
        return StopEvent(
            result=self._create_extraction_result(
                ev.extraction_result,
                ev.confidence_score,
                ev.confidence_score >= CONFIDENCE_THRESHOLD_LOW,
                ctx.data["text"],
                warnings=ev.validation_errors
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
        
        # Wait for human response
        human_response = await ctx.wait_for_event(HumanReviewResponseEvent)
        
        if human_response.approved:
            # Apply any corrections from human review
            final_result = ev.extraction_result
            if human_response.corrections:
                final_result.update(human_response.corrections)
            
            return StopEvent(
                result=self._create_extraction_result(
                    final_result,
                    1.0,  # Human-reviewed results have perfect confidence
                    True,
                    ctx.data["text"],
                    warnings=["Human-reviewed and approved"]
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
                    warnings=["Human review rejected extraction"]
                )
            )
    
    def _build_extraction_prompt(
        self,
        text: str,
        previous_result: Optional[Dict[str, Any]] = None,
        feedback: Optional[List[str]] = None
    ) -> str:
        """Build extraction prompt with feedback from previous attempts"""
        
        base_prompt = f"""
Extract laboratory submission information from the following text.
Focus on accuracy and completeness.

Text: {text}
"""
        
        if previous_result and feedback:
            feedback_prompt = f"""

Previous extraction had these issues:
{json.dumps(feedback, indent=2)}

Previous result:
{json.dumps(previous_result, indent=2)}

Please correct these issues in your extraction.
"""
            base_prompt += feedback_prompt
        
        return base_prompt
    
    def _calculate_confidence_score(
        self, extraction_result: Dict[str, Any]
    ) -> float:
        """Calculate confidence score based on extraction completeness and quality"""
        
        if not extraction_result:
            return 0.0
        
        # Count filled fields
        total_fields = 0
        filled_fields = 0
        
        for category in ["administrative", "sample", "sequencing"]:
            if category in extraction_result:
                for field, value in extraction_result[category].items():
                    total_fields += 1
                    if value and value != "null":
                        filled_fields += 1
        
        # Base confidence on field completeness
        if total_fields == 0:
            return 0.0
        
        completeness_score = filled_fields / total_fields
        
        # Adjust for critical fields
        critical_fields_filled = 0
        critical_fields = ["submitter_email", "sample_id", "sample_type"]
        
        for field in critical_fields:
            for category in extraction_result.values():
                if isinstance(category, dict) and field in category:
                    if category[field] and category[field] != "null":
                        critical_fields_filled += 1
                        break
        
        critical_score = critical_fields_filled / len(critical_fields)
        
        # Weighted average
        confidence = (completeness_score * 0.6) + (critical_score * 0.4)
        
        return min(confidence, 1.0)
    
    def _validate_extraction(
        self, extraction_result: Dict[str, Any]
    ) -> tuple[List[str], List[str]]:
        """Validate extraction result and return errors and missing fields"""
        
        validation_errors = []
        missing_fields = []
        
        # Check required structure
        required_categories = ["administrative", "sample", "sequencing"]
        for category in required_categories:
            if category not in extraction_result:
                validation_errors.append(f"Missing category: {category}")
        
        # Validate administrative info
        if "administrative" in extraction_result:
            admin = extraction_result["administrative"]
            if admin.get("submitter_email"):
                # Simple email validation
                if "@" not in str(admin["submitter_email"]):
                    validation_errors.append("Invalid email format")
            else:
                missing_fields.append("submitter_email")
        
        # Validate sample info
        if "sample" in extraction_result:
            sample = extraction_result["sample"]
            if not sample.get("sample_id"):
                missing_fields.append("sample_id")
            if not sample.get("sample_type"):
                missing_fields.append("sample_type")
        
        # Validate sequencing info
        if "sequencing" in extraction_result:
            seq = extraction_result["sequencing"]
            # Check for valid platforms
            valid_platforms = ["illumina", "pacbio", "nanopore", "ion torrent"]
            if seq.get("platform"):
                if seq["platform"].lower() not in valid_platforms:
                    validation_errors.append(
                        f"Unknown sequencing platform: {seq['platform']}"
                    )
        
        return validation_errors, missing_fields
    
    def _create_extraction_result(
        self,
        extraction_data: Dict[str, Any],
        confidence_score: float,
        success: bool,
        source_text: str,
        warnings: Optional[List[str]] = None
    ) -> ExtractionResult:
        """Create extraction result from validated data"""
        
        # Convert to LabSubmission if successful
        submission = None
        if success and extraction_data:
            try:
                # Create submission from extraction data
                submission = LabSubmission(
                    administrative=extraction_data.get("administrative", {}),
                    sample=extraction_data.get("sample", {}),
                    sequencing=extraction_data.get("sequencing", {}),
                    raw_text=source_text[:1000],  # Store first 1000 chars
                    confidence_score=confidence_score
                )
            except Exception as e:
                logger.error(f"Failed to create LabSubmission: {e}")
                success = False
                warnings = (warnings or []) + [f"Failed to create submission: {str(e)}"]
        
        return ExtractionResult(
            success=success,
            submission=submission,
            confidence_score=confidence_score,
            missing_fields=[],
            warnings=warnings or [],
            processing_time=0.0,  # Will be set by caller
            source_document="quality_control_workflow"
        )


# Example usage function
async def extract_with_quality_control(
    text: str,
    require_human_review: bool = False
) -> ExtractionResult:
    """Extract laboratory submission with quality control"""
    
    workflow = QualityControlWorkflow()
    result = await workflow.run(
        text=text,
        require_human_review=require_human_review
    )
    
    return result