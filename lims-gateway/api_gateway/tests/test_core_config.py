"""
Tests for the core configuration module.

This module tests the hierarchical configuration system, environment variable
handling, and validation logic.
"""

import os
import pytest
from unittest.mock import patch
from pydantic import ValidationError

from api_gateway.core.config import (
    AppConfig,
    DatabaseConfig,
    SecurityConfig,
    GatewayConfig,
    ServiceConfig,
    LoggingConfig,
    MonitoringConfig,
    CORSConfig,
    ChatConfig,
    get_config,
    reload_config,
    load_config_from_env
)


class TestDatabaseConfig:
    """Test database configuration."""
    
    def test_default_values(self):
        """Test default database configuration values."""
        config = DatabaseConfig()
        assert config.url == "postgres://postgres:postgres@localhost:5432/lims_db"
        assert config.pool_min_size == 2
        assert config.pool_max_size == 10
        assert config.connection_timeout == 30
        assert config.pool_recycle == 3600
        assert config.echo == False
    
    def test_custom_values(self):
        """Test custom database configuration values."""
        config = DatabaseConfig(
            url="postgres://user:pass@host:5432/db",
            pool_min_size=5,
            pool_max_size=20,
            connection_timeout=60,
            pool_recycle=7200,
            echo=True
        )
        assert config.url == "postgres://user:pass@host:5432/db"
        assert config.pool_min_size == 5
        assert config.pool_max_size == 20
        assert config.connection_timeout == 60
        assert config.pool_recycle == 7200
        assert config.echo == True


class TestSecurityConfig:
    """Test security configuration."""
    
    def test_default_values(self):
        """Test default security configuration values."""
        config = SecurityConfig()
        assert config.jwt_secret_key == "dev-secret-key-change-in-production"
        assert config.jwt_algorithm == "HS256"
        assert config.jwt_expiration_hours == 24
        assert config.rate_limit_requests == 100
        assert config.rate_limit_window == 60
        assert config.adaptive_rate_limiting == True
        assert config.enable_csrf_protection == True
    
    def test_jwt_secret_validation(self):
        """Test JWT secret key validation."""
        # Valid secret key (32+ characters)
        config = SecurityConfig(jwt_secret_key="this-is-a-valid-32-character-secret-key")
        assert config.jwt_secret_key == "this-is-a-valid-32-character-secret-key"
        
        # Invalid secret key (too short)
        with pytest.raises(ValidationError) as exc_info:
            SecurityConfig(jwt_secret_key="short")
        assert "JWT secret key must be at least 32 characters long" in str(exc_info.value)


class TestGatewayConfig:
    """Test gateway configuration."""
    
    def test_default_values(self):
        """Test default gateway configuration values."""
        config = GatewayConfig()
        assert config.host == "0.0.0.0"
        assert config.port == 8000
        assert config.debug == False
        assert config.workers == 1
        assert config.max_request_size == 16 * 1024 * 1024
        assert config.request_timeout == 30
    
    def test_custom_values(self):
        """Test custom gateway configuration values."""
        config = GatewayConfig(
            host="127.0.0.1",
            port=8080,
            debug=True,
            workers=4,
            max_request_size=32 * 1024 * 1024,
            request_timeout=60
        )
        assert config.host == "127.0.0.1"
        assert config.port == 8080
        assert config.debug == True
        assert config.workers == 4
        assert config.max_request_size == 32 * 1024 * 1024
        assert config.request_timeout == 60


class TestServiceConfig:
    """Test service configuration."""
    
    def test_default_values(self):
        """Test default service configuration values."""
        config = ServiceConfig()
        assert config.auth_service_url == "http://lims-auth:8000"
        assert config.sample_service_url == "http://sample-service:8081"
        assert config.storage_service_url == "http://storage-service:8082"
        assert config.template_service_url == "http://lims-templates:8000"
        assert config.sequencing_service_url == "http://sequencing-service:8084"
        assert config.rag_service_url == "http://rag-service:8000"
        assert config.notification_service_url == "http://notification-service:8085"
        assert config.service_timeout == 30
        assert config.service_retries == 3
        assert config.service_retry_delay == 1.0


class TestLoggingConfig:
    """Test logging configuration."""
    
    def test_default_values(self):
        """Test default logging configuration values."""
        config = LoggingConfig()
        assert config.log_level == "INFO"
        assert config.log_file == None
        assert config.enable_access_log == True
        assert config.enable_sql_logging == False
        assert config.log_format == "json"
        assert config.log_rotation == True
        assert config.log_max_size == "100MB"
        assert config.log_backup_count == 5
    
    def test_log_level_validation(self):
        """Test log level validation."""
        # Valid log levels
        for level in ['DEBUG', 'INFO', 'WARNING', 'ERROR', 'CRITICAL']:
            config = LoggingConfig(log_level=level)
            assert config.log_level == level
        
        # Invalid log level
        with pytest.raises(ValidationError) as exc_info:
            LoggingConfig(log_level="INVALID")
        assert "Log level must be one of" in str(exc_info.value)


class TestMonitoringConfig:
    """Test monitoring configuration."""
    
    def test_default_values(self):
        """Test default monitoring configuration values."""
        config = MonitoringConfig()
        assert config.enable_metrics == True
        assert config.metrics_port == 9090
        assert config.health_check_interval == 30
        assert config.circuit_breaker_failure_threshold == 5
        assert config.circuit_breaker_recovery_timeout == 60
        assert config.circuit_breaker_half_open_max_calls == 3
        assert config.enable_tracing == False
        assert config.tracing_sample_rate == 0.1


class TestCORSConfig:
    """Test CORS configuration."""
    
    def test_default_values(self):
        """Test default CORS configuration values."""
        config = CORSConfig()
        assert config.allow_origins == ["http://localhost:3000", "http://localhost:8080"]
        assert config.allow_credentials == True
        assert config.allow_methods == ["*"]
        assert config.allow_headers == ["*"]
        assert config.max_age == 600


class TestChatConfig:
    """Test chat configuration."""
    
    def test_default_values(self):
        """Test default chat configuration values."""
        config = ChatConfig()
        assert config.enable_chat == True
        assert config.chat_model == "gpt-3.5-turbo"
        assert config.chat_max_tokens == 2000
        assert config.chat_temperature == 0.7
        assert config.chat_timeout == 30


class TestAppConfig:
    """Test main application configuration."""
    
    def test_default_values(self):
        """Test default application configuration values."""
        config = AppConfig()
        assert config.environment == "development"
        assert config.app_name == "TracSeq API Gateway"
        assert config.app_version == "2.0.0"
        assert config.debug == False
        assert config.testing == False
        
        # Test sub-configurations
        assert isinstance(config.database, DatabaseConfig)
        assert isinstance(config.security, SecurityConfig)
        assert isinstance(config.gateway, GatewayConfig)
        assert isinstance(config.services, ServiceConfig)
        assert isinstance(config.logging, LoggingConfig)
        assert isinstance(config.monitoring, MonitoringConfig)
        assert isinstance(config.cors, CORSConfig)
        assert isinstance(config.chat, ChatConfig)
    
    def test_environment_validation(self):
        """Test environment validation."""
        # Valid environments
        for env in ['development', 'staging', 'production', 'testing']:
            config = AppConfig(environment=env)
            assert config.environment == env
        
        # Invalid environment
        with pytest.raises(ValidationError) as exc_info:
            AppConfig(environment="invalid")
        assert "Environment must be one of" in str(exc_info.value)
    
    def test_environment_helpers(self):
        """Test environment helper methods."""
        # Development
        config = AppConfig(environment="development")
        assert config.is_development() == True
        assert config.is_production() == False
        assert config.is_testing() == False
        
        # Production
        config = AppConfig(environment="production")
        assert config.is_development() == False
        assert config.is_production() == True
        assert config.is_testing() == False
        
        # Testing
        config = AppConfig(environment="testing")
        assert config.is_development() == False
        assert config.is_production() == False
        assert config.is_testing() == True
        
        # Testing flag
        config = AppConfig(environment="development", testing=True)
        assert config.is_testing() == True


class TestEnvironmentLoading:
    """Test environment variable loading."""
    
    def test_load_config_from_env(self):
        """Test loading configuration from environment variables."""
        env_vars = {
            'DATABASE_URL': 'postgres://test:test@localhost:5432/test_db',
            'GATEWAY_HOST': '127.0.0.1',
            'GATEWAY_PORT': '8080',
            'JWT_SECRET_KEY': 'test-secret-key-32-characters-long',
            'LOG_LEVEL': 'DEBUG',
            'ENVIRONMENT': 'testing'
        }
        
        with patch.dict(os.environ, env_vars):
            config = load_config_from_env()
            
            assert config.database.url == 'postgres://test:test@localhost:5432/test_db'
            assert config.gateway.host == '127.0.0.1'
            assert config.gateway.port == 8080
            assert config.security.jwt_secret_key == 'test-secret-key-32-characters-long'
            assert config.logging.log_level == 'DEBUG'
            assert config.environment == 'testing'
    
    def test_cors_origins_parsing(self):
        """Test CORS origins parsing from environment."""
        env_vars = {
            'CORS_ORIGINS': 'http://localhost:3000,http://localhost:8080,https://example.com'
        }
        
        with patch.dict(os.environ, env_vars):
            config = load_config_from_env()
            assert config.cors.allow_origins == [
                'http://localhost:3000',
                'http://localhost:8080', 
                'https://example.com'
            ]
    
    def test_boolean_parsing(self):
        """Test boolean environment variable parsing."""
        env_vars = {
            'GATEWAY_DEBUG': 'true',
            'ENABLE_METRICS': 'false',
            'CORS_CREDENTIALS': 'True',
            'ENABLE_CHAT': 'FALSE'
        }
        
        with patch.dict(os.environ, env_vars):
            config = load_config_from_env()
            assert config.gateway.debug == True
            assert config.monitoring.enable_metrics == False
            assert config.cors.allow_credentials == True
            assert config.chat.enable_chat == False


class TestConfigurationSingleton:
    """Test configuration singleton pattern."""
    
    def test_get_config_singleton(self):
        """Test that get_config returns the same instance."""
        config1 = get_config()
        config2 = get_config()
        assert config1 is config2
    
    def test_reload_config(self):
        """Test configuration reloading."""
        # Get initial config
        config1 = get_config()
        
        # Reload config
        config2 = reload_config()
        
        # Should be different instances
        assert config1 is not config2
        
        # But get_config should now return the new instance
        config3 = get_config()
        assert config2 is config3


class TestConfigurationIntegration:
    """Test configuration integration."""
    
    def test_complete_configuration(self):
        """Test complete configuration with all components."""
        config = AppConfig(
            environment="production",
            app_name="Test Gateway",
            app_version="1.0.0",
            debug=False,
            testing=False,
            database=DatabaseConfig(
                url="postgres://prod:prod@db:5432/prod_db",
                pool_min_size=10,
                pool_max_size=50
            ),
            security=SecurityConfig(
                jwt_secret_key="production-secret-key-32-characters-long",
                rate_limit_requests=200
            ),
            gateway=GatewayConfig(
                host="0.0.0.0",
                port=80,
                workers=8
            )
        )
        
        assert config.environment == "production"
        assert config.app_name == "Test Gateway"
        assert config.database.url == "postgres://prod:prod@db:5432/prod_db"
        assert config.security.jwt_secret_key == "production-secret-key-32-characters-long"
        assert config.gateway.port == 80
        assert config.is_production() == True
        assert config.is_development() == False