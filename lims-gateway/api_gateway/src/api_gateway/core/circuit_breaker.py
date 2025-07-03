"""
Circuit Breaker Pattern Implementation for API Gateway

Provides fault tolerance and prevents cascading failures across microservices.
"""

import asyncio
import time
from datetime import datetime, timedelta
from enum import Enum
from typing import Any, Callable, Dict, Optional, TypeVar
from dataclasses import dataclass, field
import structlog
from collections import deque

logger = structlog.get_logger(__name__)

T = TypeVar('T')


class CircuitState(Enum):
    """Circuit breaker states."""
    CLOSED = "closed"  # Normal operation
    OPEN = "open"      # Circuit is tripped, rejecting calls
    HALF_OPEN = "half_open"  # Testing if service recovered


@dataclass
class CircuitBreakerConfig:
    """Configuration for circuit breaker behavior."""
    failure_threshold: int = 5          # Failures before opening circuit
    success_threshold: int = 3          # Successes in half-open before closing
    timeout: float = 60.0              # Seconds before trying half-open
    window_size: int = 100             # Size of sliding window for failure rate
    failure_rate_threshold: float = 0.5 # Failure rate to open circuit
    slow_call_duration: float = 5.0    # Seconds to consider a call slow
    slow_call_rate_threshold: float = 0.5  # Slow call rate to open circuit
    excluded_exceptions: tuple = ()     # Exceptions that don't count as failures


@dataclass
class CircuitBreakerStats:
    """Statistics for circuit breaker monitoring."""
    total_calls: int = 0
    failed_calls: int = 0
    successful_calls: int = 0
    slow_calls: int = 0
    last_failure_time: Optional[datetime] = None
    last_success_time: Optional[datetime] = None
    consecutive_failures: int = 0
    consecutive_successes: int = 0
    state_changes: list = field(default_factory=list)
    call_durations: deque = field(default_factory=lambda: deque(maxlen=100))


class CircuitBreaker:
    """
    Circuit breaker implementation with advanced features:
    - Sliding window for failure rate calculation
    - Slow call detection
    - Adaptive timeout based on recovery patterns
    - Detailed statistics and monitoring
    """

    def __init__(self, name: str, config: Optional[CircuitBreakerConfig] = None):
        self.name = name
        self.config = config or CircuitBreakerConfig()
        self.state = CircuitState.CLOSED
        self.stats = CircuitBreakerStats()
        self._last_state_change = datetime.now()
        self._half_open_calls = 0
        self._lock = asyncio.Lock()
        
        # Sliding window for failure tracking
        self._call_results = deque(maxlen=self.config.window_size)
        
        logger.info(
            "Circuit breaker initialized",
            name=name,
            config=self.config
        )

    async def __aenter__(self):
        """Context manager entry."""
        await self._check_state()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit with automatic state management."""
        if exc_type is None:
            await self._record_success()
        else:
            await self._record_failure(exc_val)
        return False

    async def call(self, func: Callable[..., T], *args, **kwargs) -> T:
        """
        Execute a function with circuit breaker protection.
        
        Args:
            func: The function to execute
            *args: Positional arguments for the function
            **kwargs: Keyword arguments for the function
            
        Returns:
            The function result
            
        Raises:
            CircuitOpenError: If circuit is open
            Original exception: If function fails and circuit allows
        """
        await self._check_state()
        
        start_time = time.time()
        try:
            result = await func(*args, **kwargs)
            duration = time.time() - start_time
            await self._record_success(duration)
            return result
        except Exception as e:
            duration = time.time() - start_time
            await self._record_failure(e, duration)
            raise

    async def _check_state(self):
        """Check if the circuit allows the call to proceed."""
        async with self._lock:
            if self.state == CircuitState.OPEN:
                # Check if timeout has elapsed
                if self._should_attempt_reset():
                    await self._transition_to_half_open()
                else:
                    raise CircuitOpenError(
                        f"Circuit breaker '{self.name}' is OPEN",
                        self.stats
                    )
            
            elif self.state == CircuitState.HALF_OPEN:
                # Limit concurrent calls in half-open state
                if self._half_open_calls >= self.config.success_threshold:
                    raise CircuitOpenError(
                        f"Circuit breaker '{self.name}' is testing recovery",
                        self.stats
                    )
                self._half_open_calls += 1

    async def _record_success(self, duration: float = 0.0):
        """Record a successful call."""
        async with self._lock:
            self.stats.total_calls += 1
            self.stats.successful_calls += 1
            self.stats.consecutive_successes += 1
            self.stats.consecutive_failures = 0
            self.stats.last_success_time = datetime.now()
            self.stats.call_durations.append(duration)
            
            # Track slow calls
            if duration > self.config.slow_call_duration:
                self.stats.slow_calls += 1
            
            self._call_results.append((True, duration))
            
            # Handle state transitions
            if self.state == CircuitState.HALF_OPEN:
                if self.stats.consecutive_successes >= self.config.success_threshold:
                    await self._transition_to_closed()
                    
            logger.debug(
                "Call succeeded",
                circuit=self.name,
                state=self.state.value,
                duration=duration
            )

    async def _record_failure(self, exception: Exception, duration: float = 0.0):
        """Record a failed call."""
        # Check if exception should be ignored
        if isinstance(exception, self.config.excluded_exceptions):
            return
            
        async with self._lock:
            self.stats.total_calls += 1
            self.stats.failed_calls += 1
            self.stats.consecutive_failures += 1
            self.stats.consecutive_successes = 0
            self.stats.last_failure_time = datetime.now()
            self.stats.call_durations.append(duration)
            
            self._call_results.append((False, duration))
            
            # Check if we should open the circuit
            if self.state == CircuitState.CLOSED:
                if self._should_open_circuit():
                    await self._transition_to_open()
                    
            elif self.state == CircuitState.HALF_OPEN:
                # Single failure in half-open returns to open
                await self._transition_to_open()
                
            logger.warning(
                "Call failed",
                circuit=self.name,
                state=self.state.value,
                exception=str(exception),
                consecutive_failures=self.stats.consecutive_failures
            )

    def _should_open_circuit(self) -> bool:
        """Determine if circuit should open based on failure conditions."""
        # Check consecutive failures
        if self.stats.consecutive_failures >= self.config.failure_threshold:
            return True
            
        # Check failure rate in sliding window
        if len(self._call_results) >= self.config.window_size // 2:
            failure_rate = sum(
                1 for success, _ in self._call_results if not success
            ) / len(self._call_results)
            
            if failure_rate > self.config.failure_rate_threshold:
                return True
                
        # Check slow call rate
        if len(self.stats.call_durations) >= self.config.window_size // 2:
            slow_rate = sum(
                1 for d in self.stats.call_durations 
                if d > self.config.slow_call_duration
            ) / len(self.stats.call_durations)
            
            if slow_rate > self.config.slow_call_rate_threshold:
                return True
                
        return False

    def _should_attempt_reset(self) -> bool:
        """Check if enough time has passed to attempt reset."""
        elapsed = (datetime.now() - self._last_state_change).total_seconds()
        
        # Adaptive timeout based on failure patterns
        adaptive_timeout = self.config.timeout
        if self.stats.consecutive_failures > 10:
            # Exponential backoff for repeated failures
            adaptive_timeout *= min(2 ** (self.stats.consecutive_failures // 10), 10)
            
        return elapsed >= adaptive_timeout

    async def _transition_to_open(self):
        """Transition to OPEN state."""
        old_state = self.state
        self.state = CircuitState.OPEN
        self._last_state_change = datetime.now()
        self._half_open_calls = 0
        
        self.stats.state_changes.append({
            'from': old_state.value,
            'to': CircuitState.OPEN.value,
            'timestamp': self._last_state_change,
            'reason': f'Failures: {self.stats.consecutive_failures}'
        })
        
        logger.error(
            "Circuit opened",
            circuit=self.name,
            consecutive_failures=self.stats.consecutive_failures,
            failure_rate=self._calculate_failure_rate()
        )

    async def _transition_to_half_open(self):
        """Transition to HALF_OPEN state."""
        old_state = self.state
        self.state = CircuitState.HALF_OPEN
        self._last_state_change = datetime.now()
        self._half_open_calls = 0
        self.stats.consecutive_successes = 0
        
        self.stats.state_changes.append({
            'from': old_state.value,
            'to': CircuitState.HALF_OPEN.value,
            'timestamp': self._last_state_change,
            'reason': 'Timeout elapsed'
        })
        
        logger.info(
            "Circuit half-open, testing recovery",
            circuit=self.name
        )

    async def _transition_to_closed(self):
        """Transition to CLOSED state."""
        old_state = self.state
        self.state = CircuitState.CLOSED
        self._last_state_change = datetime.now()
        self._half_open_calls = 0
        self.stats.consecutive_failures = 0
        
        self.stats.state_changes.append({
            'from': old_state.value,
            'to': CircuitState.CLOSED.value,
            'timestamp': self._last_state_change,
            'reason': f'Recovery successful after {self.stats.consecutive_successes} calls'
        })
        
        logger.info(
            "Circuit closed, service recovered",
            circuit=self.name,
            recovery_calls=self.stats.consecutive_successes
        )

    def _calculate_failure_rate(self) -> float:
        """Calculate current failure rate."""
        if not self._call_results:
            return 0.0
        return sum(
            1 for success, _ in self._call_results if not success
        ) / len(self._call_results)

    def get_status(self) -> Dict[str, Any]:
        """Get comprehensive circuit breaker status."""
        return {
            'name': self.name,
            'state': self.state.value,
            'stats': {
                'total_calls': self.stats.total_calls,
                'failed_calls': self.stats.failed_calls,
                'successful_calls': self.stats.successful_calls,
                'slow_calls': self.stats.slow_calls,
                'failure_rate': self._calculate_failure_rate(),
                'consecutive_failures': self.stats.consecutive_failures,
                'consecutive_successes': self.stats.consecutive_successes,
                'last_failure': self.stats.last_failure_time.isoformat() if self.stats.last_failure_time else None,
                'last_success': self.stats.last_success_time.isoformat() if self.stats.last_success_time else None,
                'average_duration': sum(self.stats.call_durations) / len(self.stats.call_durations) if self.stats.call_durations else 0
            },
            'config': {
                'failure_threshold': self.config.failure_threshold,
                'timeout': self.config.timeout,
                'window_size': self.config.window_size
            },
            'state_changes': self.stats.state_changes[-10:]  # Last 10 state changes
        }

    async def reset(self):
        """Manually reset the circuit breaker."""
        async with self._lock:
            self.state = CircuitState.CLOSED
            self.stats.consecutive_failures = 0
            self.stats.consecutive_successes = 0
            self._half_open_calls = 0
            self._call_results.clear()
            logger.info("Circuit breaker manually reset", circuit=self.name)


class CircuitOpenError(Exception):
    """Exception raised when circuit is open."""
    
    def __init__(self, message: str, stats: CircuitBreakerStats):
        super().__init__(message)
        self.stats = stats


class CircuitBreakerManager:
    """Manages multiple circuit breakers for different services."""
    
    def __init__(self):
        self._breakers: Dict[str, CircuitBreaker] = {}
        self._default_config = CircuitBreakerConfig()
        
    def get_breaker(
        self, 
        name: str, 
        config: Optional[CircuitBreakerConfig] = None
    ) -> CircuitBreaker:
        """Get or create a circuit breaker for a service."""
        if name not in self._breakers:
            self._breakers[name] = CircuitBreaker(
                name, 
                config or self._default_config
            )
        return self._breakers[name]
        
    def get_all_status(self) -> Dict[str, Any]:
        """Get status of all circuit breakers."""
        return {
            name: breaker.get_status() 
            for name, breaker in self._breakers.items()
        }
        
    async def reset_all(self):
        """Reset all circuit breakers."""
        for breaker in self._breakers.values():
            await breaker.reset()


# Global circuit breaker manager
circuit_manager = CircuitBreakerManager()