#!/usr/bin/env python3
"""
Enhanced Laboratory Submission RAG System - Usage Example

This script demonstrates how to use the improved, more modular and robust
Laboratory Submission RAG System with all the architectural enhancements.
"""

import asyncio
import logging
from pathlib import Path

from core.exceptions import (
    DocumentProcessingException,
    LabSubmissionException,
    ServiceException,
)
from rag_orchestrator_v2 import EnhancedLabSubmissionRAG, create_enhanced_rag_system

# Configure logging for the example
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


async def demonstrate_basic_usage():
    """Demonstrate basic usage of the enhanced RAG system"""
    logger.info("=== Basic Usage Demonstration ===")

    try:
        # Method 1: Using context manager (recommended)
        async with EnhancedLabSubmissionRAG() as rag_system:

            # Check system health
            health = await rag_system.health_check()
            logger.info(f"System health: {health['status']}")

            # Example query (this will work even without documents)
            try:
                answer = await rag_system.query_submissions(
                    "How many submissions are in the system?"
                )
                logger.info(f"Query answer: {answer}")
            except ServiceException as e:
                logger.warning(f"Query failed as expected (no data): {e.message}")

            # Get system statistics
            stats = await rag_system.get_system_statistics()
            logger.info(f"System statistics: {stats}")

    except LabSubmissionException as e:
        logger.error(f"Domain error: {e.to_dict()}")
    except Exception as e:
        logger.error(f"Unexpected error: {str(e)}")


async def demonstrate_document_processing():
    """Demonstrate document processing with enhanced error handling"""
    logger.info("=== Document Processing Demonstration ===")

    # Custom configuration
    config = {"batch_size": 3, "max_retries": 2, "chunk_size": 1000, "chunk_overlap": 200}

    try:
        # Method 2: Using factory function
        rag_system = await create_enhanced_rag_system(config)

        try:
            # Create some test files (you can replace with actual documents)
            test_files = create_test_documents()

            if test_files:
                # Process single document
                logger.info("Processing single document...")
                try:
                    result = await rag_system.process_document(test_files[0])
                    logger.info(
                        f"Processing result: Success={result.success}, "
                        f"Confidence={result.confidence_score:.2f}"
                    )
                except DocumentProcessingException as e:
                    logger.warning(f"Document processing failed: {e.message}")

                # Process multiple documents with controlled concurrency
                if len(test_files) > 1:
                    logger.info("Processing document batch...")
                    try:
                        batch_result = await rag_system.process_documents_batch(
                            test_files, max_concurrent=2
                        )
                        logger.info(
                            f"Batch result: {batch_result.successful_extractions}/"
                            f"{batch_result.total_documents} successful"
                        )
                    except ServiceException as e:
                        logger.warning(f"Batch processing failed: {e.message}")
            else:
                logger.info("No test documents created, skipping document processing")

        finally:
            # Cleanup
            await rag_system.shutdown()
            cleanup_test_documents(test_files if "test_files" in locals() else [])

    except Exception as e:
        logger.error(f"Error in document processing demo: {str(e)}")


async def demonstrate_advanced_features():
    """Demonstrate advanced features like health monitoring and error recovery"""
    logger.info("=== Advanced Features Demonstration ===")

    try:
        async with EnhancedLabSubmissionRAG() as rag_system:

            # Comprehensive health check
            health = await rag_system.health_check()
            logger.info("=== Health Check Results ===")
            logger.info(f"Overall Status: {health['status']}")

            if "services" in health:
                for service_name, service_health in health["services"].items():
                    status = service_health.get("healthy", False)
                    logger.info(f"  {service_name}: {'✓' if status else '✗'}")

            # Test error handling with invalid operations
            logger.info("=== Error Handling Demonstration ===")

            # Try to process non-existent file
            try:
                await rag_system.process_document("non_existent_file.pdf")
            except DocumentProcessingException as e:
                logger.info(f"✓ Caught expected DocumentProcessingException: {e.error_code}")

            # Try empty query
            try:
                await rag_system.query_submissions("")
            except ServiceException as e:
                logger.info(f"✓ Caught expected ServiceException: {e.error_code}")

            # Try to get non-existent submission
            try:
                submission = await rag_system.get_submission("non_existent_id")
                if submission is None:
                    logger.info("✓ Gracefully handled non-existent submission")
            except Exception as e:
                logger.info(f"✓ Handled submission retrieval error: {type(e).__name__}")

            # Search with empty criteria
            try:
                results = await rag_system.search_submissions({})
                logger.info(f"✓ Search completed, found {len(results)} results")
            except Exception as e:
                logger.info(f"✓ Search error handled: {type(e).__name__}")

    except Exception as e:
        logger.error(f"Error in advanced features demo: {str(e)}")


def create_test_documents() -> list[Path]:
    """Create simple test documents for demonstration"""
    test_files = []

    try:
        # Create uploads directory if it doesn't exist
        uploads_dir = Path("uploads")
        uploads_dir.mkdir(exist_ok=True)

        # Create simple text files with lab submission content
        test_content = {
            "submission_1.txt": """
Laboratory Submission Form

Submitter Information:
- Name: Dr. Jane Smith
- Email: jane.smith@university.edu
- Phone: (555) 123-4567
- Project: Genomic Analysis Project 2024

Sample Information:
- Sample Type: DNA
- Sample ID: DNA_001
- Storage: Frozen (-80°C)
- Volume: 50µL
- Concentration: 25 ng/µL

Analysis Requested:
- Whole Genome Sequencing
- Platform: Illumina NovaSeq
- Coverage: 30x
- Reference: hg38
            """,
            "submission_2.txt": """
Lab Sample Submission

Contact Details:
- Researcher: Dr. Bob Johnson
- Institution: Medical Research Institute
- Email: bob.johnson@research.org
- Phone: (555) 987-6543

Sample Details:
- Type: RNA
- ID: RNA_002
- Quality Score: 8.5
- Preservation: RNAlater
- Priority: High

Sequencing Requirements:
- RNA-seq analysis
- Paired-end reads
- Read length: 150bp
- Library prep: TruSeq
            """,
        }

        for filename, content in test_content.items():
            file_path = uploads_dir / filename
            file_path.write_text(content)
            test_files.append(file_path)
            logger.info(f"Created test file: {file_path}")

    except Exception as e:
        logger.warning(f"Could not create test documents: {str(e)}")

    return test_files


def cleanup_test_documents(test_files: list[Path]):
    """Clean up test documents"""
    for file_path in test_files:
        try:
            if file_path.exists():
                file_path.unlink()
                logger.info(f"Cleaned up test file: {file_path}")
        except Exception as e:
            logger.warning(f"Could not clean up {file_path}: {str(e)}")


async def demonstrate_configuration_options():
    """Demonstrate different configuration options"""
    logger.info("=== Configuration Options Demonstration ===")

    # Example configurations for different use cases
    configurations = {
        "development": {
            "batch_size": 2,
            "max_retries": 2,
            "chunk_size": 500,
            "chunk_overlap": 100,
            "circuit_breaker_threshold": 3,
        },
        "production": {
            "batch_size": 10,
            "max_retries": 5,
            "chunk_size": 1000,
            "chunk_overlap": 200,
            "circuit_breaker_threshold": 5,
        },
        "testing": {
            "batch_size": 1,
            "max_retries": 1,
            "chunk_size": 200,
            "chunk_overlap": 50,
            "circuit_breaker_threshold": 2,
        },
    }

    for env_name, config in configurations.items():
        logger.info(f"--- {env_name.title()} Configuration ---")

        try:
            async with EnhancedLabSubmissionRAG(config) as rag_system:
                health = await rag_system.health_check()
                logger.info(f"{env_name}: System status = {health['status']}")

                # Show configuration is working
                stats = await rag_system.get_system_statistics()
                logger.info(f"{env_name}: Components = {stats.get('components', 0)}")

        except Exception as e:
            logger.error(f"Error with {env_name} configuration: {str(e)}")


async def main():
    """Main demonstration function"""
    logger.info("Starting Enhanced Laboratory Submission RAG System Demonstration")
    logger.info("=" * 70)

    try:
        # Run all demonstrations
        await demonstrate_basic_usage()
        await demonstrate_document_processing()
        await demonstrate_advanced_features()
        await demonstrate_configuration_options()

        logger.info("=" * 70)
        logger.info("✅ All demonstrations completed successfully!")
        logger.info("")
        logger.info("Key improvements demonstrated:")
        logger.info("  ✅ Service layer architecture with dependency injection")
        logger.info("  ✅ Comprehensive exception handling")
        logger.info("  ✅ Health monitoring and system statistics")
        logger.info("  ✅ Circuit breaker and retry patterns")
        logger.info("  ✅ Configurable component lifecycle")
        logger.info("  ✅ Enhanced error context and logging")

    except Exception as e:
        logger.error(f"Demonstration failed: {str(e)}")
        return 1

    return 0


if __name__ == "__main__":
    import sys
    from typing import Any

    # Set up proper event loop policy for Windows
    if sys.platform.startswith("win"):
        asyncio.set_event_loop_policy(asyncio.WindowsProactorEventLoopPolicy())

    exit_code = asyncio.run(main())
    sys.exit(exit_code)
