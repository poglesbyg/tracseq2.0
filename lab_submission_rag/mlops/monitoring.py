"""
Model Monitoring System for TracSeq 2.0 MLOps Pipeline

Real-time performance monitoring, alerting, and observability.
"""

import json
import uuid
import asyncio
from datetime import datetime, timedelta
from pathlib import Path
from typing import Dict, List, Optional, Any, Union, Callable
from dataclasses import dataclass, asdict, field
from enum import Enum
import structlog
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from sqlalchemy import create_engine, Column, String, DateTime, JSON, Float, Integer, Boolean, Text
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker
import aiofiles
import psutil
import asyncio
from collections import defaultdict, deque

logger = structlog.get_logger(__name__)

class AlertSeverity(Enum):
    INFO = "info"
    WARNING = "warning"
    CRITICAL = "critical"

class MetricType(Enum):
    ACCURACY = "accuracy"
    LATENCY = "latency"
    THROUGHPUT = "throughput"
    ERROR_RATE = "error_rate"
    MEMORY_USAGE = "memory_usage"
    CPU_USAGE = "cpu_usage"
    CUSTOM = "custom"

@dataclass
class MetricThreshold:
    """Configuration for metric thresholds and alerting"""
    metric_name: str
    metric_type: MetricType
    
    # Thresholds
    warning_threshold: Optional[float] = None
    critical_threshold: Optional[float] = None
    
    # Comparison operator ("gt", "lt", "eq")
    operator: str = "gt"
    
    # Window for evaluation
    evaluation_window_minutes: int = 5
    minimum_samples: int = 10
    
    # Alert settings
    alert_enabled: bool = True
    alert_channels: List[str] = field(default_factory=list)  # ["email", "slack", "webhook"]
    
    # Metadata
    description: str = ""
    created_at: datetime = field(default_factory=datetime.utcnow)

@dataclass
class MonitoringMetric:
    """A single monitoring metric data point"""
    metric_id: str
    model_id: str
    model_version: str
    metric_name: str
    metric_type: MetricType
    
    # Value and metadata
    value: float
    tags: Dict[str, str] = field(default_factory=dict)
    
    # Context
    request_id: Optional[str] = None
    user_id: Optional[str] = None
    
    # Timing
    timestamp: datetime = field(default_factory=datetime.utcnow)

@dataclass
class Alert:
    """Monitoring alert"""
    alert_id: str
    threshold_id: str
    model_id: str
    metric_name: str
    severity: AlertSeverity
    
    # Alert details
    message: str
    current_value: float
    threshold_value: float
    
    # Context
    triggered_by_values: List[float] = field(default_factory=list)
    evaluation_window: int = 5
    
    # Status
    is_resolved: bool = False
    acknowledged: bool = False
    acknowledged_by: Optional[str] = None
    acknowledged_at: Optional[datetime] = None
    
    # Timing
    created_at: datetime = field(default_factory=datetime.utcnow)
    resolved_at: Optional[datetime] = None

@dataclass
class ModelHealthStatus:
    """Overall health status of a model"""
    model_id: str
    model_version: str
    
    # Health indicators
    overall_health: str  # "healthy", "degraded", "unhealthy"
    health_score: float  # 0.0 to 1.0
    
    # Current metrics
    current_metrics: Dict[str, float] = field(default_factory=dict)
    
    # Active alerts
    active_alerts: List[str] = field(default_factory=list)  # Alert IDs
    
    # Recent performance
    predictions_last_hour: int = 0
    average_latency_ms: float = 0.0
    error_rate_last_hour: float = 0.0
    
    # System resources
    memory_usage_mb: float = 0.0
    cpu_usage_percent: float = 0.0
    
    # Timestamps
    last_prediction: Optional[datetime] = None
    last_updated: datetime = field(default_factory=datetime.utcnow)

Base = declarative_base()

class MetricThresholdRecord(Base):
    """Database model for metric thresholds"""
    __tablename__ = "metric_thresholds"
    
    threshold_id = Column(String, primary_key=True)
    model_id = Column(String, nullable=False)
    metric_name = Column(String, nullable=False)
    metric_type = Column(String, nullable=False)
    
    # Thresholds
    warning_threshold = Column(Float)
    critical_threshold = Column(Float)
    operator = Column(String, default="gt")
    
    # Window
    evaluation_window_minutes = Column(Integer, default=5)
    minimum_samples = Column(Integer, default=10)
    
    # Alerts
    alert_enabled = Column(Boolean, default=True)
    alert_channels = Column(JSON)
    
    # Metadata
    description = Column(Text)
    created_at = Column(DateTime, default=datetime.utcnow)

class MonitoringMetricRecord(Base):
    """Database model for monitoring metrics"""
    __tablename__ = "monitoring_metrics"
    
    metric_id = Column(String, primary_key=True)
    model_id = Column(String, nullable=False)
    model_version = Column(String, nullable=False)
    metric_name = Column(String, nullable=False)
    metric_type = Column(String, nullable=False)
    
    # Value and metadata
    value = Column(Float, nullable=False)
    tags = Column(JSON)
    
    # Context
    request_id = Column(String)
    user_id = Column(String)
    
    # Timing
    timestamp = Column(DateTime, default=datetime.utcnow)

class AlertRecord(Base):
    """Database model for alerts"""
    __tablename__ = "monitoring_alerts"
    
    alert_id = Column(String, primary_key=True)
    threshold_id = Column(String, nullable=False)
    model_id = Column(String, nullable=False)
    metric_name = Column(String, nullable=False)
    severity = Column(String, nullable=False)
    
    # Alert details
    message = Column(Text, nullable=False)
    current_value = Column(Float, nullable=False)
    threshold_value = Column(Float, nullable=False)
    
    # Context
    triggered_by_values = Column(JSON)
    evaluation_window = Column(Integer, default=5)
    
    # Status
    is_resolved = Column(Boolean, default=False)
    acknowledged = Column(Boolean, default=False)
    acknowledged_by = Column(String)
    acknowledged_at = Column(DateTime)
    
    # Timing
    created_at = Column(DateTime, default=datetime.utcnow)
    resolved_at = Column(DateTime)

class ModelMonitor:
    """
    Comprehensive model monitoring system.
    
    Features:
    - Real-time metric collection
    - Threshold-based alerting
    - Performance dashboards
    - Health status tracking
    - Resource monitoring
    - Custom metric support
    """
    
    def __init__(
        self,
        database_url: str,
        dashboard_dir: Union[str, Path],
        alert_handlers: Optional[Dict[str, Callable]] = None
    ):
        self.dashboard_dir = Path(dashboard_dir)
        self.dashboard_dir.mkdir(parents=True, exist_ok=True)
        
        # Database setup
        self.engine = create_engine(database_url)
        Base.metadata.create_all(self.engine)
        self.SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=self.engine)
        
        # Alert handlers
        self.alert_handlers = alert_handlers or {}
        
        # In-memory metric storage for fast access
        self.recent_metrics: Dict[str, deque] = defaultdict(lambda: deque(maxlen=1000))
        self.model_health_cache: Dict[str, ModelHealthStatus] = {}
        
        # Background tasks
        self._monitoring_tasks: List[asyncio.Task] = []
        
        # Metric collection interval
        self.collection_interval_seconds = 10
    
    async def start_monitoring(self):
        """Start background monitoring tasks."""
        # Start metric aggregation task
        task = asyncio.create_task(self._aggregate_metrics_loop())
        self._monitoring_tasks.append(task)
        
        # Start threshold checking task
        task = asyncio.create_task(self._check_thresholds_loop())
        self._monitoring_tasks.append(task)
        
        # Start health status updates
        task = asyncio.create_task(self._update_health_status_loop())
        self._monitoring_tasks.append(task)
        
        # Start system resource monitoring
        task = asyncio.create_task(self._monitor_system_resources_loop())
        self._monitoring_tasks.append(task)
        
        logger.info(
            "Model monitoring started",
            active_tasks=len(self._monitoring_tasks)
        )
    
    async def stop_monitoring(self):
        """Stop all monitoring tasks."""
        for task in self._monitoring_tasks:
            task.cancel()
        
        await asyncio.gather(*self._monitoring_tasks, return_exceptions=True)
        self._monitoring_tasks.clear()
        
        logger.info("Model monitoring stopped")
    
    async def record_metric(self, metric: MonitoringMetric):
        """Record a monitoring metric."""
        # Generate metric ID if not provided
        if not metric.metric_id:
            metric.metric_id = f"metric_{uuid.uuid4().hex[:8]}"
        
        # Store in database
        with self.SessionLocal() as session:
            record = MonitoringMetricRecord(
                metric_id=metric.metric_id,
                model_id=metric.model_id,
                model_version=metric.model_version,
                metric_name=metric.metric_name,
                metric_type=metric.metric_type.value,
                value=metric.value,
                tags=metric.tags,
                request_id=metric.request_id,
                user_id=metric.user_id,
                timestamp=metric.timestamp
            )
            session.add(record)
            session.commit()
        
        # Store in memory for fast access
        key = f"{metric.model_id}:{metric.metric_name}"
        self.recent_metrics[key].append((metric.timestamp, metric.value))
        
        # Update health cache
        await self._update_model_health_cache(metric.model_id, metric.model_version)
    
    async def record_prediction_metrics(
        self,
        model_id: str,
        model_version: str,
        prediction_latency_ms: float,
        prediction_success: bool,
        confidence_score: Optional[float] = None,
        accuracy: Optional[float] = None,
        request_id: Optional[str] = None,
        user_id: Optional[str] = None,
        tags: Optional[Dict[str, str]] = None
    ):
        """Record metrics for a single prediction."""
        timestamp = datetime.utcnow()
        base_tags = tags or {}
        
        # Record latency
        await self.record_metric(MonitoringMetric(
            metric_id="",
            model_id=model_id,
            model_version=model_version,
            metric_name="prediction_latency",
            metric_type=MetricType.LATENCY,
            value=prediction_latency_ms,
            tags=base_tags,
            request_id=request_id,
            user_id=user_id,
            timestamp=timestamp
        ))
        
        # Record success/error
        error_value = 0.0 if prediction_success else 1.0
        await self.record_metric(MonitoringMetric(
            metric_id="",
            model_id=model_id,
            model_version=model_version,
            metric_name="prediction_error",
            metric_type=MetricType.ERROR_RATE,
            value=error_value,
            tags=base_tags,
            request_id=request_id,
            user_id=user_id,
            timestamp=timestamp
        ))
        
        # Record confidence if provided
        if confidence_score is not None:
            await self.record_metric(MonitoringMetric(
                metric_id="",
                model_id=model_id,
                model_version=model_version,
                metric_name="confidence_score",
                metric_type=MetricType.CUSTOM,
                value=confidence_score,
                tags=base_tags,
                request_id=request_id,
                user_id=user_id,
                timestamp=timestamp
            ))
        
        # Record accuracy if provided
        if accuracy is not None:
            await self.record_metric(MonitoringMetric(
                metric_id="",
                model_id=model_id,
                model_version=model_version,
                metric_name="prediction_accuracy",
                metric_type=MetricType.ACCURACY,
                value=accuracy,
                tags=base_tags,
                request_id=request_id,
                user_id=user_id,
                timestamp=timestamp
            ))
    
    async def create_threshold(self, threshold: MetricThreshold) -> str:
        """Create a new metric threshold."""
        if not threshold.threshold_id:
            threshold.threshold_id = f"threshold_{uuid.uuid4().hex[:8]}"
        
        # Store in database
        with self.SessionLocal() as session:
            record = MetricThresholdRecord(
                threshold_id=threshold.threshold_id,
                model_id=threshold.model_id,
                metric_name=threshold.metric_name,
                metric_type=threshold.metric_type.value,
                warning_threshold=threshold.warning_threshold,
                critical_threshold=threshold.critical_threshold,
                operator=threshold.operator,
                evaluation_window_minutes=threshold.evaluation_window_minutes,
                minimum_samples=threshold.minimum_samples,
                alert_enabled=threshold.alert_enabled,
                alert_channels=threshold.alert_channels,
                description=threshold.description,
                created_at=threshold.created_at
            )
            session.add(record)
            session.commit()
        
        logger.info(
            "Metric threshold created",
            threshold_id=threshold.threshold_id,
            model_id=threshold.model_id,
            metric_name=threshold.metric_name
        )
        
        return threshold.threshold_id
    
    async def get_model_health(self, model_id: str) -> Optional[ModelHealthStatus]:
        """Get current health status of a model."""
        return self.model_health_cache.get(model_id)
    
    async def get_metrics(
        self,
        model_id: str,
        metric_name: Optional[str] = None,
        start_time: Optional[datetime] = None,
        end_time: Optional[datetime] = None,
        limit: int = 1000
    ) -> pd.DataFrame:
        """Get metrics for a model."""
        with self.SessionLocal() as session:
            query = session.query(MonitoringMetricRecord).filter(
                MonitoringMetricRecord.model_id == model_id
            )
            
            if metric_name:
                query = query.filter(MonitoringMetricRecord.metric_name == metric_name)
            
            if start_time:
                query = query.filter(MonitoringMetricRecord.timestamp >= start_time)
            
            if end_time:
                query = query.filter(MonitoringMetricRecord.timestamp <= end_time)
            
            query = query.order_by(MonitoringMetricRecord.timestamp.desc()).limit(limit)
            records = query.all()
            
            if not records:
                return pd.DataFrame()
            
            # Convert to DataFrame
            data = []
            for record in records:
                data.append({
                    'metric_id': record.metric_id,
                    'model_id': record.model_id,
                    'model_version': record.model_version,
                    'metric_name': record.metric_name,
                    'metric_type': record.metric_type,
                    'value': record.value,
                    'timestamp': record.timestamp,
                    'request_id': record.request_id,
                    'user_id': record.user_id,
                    **record.tags if record.tags else {}
                })
            
            return pd.DataFrame(data)
    
    async def get_active_alerts(
        self,
        model_id: Optional[str] = None,
        severity: Optional[AlertSeverity] = None
    ) -> List[Alert]:
        """Get active alerts."""
        with self.SessionLocal() as session:
            query = session.query(AlertRecord).filter(
                AlertRecord.is_resolved == False
            )
            
            if model_id:
                query = query.filter(AlertRecord.model_id == model_id)
            
            if severity:
                query = query.filter(AlertRecord.severity == severity.value)
            
            query = query.order_by(AlertRecord.created_at.desc())
            records = query.all()
            
            alerts = []
            for record in records:
                alert = Alert(
                    alert_id=record.alert_id,
                    threshold_id=record.threshold_id,
                    model_id=record.model_id,
                    metric_name=record.metric_name,
                    severity=AlertSeverity(record.severity),
                    message=record.message,
                    current_value=record.current_value,
                    threshold_value=record.threshold_value,
                    triggered_by_values=record.triggered_by_values or [],
                    evaluation_window=record.evaluation_window,
                    is_resolved=record.is_resolved,
                    acknowledged=record.acknowledged,
                    acknowledged_by=record.acknowledged_by,
                    acknowledged_at=record.acknowledged_at,
                    created_at=record.created_at,
                    resolved_at=record.resolved_at
                )
                alerts.append(alert)
            
            return alerts
    
    async def acknowledge_alert(self, alert_id: str, acknowledged_by: str) -> bool:
        """Acknowledge an alert."""
        with self.SessionLocal() as session:
            record = session.query(AlertRecord).filter(
                AlertRecord.alert_id == alert_id
            ).first()
            
            if record:
                record.acknowledged = True
                record.acknowledged_by = acknowledged_by
                record.acknowledged_at = datetime.utcnow()
                session.commit()
                
                logger.info(
                    "Alert acknowledged",
                    alert_id=alert_id,
                    acknowledged_by=acknowledged_by
                )
                return True
            
            return False
    
    async def resolve_alert(self, alert_id: str) -> bool:
        """Resolve an alert."""
        with self.SessionLocal() as session:
            record = session.query(AlertRecord).filter(
                AlertRecord.alert_id == alert_id
            ).first()
            
            if record:
                record.is_resolved = True
                record.resolved_at = datetime.utcnow()
                session.commit()
                
                logger.info("Alert resolved", alert_id=alert_id)
                return True
            
            return False
    
    async def generate_dashboard(self, model_id: str, hours: int = 24) -> str:
        """Generate monitoring dashboard for a model."""
        end_time = datetime.utcnow()
        start_time = end_time - timedelta(hours=hours)
        
        # Get metrics
        metrics_df = await self.get_metrics(model_id, start_time=start_time, end_time=end_time)
        
        if metrics_df.empty:
            logger.warning("No metrics found for dashboard", model_id=model_id)
            return ""
        
        # Create dashboard
        fig, axes = plt.subplots(2, 2, figsize=(15, 10))
        fig.suptitle(f'Model Monitoring Dashboard - {model_id}', fontsize=16)
        
        # Plot latency over time
        latency_data = metrics_df[metrics_df['metric_name'] == 'prediction_latency']
        if not latency_data.empty:
            axes[0, 0].plot(latency_data['timestamp'], latency_data['value'], alpha=0.7)
            axes[0, 0].set_title('Prediction Latency (ms)')
            axes[0, 0].set_xlabel('Time')
            axes[0, 0].set_ylabel('Latency (ms)')
            axes[0, 0].grid(True, alpha=0.3)
        
        # Plot error rate over time
        error_data = metrics_df[metrics_df['metric_name'] == 'prediction_error']
        if not error_data.empty:
            # Calculate rolling error rate
            error_data = error_data.set_index('timestamp').sort_index()
            rolling_error_rate = error_data['value'].rolling('1H').mean()
            
            axes[0, 1].plot(rolling_error_rate.index, rolling_error_rate.values, alpha=0.7)
            axes[0, 1].set_title('Error Rate (1-hour rolling)')
            axes[0, 1].set_xlabel('Time')
            axes[0, 1].set_ylabel('Error Rate')
            axes[0, 1].grid(True, alpha=0.3)
        
        # Plot accuracy distribution
        accuracy_data = metrics_df[metrics_df['metric_name'] == 'prediction_accuracy']
        if not accuracy_data.empty:
            axes[1, 0].hist(accuracy_data['value'], bins=30, alpha=0.7, edgecolor='black')
            axes[1, 0].set_title('Accuracy Distribution')
            axes[1, 0].set_xlabel('Accuracy')
            axes[1, 0].set_ylabel('Frequency')
            axes[1, 0].grid(True, alpha=0.3)
        
        # Plot confidence score over time
        confidence_data = metrics_df[metrics_df['metric_name'] == 'confidence_score']
        if not confidence_data.empty:
            axes[1, 1].plot(confidence_data['timestamp'], confidence_data['value'], alpha=0.7)
            axes[1, 1].set_title('Confidence Score')
            axes[1, 1].set_xlabel('Time')
            axes[1, 1].set_ylabel('Confidence')
            axes[1, 1].grid(True, alpha=0.3)
        
        # Hide unused subplots
        for i, ax in enumerate(axes.flat):
            if i >= 4:
                ax.set_visible(False)
        
        plt.tight_layout()
        
        # Save dashboard
        dashboard_path = self.dashboard_dir / f"{model_id}_dashboard_{int(datetime.utcnow().timestamp())}.png"
        plt.savefig(dashboard_path, dpi=300, bbox_inches='tight')
        plt.close()
        
        logger.info(
            "Dashboard generated",
            model_id=model_id,
            dashboard_path=str(dashboard_path)
        )
        
        return str(dashboard_path)
    
    async def _aggregate_metrics_loop(self):
        """Background task for metric aggregation."""
        while True:
            try:
                await self._aggregate_recent_metrics()
                await asyncio.sleep(self.collection_interval_seconds)
            except Exception as e:
                logger.error("Error in metric aggregation", error=str(e))
                await asyncio.sleep(30)
    
    async def _check_thresholds_loop(self):
        """Background task for threshold checking."""
        while True:
            try:
                await self._check_all_thresholds()
                await asyncio.sleep(60)  # Check every minute
            except Exception as e:
                logger.error("Error in threshold checking", error=str(e))
                await asyncio.sleep(30)
    
    async def _update_health_status_loop(self):
        """Background task for updating health status."""
        while True:
            try:
                await self._update_all_health_status()
                await asyncio.sleep(30)  # Update every 30 seconds
            except Exception as e:
                logger.error("Error in health status update", error=str(e))
                await asyncio.sleep(30)
    
    async def _monitor_system_resources_loop(self):
        """Background task for system resource monitoring."""
        while True:
            try:
                await self._record_system_metrics()
                await asyncio.sleep(self.collection_interval_seconds)
            except Exception as e:
                logger.error("Error in system monitoring", error=str(e))
                await asyncio.sleep(30)
    
    async def _aggregate_recent_metrics(self):
        """Aggregate recent metrics for fast access."""
        # This is a placeholder for metric aggregation logic
        # In a production system, you might aggregate metrics into time buckets
        pass
    
    async def _check_all_thresholds(self):
        """Check all configured thresholds."""
        with self.SessionLocal() as session:
            thresholds = session.query(MetricThresholdRecord).filter(
                MetricThresholdRecord.alert_enabled == True
            ).all()
            
            for threshold in thresholds:
                await self._check_threshold(threshold)
    
    async def _check_threshold(self, threshold: MetricThresholdRecord):
        """Check a single threshold."""
        # Get recent metrics
        end_time = datetime.utcnow()
        start_time = end_time - timedelta(minutes=threshold.evaluation_window_minutes)
        
        metrics_df = await self.get_metrics(
            model_id=threshold.model_id,
            metric_name=threshold.metric_name,
            start_time=start_time,
            end_time=end_time
        )
        
        if len(metrics_df) < threshold.minimum_samples:
            return
        
        # Calculate current value (average over window)
        current_value = metrics_df['value'].mean()
        triggered_values = metrics_df['value'].tolist()
        
        # Check thresholds
        alert_severity = None
        threshold_value = None
        
        if threshold.critical_threshold is not None:
            if self._threshold_exceeded(current_value, threshold.critical_threshold, threshold.operator):
                alert_severity = AlertSeverity.CRITICAL
                threshold_value = threshold.critical_threshold
        
        if alert_severity is None and threshold.warning_threshold is not None:
            if self._threshold_exceeded(current_value, threshold.warning_threshold, threshold.operator):
                alert_severity = AlertSeverity.WARNING
                threshold_value = threshold.warning_threshold
        
        if alert_severity:
            # Check if alert already exists
            existing_alert = await self._get_existing_alert(threshold.threshold_id)
            
            if not existing_alert:
                # Create new alert
                await self._create_alert(
                    threshold,
                    alert_severity,
                    current_value,
                    threshold_value,
                    triggered_values
                )
        else:
            # Resolve existing alerts if metric is back to normal
            await self._auto_resolve_alerts(threshold.threshold_id)
    
    def _threshold_exceeded(self, current_value: float, threshold: float, operator: str) -> bool:
        """Check if threshold is exceeded."""
        if operator == "gt":
            return current_value > threshold
        elif operator == "lt":
            return current_value < threshold
        elif operator == "eq":
            return abs(current_value - threshold) < 0.001  # Small epsilon for float comparison
        else:
            return False
    
    async def _get_existing_alert(self, threshold_id: str) -> Optional[AlertRecord]:
        """Get existing unresolved alert for threshold."""
        with self.SessionLocal() as session:
            return session.query(AlertRecord).filter(
                AlertRecord.threshold_id == threshold_id,
                AlertRecord.is_resolved == False
            ).first()
    
    async def _create_alert(
        self,
        threshold: MetricThresholdRecord,
        severity: AlertSeverity,
        current_value: float,
        threshold_value: float,
        triggered_values: List[float]
    ):
        """Create a new alert."""
        alert_id = f"alert_{uuid.uuid4().hex[:8]}"
        
        message = (
            f"Model {threshold.model_id} metric '{threshold.metric_name}' "
            f"has exceeded {severity.value} threshold. "
            f"Current value: {current_value:.4f}, "
            f"Threshold: {threshold_value:.4f}"
        )
        
        alert = Alert(
            alert_id=alert_id,
            threshold_id=threshold.threshold_id,
            model_id=threshold.model_id,
            metric_name=threshold.metric_name,
            severity=severity,
            message=message,
            current_value=current_value,
            threshold_value=threshold_value,
            triggered_by_values=triggered_values,
            evaluation_window=threshold.evaluation_window_minutes
        )
        
        # Store in database
        with self.SessionLocal() as session:
            record = AlertRecord(
                alert_id=alert.alert_id,
                threshold_id=alert.threshold_id,
                model_id=alert.model_id,
                metric_name=alert.metric_name,
                severity=alert.severity.value,
                message=alert.message,
                current_value=alert.current_value,
                threshold_value=alert.threshold_value,
                triggered_by_values=alert.triggered_by_values,
                evaluation_window=alert.evaluation_window,
                created_at=alert.created_at
            )
            session.add(record)
            session.commit()
        
        # Send alert notifications
        await self._send_alert_notifications(alert, threshold.alert_channels)
        
        logger.warning(
            "Alert created",
            alert_id=alert_id,
            model_id=threshold.model_id,
            metric_name=threshold.metric_name,
            severity=severity.value,
            current_value=current_value,
            threshold_value=threshold_value
        )
    
    async def _auto_resolve_alerts(self, threshold_id: str):
        """Auto-resolve alerts when metric returns to normal."""
        with self.SessionLocal() as session:
            alerts = session.query(AlertRecord).filter(
                AlertRecord.threshold_id == threshold_id,
                AlertRecord.is_resolved == False
            ).all()
            
            for alert in alerts:
                alert.is_resolved = True
                alert.resolved_at = datetime.utcnow()
            
            session.commit()
            
            if alerts:
                logger.info(
                    "Auto-resolved alerts",
                    threshold_id=threshold_id,
                    count=len(alerts)
                )
    
    async def _send_alert_notifications(self, alert: Alert, channels: List[str]):
        """Send alert notifications to configured channels."""
        for channel in channels:
            if channel in self.alert_handlers:
                try:
                    await self.alert_handlers[channel](alert)
                except Exception as e:
                    logger.error(
                        "Failed to send alert notification",
                        channel=channel,
                        alert_id=alert.alert_id,
                        error=str(e)
                    )
    
    async def _update_all_health_status(self):
        """Update health status for all monitored models."""
        # Get all unique model IDs
        with self.SessionLocal() as session:
            model_ids = session.query(MonitoringMetricRecord.model_id).distinct().all()
            
            for (model_id,) in model_ids:
                await self._update_model_health_cache(model_id, "latest")
    
    async def _update_model_health_cache(self, model_id: str, model_version: str):
        """Update health status cache for a model."""
        # Get recent metrics
        end_time = datetime.utcnow()
        start_time = end_time - timedelta(hours=1)
        
        metrics_df = await self.get_metrics(model_id, start_time=start_time, end_time=end_time)
        
        # Get active alerts
        active_alerts = await self.get_active_alerts(model_id=model_id)
        
        # Calculate health indicators
        health_score = 1.0
        overall_health = "healthy"
        current_metrics = {}
        
        if not metrics_df.empty:
            # Calculate current metric values
            for metric_name in metrics_df['metric_name'].unique():
                metric_data = metrics_df[metrics_df['metric_name'] == metric_name]
                current_metrics[metric_name] = metric_data['value'].mean()
            
            # Adjust health score based on active alerts
            critical_alerts = [a for a in active_alerts if a.severity == AlertSeverity.CRITICAL]
            warning_alerts = [a for a in active_alerts if a.severity == AlertSeverity.WARNING]
            
            health_score -= len(critical_alerts) * 0.3
            health_score -= len(warning_alerts) * 0.1
            health_score = max(0.0, health_score)
            
            if health_score < 0.3:
                overall_health = "unhealthy"
            elif health_score < 0.7:
                overall_health = "degraded"
        
        # Performance metrics
        predictions_last_hour = len(metrics_df[metrics_df['metric_name'] == 'prediction_latency'])
        
        latency_data = metrics_df[metrics_df['metric_name'] == 'prediction_latency']
        average_latency = latency_data['value'].mean() if not latency_data.empty else 0.0
        
        error_data = metrics_df[metrics_df['metric_name'] == 'prediction_error']
        error_rate = error_data['value'].mean() if not error_data.empty else 0.0
        
        # Update cache
        health_status = ModelHealthStatus(
            model_id=model_id,
            model_version=model_version,
            overall_health=overall_health,
            health_score=health_score,
            current_metrics=current_metrics,
            active_alerts=[a.alert_id for a in active_alerts],
            predictions_last_hour=predictions_last_hour,
            average_latency_ms=average_latency,
            error_rate_last_hour=error_rate,
            last_prediction=metrics_df['timestamp'].max() if not metrics_df.empty else None
        )
        
        self.model_health_cache[model_id] = health_status
    
    async def _record_system_metrics(self):
        """Record system resource metrics."""
        try:
            # CPU usage
            cpu_percent = psutil.cpu_percent(interval=1)
            
            # Memory usage
            memory = psutil.virtual_memory()
            memory_used_mb = memory.used / (1024 * 1024)
            memory_percent = memory.percent
            
            # For each model in cache, record system metrics
            for model_id, health_status in self.model_health_cache.items():
                # Record CPU usage
                await self.record_metric(MonitoringMetric(
                    metric_id="",
                    model_id=model_id,
                    model_version=health_status.model_version,
                    metric_name="cpu_usage_percent",
                    metric_type=MetricType.CPU_USAGE,
                    value=cpu_percent,
                    tags={"resource_type": "system"}
                ))
                
                # Record memory usage
                await self.record_metric(MonitoringMetric(
                    metric_id="",
                    model_id=model_id,
                    model_version=health_status.model_version,
                    metric_name="memory_usage_mb",
                    metric_type=MetricType.MEMORY_USAGE,
                    value=memory_used_mb,
                    tags={"resource_type": "system"}
                ))
                
                # Update health status cache
                health_status.memory_usage_mb = memory_used_mb
                health_status.cpu_usage_percent = cpu_percent
        
        except Exception as e:
            logger.error("Failed to record system metrics", error=str(e)) 
