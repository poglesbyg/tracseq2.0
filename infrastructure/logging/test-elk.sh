#!/bin/bash

# Test script for ELK stack functionality

echo "🧪 Testing ELK Stack Functionality"
echo "================================="

# Test 1: Check if services are running
echo "1. Checking service health..."
curl -s http://localhost:9200/_cluster/health | jq '.status' && echo "✅ Elasticsearch is healthy" || echo "❌ Elasticsearch is not healthy"
curl -s http://localhost:5601/api/status | jq '.status.overall.level' && echo "✅ Kibana is healthy" || echo "❌ Kibana is not healthy"
curl -s http://localhost:9600/_node/stats | jq '.pipeline.main.plugins.inputs' && echo "✅ Logstash is healthy" || echo "❌ Logstash is not healthy"

echo
echo "2. Testing log ingestion..."

# Test 2: Send a test log via TCP
echo "Sending test log via TCP..."
echo '{"timestamp":"2025-07-14T19:05:00.000Z","level":"INFO","logger":"test_service","message":"ELK stack test log","request_id":"test-001","processing_time_ms":100}' | nc localhost 5000

# Wait for processing
sleep 3

# Test 3: Check if log was indexed
echo "3. Checking if log was indexed..."
curl -s "http://localhost:9200/_cat/indices?v" | grep tracseq && echo "✅ Index created" || echo "❌ No index found"

# Test 4: Search for the test log
echo "4. Searching for test log..."
curl -s "http://localhost:9200/tracseq-logs-*/_search?q=test_service&pretty" | jq '.hits.total.value' && echo "✅ Log found" || echo "❌ Log not found"

echo
echo "5. Current indices:"
curl -s "http://localhost:9200/_cat/indices?v"

echo
echo "6. Sample log search:"
curl -s "http://localhost:9200/tracseq-logs-*/_search?pretty&size=1" | jq '.hits.hits[0]._source'

echo
echo "🎉 ELK Stack test completed!" 