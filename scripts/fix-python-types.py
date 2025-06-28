#!/usr/bin/env python3
"""
Comprehensive Python Type Fixing Script for TracSeq 2.0
Automatically fixes the most common type annotation issues.
"""

import re
import os
import sys
from pathlib import Path
from typing import List, Tuple, Dict

# Common type fixes mapping
TYPE_FIXES = {
    # Missing return type annotations for common patterns
    r'def (\w+)\(self\)\s*:': r'def \1(self) -> None:',
    r'def (\w+)\(self,([^)]*)\)\s*:': r'def \1(self,\2) -> None:',
    r'async def (\w+)\(self\)\s*:': r'async def \1(self) -> None:',
    r'async def (\w+)\(self,([^)]*)\)\s*:': r'async def \1(self,\2) -> None:',
    
    # Common function patterns
    r'def __init__\(self([^)]*)\)\s*:': r'def __init__(self\1) -> None:',
    r'def cleanup\(self\)\s*:': r'def cleanup(self) -> None:',
    r'def setup\(self\)\s*:': r'def setup(self) -> None:',
    r'def teardown\(self\)\s*:': r'def teardown(self) -> None:',
    
    # Type annotation improvements
    r'List\[': r'list[',
    r'Dict\[': r'dict[',
    r'Tuple\[': r'tuple[',
    r'Optional\[([^\]]+)\]': r'\1 | None',
    r'Union\[([^,]+),\s*None\]': r'\1 | None',
}

# Files to skip (too complex or external)
SKIP_FILES = {
    'mlops/monitoring.py',
    'mlops/deployment_manager.py', 
    'mlops/data_pipeline.py',
    'mlops/continuous_learning.py',
    'rag/llm_interface.py',  # Complex OpenAI/LLM integrations
}

# Patterns for specific return types
RETURN_TYPE_PATTERNS = [
    # Fixtures and test functions
    (r'@pytest\.fixture[^\n]*\ndef (\w+)\([^)]*\) -> ([^:]+):', r'@pytest.fixture\ndef \1(\2) -> \3:'),
    
    # Mock creation functions
    (r'def create_mock_(\w+)\([^)]*\)\s*:', r'def create_mock_\1(\2) -> Mock:'),
    (r'def create_(\w+)_mock\([^)]*\)\s*:', r'def create_\1_mock(\2) -> Mock:'),
    
    # Test helper functions
    (r'def assert_(\w+)\([^)]*\)\s*:', r'def assert_\1(\2) -> None:'),
    (r'def test_(\w+)\([^)]*\)\s*:', r'def test_\1(\2) -> None:'),
    
    # Async test functions
    (r'async def test_(\w+)\([^)]*\)\s*:', r'async def test_\1(\2) -> None:'),
]


def fix_missing_imports(content: str) -> str:
    """Add missing imports for type annotations."""
    imports_to_add = []
    
    # Check if we need typing imports
    if ' -> list[' in content or ' -> dict[' in content or ' -> tuple[' in content:
        if 'from typing import' not in content:
            imports_to_add.append('from typing import Any')
    
    if ' -> Mock' in content and 'from unittest.mock import' not in content:
        imports_to_add.append('from unittest.mock import Mock, AsyncMock')
    
    if imports_to_add:
        # Find the last import line
        lines = content.split('\n')
        last_import_idx = -1
        for i, line in enumerate(lines):
            if line.strip().startswith(('import ', 'from ')) and not line.strip().startswith('#'):
                last_import_idx = i
        
        if last_import_idx >= 0:
            for import_line in reversed(imports_to_add):
                lines.insert(last_import_idx + 1, import_line)
            content = '\n'.join(lines)
    
    return content


def fix_function_annotations(content: str) -> str:
    """Fix missing function return type annotations."""
    
    # Common patterns for functions that should return None
    none_return_patterns = [
        r'def (__init__\([^)]*\))\s*:',
        r'def (cleanup\([^)]*\))\s*:',
        r'def (setup\([^)]*\))\s*:',
        r'def (teardown\([^)]*\))\s*:',
        r'def (test_\w+\([^)]*\))\s*:',
        r'async def (test_\w+\([^)]*\))\s*:',
        r'def (assert_\w+\([^)]*\))\s*:',
    ]
    
    for pattern in none_return_patterns:
        content = re.sub(pattern, r'def \1 -> None:', content)
    
    return content


def fix_file_types(file_path: Path) -> bool:
    """Fix type annotations in a single file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Apply basic type fixes
        for pattern, replacement in TYPE_FIXES.items():
            content = re.sub(pattern, replacement, content)
        
        # Fix function annotations
        content = fix_function_annotations(content)
        
        # Fix missing imports
        content = fix_missing_imports(content)
        
        # Only write if content changed
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            return True
        
        return False
    
    except Exception as e:
        print(f"❌ Error processing {file_path}: {e}")
        return False


def should_skip_file(file_path: Path, base_path: Path) -> bool:
    """Check if a file should be skipped."""
    relative_path = file_path.relative_to(base_path)
    return str(relative_path) in SKIP_FILES or file_path.name.startswith('.')


def main():
    """Main type fixing function."""
    print("🔧 PYTHON TYPE ANNOTATION FIXER")
    print("================================")
    print()
    
    # Find Python files to process
    base_path = Path.cwd()
    python_files = []
    
    for root, dirs, files in os.walk(base_path):
        # Skip common directories
        dirs[:] = [d for d in dirs if d not in {'.git', '__pycache__', '.pytest_cache', 'node_modules', 'target'}]
        
        for file in files:
            if file.endswith('.py'):
                file_path = Path(root) / file
                if not should_skip_file(file_path, base_path):
                    python_files.append(file_path)
    
    print(f"📁 Found {len(python_files)} Python files to process")
    print()
    
    # Process files
    fixed_files = []
    for file_path in python_files:
        try:
            if fix_file_types(file_path):
                fixed_files.append(file_path)
                print(f"✅ Fixed: {file_path.relative_to(base_path)}")
            else:
                print(f"⏭️  Skipped: {file_path.relative_to(base_path)} (no changes needed)")
                
        except Exception as e:
            print(f"❌ Error: {file_path.relative_to(base_path)} - {e}")
    
    print()
    print("📊 SUMMARY")
    print("==========")
    print(f"🔧 Files processed: {len(python_files)}")
    print(f"✅ Files fixed: {len(fixed_files)}")
    print(f"⏭️  Files skipped: {len(python_files) - len(fixed_files)}")
    
    if fixed_files:
        print()
        print("🎯 KEY FIXES APPLIED:")
        print("  • Added missing return type annotations")
        print("  • Fixed function signatures")
        print("  • Added missing imports")
        print("  • Modernized type annotations (List -> list, etc.)")
        print()
        print("🔍 Run mypy again to check remaining issues:")
        print("  uv run mypy . --no-error-summary")


if __name__ == "__main__":
    main() 