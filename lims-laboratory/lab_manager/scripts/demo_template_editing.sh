#!/bin/bash

# Demo script for template editing functionality
# This script demonstrates the new template editing API endpoints

echo "📝 Lab Manager - Template Editing Demo"
echo "======================================"

# Get template ID from existing templates
TEMPLATE_ID=$(curl -s http://localhost:3000/api/templates | jq -r '.[0].id')

if [ "$TEMPLATE_ID" = "null" ] || [ -z "$TEMPLATE_ID" ]; then
    echo "❌ No templates found. Please upload a template first."
    exit 1
fi

echo "📋 Template ID: $TEMPLATE_ID"
echo ""

echo "1️⃣  Current template details:"
echo "----------------------------"
curl -s http://localhost:3000/api/templates/$TEMPLATE_ID | jq .
echo ""

echo "2️⃣  Updating template name and description..."
echo "--------------------------------------------"
curl -X PUT http://localhost:3000/api/templates/$TEMPLATE_ID \
  -H "Content-Type: application/json" \
  -d '{"name": "Enhanced Lab Template", "description": "Updated via template editing API with comprehensive metadata"}' | jq .
echo ""

echo "3️⃣  Updating only the description..."
echo "-----------------------------------"
curl -X PUT http://localhost:3000/api/templates/$TEMPLATE_ID \
  -H "Content-Type: application/json" \
  -d '{"description": "Further refined template description with editing capabilities"}' | jq .
echo ""

echo "4️⃣  Final template state:"
echo "------------------------"
curl -s http://localhost:3000/api/templates/$TEMPLATE_ID | jq .
echo ""

echo "✅ Demo complete!"
echo ""
echo "💡 Frontend Features:"
echo "   - Visit http://localhost:5173/templates"
echo "   - Click 'Edit' button on any template"
echo "   - Modify name and description"
echo "   - Changes are saved in real-time"
echo "   - Dashboard stats update automatically"
echo ""
echo "🔧 API Endpoints:"
echo "   GET    /api/templates/:id     - Get single template"
echo "   PUT    /api/templates/:id     - Update template"
echo "   POST   /api/templates/upload  - Upload template"
echo "   GET    /api/templates         - List all templates" 
