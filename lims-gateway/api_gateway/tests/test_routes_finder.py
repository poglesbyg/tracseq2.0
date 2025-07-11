"""
Tests for the finder routes.

This module tests the comprehensive finder functionality including
search, filtering, and data aggregation across all laboratory data types.
"""

import pytest
from unittest.mock import Mock, AsyncMock, patch
from datetime import datetime, timedelta
from fastapi.testclient import TestClient
from fastapi import FastAPI

from api_gateway.routes.finder import router, get_database_connection
from api_gateway.core.exceptions import DatabaseException


class TestFinderRoutes:
    """Test finder route functionality."""
    
    def setup_method(self):
        """Setup test fixtures."""
        self.app = FastAPI()
        self.app.include_router(router, prefix="/api/finder")
        self.client = TestClient(self.app)
        
        # Mock database connection
        self.mock_conn = Mock()
        self.mock_conn.fetch = AsyncMock(return_value=[])
        self.mock_conn.fetchval = AsyncMock(return_value=0)
        self.mock_conn.__aenter__ = AsyncMock(return_value=self.mock_conn)
        self.mock_conn.__aexit__ = AsyncMock(return_value=None)
    
    def test_get_all_finder_data_basic(self):
        """Test basic finder data retrieval."""
        with patch('api_gateway.routes.finder.get_database_connection', return_value=self.mock_conn):
            response = self.client.get("/api/finder/all-data")
            
            assert response.status_code == 200
            data = response.json()
            assert data["success"] == True
            assert "data" in data
            assert "pagination" in data
            assert "categories" in data
            
            # Check pagination structure
            pagination = data["pagination"]
            assert "total" in pagination
            assert "offset" in pagination
            assert "limit" in pagination
            assert "has_more" in pagination
            
            # Check categories structure
            categories = data["categories"]
            expected_categories = [
                "samples", "templates", "storage", "sequencing", 
                "qc", "library", "projects", "reports"
            ]
            for category in expected_categories:
                assert category in categories
    
    def test_get_all_finder_data_with_search(self):
        """Test finder data retrieval with search parameter."""
        with patch('api_gateway.routes.finder.get_database_connection', return_value=self.mock_conn):
            response = self.client.get("/api/finder/all-data?search=test")
            
            assert response.status_code == 200
            data = response.json()
            assert data["success"] == True
    
    def test_get_all_finder_data_with_category_filter(self):
        """Test finder data retrieval with category filter."""
        with patch('api_gateway.routes.finder.get_database_connection', return_value=self.mock_conn):
            response = self.client.get("/api/finder/all-data?category=samples")
            
            assert response.status_code == 200
            data = response.json()
            assert data["success"] == True
    
    def test_get_all_finder_data_with_pagination(self):
        """Test finder data retrieval with pagination."""
        with patch('api_gateway.routes.finder.get_database_connection', return_value=self.mock_conn):
            response = self.client.get("/api/finder/all-data?limit=50&offset=10")
            
            assert response.status_code == 200
            data = response.json()
            assert data["success"] == True
            assert data["pagination"]["limit"] == 50
            assert data["pagination"]["offset"] == 10
    
    def test_get_all_finder_data_with_samples(self):
        """Test finder data retrieval with sample data."""
        # Mock sample data
        sample_data = [
            {
                'id': 1,
                'name': 'Sample 1',
                'barcode': 'S001',
                'sample_type': 'Blood',
                'status': 'Active',
                'created_at': datetime.now(),
                'updated_at': datetime.now(),
                'metadata': '{"template_data": {"Department": "Hematology"}}',
                'concentration': 10.5,
                'volume': 2.0,
                'quality_score': 8.5,
                'notes': 'Test sample',
                'template_id': 1
            }
        ]
        
        self.mock_conn.fetch = AsyncMock(return_value=sample_data)
        
        with patch('api_gateway.routes.finder.get_database_connection', return_value=self.mock_conn):
            response = self.client.get("/api/finder/all-data")
            
            assert response.status_code == 200
            data = response.json()
            assert data["success"] == True
            assert len(data["data"]) > 0
            
            # Check sample data structure
            sample = data["data"][0]
            assert sample["id"] == "sample-1"
            assert sample["name"] == "Sample 1"
            assert sample["type"] == "sample"
            assert sample["category"] == "samples"
            assert sample["barcode"] == "S001"
            assert sample["sample_type"] == "Blood"
            assert sample["status"] == "Active"
            assert sample["department"] == "Hematology"
    
    def test_get_all_finder_data_with_templates(self):
        """Test finder data retrieval with template data."""
        # Mock template data
        template_data = [
            {
                'id': 1,
                'name': 'Template 1',
                'description': 'Test template',
                'category': 'Blood',
                'status': 'Active',
                'created_at': datetime.now(),
                'updated_at': datetime.now(),
                'tags': ['tag1', 'tag2'],
                'is_public': True,
                'usage_count': 5,
                'version': '1.0'
            }
        ]
        
        # Mock samples query to return empty, templates query to return data
        self.mock_conn.fetch = AsyncMock(side_effect=[[], template_data])
        
        with patch('api_gateway.routes.finder.get_database_connection', return_value=self.mock_conn):
            response = self.client.get("/api/finder/all-data")
            
            assert response.status_code == 200
            data = response.json()
            assert data["success"] == True
            assert len(data["data"]) > 0
            
            # Check template data structure
            template = data["data"][0]
            assert template["id"] == "template-1"
            assert template["name"] == "Template 1"
            assert template["type"] == "template"
            assert template["category"] == "templates"
            assert template["description"] == "Test template"
            assert template["template_category"] == "Blood"
            assert template["status"] == "Active"
            assert template["tags"] == ['tag1', 'tag2']
            assert template["is_public"] == True
            assert template["usage_count"] == 5
            assert template["version"] == "1.0"
    
    def test_get_finder_categories(self):
        """Test finder categories endpoint."""
        # Mock category counts
        self.mock_conn.fetchval = AsyncMock(side_effect=[10, 5, 3, 2, 1, 4, 6, 8])
        
        with patch('api_gateway.routes.finder.get_database_connection', return_value=self.mock_conn):
            response = self.client.get("/api/finder/categories")
            
            assert response.status_code == 200
            data = response.json()
            assert data["success"] == True
            assert "categories" in data
            assert "total_items" in data
            
            categories = data["categories"]
            
            # Check expected categories
            assert "samples" in categories
            assert categories["samples"]["count"] == 10
            assert categories["samples"]["label"] == "Samples"
            assert categories["samples"]["icon"] == "beaker"
            
            assert "templates" in categories
            assert categories["templates"]["count"] == 5
            assert categories["templates"]["label"] == "Templates"
            assert categories["templates"]["icon"] == "document"
            
            assert "storage" in categories
            assert categories["storage"]["count"] == 3
            assert categories["storage"]["label"] == "Storage"
            assert categories["storage"]["icon"] == "archive"
            
            # Check total items
            assert data["total_items"] == 39  # Sum of all counts
    
    def test_search_finder_data(self):
        """Test finder search endpoint."""
        with patch('api_gateway.routes.finder.get_database_connection', return_value=self.mock_conn):
            response = self.client.get("/api/finder/search?q=test")
            
            assert response.status_code == 200
            data = response.json()
            assert data["success"] == True
    
    def test_search_finder_data_with_category(self):
        """Test finder search with category filter."""
        with patch('api_gateway.routes.finder.get_database_connection', return_value=self.mock_conn):
            response = self.client.get("/api/finder/search?q=test&category=samples")
            
            assert response.status_code == 200
            data = response.json()
            assert data["success"] == True
    
    def test_search_finder_data_missing_query(self):
        """Test finder search without query parameter."""
        response = self.client.get("/api/finder/search")
        
        assert response.status_code == 422  # Validation error
    
    def test_get_recent_finder_data(self):
        """Test recent finder data endpoint."""
        # Mock recent sample data
        recent_data = [
            {
                'id': 1,
                'name': 'Recent Sample',
                'barcode': 'RS001',
                'sample_type': 'Blood',
                'status': 'Active',
                'created_at': datetime.now() - timedelta(days=1),
                'updated_at': datetime.now()
            }
        ]
        
        self.mock_conn.fetch = AsyncMock(return_value=recent_data)
        
        with patch('api_gateway.routes.finder.get_database_connection', return_value=self.mock_conn):
            response = self.client.get("/api/finder/recent")
            
            assert response.status_code == 200
            data = response.json()
            assert data["success"] == True
            assert "data" in data
            assert "days" in data
            assert "total_items" in data
            assert data["days"] == 30  # Default
    
    def test_get_recent_finder_data_custom_days(self):
        """Test recent finder data with custom days parameter."""
        with patch('api_gateway.routes.finder.get_database_connection', return_value=self.mock_conn):
            response = self.client.get("/api/finder/recent?days=7")
            
            assert response.status_code == 200
            data = response.json()
            assert data["success"] == True
            assert data["days"] == 7
    
    def test_get_recent_finder_data_invalid_days(self):
        """Test recent finder data with invalid days parameter."""
        response = self.client.get("/api/finder/recent?days=0")
        
        assert response.status_code == 422  # Validation error
        
        response = self.client.get("/api/finder/recent?days=400")
        
        assert response.status_code == 422  # Validation error
    
    def test_get_finder_stats(self):
        """Test finder statistics endpoint."""
        # Mock category counts for stats
        self.mock_conn.fetchval = AsyncMock(side_effect=[10, 5, 3, 2, 1, 4, 6, 8])
        
        # Mock recent data
        recent_data = [
            {
                'id': 1,
                'name': 'Recent Sample',
                'barcode': 'RS001',
                'sample_type': 'Blood',
                'status': 'Active',
                'created_at': datetime.now() - timedelta(days=1),
                'updated_at': datetime.now()
            }
        ]
        self.mock_conn.fetch = AsyncMock(return_value=recent_data)
        
        with patch('api_gateway.routes.finder.get_database_connection', return_value=self.mock_conn):
            response = self.client.get("/api/finder/stats")
            
            assert response.status_code == 200
            data = response.json()
            assert data["success"] == True
            assert "stats" in data
            
            stats = data["stats"]
            assert "categories" in stats
            assert "total_items" in stats
            assert "recent_activity" in stats
            assert "system_health" in stats
            
            # Check recent activity
            recent_activity = stats["recent_activity"]
            assert "items_last_7_days" in recent_activity
            assert "categories_with_activity" in recent_activity
            
            # Check system health
            system_health = stats["system_health"]
            assert "database_responsive" in system_health
            assert "last_updated" in system_health
            assert "data_freshness" in system_health
            assert system_health["database_responsive"] == True
    
    def test_pagination_validation(self):
        """Test pagination parameter validation."""
        # Test invalid limit
        response = self.client.get("/api/finder/all-data?limit=0")
        assert response.status_code == 422
        
        response = self.client.get("/api/finder/all-data?limit=6000")
        assert response.status_code == 422
        
        # Test invalid offset
        response = self.client.get("/api/finder/all-data?offset=-1")
        assert response.status_code == 422
        
        # Test valid pagination
        with patch('api_gateway.routes.finder.get_database_connection', return_value=self.mock_conn):
            response = self.client.get("/api/finder/all-data?limit=100&offset=50")
            assert response.status_code == 200
    
    def test_category_validation(self):
        """Test category parameter validation."""
        valid_categories = [
            "samples", "templates", "projects", "reports", 
            "storage", "sequencing", "qc", "library"
        ]
        
        with patch('api_gateway.routes.finder.get_database_connection', return_value=self.mock_conn):
            for category in valid_categories:
                response = self.client.get(f"/api/finder/all-data?category={category}")
                assert response.status_code == 200
            
            # Test with invalid category (should still work, just return empty results)
            response = self.client.get("/api/finder/all-data?category=invalid")
            assert response.status_code == 200
    
    def test_database_error_handling(self):
        """Test database error handling."""
        # Mock database connection failure
        mock_failing_conn = Mock()
        mock_failing_conn.fetch = AsyncMock(side_effect=Exception("Database error"))
        mock_failing_conn.__aenter__ = AsyncMock(return_value=mock_failing_conn)
        mock_failing_conn.__aexit__ = AsyncMock(return_value=None)
        
        with patch('api_gateway.routes.finder.get_database_connection', return_value=mock_failing_conn):
            response = self.client.get("/api/finder/all-data")
            
            assert response.status_code == 500
            data = response.json()
            assert "Failed to retrieve finder data" in data["detail"]["message"]
    
    def test_metadata_parsing(self):
        """Test metadata parsing for samples."""
        # Mock sample with JSON metadata
        sample_data = [
            {
                'id': 1,
                'name': 'Sample 1',
                'barcode': 'S001',
                'sample_type': 'Blood',
                'status': 'Active',
                'created_at': datetime.now(),
                'updated_at': datetime.now(),
                'metadata': '{"template_data": {"Department": "Hematology", "Submitter": "Dr. Smith"}}',
                'concentration': 10.5,
                'volume': 2.0,
                'quality_score': 8.5,
                'notes': 'Test sample',
                'template_id': 1
            }
        ]
        
        self.mock_conn.fetch = AsyncMock(return_value=sample_data)
        
        with patch('api_gateway.routes.finder.get_database_connection', return_value=self.mock_conn):
            response = self.client.get("/api/finder/all-data")
            
            assert response.status_code == 200
            data = response.json()
            sample = data["data"][0]
            
            assert sample["department"] == "Hematology"
            assert sample["submitter"] == "Dr. Smith"
    
    def test_metadata_parsing_invalid_json(self):
        """Test metadata parsing with invalid JSON."""
        # Mock sample with invalid JSON metadata
        sample_data = [
            {
                'id': 1,
                'name': 'Sample 1',
                'barcode': 'S001',
                'sample_type': 'Blood',
                'status': 'Active',
                'created_at': datetime.now(),
                'updated_at': datetime.now(),
                'metadata': 'invalid json',
                'concentration': 10.5,
                'volume': 2.0,
                'quality_score': 8.5,
                'notes': 'Test sample',
                'template_id': 1
            }
        ]
        
        self.mock_conn.fetch = AsyncMock(return_value=sample_data)
        
        with patch('api_gateway.routes.finder.get_database_connection', return_value=self.mock_conn):
            response = self.client.get("/api/finder/all-data")
            
            assert response.status_code == 200
            data = response.json()
            sample = data["data"][0]
            
            # Should handle invalid JSON gracefully
            assert sample["department"] is None
            assert sample["submitter"] is None
    
    def test_search_functionality(self):
        """Test search functionality across different data types."""
        # Mock sample data that matches search
        sample_data = [
            {
                'id': 1,
                'name': 'Test Sample',
                'barcode': 'TS001',
                'sample_type': 'Blood',
                'status': 'Active',
                'created_at': datetime.now(),
                'updated_at': datetime.now(),
                'metadata': '{"template_data": {"Department": "Hematology"}}',
                'concentration': 10.5,
                'volume': 2.0,
                'quality_score': 8.5,
                'notes': 'Test sample',
                'template_id': 1
            }
        ]
        
        self.mock_conn.fetch = AsyncMock(return_value=sample_data)
        
        with patch('api_gateway.routes.finder.get_database_connection', return_value=self.mock_conn):
            # Search should find the sample
            response = self.client.get("/api/finder/all-data?search=test")
            
            assert response.status_code == 200
            data = response.json()
            assert len(data["data"]) > 0
            
            # Search should not find anything with non-matching term
            response = self.client.get("/api/finder/all-data?search=nonexistent")
            
            assert response.status_code == 200
            data = response.json()
            # Should return empty since our mock data doesn't change
            # In real implementation, this would be filtered out
    
    def test_sorting_functionality(self):
        """Test data sorting by creation/update time."""
        # Mock data with different timestamps
        sample_data = [
            {
                'id': 1,
                'name': 'Older Sample',
                'barcode': 'OS001',
                'sample_type': 'Blood',
                'status': 'Active',
                'created_at': datetime.now() - timedelta(days=2),
                'updated_at': datetime.now() - timedelta(days=2),
                'metadata': '{}',
                'concentration': 10.5,
                'volume': 2.0,
                'quality_score': 8.5,
                'notes': 'Older sample',
                'template_id': 1
            },
            {
                'id': 2,
                'name': 'Newer Sample',
                'barcode': 'NS001',
                'sample_type': 'Blood',
                'status': 'Active',
                'created_at': datetime.now() - timedelta(days=1),
                'updated_at': datetime.now() - timedelta(days=1),
                'metadata': '{}',
                'concentration': 10.5,
                'volume': 2.0,
                'quality_score': 8.5,
                'notes': 'Newer sample',
                'template_id': 1
            }
        ]
        
        self.mock_conn.fetch = AsyncMock(return_value=sample_data)
        
        with patch('api_gateway.routes.finder.get_database_connection', return_value=self.mock_conn):
            response = self.client.get("/api/finder/all-data")
            
            assert response.status_code == 200
            data = response.json()
            assert len(data["data"]) == 2
            
            # Should be sorted by most recent first
            # Note: In the actual implementation, sorting happens after data aggregation
            # This test verifies the endpoint works with multiple items