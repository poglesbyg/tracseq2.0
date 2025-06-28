# Updated RAG extraction model aligned with lab_manager


class AlignedLabSubmission(BaseModel):
    """RAG submission model aligned with lab_manager schema"""

    # Administrative (maps to potential submissions table)
    submitter_name: str | None = Field(description="Person submitting sample")
    submitter_email: str | None = Field(description="Contact email")
    project_name: str | None = Field(description="Research project name")
    institution: str | None = Field(description="Submitting institution")

    # Sample Information (maps to samples table)
    sample_barcode: str | None = Field(description="Maps to samples.barcode")
    sample_name: str | None = Field(description="Maps to samples.name")
    material_type: str | None = Field(description="Maps to samples.material_type")
    concentration: float | None = Field(description="Maps to samples.concentration")
    volume: float | None = Field(description="Maps to samples.volume")

    # Storage (maps to storage_locations via samples.storage_location_id)
    storage_location: str | None = Field(description="Storage location name")
    storage_type: str | None = Field(description="Storage condition type")

    # Sequencing (maps to sequencing_jobs table)
    sequencer: str | None = Field(description="Maps to sequencing_jobs.sequencer")
    analysis_type: str | None = Field(description="Maps to sequencing_jobs.analysis_type")
    target_coverage: str | None = Field(description="Maps to sequencing_jobs.target_coverage")
    read_length: str | None = Field(description="Maps to sequencing_jobs.read_length")

    # Metadata
    extraction_confidence: float | None = Field(default=0.0)
    source_document: str | None = Field(description="Original document path")
