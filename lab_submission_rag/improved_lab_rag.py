#!/usr/bin/env python3
"""
Improved Laboratory RAG System - Aligned with lab_manager schema
"""

import asyncio
import json
import logging
import uuid
from datetime import datetime
from pathlib import Path
from typing import Any

import asyncpg
import ollama
from pydantic import BaseModel, Field
from sentence_transformers import SentenceTransformer

# Setup logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# ============================================================================
# ALIGNED DATA MODELS (matching lab_manager schema)
# ============================================================================


class LabManagerSample(BaseModel):
    """Model aligned with lab_manager samples table"""

    # Core sample fields (matching lab_manager.samples)
    name: str | None = Field(description="Sample name (maps to samples.name)")
    barcode: str | None = Field(description="Sample barcode (maps to samples.barcode)")
    location: str | None = Field(description="Storage location (maps to samples.location)")
    status: str | None = Field(description="Sample status", default="received")
    metadata: dict[str, Any] = Field(
        default_factory=dict, description="Additional metadata as JSON"
    )


class LabManagerSubmission(BaseModel):
    """RAG extraction model aligned with lab_manager workflow"""

    # Administrative Information
    submitter_name: str | None = Field(description="Person submitting the sample")
    submitter_email: str | None = Field(description="Contact email address")
    submitter_phone: str | None = Field(description="Contact phone number")
    institution: str | None = Field(description="Submitting institution/organization")
    project_name: str | None = Field(description="Research project name")

    # Sample Information (aligned with samples table)
    sample_name: str | None = Field(description="Descriptive sample name")
    sample_barcode: str | None = Field(description="Unique sample identifier")
    material_type: str | None = Field(description="Type of biological material (DNA, RNA, etc.)")
    concentration: str | None = Field(description="Sample concentration")
    volume: str | None = Field(description="Sample volume")

    # Storage Information (aligned with storage_locations)
    storage_location: str | None = Field(description="Storage location name")
    storage_temperature: str | None = Field(description="Storage temperature requirement")
    storage_conditions: str | None = Field(description="Special storage conditions")

    # Sequencing Requirements (aligned with sequencing_jobs table structure)
    sequencing_platform: str | None = Field(description="Sequencing instrument/platform")
    analysis_type: str | None = Field(description="Type of sequencing analysis")
    target_coverage: str | None = Field(description="Desired sequencing coverage")
    read_length: str | None = Field(description="Sequencing read length")
    library_prep: str | None = Field(description="Library preparation method")

    # Quality and Priority
    priority_level: str | None = Field(description="Processing priority (high, medium, low)")
    quality_metrics: str | None = Field(description="Quality assessment data")
    special_instructions: str | None = Field(description="Special handling instructions")

    # Metadata
    submission_date: datetime | None = Field(default_factory=datetime.now)
    extraction_confidence: float = Field(default=0.0, description="AI extraction confidence score")
    source_document: str | None = Field(description="Original document filename")


class ExtractionResult(BaseModel):
    """Result of document processing"""

    success: bool
    submission: LabManagerSubmission | None = None
    confidence_score: float = 0.0
    warnings: list[str] = Field(default_factory=list)
    processing_time: float = 0.0


# ============================================================================
# IMPROVED LLM INTERFACE WITH ALIGNED PROMPTS
# ============================================================================


class ImprovedLLMInterface:
    """Enhanced LLM interface with lab_manager-aligned extraction"""

    def __init__(self, model: str = "llama3.2:3b") -> None:
        self.model = model

        # Enhanced extraction prompt aligned with lab_manager schema
        self.extraction_prompt = """
You are an expert laboratory information extraction system. Extract information from the laboratory submission document below and format it as JSON.

Focus on these key areas that align with our laboratory management system:

**ADMINISTRATIVE INFORMATION:**
- submitter_name: Full name of person submitting
- submitter_email: Email address  
- submitter_phone: Phone number
- institution: Organization/institution name
- project_name: Research project or study name

**SAMPLE INFORMATION:**
- sample_name: Descriptive name for the sample
- sample_barcode: Unique sample identifier/barcode
- material_type: Type (DNA, RNA, Protein, Tissue, etc.)
- concentration: Concentration value with units
- volume: Volume value with units

**STORAGE REQUIREMENTS:**
- storage_location: Specific storage location/freezer
- storage_temperature: Temperature requirement (e.g., -80°C, -20°C, 4°C)
- storage_conditions: Any special storage conditions

**SEQUENCING REQUIREMENTS:**
- sequencing_platform: Instrument/platform (Illumina, PacBio, etc.)
- analysis_type: Type of analysis (WGS, WES, RNA-seq, etc.)
- target_coverage: Desired coverage (e.g., 30x, 100x)
- read_length: Read length specification
- library_prep: Library preparation method

**ADDITIONAL DETAILS:**
- priority_level: Processing priority (high, medium, low)
- quality_metrics: Any quality measurements
- special_instructions: Special handling notes

Respond with valid JSON only, using null for missing information:

{{
  "submitter_name": "value or null",
  "submitter_email": "value or null",
  "submitter_phone": "value or null", 
  "institution": "value or null",
  "project_name": "value or null",
  "sample_name": "value or null",
  "sample_barcode": "value or null",
  "material_type": "value or null",
  "concentration": "value or null",
  "volume": "value or null",
  "storage_location": "value or null",
  "storage_temperature": "value or null",
  "storage_conditions": "value or null",
  "sequencing_platform": "value or null",
  "analysis_type": "value or null",
  "target_coverage": "value or null",
  "read_length": "value or null",
  "library_prep": "value or null",
  "priority_level": "value or null",
  "quality_metrics": "value or null",
  "special_instructions": "value or null"
}}

Document to analyze:
{text}
"""

    def extract_submission_info(self, text: str) -> dict[str, Any]:
        """Extract structured information using improved prompts"""
        try:
            logger.info(f"Extracting information with model: {self.model}")

            response = ollama.generate(
                model=self.model,
                prompt=self.extraction_prompt.format(text=text),
                options={"temperature": 0.1, "num_predict": 1500},
            )

            # Parse response
            result_text = response.response.strip()
            logger.info(f"Raw response: {result_text[:200]}...")

            # Clean up response
            if result_text.startswith("```json"):
                result_text = result_text[7:-3].strip()
            elif result_text.startswith("```"):
                result_text = result_text[3:-3].strip()

            result_text = result_text.strip()
            if not result_text.startswith("{"):
                start = result_text.find("{")
                end = result_text.rfind("}")
                if start != -1 and end != -1:
                    result_text = result_text[start : end + 1]

            return json.loads(result_text)

        except Exception as e:
            logger.error(f"Extraction failed: {e}")
            return {"error": str(e)}


# ============================================================================
# IMPROVED RAG SYSTEM WITH LAB_MANAGER INTEGRATION
# ============================================================================


class ImprovedLabRAG:
    """Enhanced Laboratory RAG System with lab_manager integration"""

    def __init__(self, model: str = "llama3.2:3b") -> None:
        self.model = model
        self.llm = ImprovedLLMInterface(model)
        self.embeddings_model = SentenceTransformer("all-MiniLM-L6-v2")

        # Database connection details
        self.db_config = {
            "host": "localhost",
            "port": 5433,
            "database": "lab_manager",
            "user": "postgres",
            "password": "postgres",
        }

    async def connect_to_lab_manager(self) -> None:
        """Connect to lab_manager database"""
        return await asyncpg.connect(**self.db_config)

    async def process_document(self, file_path: str) -> ExtractionResult:
        """Process a laboratory document with improved extraction"""
        start_time = datetime.now()

        try:
            # Read document
            with open(file_path, encoding="utf-8") as f:
                text = f.read()

            logger.info(f"Processing document: {file_path}")

            # Extract structured information
            extracted_data = self.llm.extract_submission_info(text)

            if "error" in extracted_data:
                return ExtractionResult(
                    success=False,
                    warnings=[extracted_data["error"]],
                    processing_time=(datetime.now() - start_time).total_seconds(),
                )

            # Create submission object
            submission = LabManagerSubmission(
                **extracted_data,
                source_document=file_path,
                extraction_confidence=0.85,  # Could be calculated based on completeness
            )

            # Store in lab_manager database
            await self._store_in_lab_manager(submission)

            processing_time = (datetime.now() - start_time).total_seconds()

            return ExtractionResult(
                success=True,
                submission=submission,
                confidence_score=0.85,
                processing_time=processing_time,
            )

        except Exception as e:
            logger.error(f"Document processing failed: {e}")
            return ExtractionResult(
                success=False,
                warnings=[str(e)],
                processing_time=(datetime.now() - start_time).total_seconds(),
            )

    async def _store_in_lab_manager(self, submission: LabManagerSubmission) -> None:
        """Store processed submission in lab_manager database"""
        try:
            conn = await self.connect_to_lab_manager()

            # Store in rag_submissions table
            await conn.execute(
                """
                INSERT INTO rag_submissions (
                    submission_id, 
                    document_name, 
                    submitter_name, 
                    submitter_email,
                    sample_type,
                    extracted_data,
                    confidence_score,
                    source_document
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            """,
                str(uuid.uuid4()),
                submission.sample_name,
                submission.submitter_name,
                submission.submitter_email,
                submission.material_type,
                submission.dict(),
                submission.extraction_confidence,
                submission.source_document,
            )

            # If sample information is complete, consider creating sample record
            if submission.sample_barcode and submission.sample_name:
                await self._create_sample_record(conn, submission)

            await conn.close()
            logger.info("✅ Stored submission in lab_manager database")

        except Exception as e:
            logger.error(f"Failed to store in lab_manager: {e}")

    async def _create_sample_record(self, conn, submission: LabManagerSubmission) -> None:
        """Create a sample record in lab_manager if data is sufficient"""
        try:
            # Check if sample already exists
            existing = await conn.fetchval(
                "SELECT id FROM samples WHERE barcode = $1", submission.sample_barcode
            )

            if not existing:
                # Create new sample record
                sample_id = str(uuid.uuid4())
                await conn.execute(
                    """
                    INSERT INTO samples (
                        id, name, barcode, location, status, metadata
                    ) VALUES ($1, $2, $3, $4, $5, $6)
                """,
                    sample_id,
                    submission.sample_name,
                    submission.sample_barcode,
                    submission.storage_location or "Pending",
                    "received",
                    json.dumps(
                        {
                            "material_type": submission.material_type,
                            "concentration": submission.concentration,
                            "volume": submission.volume,
                            "submitter": submission.submitter_name,
                            "project": submission.project_name,
                            "rag_processed": True,
                        }
                    ),
                )
                logger.info(f"✅ Created sample record: {submission.sample_barcode}")

        except Exception as e:
            logger.warning(f"Could not create sample record: {e}")


# ============================================================================
# TESTING AND DEMO FUNCTIONS
# ============================================================================


async def test_improved_system() -> None:
    """Test the improved RAG system"""
    print("🧬 Testing Improved Lab RAG System")
    print("=" * 50)

    rag = ImprovedLabRAG()

    # Test database connection
    try:
        conn = await rag.connect_to_lab_manager()
        await conn.close()
        print("✅ Database connection successful")
    except Exception as e:
        print(f"❌ Database connection failed: {e}")
        return

    # Create test document
    test_doc = Path("test_improved_submission.txt")
    test_content = """
Laboratory Sample Submission Request

Submitter Information:
Name: Dr. Sarah Chen
Email: sarah.chen@research.edu  
Phone: (555) 123-4567
Institution: Genomics Research Institute
Project: Metabolic Disease Study 2024

Sample Details:
Sample ID: MDS_2024_001
Sample Name: Patient_001_Plasma
Barcode: MDS001
Material Type: Blood Plasma
Concentration: 45 mg/mL
Volume: 500 μL

Storage Requirements:
Location: Freezer B
Temperature: -80°C
Conditions: Aliquot into 50μL tubes

Sequencing Requirements:
Platform: Illumina NovaSeq 6000
Analysis: Whole Exome Sequencing
Coverage: 100x
Read Length: 150bp paired-end
Library Prep: TruSeq Exome

Priority: High
Quality: A260/A280 = 1.8
Instructions: Process within 48 hours
"""

    test_doc.write_text(test_content)

    # Process document
    print("\n🔄 Processing test document...")
    result = await rag.process_document(str(test_doc))

    if result.success:
        print("✅ Processing successful!")
        print(f"   Confidence: {result.confidence_score:.2f}")
        print(f"   Processing time: {result.processing_time:.2f}s")

        submission = result.submission
        print("\n📋 Extracted Information:")
        print(f"   Submitter: {submission.submitter_name}")
        print(f"   Email: {submission.submitter_email}")
        print(f"   Sample: {submission.sample_name} ({submission.sample_barcode})")
        print(f"   Material: {submission.material_type}")
        print(f"   Storage: {submission.storage_temperature} in {submission.storage_location}")
        print(f"   Platform: {submission.sequencing_platform}")
        print(f"   Analysis: {submission.analysis_type}")

    else:
        print(f"❌ Processing failed: {result.warnings}")

    # Cleanup
    test_doc.unlink()
    print("\n🎉 Test completed!")


if __name__ == "__main__":
    import os

    os.environ["OLLAMA_MODEL"] = "llama3.2:3b"
    asyncio.run(test_improved_system())
