#!/bin/bash

# Demo script for sample editing functionality
# This script demonstrates the new sample editing API endpoints

echo "🧪 Lab Manager - Sample Editing Demo"
echo "===================================="

# Get sample ID from existing samples
SAMPLE_ID=$(curl -s http://localhost:3000/api/samples | jq -r '.[0].id')

if [ "$SAMPLE_ID" = "null" ] || [ -z "$SAMPLE_ID" ]; then
    echo "❌ No samples found. Please create a sample first."
    exit 1
fi

echo "📋 Sample ID: $SAMPLE_ID"
echo ""

echo "1️⃣  Current sample details:"
echo "----------------------------"
curl -s http://localhost:3000/api/samples/$SAMPLE_ID | jq .
echo ""

echo "2️⃣  Updating sample name and location..."
echo "---------------------------------------"
curl -X PUT http://localhost:3000/api/samples/$SAMPLE_ID \
  -H "Content-Type: application/json" \
  -d '{"name": "Edited Sample", "location": "Storage Room B"}' | jq .
echo ""

echo "3️⃣  Updating sample status to InStorage..."
echo "------------------------------------------"
curl -X PUT http://localhost:3000/api/samples/$SAMPLE_ID \
  -H "Content-Type: application/json" \
  -d '{"status": "InStorage"}' | jq .
echo ""

echo "4️⃣  Final sample state:"
echo "----------------------"
curl -s http://localhost:3000/api/samples/$SAMPLE_ID | jq .
echo ""

echo "✅ Demo complete!"
echo ""
echo "💡 Frontend Features:"
echo "   - Visit http://localhost:5173/samples"
echo "   - Click 'Edit' button on any sample"
echo "   - Modify name, barcode, location, or status"
echo "   - Changes are saved in real-time"
echo "   - Dashboard stats update automatically"
echo ""
echo "🔧 API Endpoints:"
echo "   GET    /api/samples/:id    - Get single sample"
echo "   PUT    /api/samples/:id    - Update sample"
echo "   POST   /api/samples        - Create sample"
echo "   GET    /api/samples        - List all samples" 
