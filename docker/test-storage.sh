#!/bin/bash

echo "Testing storage service binary..."

# Extract the binary from the Docker image
docker create --name temp-storage docker-storage-service
docker cp temp-storage:/app/enhanced_storage_service ./test-binary
docker rm temp-storage

echo "Binary extracted. Checking file type..."
file ./test-binary

echo "Checking binary contents with strings..."
strings ./test-binary | grep -E "(main|storage|tracseq)" | head -20

echo "Checking if it's actually a script..."
head -10 ./test-binary

# Clean up
rm -f ./test-binary

echo "Done." 