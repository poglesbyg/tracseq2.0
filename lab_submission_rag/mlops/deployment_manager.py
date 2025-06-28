"""
Model Deployment Manager for TracSeq 2.0 MLOps Pipeline

Automated model deployment, rollback, and environment management.
"""

import asyncio
import uuid
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from enum import Enum
from pathlib import Path

import aiofiles
import docker
import structlog
from sqlalchemy import (
    JSON,
    Boolean,
    Column,
    DateTime,
    Integer,
    String,
    Text,
    create_engine,
)
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker

logger = structlog.get_logger(__name__)


class DeploymentStatus(Enum):
    PENDING = "pending"
    DEPLOYING = "deploying"
    DEPLOYED = "deployed"
    FAILED = "failed"
    ROLLING_BACK = "rolling_back"
    ROLLED_BACK = "rolled_back"


class DeploymentEnvironment(Enum):
    STAGING = "staging"
    PRODUCTION = "production"
    CANARY = "canary"
    DEVELOPMENT = "development"


class DeploymentStrategy(Enum):
    BLUE_GREEN = "blue_green"
    ROLLING = "rolling"
    CANARY = "canary"
    IMMEDIATE = "immediate"


@dataclass
class DeploymentConfig:
    """Configuration for model deployment"""

    deployment_id: str
    model_id: str
    model_version: str
    environment: DeploymentEnvironment
    strategy: DeploymentStrategy

    # Deployment settings
    replicas: int = 1
    resource_requests: dict[str, str] = field(
        default_factory=lambda: {"cpu": "500m", "memory": "1Gi"}
    )
    resource_limits: dict[str, str] = field(
        default_factory=lambda: {"cpu": "1000m", "memory": "2Gi"}
    )

    # Health checks
    health_check_path: str = "/health"
    health_check_timeout_seconds: int = 30
    ready_timeout_seconds: int = 300

    # Traffic management
    traffic_percentage: float = 100.0
    canary_traffic_percentage: float = 10.0

    # Rollback settings
    auto_rollback_enabled: bool = True
    rollback_error_threshold: float = 0.1  # 10% error rate
    rollback_latency_threshold_ms: float = 2000
    monitoring_window_minutes: int = 15

    # Environment variables
    environment_variables: dict[str, str] = field(default_factory=dict)

    # Metadata
    created_at: datetime = field(default_factory=datetime.utcnow)
    created_by: str = ""
    description: str = ""


@dataclass
class DeploymentRecord:
    """Record of a deployment"""

    deployment_id: str
    model_id: str
    model_version: str
    environment: DeploymentEnvironment
    strategy: DeploymentStrategy

    # Status and timing
    status: DeploymentStatus
    started_at: datetime | None = None
    completed_at: datetime | None = None

    # Deployment details
    service_name: str = ""
    endpoint_url: str = ""
    replicas_deployed: int = 0

    # Previous deployment for rollback
    previous_deployment_id: str | None = None

    # Health and performance
    health_check_passed: bool = False
    performance_validated: bool = False

    # Logs and errors
    deployment_logs: list[str] = field(default_factory=list)
    error_message: str | None = None

    # Metrics
    deployment_duration_seconds: float = 0.0
    validation_metrics: dict[str, float] = field(default_factory=dict)

    created_at: datetime = field(default_factory=datetime.utcnow)


@dataclass
class ServiceEndpoint:
    """Service endpoint information"""

    service_name: str
    model_id: str
    model_version: str
    environment: DeploymentEnvironment

    # Endpoint details
    url: str
    port: int
    protocol: str = "http"

    # Health status
    is_healthy: bool = True
    last_health_check: datetime | None = None

    # Traffic
    traffic_percentage: float = 0.0

    # Metadata
    created_at: datetime = field(default_factory=datetime.utcnow)
    updated_at: datetime = field(default_factory=datetime.utcnow)


Base = declarative_base()


class DeploymentConfigRecord(Base):
    """Database model for deployment configurations"""

    __tablename__ = "deployment_configs"

    deployment_id = Column(String, primary_key=True)
    model_id = Column(String, nullable=False)
    model_version = Column(String, nullable=False)
    environment = Column(String, nullable=False)
    strategy = Column(String, nullable=False)

    # Settings
    replicas = Column(Integer, default=1)
    resource_requests = Column(JSON)
    resource_limits = Column(JSON)

    # Health checks
    health_check_path = Column(String, default="/health")
    health_check_timeout_seconds = Column(Integer, default=30)
    ready_timeout_seconds = Column(Integer, default=300)

    # Traffic
    traffic_percentage = Column(Integer, default=100)
    canary_traffic_percentage = Column(Integer, default=10)

    # Rollback
    auto_rollback_enabled = Column(Boolean, default=True)
    rollback_error_threshold = Column(Integer, default=0.1)
    rollback_latency_threshold_ms = Column(Integer, default=2000)
    monitoring_window_minutes = Column(Integer, default=15)

    # Environment
    environment_variables = Column(JSON)

    # Metadata
    created_at = Column(DateTime, default=datetime.utcnow)
    created_by = Column(String)
    description = Column(Text)


class DeploymentRecordTable(Base):
    """Database model for deployment records"""

    __tablename__ = "deployment_records"

    deployment_id = Column(String, primary_key=True)
    model_id = Column(String, nullable=False)
    model_version = Column(String, nullable=False)
    environment = Column(String, nullable=False)
    strategy = Column(String, nullable=False)

    # Status
    status = Column(String, nullable=False)
    started_at = Column(DateTime)
    completed_at = Column(DateTime)

    # Details
    service_name = Column(String)
    endpoint_url = Column(String)
    replicas_deployed = Column(Integer, default=0)

    # Previous deployment
    previous_deployment_id = Column(String)

    # Health
    health_check_passed = Column(Boolean, default=False)
    performance_validated = Column(Boolean, default=False)

    # Logs and errors
    deployment_logs = Column(JSON)
    error_message = Column(Text)

    # Metrics
    deployment_duration_seconds = Column(Integer, default=0)
    validation_metrics = Column(JSON)

    created_at = Column(DateTime, default=datetime.utcnow)


class ServiceEndpointRecord(Base):
    """Database model for service endpoints"""

    __tablename__ = "service_endpoints"

    service_name = Column(String, primary_key=True)
    model_id = Column(String, nullable=False)
    model_version = Column(String, nullable=False)
    environment = Column(String, nullable=False)

    # Endpoint
    url = Column(String, nullable=False)
    port = Column(Integer, nullable=False)
    protocol = Column(String, default="http")

    # Health
    is_healthy = Column(Boolean, default=True)
    last_health_check = Column(DateTime)

    # Traffic
    traffic_percentage = Column(Integer, default=0)

    # Metadata
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow)


class ModelDeploymentManager:
    """
    Comprehensive model deployment management system.

    Features:
    - Multiple deployment strategies (blue-green, rolling, canary)
    - Automated health checks and validation
    - Traffic management and routing
    - Automatic rollback on failures
    - Multi-environment support
    - Container orchestration
    - Service discovery
    """

    def __init__(
        self,
        database_url: str,
        container_registry_url: str,
        kubernetes_config_path: str | None = None,
        docker_client: docker.DockerClient | None = None,
    ):
        # Database setup
        self.engine = create_engine(database_url)
        Base.metadata.create_all(self.engine)
        self.SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=self.engine)

        # Container infrastructure
        self.container_registry_url = container_registry_url
        self.docker_client = docker_client or docker.from_env()
        self.kubernetes_config_path = kubernetes_config_path

        # Service registry
        self.active_services: dict[str, ServiceEndpoint] = {}

        # Background tasks
        self._monitoring_tasks: list[asyncio.Task] = []

    async def start_monitoring(self):
        """Start background monitoring tasks."""
        # Health check monitoring
        task = asyncio.create_task(self._health_check_loop())
        self._monitoring_tasks.append(task)

        # Rollback monitoring
        task = asyncio.create_task(self._rollback_monitoring_loop())
        self._monitoring_tasks.append(task)

        # Service discovery updates
        task = asyncio.create_task(self._service_discovery_loop())
        self._monitoring_tasks.append(task)

        logger.info("Deployment monitoring started", active_tasks=len(self._monitoring_tasks))

    async def stop_monitoring(self):
        """Stop all monitoring tasks."""
        for task in self._monitoring_tasks:
            task.cancel()

        await asyncio.gather(*self._monitoring_tasks, return_exceptions=True)
        self._monitoring_tasks.clear()

        logger.info("Deployment monitoring stopped")

    async def deploy_model(self, config: DeploymentConfig) -> str:
        """Deploy a model using the specified configuration."""
        if not config.deployment_id:
            config.deployment_id = f"deploy_{uuid.uuid4().hex[:8]}"

        # Create deployment record
        deployment_record = DeploymentRecord(
            deployment_id=config.deployment_id,
            model_id=config.model_id,
            model_version=config.model_version,
            environment=config.environment,
            strategy=config.strategy,
            status=DeploymentStatus.PENDING,
        )

        await self._store_deployment_record(deployment_record)

        # Start deployment asynchronously
        task = asyncio.create_task(self._execute_deployment(config, deployment_record))

        logger.info(
            "Model deployment initiated",
            deployment_id=config.deployment_id,
            model_id=config.model_id,
            model_version=config.model_version,
            environment=config.environment.value,
            strategy=config.strategy.value,
        )

        return config.deployment_id

    async def rollback_deployment(
        self, deployment_id: str, reason: str = "Manual rollback"
    ) -> bool:
        """Rollback a deployment to the previous version."""
        deployment_record = await self.get_deployment_record(deployment_id)
        if not deployment_record:
            logger.error("Deployment not found", deployment_id=deployment_id)
            return False

        if not deployment_record.previous_deployment_id:
            logger.error("No previous deployment to rollback to", deployment_id=deployment_id)
            return False

        # Update status
        deployment_record.status = DeploymentStatus.ROLLING_BACK
        await self._update_deployment_record(deployment_record)

        # Execute rollback
        success = await self._execute_rollback(deployment_record, reason)

        # Update final status
        final_status = DeploymentStatus.ROLLED_BACK if success else DeploymentStatus.FAILED
        deployment_record.status = final_status
        deployment_record.completed_at = datetime.utcnow()

        if not success:
            deployment_record.error_message = f"Rollback failed: {reason}"

        await self._update_deployment_record(deployment_record)

        logger.info(
            "Deployment rollback completed",
            deployment_id=deployment_id,
            success=success,
            reason=reason,
        )

        return success

    async def get_deployment_record(self, deployment_id: str) -> DeploymentRecord | None:
        """Get deployment record by ID."""
        with self.SessionLocal() as session:
            record = (
                session.query(DeploymentRecordTable)
                .filter(DeploymentRecordTable.deployment_id == deployment_id)
                .first()
            )

            if not record:
                return None

            return DeploymentRecord(
                deployment_id=record.deployment_id,
                model_id=record.model_id,
                model_version=record.model_version,
                environment=DeploymentEnvironment(record.environment),
                strategy=DeploymentStrategy(record.strategy),
                status=DeploymentStatus(record.status),
                started_at=record.started_at,
                completed_at=record.completed_at,
                service_name=record.service_name or "",
                endpoint_url=record.endpoint_url or "",
                replicas_deployed=record.replicas_deployed,
                previous_deployment_id=record.previous_deployment_id,
                health_check_passed=record.health_check_passed,
                performance_validated=record.performance_validated,
                deployment_logs=record.deployment_logs or [],
                error_message=record.error_message,
                deployment_duration_seconds=record.deployment_duration_seconds or 0.0,
                validation_metrics=record.validation_metrics or {},
                created_at=record.created_at,
            )

    async def list_deployments(
        self,
        model_id: str | None = None,
        environment: DeploymentEnvironment | None = None,
        status: DeploymentStatus | None = None,
        limit: int = 50,
    ) -> list[DeploymentRecord]:
        """List deployment records."""
        with self.SessionLocal() as session:
            query = session.query(DeploymentRecordTable)

            if model_id:
                query = query.filter(DeploymentRecordTable.model_id == model_id)
            if environment:
                query = query.filter(DeploymentRecordTable.environment == environment.value)
            if status:
                query = query.filter(DeploymentRecordTable.status == status.value)

            query = query.order_by(DeploymentRecordTable.created_at.desc()).limit(limit)
            records = query.all()

            deployments = []
            for record in records:
                deployment = await self.get_deployment_record(record.deployment_id)
                if deployment:
                    deployments.append(deployment)

            return deployments

    async def get_active_services(
        self, environment: DeploymentEnvironment | None = None
    ) -> list[ServiceEndpoint]:
        """Get active service endpoints."""
        with self.SessionLocal() as session:
            query = session.query(ServiceEndpointRecord)

            if environment:
                query = query.filter(ServiceEndpointRecord.environment == environment.value)

            records = query.all()

            services = []
            for record in records:
                service = ServiceEndpoint(
                    service_name=record.service_name,
                    model_id=record.model_id,
                    model_version=record.model_version,
                    environment=DeploymentEnvironment(record.environment),
                    url=record.url,
                    port=record.port,
                    protocol=record.protocol,
                    is_healthy=record.is_healthy,
                    last_health_check=record.last_health_check,
                    traffic_percentage=record.traffic_percentage,
                    created_at=record.created_at,
                    updated_at=record.updated_at,
                )
                services.append(service)

            return services

    async def update_traffic_routing(self, service_name: str, traffic_percentage: float) -> bool:
        """Update traffic routing for a service."""
        with self.SessionLocal() as session:
            record = (
                session.query(ServiceEndpointRecord)
                .filter(ServiceEndpointRecord.service_name == service_name)
                .first()
            )

            if record:
                record.traffic_percentage = traffic_percentage
                record.updated_at = datetime.utcnow()
                session.commit()

                # Update load balancer configuration
                await self._update_load_balancer_config(service_name, traffic_percentage)

                logger.info(
                    "Traffic routing updated",
                    service_name=service_name,
                    traffic_percentage=traffic_percentage,
                )
                return True

            return False

    async def _execute_deployment(self, config: DeploymentConfig, record: DeploymentRecord):
        """Execute the deployment process."""
        try:
            start_time = datetime.utcnow()

            # Update status to deploying
            record.status = DeploymentStatus.DEPLOYING
            record.started_at = start_time
            await self._update_deployment_record(record)

            # Get previous deployment for rollback reference
            previous_deployment = await self._get_current_deployment(
                config.model_id, config.environment
            )
            if previous_deployment:
                record.previous_deployment_id = previous_deployment.deployment_id

            # Build and push container image
            image_tag = await self._build_and_push_image(config, record)

            # Deploy based on strategy
            if config.strategy == DeploymentStrategy.BLUE_GREEN:
                success = await self._deploy_blue_green(config, record, image_tag)
            elif config.strategy == DeploymentStrategy.ROLLING:
                success = await self._deploy_rolling(config, record, image_tag)
            elif config.strategy == DeploymentStrategy.CANARY:
                success = await self._deploy_canary(config, record, image_tag)
            else:  # IMMEDIATE
                success = await self._deploy_immediate(config, record, image_tag)

            if not success:
                raise Exception("Deployment strategy execution failed")

            # Health checks
            record.health_check_passed = await self._wait_for_health_checks(config, record)
            if not record.health_check_passed:
                raise Exception("Health checks failed")

            # Performance validation
            record.performance_validated = await self._validate_performance(config, record)
            if not record.performance_validated:
                logger.warning("Performance validation failed", deployment_id=config.deployment_id)

            # Update service registry
            await self._register_service_endpoint(config, record)

            # Calculate deployment duration
            end_time = datetime.utcnow()
            record.deployment_duration_seconds = (end_time - start_time).total_seconds()

            # Mark as deployed
            record.status = DeploymentStatus.DEPLOYED
            record.completed_at = end_time

            await self._update_deployment_record(record)

            logger.info(
                "Model deployment completed successfully",
                deployment_id=config.deployment_id,
                duration_seconds=record.deployment_duration_seconds,
            )

        except Exception as e:
            # Mark deployment as failed
            record.status = DeploymentStatus.FAILED
            record.completed_at = datetime.utcnow()
            record.error_message = str(e)

            await self._update_deployment_record(record)

            logger.error(
                "Model deployment failed", deployment_id=config.deployment_id, error=str(e)
            )

            # Attempt automatic rollback if enabled
            if config.auto_rollback_enabled and record.previous_deployment_id:
                await self._execute_rollback(
                    record, f"Auto-rollback due to deployment failure: {str(e)}"
                )

    async def _build_and_push_image(
        self, config: DeploymentConfig, record: DeploymentRecord
    ) -> str:
        """Build and push container image."""
        image_tag = f"{self.container_registry_url}/{config.model_id}:{config.model_version}"

        # Create Dockerfile
        dockerfile_content = await self._generate_dockerfile(config)

        # Build image
        build_logs = []
        try:
            # Write Dockerfile
            dockerfile_path = Path(f"/tmp/Dockerfile_{config.deployment_id}")
            async with aiofiles.open(dockerfile_path, "w") as f:
                await f.write(dockerfile_content)

            # Build Docker image
            image, build_logs = self.docker_client.images.build(
                path=str(dockerfile_path.parent),
                dockerfile=str(dockerfile_path),
                tag=image_tag,
                rm=True,
            )

            # Push to registry
            push_logs = self.docker_client.images.push(image_tag, stream=True, decode=True)

            # Collect logs
            for log in push_logs:
                if "stream" in log:
                    build_logs.append(log["stream"].strip())

            record.deployment_logs.extend(build_logs)

            logger.info(
                "Container image built and pushed",
                deployment_id=config.deployment_id,
                image_tag=image_tag,
            )

            return image_tag

        except Exception as e:
            record.deployment_logs.extend(build_logs)
            record.deployment_logs.append(f"Error: {str(e)}")
            raise

    async def _deploy_blue_green(
        self, config: DeploymentConfig, record: DeploymentRecord, image_tag: str
    ) -> bool:
        """Execute blue-green deployment."""
        # This is a simplified implementation
        # In production, integrate with Kubernetes or your orchestration platform

        service_name = f"{config.model_id}-{config.environment.value}-green"

        try:
            # Deploy green environment
            container = self.docker_client.containers.run(
                image_tag,
                name=service_name,
                ports={"8080/tcp": None},  # Let Docker assign port
                environment=config.environment_variables,
                detach=True,
            )

            # Get assigned port
            container.reload()
            port_info = container.attrs["NetworkSettings"]["Ports"]["8080/tcp"][0]
            port = int(port_info["HostPort"])

            record.service_name = service_name
            record.endpoint_url = f"http://localhost:{port}"
            record.replicas_deployed = 1

            logger.info(
                "Blue-green deployment completed",
                deployment_id=config.deployment_id,
                service_name=service_name,
                port=port,
            )

            return True

        except Exception as e:
            logger.error(
                "Blue-green deployment failed", deployment_id=config.deployment_id, error=str(e)
            )
            return False

    async def _deploy_rolling(
        self, config: DeploymentConfig, record: DeploymentRecord, image_tag: str
    ) -> bool:
        """Execute rolling deployment."""
        # Simplified rolling deployment
        service_name = f"{config.model_id}-{config.environment.value}"

        try:
            # Stop old container if exists
            try:
                old_container = self.docker_client.containers.get(service_name)
                old_container.stop()
                old_container.remove()
            except docker.errors.NotFound:
                pass

            # Start new container
            container = self.docker_client.containers.run(
                image_tag,
                name=service_name,
                ports={"8080/tcp": None},
                environment=config.environment_variables,
                detach=True,
            )

            # Get assigned port
            container.reload()
            port_info = container.attrs["NetworkSettings"]["Ports"]["8080/tcp"][0]
            port = int(port_info["HostPort"])

            record.service_name = service_name
            record.endpoint_url = f"http://localhost:{port}"
            record.replicas_deployed = 1

            logger.info(
                "Rolling deployment completed",
                deployment_id=config.deployment_id,
                service_name=service_name,
            )

            return True

        except Exception as e:
            logger.error(
                "Rolling deployment failed", deployment_id=config.deployment_id, error=str(e)
            )
            return False

    async def _deploy_canary(
        self, config: DeploymentConfig, record: DeploymentRecord, image_tag: str
    ) -> bool:
        """Execute canary deployment."""
        # Simplified canary deployment
        service_name = f"{config.model_id}-{config.environment.value}-canary"

        try:
            # Deploy canary version
            container = self.docker_client.containers.run(
                image_tag,
                name=service_name,
                ports={"8080/tcp": None},
                environment=config.environment_variables,
                detach=True,
            )

            # Get assigned port
            container.reload()
            port_info = container.attrs["NetworkSettings"]["Ports"]["8080/tcp"][0]
            port = int(port_info["HostPort"])

            record.service_name = service_name
            record.endpoint_url = f"http://localhost:{port}"
            record.replicas_deployed = 1

            # Set initial canary traffic
            await self._update_canary_traffic(service_name, config.canary_traffic_percentage)

            logger.info(
                "Canary deployment completed",
                deployment_id=config.deployment_id,
                service_name=service_name,
                canary_traffic=config.canary_traffic_percentage,
            )

            return True

        except Exception as e:
            logger.error(
                "Canary deployment failed", deployment_id=config.deployment_id, error=str(e)
            )
            return False

    async def _deploy_immediate(
        self, config: DeploymentConfig, record: DeploymentRecord, image_tag: str
    ) -> bool:
        """Execute immediate deployment."""
        return await self._deploy_rolling(config, record, image_tag)

    async def _wait_for_health_checks(
        self, config: DeploymentConfig, record: DeploymentRecord
    ) -> bool:
        """Wait for service to pass health checks."""
        if not record.endpoint_url:
            return False

        import aiohttp

        health_url = f"{record.endpoint_url}{config.health_check_path}"
        timeout = aiohttp.ClientTimeout(total=config.health_check_timeout_seconds)

        start_time = datetime.utcnow()
        max_wait_time = timedelta(seconds=config.ready_timeout_seconds)

        async with aiohttp.ClientSession(timeout=timeout) as session:
            while datetime.utcnow() - start_time < max_wait_time:
                try:
                    async with session.get(health_url) as response:
                        if response.status == 200:
                            logger.info(
                                "Health check passed",
                                deployment_id=config.deployment_id,
                                health_url=health_url,
                            )
                            return True
                except Exception as e:
                    logger.debug(
                        "Health check failed, retrying",
                        deployment_id=config.deployment_id,
                        error=str(e),
                    )

                await asyncio.sleep(5)  # Wait 5 seconds before retry

        logger.error(
            "Health checks timed out",
            deployment_id=config.deployment_id,
            timeout_seconds=config.ready_timeout_seconds,
        )
        return False

    async def _validate_performance(
        self, config: DeploymentConfig, record: DeploymentRecord
    ) -> bool:
        """Validate deployment performance."""
        # This is a placeholder for performance validation
        # In production, implement actual performance tests

        # Simulate performance validation
        await asyncio.sleep(2)

        # Mock validation metrics
        record.validation_metrics = {
            "avg_latency_ms": 150.0,
            "throughput_rps": 100.0,
            "error_rate": 0.01,
        }

        # Check if metrics meet criteria
        if record.validation_metrics["avg_latency_ms"] > 1000:
            return False

        if record.validation_metrics["error_rate"] > 0.05:
            return False

        return True

    async def _execute_rollback(self, record: DeploymentRecord, reason: str) -> bool:
        """Execute rollback to previous deployment."""
        try:
            if not record.previous_deployment_id:
                return False

            previous_deployment = await self.get_deployment_record(record.previous_deployment_id)
            if not previous_deployment:
                return False

            # Stop current service
            if record.service_name:
                try:
                    container = self.docker_client.containers.get(record.service_name)
                    container.stop()
                    container.remove()
                except docker.errors.NotFound:
                    pass

            # Restart previous service
            # This is simplified - in production, implement proper rollback logic

            logger.info(
                "Rollback completed",
                deployment_id=record.deployment_id,
                previous_deployment_id=previous_deployment.deployment_id,
                reason=reason,
            )

            return True

        except Exception as e:
            logger.error("Rollback failed", deployment_id=record.deployment_id, error=str(e))
            return False

    async def _generate_dockerfile(self, config: DeploymentConfig) -> str:
        """Generate Dockerfile for model deployment."""
        dockerfile = f"""
FROM python:3.9-slim

# Install dependencies
COPY requirements.txt .
RUN pip install -r requirements.txt

# Copy model and application code
COPY model/ /app/model/
COPY app/ /app/
WORKDIR /app

# Set environment variables
ENV MODEL_ID={config.model_id}
ENV MODEL_VERSION={config.model_version}
ENV ENVIRONMENT={config.environment.value}

# Additional environment variables
"""

        for key, value in config.environment_variables.items():
            dockerfile += f"ENV {key}={value}\n"

        dockerfile += """
# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \\
    CMD curl -f http://localhost:8080/health || exit 1

# Start application
CMD ["python", "main.py"]
"""

        return dockerfile

    async def _register_service_endpoint(self, config: DeploymentConfig, record: DeploymentRecord):
        """Register service endpoint in service registry."""
        if not record.endpoint_url or not record.service_name:
            return

        # Parse URL to get port
        from urllib.parse import urlparse

        parsed_url = urlparse(record.endpoint_url)
        port = parsed_url.port or 80

        with self.SessionLocal() as session:
            # Remove old endpoint if exists
            old_record = (
                session.query(ServiceEndpointRecord)
                .filter(
                    ServiceEndpointRecord.model_id == config.model_id,
                    ServiceEndpointRecord.environment == config.environment.value,
                )
                .first()
            )

            if old_record:
                session.delete(old_record)

            # Add new endpoint
            endpoint_record = ServiceEndpointRecord(
                service_name=record.service_name,
                model_id=config.model_id,
                model_version=config.model_version,
                environment=config.environment.value,
                url=record.endpoint_url,
                port=port,
                traffic_percentage=config.traffic_percentage,
            )

            session.add(endpoint_record)
            session.commit()

        logger.info(
            "Service endpoint registered", service_name=record.service_name, url=record.endpoint_url
        )

    async def _get_current_deployment(
        self, model_id: str, environment: DeploymentEnvironment
    ) -> DeploymentRecord | None:
        """Get current active deployment for model in environment."""
        deployments = await self.list_deployments(
            model_id=model_id, environment=environment, status=DeploymentStatus.DEPLOYED, limit=1
        )

        return deployments[0] if deployments else None

    async def _store_deployment_record(self, record: DeploymentRecord):
        """Store deployment record in database."""
        with self.SessionLocal() as session:
            db_record = DeploymentRecordTable(
                deployment_id=record.deployment_id,
                model_id=record.model_id,
                model_version=record.model_version,
                environment=record.environment.value,
                strategy=record.strategy.value,
                status=record.status.value,
                started_at=record.started_at,
                completed_at=record.completed_at,
                service_name=record.service_name,
                endpoint_url=record.endpoint_url,
                replicas_deployed=record.replicas_deployed,
                previous_deployment_id=record.previous_deployment_id,
                health_check_passed=record.health_check_passed,
                performance_validated=record.performance_validated,
                deployment_logs=record.deployment_logs,
                error_message=record.error_message,
                deployment_duration_seconds=record.deployment_duration_seconds,
                validation_metrics=record.validation_metrics,
                created_at=record.created_at,
            )
            session.add(db_record)
            session.commit()

    async def _update_deployment_record(self, record: DeploymentRecord):
        """Update deployment record in database."""
        with self.SessionLocal() as session:
            db_record = (
                session.query(DeploymentRecordTable)
                .filter(DeploymentRecordTable.deployment_id == record.deployment_id)
                .first()
            )

            if db_record:
                db_record.status = record.status.value
                db_record.started_at = record.started_at
                db_record.completed_at = record.completed_at
                db_record.service_name = record.service_name
                db_record.endpoint_url = record.endpoint_url
                db_record.replicas_deployed = record.replicas_deployed
                db_record.previous_deployment_id = record.previous_deployment_id
                db_record.health_check_passed = record.health_check_passed
                db_record.performance_validated = record.performance_validated
                db_record.deployment_logs = record.deployment_logs
                db_record.error_message = record.error_message
                db_record.deployment_duration_seconds = record.deployment_duration_seconds
                db_record.validation_metrics = record.validation_metrics
                session.commit()

    # Background monitoring methods
    async def _health_check_loop(self):
        """Continuous health checking of deployed services."""
        while True:
            try:
                services = await self.get_active_services()
                for service in services:
                    await self._check_service_health(service)

                await asyncio.sleep(30)  # Check every 30 seconds
            except Exception as e:
                logger.error("Error in health check loop", error=str(e))
                await asyncio.sleep(30)

    async def _rollback_monitoring_loop(self):
        """Monitor deployments for automatic rollback conditions."""
        while True:
            try:
                # Get active deployments with auto-rollback enabled
                deployments = await self.list_deployments(status=DeploymentStatus.DEPLOYED)

                for deployment in deployments:
                    # Check if rollback conditions are met
                    # This would integrate with your monitoring system
                    pass

                await asyncio.sleep(60)  # Check every minute
            except Exception as e:
                logger.error("Error in rollback monitoring", error=str(e))
                await asyncio.sleep(60)

    async def _service_discovery_loop(self):
        """Update service discovery information."""
        while True:
            try:
                # Update service registry with current container states
                # This would integrate with your service discovery system
                await asyncio.sleep(60)
            except Exception as e:
                logger.error("Error in service discovery", error=str(e))
                await asyncio.sleep(60)

    async def _check_service_health(self, service: ServiceEndpoint):
        """Check health of a single service."""
        try:
            import aiohttp

            health_url = f"{service.url}/health"
            timeout = aiohttp.ClientTimeout(total=10)

            async with aiohttp.ClientSession(timeout=timeout) as session:
                async with session.get(health_url) as response:
                    is_healthy = response.status == 200

            # Update health status in database
            with self.SessionLocal() as session:
                record = (
                    session.query(ServiceEndpointRecord)
                    .filter(ServiceEndpointRecord.service_name == service.service_name)
                    .first()
                )

                if record:
                    record.is_healthy = is_healthy
                    record.last_health_check = datetime.utcnow()
                    session.commit()

        except Exception as e:
            logger.warning(
                "Service health check failed", service_name=service.service_name, error=str(e)
            )

    async def _update_load_balancer_config(self, service_name: str, traffic_percentage: float):
        """Update load balancer configuration for traffic routing."""
        # This is a placeholder for load balancer integration
        # In production, integrate with your load balancer (nginx, HAProxy, etc.)
        logger.info(
            "Load balancer configuration updated",
            service_name=service_name,
            traffic_percentage=traffic_percentage,
        )

    async def _update_canary_traffic(self, service_name: str, traffic_percentage: float):
        """Update canary traffic routing."""
        # This is a placeholder for canary traffic management
        # In production, integrate with your traffic management system
        logger.info(
            "Canary traffic updated",
            service_name=service_name,
            traffic_percentage=traffic_percentage,
        )
