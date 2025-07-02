"""
Pydantic models for laboratory submission data
"""

from datetime import datetime
from enum import Enum
from typing import Any

from pydantic import BaseModel, ConfigDict, EmailStr, Field


class PriorityLevel(str, Enum):
    HIGH = "high"
    MEDIUM = "medium"
    LOW = "low"


class SampleType(str, Enum):
    """Types of laboratory samples"""

    BLOOD = "blood"
    SALIVA = "saliva"
    TISSUE = "tissue"
    URINE = "urine"
    DNA = "dna"
    RNA = "rna"
    OTHER = "other"
    SWAB = "swab"


class AnalysisType(str, Enum):
    WGS = "wgs"
    WES = "wes"
    TARGETED_PANEL = "targeted_panel"
    RNA_SEQ = "rna_seq"
    CHIP_SEQ = "chip_seq"
    ATAC_SEQ = "atac_seq"
    OTHER = "other"


class StorageCondition(str, Enum):
    """Storage conditions for samples"""

    ROOM_TEMP = "room_temperature"
    REFRIGERATED = "refrigerated"
    FROZEN = "frozen"
    CRYOGENIC = "cryogenic"


class ProcessingStatus(str, Enum):
    """Status of sample processing"""

    RECEIVED = "received"
    IN_PROCESS = "in_process"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


# 1. Administrative Information
class AdministrativeInfo(BaseModel):
    """Administrative information for lab submission"""

    submitter_first_name: str = Field(..., description="Submitter's first name")
    submitter_last_name: str = Field(..., description="Submitter's last name")
    submitter_email: EmailStr = Field(..., description="Submitter's email address")
    submitter_phone: str = Field(..., description="Submitter's phone number")
    assigned_project: str = Field(..., description="Project assignment identifier")
    submission_date: datetime | None = Field(default_factory=datetime.now)
    department: str | None = Field(None, description="Submitting department")
    institution: str | None = Field(None, description="Submitting institution")


# 2. Source and Submitting Material
class SourceMaterial(BaseModel):
    """Source and submitting material information"""

    source_type: SampleType = Field(..., description="Type of source material")
    collection_date: datetime | None = Field(None, description="Date of collection")
    collection_method: str | None = Field(None, description="Method used for collection")
    source_organism: str | None = Field(None, description="Source organism")
    tissue_type: str | None = Field(None, description="Specific tissue type")
    preservation_method: str | None = Field(None, description="Preservation method")
    storage_conditions: str | None = Field(None, description="Storage conditions")
    chain_of_custody: list[str] | None = Field(
        default_factory=list, description="Chain of custody information"
    )


# 3. Pooling (Multiplexing)
class PoolingInfo(BaseModel):
    """Pooling and multiplexing information"""

    is_pooled: bool = Field(False, description="Whether samples are pooled")
    pool_id: str | None = Field(None, description="Pool identifier")
    samples_in_pool: list[str] | None = Field(
        default_factory=list, description="Sample IDs in pool"
    )
    pooling_ratio: dict[str, float] | None = Field(
        default_factory=dict, description="Pooling ratios"
    )
    barcode_sequences: dict[str, str] | None = Field(
        default_factory=dict, description="Barcode sequences"
    )
    multiplex_strategy: str | None = Field(None, description="Multiplexing strategy")


# 4. Sequence Generation
class SequenceGeneration(BaseModel):
    """Sequence generation parameters"""

    sequencing_platform: str | None = Field(None, description="Sequencing platform")
    read_length: int | None = Field(None, description="Read length")
    read_type: str | None = Field(None, description="Single-end or paired-end")
    target_coverage: float | None = Field(None, description="Target coverage depth")
    library_prep_kit: str | None = Field(None, description="Library preparation kit")
    index_sequences: list[str] | None = Field(
        default_factory=list, description="Index sequences"
    )
    quality_metrics: dict[str, float] | None = Field(
        default_factory=dict, description="Quality metrics"
    )


# 5. Container and Diluent
class ContainerInfo(BaseModel):
    """Container and diluent information"""

    container_type: str | None = Field(None, description="Type of container")
    container_id: str | None = Field(None, description="Container identifier")
    volume: float | None = Field(None, description="Sample volume in mL")
    concentration: float | None = Field(None, description="Concentration in ng/μL")
    diluent_used: str | None = Field(None, description="Diluent used")
    storage_temperature: str | None = Field(None, description="Storage temperature")
    container_barcode: str | None = Field(None, description="Container barcode")


# 6. Informatics
class InformaticsInfo(BaseModel):
    """Informatics and analysis information"""

    analysis_type: AnalysisType = Field(..., description="Type of analysis requested")
    reference_genome: str | None = Field(None, description="Reference genome")
    analysis_pipeline: str | None = Field(None, description="Analysis pipeline")
    custom_parameters: dict[str, Any] | None = Field(
        default_factory=dict, description="Custom analysis parameters"
    )
    data_delivery_format: str | None = Field(None, description="Preferred data delivery format")
    computational_requirements: str | None = Field(
        None, description="Special computational requirements"
    )


# 7. Sample Details
class SampleDetails(BaseModel):
    """Detailed sample information"""

    sample_id: str = Field(..., description="Unique sample identifier")
    patient_id: str | None = Field(None, description="Patient identifier")
    sample_name: str | None = Field(None, description="Sample name or description")
    priority: PriorityLevel = Field(PriorityLevel.MEDIUM, description="Processing priority")
    quality_score: float | None = Field(None, description="Sample quality score")
    purity_ratio: float | None = Field(None, description="260/280 purity ratio")
    integrity_number: float | None = Field(None, description="DNA/RNA integrity number")
    notes: str | None = Field(None, description="Additional notes")
    special_instructions: str | None = Field(None, description="Special handling instructions")


# Complete Submission Model
class LabSubmission(BaseModel):
    """Laboratory sample submission model"""

    submission_id: str = Field(..., description="Unique identifier for the submission")
    client_id: str = Field(..., description="Identifier of the submitting client")
    client_name: str = Field(..., description="Name of the submitting client")
    client_email: EmailStr = Field(..., description="Email of the submitting client")

    # Sample information
    sample_type: SampleType = Field(..., description="Type of sample submitted")
    sample_count: int = Field(..., ge=1, description="Number of samples in submission")
    sample_volume: float | None = Field(None, description="Volume of each sample in mL")
    storage_condition: StorageCondition = Field(..., description="Required storage condition")

    # Processing requirements
    processing_requirements: list[str] = Field(
        default_factory=list, description="List of processing requirements"
    )
    special_instructions: str | None = Field(None, description="Special handling instructions")

    # Administrative tracking
    submission_date: datetime = Field(
        default_factory=datetime.utcnow, description="Date and time of submission"
    )
    status: ProcessingStatus = Field(
        default=ProcessingStatus.RECEIVED, description="Current processing status"
    )
    priority: int = Field(
        default=1, ge=1, le=5, description="Processing priority (1-5, 1 being highest)"
    )

    # Metadata
    metadata: dict[str, Any] = Field(default_factory=dict, description="Additional metadata")
    created_at: datetime = Field(
        default_factory=datetime.utcnow, description="Timestamp when record was created"
    )
    updated_at: datetime = Field(
        default_factory=datetime.utcnow, description="Timestamp when record was last updated"
    )

    model_config = ConfigDict(use_enum_values=True)


# Response Models
class ExtractionResult(BaseModel):
    """Result of document extraction"""

    success: bool
    submission: LabSubmission | None = None
    extracted_data: dict[str, Any] | None = None
    confidence_score: float
    missing_fields: list[str] = Field(default_factory=list)
    warnings: list[str] = Field(default_factory=list)
    processing_time: float
    source_document: str


class BatchExtractionResult(BaseModel):
    """Result of batch document extraction"""

    total_documents: int
    successful_extractions: int
    failed_extractions: int
    results: list[ExtractionResult]
    overall_confidence: float
    processing_time: float
