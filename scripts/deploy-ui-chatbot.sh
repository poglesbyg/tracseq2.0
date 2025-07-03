#!/bin/bash

# Deploy UI with ChatBot changes to Docker

echo "🚀 Building and deploying TracSeq UI with enhanced ChatBot..."

# Navigate to docker directory
cd docker || exit 1

# Stop existing frontend container
echo "🛑 Stopping existing frontend container..."
docker-compose stop frontend

# Remove existing frontend container to ensure fresh build
echo "🗑️ Removing existing frontend container..."
docker-compose rm -f frontend

# Build the frontend with the new changes
echo "🔨 Building frontend with ChatBot enhancements..."
docker-compose build frontend

# Start the frontend service
echo "▶️ Starting frontend service..."
docker-compose up -d frontend

# Wait for service to be ready
echo "⏳ Waiting for frontend to be ready..."
sleep 5

# Check if frontend is running
if docker-compose ps | grep -q "lims-frontend.*Up"; then
    echo "✅ Frontend deployed successfully!"
    echo "🌐 Access the UI at: http://localhost:3000"
    echo "💬 ChatBot is now available in the Desktop interface!"
else
    echo "❌ Frontend deployment failed!"
    echo "Check logs with: docker-compose logs frontend"
    exit 1
fi

# Show logs
echo ""
echo "📋 Recent frontend logs:"
docker-compose logs --tail=20 frontend

echo ""
echo "🎉 Deployment complete! The enhanced ChatBot is now available." 