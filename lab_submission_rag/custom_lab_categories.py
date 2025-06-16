#!/usr/bin/env python3
"""
Custom Laboratory Categories Configuration
Allows customization of the 7 extraction categories for specific lab workflows
"""

from typing import Dict, List, Any
from pydantic import BaseModel, Field
from enum import Enum

class CategoryType(str, Enum):
    """Types of laboratory information categories"""
    ADMINISTRATIVE = "administrative"
    SAMPLE_INFO = "sample_info"
    STORAGE = "storage"
    SEQUENCING = "sequencing"
    QUALITY = "quality"
    WORKFLOW = "workflow"
    CUSTOM = "custom"

class FieldDefinition(BaseModel):
    """Definition of an extractable field"""
    name: str = Field(description="Field name in database")
    display_name: str = Field(description="Human-readable field name")
    description: str = Field(description="What this field contains")
    data_type: str = Field(description="Expected data type (string, number, date, etc.)")
    required: bool = Field(default=False, description="Is this field required?")
    examples: List[str] = Field(default_factory=list, description="Example values")
    validation_pattern: str = Field(default="", description="Regex pattern for validation")

class CategoryDefinition(BaseModel):
    """Definition of an extraction category"""
    name: str = Field(description="Category name")
    type: CategoryType = Field(description="Category type")
    description: str = Field(description="What this category extracts")
    priority: int = Field(description="Extraction priority (1=highest)")
    fields: List[FieldDefinition] = Field(description="Fields in this category")

class LabCategoryConfig:
    """Laboratory category configuration manager"""
    
    def __init__(self):
        self.categories = self._create_default_categories()
    
    def _create_default_categories(self) -> List[CategoryDefinition]:
        """Create default laboratory categories aligned with lab_manager"""
        
        return [
            CategoryDefinition(
                name="Submitter Information",
                type=CategoryType.ADMINISTRATIVE,
                description="Information about who is submitting the sample",
                priority=1,
                fields=[
                    FieldDefinition(
                        name="submitter_name",
                        display_name="Submitter Name",
                        description="Full name of the person submitting",
                        data_type="string",
                        required=True,
                        examples=["Dr. Sarah Chen", "John Smith", "Prof. Maria Rodriguez"]
                    ),
                    FieldDefinition(
                        name="submitter_email",
                        display_name="Email Address",
                        description="Contact email for the submitter",
                        data_type="email",
                        required=True,
                        examples=["sarah.chen@uni.edu", "john@lab.org"],
                        validation_pattern=r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
                    ),
                    FieldDefinition(
                        name="submitter_phone",
                        display_name="Phone Number",
                        description="Contact phone number",
                        data_type="string",
                        examples=["(555) 123-4567", "+1-555-123-4567"]
                    ),
                    FieldDefinition(
                        name="institution",
                        display_name="Institution",
                        description="Submitting organization or institution",
                        data_type="string",
                        required=True,
                        examples=["Harvard Medical School", "NIH", "Stanford University"]
                    ),
                    FieldDefinition(
                        name="project_name",
                        display_name="Project Name",
                        description="Research project or study name",
                        data_type="string",
                        examples=["Cancer Genomics Study 2024", "COVID-19 Variants", "Metabolic Disease Research"]
                    )
                ]
            ),
            
            CategoryDefinition(
                name="Sample Identification",
                type=CategoryType.SAMPLE_INFO,
                description="Core sample identification and characteristics",
                priority=2,
                fields=[
                    FieldDefinition(
                        name="sample_name",
                        display_name="Sample Name",
                        description="Descriptive name for the sample",
                        data_type="string",
                        required=True,
                        examples=["Patient_001_Tumor", "Control_Blood_01", "Mouse_Liver_A1"]
                    ),
                    FieldDefinition(
                        name="sample_barcode",
                        display_name="Sample Barcode/ID",
                        description="Unique sample identifier or barcode",
                        data_type="string",
                        required=True,
                        examples=["CGS001", "SAMPLE_2024_001", "BC123456"]
                    ),
                    FieldDefinition(
                        name="material_type",
                        display_name="Material Type",
                        description="Type of biological material",
                        data_type="string",
                        required=True,
                        examples=["DNA", "RNA", "Protein", "Blood", "Tissue", "Saliva", "FFPE"]
                    ),
                    FieldDefinition(
                        name="concentration",
                        display_name="Concentration",
                        description="Sample concentration with units",
                        data_type="string",
                        examples=["50 ng/uL", "2.5 mg/mL", "100 pM"]
                    ),
                    FieldDefinition(
                        name="volume",
                        display_name="Volume",
                        description="Sample volume with units",
                        data_type="string",
                        examples=["100 uL", "50 mL", "1.5 mL"]
                    )
                ]
            ),
            
            CategoryDefinition(
                name="Storage Requirements",
                type=CategoryType.STORAGE,
                description="Storage location and conditions for the sample",
                priority=3,
                fields=[
                    FieldDefinition(
                        name="storage_location",
                        display_name="Storage Location",
                        description="Specific storage location or freezer",
                        data_type="string",
                        examples=["Freezer A", "Room 123 -80C", "Liquid Nitrogen Tank 1"]
                    ),
                    FieldDefinition(
                        name="storage_temperature",
                        display_name="Storage Temperature",
                        description="Required storage temperature",
                        data_type="string",
                        examples=["-80°C", "-20°C", "4°C", "Room Temperature", "-196°C"]
                    ),
                    FieldDefinition(
                        name="storage_conditions",
                        display_name="Storage Conditions",
                        description="Special storage requirements or conditions",
                        data_type="string",
                        examples=["Aliquot into 50uL tubes", "Store in dark", "Avoid freeze-thaw cycles"]
                    )
                ]
            ),
            
            CategoryDefinition(
                name="Sequencing Parameters",
                type=CategoryType.SEQUENCING,
                description="Sequencing platform and analysis requirements",
                priority=4,
                fields=[
                    FieldDefinition(
                        name="sequencing_platform",
                        display_name="Sequencing Platform",
                        description="Sequencing instrument or platform",
                        data_type="string",
                        examples=["Illumina NovaSeq 6000", "PacBio Sequel", "Oxford Nanopore", "Ion Torrent"]
                    ),
                    FieldDefinition(
                        name="analysis_type",
                        display_name="Analysis Type",
                        description="Type of sequencing analysis requested",
                        data_type="string",
                        required=True,
                        examples=["WGS", "WES", "RNA-seq", "ChIP-seq", "ATAC-seq", "16S rRNA"]
                    ),
                    FieldDefinition(
                        name="target_coverage",
                        display_name="Target Coverage",
                        description="Desired sequencing coverage depth",
                        data_type="string",
                        examples=["30x", "100x", "10M reads", "50M reads"]
                    ),
                    FieldDefinition(
                        name="read_length",
                        display_name="Read Length",
                        description="Sequencing read length specification",
                        data_type="string",
                        examples=["150bp paired-end", "100bp single-end", "2x150bp", "Long reads"]
                    ),
                    FieldDefinition(
                        name="library_prep",
                        display_name="Library Preparation",
                        description="Library preparation method or kit",
                        data_type="string",
                        examples=["TruSeq Exome", "NEBNext Ultra", "KAPA HyperPrep", "Custom protocol"]
                    )
                ]
            ),
            
            CategoryDefinition(
                name="Quality and Priority",
                type=CategoryType.QUALITY,
                description="Quality metrics and processing priority",
                priority=5,
                fields=[
                    FieldDefinition(
                        name="priority_level",
                        display_name="Priority Level",
                        description="Processing priority for the sample",
                        data_type="string",
                        examples=["High", "Medium", "Low", "Urgent", "Standard"]
                    ),
                    FieldDefinition(
                        name="quality_metrics",
                        display_name="Quality Metrics",
                        description="Quality assessment measurements",
                        data_type="string",
                        examples=["A260/A280 = 1.8", "RIN = 8.5", "DV200 = 75%", "Qubit: 50ng/uL"]
                    ),
                    FieldDefinition(
                        name="quality_notes",
                        display_name="Quality Notes",
                        description="Additional quality-related observations",
                        data_type="string",
                        examples=["High molecular weight", "Slightly degraded", "Excellent quality"]
                    )
                ]
            ),
            
            CategoryDefinition(
                name="Workflow Instructions",
                type=CategoryType.WORKFLOW,
                description="Special instructions and workflow requirements",
                priority=6,
                fields=[
                    FieldDefinition(
                        name="special_instructions",
                        display_name="Special Instructions",
                        description="Special handling or processing instructions",
                        data_type="string",
                        examples=["Process within 48 hours", "Keep on ice", "Avoid contamination", "Rush processing"]
                    ),
                    FieldDefinition(
                        name="turnaround_time",
                        display_name="Required Turnaround Time",
                        description="Expected completion timeline",
                        data_type="string",
                        examples=["1 week", "3 days", "ASAP", "Standard (2 weeks)"]
                    ),
                    FieldDefinition(
                        name="delivery_method",
                        display_name="Result Delivery Method",
                        description="How results should be delivered",
                        data_type="string",
                        examples=["Email", "Secure portal", "Hard drive", "FTP upload"]
                    )
                ]
            ),
            
            CategoryDefinition(
                name="Additional Metadata",
                type=CategoryType.CUSTOM,
                description="Custom fields and additional metadata",
                priority=7,
                fields=[
                    FieldDefinition(
                        name="collection_date",
                        display_name="Collection Date",
                        description="When the sample was collected",
                        data_type="date",
                        examples=["2024-06-13", "June 13, 2024"]
                    ),
                    FieldDefinition(
                        name="patient_id",
                        display_name="Patient/Subject ID",
                        description="De-identified patient or subject identifier",
                        data_type="string",
                        examples=["P001", "Subject_123", "Mouse_A1"]
                    ),
                    FieldDefinition(
                        name="consent_status",
                        display_name="Consent Status",
                        description="IRB or consent information",
                        data_type="string",
                        examples=["IRB approved", "Consent obtained", "Exempt"]
                    ),
                    FieldDefinition(
                        name="custom_notes",
                        display_name="Additional Notes",
                        description="Any other relevant information",
                        data_type="string",
                        examples=["Treatment-naive", "Post-surgery", "Control sample"]
                    )
                ]
            )
        ]
    
    def get_category_by_name(self, name: str) -> CategoryDefinition:
        """Get a category by name"""
        for category in self.categories:
            if category.name == name:
                return category
        raise ValueError(f"Category '{name}' not found")
    
    def get_categories_by_type(self, category_type: CategoryType) -> List[CategoryDefinition]:
        """Get all categories of a specific type"""
        return [cat for cat in self.categories if cat.type == category_type]
    
    def add_custom_category(self, category: CategoryDefinition):
        """Add a custom category"""
        self.categories.append(category)
    
    def generate_extraction_prompt(self) -> str:
        """Generate extraction prompt based on configured categories"""
        prompt = """
You are an expert laboratory information extraction system. Extract information from the laboratory submission document below and format it as JSON.

Focus on these categories and fields:

"""
        
        for category in sorted(self.categories, key=lambda x: x.priority):
            prompt += f"**{category.name.upper()}:**\n"
            prompt += f"{category.description}\n"
            
            for field in category.fields:
                required_str = " (REQUIRED)" if field.required else ""
                prompt += f"- {field.name}: {field.description}{required_str}\n"
                if field.examples:
                    prompt += f"  Examples: {', '.join(field.examples[:3])}\n"
            prompt += "\n"
        
        # Generate JSON template
        prompt += "Respond with valid JSON only, using null for missing information:\n\n{\n"
        
        all_fields = []
        for category in self.categories:
            all_fields.extend([field.name for field in category.fields])
        
        for i, field_name in enumerate(all_fields):
            comma = "," if i < len(all_fields) - 1 else ""
            prompt += f'  "{field_name}": "value or null"{comma}\n'
        
        prompt += "}\n\nDocument to analyze:\n{text}"
        
        return prompt
    
    def export_configuration(self) -> Dict[str, Any]:
        """Export configuration as dictionary"""
        return {
            "categories": [cat.dict() for cat in self.categories],
            "total_categories": len(self.categories),
            "total_fields": sum(len(cat.fields) for cat in self.categories),
            "required_fields": [
                field.name for cat in self.categories 
                for field in cat.fields if field.required
            ]
        }

def create_custom_genomics_config():
    """Example: Create a custom configuration for genomics laboratory"""
    config = LabCategoryConfig()
    
    # Add genomics-specific category
    genomics_category = CategoryDefinition(
        name="Genomics Specific",
        type=CategoryType.CUSTOM,
        description="Genomics-specific information and requirements",
        priority=8,
        fields=[
            FieldDefinition(
                name="reference_genome",
                display_name="Reference Genome",
                description="Reference genome version for analysis",
                data_type="string",
                examples=["GRCh38", "GRCh37", "mm10", "dm6"]
            ),
            FieldDefinition(
                name="variant_calling",
                display_name="Variant Calling Required",
                description="Whether variant calling is needed",
                data_type="boolean",
                examples=["Yes", "No", "SNPs only", "Structural variants"]
            ),
            FieldDefinition(
                name="annotation_databases",
                display_name="Annotation Databases",
                description="Required annotation databases",
                data_type="string",
                examples=["ClinVar", "dbSNP", "COSMIC", "OMIM"]
            )
        ]
    )
    
    config.add_custom_category(genomics_category)
    return config

# Example usage and testing
if __name__ == "__main__":
    print("🧬 Laboratory Categories Configuration")
    print("=" * 50)
    
    # Standard configuration
    config = LabCategoryConfig()
    
    print(f"📋 Standard Configuration:")
    print(f"   Categories: {len(config.categories)}")
    print(f"   Total fields: {sum(len(cat.fields) for cat in config.categories)}")
    
    print(f"\n📝 Categories:")
    for cat in config.categories:
        required_count = sum(1 for field in cat.fields if field.required)
        print(f"   {cat.priority}. {cat.name} ({len(cat.fields)} fields, {required_count} required)")
    
    # Generate extraction prompt
    print(f"\n🤖 Generated Extraction Prompt:")
    prompt = config.generate_extraction_prompt()
    print(f"   Length: {len(prompt)} characters")
    print(f"   First 200 chars: {prompt[:200]}...")
    
    # Custom genomics configuration
    print(f"\n🧬 Custom Genomics Configuration:")
    genomics_config = create_custom_genomics_config()
    print(f"   Categories: {len(genomics_config.categories)}")
    print(f"   Added genomics-specific fields")
    
    print(f"\n✅ Configuration system ready!")
    print(f"💡 You can now customize categories for your specific laboratory workflow") 
