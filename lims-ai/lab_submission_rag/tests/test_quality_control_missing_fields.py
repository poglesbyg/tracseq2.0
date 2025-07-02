"""Test to verify missing fields are properly propagated in quality control workflow"""

import asyncio
import pytest
from unittest.mock import AsyncMock, MagicMock, patch

from ..workflows.quality_control import QualityControlWorkflow


@pytest.mark.asyncio
async def test_missing_fields_propagation():
    """Test that missing fields identified during validation are included in the final result"""
    
    # Create workflow instance
    workflow = QualityControlWorkflow()
    
    # Mock the LLM interface to return incomplete extraction
    mock_extraction = {
        "administrative": {
            # Missing submitter_email
            "institution": "Test Lab"
        },
        "sample": {
            # Missing sample_id and sample_type
            "volume": "100ul"
        },
        "sequencing": {
            "platform": "illumina"
        }
    }
    
    with patch.object(workflow.llm_interface, 'extract_submission_info_with_prompt', 
                      return_value=mock_extraction):
        
        # Run the workflow
        result = await workflow.run(
            text="Test laboratory submission text",
            require_human_review=False
        )
        
        # Verify the result includes the missing fields
        assert result.missing_fields is not None
        assert "submitter_email" in result.missing_fields
        assert "sample_id" in result.missing_fields
        assert "sample_type" in result.missing_fields
        assert len(result.missing_fields) == 3
        
        # The extraction should still be marked as partially successful
        # (depending on confidence threshold)
        assert result.confidence_score < 1.0
        
        print(f"✓ Test passed! Missing fields correctly propagated: {result.missing_fields}")


if __name__ == "__main__":
    asyncio.run(test_missing_fields_propagation()) 