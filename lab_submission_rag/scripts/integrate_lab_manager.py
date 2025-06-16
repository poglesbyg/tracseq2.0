#!/usr/bin/env python3
"""
Integration script for connecting RAG system with existing lab_manager database
"""

import asyncio
import logging
import sys
from pathlib import Path
from typing import List, Dict, Any

# Add the parent directory to the path so we can import our modules
sys.path.append(str(Path(__file__).parent.parent))

from database import db_manager
from config import settings
from sqlalchemy import text, inspect
from sqlalchemy.ext.asyncio import AsyncSession

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

async def check_database_connection():
    """Check if we can connect to the lab_manager database"""
    try:
        await db_manager.initialize()
        async with db_manager.get_session() as session:
            result = await session.execute(text("SELECT version()"))
            version = result.scalar()
            logger.info(f"Successfully connected to PostgreSQL: {version}")
            return True
    except Exception as e:
        logger.error(f"Failed to connect to database: {e}")
        return False

async def list_existing_tables():
    """List all existing tables in the database"""
    try:
        async with db_manager.get_session() as session:
            # Get table names
            result = await session.execute(text("""
                SELECT table_name 
                FROM information_schema.tables 
                WHERE table_schema = 'public' 
                ORDER BY table_name
            """))
            tables = [row[0] for row in result.fetchall()]
            
            logger.info("Existing tables in database:")
            for table in tables:
                logger.info(f"  - {table}")
            
            return tables
    except Exception as e:
        logger.error(f"Failed to list tables: {e}")
        return []

async def check_table_conflicts():
    """Check if RAG tables would conflict with existing tables"""
    try:
        existing_tables = await list_existing_tables()
        
        # RAG table names with prefix
        prefix = settings.table_prefix
        rag_tables = [
            f"{prefix}lab_submissions",
            f"{prefix}samples",
            f"{prefix}documents",
            f"{prefix}document_chunks",
            f"{prefix}extraction_results",
            f"{prefix}query_logs",
            f"{prefix}pooling_info",
            f"{prefix}sequence_generation",
            f"{prefix}informatics_info"
        ]
        
        conflicts = [table for table in rag_tables if table in existing_tables]
        
        if conflicts:
            logger.warning("Table name conflicts detected:")
            for conflict in conflicts:
                logger.warning(f"  - {conflict}")
            return conflicts
        else:
            logger.info("No table name conflicts detected")
            return []
            
    except Exception as e:
        logger.error(f"Failed to check table conflicts: {e}")
        return []

async def create_rag_tables():
    """Create RAG system tables in the lab_manager database"""
    try:
        logger.info("Creating RAG system tables...")
        await db_manager.create_tables()
        logger.info("RAG tables created successfully!")
        
        # List the created tables
        await list_existing_tables()
        
    except Exception as e:
        logger.error(f"Failed to create RAG tables: {e}")
        raise

async def analyze_lab_manager_schema():
    """Analyze existing lab_manager schema to understand the structure"""
    try:
        async with db_manager.get_session() as session:
            # Get table information
            result = await session.execute(text("""
                SELECT 
                    table_name,
                    column_name,
                    data_type,
                    is_nullable,
                    column_default
                FROM information_schema.columns 
                WHERE table_schema = 'public' 
                AND table_name NOT LIKE 'rag_%'
                ORDER BY table_name, ordinal_position
            """))
            
            schema_info = {}
            for row in result.fetchall():
                table_name, column_name, data_type, is_nullable, column_default = row
                if table_name not in schema_info:
                    schema_info[table_name] = []
                schema_info[table_name].append({
                    'column': column_name,
                    'type': data_type,
                    'nullable': is_nullable,
                    'default': column_default
                })
            
            logger.info("Lab Manager Database Schema Analysis:")
            for table_name, columns in schema_info.items():
                logger.info(f"\nTable: {table_name}")
                for col in columns:
                    logger.info(f"  - {col['column']}: {col['type']} ({'NULL' if col['nullable'] == 'YES' else 'NOT NULL'})")
            
            return schema_info
            
    except Exception as e:
        logger.error(f"Failed to analyze schema: {e}")
        return {}

async def suggest_integration_strategy():
    """Suggest integration strategy based on existing schema"""
    logger.info("\n" + "="*60)
    logger.info("INTEGRATION STRATEGY SUGGESTIONS")
    logger.info("="*60)
    
    # Check existing tables
    existing_tables = await list_existing_tables()
    
    # Look for common lab management table patterns
    sample_tables = [t for t in existing_tables if 'sample' in t.lower()]
    submission_tables = [t for t in existing_tables if any(word in t.lower() for word in ['submission', 'request', 'order'])]
    
    if sample_tables:
        logger.info(f"\nFound existing sample-related tables: {sample_tables}")
        logger.info("RECOMMENDATION: Consider creating views or adapters to connect RAG data with existing sample data")
    
    if submission_tables:
        logger.info(f"\nFound existing submission-related tables: {submission_tables}")
        logger.info("RECOMMENDATION: Consider linking RAG submissions to existing submission workflow")
    
    logger.info(f"\nRAG tables will be created with prefix '{settings.table_prefix}' to avoid conflicts")
    logger.info("This allows both systems to coexist in the same database")

async def main():
    """Main integration workflow"""
    logger.info("Starting lab_manager database integration...")
    
    # Step 1: Check database connection
    if not await check_database_connection():
        logger.error("Cannot proceed without database connection")
        return
    
    # Step 2: Analyze existing schema
    await analyze_lab_manager_schema()
    
    # Step 3: Check for conflicts
    conflicts = await check_table_conflicts()
    
    # Step 4: Suggest integration strategy
    await suggest_integration_strategy()
    
    # Step 5: Create RAG tables if no conflicts
    if not conflicts:
        create_tables = input("\nCreate RAG tables now? (y/n): ")
        if create_tables.lower() == 'y':
            await create_rag_tables()
    else:
        logger.warning("Please resolve table conflicts before creating RAG tables")
        logger.info("Consider changing the TABLE_PREFIX in your environment variables")
    
    await db_manager.close()

if __name__ == "__main__":
    asyncio.run(main()) 
