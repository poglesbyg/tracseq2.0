"""
Pytest configuration and common test fixtures.

This module provides shared test fixtures and configuration for the
API Gateway test suite.
"""

import pytest
import asyncio
from unittest.mock import Mock, AsyncMock
from datetime import datetime, timedelta
import os
import sys

# Add the parent directory to the path so we can import our modules
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

@pytest.fixture(scope="session")
def event_loop():
    """Create an instance of the default event loop for the test session."""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()

@pytest.fixture
def mock_database_connection():
    """Mock database connection for testing."""
    mock_conn = Mock()
    mock_conn.fetch = AsyncMock(return_value=[])
    mock_conn.fetchval = AsyncMock(return_value=0)
    mock_conn.execute = AsyncMock(return_value=None)
    mock_conn.executemany = AsyncMock(return_value=None)
    mock_conn.__aenter__ = AsyncMock(return_value=mock_conn)
    mock_conn.__aexit__ = AsyncMock(return_value=None)
    return mock_conn

@pytest.fixture
def mock_http_client():
    """Mock HTTP client for testing."""
    mock_client = Mock()
    mock_client.get = AsyncMock()
    mock_client.post = AsyncMock()
    mock_client.put = AsyncMock()
    mock_client.delete = AsyncMock()
    mock_client.request = AsyncMock()
    return mock_client

@pytest.fixture
def sample_data():
    """Sample test data for database queries."""
    return [
        {
            'id': 1,
            'name': 'Test Sample 1',
            'barcode': 'TS001',
            'sample_type': 'Blood',
            'status': 'Active',
            'created_at': datetime.now() - timedelta(days=1),
            'updated_at': datetime.now(),
            'metadata': '{"template_data": {"Department": "Hematology", "Submitter": "Dr. Smith"}}',
            'concentration': 10.5,
            'volume': 2.0,
            'quality_score': 8.5,
            'notes': 'Test sample for unit tests',
            'template_id': 1
        },
        {
            'id': 2,
            'name': 'Test Sample 2',
            'barcode': 'TS002',
            'sample_type': 'Serum',
            'status': 'Processing',
            'created_at': datetime.now() - timedelta(days=2),
            'updated_at': datetime.now() - timedelta(hours=2),
            'metadata': '{"template_data": {"Department": "Chemistry", "Submitter": "Dr. Johnson"}}',
            'concentration': 15.2,
            'volume': 1.5,
            'quality_score': 9.0,
            'notes': 'Second test sample',
            'template_id': 2
        }
    ]

@pytest.fixture
def template_data():
    """Template test data for database queries."""
    return [
        {
            'id': 1,
            'name': 'Blood Template',
            'description': 'Template for blood samples',
            'category': 'Blood',
            'status': 'Active',
            'created_at': datetime.now() - timedelta(days=5),
            'updated_at': datetime.now() - timedelta(days=1),
            'tags': ['blood', 'hematology'],
            'is_public': True,
            'usage_count': 25,
            'version': '1.2'
        },
        {
            'id': 2,
            'name': 'Serum Template',
            'description': 'Template for serum samples',
            'category': 'Serum',
            'status': 'Active',
            'created_at': datetime.now() - timedelta(days=3),
            'updated_at': datetime.now() - timedelta(hours=6),
            'tags': ['serum', 'chemistry'],
            'is_public': False,
            'usage_count': 12,
            'version': '1.0'
        }
    ]

@pytest.fixture
def storage_data():
    """Storage location test data."""
    return [
        {
            'id': 1,
            'name': 'Freezer A1',
            'location': 'Lab 1 - Freezer A1',
            'temperature': -80,
            'capacity': 100,
            'current_samples': 45,
            'status': 'Active',
            'created_at': datetime.now() - timedelta(days=10),
            'updated_at': datetime.now() - timedelta(hours=1),
            'notes': 'Ultra-low temperature freezer'
        },
        {
            'id': 2,
            'name': 'Refrigerator B2',
            'location': 'Lab 2 - Refrigerator B2',
            'temperature': 4,
            'capacity': 50,
            'current_samples': 23,
            'status': 'Active',
            'created_at': datetime.now() - timedelta(days=8),
            'updated_at': datetime.now() - timedelta(hours=3),
            'notes': 'Standard laboratory refrigerator'
        }
    ]

@pytest.fixture
def sequencing_data():
    """Sequencing job test data."""
    return [
        {
            'id': 1,
            'name': 'Sequencing Job 1',
            'sample_count': 24,
            'platform': 'Illumina',
            'status': 'Running',
            'created_at': datetime.now() - timedelta(days=2),
            'updated_at': datetime.now() - timedelta(hours=4),
            'estimated_completion': datetime.now() + timedelta(days=1),
            'cost': 2400.00,
            'notes': 'Whole genome sequencing'
        },
        {
            'id': 2,
            'name': 'Sequencing Job 2',
            'sample_count': 12,
            'platform': 'Oxford Nanopore',
            'status': 'Completed',
            'created_at': datetime.now() - timedelta(days=5),
            'updated_at': datetime.now() - timedelta(days=1),
            'estimated_completion': datetime.now() - timedelta(days=1),
            'cost': 1800.00,
            'notes': 'Long-read sequencing'
        }
    ]

@pytest.fixture
def qc_data():
    """Quality control test data."""
    return [
        {
            'id': 1,
            'sample_id': 1,
            'assessment_type': 'Purity',
            'result': 'Pass',
            'score': 8.5,
            'performed_by': 'QC Tech 1',
            'created_at': datetime.now() - timedelta(days=1),
            'updated_at': datetime.now() - timedelta(hours=2),
            'notes': 'Sample purity within acceptable range'
        },
        {
            'id': 2,
            'sample_id': 2,
            'assessment_type': 'Contamination',
            'result': 'Pass',
            'score': 9.2,
            'performed_by': 'QC Tech 2',
            'created_at': datetime.now() - timedelta(hours=8),
            'updated_at': datetime.now() - timedelta(hours=1),
            'notes': 'No contamination detected'
        }
    ]

@pytest.fixture
def library_data():
    """Library preparation test data."""
    return [
        {
            'id': 1,
            'name': 'Library Prep 1',
            'sample_id': 1,
            'protocol': 'TruSeq',
            'status': 'Completed',
            'created_at': datetime.now() - timedelta(days=3),
            'updated_at': datetime.now() - timedelta(days=2),
            'concentration': 25.4,
            'volume': 50.0,
            'notes': 'Standard library preparation'
        },
        {
            'id': 2,
            'name': 'Library Prep 2',
            'sample_id': 2,
            'protocol': 'Nextera',
            'status': 'In Progress',
            'created_at': datetime.now() - timedelta(days=1),
            'updated_at': datetime.now() - timedelta(hours=6),
            'concentration': 18.7,
            'volume': 45.0,
            'notes': 'Nextera library preparation'
        }
    ]

@pytest.fixture
def project_data():
    """Project test data."""
    return [
        {
            'id': 1,
            'name': 'Cancer Research Project',
            'description': 'Comprehensive cancer genomics study',
            'principal_investigator': 'Dr. Wilson',
            'status': 'Active',
            'created_at': datetime.now() - timedelta(days=30),
            'updated_at': datetime.now() - timedelta(days=1),
            'budget': 50000.00,
            'sample_count': 150,
            'completion_percentage': 65.0
        },
        {
            'id': 2,
            'name': 'Infectious Disease Study',
            'description': 'Pathogen identification and resistance profiling',
            'principal_investigator': 'Dr. Martinez',
            'status': 'Planning',
            'created_at': datetime.now() - timedelta(days=7),
            'updated_at': datetime.now() - timedelta(hours=12),
            'budget': 25000.00,
            'sample_count': 75,
            'completion_percentage': 5.0
        }
    ]

@pytest.fixture
def report_data():
    """Report test data."""
    return [
        {
            'id': 1,
            'name': 'Monthly QC Report',
            'type': 'QC Summary',
            'generated_by': 'System',
            'created_at': datetime.now() - timedelta(days=1),
            'updated_at': datetime.now() - timedelta(hours=2),
            'file_path': '/reports/qc_monthly_2024_01.pdf',
            'file_size': 2048576,
            'status': 'Generated'
        },
        {
            'id': 2,
            'name': 'Project Progress Report',
            'type': 'Project Status',
            'generated_by': 'Dr. Wilson',
            'created_at': datetime.now() - timedelta(days=3),
            'updated_at': datetime.now() - timedelta(days=3),
            'file_path': '/reports/project_progress_2024_01.pdf',
            'file_size': 1536000,
            'status': 'Generated'
        }
    ]

@pytest.fixture
def mock_config():
    """Mock configuration for testing."""
    config = Mock()
    config.database = Mock()
    config.database.url = "postgres://test:test@localhost:5432/test_db"
    config.database.pool_min_size = 2
    config.database.pool_max_size = 10
    
    config.security = Mock()
    config.security.jwt_secret_key = "test-secret-key-32-characters-long"
    config.security.jwt_algorithm = "HS256"
    config.security.jwt_expiration_hours = 24
    
    config.gateway = Mock()
    config.gateway.host = "127.0.0.1"
    config.gateway.port = 8000
    config.gateway.debug = True
    
    config.services = Mock()
    config.services.auth_service_url = "http://test-auth:8000"
    config.services.sample_service_url = "http://test-sample:8001"
    config.services.service_timeout = 30
    config.services.service_retries = 3
    
    config.logging = Mock()
    config.logging.log_level = "DEBUG"
    config.logging.enable_access_log = True
    
    config.monitoring = Mock()
    config.monitoring.enable_metrics = True
    config.monitoring.circuit_breaker_failure_threshold = 5
    config.monitoring.circuit_breaker_recovery_timeout = 60
    
    return config

# Test environment setup
def pytest_configure(config):
    """Configure pytest environment."""
    # Set test environment variables
    os.environ.setdefault('ENVIRONMENT', 'testing')
    os.environ.setdefault('DATABASE_URL', 'postgres://test:test@localhost:5432/test_db')
    os.environ.setdefault('JWT_SECRET_KEY', 'test-secret-key-32-characters-long')
    os.environ.setdefault('LOG_LEVEL', 'DEBUG')

def pytest_unconfigure(config):
    """Clean up after tests."""
    # Clean up test environment variables
    test_env_vars = [
        'ENVIRONMENT', 'DATABASE_URL', 'JWT_SECRET_KEY', 'LOG_LEVEL'
    ]
    for var in test_env_vars:
        if var in os.environ:
            del os.environ[var]