#!/usr/bin/env python3
import subprocess
import sys

print("Debugging storage service issue...")

# Check Docker logs
try:
    result = subprocess.run(['docker', 'logs', 'lims-storage'], capture_output=True, text=True)
    print(f"Docker logs stdout: {repr(result.stdout)}")
    print(f"Docker logs stderr: {repr(result.stderr)}")
except Exception as e:
    print(f"Error getting logs: {e}")

# Check container status
try:
    result = subprocess.run(['docker', 'ps', '-a', '--filter', 'name=lims-storage', '--format', 'table {{.Names}}\t{{.Status}}'], capture_output=True, text=True)
    print(f"\nContainer status:\n{result.stdout}")
except Exception as e:
    print(f"Error checking status: {e}")

# Try to run the binary directly
print("\nTrying to run binary in container...")
try:
    result = subprocess.run([
        'docker', 'run', '--rm', 
        '--network', 'docker_lims-network',
        '-e', 'STORAGE_DATABASE_URL=postgres://postgres:postgres@postgres:5432/lims_db',
        '-e', 'RUST_LOG=trace',
        'docker-storage-service',
        'sh', '-c', 'echo "Starting..." && ./enhanced_storage_service; echo "Exit code: $?"'
    ], capture_output=True, text=True)
    print(f"Run output: {repr(result.stdout)}")
    print(f"Run error: {repr(result.stderr)}")
except Exception as e:
    print(f"Error running: {e}") 