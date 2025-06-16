#!/usr/bin/env python3
"""
Simple Data Models
Extracted from simple_lab_rag.py for better modularity
"""

import uuid
from datetime import datetime
from typing import List, Optional, Dict, Any
from pydantic import BaseModel, Field


class AdministrativeInfo(BaseModel):
    """Administrative information from lab submissions"""
    submitter_name: Optional[str] = None
    submitter_email: Optional[str] = None
    submitter_phone: Optional[str] = None
    project_name: Optional[str] = None
    institution: Optional[str] = None


class SampleInfo(BaseModel):
    """Sample information from lab submissions"""
    sample_id: Optional[str] = None
    sample_type: Optional[str] = None  # DNA, RNA, etc.
    concentration: Optional[str] = None
    volume: Optional[str] = None
    storage_conditions: Optional[str] = None


class SequencingInfo(BaseModel):
    """Sequencing information from lab submissions"""
    platform: Optional[str] = None  # Illumina, PacBio, etc.
    analysis_type: Optional[str] = None  # WGS, RNA-seq, etc.
    coverage: Optional[str] = None
    read_length: Optional[str] = None


class LabSubmission(BaseModel):
    """Simplified lab submission model"""
    submission_id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    created_at: datetime = Field(default_factory=datetime.now)
    
    # Core information categories
    administrative: AdministrativeInfo = Field(default_factory=AdministrativeInfo)
    sample: SampleInfo = Field(default_factory=SampleInfo) 
    sequencing: SequencingInfo = Field(default_factory=SequencingInfo)
    
    # Raw extracted text and metadata
    raw_text: Optional[str] = None
    confidence_score: Optional[float] = None
    source_document: Optional[str] = None


class ExtractionResult(BaseModel):
    """Result of document processing"""
    success: bool
    submission: Optional[LabSubmission] = None
    submission_id: Optional[str] = None
    extracted_data: Optional[Dict[str, Any]] = None
    confidence_score: Optional[float] = None
    error: Optional[str] = None
    warnings: List[str] = Field(default_factory=list) 
