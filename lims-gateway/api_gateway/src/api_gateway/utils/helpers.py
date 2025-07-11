"""
Utility helper functions for the API Gateway.

This module provides common utility functions used across the API Gateway
components for data processing, validation, and formatting.
"""

import re
import json
import hashlib
import secrets
import uuid
from datetime import datetime, timezone
from typing import Any, Dict, List, Optional, Union
from pathlib import Path


def generate_request_id() -> str:
    """
    Generate a unique request ID for tracing.
    
    Returns:
        str: A unique request ID
    """
    return str(uuid.uuid4())


def generate_correlation_id() -> str:
    """
    Generate a correlation ID for request tracing.
    
    Returns:
        str: A unique correlation ID
    """
    return secrets.token_hex(16)


def sanitize_string(value: str, max_length: int = 1000) -> str:
    """
    Sanitize a string by removing potentially dangerous characters.
    
    Args:
        value: The string to sanitize
        max_length: Maximum allowed length
        
    Returns:
        str: Sanitized string
    """
    if not isinstance(value, str):
        return str(value)
    
    # Remove control characters except newlines and tabs
    sanitized = re.sub(r'[\x00-\x08\x0b\x0c\x0e-\x1f\x7f-\x9f]', '', value)
    
    # Truncate if too long
    if len(sanitized) > max_length:
        sanitized = sanitized[:max_length]
    
    return sanitized.strip()


def validate_email(email: str) -> bool:
    """
    Validate email address format.
    
    Args:
        email: Email address to validate
        
    Returns:
        bool: True if valid email format
    """
    pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
    return bool(re.match(pattern, email))


def validate_barcode(barcode: str) -> bool:
    """
    Validate sample barcode format.
    
    Args:
        barcode: Barcode to validate
        
    Returns:
        bool: True if valid barcode format
    """
    # Allow alphanumeric characters, hyphens, and underscores
    pattern = r'^[A-Za-z0-9_-]+$'
    return bool(re.match(pattern, barcode)) and 3 <= len(barcode) <= 50


def format_file_size(size_bytes: int) -> str:
    """
    Format file size in human-readable format.
    
    Args:
        size_bytes: Size in bytes
        
    Returns:
        str: Formatted size string
    """
    if size_bytes == 0:
        return "0 B"
    
    size_names = ["B", "KB", "MB", "GB", "TB"]
    i = 0
    while size_bytes >= 1024 and i < len(size_names) - 1:
        size_bytes /= 1024.0
        i += 1
    
    return f"{size_bytes:.1f} {size_names[i]}"


def format_duration(seconds: float) -> str:
    """
    Format duration in human-readable format.
    
    Args:
        seconds: Duration in seconds
        
    Returns:
        str: Formatted duration string
    """
    if seconds < 1:
        return f"{seconds * 1000:.1f} ms"
    elif seconds < 60:
        return f"{seconds:.1f} s"
    elif seconds < 3600:
        minutes = seconds / 60
        return f"{minutes:.1f} min"
    else:
        hours = seconds / 3600
        return f"{hours:.1f} h"


def safe_json_parse(json_str: str) -> Optional[Dict[str, Any]]:
    """
    Safely parse JSON string.
    
    Args:
        json_str: JSON string to parse
        
    Returns:
        Optional[Dict[str, Any]]: Parsed JSON or None if invalid
    """
    try:
        return json.loads(json_str)
    except (json.JSONDecodeError, TypeError):
        return None


def safe_json_dumps(obj: Any) -> str:
    """
    Safely serialize object to JSON string.
    
    Args:
        obj: Object to serialize
        
    Returns:
        str: JSON string or empty string if serialization fails
    """
    try:
        return json.dumps(obj, default=str, ensure_ascii=False)
    except (TypeError, ValueError):
        return ""


def extract_metadata_field(metadata: Optional[str], field_path: str) -> Optional[str]:
    """
    Extract a field from JSON metadata using dot notation.
    
    Args:
        metadata: JSON metadata string
        field_path: Dot-separated field path (e.g., "template_data.Department")
        
    Returns:
        Optional[str]: Field value or None if not found
    """
    if not metadata:
        return None
    
    data = safe_json_parse(metadata)
    if not data:
        return None
    
    # Navigate through the field path
    current = data
    for field in field_path.split('.'):
        if isinstance(current, dict) and field in current:
            current = current[field]
        else:
            return None
    
    return str(current) if current is not None else None


def calculate_hash(data: Union[str, bytes]) -> str:
    """
    Calculate SHA-256 hash of data.
    
    Args:
        data: Data to hash
        
    Returns:
        str: Hex-encoded hash
    """
    if isinstance(data, str):
        data = data.encode('utf-8')
    
    return hashlib.sha256(data).hexdigest()


def mask_sensitive_data(data: str, mask_char: str = "*", visible_chars: int = 4) -> str:
    """
    Mask sensitive data showing only the last few characters.
    
    Args:
        data: Data to mask
        mask_char: Character to use for masking
        visible_chars: Number of characters to leave visible
        
    Returns:
        str: Masked data
    """
    if len(data) <= visible_chars:
        return mask_char * len(data)
    
    return mask_char * (len(data) - visible_chars) + data[-visible_chars:]


def normalize_temperature(temp: float, unit: str = "C") -> float:
    """
    Normalize temperature to Celsius.
    
    Args:
        temp: Temperature value
        unit: Temperature unit ("C", "F", "K")
        
    Returns:
        float: Temperature in Celsius
    """
    unit = unit.upper()
    
    if unit == "C":
        return temp
    elif unit == "F":
        return (temp - 32) * 5/9
    elif unit == "K":
        return temp - 273.15
    else:
        raise ValueError(f"Unsupported temperature unit: {unit}")


def validate_concentration(concentration: float, unit: str = "ng/μL") -> bool:
    """
    Validate concentration value.
    
    Args:
        concentration: Concentration value
        unit: Concentration unit
        
    Returns:
        bool: True if valid concentration
    """
    if concentration < 0:
        return False
    
    # Set reasonable upper limits based on unit
    max_values = {
        "ng/μL": 10000,
        "μg/mL": 10000,
        "mg/mL": 1000,
        "g/L": 1000,
        "M": 10,
        "mM": 10000,
        "μM": 1000000
    }
    
    max_value = max_values.get(unit, 100000)  # Default max
    return concentration <= max_value


def parse_storage_location(location: str) -> Dict[str, Optional[str]]:
    """
    Parse storage location string into components.
    
    Args:
        location: Storage location string (e.g., "Lab 1 - Freezer A1 - Rack 2 - Box 3")
        
    Returns:
        Dict[str, Optional[str]]: Parsed location components
    """
    parts = [part.strip() for part in location.split('-')]
    
    result: Dict[str, Optional[str]] = {
        'lab': None,
        'equipment': None,
        'rack': None,
        'box': None,
        'position': None
    }
    
    for i, part in enumerate(parts):
        if i == 0:
            result['lab'] = part
        elif i == 1:
            result['equipment'] = part
        elif i == 2:
            result['rack'] = part
        elif i == 3:
            result['box'] = part
        elif i == 4:
            result['position'] = part
    
    return result


def format_sample_name(prefix: str, number: int, suffix: str = "") -> str:
    """
    Format sample name with consistent numbering.
    
    Args:
        prefix: Sample name prefix
        number: Sample number
        suffix: Optional suffix
        
    Returns:
        str: Formatted sample name
    """
    formatted_number = f"{number:04d}"  # Zero-padded to 4 digits
    
    if suffix:
        return f"{prefix}-{formatted_number}-{suffix}"
    else:
        return f"{prefix}-{formatted_number}"


def validate_sample_type(sample_type: str) -> bool:
    """
    Validate sample type against known types.
    
    Args:
        sample_type: Sample type to validate
        
    Returns:
        bool: True if valid sample type
    """
    valid_types = {
        "Blood", "Serum", "Plasma", "Urine", "Saliva", "Tissue", "DNA", "RNA",
        "Protein", "Cell Culture", "Bacterial Culture", "Viral Culture",
        "Biopsy", "Swab", "Aspirate", "Lavage", "CSF", "Synovial Fluid",
        "Amniotic Fluid", "Pleural Fluid", "Ascites", "Other"
    }
    
    return sample_type in valid_types


def calculate_storage_utilization(current: int, capacity: int) -> float:
    """
    Calculate storage utilization percentage.
    
    Args:
        current: Current number of samples
        capacity: Total capacity
        
    Returns:
        float: Utilization percentage (0-100)
    """
    if capacity <= 0:
        return 0.0
    
    utilization = (current / capacity) * 100
    return min(utilization, 100.0)  # Cap at 100%


def get_file_extension(filename: str) -> str:
    """
    Get file extension from filename.
    
    Args:
        filename: Filename to extract extension from
        
    Returns:
        str: File extension (without dot)
    """
    return Path(filename).suffix.lstrip('.')


def is_valid_file_type(filename: str, allowed_types: List[str]) -> bool:
    """
    Check if file type is allowed.
    
    Args:
        filename: Filename to check
        allowed_types: List of allowed file extensions
        
    Returns:
        bool: True if file type is allowed
    """
    extension = get_file_extension(filename).lower()
    return extension in [ext.lower() for ext in allowed_types]


def get_current_timestamp() -> datetime:
    """
    Get current timestamp in UTC.
    
    Returns:
        datetime: Current UTC timestamp
    """
    return datetime.now(timezone.utc)


def format_timestamp(dt: datetime, format_str: str = "%Y-%m-%d %H:%M:%S UTC") -> str:
    """
    Format datetime as string.
    
    Args:
        dt: Datetime to format
        format_str: Format string
        
    Returns:
        str: Formatted datetime string
    """
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=timezone.utc)
    
    return dt.strftime(format_str)


def chunks(lst: List[Any], n: int) -> List[List[Any]]:
    """
    Create successive n-sized chunks from list.
    
    Args:
        lst: List to chunk
        n: Chunk size
        
    Returns:
        List[List[Any]]: List of chunks
    """
    result = []
    for i in range(0, len(lst), n):
        result.append(lst[i:i + n])
    return result


def flatten_dict(d: Dict[str, Any], parent_key: str = '', sep: str = '.') -> Dict[str, Any]:
    """
    Flatten nested dictionary.
    
    Args:
        d: Dictionary to flatten
        parent_key: Parent key prefix
        sep: Separator for nested keys
        
    Returns:
        Dict[str, Any]: Flattened dictionary
    """
    items = []
    for k, v in d.items():
        new_key = f"{parent_key}{sep}{k}" if parent_key else k
        if isinstance(v, dict):
            items.extend(flatten_dict(v, new_key, sep=sep).items())
        else:
            items.append((new_key, v))
    return dict(items)


def deep_merge(dict1: Dict[str, Any], dict2: Dict[str, Any]) -> Dict[str, Any]:
    """
    Deep merge two dictionaries.
    
    Args:
        dict1: First dictionary
        dict2: Second dictionary
        
    Returns:
        Dict[str, Any]: Merged dictionary
    """
    result = dict1.copy()
    
    for key, value in dict2.items():
        if key in result and isinstance(result[key], dict) and isinstance(value, dict):
            result[key] = deep_merge(result[key], value)
        else:
            result[key] = value
    
    return result