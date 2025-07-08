#!/usr/bin/env python3
"""
TracSeq Database Connectivity Test Script

This script tests database connectivity across all TracSeq microservices
to ensure the standardized database configuration is working correctly.
"""

import asyncio
import asyncpg
import aioredis
import httpx
import json
import sys
import time
from typing import Dict, Any, List, Optional
from datetime import datetime
import argparse


# Service configuration
SERVICES = {
    "api-gateway": {
        "name": "API Gateway",
        "url": "http://localhost:8000",
        "health_endpoint": "/health",
        "critical": True
    },
    "auth-service": {
        "name": "Authentication Service", 
        "url": "http://localhost:8080",
        "health_endpoint": "/health",
        "critical": True
    },
    "sample-service": {
        "name": "Sample Management Service",
        "url": "http://localhost:8081", 
        "health_endpoint": "/health",
        "critical": True
    },
    "storage-service": {
        "name": "Enhanced Storage Service",
        "url": "http://localhost:8082",
        "health_endpoint": "/health", 
        "critical": True
    },
    "template-service": {
        "name": "Template Service",
        "url": "http://localhost:8083",
        "health_endpoint": "/health",
        "critical": True
    },
    "sequencing-service": {
        "name": "Sequencing Service",
        "url": "http://localhost:8084",
        "health_endpoint": "/health",
        "critical": True
    },
    "notification-service": {
        "name": "Notification Service",
        "url": "http://localhost:8085",
        "health_endpoint": "/health",
        "critical": False
    },
    "rag-service": {
        "name": "RAG Service",
        "url": "http://localhost:8086",
        "health_endpoint": "/health",
        "critical": False
    },
    "event-service": {
        "name": "Event Service",
        "url": "http://localhost:8087",
        "health_endpoint": "/health",
        "critical": False
    },
    "transaction-service": {
        "name": "Transaction Service",
        "url": "http://localhost:8088",
        "health_endpoint": "/health",
        "critical": False
    }
}

# Database configuration
DATABASE_CONFIG = {
    "host": "localhost",
    "port": 5432,
    "database": "lims_db",
    "user": "postgres", 
    "password": "postgres"
}

# Redis configuration
REDIS_CONFIG = {
    "host": "localhost",
    "port": 6379,
    "db": 0
}


class DatabaseConnectivityTester:
    """Test database connectivity across TracSeq microservices."""
    
    def __init__(self, verbose: bool = False, quick: bool = False):
        self.verbose = verbose
        self.quick = quick
        self.results = {}
        self.start_time = time.time()
        
    def log(self, message: str, level: str = "INFO"):
        """Log a message with timestamp."""
        timestamp = datetime.now().strftime("%H:%M:%S")
        prefix = {
            "INFO": "ℹ️ ",
            "SUCCESS": "✅",
            "WARNING": "⚠️ ",
            "ERROR": "❌",
            "DEBUG": "🔍"
        }.get(level, "")
        
        print(f"[{timestamp}] {prefix} {message}")
        
        if self.verbose and level == "DEBUG":
            print(f"    {message}")
    
    async def test_postgresql_connectivity(self) -> Dict[str, Any]:
        """Test PostgreSQL database connectivity."""
        self.log("Testing PostgreSQL connectivity...", "INFO")
        
        try:
            # Test basic connection
            conn = await asyncpg.connect(
                host=DATABASE_CONFIG["host"],
                port=DATABASE_CONFIG["port"],
                database=DATABASE_CONFIG["database"],
                user=DATABASE_CONFIG["user"],
                password=DATABASE_CONFIG["password"]
            )
            
            # Test basic query
            result = await conn.fetchval("SELECT 1")
            assert result == 1
            
            # Get database info
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
            
            await conn.close()
            
            result = {
                "status": "healthy",
                "database_name": db_info["database_name"],
                "current_user": db_info["current_user"],
                "version": db_info["version"],
                "server_time": db_info["server_time"].isoformat(),
                "database_size": db_size,
                "table_count": table_count,
                "active_connections": active_connections
            }
            
            self.log("PostgreSQL connectivity test passed", "SUCCESS")
            if self.verbose:
                self.log(f"Database: {result['database_name']}", "DEBUG")
                self.log(f"Size: {result['database_size']}", "DEBUG")
                self.log(f"Tables: {result['table_count']}", "DEBUG")
                self.log(f"Active connections: {result['active_connections']}", "DEBUG")
            
            return result
            
        except Exception as e:
            self.log(f"PostgreSQL connectivity test failed: {e}", "ERROR")
            return {
                "status": "unhealthy",
                "error": str(e)
            }
    
    async def test_redis_connectivity(self) -> Dict[str, Any]:
        """Test Redis connectivity."""
        self.log("Testing Redis connectivity...", "INFO")
        
        try:
            # Test basic connection
            redis = aioredis.from_url(
                f"redis://{REDIS_CONFIG['host']}:{REDIS_CONFIG['port']}/{REDIS_CONFIG['db']}"
            )
            
            # Test basic operations
            await redis.ping()
            
            # Test set/get
            test_key = "tracseq:test:connectivity"
            test_value = f"test-{int(time.time())}"
            await redis.set(test_key, test_value, ex=60)
            retrieved_value = await redis.get(test_key)
            assert retrieved_value.decode() == test_value
            
            # Get Redis info
            info = await redis.info()
            
            await redis.close()
            
            result = {
                "status": "healthy",
                "redis_version": info.get("redis_version"),
                "used_memory": info.get("used_memory_human"),
                "connected_clients": info.get("connected_clients"),
                "uptime_in_seconds": info.get("uptime_in_seconds")
            }
            
            self.log("Redis connectivity test passed", "SUCCESS")
            if self.verbose:
                self.log(f"Redis version: {result['redis_version']}", "DEBUG")
                self.log(f"Memory usage: {result['used_memory']}", "DEBUG")
                self.log(f"Connected clients: {result['connected_clients']}", "DEBUG")
            
            return result
            
        except Exception as e:
            self.log(f"Redis connectivity test failed: {e}", "ERROR")
            return {
                "status": "unhealthy",
                "error": str(e)
            }
    
    async def test_service_health(self, service_id: str, service_config: Dict[str, Any]) -> Dict[str, Any]:
        """Test health of a specific service."""
        service_name = service_config["name"]
        
        if not self.quick:
            self.log(f"Testing {service_name}...", "INFO")
        
        try:
            async with httpx.AsyncClient(timeout=10.0) as client:
                url = f"{service_config['url']}{service_config['health_endpoint']}"
                
                start_time = time.time()
                response = await client.get(url)
                response_time = time.time() - start_time
                
                if response.status_code == 200:
                    try:
                        health_data = response.json()
                    except:
                        health_data = {"message": "Service responded but no JSON data"}
                    
                    result = {
                        "status": "healthy",
                        "response_time": round(response_time, 3),
                        "status_code": response.status_code,
                        "health_data": health_data
                    }
                    
                    if not self.quick:
                        self.log(f"{service_name} is healthy ({response_time:.3f}s)", "SUCCESS")
                    
                    return result
                else:
                    result = {
                        "status": "unhealthy",
                        "response_time": round(response_time, 3),
                        "status_code": response.status_code,
                        "error": f"HTTP {response.status_code}"
                    }
                    
                    self.log(f"{service_name} returned HTTP {response.status_code}", "WARNING")
                    return result
                    
        except httpx.ConnectError:
            self.log(f"{service_name} is not reachable", "ERROR")
            return {
                "status": "unreachable",
                "error": "Connection failed"
            }
        except httpx.TimeoutException:
            self.log(f"{service_name} timed out", "ERROR")
            return {
                "status": "timeout",
                "error": "Request timeout"
            }
        except Exception as e:
            self.log(f"{service_name} health check failed: {e}", "ERROR")
            return {
                "status": "error",
                "error": str(e)
            }
    
    async def test_api_gateway_health_aggregation(self) -> Dict[str, Any]:
        """Test API Gateway's health aggregation functionality."""
        self.log("Testing API Gateway health aggregation...", "INFO")
        
        try:
            async with httpx.AsyncClient(timeout=15.0) as client:
                # Test comprehensive health endpoint
                url = "http://localhost:8000/api/health"
                response = await client.get(url)
                
                if response.status_code == 200:
                    health_data = response.json()
                    
                    result = {
                        "status": "healthy",
                        "aggregated_health": health_data,
                        "features": {
                            "database_info": "info" in health_data.get("database", {}),
                            "standardized_db": health_data.get("features", {}).get("standardized_db", False),
                            "enhanced_monitoring": health_data.get("features", {}).get("enhanced_monitoring", False)
                        }
                    }
                    
                    self.log("API Gateway health aggregation working", "SUCCESS")
                    if self.verbose:
                        db_status = health_data.get("database", {}).get("healthy", False)
                        self.log(f"Database health: {'✓' if db_status else '✗'}", "DEBUG")
                        
                        features = result["features"]
                        self.log(f"Standardized DB: {'✓' if features['standardized_db'] else '✗'}", "DEBUG")
                        self.log(f"Enhanced monitoring: {'✓' if features['enhanced_monitoring'] else '✗'}", "DEBUG")
                    
                    return result
                else:
                    return {
                        "status": "unhealthy",
                        "status_code": response.status_code,
                        "error": f"HTTP {response.status_code}"
                    }
                    
        except Exception as e:
            self.log(f"API Gateway health aggregation test failed: {e}", "ERROR")
            return {
                "status": "error",
                "error": str(e)
            }
    
    async def run_all_tests(self) -> Dict[str, Any]:
        """Run all connectivity tests."""
        self.log("🚀 Starting TracSeq Database Connectivity Tests", "INFO")
        self.log("=" * 60, "INFO")
        
        # Test infrastructure
        self.log("Testing Infrastructure Services...", "INFO")
        
        postgresql_result = await self.test_postgresql_connectivity()
        redis_result = await self.test_redis_connectivity()
        
        # Test microservices
        self.log("Testing Microservices...", "INFO")
        
        service_results = {}
        
        if self.quick:
            # Test services in parallel for quick mode
            tasks = [
                self.test_service_health(service_id, service_config)
                for service_id, service_config in SERVICES.items()
            ]
            
            results = await asyncio.gather(*tasks, return_exceptions=True)
            
            for (service_id, service_config), result in zip(SERVICES.items(), results):
                if isinstance(result, Exception):
                    service_results[service_id] = {
                        "status": "error",
                        "error": str(result)
                    }
                else:
                    service_results[service_id] = result
        else:
            # Test services sequentially for detailed output
            for service_id, service_config in SERVICES.items():
                service_results[service_id] = await self.test_service_health(service_id, service_config)
        
        # Test API Gateway health aggregation
        gateway_health = await self.test_api_gateway_health_aggregation()
        
        # Compile results
        total_time = time.time() - self.start_time
        
        # Count service statuses
        healthy_services = sum(1 for result in service_results.values() if result["status"] == "healthy")
        total_services = len(service_results)
        
        critical_services = [
            service_id for service_id, config in SERVICES.items() 
            if config["critical"]
        ]
        
        critical_healthy = sum(
            1 for service_id in critical_services 
            if service_results.get(service_id, {}).get("status") == "healthy"
        )
        
        # Determine overall status
        if postgresql_result["status"] != "healthy" or redis_result["status"] != "healthy":
            overall_status = "critical"
            status_message = "Infrastructure services are unhealthy"
        elif critical_healthy < len(critical_services):
            overall_status = "degraded"
            status_message = f"{len(critical_services) - critical_healthy} critical service(s) are unhealthy"
        elif healthy_services < total_services:
            overall_status = "degraded"
            status_message = f"{total_services - healthy_services} service(s) are unhealthy"
        else:
            overall_status = "healthy"
            status_message = "All services are healthy"
        
        results = {
            "overall_status": overall_status,
            "status_message": status_message,
            "test_duration": round(total_time, 2),
            "timestamp": datetime.now().isoformat(),
            "infrastructure": {
                "postgresql": postgresql_result,
                "redis": redis_result
            },
            "services": service_results,
            "gateway_health": gateway_health,
            "summary": {
                "total_services": total_services,
                "healthy_services": healthy_services,
                "critical_services": len(critical_services),
                "critical_healthy": critical_healthy,
                "uptime_percentage": round((healthy_services / total_services) * 100, 1) if total_services > 0 else 0
            }
        }
        
        # Print summary
        self.log("=" * 60, "INFO")
        self.log("📊 Test Results Summary", "INFO")
        self.log(f"Overall Status: {overall_status.upper()}", "SUCCESS" if overall_status == "healthy" else "WARNING")
        self.log(f"Status Message: {status_message}", "INFO")
        self.log(f"Services: {healthy_services}/{total_services} healthy ({results['summary']['uptime_percentage']}%)", "INFO")
        self.log(f"Critical Services: {critical_healthy}/{len(critical_services)} healthy", "INFO")
        self.log(f"Test Duration: {total_time:.2f}s", "INFO")
        
        # Print detailed results if verbose
        if self.verbose:
            self.log("\n📋 Detailed Results:", "INFO")
            self.log(json.dumps(results, indent=2, default=str), "DEBUG")
        
        return results


async def main():
    """Main function."""
    parser = argparse.ArgumentParser(description="Test TracSeq database connectivity")
    parser.add_argument("-v", "--verbose", action="store_true", help="Enable verbose output")
    parser.add_argument("-q", "--quick", action="store_true", help="Quick test mode (parallel execution)")
    parser.add_argument("--service", help="Test specific service only")
    parser.add_argument("--json", action="store_true", help="Output results in JSON format")
    
    args = parser.parse_args()
    
    tester = DatabaseConnectivityTester(verbose=args.verbose, quick=args.quick)
    
    if args.service:
        # Test specific service
        if args.service not in SERVICES:
            print(f"❌ Unknown service: {args.service}")
            print(f"Available services: {', '.join(SERVICES.keys())}")
            sys.exit(1)
        
        service_config = SERVICES[args.service]
        result = await tester.test_service_health(args.service, service_config)
        
        if args.json:
            print(json.dumps(result, indent=2))
        else:
            status = result["status"]
            print(f"Service: {service_config['name']}")
            print(f"Status: {status}")
            if status == "healthy":
                print(f"Response time: {result.get('response_time', 'N/A')}s")
            else:
                print(f"Error: {result.get('error', 'Unknown error')}")
    else:
        # Run all tests
        results = await tester.run_all_tests()
        
        if args.json:
            print(json.dumps(results, indent=2, default=str))
        
        # Exit with appropriate code
        if results["overall_status"] == "healthy":
            sys.exit(0)
        elif results["overall_status"] == "degraded":
            sys.exit(1)
        else:
            sys.exit(2)


if __name__ == "__main__":
    asyncio.run(main()) 