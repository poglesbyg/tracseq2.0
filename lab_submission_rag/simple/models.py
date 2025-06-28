#!/usr/bin/env python3
"""
Simple Data Models
Extracted from simple_lab_rag.py for better modularity
"""

import uuid
from datetime import datetime
from typing import Any

from pydantic import BaseModel, Field


class AdministrativeInfo(BaseModel):
    """Administrative information from lab submissions"""

    submitter_name: str | None = None
    submitter_email: str | None = None
    submitter_phone: str | None = None
    project_name: str | None = None
    institution: str | None = None


class SampleInfo(BaseModel):
    """Sample information from lab submissions"""

    sample_id: str | None = None
    sample_type: str | None = None  # DNA, RNA, etc.
    concentration: str | None = None
    volume: str | None = None
    storage_conditions: str | None = None


class SequencingInfo(BaseModel):
    """Sequencing information from lab submissions"""

    platform: str | None = None  # Illumina, PacBio, etc.
    analysis_type: str | None = None  # WGS, RNA-seq, etc.
    coverage: str | None = None
    read_length: str | None = None


class LabSubmission(BaseModel):
    """Simplified lab submission model"""

    submission_id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    created_at: datetime = Field(default_factory=datetime.now)

    # Core information categories
    administrative: AdministrativeInfo = Field(default_factory=AdministrativeInfo)
    sample: SampleInfo = Field(default_factory=SampleInfo)
    sequencing: SequencingInfo = Field(default_factory=SequencingInfo)

    # Raw extracted text and metadata
    raw_text: str | None = None
    confidence_score: float | None = None
    source_document: str | None = None


class ExtractionResult(BaseModel):
    """Result of document processing"""

    success: bool
    submission: LabSubmission | None = None
    submission_id: str | None = None
    extracted_data: dict[str, Any] | None = None
    confidence_score: float | None = None
    error: str | None = None
    warnings: list[str] = Field(default_factory=list)
