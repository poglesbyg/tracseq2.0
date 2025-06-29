#!/bin/bash

echo "🏥 TracSeq 2.0 - Service Health Check"
echo "====================================="

check_service() {
    local service_name="$1"
    local url="$2"
    local timeout="${3:-5}"
    
    echo -n "Checking $service_name... "
    
    if curl -f -s --max-time "$timeout" "$url" > /dev/null 2>&1; then
        echo "✅ Healthy"
        return 0
    else
        echo "❌ Unhealthy"
        return 1
    fi
}

check_port() {
    local service_name="$1" 
    local port="$2"
    
    echo -n "Checking $service_name... "
    
    if nc -z localhost "$port" 2>/dev/null; then
        echo "✅ Available"
        return 0
    else
        echo "❌ Unavailable"
        return 1
    fi
}

healthy_count=0
total_count=0

echo "🔍 Checking Core Services:"

# Core services
if check_service "Lab Manager" "http://localhost:3000/health"; then ((healthy_count++)); fi; ((total_count++))
if check_service "API Gateway" "http://localhost:8089/health"; then ((healthy_count++)); fi; ((total_count++))
if check_service "RAG Service" "http://localhost:8000/health"; then ((healthy_count++)); fi; ((total_count++))

echo ""
echo "🤖 Checking ML Platform Services:"

if check_service "MLflow" "http://localhost:5000/health"; then ((healthy_count++)); fi; ((total_count++))
if check_service "Jupyter Lab" "http://localhost:8888" 10; then ((healthy_count++)); fi; ((total_count++))
if check_service "TensorBoard" "http://localhost:6006"; then ((healthy_count++)); fi; ((total_count++))

echo ""
echo "🗄️ Checking Databases:"

if check_port "PostgreSQL (Lab Manager)" 5433; then ((healthy_count++)); fi; ((total_count++))
if check_port "PostgreSQL (ML Platform)" 5438; then ((healthy_count++)); fi; ((total_count++))
if check_port "Redis (ML Platform)" 6380; then ((healthy_count++)); fi; ((total_count++))

echo ""
echo "📊 Health Check Summary:"
echo "   - Healthy Services: $healthy_count / $total_count"
echo "   - Health Score: $(( healthy_count * 100 / total_count ))%"

if [ "$healthy_count" -eq "$total_count" ]; then
    echo "   - Overall Status: ✅ All services healthy"
    exit 0
elif [ "$healthy_count" -gt $(( total_count / 2 )) ]; then
    echo "   - Overall Status: ⚠️  Some services unhealthy"
    exit 1
else
    echo "   - Overall Status: ❌ Multiple services unhealthy"
    exit 2
fi
