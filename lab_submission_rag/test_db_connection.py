#!/usr/bin/env python3
"""
Test database connection
"""

import asyncio

import asyncpg


async def test_connection() -> None:
    try:
        print("Attempting to connect to database...")
        conn = await asyncpg.connect(
            host='localhost',
            port=5433,
            database='lab_manager',
            user='postgres',
            password='postgres'
        )
        print("✅ Successfully connected to database!")

        # Test a simple query
        result = await conn.fetchval("SELECT version()")
        print(f"Database version: {result}")

        await conn.close()
        print("✅ Connection closed successfully")

    except Exception as e:
        print(f"❌ Connection failed: {e}")
        print(f"Error type: {type(e)}")

if __name__ == "__main__":
    asyncio.run(test_connection())
