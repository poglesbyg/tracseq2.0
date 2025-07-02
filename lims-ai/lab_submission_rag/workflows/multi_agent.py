"""
Multi-Agent Laboratory Workflow

This workflow coordinates multiple specialized agents:
1. Document Processing Agent - Handles document ingestion and extraction
2. Quality Control Agent - Validates and improves data quality
3. Storage Allocation Agent - Manages sample storage decisions
4. Notification Agent - Handles alerts and communications
"""

import asyncio
import json
import logging
from datetime import datetime, timedelta
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

from ..config import settings
from ..rag.document_processor import DocumentProcessor
from ..rag.llm_interface import LLMInterface
from ..rag.vector_store import VectorStore
from ..models.submission import ExtractionResult

logger = logging.getLogger(__name__)


# Define agent events
class DocumentReceivedEvent(Event):
    """New document received for processing"""
    document_path: str
    submission_type: str
    priority: str = "normal"
    metadata: Dict[str, Any] = Field(default_factory=dict)


class ExtractionRequestEvent(Event):
    """Request document extraction"""
    document_path: str
    agent: str = "document_processor"
    retry_count: int = 0


class ExtractionResponseEvent(Event):
    """Extraction completed"""
    extracted_data: Dict[str, Any]
    confidence_score: float
    source_agent: str
    processing_time: float
    chunk_count: int = 0


class QualityCheckRequestEvent(Event):
    """Request quality validation"""
    data: Dict[str, Any]
    confidence_score: float
    check_type: str = "standard"
    previous_issues: List[str] = Field(default_factory=list)


class QualityCheckResponseEvent(Event):
    """Quality check completed"""
    validated_data: Dict[str, Any]
    quality_score: float
    issues_found: List[str]
    corrections_made: List[str]
    recommendations: List[str]


class StorageAllocationRequestEvent(Event):
    """Request storage allocation"""
    sample_data: Dict[str, Any]
    storage_requirements: Dict[str, Any]
    priority: str = "normal"


class StorageAllocationResponseEvent(Event):
    """Storage allocation completed"""
    location_id: str
    zone: str
    position: str
    temperature: float
    capacity_remaining: float
    allocation_time: datetime


class NotificationRequestEvent(Event):
    """Request to send notification"""
    notification_type: str
    recipients: List[str]
    data: Dict[str, Any]
    priority: str = "normal"
    template: Optional[str] = None


class NotificationSentEvent(Event):
    """Notification sent"""
    notification_id: str
    status: str
    timestamp: datetime
    delivery_method: str
    recipients_notified: List[str]


class AgentErrorEvent(Event):
    """Agent encountered an error"""
    agent_name: str
    error_message: str
    recoverable: bool
    context: Dict[str, Any]


class WorkflowCompleteEvent(Event):
    """Workflow completed"""
    submission_id: str
    status: str
    summary: Dict[str, Any]


# Agent definitions
class DocumentProcessorAgent:
    """Agent responsible for document processing and extraction"""
    
    def __init__(self):
        self.processor = DocumentProcessor()
        self.llm_interface = LLMInterface()
        self.vector_store = VectorStore()
    
    async def process_document(
        self, document_path: str, priority: str = "normal"
    ) -> tuple[Dict[str, Any], float, int]:
        """Process document and extract information"""
        logger.info(f"DocumentProcessorAgent: Processing {document_path} (priority: {priority})")
        
        start_time = datetime.utcnow()
        
        try:
            # Process document into chunks
            chunks = await self.processor.process_document(document_path)
            
            if not chunks:
                raise ValueError("No content extracted from document")
            
            # Store chunks in vector store
            await self.vector_store.add_chunks(chunks)
            
            # Extract relevant chunks for submission info
            relevant_chunks = await self._get_relevant_chunks(chunks)
            
            # Extract submission information
            extraction_result = await self.llm_interface.extract_submission_info(
                relevant_chunks,
                document_path
            )
            
            # Prepare extracted data
            extracted_data = getattr(extraction_result, 'extracted_data', {})
            if not extracted_data and extraction_result.submission:
                # Convert submission to dict if available
                extracted_data = extraction_result.submission.dict()
            
            processing_time = (datetime.utcnow() - start_time).total_seconds()
            
            return extracted_data, extraction_result.confidence_score, len(chunks)
            
        except Exception as e:
            logger.error(f"Document processing failed: {e}")
            # Return minimal extraction with error info
            return {
                "error": str(e),
                "document": document_path
            }, 0.0, 0
    
    async def _get_relevant_chunks(self, chunks: list) -> List[tuple]:
        """Get relevant chunks for extraction"""
        # Search for chunks related to submission information
        search_queries = [
            "submitter contact information",
            "sample details specimen",
            "sequencing requirements",
            "storage conditions"
        ]
        
        relevant_chunks = []
        seen_content = set()
        
        for query in search_queries:
            try:
                results = await self.vector_store.search(query, k=3)
                for result in results:
                    content = result.get("content", "")
                    score = result.get("score", 0.5)
                    
                    content_hash = hash(content[:100])
                    if content_hash not in seen_content:
                        seen_content.add(content_hash)
                        relevant_chunks.append((content, score))
            except:
                # Fallback to using first chunks
                pass
        
        # If no relevant chunks found, use first chunks
        if not relevant_chunks:
            relevant_chunks = [(chunk.content, 0.7) for chunk in chunks[:10]]
        
        return relevant_chunks


class QualityControlAgent:
    """Agent responsible for data quality validation"""
    
    def __init__(self):
        self.validation_rules = self._load_validation_rules()
    
    def _load_validation_rules(self) -> Dict[str, Any]:
        """Load validation rules"""
        return {
            "required_fields": {
                "administrative": ["submitter_email", "submitter_first_name", "submitter_last_name"],
                "sample": ["sample_id", "sample_type"],
                "sequencing": ["platform"]
            },
            "field_formats": {
                "email": r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$',
                "phone": r'^\+?1?\d{10,14}$',
                "sample_id": r'^[A-Z0-9\-_]+$'
            },
            "valid_values": {
                "sample_type": ["blood", "saliva", "tissue", "dna", "rna", "plasma", "serum"],
                "platform": ["illumina", "pacbio", "nanopore", "ion torrent"],
                "storage_temperature": ["-80", "-20", "4", "RT"]
            }
        }
    
    async def validate_data(
        self, data: Dict[str, Any], confidence_score: float, previous_issues: List[str] = None
    ) -> tuple[Dict[str, Any], float, List[str], List[str], List[str]]:
        """Validate and improve data quality"""
        logger.info("QualityControlAgent: Validating data")
        
        issues_found = []
        corrections_made = []
        recommendations = []
        validated_data = data.copy()
        
        # Skip if error in data
        if "error" in data:
            issues_found.append(f"Extraction error: {data['error']}")
            return validated_data, 0.0, issues_found, corrections_made, recommendations
        
        # Check for missing required fields
        for category, required_fields in self.validation_rules["required_fields"].items():
            if category not in validated_data:
                validated_data[category] = {}
                corrections_made.append(f"Added missing category: {category}")
            
            for field in required_fields:
                if field not in validated_data[category] or not validated_data[category][field]:
                    issues_found.append(f"Missing required field: {category}.{field}")
                    
                    # Try to auto-generate certain fields
                    if field == "sample_id" and category == "sample":
                        sample_id = f"AUTO-{datetime.now().strftime('%Y%m%d%H%M%S')}"
                        validated_data[category][field] = sample_id
                        corrections_made.append(f"Generated sample ID: {sample_id}")
        
        # Validate field formats
        import re
        
        # Email validation
        if "administrative" in validated_data:
            email = validated_data["administrative"].get("submitter_email")
            if email and not re.match(self.validation_rules["field_formats"]["email"], str(email)):
                issues_found.append(f"Invalid email format: {email}")
                recommendations.append("Please provide a valid email address")
        
        # Sample ID validation
        if "sample" in validated_data:
            sample_id = validated_data["sample"].get("sample_id")
            if sample_id and not re.match(self.validation_rules["field_formats"]["sample_id"], str(sample_id)):
                issues_found.append(f"Invalid sample ID format: {sample_id}")
                # Try to fix by converting to uppercase
                fixed_id = str(sample_id).upper().replace(" ", "-")
                validated_data["sample"]["sample_id"] = fixed_id
                corrections_made.append(f"Reformatted sample ID to: {fixed_id}")
        
        # Validate enumerated values
        if "sample" in validated_data:
            sample_type = validated_data["sample"].get("sample_type", "").lower()
            if sample_type and sample_type not in self.validation_rules["valid_values"]["sample_type"]:
                issues_found.append(f"Unknown sample type: {sample_type}")
                recommendations.append(f"Valid sample types: {', '.join(self.validation_rules['valid_values']['sample_type'])}")
        
        # Calculate quality score
        total_checks = len(self.validation_rules["required_fields"]) * 3  # Approximate
        issues_weight = len(issues_found) / total_checks if total_checks > 0 else 0
        quality_score = confidence_score * (1 - issues_weight)
        
        # Add quality improvement recommendations
        if quality_score < 0.7:
            recommendations.append("Consider manual review for low quality score")
        
        if len(corrections_made) > 2:
            recommendations.append("Multiple corrections applied - please verify accuracy")
        
        # Check if issues were resolved from previous attempt
        if previous_issues:
            resolved_issues = [issue for issue in previous_issues if issue not in issues_found]
            if resolved_issues:
                logger.info(f"Resolved {len(resolved_issues)} previous issues")
        
        return validated_data, quality_score, issues_found, corrections_made, recommendations


class StorageAllocationAgent:
    """Agent responsible for sample storage allocation"""
    
    def __init__(self):
        self.storage_zones = self._initialize_storage_zones()
        self.allocation_history = []
    
    def _initialize_storage_zones(self) -> Dict[str, Any]:
        """Initialize storage zone configurations"""
        return {
            "ultra_low_freezer": {
                "temperature": -80,
                "capacity": 1000,
                "used": 0,
                "positions": self._generate_positions("ULF", 10, 10)
            },
            "freezer": {
                "temperature": -20,
                "capacity": 1500,
                "used": 0,
                "positions": self._generate_positions("FRZ", 15, 10)
            },
            "refrigerator": {
                "temperature": 4,
                "capacity": 800,
                "used": 0,
                "positions": self._generate_positions("REF", 8, 10)
            },
            "room_temperature": {
                "temperature": 22,
                "capacity": 500,
                "used": 0,
                "positions": self._generate_positions("RT", 5, 10)
            }
        }
    
    def _generate_positions(self, prefix: str, rows: int, cols: int) -> List[str]:
        """Generate storage positions"""
        positions = []
        for row in range(1, rows + 1):
            for col in range(1, cols + 1):
                positions.append(f"{prefix}-{row:02d}-{chr(64 + col)}")
        return positions
    
    async def allocate_storage(
        self, sample_data: Dict[str, Any], priority: str = "normal"
    ) -> Dict[str, Any]:
        """Allocate storage location for sample"""
        logger.info(f"StorageAllocationAgent: Allocating storage (priority: {priority})")
        
        # Extract storage requirements
        storage_temp = None
        if "storage" in sample_data:
            storage_temp = sample_data["storage"].get("temperature")
        elif "sample" in sample_data:
            storage_temp = sample_data["sample"].get("storage_conditions")
        
        sample_type = sample_data.get("sample", {}).get("sample_type", "unknown")
        
        # Determine appropriate zone
        zone_name = self._determine_zone(storage_temp, sample_type)
        zone = self.storage_zones[zone_name]
        
        # Check capacity
        if zone["used"] >= zone["capacity"]:
            # Try alternative zone
            logger.warning(f"Zone {zone_name} at capacity, finding alternative")
            zone_name = self._find_alternative_zone(zone_name)
            zone = self.storage_zones[zone_name]
        
        # Allocate position
        available_positions = [
            pos for pos in zone["positions"] 
            if not any(h["position"] == pos for h in self.allocation_history)
        ]
        
        if not available_positions:
            raise ValueError(f"No available positions in zone {zone_name}")
        
        # Priority allocation - better positions for high priority
        if priority == "high" and len(available_positions) > 10:
            position = available_positions[0]  # First position
        else:
            position = available_positions[len(available_positions) // 2]  # Middle position
        
        # Update zone usage
        zone["used"] += 1
        
        # Record allocation
        allocation = {
            "location_id": f"LOC-{zone_name.upper()}-{len(self.allocation_history) + 1:04d}",
            "zone": zone_name,
            "position": position,
            "temperature": zone["temperature"],
            "capacity_remaining": (zone["capacity"] - zone["used"]) / zone["capacity"],
            "allocation_time": datetime.utcnow()
        }
        
        self.allocation_history.append({
            **allocation,
            "sample_id": sample_data.get("sample", {}).get("sample_id", "unknown")
        })
        
        return allocation
    
    def _determine_zone(self, storage_temp: str, sample_type: str) -> str:
        """Determine appropriate storage zone"""
        if storage_temp:
            temp_str = str(storage_temp).lower()
            if "-80" in temp_str or "ultra" in temp_str:
                return "ultra_low_freezer"
            elif "-20" in temp_str or "freezer" in temp_str:
                return "freezer"
            elif "4" in temp_str or "refrigerat" in temp_str:
                return "refrigerator"
            elif "room" in temp_str or "rt" in temp_str or "ambient" in temp_str:
                return "room_temperature"
        
        # Default based on sample type
        if sample_type in ["dna", "rna", "tissue"]:
            return "ultra_low_freezer"
        elif sample_type in ["blood", "plasma", "serum"]:
            return "freezer"
        elif sample_type in ["urine", "saliva"]:
            return "refrigerator"
        else:
            return "room_temperature"
    
    def _find_alternative_zone(self, preferred_zone: str) -> str:
        """Find alternative storage zone"""
        # Define zone compatibility
        alternatives = {
            "ultra_low_freezer": ["freezer"],
            "freezer": ["ultra_low_freezer", "refrigerator"],
            "refrigerator": ["freezer", "room_temperature"],
            "room_temperature": ["refrigerator"]
        }
        
        for alt_zone in alternatives.get(preferred_zone, []):
            if self.storage_zones[alt_zone]["used"] < self.storage_zones[alt_zone]["capacity"]:
                return alt_zone
        
        # Return any zone with capacity
        for zone_name, zone in self.storage_zones.items():
            if zone["used"] < zone["capacity"]:
                return zone_name
        
        raise ValueError("All storage zones at capacity")


class NotificationAgent:
    """Agent responsible for sending notifications"""
    
    def __init__(self):
        self.notification_templates = self._load_templates()
        self.notification_history = []
    
    def _load_templates(self) -> Dict[str, str]:
        """Load notification templates"""
        return {
            "submission_complete": """
Subject: Laboratory Submission Complete - {submission_id}

Dear {recipient_name},

Your laboratory submission has been successfully processed.

Submission Details:
- Submission ID: {submission_id}
- Sample ID: {sample_id}
- Storage Location: {storage_location}
- Storage Zone: {storage_zone}
- Quality Score: {quality_score:.2f}

Next Steps:
{next_steps}

Best regards,
Laboratory Management System
            """,
            "quality_issue": """
Subject: Quality Issues Detected - {submission_id}

Dear {recipient_name},

Quality issues were detected in your submission that require attention.

Issues Found:
{issues_list}

Recommendations:
{recommendations_list}

Please contact the laboratory for assistance.

Best regards,
Laboratory Management System
            """,
            "high_priority": """
Subject: URGENT - High Priority Submission - {submission_id}

Dear {recipient_name},

A high priority submission has been processed and requires immediate attention.

Priority Details:
- Submission ID: {submission_id}
- Priority Level: HIGH
- Processing Time: {processing_time}
- Storage Location: {storage_location}

Immediate Action Required:
{action_items}

Best regards,
Laboratory Management System
            """
        }
    
    async def send_notification(
        self, 
        notification_type: str, 
        recipients: List[str], 
        data: Dict[str, Any],
        priority: str = "normal",
        template: Optional[str] = None
    ) -> tuple[str, str, str, List[str]]:
        """Send notification to recipients"""
        logger.info(f"NotificationAgent: Sending {notification_type} notification to {len(recipients)} recipients")
        
        # Generate notification ID
        notification_id = f"NOTIF-{datetime.now().strftime('%Y%m%d%H%M%S')}-{len(self.notification_history) + 1:04d}"
        
        # Select template
        if template:
            notification_content = template
        else:
            notification_content = self.notification_templates.get(
                notification_type,
                "Subject: Laboratory Notification\n\n{content}"
            )
        
        # Format notification content
        try:
            # Prepare data for formatting
            format_data = {
                "submission_id": data.get("submission_id", "N/A"),
                "sample_id": data.get("sample_id", "N/A"),
                "storage_location": data.get("storage_location", "N/A"),
                "storage_zone": data.get("storage_zone", "N/A"),
                "quality_score": data.get("quality_score", 0.0),
                "recipient_name": "Laboratory User",
                "processing_time": f"{data.get('processing_time', 0):.2f} seconds",
                "next_steps": "Please verify the submission details in the system.",
                "action_items": "Review and approve the submission immediately."
            }
            
            # Add issues and recommendations if present
            if "issues" in data:
                format_data["issues_list"] = "\n".join(f"- {issue}" for issue in data["issues"])
            if "recommendations" in data:
                format_data["recommendations_list"] = "\n".join(f"- {rec}" for rec in data["recommendations"])
            
            formatted_content = notification_content.format(**format_data)
        except Exception as e:
            logger.error(f"Error formatting notification: {e}")
            formatted_content = f"Laboratory notification for submission {data.get('submission_id', 'unknown')}"
        
        # Simulate sending notification
        delivery_method = "email"
        if priority == "high":
            delivery_method = "email+sms"
        
        # Filter valid recipients
        valid_recipients = [r for r in recipients if "@" in r]
        if not valid_recipients and recipients:
            valid_recipients = ["admin@laboratory.com"]  # Fallback
        
        # Record notification
        self.notification_history.append({
            "notification_id": notification_id,
            "type": notification_type,
            "recipients": valid_recipients,
            "timestamp": datetime.utcnow(),
            "priority": priority,
            "delivery_method": delivery_method,
            "status": "sent"
        })
        
        # Simulate delivery delay for realism
        await asyncio.sleep(0.1)
        
        status = "sent"
        
        return notification_id, status, delivery_method, valid_recipients


class MultiAgentLabWorkflow(Workflow):
    """
    Multi-agent workflow for laboratory operations.
    
    Coordinates document processing, quality control, storage allocation,
    and notifications through specialized agents.
    """
    
    def __init__(self, timeout: int = 600, verbose: bool = True):
        super().__init__(timeout=timeout, verbose=verbose)
        
        # Initialize agents
        self.document_processor = DocumentProcessorAgent()
        self.quality_controller = QualityControlAgent()
        self.storage_allocator = StorageAllocationAgent()
        self.notifier = NotificationAgent()
        
        # Agent timeouts
        self.agent_timeouts = {
            "document_processor": 120,
            "quality_controller": 30,
            "storage_allocator": 20,
            "notifier": 10
        }
    
    @step(pass_context=True)
    async def receive_document(
        self, ctx: Context, ev: StartEvent
    ) -> DocumentReceivedEvent:
        """Receive and prepare document for processing"""
        
        document_path = getattr(ev, "document_path", None)
        submission_type = getattr(ev, "submission_type", "standard")
        priority = getattr(ev, "priority", "normal")
        metadata = getattr(ev, "metadata", {})
        
        # Initialize workflow context
        ctx.data["submission_id"] = f"SUB-{datetime.now().strftime('%Y%m%d%H%M%S')}"
        ctx.data["start_time"] = datetime.utcnow()
        ctx.data["workflow_log"] = []
        ctx.data["priority"] = priority
        ctx.data["agent_metrics"] = {}
        
        logger.info(f"Starting multi-agent workflow for submission {ctx.data['submission_id']}")
        
        return DocumentReceivedEvent(
            document_path=document_path,
            submission_type=submission_type,
            priority=priority,
            metadata=metadata
        )
    
    @step(pass_context=True)
    async def process_document_agent(
        self, ctx: Context, ev: DocumentReceivedEvent | ExtractionRequestEvent
    ) -> ExtractionResponseEvent | AgentErrorEvent:
        """Document processor agent handles extraction"""
        
        agent_start = datetime.utcnow()
        
        # Log agent activity
        ctx.data["workflow_log"].append({
            "agent": "document_processor",
            "action": "extraction_started",
            "timestamp": agent_start.isoformat()
        })
        
        try:
            # Process with timeout
            document_path = ev.document_path if hasattr(ev, 'document_path') else ctx.data.get("document_path")
            priority = getattr(ev, 'priority', ctx.data.get("priority", "normal"))
            
            process_task = asyncio.create_task(
                self.document_processor.process_document(document_path, priority)
            )
            
            extracted_data, confidence_score, chunk_count = await asyncio.wait_for(
                process_task,
                timeout=self.agent_timeouts["document_processor"]
            )
            
            processing_time = (datetime.utcnow() - agent_start).total_seconds()
            
            # Record metrics
            ctx.data["agent_metrics"]["document_processor"] = {
                "processing_time": processing_time,
                "confidence_score": confidence_score,
                "chunk_count": chunk_count
            }
            
            return ExtractionResponseEvent(
                extracted_data=extracted_data,
                confidence_score=confidence_score,
                source_agent="document_processor",
                processing_time=processing_time,
                chunk_count=chunk_count
            )
            
        except asyncio.TimeoutError:
            logger.error("Document processor timeout")
            return AgentErrorEvent(
                agent_name="document_processor",
                error_message="Processing timeout",
                recoverable=True,
                context={"document_path": document_path}
            )
        except Exception as e:
            logger.error(f"Document processor error: {e}")
            return AgentErrorEvent(
                agent_name="document_processor",
                error_message=str(e),
                recoverable=False,
                context={"document_path": document_path}
            )
    
    @step(pass_context=True)
    async def handle_agent_error(
        self, ctx: Context, ev: AgentErrorEvent
    ) -> ExtractionRequestEvent | StopEvent:
        """Handle agent errors with retry logic"""
        
        logger.warning(f"Agent error from {ev.agent_name}: {ev.error_message}")
        
        # Log error
        ctx.data["workflow_log"].append({
            "agent": ev.agent_name,
            "action": "error",
            "error": ev.error_message,
            "timestamp": datetime.utcnow().isoformat()
        })
        
        if ev.recoverable and ev.agent_name == "document_processor":
            # Retry document processing once
            retry_count = ctx.data.get("retry_count", 0)
            if retry_count < 1:
                ctx.data["retry_count"] = retry_count + 1
                logger.info(f"Retrying document processing (attempt {retry_count + 2})")
                
                return ExtractionRequestEvent(
                    document_path=ev.context.get("document_path", ""),
                    agent="document_processor",
                    retry_count=retry_count + 1
                )
        
        # Non-recoverable or max retries reached
        return StopEvent(
            result={
                "submission_id": ctx.data["submission_id"],
                "status": "failed",
                "error": f"Agent {ev.agent_name} failed: {ev.error_message}",
                "workflow_log": ctx.data["workflow_log"]
            }
        )
    
    @step(pass_context=True)
    async def quality_control_agent(
        self, ctx: Context, ev: ExtractionResponseEvent
    ) -> QualityCheckResponseEvent:
        """Quality control agent validates data"""
        
        agent_start = datetime.utcnow()
        
        # Log agent activity
        ctx.data["workflow_log"].append({
            "agent": "quality_controller",
            "action": "validation_started",
            "timestamp": agent_start.isoformat()
        })
        
        try:
            # Get previous issues if this is a retry
            previous_issues = ctx.data.get("previous_quality_issues", [])
            
            # Validate data with timeout
            validate_task = asyncio.create_task(
                self.quality_controller.validate_data(
                    ev.extracted_data,
                    ev.confidence_score,
                    previous_issues
                )
            )
            
            validated_data, quality_score, issues, corrections, recommendations = await asyncio.wait_for(
                validate_task,
                timeout=self.agent_timeouts["quality_controller"]
            )
            
            processing_time = (datetime.utcnow() - agent_start).total_seconds()
            
            # Store for potential retry
            ctx.data["previous_quality_issues"] = issues
            ctx.data["validated_data"] = validated_data
            ctx.data["quality_score"] = quality_score
            
            # Record metrics
            ctx.data["agent_metrics"]["quality_controller"] = {
                "processing_time": processing_time,
                "quality_score": quality_score,
                "issues_count": len(issues),
                "corrections_count": len(corrections)
            }
            
            return QualityCheckResponseEvent(
                validated_data=validated_data,
                quality_score=quality_score,
                issues_found=issues,
                corrections_made=corrections,
                recommendations=recommendations
            )
            
        except Exception as e:
            logger.error(f"Quality control error: {e}")
            # Return minimal quality check result
            return QualityCheckResponseEvent(
                validated_data=ev.extracted_data,
                quality_score=ev.confidence_score * 0.5,
                issues_found=[f"Quality check error: {str(e)}"],
                corrections_made=[],
                recommendations=["Manual review required"]
            )
    
    @step(pass_context=True)
    async def storage_allocation_agent(
        self, ctx: Context, ev: QualityCheckResponseEvent
    ) -> StorageAllocationResponseEvent:
        """Storage allocation agent assigns storage location"""
        
        agent_start = datetime.utcnow()
        
        # Log agent activity
        ctx.data["workflow_log"].append({
            "agent": "storage_allocator",
            "action": "allocation_started",
            "timestamp": agent_start.isoformat()
        })
        
        # Only allocate storage if quality is acceptable
        if ev.quality_score < 0.3:
            logger.warning("Quality score too low for storage allocation")
            return StorageAllocationResponseEvent(
                location_id="PENDING-REVIEW",
                zone="quality_review",
                position="N/A",
                temperature=0,
                capacity_remaining=1.0,
                allocation_time=datetime.utcnow()
            )
        
        try:
            # Allocate storage with timeout
            allocate_task = asyncio.create_task(
                self.storage_allocator.allocate_storage(
                    ev.validated_data,
                    priority=ctx.data.get("priority", "normal")
                )
            )
            
            allocation = await asyncio.wait_for(
                allocate_task,
                timeout=self.agent_timeouts["storage_allocator"]
            )
            
            processing_time = (datetime.utcnow() - agent_start).total_seconds()
            
            # Store allocation info
            ctx.data["storage_allocation"] = allocation
            
            # Record metrics
            ctx.data["agent_metrics"]["storage_allocator"] = {
                "processing_time": processing_time,
                "zone": allocation["zone"],
                "capacity_remaining": allocation["capacity_remaining"]
            }
            
            return StorageAllocationResponseEvent(**allocation)
            
        except Exception as e:
            logger.error(f"Storage allocation error: {e}")
            # Return error allocation
            return StorageAllocationResponseEvent(
                location_id="ERROR",
                zone="allocation_failed",
                position="N/A",
                temperature=0,
                capacity_remaining=0,
                allocation_time=datetime.utcnow()
            )
    
    @step(pass_context=True)
    async def notification_agent(
        self, ctx: Context, ev: StorageAllocationResponseEvent
    ) -> NotificationSentEvent:
        """Notification agent sends completion notification"""
        
        agent_start = datetime.utcnow()
        
        # Log agent activity
        ctx.data["workflow_log"].append({
            "agent": "notifier",
            "action": "notification_started",
            "timestamp": agent_start.isoformat()
        })
        
        # Prepare notification data
        validated_data = ctx.data.get("validated_data", {})
        quality_score = ctx.data.get("quality_score", 0)
        
        # Extract recipient email
        submitter_email = validated_data.get("administrative", {}).get("submitter_email", "")
        recipients = [submitter_email] if submitter_email else ["laboratory-admin@example.com"]
        
        # Determine notification type
        if quality_score < 0.5:
            notification_type = "quality_issue"
        elif ctx.data.get("priority") == "high":
            notification_type = "high_priority"
        else:
            notification_type = "submission_complete"
        
        notification_data = {
            "submission_id": ctx.data["submission_id"],
            "sample_id": validated_data.get("sample", {}).get("sample_id", "N/A"),
            "storage_location": ev.location_id,
            "storage_zone": ev.zone,
            "quality_score": quality_score,
            "processing_time": (datetime.utcnow() - ctx.data["start_time"]).total_seconds()
        }
        
        # Add issues and recommendations if quality issues exist
        previous_issues = ctx.data.get("previous_quality_issues", [])
        if previous_issues:
            notification_data["issues"] = previous_issues
            notification_data["recommendations"] = ["Please review and correct the identified issues"]
        
        try:
            # Send notification with timeout
            notify_task = asyncio.create_task(
                self.notifier.send_notification(
                    notification_type=notification_type,
                    recipients=recipients,
                    data=notification_data,
                    priority=ctx.data.get("priority", "normal")
                )
            )
            
            notification_id, status, delivery_method, notified_recipients = await asyncio.wait_for(
                notify_task,
                timeout=self.agent_timeouts["notifier"]
            )
            
            processing_time = (datetime.utcnow() - agent_start).total_seconds()
            
            # Record metrics
            ctx.data["agent_metrics"]["notifier"] = {
                "processing_time": processing_time,
                "notification_type": notification_type,
                "recipients_count": len(notified_recipients)
            }
            
            return NotificationSentEvent(
                notification_id=notification_id,
                status=status,
                timestamp=datetime.utcnow(),
                delivery_method=delivery_method,
                recipients_notified=notified_recipients
            )
            
        except Exception as e:
            logger.error(f"Notification error: {e}")
            # Return minimal notification event
            return NotificationSentEvent(
                notification_id="ERROR",
                status="failed",
                timestamp=datetime.utcnow(),
                delivery_method="none",
                recipients_notified=[]
            )
    
    @step(pass_context=True)
    async def finalize_workflow(
        self, ctx: Context, ev: NotificationSentEvent
    ) -> StopEvent:
        """Finalize workflow and prepare summary"""
        
        # Calculate total processing time
        total_time = (datetime.utcnow() - ctx.data["start_time"]).total_seconds()
        
        # Determine overall status
        quality_score = ctx.data.get("quality_score", 0)
        if quality_score >= 0.7 and ev.status == "sent":
            overall_status = "completed"
        elif quality_score >= 0.5:
            overall_status = "completed_with_warnings"
        else:
            overall_status = "requires_review"
        
        # Prepare workflow summary
        summary = {
            "submission_id": ctx.data["submission_id"],
            "status": overall_status,
            "total_processing_time": total_time,
            "quality_score": quality_score,
            "storage_allocation": ctx.data.get("storage_allocation", {}),
            "notification_sent": ev.status == "sent",
            "notification_id": ev.notification_id,
            "recipients_notified": ev.recipients_notified,
            "workflow_log": ctx.data["workflow_log"],
            "agent_metrics": ctx.data["agent_metrics"],
            "agents_involved": [
                "document_processor",
                "quality_controller",
                "storage_allocator",
                "notifier"
            ],
            "priority": ctx.data.get("priority", "normal"),
            "retry_count": ctx.data.get("retry_count", 0)
        }
        
        logger.info(
            f"Multi-agent workflow completed in {total_time:.2f}s with status: {overall_status}"
        )
        
        return StopEvent(result=summary)


# Convenience function for running the multi-agent workflow
async def process_laboratory_submission(
    document_path: str,
    submission_type: str = "standard",
    priority: str = "normal",
    metadata: Dict[str, Any] = None
) -> Dict[str, Any]:
    """Process a laboratory submission using multi-agent workflow"""
    
    workflow = MultiAgentLabWorkflow()
    
    result = await workflow.run(
        document_path=document_path,
        submission_type=submission_type,
        priority=priority,
        metadata=metadata or {}
    )
    
    return result


# Example of custom agent coordination
async def coordinate_agents_example():
    """Example of how agents can coordinate through events"""
    
    # Create workflow with custom timeout
    workflow = MultiAgentLabWorkflow(timeout=1200, verbose=True)
    
    # Process with event streaming for monitoring
    handler = workflow.run(
        document_path="/path/to/document.pdf",
        submission_type="urgent",
        priority="high"
    )
    
    # Stream events as they occur
    async for event in handler.stream_events():
        if hasattr(event, "source_agent"):
            print(f"Agent {event.source_agent} completed its task")
        elif hasattr(event, "agent"):
            print(f"Agent {event.agent} is working...")
    
    # Get final result
    result = await handler
    return result