#!/usr/bin/env python3
"""
Analyze lab_manager database schema to align with RAG extraction models
"""

import asyncio

import asyncpg
from dotenv import load_dotenv

load_dotenv()


async def analyze_lab_manager_schema():
    """Analyze existing lab_manager tables to understand data structure"""
    print("🔍 Analyzing lab_manager database schema...")

    conn = await asyncpg.connect(
        host="localhost", port=5433, database="lab_manager", user="postgres", password="postgres"
    )

    # Key tables to analyze
    key_tables = ["samples", "sequencing_jobs", "storage_locations", "sample_locations"]

    schema_analysis = {}

    for table_name in key_tables:
        try:
            # Get column information
            columns = await conn.fetch(
                f"""
                SELECT column_name, data_type, is_nullable, column_default
                FROM information_schema.columns 
                WHERE table_name = '{table_name}' AND table_schema = 'public'
                ORDER BY ordinal_position
            """
            )

            if columns:
                print(f"\n📋 {table_name.upper()} TABLE:")
                schema_analysis[table_name] = []

                for col in columns:
                    col_info = {
                        "name": col["column_name"],
                        "type": col["data_type"],
                        "nullable": col["is_nullable"] == "YES",
                        "default": col["column_default"],
                    }
                    schema_analysis[table_name].append(col_info)
                    nullable = "NULL" if col["is_nullable"] == "YES" else "NOT NULL"
                    print(f"   - {col['column_name']}: {col['data_type']} ({nullable})")

                # Get sample data to understand values
                sample_data = await conn.fetch(f"SELECT * FROM {table_name} LIMIT 3")
                if sample_data:
                    print(f"   📊 Sample data ({len(sample_data)} rows):")
                    for i, row in enumerate(sample_data[:2]):  # Show first 2 rows
                        print(f"      Row {i+1}: {dict(row)}")

        except Exception as e:
            print(f"   ❌ Could not analyze {table_name}: {e}")

    await conn.close()
    return schema_analysis


async def suggest_rag_alignment():
    """Suggest how to align RAG extraction with lab_manager schema"""
    print("\n" + "=" * 60)
    print("🎯 RAG ALIGNMENT SUGGESTIONS")
    print("=" * 60)

    print(
        """
Based on lab_manager schema, here's how to align RAG extraction:

🧪 SAMPLE INFORMATION MAPPING:
   RAG Extraction → lab_manager Column
   ----------------------------------------
   sample_id → samples.barcode or samples.name
   sample_type → samples.material_type 
   concentration → samples.concentration
   volume → samples.volume
   storage_conditions → samples.storage_location_id (FK)

🔬 SEQUENCING INFORMATION MAPPING:
   RAG Extraction → sequencing_jobs Column
   ----------------------------------------
   platform → sequencing_jobs.sequencer
   analysis_type → sequencing_jobs.analysis_type
   coverage → sequencing_jobs.target_coverage
   read_length → sequencing_jobs.read_length

📍 STORAGE INFORMATION MAPPING:
   RAG Extraction → storage_locations Column
   ----------------------------------------  
   storage_conditions → storage_locations.name
   location → storage_locations.location_type

👤 SUBMITTER INFORMATION:
   Create new submission_requests table or extend existing
   """
    )


async def create_aligned_rag_model():
    """Create RAG models aligned with lab_manager schema"""

    aligned_model = '''
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
    '''

    print("\n📝 ALIGNED RAG MODEL:")
    print(aligned_model)

    # Save to file
    with open("aligned_rag_model.py", "w") as f:
        f.write(aligned_model)
    print("\n✅ Saved aligned model to 'aligned_rag_model.py'")


async def main():
    """Main analysis workflow"""
    schema_info = await analyze_lab_manager_schema()
    await suggest_rag_alignment()
    await create_aligned_rag_model()

    print("\n🎉 Schema analysis complete!")
    print("💡 Next steps:")
    print("   1. Update RAG extraction prompts with lab_manager field names")
    print("   2. Create data mapping functions")
    print("   3. Test extraction with aligned model")


if __name__ == "__main__":
    asyncio.run(main())
