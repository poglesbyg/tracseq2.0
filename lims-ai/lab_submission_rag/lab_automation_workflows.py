#!/usr/bin/env python3
"""
Laboratory Document Processing Automation Workflows
Automated processing of uploaded documents with customizable workflows
"""

import asyncio
import json
import logging
import shutil
import uuid
from collections.abc import Callable
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from enum import Enum
from pathlib import Path
from typing import Any

from watchdog.events import FileSystemEventHandler
from watchdog.observers import Observer

from custom_lab_categories import LabCategoryConfig
from improved_lab_rag import ExtractionResult, ImprovedLabRAG

# Setup logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


class WorkflowStatus(str, Enum):
    """Status of workflow execution"""

    PENDING = "pending"
    PROCESSING = "processing"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


class ProcessingPriority(str, Enum):
    """Processing priority levels"""

    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    URGENT = "urgent"


@dataclass
class ProcessingJob:
    """A document processing job"""

    job_id: str = field(default_factory=lambda: str(uuid.uuid4()))
    file_path: str = ""
    original_filename: str = ""
    status: WorkflowStatus = WorkflowStatus.PENDING
    priority: ProcessingPriority = ProcessingPriority.MEDIUM
    created_at: datetime = field(default_factory=datetime.now)
    started_at: datetime | None = None
    completed_at: datetime | None = None
    result: ExtractionResult | None = None
    error_message: str | None = None
    retry_count: int = 0
    max_retries: int = 3

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for JSON serialization"""
        return {
            "job_id": self.job_id,
            "file_path": self.file_path,
            "original_filename": self.original_filename,
            "status": self.status.value,
            "priority": self.priority.value,
            "created_at": self.created_at.isoformat(),
            "started_at": self.started_at.isoformat() if self.started_at else None,
            "completed_at": self.completed_at.isoformat() if self.completed_at else None,
            "error_message": self.error_message,
            "retry_count": self.retry_count,
            "processing_time": (
                (self.completed_at - self.started_at).total_seconds()
                if self.started_at and self.completed_at
                else None
            ),
        }


class DocumentWatcher(FileSystemEventHandler):
    """File system watcher for automatic document processing"""

    def __init__(self, automation_manager) -> None:
        self.automation_manager = automation_manager
        self.supported_extensions = {".txt", ".pdf", ".docx", ".doc", ".rtf"}

    def on_created(self, event) -> None:
        """Handle new file creation"""
        if not event.is_directory:
            file_path = Path(event.src_path)
            if file_path.suffix.lower() in self.supported_extensions:
                logger.info(f"📄 New document detected: {file_path.name}")
                # Create task and store reference to prevent garbage collection
                task = asyncio.create_task(
                    self.automation_manager.queue_document_for_processing(str(file_path))
                )
                self.automation_manager.background_tasks.add(task)
                task.add_done_callback(self.automation_manager.background_tasks.discard)

    def on_moved(self, event) -> None:
        """Handle file moves (like drag & drop)"""
        if not event.is_directory:
            file_path = Path(event.dest_path)
            if file_path.suffix.lower() in self.supported_extensions:
                logger.info(f"📄 Document moved to watch folder: {file_path.name}")
                # Create task and store reference to prevent garbage collection
                task = asyncio.create_task(
                    self.automation_manager.queue_document_for_processing(str(file_path))
                )
                self.automation_manager.background_tasks.add(task)
                task.add_done_callback(self.automation_manager.background_tasks.discard)


class LabAutomationManager:
    """Main automation manager for lab document processing"""

    def __init__(self, config: LabCategoryConfig | None = None) -> None:
        self.config = config or LabCategoryConfig()
        self.rag_system = ImprovedLabRAG()

        # Directory structure
        self.base_dir = Path("automation")
        self.inbox_dir = self.base_dir / "inbox"
        self.processing_dir = self.base_dir / "processing"
        self.completed_dir = self.base_dir / "completed"
        self.failed_dir = self.base_dir / "failed"
        self.archive_dir = self.base_dir / "archive"

        # Create directories
        for dir_path in [
            self.inbox_dir,
            self.processing_dir,
            self.completed_dir,
            self.failed_dir,
            self.archive_dir,
        ]:
            dir_path.mkdir(parents=True, exist_ok=True)

        # Job management
        self.job_queue: list[ProcessingJob] = []
        self.active_jobs: dict[str, ProcessingJob] = {}
        self.completed_jobs: list[ProcessingJob] = []

        # Processing settings
        self.max_concurrent_jobs = 3
        self.check_interval = 5  # seconds
        self.auto_archive_days = 30

        # File watcher
        self.observer = Observer()
        self.watcher = DocumentWatcher(self)

        # Task management - store references to prevent garbage collection
        self.background_tasks: set = set()

        # Callbacks for custom processing
        self.pre_processing_callbacks: list[Callable] = []
        self.post_processing_callbacks: list[Callable] = []

    async def start_automation(self) -> None:
        """Start the automation system"""
        logger.info("🚀 Starting Laboratory Document Automation System")

        # Start file watcher
        self.observer.schedule(self.watcher, str(self.inbox_dir), recursive=False)
        self.observer.start()
        logger.info(f"📁 Watching for documents in: {self.inbox_dir}")

        # Start processing loop with proper task management
        processing_task = asyncio.create_task(self._processing_loop())
        self.background_tasks.add(processing_task)
        processing_task.add_done_callback(self.background_tasks.discard)
        logger.info("⚙️ Processing loop started")

        # Start cleanup task with proper task management
        cleanup_task = asyncio.create_task(self._cleanup_loop())
        self.background_tasks.add(cleanup_task)
        cleanup_task.add_done_callback(self.background_tasks.discard)
        logger.info("🧹 Cleanup task started")

        print("\n" + "=" * 60)
        print("🧬 LAB AUTOMATION SYSTEM READY")
        print("=" * 60)
        print(f"📥 Drop documents in: {self.inbox_dir}")
        print(f"⚙️ Max concurrent jobs: {self.max_concurrent_jobs}")
        print(f"🔄 Check interval: {self.check_interval}s")
        print(f"📊 Categories configured: {len(self.config.categories)}")
        print("=" * 60)

    async def stop_automation(self) -> None:
        """Stop the automation system"""
        logger.info("🛑 Stopping automation system...")
        self.observer.stop()
        self.observer.join()

        # Cancel all background tasks
        for task in self.background_tasks.copy():
            if not task.done():
                task.cancel()
                try:
                    await task
                except asyncio.CancelledError:
                    pass

        self.background_tasks.clear()

        # Wait for active jobs to complete
        while self.active_jobs:
            logger.info(f"⏳ Waiting for {len(self.active_jobs)} active jobs to complete...")
            await asyncio.sleep(2)

        logger.info("✅ Automation system stopped")

    async def queue_document_for_processing(
        self, file_path: str, priority: ProcessingPriority = ProcessingPriority.MEDIUM
    ):
        """Add a document to the processing queue"""
        try:
            file_path_obj = Path(file_path)

            # Move file to processing directory
            processing_path = self.processing_dir / f"{uuid.uuid4()}_{file_path_obj.name}"
            shutil.move(str(file_path_obj), str(processing_path))

            # Create processing job
            job = ProcessingJob(
                file_path=str(processing_path),
                original_filename=file_path_obj.name,
                priority=priority,
            )

            # Add to queue (sort by priority)
            self.job_queue.append(job)
            self.job_queue.sort(
                key=lambda x: {
                    ProcessingPriority.URGENT: 0,
                    ProcessingPriority.HIGH: 1,
                    ProcessingPriority.MEDIUM: 2,
                    ProcessingPriority.LOW: 3,
                }[x.priority]
            )

            logger.info(
                f"📋 Queued job {job.job_id[:8]} for '{job.original_filename}' (Priority: {priority.value})"
            )
            await self._save_job_status()

        except Exception as e:
            logger.error(f"❌ Failed to queue document {file_path}: {e}")

    async def _processing_loop(self) -> None:
        """Main processing loop"""
        while True:
            try:
                # Process queued jobs
                while len(self.active_jobs) < self.max_concurrent_jobs and self.job_queue:

                    job = self.job_queue.pop(0)
                    # Create task and store reference to prevent garbage collection
                    task = asyncio.create_task(self._process_job(job))
                    self.background_tasks.add(task)
                    task.add_done_callback(self.background_tasks.discard)

                await asyncio.sleep(self.check_interval)

            except Exception as e:
                logger.error(f"❌ Error in processing loop: {e}")
                await asyncio.sleep(10)

    async def _process_job(self, job: ProcessingJob) -> None:
        """Process a single job"""
        job.started_at = datetime.now()
        job.status = WorkflowStatus.PROCESSING
        self.active_jobs[job.job_id] = job

        logger.info(f"🔄 Processing job {job.job_id[:8]}: {job.original_filename}")

        try:
            # Pre-processing callbacks
            for callback in self.pre_processing_callbacks:
                await callback(job)

            # Process document with RAG system
            result = await self.rag_system.process_document(job.file_path)

            job.result = result

            if result.success:
                job.status = WorkflowStatus.COMPLETED
                job.completed_at = datetime.now()

                # Move to completed directory
                completed_path = self.completed_dir / Path(job.file_path).name
                shutil.move(job.file_path, str(completed_path))
                job.file_path = str(completed_path)

                logger.info(f"✅ Completed job {job.job_id[:8]} in {result.processing_time:.2f}s")

                # Post-processing callbacks
                for callback in self.post_processing_callbacks:
                    await callback(job)

            else:
                raise Exception(f"Processing failed: {result.warnings}")

        except Exception as e:
            job.error_message = str(e)
            job.retry_count += 1

            if job.retry_count < job.max_retries:
                # Retry job
                job.status = WorkflowStatus.PENDING
                self.job_queue.append(job)
                logger.warning(
                    f"⚠️ Job {job.job_id[:8]} failed, retrying ({job.retry_count}/{job.max_retries})"
                )
            else:
                # Job failed permanently
                job.status = WorkflowStatus.FAILED
                job.completed_at = datetime.now()

                # Move to failed directory
                failed_path = self.failed_dir / Path(job.file_path).name
                shutil.move(job.file_path, str(failed_path))
                job.file_path = str(failed_path)

                logger.error(f"❌ Job {job.job_id[:8]} failed permanently: {e}")

        finally:
            # Remove from active jobs
            if job.job_id in self.active_jobs:
                del self.active_jobs[job.job_id]

            # Add to completed jobs
            self.completed_jobs.append(job)
            await self._save_job_status()

    async def _cleanup_loop(self) -> None:
        """Periodic cleanup of old files and jobs"""
        while True:
            try:
                await asyncio.sleep(3600)  # Run every hour

                cutoff_date = datetime.now() - timedelta(days=self.auto_archive_days)

                # Archive old completed jobs
                archived_count = 0
                remaining_jobs = []

                for job in self.completed_jobs:
                    if job.completed_at and job.completed_at < cutoff_date:
                        # Move file to archive
                        if Path(job.file_path).exists():
                            archive_path = self.archive_dir / Path(job.file_path).name
                            shutil.move(job.file_path, str(archive_path))
                            archived_count += 1
                    else:
                        remaining_jobs.append(job)

                self.completed_jobs = remaining_jobs

                if archived_count > 0:
                    logger.info(f"📦 Archived {archived_count} old processed documents")

            except Exception as e:
                logger.error(f"❌ Error in cleanup loop: {e}")

    async def _save_job_status(self) -> None:
        """Save job status to file"""
        try:
            status_file = self.base_dir / "job_status.json"

            status_data = {
                "queued_jobs": [job.to_dict() for job in self.job_queue],
                "active_jobs": [job.to_dict() for job in self.active_jobs.values()],
                "completed_jobs": [
                    job.to_dict() for job in self.completed_jobs[-50:]
                ],  # Keep last 50
            }

            with open(status_file, "w") as f:
                json.dump(status_data, f, indent=2, default=str)

        except Exception as e:
            logger.warning(f"⚠️ Could not save job status: {e}")

    def get_system_status(self) -> dict[str, Any]:
        """Get current system status"""
        return {
            "system_status": "running",
            "queued_jobs": len(self.job_queue),
            "active_jobs": len(self.active_jobs),
            "completed_jobs": len(self.completed_jobs),
            "total_processed": len(self.completed_jobs),
            "success_rate": (
                len([j for j in self.completed_jobs if j.status == WorkflowStatus.COMPLETED])
                / max(len(self.completed_jobs), 1)
            ),
            "directories": {
                "inbox": str(self.inbox_dir),
                "processing": str(self.processing_dir),
                "completed": str(self.completed_dir),
                "failed": str(self.failed_dir),
                "archive": str(self.archive_dir),
            },
            "configuration": {
                "max_concurrent_jobs": self.max_concurrent_jobs,
                "categories": len(self.config.categories),
                "auto_archive_days": self.auto_archive_days,
            },
        }

    def add_pre_processing_callback(self, callback: Callable) -> None:
        """Add callback to run before processing each document"""
        self.pre_processing_callbacks.append(callback)

    def add_post_processing_callback(self, callback: Callable) -> None:
        """Add callback to run after processing each document"""
        self.post_processing_callbacks.append(callback)


# Example callbacks
async def email_notification_callback(job: ProcessingJob):
    """Example callback to send email notifications"""
    if job.status == WorkflowStatus.COMPLETED and job.result and job.result.submission:
        submission = job.result.submission
        logger.info(
            f"📧 Would send email to {submission.submitter_email} about job {job.job_id[:8]}"
        )


async def lab_manager_integration_callback(job: ProcessingJob):
    """Example callback for deeper lab_manager integration"""
    if job.status == WorkflowStatus.COMPLETED and job.result and job.result.submission:
        logger.info(f"🔗 Would create workflow in lab_manager for job {job.job_id[:8]}")


# Testing and demo
async def test_automation_system() -> None:
    """Test the automation system"""
    print("🧬 Testing Laboratory Automation System")
    print("=" * 50)

    # Create automation manager
    automation = LabAutomationManager()

    # Add callbacks
    automation.add_post_processing_callback(email_notification_callback)
    automation.add_post_processing_callback(lab_manager_integration_callback)

    print("📁 Created automation directories:")
    print(f"   Inbox: {automation.inbox_dir}")
    print(f"   Processing: {automation.processing_dir}")
    print(f"   Completed: {automation.completed_dir}")

    # Create test document in inbox
    test_doc_content = """
Laboratory Sample Submission

Submitter: Dr. Test User
Email: test@lab.edu
Phone: (555) 000-0000
Institution: Test Laboratory
Project: Automation Test

Sample: TEST_001
Barcode: AUTO_TEST_001
Material: DNA
Concentration: 50 ng/uL
Volume: 100 uL

Storage: -80C Freezer
Platform: Illumina
Analysis: WES
Coverage: 50x
Priority: High
"""

    test_file = automation.inbox_dir / "test_automation.txt"
    test_file.write_text(test_doc_content)
    print(f"📄 Created test document: {test_file.name}")

    # Start automation (this would run continuously in production)
    print("\n🚀 Starting automation system...")
    await automation.start_automation()

    # Wait for processing
    print("⏳ Waiting for document processing...")
    await asyncio.sleep(10)

    # Check status
    status = automation.get_system_status()
    print("\n📊 System Status:")
    print(f"   Queued: {status['queued_jobs']}")
    print(f"   Active: {status['active_jobs']}")
    print(f"   Completed: {status['completed_jobs']}")

    # Stop automation
    await automation.stop_automation()
    print("\n✅ Automation test completed!")


if __name__ == "__main__":
    asyncio.run(test_automation_system())
