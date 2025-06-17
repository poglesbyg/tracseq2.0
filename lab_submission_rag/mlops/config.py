"""
Configuration Management for TracSeq 2.0 MLOps Pipeline

Centralized configuration for all MLOps components.
"""

import os
import json
from pathlib import Path
from typing import Dict, Any, Optional
from dataclasses import dataclass, asdict
from enum import Enum

class Environment(Enum):
    DEVELOPMENT = "development"
    STAGING = "staging"
    PRODUCTION = "production"

@dataclass
class DatabaseConfig:
    """Database configuration"""
    url: str
    max_connections: int = 20
    connection_timeout: int = 30
    pool_size: int = 10

@dataclass
class ModelRegistryConfig:
    """Model registry configuration"""
    registry_path: str
    max_model_versions: int = 10
    automatic_cleanup: bool = True
    backup_enabled: bool = True
    backup_interval_hours: int = 24

@dataclass
class ExperimentTrackingConfig:
    """Experiment tracking configuration"""
    tracking_dir: str
    max_experiments: int = 1000
    artifact_retention_days: int = 90
    metrics_batch_size: int = 100

@dataclass
class ABTestingConfig:
    """A/B testing configuration"""
    results_dir: str
    default_confidence_level: float = 0.95
    default_statistical_power: float = 0.8
    max_concurrent_tests: int = 10
    automatic_stopping: bool = True

@dataclass
class ContinuousLearningConfig:
    """Continuous learning configuration"""
    data_dir: str
    monitoring_interval_minutes: int = 60
    max_training_time_hours: int = 4
    quality_threshold: float = 0.8
    improvement_threshold: float = 0.01

@dataclass
class MonitoringConfig:
    """Model monitoring configuration"""
    dashboard_dir: str
    metrics_retention_days: int = 30
    health_check_interval_seconds: int = 30
    alert_cooldown_minutes: int = 15
    default_error_threshold: float = 0.05

@dataclass
class DeploymentConfig:
    """Deployment configuration"""
    container_registry_url: str
    kubernetes_config_path: Optional[str] = None
    default_health_timeout: int = 300
    default_replicas: int = 2
    rollback_enabled: bool = True

@dataclass
class DataPipelineConfig:
    """Data pipeline configuration"""
    data_dir: str
    batch_size: int = 1000
    validation_enabled: bool = True
    quality_threshold: float = 0.7
    feature_store_enabled: bool = False

@dataclass
class SecurityConfig:
    """Security configuration"""
    api_key_required: bool = True
    jwt_secret_key: str = ""
    session_timeout_minutes: int = 60
    encryption_enabled: bool = True
    audit_logging: bool = True

@dataclass
class MLOpsConfig:
    """Complete MLOps configuration"""
    environment: Environment
    database: DatabaseConfig
    model_registry: ModelRegistryConfig
    experiment_tracking: ExperimentTrackingConfig
    ab_testing: ABTestingConfig
    continuous_learning: ContinuousLearningConfig
    monitoring: MonitoringConfig
    deployment: DeploymentConfig
    data_pipeline: DataPipelineConfig
    security: SecurityConfig

class ConfigManager:
    """
    Configuration manager for MLOps pipeline.
    
    Handles configuration loading, validation, and environment-specific settings.
    """
    
    def __init__(self, config_path: Optional[str] = None):
        self.config_path = Path(config_path) if config_path else Path("mlops_config.json")
        self._config: Optional[MLOpsConfig] = None
    
    def load_config(self, environment: Environment = Environment.DEVELOPMENT) -> MLOpsConfig:
        """Load configuration for specified environment."""
        if self.config_path.exists():
            config_data = self._load_from_file()
        else:
            config_data = self._get_default_config()
        
        # Override with environment-specific settings
        env_config = config_data.get(environment.value, {})
        base_config = config_data.get("default", {})
        
        # Merge configurations
        merged_config = self._merge_configs(base_config, env_config)
        
        # Override with environment variables
        merged_config = self._override_with_env_vars(merged_config)
        
        # Create configuration object
        self._config = self._create_config_object(merged_config, environment)
        
        return self._config
    
    def save_config(self, config: MLOpsConfig):
        """Save configuration to file."""
        config_data = {
            config.environment.value: asdict(config)
        }
        
        # Load existing config if exists
        if self.config_path.exists():
            existing_data = self._load_from_file()
            existing_data.update(config_data)
            config_data = existing_data
        
        with open(self.config_path, 'w') as f:
            json.dump(config_data, f, indent=2, default=str)
    
    def get_config(self) -> Optional[MLOpsConfig]:
        """Get current configuration."""
        return self._config
    
    def validate_config(self, config: MLOpsConfig) -> bool:
        """Validate configuration settings."""
        try:
            # Validate database URL
            if not config.database.url:
                raise ValueError("Database URL is required")
            
            # Validate directories exist or can be created
            directories = [
                config.model_registry.registry_path,
                config.experiment_tracking.tracking_dir,
                config.ab_testing.results_dir,
                config.continuous_learning.data_dir,
                config.monitoring.dashboard_dir,
                config.data_pipeline.data_dir
            ]
            
            for directory in directories:
                path = Path(directory)
                if not path.exists():
                    path.mkdir(parents=True, exist_ok=True)
            
            # Validate thresholds
            if not (0.0 <= config.continuous_learning.quality_threshold <= 1.0):
                raise ValueError("Quality threshold must be between 0 and 1")
            
            if not (0.0 <= config.monitoring.default_error_threshold <= 1.0):
                raise ValueError("Error threshold must be between 0 and 1")
            
            return True
            
        except Exception as e:
            print(f"Configuration validation failed: {e}")
            return False
    
    def _load_from_file(self) -> Dict[str, Any]:
        """Load configuration from JSON file."""
        with open(self.config_path, 'r') as f:
            return json.load(f)
    
    def _get_default_config(self) -> Dict[str, Any]:
        """Get default configuration."""
        return {
            "default": {
                "database": {
                    "url": "sqlite:///mlops.db",
                    "max_connections": 20,
                    "connection_timeout": 30,
                    "pool_size": 10
                },
                "model_registry": {
                    "registry_path": "./mlops_data/model_registry",
                    "max_model_versions": 10,
                    "automatic_cleanup": True,
                    "backup_enabled": True,
                    "backup_interval_hours": 24
                },
                "experiment_tracking": {
                    "tracking_dir": "./mlops_data/experiments",
                    "max_experiments": 1000,
                    "artifact_retention_days": 90,
                    "metrics_batch_size": 100
                },
                "ab_testing": {
                    "results_dir": "./mlops_data/ab_tests",
                    "default_confidence_level": 0.95,
                    "default_statistical_power": 0.8,
                    "max_concurrent_tests": 10,
                    "automatic_stopping": True
                },
                "continuous_learning": {
                    "data_dir": "./mlops_data/continuous_learning",
                    "monitoring_interval_minutes": 60,
                    "max_training_time_hours": 4,
                    "quality_threshold": 0.8,
                    "improvement_threshold": 0.01
                },
                "monitoring": {
                    "dashboard_dir": "./mlops_data/dashboards",
                    "metrics_retention_days": 30,
                    "health_check_interval_seconds": 30,
                    "alert_cooldown_minutes": 15,
                    "default_error_threshold": 0.05
                },
                "deployment": {
                    "container_registry_url": "localhost:5000",
                    "kubernetes_config_path": None,
                    "default_health_timeout": 300,
                    "default_replicas": 2,
                    "rollback_enabled": True
                },
                "data_pipeline": {
                    "data_dir": "./mlops_data/data_pipeline",
                    "batch_size": 1000,
                    "validation_enabled": True,
                    "quality_threshold": 0.7,
                    "feature_store_enabled": False
                },
                "security": {
                    "api_key_required": True,
                    "jwt_secret_key": "your-secret-key-here",
                    "session_timeout_minutes": 60,
                    "encryption_enabled": True,
                    "audit_logging": True
                }
            },
            "development": {
                "database": {
                    "url": "sqlite:///mlops_dev.db"
                },
                "monitoring": {
                    "health_check_interval_seconds": 60
                },
                "security": {
                    "api_key_required": False,
                    "encryption_enabled": False
                }
            },
            "staging": {
                "database": {
                    "url": "postgresql://user:pass@localhost:5432/mlops_staging"
                },
                "deployment": {
                    "container_registry_url": "registry.staging.company.com"
                }
            },
            "production": {
                "database": {
                    "url": "postgresql://user:pass@prod-db:5432/mlops_production",
                    "max_connections": 50,
                    "pool_size": 20
                },
                "model_registry": {
                    "backup_enabled": True,
                    "backup_interval_hours": 6
                },
                "monitoring": {
                    "health_check_interval_seconds": 15,
                    "alert_cooldown_minutes": 5
                },
                "deployment": {
                    "container_registry_url": "registry.company.com",
                    "default_replicas": 3
                },
                "security": {
                    "audit_logging": True,
                    "session_timeout_minutes": 30
                }
            }
        }
    
    def _merge_configs(self, base: Dict[str, Any], override: Dict[str, Any]) -> Dict[str, Any]:
        """Merge two configuration dictionaries."""
        result = base.copy()
        
        for key, value in override.items():
            if key in result and isinstance(result[key], dict) and isinstance(value, dict):
                result[key] = self._merge_configs(result[key], value)
            else:
                result[key] = value
        
        return result
    
    def _override_with_env_vars(self, config: Dict[str, Any]) -> Dict[str, Any]:
        """Override configuration with environment variables."""
        env_mappings = {
            "MLOPS_DATABASE_URL": ["database", "url"],
            "MLOPS_REGISTRY_PATH": ["model_registry", "registry_path"],
            "MLOPS_TRACKING_DIR": ["experiment_tracking", "tracking_dir"],
            "MLOPS_MONITORING_DIR": ["monitoring", "dashboard_dir"],
            "MLOPS_DATA_DIR": ["data_pipeline", "data_dir"],
            "MLOPS_CONTAINER_REGISTRY": ["deployment", "container_registry_url"],
            "MLOPS_JWT_SECRET": ["security", "jwt_secret_key"],
            "MLOPS_HEALTH_CHECK_INTERVAL": ["monitoring", "health_check_interval_seconds"],
            "MLOPS_MAX_CONNECTIONS": ["database", "max_connections"],
            "MLOPS_ERROR_THRESHOLD": ["monitoring", "default_error_threshold"],
            "MLOPS_QUALITY_THRESHOLD": ["continuous_learning", "quality_threshold"]
        }
        
        for env_var, config_path in env_mappings.items():
            env_value = os.getenv(env_var)
            if env_value:
                # Navigate to the nested config location
                current = config
                for key in config_path[:-1]:
                    if key not in current:
                        current[key] = {}
                    current = current[key]
                
                # Convert value to appropriate type
                final_key = config_path[-1]
                if final_key.endswith(("_seconds", "_minutes", "_hours", "_days", "connections", "size")):
                    current[final_key] = int(env_value)
                elif final_key.endswith(("threshold", "level", "power")):
                    current[final_key] = float(env_value)
                elif env_value.lower() in ("true", "false"):
                    current[final_key] = env_value.lower() == "true"
                else:
                    current[final_key] = env_value
        
        return config
    
    def _create_config_object(self, config_data: Dict[str, Any], environment: Environment) -> MLOpsConfig:
        """Create MLOpsConfig object from dictionary."""
        return MLOpsConfig(
            environment=environment,
            database=DatabaseConfig(**config_data["database"]),
            model_registry=ModelRegistryConfig(**config_data["model_registry"]),
            experiment_tracking=ExperimentTrackingConfig(**config_data["experiment_tracking"]),
            ab_testing=ABTestingConfig(**config_data["ab_testing"]),
            continuous_learning=ContinuousLearningConfig(**config_data["continuous_learning"]),
            monitoring=MonitoringConfig(**config_data["monitoring"]),
            deployment=DeploymentConfig(**config_data["deployment"]),
            data_pipeline=DataPipelineConfig(**config_data["data_pipeline"]),
            security=SecurityConfig(**config_data["security"])
        )

# Global configuration instance
_config_manager = ConfigManager()

def get_config(environment: Environment = Environment.DEVELOPMENT) -> MLOpsConfig:
    """Get MLOps configuration for specified environment."""
    return _config_manager.load_config(environment)

def save_config(config: MLOpsConfig):
    """Save MLOps configuration."""
    _config_manager.save_config(config)

def validate_config(config: MLOpsConfig) -> bool:
    """Validate MLOps configuration."""
    return _config_manager.validate_config(config)

# Example usage
if __name__ == "__main__":
    # Load development configuration
    dev_config = get_config(Environment.DEVELOPMENT)
    print(f"Development config loaded: {dev_config.environment.value}")
    
    # Load production configuration
    prod_config = get_config(Environment.PRODUCTION)
    print(f"Production config loaded: {prod_config.environment.value}")
    
    # Validate configuration
    is_valid = validate_config(dev_config)
    print(f"Configuration valid: {is_valid}")
    
    # Save configuration
    save_config(dev_config)
    print("Configuration saved") 
