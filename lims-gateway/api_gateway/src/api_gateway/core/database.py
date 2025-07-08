"""
TracSeq API Gateway - Database Configuration and Connectivity

Provides standardized database connection patterns for all microservices.
"""

import asyncio
import os
import logging
from typing import Optional, Dict, Any, List
from dataclasses import dataclass
from contextlib import asynccontextmanager
from datetime import datetime, timedelta
import time

import asyncpg
from asyncpg import Pool

logger = logging.getLogger(__name__)


@dataclass
class DatabaseConfig:
    """Database configuration for TracSeq services."""
    
    # Connection parameters
    host: str = "lims-postgres"
    port: int = 5432
    database: str = "lims_db"
    username: str = "postgres"
    password: str = "postgres"
    
    # Connection pool settings
    min_connections: int = 2
    max_connections: int = 10
    connection_timeout: int = 30
    
    # Health check settings
    health_check_interval: int = 30
    health_check_timeout: int = 5
    
    @classmethod
    def from_env(cls) -> "DatabaseConfig":
        """Create database configuration from environment variables."""
        # Parse DATABASE_URL if provided
        database_url = os.getenv("DATABASE_URL")
        if database_url:
            return cls.from_url(database_url)
        
        # Otherwise use individual environment variables
        return cls(
            host=os.getenv("DB_HOST", "lims-postgres"),
            port=int(os.getenv("DB_PORT", "5432")),
            database=os.getenv("DB_NAME", "lims_db"),
            username=os.getenv("DB_USER", "postgres"),
            password=os.getenv("DB_PASSWORD", "postgres"),
            min_connections=int(os.getenv("DB_MIN_CONNECTIONS", "2")),
            max_connections=int(os.getenv("DB_MAX_CONNECTIONS", "10")),
            connection_timeout=int(os.getenv("DB_CONNECTION_TIMEOUT", "30")),
        )
    
    @classmethod
    def from_url(cls, database_url: str) -> "DatabaseConfig":
        """Create database configuration from a PostgreSQL URL."""
        from urllib.parse import urlparse
        
        parsed = urlparse(database_url)
        
        return cls(
            host=parsed.hostname or "lims-postgres",
            port=parsed.port or 5432,
            database=parsed.path.lstrip("/") or "lims_db",
            username=parsed.username or "postgres",
            password=parsed.password or "postgres",
        )
    
    def to_url(self) -> str:
        """Convert configuration to PostgreSQL URL."""
        return f"postgresql://{self.username}:{self.password}@{self.host}:{self.port}/{self.database}"
    
    def to_asyncpg_kwargs(self) -> Dict[str, Any]:
        """Convert configuration to asyncpg connection parameters."""
        return {
            "host": self.host,
            "port": self.port,
            "database": self.database,
            "user": self.username,
            "password": self.password,
            "min_size": self.min_connections,
            "max_size": self.max_connections,
            "command_timeout": self.connection_timeout,
        }


class DatabaseManager:
    """Manages database connections and health checks for TracSeq services."""
    
    def __init__(self, config: DatabaseConfig):
        self.config = config
        self.pool: Optional[Pool] = None
        self._health_check_task: Optional[asyncio.Task] = None
        self._is_healthy = False
        self._last_health_check = None
        self._health_check_history: List[Dict[str, Any]] = []
        self._connection_stats = {
            "total_connections": 0,
            "successful_connections": 0,
            "failed_connections": 0,
            "last_connection_time": None,
            "average_connection_time": 0.0,
        }
        
    async def initialize(self) -> None:
        """Initialize the database connection pool."""
        try:
            logger.info("Initializing database connection pool...")
            start_time = time.time()
            
            self.pool = await asyncpg.create_pool(
                **self.config.to_asyncpg_kwargs()
            )
            
            # Test the connection
            await self.health_check()
            
            # Update connection stats
            connection_time = time.time() - start_time
            self._connection_stats["total_connections"] += 1
            self._connection_stats["successful_connections"] += 1
            self._connection_stats["last_connection_time"] = datetime.now()
            self._update_average_connection_time(connection_time)
            
            # Start health check task
            self._health_check_task = asyncio.create_task(
                self._periodic_health_check()
            )
            
            logger.info(
                "Database connection pool initialized successfully",
                extra={
                    "host": self.config.host,
                    "port": self.config.port,
                    "database": self.config.database,
                    "min_connections": self.config.min_connections,
                    "max_connections": self.config.max_connections,
                    "connection_time": f"{connection_time:.3f}s",
                }
            )
            
        except Exception as e:
            logger.error(f"Failed to initialize database pool: {e}")
            self._connection_stats["total_connections"] += 1
            self._connection_stats["failed_connections"] += 1
            self.pool = None
            raise
    
    async def close(self) -> None:
        """Close the database connection pool."""
        if self._health_check_task:
            self._health_check_task.cancel()
            try:
                await self._health_check_task
            except asyncio.CancelledError:
                pass
        
        if self.pool:
            await self.pool.close()
            logger.info("Database connection pool closed")
    
    async def health_check(self) -> bool:
        """Perform a database health check."""
        if not self.pool:
            self._is_healthy = False
            self._record_health_check(False, "No database pool available")
            return False
        
        try:
            start_time = time.time()
            async with self.pool.acquire() as conn:
                # Test basic connectivity
                await conn.execute("SELECT 1")
                
                # Test database-specific functionality
                await conn.execute("SELECT current_database(), current_user, version()")
                
            check_time = time.time() - start_time
            self._is_healthy = True
            self._record_health_check(True, f"Health check passed in {check_time:.3f}s")
            return True
            
        except Exception as e:
            check_time = time.time() - start_time
            error_msg = f"Health check failed after {check_time:.3f}s: {str(e)}"
            logger.warning(error_msg)
            self._is_healthy = False
            self._record_health_check(False, error_msg)
            return False
    
    def _record_health_check(self, healthy: bool, details: str) -> None:
        """Record health check result in history."""
        check_record = {
            "timestamp": datetime.now(),
            "healthy": healthy,
            "details": details,
        }
        
        self._health_check_history.append(check_record)
        self._last_health_check = check_record
        
        # Keep only last 100 health checks
        if len(self._health_check_history) > 100:
            self._health_check_history = self._health_check_history[-100:]
    
    def _update_average_connection_time(self, connection_time: float) -> None:
        """Update average connection time statistics."""
        current_avg = self._connection_stats["average_connection_time"]
        total_connections = self._connection_stats["total_connections"]
        
        if total_connections == 1:
            self._connection_stats["average_connection_time"] = connection_time
        else:
            # Calculate running average
            new_avg = ((current_avg * (total_connections - 1)) + connection_time) / total_connections
            self._connection_stats["average_connection_time"] = new_avg
    
    async def _periodic_health_check(self) -> None:
        """Perform periodic health checks."""
        while True:
            try:
                await asyncio.sleep(self.config.health_check_interval)
                await self.health_check()
            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"Error in periodic health check: {e}")
    
    @property
    def is_healthy(self) -> bool:
        """Check if the database is healthy."""
        return self._is_healthy
    
    @asynccontextmanager
    async def get_connection(self):
        """Get a database connection from the pool."""
        if not self.pool:
            raise RuntimeError("Database pool not initialized")
        
        async with self.pool.acquire() as conn:
            yield conn
    
    async def execute(self, query: str, *args) -> str:
        """Execute a query and return the result."""
        async with self.get_connection() as conn:
            return await conn.execute(query, *args)
    
    async def fetch(self, query: str, *args) -> list:
        """Fetch multiple rows from a query."""
        async with self.get_connection() as conn:
            return await conn.fetch(query, *args)
    
    async def fetchrow(self, query: str, *args) -> Optional[asyncpg.Record]:
        """Fetch a single row from a query."""
        async with self.get_connection() as conn:
            return await conn.fetchrow(query, *args)
    
    async def fetchval(self, query: str, *args) -> Any:
        """Fetch a single value from a query."""
        async with self.get_connection() as conn:
            return await conn.fetchval(query, *args)
    
    def get_health_status(self) -> Dict[str, Any]:
        """Get comprehensive health status information."""
        recent_checks = [
            {
                "timestamp": check["timestamp"].isoformat(),
                "healthy": check["healthy"],
                "details": check["details"],
            }
            for check in self._health_check_history[-10:]  # Last 10 checks
        ]
        
        # Calculate uptime percentage from last 50 checks
        recent_history = self._health_check_history[-50:]
        if recent_history:
            healthy_count = sum(1 for check in recent_history if check["healthy"])
            uptime_percentage = (healthy_count / len(recent_history)) * 100
        else:
            uptime_percentage = 100.0 if self._is_healthy else 0.0
        
        return {
            "healthy": self._is_healthy,
            "pool_initialized": self.pool is not None,
            "last_check": self._last_health_check["timestamp"].isoformat() if self._last_health_check else None,
            "uptime_percentage": round(uptime_percentage, 2),
            "config": {
                "host": self.config.host,
                "port": self.config.port,
                "database": self.config.database,
                "min_connections": self.config.min_connections,
                "max_connections": self.config.max_connections,
            },
            "pool_stats": self._get_pool_stats() if self.pool else None,
            "connection_stats": self._connection_stats.copy(),
            "recent_checks": recent_checks,
        }
    
    def _get_pool_stats(self) -> Optional[Dict[str, Any]]:
        """Get connection pool statistics."""
        if not self.pool:
            return None
        
        return {
            "size": self.pool.get_size(),
            "min_size": self.pool.get_min_size(),
            "max_size": self.pool.get_max_size(),
            "idle_size": self.pool.get_idle_size(),
        }
    
    async def get_database_info(self) -> Dict[str, Any]:
        """Get detailed database information."""
        if not self.pool:
            return {"error": "Database pool not initialized"}
        
        try:
            async with self.get_connection() as conn:
                # Get basic database info
                db_info = await conn.fetchrow("""
                    SELECT 
                        current_database() as database_name,
                        current_user as current_user,
                        version() as version,
                        current_timestamp as server_time
                """)
                
                # Get database size
                db_size = await conn.fetchval("""
                    SELECT pg_size_pretty(pg_database_size(current_database()))
                """)
                
                # Get table count
                table_count = await conn.fetchval("""
                    SELECT count(*) FROM information_schema.tables 
                    WHERE table_schema = 'public'
                """)
                
                # Get active connections
                active_connections = await conn.fetchval("""
                    SELECT count(*) FROM pg_stat_activity 
                    WHERE datname = current_database()
                """)
                
                return {
                    "database_name": db_info["database_name"],
                    "current_user": db_info["current_user"],
                    "version": db_info["version"],
                    "server_time": db_info["server_time"].isoformat(),
                    "database_size": db_size,
                    "table_count": table_count,
                    "active_connections": active_connections,
                }
                
        except Exception as e:
            logger.error(f"Failed to get database info: {e}")
            return {"error": str(e)}


# Global database manager instance
_db_manager: Optional[DatabaseManager] = None


def get_database_manager() -> DatabaseManager:
    """Get the global database manager instance."""
    global _db_manager
    if _db_manager is None:
        config = DatabaseConfig.from_env()
        _db_manager = DatabaseManager(config)
    return _db_manager


async def init_database() -> DatabaseManager:
    """Initialize the global database manager."""
    db_manager = get_database_manager()
    await db_manager.initialize()
    return db_manager


async def close_database() -> None:
    """Close the global database manager."""
    global _db_manager
    if _db_manager:
        await _db_manager.close()
        _db_manager = None


# Convenience functions for common database operations
async def execute_query(query: str, *args) -> str:
    """Execute a query using the global database manager."""
    db_manager = get_database_manager()
    return await db_manager.execute(query, *args)


async def fetch_rows(query: str, *args) -> list:
    """Fetch multiple rows using the global database manager."""
    db_manager = get_database_manager()
    return await db_manager.fetch(query, *args)


async def fetch_one(query: str, *args) -> Optional[asyncpg.Record]:
    """Fetch a single row using the global database manager."""
    db_manager = get_database_manager()
    return await db_manager.fetchrow(query, *args)


async def fetch_value(query: str, *args) -> Any:
    """Fetch a single value using the global database manager."""
    db_manager = get_database_manager()
    return await db_manager.fetchval(query, *args)


@asynccontextmanager
async def get_db_connection():
    """Get a database connection from the global manager."""
    db_manager = get_database_manager()
    async with db_manager.get_connection() as conn:
        yield conn


def get_db_health_status() -> Dict[str, Any]:
    """Get database health status from the global manager."""
    global _db_manager
    if _db_manager:
        return _db_manager.get_health_status()
    return {"healthy": False, "error": "Database manager not initialized"}


async def get_db_info() -> Dict[str, Any]:
    """Get detailed database information from the global manager."""
    global _db_manager
    if _db_manager:
        return await _db_manager.get_database_info()
    return {"error": "Database manager not initialized"} 