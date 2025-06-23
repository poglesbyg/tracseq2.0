"""
Repository for laboratory submission database operations
"""

import logging
from datetime import datetime
from typing import Any, Dict, List, Optional

from sqlalchemy import delete, func, or_, select, update
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from models.database import (
    DocumentChunkDB,
    DocumentDB,
    ExtractionResultDB,
    LabSubmissionDB,
    QueryLogDB,
    SampleDB,
)
from models.submission import LabSubmission

logger = logging.getLogger(__name__)


class SubmissionRepository:
    """Repository for laboratory submission operations"""

    def __init__(self, session: AsyncSession):
        self.session = session

    async def create_submission(self, submission: LabSubmission) -> LabSubmissionDB:
        """Create a new laboratory submission"""
        try:
            db_submission = LabSubmissionDB(
                submission_id=submission.submission_id,
                client_id=submission.client_id,
                client_name=submission.client_name,
                client_email=submission.client_email,
                sample_type=submission.sample_type,
                sample_count=submission.sample_count,
                sample_volume=submission.sample_volume,
                storage_condition=submission.storage_condition,
                processing_requirements=submission.processing_requirements,
                special_instructions=submission.special_instructions,
                submission_date=submission.submission_date,
                status=submission.status,
                priority=submission.priority,
                meta_data=submission.metadata,
            )

            self.session.add(db_submission)
            await self.session.flush()
            await self.session.refresh(db_submission)

            logger.info(f"Created submission: {submission.submission_id}")
            return db_submission

        except Exception as e:
            logger.error(f"Error creating submission: {e}")
            raise

    async def get_submission(self, submission_id: str) -> Optional[LabSubmissionDB]:
        """Get a submission by ID"""
        try:
            result = await self.session.execute(
                select(LabSubmissionDB)
                .options(
                    selectinload(LabSubmissionDB.samples),
                    selectinload(LabSubmissionDB.documents),
                    selectinload(LabSubmissionDB.extractions),
                )
                .where(LabSubmissionDB.submission_id == submission_id)
            )
            return result.scalar_one_or_none()
        except Exception as e:
            logger.error(f"Error getting submission {submission_id}: {e}")
            raise

    async def get_submissions(
        self,
        limit: int = 100,
        offset: int = 0,
        client_id: Optional[str] = None,
        status: Optional[str] = None,
        sample_type: Optional[str] = None,
    ) -> List[LabSubmissionDB]:
        """Get submissions with optional filtering"""
        try:
            query = select(LabSubmissionDB).options(selectinload(LabSubmissionDB.samples))

            # Apply filters
            if client_id:
                query = query.where(LabSubmissionDB.client_id == client_id)
            if status:
                query = query.where(LabSubmissionDB.status == status)
            if sample_type:
                query = query.where(LabSubmissionDB.sample_type == sample_type)

            query = query.offset(offset).limit(limit).order_by(LabSubmissionDB.created_at.desc())

            result = await self.session.execute(query)
            return result.scalars().all()

        except Exception as e:
            logger.error(f"Error getting submissions: {e}")
            raise

    async def update_submission_status(self, submission_id: str, status: str) -> bool:
        """Update submission status"""
        try:
            result = await self.session.execute(
                update(LabSubmissionDB)
                .where(LabSubmissionDB.submission_id == submission_id)
                .values(status=status, updated_at=datetime.utcnow())
            )
            return result.rowcount > 0
        except Exception as e:
            logger.error(f"Error updating submission status: {e}")
            raise

    async def delete_submission(self, submission_id: str) -> bool:
        """Delete a submission and all related data"""
        try:
            result = await self.session.execute(
                delete(LabSubmissionDB).where(LabSubmissionDB.submission_id == submission_id)
            )
            return result.rowcount > 0
        except Exception as e:
            logger.error(f"Error deleting submission: {e}")
            raise

    async def create_sample(self, sample_data: Dict[str, Any]) -> SampleDB:
        """Create a new sample"""
        try:
            db_sample = SampleDB(**sample_data)
            self.session.add(db_sample)
            await self.session.flush()
            await self.session.refresh(db_sample)

            logger.info(f"Created sample: {sample_data.get('sample_id')}")
            return db_sample

        except Exception as e:
            logger.error(f"Error creating sample: {e}")
            raise

    async def get_samples(
        self,
        submission_id: Optional[str] = None,
        patient_id: Optional[str] = None,
        sample_type: Optional[str] = None,
        limit: int = 100,
        offset: int = 0,
    ) -> List[SampleDB]:
        """Get samples with optional filtering"""
        try:
            query = select(SampleDB).options(
                selectinload(SampleDB.pooling_info),
                selectinload(SampleDB.sequence_generation),
                selectinload(SampleDB.informatics_info),
            )

            # Apply filters
            if submission_id:
                query = query.where(SampleDB.submission_id == submission_id)
            if patient_id:
                query = query.where(SampleDB.patient_id == patient_id)
            if sample_type:
                query = query.where(SampleDB.source_type == sample_type)

            query = query.offset(offset).limit(limit).order_by(SampleDB.created_at.desc())

            result = await self.session.execute(query)
            return result.scalars().all()

        except Exception as e:
            logger.error(f"Error getting samples: {e}")
            raise

    async def get_sample_count(
        self,
        submission_id: Optional[str] = None,
        sample_type: Optional[str] = None,
        storage_condition: Optional[str] = None,
    ) -> int:
        """Get count of samples with optional filtering"""
        try:
            query = select(func.count(SampleDB.id))

            # Apply filters
            if submission_id:
                query = query.where(SampleDB.submission_id == submission_id)
            if sample_type:
                query = query.where(SampleDB.source_type == sample_type)
            if storage_condition:
                query = query.where(SampleDB.storage_conditions == storage_condition)

            result = await self.session.execute(query)
            return result.scalar()

        except Exception as e:
            logger.error(f"Error getting sample count: {e}")
            raise

    async def get_sample_statistics(self) -> Dict[str, Any]:
        """Get comprehensive sample statistics"""
        try:
            # Total samples
            total_samples = await self.session.execute(select(func.count(SampleDB.id)))
            total_count = total_samples.scalar()

            # Samples by type
            samples_by_type = await self.session.execute(
                select(SampleDB.source_type, func.count(SampleDB.id)).group_by(SampleDB.source_type)
            )
            type_counts = dict(samples_by_type.all())

            # Samples by storage condition
            samples_by_storage = await self.session.execute(
                select(SampleDB.storage_conditions, func.count(SampleDB.id)).group_by(
                    SampleDB.storage_conditions
                )
            )
            storage_counts = dict(samples_by_storage.all())

            # Samples by priority
            samples_by_priority = await self.session.execute(
                select(SampleDB.priority, func.count(SampleDB.id)).group_by(SampleDB.priority)
            )
            priority_counts = dict(samples_by_priority.all())

            return {
                "total_samples": total_count,
                "by_type": type_counts,
                "by_storage_condition": storage_counts,
                "by_priority": priority_counts,
            }

        except Exception as e:
            logger.error(f"Error getting sample statistics: {e}")
            raise

    async def create_document(self, document_data: Dict[str, Any]) -> DocumentDB:
        """Create a new document record"""
        try:
            db_document = DocumentDB(**document_data)
            self.session.add(db_document)
            await self.session.flush()
            await self.session.refresh(db_document)

            logger.info(f"Created document: {document_data.get('document_id')}")
            return db_document

        except Exception as e:
            logger.error(f"Error creating document: {e}")
            raise

    async def create_document_chunk(self, chunk_data: Dict[str, Any]) -> DocumentChunkDB:
        """Create a new document chunk"""
        try:
            db_chunk = DocumentChunkDB(**chunk_data)
            self.session.add(db_chunk)
            await self.session.flush()
            await self.session.refresh(db_chunk)

            return db_chunk

        except Exception as e:
            logger.error(f"Error creating document chunk: {e}")
            raise

    async def create_extraction_result(self, extraction_data: Dict[str, Any]) -> ExtractionResultDB:
        """Create a new extraction result"""
        try:
            db_extraction = ExtractionResultDB(**extraction_data)
            self.session.add(db_extraction)
            await self.session.flush()
            await self.session.refresh(db_extraction)

            logger.info(f"Created extraction result: {extraction_data.get('extraction_id')}")
            return db_extraction

        except Exception as e:
            logger.error(f"Error creating extraction result: {e}")
            raise

    async def log_query(self, query_data: Dict[str, Any]) -> QueryLogDB:
        """Log a query for analytics"""
        try:
            db_query_log = QueryLogDB(**query_data)
            self.session.add(db_query_log)
            await self.session.flush()
            await self.session.refresh(db_query_log)

            return db_query_log

        except Exception as e:
            logger.error(f"Error logging query: {e}")
            raise

    async def search_samples(self, search_term: str, limit: int = 50) -> List[SampleDB]:
        """Search samples by various fields"""
        try:
            search_pattern = f"%{search_term}%"

            query = (
                select(SampleDB)
                .where(
                    or_(
                        SampleDB.sample_id.ilike(search_pattern),
                        SampleDB.sample_name.ilike(search_pattern),
                        SampleDB.patient_id.ilike(search_pattern),
                        SampleDB.notes.ilike(search_pattern),
                        SampleDB.container_id.ilike(search_pattern),
                        SampleDB.container_barcode.ilike(search_pattern),
                    )
                )
                .limit(limit)
            )

            result = await self.session.execute(query)
            return result.scalars().all()

        except Exception as e:
            logger.error(f"Error searching samples: {e}")
            raise
