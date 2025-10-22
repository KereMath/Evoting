#!/bin/bash

# E-Voting System - Full Rebuild Script
# This script ensures all services are rebuilt from scratch with latest code

echo "🧹 Cleaning up all Docker resources..."

# Stop all containers
docker-compose down

# Remove all project-related containers
docker ps -aq --filter "name=evoting" | xargs -r docker rm -f

# Remove all project-related images (force fresh rebuild)
docker images | grep -E "evoting|e-votingapp" | awk '{print $3}' | xargs -r docker rmi -f

# Remove project-related networks
docker network ls --filter "name=election" --format "{{.Name}}" | xargs -r -n1 docker network rm
docker network ls --filter "name=evoting" --format "{{.Name}}" | xargs -r -n1 docker network rm

# Remove project-related volumes (optional - preserves database data)
# docker volume ls --filter "name=evoting" --format "{{.Name}}" | xargs -r docker volume rm

echo "✅ Cleanup complete!"
echo ""
echo "🔨 Building all services from scratch (no cache)..."

# Build all services without cache
docker-compose build --no-cache --parallel

echo ""
echo "✅ All services rebuilt successfully!"
echo ""
echo "🚀 Starting all services..."

# Start all services
docker-compose up -d

echo ""
echo "⏳ Waiting for services to be ready..."
sleep 5

# Check service status
docker-compose ps

echo ""
echo "✅ System is ready!"
echo ""
echo "📊 Access points:"
echo "   - Frontend: http://localhost:3000"
echo "   - Backend API: http://localhost:8080"
echo "   - PostgreSQL: localhost:5432"
echo ""
