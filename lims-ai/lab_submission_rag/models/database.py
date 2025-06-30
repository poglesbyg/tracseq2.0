"""
SQLAlchemy database models for laboratory submission system
Compatible with existing lab_manager database
"""

import enum

from sqlalchemy import (
    JSON,
    Boolean,
    Column,
    DateTime,
    Float,
    ForeignKey,
    Integer,
    String,
    Text,
)
from sqlalchemy import (
    Enum as SQLEnum,
)
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import relationship
from sqlalchemy.sql import func

from config import settings

Base = declarative_base()

# Use table prefix from settings to avoid conflicts
TABLE_PREFIX = getattr(settings, "table_prefix", "rag_")


class SampleTypeEnum(enum.Enum):
    BLOOD = "blood"
    SALIVA = "saliva"
    TISSUE = "tissue"
    URINE = "urine"
    DNA = "dna"
    RNA = "rna"
    OTHER = "other"
    SWAB = "swab"


class AnalysisTypeEnum(enum.Enum):
    WGS = "wgs"
    WES = "wes"
    TARGETED_PANEL = "targeted_panel"
    RNA_SEQ = "rna_seq"
    CHIP_SEQ = "chip_seq"
    ATAC_SEQ = "atac_seq"
    OTHER = "other"


class StorageConditionEnum(enum.Enum):
    ROOM_TEMP = "room_temperature"
    REFRIGERATED = "refrigerated"
    FROZEN = "frozen"
    CRYOGENIC = "cryogenic"


class ProcessingStatusEnum(enum.Enum):
    RECEIVED = "received"
    IN_PROCESS = "in_process"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


class PriorityLevelEnum(enum.Enum):
    HIGH = "high"
    MEDIUM = "medium"
    LOW = "low"


class LabSubmissionDB(Base):
    """Database model for laboratory submissions (RAG system)"""

    __tablename__ = f"{TABLE_PREFIX}lab_submissions"

    id = Column(Integer, primary_key=True, index=True)
    submission_id = Column(String(255), unique=True, index=True, nullable=False)
    client_id = Column(String(255), nullable=False, index=True)
    client_name = Column(String(255), nullable=False)
    client_email = Column(String(255), nullable=False)

    # Sample information
    sample_type = Column(SQLEnum(SampleTypeEnum), nullable=False)
    sample_count = Column(Integer, nullable=False)
    sample_volume = Column(Float, nullable=True)
    storage_condition = Column(SQLEnum(StorageConditionEnum), nullable=False)

    # Processing requirements
    processing_requirements = Column(JSON, default=list)
    special_instructions = Column(Text, nullable=True)

    # Administrative tracking
    submission_date = Column(DateTime(timezone=True), default=func.now())
    status = Column(SQLEnum(ProcessingStatusEnum), default=ProcessingStatusEnum.RECEIVED)
    priority = Column(Integer, default=1)

    # Metadata
    meta_data = Column(JSON, default=dict)
    created_at = Column(DateTime(timezone=True), server_default=func.now())
    updated_at = Column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())

    # Relationships
    samples = relationship("SampleDB", back_populates="submission", cascade="all, delete-orphan")
    documents = relationship(
        "DocumentDB", back_populates="submission", cascade="all, delete-orphan"
    )
    extractions = relationship(
        "ExtractionResultDB", back_populates="submission", cascade="all, delete-orphan"
    )


class SampleDB(Base):
    """Database model for individual samples (RAG system)"""

    __tablename__ = f"{TABLE_PREFIX}samples"

    id = Column(Integer, primary_key=True, index=True)
    sample_id = Column(String(255), unique=True, index=True, nullable=False)
    submission_id = Column(
        String(255), ForeignKey(f"{TABLE_PREFIX}lab_submissions.submission_id"), nullable=False
    )

    # Sample details
    patient_id = Column(String(255), nullable=True, index=True)
    sample_name = Column(String(255), nullable=True)
    priority = Column(SQLEnum(PriorityLevelEnum), default=PriorityLevelEnum.MEDIUM)
    quality_score = Column(Float, nullable=True)
    purity_ratio = Column(Float, nullable=True)
    integrity_number = Column(Float, nullable=True)
    notes = Column(Text, nullable=True)
    special_instructions = Column(Text, nullable=True)

    # Source material information
    source_type = Column(SQLEnum(SampleTypeEnum), nullable=True)
    collection_date = Column(DateTime(timezone=True), nullable=True)
    collection_method = Column(String(255), nullable=True)
    source_organism = Column(String(255), nullable=True)
    tissue_type = Column(String(255), nullable=True)
    preservation_method = Column(String(255), nullable=True)
    storage_conditions = Column(String(255), nullable=True)

    # Container information
    container_type = Column(String(255), nullable=True)
    container_id = Column(String(255), nullable=True)
    volume = Column(Float, nullable=True)
    concentration = Column(Float, nullable=True)
    diluent_used = Column(String(255), nullable=True)
    storage_temperature = Column(String(255), nullable=True)
    container_barcode = Column(String(255), nullable=True)

    # Timestamps
    created_at = Column(DateTime(timezone=True), server_default=func.now())
    updated_at = Column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())

    # Relationships
    submission = relationship("LabSubmissionDB", back_populates="samples")
    pooling_info = relationship("PoolingInfoDB", back_populates="sample", uselist=False)
    sequence_generation = relationship(
        "SequenceGenerationDB", back_populates="sample", uselist=False
    )
    informatics_info = relationship("InformaticsInfoDB", back_populates="sample", uselist=False)


class PoolingInfoDB(Base):
    """Database model for pooling information"""

    __tablename__ = f"{TABLE_PREFIX}pooling_info"

    id = Column(Integer, primary_key=True, index=True)
    sample_id = Column(String(255), ForeignKey(f"{TABLE_PREFIX}samples.sample_id"), nullable=False)

    is_pooled = Column(Boolean, default=False)
    pool_id = Column(String(255), nullable=True, index=True)
    samples_in_pool = Column(JSON, default=list)
    pooling_ratio = Column(JSON, default=dict)
    barcode_sequences = Column(JSON, default=dict)
    multiplex_strategy = Column(String(255), nullable=True)

    created_at = Column(DateTime(timezone=True), server_default=func.now())
    updated_at = Column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())

    # Relationships
    sample = relationship("SampleDB", back_populates="pooling_info")


class SequenceGenerationDB(Base):
    """Database model for sequence generation parameters"""

    __tablename__ = f"{TABLE_PREFIX}sequence_generation"

    id = Column(Integer, primary_key=True, index=True)
    sample_id = Column(String(255), ForeignKey(f"{TABLE_PREFIX}samples.sample_id"), nullable=False)

    sequencing_platform = Column(String(255), nullable=True)
    read_length = Column(Integer, nullable=True)
    read_type = Column(String(255), nullable=True)
    target_coverage = Column(Float, nullable=True)
    library_prep_kit = Column(String(255), nullable=True)
    index_sequences = Column(JSON, default=list)
    quality_metrics = Column(JSON, default=dict)

    created_at = Column(DateTime(timezone=True), server_default=func.now())
    updated_at = Column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())

    # Relationships
    sample = relationship("SampleDB", back_populates="sequence_generation")


class InformaticsInfoDB(Base):
    """Database model for informatics information"""

    __tablename__ = f"{TABLE_PREFIX}informatics_info"

    id = Column(Integer, primary_key=True, index=True)
    sample_id = Column(String(255), ForeignKey(f"{TABLE_PREFIX}samples.sample_id"), nullable=False)

    analysis_type = Column(SQLEnum(AnalysisTypeEnum), nullable=False)
    reference_genome = Column(String(255), nullable=True)
    analysis_pipeline = Column(String(255), nullable=True)
    custom_parameters = Column(JSON, default=dict)
    data_delivery_format = Column(String(255), nullable=True)
    computational_requirements = Column(Text, nullable=True)

    created_at = Column(DateTime(timezone=True), server_default=func.now())
    updated_at = Column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())

    # Relationships
    sample = relationship("SampleDB", back_populates="informatics_info")


class DocumentDB(Base):
    """Database model for processed documents"""

    __tablename__ = f"{TABLE_PREFIX}documents"

    id = Column(Integer, primary_key=True, index=True)
    document_id = Column(String(255), unique=True, index=True, nullable=False)
    submission_id = Column(
        String(255), ForeignKey(f"{TABLE_PREFIX}lab_submissions.submission_id"), nullable=False
    )

    filename = Column(String(255), nullable=False)
    file_path = Column(String(500), nullable=False)
    file_type = Column(String(50), nullable=False)
    file_size = Column(Integer, nullable=True)

    # Processing information
    processed = Column(Boolean, default=False)
    processing_time = Column(Float, nullable=True)
    chunk_count = Column(Integer, default=0)

    # Timestamps
    uploaded_at = Column(DateTime(timezone=True), server_default=func.now())
    processed_at = Column(DateTime(timezone=True), nullable=True)
    created_at = Column(DateTime(timezone=True), server_default=func.now())
    updated_at = Column(DateTime(timezone=True), server_default=func.now(), onupdate=func.now())

    # Relationships
    submission = relationship("LabSubmissionDB", back_populates="documents")
    chunks = relationship(
        "DocumentChunkDB", back_populates="document", cascade="all, delete-orphan"
    )


class DocumentChunkDB(Base):
    """Database model for document chunks"""

    __tablename__ = f"{TABLE_PREFIX}document_chunks"

    id = Column(Integer, primary_key=True, index=True)
    chunk_id = Column(String(255), unique=True, index=True, nullable=False)
    document_id = Column(
        String(255), ForeignKey(f"{TABLE_PREFIX}documents.document_id"), nullable=False
    )

    content = Column(Text, nullable=False)
    chunk_index = Column(Integer, nullable=False)
    page_number = Column(Integer, default=1)

    # Vector embedding (stored as JSON for now, could use pgvector extension)
    embedding = Column(JSON, nullable=True)

    # Metadata
    meta_data = Column(JSON, default=dict)

    created_at = Column(DateTime(timezone=True), server_default=func.now())

    # Relationships
    document = relationship("DocumentDB", back_populates="chunks")


class ExtractionResultDB(Base):
    """Database model for extraction results"""

    __tablename__ = f"{TABLE_PREFIX}extraction_results"

    id = Column(Integer, primary_key=True, index=True)
    extraction_id = Column(String(255), unique=True, index=True, nullable=False)
    submission_id = Column(
        String(255), ForeignKey(f"{TABLE_PREFIX}lab_submissions.submission_id"), nullable=False
    )

    success = Column(Boolean, nullable=False)
    confidence_score = Column(Float, nullable=False)
    missing_fields = Column(JSON, default=list)
    warnings = Column(JSON, default=list)
    processing_time = Column(Float, nullable=False)
    source_document = Column(String(500), nullable=False)

    # Extracted data (JSON representation of the submission)
    extracted_data = Column(JSON, nullable=True)

    created_at = Column(DateTime(timezone=True), server_default=func.now())

    # Relationships
    submission = relationship("LabSubmissionDB", back_populates="extractions")


class QueryLogDB(Base):
    """Database model for query logging"""

    __tablename__ = f"{TABLE_PREFIX}query_logs"

    id = Column(Integer, primary_key=True, index=True)
    query_id = Column(String(255), unique=True, index=True, nullable=False)

    query_text = Column(Text, nullable=False)
    session_id = Column(String(255), nullable=True, index=True)
    response_text = Column(Text, nullable=True)
    processing_time = Column(Float, nullable=True)

    # Metadata
    filter_metadata = Column(JSON, nullable=True)
    chunks_retrieved = Column(Integer, default=0)

    created_at = Column(DateTime(timezone=True), server_default=func.now())


# Note: Lab Manager tables are managed separately by the Lab Manager service
# The RAG system uses its own tables with the rag_ prefix to avoid conflicts
