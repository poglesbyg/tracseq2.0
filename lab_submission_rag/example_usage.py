"""
Example Usage of the Laboratory Submission RAG System

This script demonstrates how to:
1. Process laboratory documents to extract submission information
2. Query the system about specific submission details
3. Export extracted data

Run this script after setting up API keys in a .env file:
OPENAI_API_KEY=your_openai_key
# OR
ANTHROPIC_API_KEY=your_anthropic_key
"""

import asyncio


async def main():
    """Demonstrate the RAG system capabilities"""

    print("🧬 Laboratory Submission RAG System Demo")
    print("=" * 50)

    # Example 1: Process a single document
    print("\n📄 Example 1: Processing a Laboratory Document")
    print("-" * 40)

    # Note: Replace with actual document path
    # sample_document = Path("sample_documents/lab_submission_form.pdf")

    # For demo purposes, we'll show what the output would look like
    print("Sample document processing (mock output):")

    # This would be the actual call:
    # result = await rag_system.process_document(sample_document)

    # Mock result for demonstration
    print(
        """
    ✅ Document processed successfully!
    
    Extracted Information:
    1. Administrative Information:
       - Submitter: John Doe
       - Email: john.doe@example.com
       - Project: PROJ-2024-001
       
    2. Source Material:
       - Type: Genomic DNA
       - Source: Blood samples
       
    3. Sequence Generation:
       - Platform: Illumina NovaSeq
       - Coverage: 30x
       
    4. Sample Details:
       - Sample ID: SAMPLE-001
       - Priority: High
       - Quality Score: 8.5/10
    
    Confidence Score: 85%
    Missing Fields: ['submitter_phone']
    """
    )

    # Example 2: Query the system
    print("\n❓ Example 2: Querying Laboratory Submissions")
    print("-" * 40)

    sample_queries = [
        "What sequencing platform is being used?",
        "Who is the submitter for this project?",
        "What is the sample quality score?",
        "What type of analysis is requested?",
        "What are the storage requirements?",
    ]

    print("Sample queries and responses:")
    for i, query in enumerate(sample_queries, 1):
        print(f"\n{i}. Query: '{query}'")
        # This would be the actual call:
        # answer = await rag_system.query_submissions(query)
        print("   Answer: [Would provide detailed answer based on stored documents]")

    # Example 3: System status
    print("\n📊 Example 3: System Status")
    print("-" * 40)

    # This would show actual system status:
    # status = await rag_system.get_system_status()

    print(
        """
    System Status:
    - Status: Operational
    - Documents Processed: 5
    - Total Chunks: 127
    - Embedding Model: all-MiniLM-L6-v2
    - Supported Categories: 7
    """
    )

    # Example 4: Supported Categories
    print("\n📋 Example 4: Supported Information Categories")
    print("-" * 40)

    categories = [
        "1. Administrative Information",
        "   - Submitter details (name, email, phone)",
        "   - Project assignment",
        "   - Contact information",
        "",
        "2. Source and Submitting Material",
        "   - Material type (Genomic DNA, RNA, Other)",
        "   - Collection details",
        "   - Storage conditions",
        "",
        "3. Pooling (Multiplexing)",
        "   - Pooling strategy",
        "   - Barcode information",
        "   - Sample pooling ratios",
        "",
        "4. Sequence Generation",
        "   - Sequencing platform",
        "   - Read length and type",
        "   - Coverage requirements",
        "   - Library preparation",
        "",
        "5. Container and Diluent",
        "   - Container specifications",
        "   - Volume and concentration",
        "   - Storage temperature",
        "",
        "6. Informatics",
        "   - Analysis type (WGS, WES, RNA-seq)",
        "   - Reference genome",
        "   - Pipeline requirements",
        "",
        "7. Sample Details",
        "   - Sample identifiers",
        "   - Quality metrics",
        "   - Priority levels",
        "   - Special instructions",
    ]

    for line in categories:
        print(line)

    # Example usage patterns
    print("\n💡 Example Usage Patterns")
    print("-" * 40)

    usage_examples = [
        "# Process a single document",
        "result = await rag_system.process_document('lab_form.pdf')",
        "",
        "# Process multiple documents",
        "results = await rag_system.process_documents_batch(['form1.pdf', 'form2.pdf'])",
        "",
        "# Query for specific information",
        "answer = await rag_system.query_submissions('What is the sample type?')",
        "",
        "# Export extracted data",
        "if result.success:",
        "    export_path = await rag_system.export_submission_data(result.submission, 'json')",
        "",
        "# Get system status",
        "status = await rag_system.get_system_status()",
    ]

    for line in usage_examples:
        print(line)

    print("\n🚀 Ready to process laboratory documents!")
    print("\nTo use with real documents:")
    print("1. Place documents in the uploads/ directory")
    print("2. Set up API keys in .env file")
    print(
        "3. Run: python -c \"import asyncio; from rag_orchestrator import rag_system; asyncio.run(rag_system.process_document('your_document.pdf'))\""
    )


if __name__ == "__main__":
    asyncio.run(main())
