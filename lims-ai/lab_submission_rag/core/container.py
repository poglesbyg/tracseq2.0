"""
Dependency Injection Container for the Laboratory Submission RAG System

This module provides a service container that manages component lifecycles,
dependencies, and provides a centralized way to configure and access services.
"""

import asyncio
import logging
from collections.abc import Callable
from contextlib import asynccontextmanager
from typing import Any, TypeVar

from .exceptions import ServiceException
from .factories import (
    CircuitBreakerFactory,
    DocumentProcessorFactory,
    LLMInterfaceFactory,
    RetryPolicyFactory,
    VectorStoreFactory,
)
from .interfaces import (
    IDocumentProcessor,
    IHealthChecker,
    ILLMInterface,
    ISubmissionRepository,
    ISubmissionService,
    IVectorStore,
)
from .services import SubmissionService

logger = logging.getLogger(__name__)

T = TypeVar("T")


class ServiceContainer:
    """
    Dependency injection container for managing services and their dependencies

    This container provides:
    - Service registration and resolution
    - Singleton lifecycle management
    - Dependency injection
    - Configuration management
    - Health checking
    """

    def __init__(self, config: dict[str, Any] | None = None) -> None:
        self.config = config or {}
        self._services: dict[str, Any] = {}
        self._factories: dict[str, Callable] = {}
        self._singletons: dict[str, Any] = {}
        self._health_checkers: dict[str, IHealthChecker] = {}
        self._initialized = False

        # Register default factories
        self._register_default_factories()

        logger.info("ServiceContainer initialized")

    def _register_default_factories(self) -> None:
        """Register default component factories"""
        self._factories.update(
            {
                "document_processor_factory": lambda: DocumentProcessorFactory(self.config),
                "vector_store_factory": lambda: VectorStoreFactory(self.config),
                "llm_interface_factory": lambda: LLMInterfaceFactory(self.config),
                "retry_policy_factory": lambda: RetryPolicyFactory(self.config),
                "circuit_breaker_factory": lambda: CircuitBreakerFactory(self.config),
            }
        )

    def register_singleton(self, name: str, factory: Callable[[], T]) -> None:
        """Register a singleton service"""
        self._factories[name] = factory
        logger.debug(f"Registered singleton factory: {name}")

    def register_transient(self, name: str, factory: Callable[[], T]) -> None:
        """Register a transient service (new instance each time)"""
        self._services[name] = factory
        logger.debug(f"Registered transient factory: {name}")

    def register_instance(self, name: str, instance: T) -> None:
        """Register a specific instance"""
        self._singletons[name] = instance
        logger.debug(f"Registered instance: {name}")

    def resolve(self, name: str) -> Any:
        """Resolve a service by name"""
        try:
            # Check for existing singleton
            if name in self._singletons:
                return self._singletons[name]

            # Check for singleton factory
            if name in self._factories:
                instance = self._factories[name]()
                self._singletons[name] = instance
                logger.debug(f"Created singleton instance: {name}")
                return instance

            # Check for transient service
            if name in self._services:
                return self._services[name]()

            raise ServiceException(
                f"Service '{name}' not registered",
                service_name="ServiceContainer",
                operation="resolve",
            )

        except Exception as e:
            logger.error(f"Error resolving service '{name}': {str(e)}")
            raise ServiceException(
                f"Failed to resolve service '{name}': {str(e)}",
                service_name="ServiceContainer",
                operation="resolve",
                cause=e,
            )

    async def initialize(self) -> None:
        """Initialize the container and all registered services"""
        if self._initialized:
            return

        try:
            logger.info("Initializing ServiceContainer...")

            # Initialize core services in dependency order
            await self._initialize_core_services()

            # Register submission service
            self._register_submission_service()

            # Initialize health checkers
            self._initialize_health_checkers()

            self._initialized = True
            logger.info("ServiceContainer initialization completed")

        except Exception as e:
            logger.error(f"ServiceContainer initialization failed: {str(e)}")
            raise ServiceException(
                f"Container initialization failed: {str(e)}",
                service_name="ServiceContainer",
                operation="initialize",
                cause=e,
            )

    async def _initialize_core_services(self) -> None:
        """Initialize core services"""
        try:
            # Create document processor
            doc_processor_factory = self.resolve("document_processor_factory")
            document_processor = doc_processor_factory.create()
            self.register_instance("document_processor", document_processor)

            # Create vector store
            vector_store_factory = self.resolve("vector_store_factory")
            vector_store = vector_store_factory.create()
            self.register_instance("vector_store", vector_store)

            # Create LLM interface
            llm_factory = self.resolve("llm_interface_factory")
            llm_interface = llm_factory.create()
            self.register_instance("llm_interface", llm_interface)

            # Create resilience components
            retry_factory = self.resolve("retry_policy_factory")
            retry_policy = retry_factory.create()
            self.register_instance("retry_policy", retry_policy)

            circuit_breaker_factory = self.resolve("circuit_breaker_factory")
            circuit_breaker = circuit_breaker_factory.create()
            self.register_instance("circuit_breaker", circuit_breaker)

            # Create repository (this would need to be implemented based on your repository)
            self._register_repository()

            logger.info("Core services initialized successfully")

        except Exception as e:
            logger.error(f"Failed to initialize core services: {str(e)}")
            raise

    def _register_repository(self) -> None:
        """Register submission repository"""
        try:
            # Import here to avoid circular imports
            from database import db_manager
            from repositories.submission_repository import SubmissionRepository

            # Create a factory for the repository
            def create_repository():
                # This would typically use dependency injection for the session
                return SubmissionRepository(db_manager.get_session())

            self.register_singleton("submission_repository", create_repository)
            logger.debug("Registered submission repository")

        except Exception as e:
            logger.warning(f"Failed to register repository: {str(e)}")
            # Register a mock repository for testing
            self.register_instance("submission_repository", MockSubmissionRepository())

    def _register_submission_service(self) -> None:
        """Register the main submission service"""
        try:

            def create_submission_service():
                return SubmissionService(
                    document_processor=self.resolve("document_processor"),
                    vector_store=self.resolve("vector_store"),
                    llm_interface=self.resolve("llm_interface"),
                    submission_repository=self.resolve("submission_repository"),
                    circuit_breaker=self.resolve("circuit_breaker"),
                    retry_policy=self.resolve("retry_policy"),
                    batch_size=self.config.get("batch_size", 5),
                )

            self.register_singleton("submission_service", create_submission_service)
            logger.debug("Registered submission service")

        except Exception as e:
            logger.error(f"Failed to register submission service: {str(e)}")
            raise

    def _initialize_health_checkers(self) -> None:
        """Initialize health checkers for all services"""
        try:
            # Register health checkers for core components
            health_checkers = {
                "document_processor": DocumentProcessorHealthChecker(
                    self.resolve("document_processor")
                ),
                "vector_store": VectorStoreHealthChecker(self.resolve("vector_store")),
                "llm_interface": LLMInterfaceHealthChecker(self.resolve("llm_interface")),
            }

            for name, health_checker in health_checkers.items():
                self._health_checkers[name] = health_checker

            logger.debug(f"Initialized {len(health_checkers)} health checkers")

        except Exception as e:
            logger.warning(f"Failed to initialize health checkers: {str(e)}")

    async def health_check(self) -> dict[str, Any]:
        """Perform health check on all registered services"""
        health_status = {
            "status": "healthy",
            "timestamp": asyncio.get_event_loop().time(),
            "services": {},
        }

        overall_healthy = True

        for name, health_checker in self._health_checkers.items():
            try:
                service_health = await health_checker.check_health()
                health_status["services"][name] = service_health

                if not service_health.get("healthy", False):
                    overall_healthy = False

            except Exception as e:
                logger.error(f"Health check failed for {name}: {str(e)}")
                health_status["services"][name] = {"healthy": False, "error": str(e)}
                overall_healthy = False

        health_status["status"] = "healthy" if overall_healthy else "unhealthy"
        return health_status

    async def shutdown(self) -> None:
        """Shutdown the container and cleanup resources"""
        logger.info("Shutting down ServiceContainer...")

        try:
            # Shutdown services in reverse order
            for name in reversed(list(self._singletons.keys())):
                instance = self._singletons[name]
                if hasattr(instance, "shutdown"):
                    try:
                        await instance.shutdown()
                        logger.debug(f"Shutdown service: {name}")
                    except Exception as e:
                        logger.error(f"Error shutting down {name}: {str(e)}")

            self._singletons.clear()
            self._services.clear()
            self._health_checkers.clear()
            self._initialized = False

            logger.info("ServiceContainer shutdown completed")

        except Exception as e:
            logger.error(f"Error during container shutdown: {str(e)}")

    @asynccontextmanager
    async def managed_lifecycle(self) -> None:
        """Context manager for automatic container lifecycle management"""
        try:
            await self.initialize()
            yield self
        finally:
            await self.shutdown()

    def get_submission_service(self) -> ISubmissionService:
        """Get the main submission service"""
        if not self._initialized:
            raise ServiceException(
                "Container not initialized",
                service_name="ServiceContainer",
                operation="get_submission_service",
            )
        return self.resolve("submission_service")


# Health checker implementations


class ComponentHealthChecker(IHealthChecker):
    """Base health checker for components"""

    def __init__(self, component: Any, component_name: str) -> None:
        self.component = component
        self.component_name = component_name

    def get_component_name(self) -> str:
        return self.component_name

    async def check_health(self) -> dict[str, Any]:
        """Basic health check implementation"""
        try:
            # Check if component is available
            if self.component is None:
                return {
                    "healthy": False,
                    "component": self.component_name,
                    "error": "Component is None",
                }

            # Component-specific health check
            is_healthy = await self._component_specific_check()

            return {
                "healthy": is_healthy,
                "component": self.component_name,
                "timestamp": asyncio.get_event_loop().time(),
            }

        except Exception as e:
            return {"healthy": False, "component": self.component_name, "error": str(e)}

    async def _component_specific_check(self) -> bool:
        """Override in subclasses for component-specific checks"""
        return True


class DocumentProcessorHealthChecker(ComponentHealthChecker):
    """Health checker for document processor"""

    def __init__(self, document_processor: IDocumentProcessor) -> None:
        super().__init__(document_processor, "document_processor")

    async def _component_specific_check(self) -> bool:
        """Check document processor health"""
        try:
            # Check if supported formats are available
            formats = self.component.get_supported_formats()
            return len(formats) > 0
        except Exception:
            return False


class VectorStoreHealthChecker(ComponentHealthChecker):
    """Health checker for vector store"""

    def __init__(self, vector_store: IVectorStore) -> None:
        super().__init__(vector_store, "vector_store")

    async def _component_specific_check(self) -> bool:
        """Check vector store health"""
        try:
            # Try to get collection stats
            stats = await self.component.get_collection_stats()
            return isinstance(stats, dict)
        except Exception:
            return False


class LLMInterfaceHealthChecker(ComponentHealthChecker):
    """Health checker for LLM interface"""

    def __init__(self, llm_interface: ILLMInterface) -> None:
        super().__init__(llm_interface, "llm_interface")

    async def _component_specific_check(self) -> bool:
        """Check LLM interface health"""
        try:
            return await self.component.health_check()
        except Exception:
            return False


# Mock implementations for testing


class MockSubmissionRepository(ISubmissionRepository):
    """Mock repository for testing"""

    def __init__(self) -> None:
        self._submissions = {}

    async def create_submission(self, submission) -> None:
        submission_id = f"mock_{len(self._submissions)}"
        self._submissions[submission_id] = submission
        return submission_id

    async def get_submission(self, submission_id) -> None:
        return self._submissions.get(submission_id)

    async def update_submission(self, submission) -> None:
        if submission.submission_id in self._submissions:
            self._submissions[submission.submission_id] = submission
            return True
        return False

    async def delete_submission(self, submission_id) -> None:
        if submission_id in self._submissions:
            del self._submissions[submission_id]
            return True
        return False

    async def search_submissions(self, criteria, limit=100, offset=0) -> None:
        # Simple mock implementation
        return list(self._submissions.values())[offset : offset + limit]

    async def get_submission_statistics(self) -> None:
        return {
            "total_submissions": len(self._submissions),
            "total_samples": len(self._submissions) * 5,  # Mock data
        }
