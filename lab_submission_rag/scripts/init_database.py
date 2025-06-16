#!/usr/bin/env python3
"""
Database initialization script for Laboratory Submission RAG system
"""

import asyncio
import logging
import sys
from pathlib import Path

# Add the parent directory to the path so we can import our modules
sys.path.append(str(Path(__file__).parent.parent))

from database import db_manager
from config import settings

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

async def init_database():
    """Initialize the database"""
    try:
        logger.info("Initializing database connection...")
        await db_manager.initialize()
        
        logger.info("Creating database tables...")
        await db_manager.create_tables()
        
        logger.info("Database initialization completed successfully!")
        
    except Exception as e:
        logger.error(f"Database initialization failed: {e}")
        raise
    finally:
        await db_manager.close()

async def reset_database():
    """Reset the database (drop and recreate all tables)"""
    try:
        logger.info("Initializing database connection...")
        await db_manager.initialize()
        
        logger.info("Dropping existing tables...")
        await db_manager.drop_tables()
        
        logger.info("Creating database tables...")
        await db_manager.create_tables()
        
        logger.info("Database reset completed successfully!")
        
    except Exception as e:
        logger.error(f"Database reset failed: {e}")
        raise
    finally:
        await db_manager.close()

if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description="Database initialization script")
    parser.add_argument("--reset", action="store_true", help="Reset database (drop and recreate tables)")
    args = parser.parse_args()
    
    if args.reset:
        print("WARNING: This will delete all existing data!")
        confirm = input("Are you sure you want to reset the database? (yes/no): ")
        if confirm.lower() == "yes":
            asyncio.run(reset_database())
        else:
            print("Database reset cancelled.")
    else:
        asyncio.run(init_database()) 
