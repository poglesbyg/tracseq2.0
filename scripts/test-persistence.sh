#!/bin/bash

# Test script to verify data persistence across container restarts

echo "🔍 Testing Docker Volume Persistence for TracSeq 2.0"
echo "=================================================="

# Function to run a command in a container
run_in_container() {
    local service=$1
    local command=$2
    docker-compose -f docker-compose.development.yml exec -T $service sh -c "$command"
}

# Test 1: Database persistence
echo "📊 Testing PostgreSQL persistence..."
echo "Creating test table and inserting data..."
run_in_container postgres "psql -U postgres -d lab_manager -c \"CREATE TABLE IF NOT EXISTS test_persistence (id SERIAL PRIMARY KEY, message TEXT, created_at TIMESTAMP DEFAULT NOW());\""
run_in_container postgres "psql -U postgres -d lab_manager -c \"INSERT INTO test_persistence (message) VALUES ('Database persistence test');\""

# Test 2: Redis persistence
echo "📈 Testing Redis persistence..."
echo "Setting test key in Redis..."
run_in_container redis "redis-cli SET test_key 'Redis persistence test'"

# Test 3: Application storage persistence
echo "📁 Testing application storage persistence..."
echo "Creating test files in mounted volumes..."
run_in_container dashboard-service "echo 'Dashboard storage test' > /app/storage/dashboard_test.txt"
run_in_container samples-service "echo 'Samples storage test' > /app/storage/samples_test.txt"
run_in_container sequencing-service "echo 'Sequencing storage test' > /app/storage/sequencing_test.txt"
run_in_container spreadsheet-service "echo 'Spreadsheet template test' > /app/templates/spreadsheet_test.txt"

# Test 4: Log persistence
echo "📝 Testing log persistence..."
run_in_container dashboard-service "echo 'Dashboard log test' > /app/logs/dashboard_test.log"
run_in_container frontend-proxy "echo 'Frontend proxy log test' > /app/logs/proxy_test.log"

echo ""
echo "🔄 Restarting all services to test persistence..."
docker-compose -f docker-compose.development.yml down
sleep 2
docker-compose -f docker-compose.development.yml up -d

echo "⏳ Waiting for services to start..."
sleep 15

echo ""
echo "🔍 Verifying data persistence after restart..."
echo "============================================="

# Verify database persistence
echo "📊 Checking PostgreSQL data..."
db_result=$(run_in_container postgres "psql -U postgres -d lab_manager -t -c \"SELECT message FROM test_persistence WHERE message = 'Database persistence test';\""  2>/dev/null | xargs)
if [ "$db_result" = "Database persistence test" ]; then
    echo "✅ PostgreSQL data persisted successfully"
else
    echo "❌ PostgreSQL data persistence failed"
fi

# Verify Redis persistence
echo "📈 Checking Redis data..."
redis_result=$(run_in_container redis "redis-cli GET test_key" 2>/dev/null | xargs)
if [ "$redis_result" = "Redis persistence test" ]; then
    echo "✅ Redis data persisted successfully"
else
    echo "❌ Redis data persistence failed"
fi

# Verify application storage persistence
echo "📁 Checking application storage..."
dashboard_result=$(run_in_container dashboard-service "cat /app/storage/dashboard_test.txt" 2>/dev/null | xargs)
samples_result=$(run_in_container samples-service "cat /app/storage/samples_test.txt" 2>/dev/null | xargs)
sequencing_result=$(run_in_container sequencing-service "cat /app/storage/sequencing_test.txt" 2>/dev/null | xargs)
spreadsheet_result=$(run_in_container spreadsheet-service "cat /app/templates/spreadsheet_test.txt" 2>/dev/null | xargs)

if [ "$dashboard_result" = "Dashboard storage test" ] && 
   [ "$samples_result" = "Samples storage test" ] && 
   [ "$sequencing_result" = "Sequencing storage test" ] && 
   [ "$spreadsheet_result" = "Spreadsheet template test" ]; then
    echo "✅ Application storage persisted successfully"
else
    echo "❌ Application storage persistence failed"
fi

# Verify log persistence
echo "📝 Checking log persistence..."
dashboard_log=$(run_in_container dashboard-service "cat /app/logs/dashboard_test.log" 2>/dev/null | xargs)
proxy_log=$(run_in_container frontend-proxy "cat /app/logs/proxy_test.log" 2>/dev/null | xargs)

if [ "$dashboard_log" = "Dashboard log test" ] && 
   [ "$proxy_log" = "Frontend proxy log test" ]; then
    echo "✅ Log files persisted successfully"
else
    echo "❌ Log persistence failed"
fi

echo ""
echo "🧹 Cleaning up test data..."
run_in_container postgres "psql -U postgres -d lab_manager -c \"DROP TABLE IF EXISTS test_persistence;\""
run_in_container redis "redis-cli DEL test_key"
run_in_container dashboard-service "rm -f /app/storage/dashboard_test.txt /app/logs/dashboard_test.log"
run_in_container samples-service "rm -f /app/storage/samples_test.txt"
run_in_container sequencing-service "rm -f /app/storage/sequencing_test.txt"
run_in_container spreadsheet-service "rm -f /app/templates/spreadsheet_test.txt"
run_in_container frontend-proxy "rm -f /app/logs/proxy_test.log"

echo ""
echo "📋 Volume Information:"
echo "====================="
docker volume ls | grep tracseq20

echo ""
echo "✅ Persistence test completed!"
echo "All data should persist across container restarts." 