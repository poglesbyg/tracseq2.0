#!/usr/bin/env python3
"""
Test script for enhanced RAG intelligence
"""

import asyncio
import sys
from pathlib import Path

# Add the app directory to the path
sys.path.insert(0, str(Path(__file__).parent))

from rag.enhanced_llm_interface import enhanced_llm


async def test_enhanced_intelligence() -> None:
    """Test the enhanced LLM interface with lab-specific queries"""

    print("🧠 Testing Enhanced Lab Assistant Intelligence")
    print("=" * 50)

    test_queries = [
        "How do I submit a new sample?",
        "What are the storage requirements for DNA samples?",
        "I need to set up a sequencing job. Can you help?",
        "What's the difference between batch and individual sample submission?",
        "How do I use the AI document processing feature?",
    ]

    session_id = "test_session_123"

    for i, query in enumerate(test_queries, 1):
        print(f"\n📝 Test Query {i}: {query}")
        print("-" * 30)

        try:
            # Test with empty chunks (should use system knowledge)
            response = await enhanced_llm.answer_query(
                query=query, relevant_chunks=[], session_id=session_id
            )

            print(f"🤖 Response: {response[:200]}...")
            if len(response) > 200:
                print("   [Response truncated - full response available]")

            # Verify response quality
            if len(response) > 50 and any(
                keyword in response.lower() for keyword in ["lab", "sample", "navigate", "system"]
            ):
                print("✅ Quality check: PASSED")
            else:
                print("❌ Quality check: FAILED (response too short or not lab-specific)")

        except Exception as e:
            print(f"❌ Error: {str(e)}")

        print()

    # Test conversation memory
    print("\n🧠 Testing Conversation Memory")
    print("-" * 30)

    try:
        # First question
        response1 = await enhanced_llm.answer_query(
            "Tell me about sample storage.", [], session_id=session_id
        )
        print(f"First response length: {len(response1)} characters")

        # Follow-up question (should reference previous context)
        response2 = await enhanced_llm.answer_query(
            "What about for RNA specifically?", [], session_id=session_id
        )
        print(f"Follow-up response length: {len(response2)} characters")

        if "rna" in response2.lower() and len(response2) > 50:
            print("✅ Conversation memory: WORKING")
        else:
            print("❌ Conversation memory: NOT WORKING PROPERLY")

    except Exception as e:
        print(f"❌ Conversation memory test failed: {str(e)}")

    print("\n🎉 Enhanced Intelligence Test Complete!")
    print("\nNext steps:")
    print("1. Start the RAG system: docker-compose up -d")
    print("2. Start Lab Manager: ./scripts/run.sh start")
    print("3. Test the chatbot in the web interface")


if __name__ == "__main__":
    asyncio.run(test_enhanced_intelligence())
