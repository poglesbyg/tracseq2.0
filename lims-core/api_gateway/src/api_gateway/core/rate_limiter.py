"""
Advanced Rate Limiting for API Gateway

Provides distributed rate limiting with multiple algorithms and adaptive behavior.
"""

import asyncio
import time
from abc import ABC, abstractmethod
from datetime import datetime, timedelta
from enum import Enum
from typing import Any, Dict, List, Optional, Tuple
from dataclasses import dataclass
import structlog
import hashlib
from collections import defaultdict, deque

logger = structlog.get_logger(__name__)


class RateLimitAlgorithm(Enum):
    """Available rate limiting algorithms."""
    TOKEN_BUCKET = "token_bucket"
    SLIDING_WINDOW = "sliding_window"
    FIXED_WINDOW = "fixed_window"
    LEAKY_BUCKET = "leaky_bucket"
    ADAPTIVE = "adaptive"


@dataclass
class RateLimitConfig:
    """Configuration for rate limiting."""
    requests_per_minute: int = 60
    burst_size: int = 10
    algorithm: RateLimitAlgorithm = RateLimitAlgorithm.TOKEN_BUCKET
    per_user: bool = True
    per_endpoint: bool = True
    adaptive_threshold: float = 0.8  # CPU/memory threshold for adaptive limiting
    penalty_duration: int = 300  # Seconds to penalize bad actors
    whitelist: List[str] = None
    blacklist: List[str] = None


@dataclass
class RateLimitResult:
    """Result of rate limit check."""
    allowed: bool
    remaining: int
    reset_at: datetime
    retry_after: Optional[int] = None
    reason: Optional[str] = None


class RateLimiter(ABC):
    """Abstract base class for rate limiters."""
    
    @abstractmethod
    async def check_limit(self, identifier: str) -> RateLimitResult:
        """Check if request is within rate limit."""
        pass
    
    @abstractmethod
    async def reset(self, identifier: str):
        """Reset rate limit for identifier."""
        pass


class TokenBucketLimiter(RateLimiter):
    """
    Token bucket algorithm implementation.
    Allows burst traffic while maintaining average rate.
    """
    
    def __init__(self, config: RateLimitConfig):
        self.config = config
        self.buckets: Dict[str, Dict[str, Any]] = {}
        self._lock = asyncio.Lock()
        
        # Calculate refill rate
        self.refill_rate = config.requests_per_minute / 60.0
        
    async def check_limit(self, identifier: str) -> RateLimitResult:
        """Check if request is within rate limit."""
        async with self._lock:
            now = time.time()
            
            if identifier not in self.buckets:
                self.buckets[identifier] = {
                    'tokens': self.config.burst_size,
                    'last_refill': now
                }
            
            bucket = self.buckets[identifier]
            
            # Refill tokens based on elapsed time
            elapsed = now - bucket['last_refill']
            tokens_to_add = elapsed * self.refill_rate
            bucket['tokens'] = min(
                bucket['tokens'] + tokens_to_add,
                self.config.burst_size
            )
            bucket['last_refill'] = now
            
            # Check if we have tokens available
            if bucket['tokens'] >= 1:
                bucket['tokens'] -= 1
                return RateLimitResult(
                    allowed=True,
                    remaining=int(bucket['tokens']),
                    reset_at=datetime.fromtimestamp(
                        now + (self.config.burst_size - bucket['tokens']) / self.refill_rate
                    )
                )
            else:
                # Calculate when next token will be available
                retry_after = int((1 - bucket['tokens']) / self.refill_rate)
                return RateLimitResult(
                    allowed=False,
                    remaining=0,
                    reset_at=datetime.fromtimestamp(now + retry_after),
                    retry_after=retry_after,
                    reason="Rate limit exceeded"
                )
    
    async def reset(self, identifier: str):
        """Reset rate limit for identifier."""
        async with self._lock:
            if identifier in self.buckets:
                self.buckets[identifier]['tokens'] = self.config.burst_size


class SlidingWindowLimiter(RateLimiter):
    """
    Sliding window log algorithm implementation.
    Most accurate but memory intensive.
    """
    
    def __init__(self, config: RateLimitConfig):
        self.config = config
        self.windows: Dict[str, deque] = defaultdict(deque)
        self._lock = asyncio.Lock()
        self.window_size = 60  # 1 minute window
        
    async def check_limit(self, identifier: str) -> RateLimitResult:
        """Check if request is within rate limit."""
        async with self._lock:
            now = time.time()
            window = self.windows[identifier]
            
            # Remove old entries outside the window
            cutoff = now - self.window_size
            while window and window[0] < cutoff:
                window.popleft()
            
            # Check if we're within limit
            if len(window) < self.config.requests_per_minute:
                window.append(now)
                remaining = self.config.requests_per_minute - len(window)
                
                # Calculate reset time (when oldest entry expires)
                reset_at = datetime.fromtimestamp(
                    window[0] + self.window_size if window else now + self.window_size
                )
                
                return RateLimitResult(
                    allowed=True,
                    remaining=remaining,
                    reset_at=reset_at
                )
            else:
                # Calculate retry after (when oldest entry expires)
                retry_after = int(window[0] + self.window_size - now)
                return RateLimitResult(
                    allowed=False,
                    remaining=0,
                    reset_at=datetime.fromtimestamp(window[0] + self.window_size),
                    retry_after=retry_after,
                    reason="Rate limit exceeded"
                )
    
    async def reset(self, identifier: str):
        """Reset rate limit for identifier."""
        async with self._lock:
            self.windows[identifier].clear()


class LeakyBucketLimiter(RateLimiter):
    """
    Leaky bucket algorithm implementation.
    Smooths out bursts by processing at constant rate.
    """
    
    def __init__(self, config: RateLimitConfig):
        self.config = config
        self.buckets: Dict[str, Dict[str, Any]] = {}
        self._lock = asyncio.Lock()
        self.leak_rate = config.requests_per_minute / 60.0
        
    async def check_limit(self, identifier: str) -> RateLimitResult:
        """Check if request is within rate limit."""
        async with self._lock:
            now = time.time()
            
            if identifier not in self.buckets:
                self.buckets[identifier] = {
                    'volume': 0,
                    'last_leak': now
                }
            
            bucket = self.buckets[identifier]
            
            # Leak water based on elapsed time
            elapsed = now - bucket['last_leak']
            leaked = elapsed * self.leak_rate
            bucket['volume'] = max(0, bucket['volume'] - leaked)
            bucket['last_leak'] = now
            
            # Check if bucket can accept more water
            if bucket['volume'] < self.config.burst_size:
                bucket['volume'] += 1
                remaining = int(self.config.burst_size - bucket['volume'])
                
                return RateLimitResult(
                    allowed=True,
                    remaining=remaining,
                    reset_at=datetime.fromtimestamp(
                        now + bucket['volume'] / self.leak_rate
                    )
                )
            else:
                # Calculate when bucket will have space
                retry_after = int((bucket['volume'] - self.config.burst_size + 1) / self.leak_rate)
                return RateLimitResult(
                    allowed=False,
                    remaining=0,
                    reset_at=datetime.fromtimestamp(now + retry_after),
                    retry_after=retry_after,
                    reason="Rate limit exceeded"
                )
    
    async def reset(self, identifier: str):
        """Reset rate limit for identifier."""
        async with self._lock:
            if identifier in self.buckets:
                self.buckets[identifier]['volume'] = 0


class AdaptiveRateLimiter(RateLimiter):
    """
    Adaptive rate limiter that adjusts based on system load.
    """
    
    def __init__(self, config: RateLimitConfig, system_monitor):
        self.config = config
        self.system_monitor = system_monitor
        self.base_limiter = TokenBucketLimiter(config)
        self._adjustment_factor = 1.0
        self._last_adjustment = time.time()
        self._lock = asyncio.Lock()
        
    async def check_limit(self, identifier: str) -> RateLimitResult:
        """Check if request is within rate limit with adaptive adjustment."""
        # Update adjustment factor based on system load
        await self._update_adjustment_factor()
        
        # Create adjusted config
        adjusted_config = RateLimitConfig(
            requests_per_minute=int(
                self.config.requests_per_minute * self._adjustment_factor
            ),
            burst_size=int(
                self.config.burst_size * self._adjustment_factor
            ),
            algorithm=self.config.algorithm
        )
        
        # Use base limiter with adjusted config
        limiter = TokenBucketLimiter(adjusted_config)
        result = await limiter.check_limit(identifier)
        
        # Add adjustment info to reason if limited
        if not result.allowed and self._adjustment_factor < 1.0:
            result.reason = f"Rate limit exceeded (adjusted due to high load: {self._adjustment_factor:.2f}x)"
            
        return result
    
    async def _update_adjustment_factor(self):
        """Update adjustment factor based on system metrics."""
        async with self._lock:
            now = time.time()
            
            # Only update every 10 seconds
            if now - self._last_adjustment < 10:
                return
                
            self._last_adjustment = now
            
            # Get system metrics
            metrics = await self.system_monitor.get_metrics()
            cpu_usage = metrics.get('cpu_usage', 0)
            memory_usage = metrics.get('memory_usage', 0)
            
            # Calculate load factor (0-1)
            load_factor = max(cpu_usage, memory_usage)
            
            # Adjust rate limit based on load
            if load_factor > self.config.adaptive_threshold:
                # Reduce rate limit when under load
                self._adjustment_factor = max(
                    0.1,  # Minimum 10% of original
                    1.0 - (load_factor - self.config.adaptive_threshold)
                )
            else:
                # Gradually restore rate limit
                self._adjustment_factor = min(
                    1.0,
                    self._adjustment_factor + 0.1
                )
                
            logger.info(
                "Adaptive rate limit adjusted",
                adjustment_factor=self._adjustment_factor,
                load_factor=load_factor
            )
    
    async def reset(self, identifier: str):
        """Reset rate limit for identifier."""
        await self.base_limiter.reset(identifier)


class DistributedRateLimiter(RateLimiter):
    """
    Distributed rate limiter using Redis for shared state.
    """
    
    def __init__(self, config: RateLimitConfig, redis_client):
        self.config = config
        self.redis = redis_client
        self.key_prefix = "rate_limit:"
        
    async def check_limit(self, identifier: str) -> RateLimitResult:
        """Check if request is within rate limit using Redis."""
        key = f"{self.key_prefix}{identifier}"
        now = time.time()
        window_start = now - 60  # 1 minute window
        
        # Use Redis sorted set for sliding window
        pipe = self.redis.pipeline()
        
        # Remove old entries
        pipe.zremrangebyscore(key, 0, window_start)
        
        # Count current entries
        pipe.zcard(key)
        
        # Add current request
        pipe.zadd(key, {str(now): now})
        
        # Set expiry
        pipe.expire(key, 60)
        
        results = await pipe.execute()
        count = results[1]
        
        if count < self.config.requests_per_minute:
            return RateLimitResult(
                allowed=True,
                remaining=self.config.requests_per_minute - count - 1,
                reset_at=datetime.fromtimestamp(now + 60)
            )
        else:
            # Get oldest entry to calculate retry
            oldest = await self.redis.zrange(key, 0, 0, withscores=True)
            if oldest:
                retry_after = int(oldest[0][1] + 60 - now)
            else:
                retry_after = 60
                
            return RateLimitResult(
                allowed=False,
                remaining=0,
                reset_at=datetime.fromtimestamp(now + retry_after),
                retry_after=retry_after,
                reason="Rate limit exceeded"
            )
    
    async def reset(self, identifier: str):
        """Reset rate limit for identifier."""
        key = f"{self.key_prefix}{identifier}"
        await self.redis.delete(key)


class RateLimitManager:
    """
    Manages rate limiting across different services and endpoints.
    """
    
    def __init__(self, redis_client=None, system_monitor=None):
        self.redis_client = redis_client
        self.system_monitor = system_monitor
        self.limiters: Dict[str, RateLimiter] = {}
        self.configs: Dict[str, RateLimitConfig] = {}
        self.blacklist: set = set()
        self.whitelist: set = set()
        self._penalty_tracker: Dict[str, datetime] = {}
        
    def configure_limiter(
        self, 
        name: str, 
        config: RateLimitConfig,
        distributed: bool = False
    ):
        """Configure a rate limiter for a service or endpoint."""
        self.configs[name] = config
        
        # Update blacklist/whitelist
        if config.blacklist:
            self.blacklist.update(config.blacklist)
        if config.whitelist:
            self.whitelist.update(config.whitelist)
        
        # Create appropriate limiter
        if distributed and self.redis_client:
            self.limiters[name] = DistributedRateLimiter(config, self.redis_client)
        elif config.algorithm == RateLimitAlgorithm.ADAPTIVE and self.system_monitor:
            self.limiters[name] = AdaptiveRateLimiter(config, self.system_monitor)
        elif config.algorithm == RateLimitAlgorithm.TOKEN_BUCKET:
            self.limiters[name] = TokenBucketLimiter(config)
        elif config.algorithm == RateLimitAlgorithm.SLIDING_WINDOW:
            self.limiters[name] = SlidingWindowLimiter(config)
        elif config.algorithm == RateLimitAlgorithm.LEAKY_BUCKET:
            self.limiters[name] = LeakyBucketLimiter(config)
        else:
            self.limiters[name] = TokenBucketLimiter(config)  # Default
    
    async def check_rate_limit(
        self,
        service: str,
        endpoint: str,
        user_id: Optional[str] = None,
        ip_address: Optional[str] = None
    ) -> RateLimitResult:
        """
        Check rate limit for a request.
        
        Args:
            service: Service name
            endpoint: Endpoint path
            user_id: Optional user identifier
            ip_address: Optional IP address
            
        Returns:
            RateLimitResult indicating if request is allowed
        """
        # Check blacklist first
        if ip_address and ip_address in self.blacklist:
            return RateLimitResult(
                allowed=False,
                remaining=0,
                reset_at=datetime.now() + timedelta(hours=24),
                retry_after=86400,
                reason="IP address blacklisted"
            )
            
        # Check whitelist
        if ip_address and ip_address in self.whitelist:
            return RateLimitResult(
                allowed=True,
                remaining=999999,
                reset_at=datetime.now() + timedelta(hours=1)
            )
            
        # Check penalty status
        penalty_key = f"{user_id or ip_address}"
        if penalty_key in self._penalty_tracker:
            penalty_end = self._penalty_tracker[penalty_key]
            if datetime.now() < penalty_end:
                retry_after = int((penalty_end - datetime.now()).total_seconds())
                return RateLimitResult(
                    allowed=False,
                    remaining=0,
                    reset_at=penalty_end,
                    retry_after=retry_after,
                    reason="Temporarily banned due to abuse"
                )
            else:
                del self._penalty_tracker[penalty_key]
        
        # Build identifier based on configuration
        identifiers = []
        
        # Service-level limiting
        if service in self.limiters:
            config = self.configs[service]
            if config.per_user and user_id:
                identifiers.append(f"{service}:user:{user_id}")
            elif config.per_endpoint:
                identifiers.append(f"{service}:endpoint:{endpoint}")
            else:
                identifiers.append(f"{service}:global")
        
        # Endpoint-specific limiting
        endpoint_key = f"{service}:{endpoint}"
        if endpoint_key in self.limiters:
            config = self.configs[endpoint_key]
            if config.per_user and user_id:
                identifiers.append(f"{endpoint_key}:user:{user_id}")
            else:
                identifiers.append(f"{endpoint_key}:global")
        
        # Global limiting
        if "global" in self.limiters:
            config = self.configs["global"]
            if config.per_user and user_id:
                identifiers.append(f"global:user:{user_id}")
            elif ip_address:
                identifiers.append(f"global:ip:{ip_address}")
            else:
                identifiers.append("global:anonymous")
        
        # Check all applicable limits
        most_restrictive_result = None
        
        for identifier in identifiers:
            limiter_name = identifier.split(":")[0]
            if limiter_name in self.limiters:
                limiter = self.limiters[limiter_name]
                result = await limiter.check_limit(identifier)
                
                if not result.allowed:
                    # Track repeated violations for penalty
                    await self._track_violation(penalty_key)
                    
                    if most_restrictive_result is None or result.retry_after > most_restrictive_result.retry_after:
                        most_restrictive_result = result
        
        # If no limits were violated, allow the request
        if most_restrictive_result is None:
            return RateLimitResult(
                allowed=True,
                remaining=999999,
                reset_at=datetime.now() + timedelta(minutes=1)
            )
        
        return most_restrictive_result
    
    async def _track_violation(self, identifier: str):
        """Track rate limit violations for penalty system."""
        violation_key = f"violations:{identifier}"
        
        if self.redis_client:
            # Use Redis for distributed tracking
            count = await self.redis_client.incr(violation_key)
            await self.redis_client.expire(violation_key, 3600)  # 1 hour window
            
            if count > 10:  # More than 10 violations in an hour
                penalty_duration = min(300 * (count // 10), 3600)  # Max 1 hour
                self._penalty_tracker[identifier] = datetime.now() + timedelta(seconds=penalty_duration)
                logger.warning(
                    "Rate limit penalty applied",
                    identifier=identifier,
                    violations=count,
                    penalty_seconds=penalty_duration
                )
    
    async def reset_limits(self, identifier: str):
        """Reset all limits for an identifier."""
        for limiter in self.limiters.values():
            await limiter.reset(identifier)
            
        # Clear penalties
        if identifier in self._penalty_tracker:
            del self._penalty_tracker[identifier]
    
    def get_status(self) -> Dict[str, Any]:
        """Get rate limiting status."""
        return {
            "configured_limiters": list(self.limiters.keys()),
            "blacklisted_ips": len(self.blacklist),
            "whitelisted_ips": len(self.whitelist),
            "active_penalties": len(self._penalty_tracker),
            "configs": {
                name: {
                    "requests_per_minute": config.requests_per_minute,
                    "algorithm": config.algorithm.value,
                    "per_user": config.per_user,
                    "per_endpoint": config.per_endpoint
                }
                for name, config in self.configs.items()
            }
        }