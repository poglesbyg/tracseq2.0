# Updated RAG extraction model aligned with lab_manager


class AlignedLabSubmission(BaseModel):
    """RAG submission model aligned with lab_manager schema"""

    # Administrative (maps to potential submissions table)
    submitter_name: Optional[str] = Field(description="Person submitting sample")
    submitter_email: Optional[str] = Field(description="Contact email")
    project_name: Optional[str] = Field(description="Research project name")
    institution: Optional[str] = Field(description="Submitting institution")

    # Sample Information (maps to samples table)
    sample_barcode: Optional[str] = Field(description="Maps to samples.barcode")
    sample_name: Optional[str] = Field(description="Maps to samples.name")
    material_type: Optional[str] = Field(description="Maps to samples.material_type")
    concentration: Optional[float] = Field(description="Maps to samples.concentration")
    volume: Optional[float] = Field(description="Maps to samples.volume")

    # Storage (maps to storage_locations via samples.storage_location_id)
    storage_location: Optional[str] = Field(description="Storage location name")
    storage_type: Optional[str] = Field(description="Storage condition type")

    # Sequencing (maps to sequencing_jobs table)
    sequencer: Optional[str] = Field(description="Maps to sequencing_jobs.sequencer")
    analysis_type: Optional[str] = Field(description="Maps to sequencing_jobs.analysis_type")
    target_coverage: Optional[str] = Field(description="Maps to sequencing_jobs.target_coverage")
    read_length: Optional[str] = Field(description="Maps to sequencing_jobs.read_length")

    # Metadata
    extraction_confidence: Optional[float] = Field(default=0.0)
    source_document: Optional[str] = Field(description="Original document path")
