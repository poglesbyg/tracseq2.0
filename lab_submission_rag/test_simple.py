#!/usr/bin/env python3
"""
Test script for Simple Laboratory Submission RAG System

This script verifies that the lightweight system works correctly.
"""

import os
import sys
from pathlib import Path

def check_environment():
    """Check if environment is set up correctly"""
    print("🔍 Checking environment...")
    
    # Check if required files exist
    required_files = [
        "simple_lab_rag.py",
        "requirements-lite.txt"
    ]
    
    for file in required_files:
        if not Path(file).exists():
            print(f"❌ Missing file: {file}")
            return False
        print(f"✅ Found: {file}")
    
    # Check API key
    if not os.getenv("OPENAI_API_KEY"):
        print("⚠️  OPENAI_API_KEY not set")
        print("   Create .env file with: OPENAI_API_KEY=your_key")
        return False
    
    print("✅ API key configured")
    return True

def test_imports():
    """Test if all imports work"""
    print("\n🔍 Testing imports...")
    
    try:
        import simple_lab_rag
        print("✅ simple_lab_rag imported successfully")
        
        # Test class instantiation
        rag = simple_lab_rag.SimpleLabRAG()
        print("✅ SimpleLabRAG class instantiated")
        
        return True
    except Exception as e:
        print(f"❌ Import failed: {e}")
        return False

def test_basic_functionality():
    """Test basic functionality"""
    print("\n🔍 Testing basic functionality...")
    
    try:
        from simple_lab_rag import SimpleLabRAG
        
        # Initialize system
        rag = SimpleLabRAG()
        
        # Test stats
        stats = rag.get_stats()
        print(f"✅ System stats: {stats['total_submissions']} submissions")
        
        # Test query on empty system
        answer = rag.query("test query")
        if "no relevant information" in answer.lower():
            print("✅ Query handling works (empty system)")
        else:
            print(f"⚠️  Unexpected response: {answer}")
        
        return True
    except Exception as e:
        print(f"❌ Basic functionality test failed: {e}")
        return False

def test_document_processing():
    """Test document processing with demo document"""
    print("\n🔍 Testing document processing...")
    
    try:
        from simple_lab_rag import SimpleLabRAG, create_demo_document
        
        # Create demo document
        demo_file = create_demo_document()
        print(f"✅ Demo document created: {demo_file}")
        
        # Initialize system
        rag = SimpleLabRAG()
        
        # Process document
        result = rag.process_document(demo_file)
        
        if result.success:
            print("✅ Document processed successfully")
            print(f"   Submitter: {result.submission.administrative.submitter_name}")
            print(f"   Sample Type: {result.submission.sample.sample_type}")
            
            # Test query after processing
            answer = rag.query("Who is the submitter?")
            if "sarah" in answer.lower():
                print("✅ Query works after processing")
            else:
                print(f"⚠️  Query result: {answer}")
            
            return True
        else:
            print(f"❌ Document processing failed: {result.error}")
            return False
            
    except Exception as e:
        print(f"❌ Document processing test failed: {e}")
        return False

def main():
    """Run all tests"""
    print("🧪 Testing Simple Laboratory Submission RAG System")
    print("=" * 55)
    
    tests = [
        ("Environment Check", check_environment),
        ("Import Test", test_imports),
        ("Basic Functionality", test_basic_functionality),
        ("Document Processing", test_document_processing)
    ]
    
    passed = 0
    total = len(tests)
    
    for test_name, test_func in tests:
        try:
            if test_func():
                passed += 1
            else:
                print(f"\n❌ {test_name} failed")
        except Exception as e:
            print(f"\n❌ {test_name} crashed: {e}")
    
    print("\n" + "=" * 55)
    print(f"🧪 Test Results: {passed}/{total} tests passed")
    
    if passed == total:
        print("🎉 All tests passed! System is working correctly.")
        return 0
    else:
        print("⚠️  Some tests failed. Check the setup.")
        return 1

if __name__ == "__main__":
    sys.exit(main()) 
