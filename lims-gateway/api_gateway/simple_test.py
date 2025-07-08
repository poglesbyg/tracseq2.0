#!/usr/bin/env python3
"""
Simple test script for TracSeq API Gateway configuration validation
"""

import sys
import os

# Add the src directory to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'src'))

try:
    from api_gateway.core.config import get_config
    from api_gateway.unified_main import TracSeqAPIGateway
    
    print("✓ Configuration validation test")
    print("="*50)
    
    # Test configuration loading
    config = get_config()
    print(f"✓ Configuration loaded successfully")
    print(f"  - Version: {config.version}")
    print(f"  - Environment: {config.environment}")
    print(f"  - Host: {config.host}")
    print(f"  - Port: {config.port}")
    print(f"  - Services configured: {len(config.services)}")
    
    # Test service configuration
    print(f"\nConfigured Services:")
    for name, service in config.services.items():
        print(f"  - {name}: {service.name} -> {service.base_url}{service.path_prefix}")
    
    # Test gateway creation
    gateway = TracSeqAPIGateway(config)
    print(f"\n✓ Gateway instance created successfully")
    
    # Test FastAPI app creation
    try:
        app = gateway.create_app()
        print(f"✓ FastAPI application created successfully")
        print(f"  - Title: {app.title}")
        print(f"  - Version: {app.version}")
    except Exception as e:
        print(f"✗ FastAPI application creation failed: {e}")
        sys.exit(1)
    
    print(f"\n✓ All configuration tests passed!")
    print(f"\nTo start the gateway:")
    print(f"  cd /Users/paulgreenwood/Dev/tracseq2.0/lims-gateway/api_gateway")
    print(f"  python3 -m api_gateway.unified_main")
    
except Exception as e:
    print(f"✗ Configuration test failed: {e}")
    import traceback
    traceback.print_exc()
    sys.exit(1)