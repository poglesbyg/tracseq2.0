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
from datetime import datetime
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

logger = logging.getLogger(__name__)


# Define agent events
class DocumentReceivedEvent(Event):
    """New document received for processing"""
    document_path: str
    submission_type: str
    priority: str = "normal"


class ExtractionRequestEvent(Event):
    """Request document extraction"""
    document_path: str
    agent: str = "document_processor"


class ExtractionResponseEvent(Event):
    """Extraction completed"""
    extracted_data: Dict[str, Any]
    confidence_score: float
    source_agent: str


class QualityCheckRequestEvent(Event):
    """Request quality validation"""
    data: Dict[str, Any]
    confidence_score: float
    check_type: str = "standard"


class QualityCheckResponseEvent(Event):
    """Quality check completed"""
    validated_data: Dict[str, Any]
    quality_score: float
    issues_found: List[str]
    corrections_made: List[str]


class StorageAllocationRequestEvent(Event):
    """Request storage allocation"""
    sample_data: Dict[str, Any]
    storage_requirements: Dict[str, Any]


class StorageAllocationResponseEvent(Event):
    """Storage allocation completed"""
    location_id: str
    zone: str
    position: str
    temperature: float
    capacity_remaining: float


class NotificationRequestEvent(Event):
    """Request to send notification"""
    notification_type: str
    recipients: List[str]
    data: Dict[str, Any]
    priority: str = "normal"


class NotificationSentEvent(Event):
    """Notification sent"""
    notification_id: str
    status: str
    timestamp: datetime


class WorkflowCompleteEvent(Event):
    """Workflow completed"""
    submission_id: str
    status: str
    summary: Dict[str, Any]


# Agent definitions
class DocumentProcessorAgent:
    """Agent responsible for document processing and extraction"""
    
    async def process_document(self, document_path: str) -> tuple[Dict[str, Any], float]:
        """Process document and extract information"""
        logger.info(f"DocumentProcessorAgent: Processing {document_path}")
        
        # Simulate document processing
        extracted_data = {
            "administrative": {
                "submitter_name": "Dr. Jane Smith",
                "submitter_email": "jane.smith@lab.com",
                "project_name": "Genomics Study 2024"
            },
            "sample": {
                "sample_id": "GS-2024-001",
                "sample_type": "DNA",
                "volume": "50 µL",
                "concentration": "100 ng/µL",
                "storage_conditions": "-80°C"
            },
            "sequencing": {
                "platform": "Illumina",
                "analysis_type": "WGS",
                "coverage": "30x"
            }
        }
        
        confidence_score = 0.85
        return extracted_data, confidence_score


class QualityControlAgent:
    """Agent responsible for data quality validation"""
    
    async def validate_data(
        self, data: Dict[str, Any], confidence_score: float
    ) -> tuple[Dict[str, Any], float, List[str], List[str]]:
        """Validate and improve data quality"""
        logger.info("QualityControlAgent: Validating data")
        
        issues_found = []
        corrections_made = []
        validated_data = data.copy()
        
        # Check for missing critical fields
        if not validated_data.get("sample", {}).get("sample_id"):
            issues_found.append("Missing sample ID")
            # Ensure the "sample" key exists before setting sample_id
            if "sample" not in validated_data:
                validated_data["sample"] = {}
            validated_data["sample"]["sample_id"] = f"AUTO-{datetime.now().strftime('%Y%m%d%H%M%S')}"
            corrections_made.append("Generated automatic sample ID")
        
        # Validate email format
        email = validated_data.get("administrative", {}).get("submitter_email", "")
        if email and "@" not in email:
            issues_found.append("Invalid email format")
        
        # Calculate quality score
        quality_score = confidence_score
        if not issues_found:
            quality_score = min(confidence_score + 0.1, 1.0)
        else:
            quality_score = confidence_score * 0.9
        
        return validated_data, quality_score, issues_found, corrections_made


class StorageAllocationAgent:
    """Agent responsible for sample storage allocation"""
    
    async def allocate_storage(
        self, sample_data: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Allocate storage location for sample"""
        logger.info("StorageAllocationAgent: Allocating storage")
        
        # Extract storage requirements
        storage_temp = sample_data.get("storage_conditions", "-80°C")
        sample_type = sample_data.get("sample_type", "unknown")
        
        # Simulate storage allocation
        if "-80" in storage_temp:
            zone = "ultra_low_freezer"
            temperature = -80
        elif "-20" in storage_temp:
            zone = "freezer"
            temperature = -20
        elif "4" in storage_temp:
            zone = "refrigerator"
            temperature = 4
        else:
            zone = "room_temperature"
            temperature = 22
        
        allocation = {
            "location_id": f"LOC-{zone.upper()}-001",
            "zone": zone,
            "position": "A1-01",
            "temperature": temperature,
            "capacity_remaining": 0.85
        }
        
        return allocation


class NotificationAgent:
    """Agent responsible for sending notifications"""
    
    async def send_notification(
        self, notification_type: str, recipients: List[str], data: Dict[str, Any]
    ) -> tuple[str, str]:
        """Send notification to recipients"""
        logger.info(f"NotificationAgent: Sending {notification_type} notification")
        
        # Simulate notification sending
        notification_id = f"NOTIF-{datetime.now().strftime('%Y%m%d%H%M%S')}"
        status = "sent"
        
        return notification_id, status


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
    
    @step(pass_context=True)
    async def receive_document(
        self, ctx: Context, ev: StartEvent
    ) -> DocumentReceivedEvent:
        """Receive and prepare document for processing"""
        
        document_path = getattr(ev, "document_path", None)
        submission_type = getattr(ev, "submission_type", "standard")
        priority = getattr(ev, "priority", "normal")
        
        # Initialize workflow context
        ctx.data["submission_id"] = f"SUB-{datetime.now().strftime('%Y%m%d%H%M%S')}"
        ctx.data["start_time"] = datetime.now()
        ctx.data["workflow_log"] = []
        
        logger.info(f"Starting multi-agent workflow for submission {ctx.data['submission_id']}")
        
        return DocumentReceivedEvent(
            document_path=document_path,
            submission_type=submission_type,
            priority=priority
        )
    
    @step(pass_context=True)
    async def process_document_agent(
        self, ctx: Context, ev: DocumentReceivedEvent
    ) -> ExtractionResponseEvent:
        """Document processor agent handles extraction"""
        
        # Log agent activity
        ctx.data["workflow_log"].append({
            "agent": "document_processor",
            "action": "extraction_started",
            "timestamp": datetime.now().isoformat()
        })
        
        # Process document
        extracted_data, confidence_score = await self.document_processor.process_document(
            ev.document_path
        )
        
        return ExtractionResponseEvent(
            extracted_data=extracted_data,
            confidence_score=confidence_score,
            source_agent="document_processor"
        )
    
    @step(pass_context=True)
    async def quality_control_agent(
        self, ctx: Context, ev: ExtractionResponseEvent
    ) -> QualityCheckResponseEvent:
        """Quality control agent validates data"""
        
        # Log agent activity
        ctx.data["workflow_log"].append({
            "agent": "quality_controller",
            "action": "validation_started",
            "timestamp": datetime.now().isoformat()
        })
        
        # Validate data
        validated_data, quality_score, issues, corrections = await self.quality_controller.validate_data(
            ev.extracted_data,
            ev.confidence_score
        )
        
        # Store validated data
        ctx.data["validated_data"] = validated_data
        ctx.data["quality_score"] = quality_score
        
        return QualityCheckResponseEvent(
            validated_data=validated_data,
            quality_score=quality_score,
            issues_found=issues,
            corrections_made=corrections
        )
    
    @step(pass_context=True)
    async def storage_allocation_agent(
        self, ctx: Context, ev: QualityCheckResponseEvent
    ) -> StorageAllocationResponseEvent:
        """Storage allocation agent assigns storage location"""
        
        # Log agent activity
        ctx.data["workflow_log"].append({
            "agent": "storage_allocator",
            "action": "allocation_started",
            "timestamp": datetime.now().isoformat()
        })
        
        # Only allocate storage if quality is acceptable
        if ev.quality_score < 0.5:
            logger.warning("Quality score too low for storage allocation")
            return StorageAllocationResponseEvent(
                location_id="PENDING",
                zone="quality_review",
                position="N/A",
                temperature=0,
                capacity_remaining=1.0
            )
        
        # Get sample data
        sample_data = ev.validated_data.get("sample", {})
        
        # Allocate storage
        allocation = await self.storage_allocator.allocate_storage(sample_data)
        
        # Store allocation info
        ctx.data["storage_allocation"] = allocation
        
        return StorageAllocationResponseEvent(**allocation)
    
    @step(pass_context=True)
    async def notification_agent(
        self, ctx: Context, ev: StorageAllocationResponseEvent
    ) -> NotificationSentEvent:
        """Notification agent sends completion notification"""
        
        # Log agent activity
        ctx.data["workflow_log"].append({
            "agent": "notifier",
            "action": "notification_started",
            "timestamp": datetime.now().isoformat()
        })
        
        # Prepare notification data
        validated_data = ctx.data.get("validated_data", {})
        submitter_email = validated_data.get("administrative", {}).get("submitter_email", "")
        
        notification_data = {
            "submission_id": ctx.data["submission_id"],
            "sample_id": validated_data.get("sample", {}).get("sample_id"),
            "storage_location": ev.location_id,
            "storage_zone": ev.zone,
            "quality_score": ctx.data.get("quality_score", 0)
        }
        
        # Send notification
        notification_id, status = await self.notifier.send_notification(
            notification_type="submission_complete",
            recipients=[submitter_email] if submitter_email else ["admin@lab.com"],
            data=notification_data
        )
        
        return NotificationSentEvent(
            notification_id=notification_id,
            status=status,
            timestamp=datetime.now()
        )
    
    @step(pass_context=True)
    async def finalize_workflow(
        self, ctx: Context, ev: NotificationSentEvent
    ) -> StopEvent:
        """Finalize workflow and prepare summary"""
        
        # Calculate total processing time
        total_time = (datetime.now() - ctx.data["start_time"]).total_seconds()
        
        # Prepare workflow summary
        summary = {
            "submission_id": ctx.data["submission_id"],
            "status": "completed",
            "total_processing_time": total_time,
            "quality_score": ctx.data.get("quality_score", 0),
            "storage_allocation": ctx.data.get("storage_allocation", {}),
            "notification_sent": ev.status == "sent",
            "workflow_log": ctx.data["workflow_log"],
            "agents_involved": [
                "document_processor",
                "quality_controller",
                "storage_allocator",
                "notifier"
            ]
        }
        
        logger.info(f"Multi-agent workflow completed in {total_time:.2f}s")
        
        return StopEvent(result=summary)


# Convenience function for running the multi-agent workflow
async def process_laboratory_submission(
    document_path: str,
    submission_type: str = "standard",
    priority: str = "normal"
) -> Dict[str, Any]:
    """Process a laboratory submission using multi-agent workflow"""
    
    workflow = MultiAgentLabWorkflow()
    
    result = await workflow.run(
        document_path=document_path,
        submission_type=submission_type,
        priority=priority
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