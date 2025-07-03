#!/bin/bash

# LIMS Service Health Monitor
# Shows real-time status of all LIMS services

echo "========================================="
echo "  TracSeq 2.0 Service Health Monitor"
echo "========================================="
echo ""

# Query Prometheus for service health
HEALTH_DATA=$(curl -s 'http://localhost:9090/api/v1/query?query=probe_success%7Bjob%3D%22blackbox-http%22%2C%20instance%3D~%22.*lims-.*%22%7D')

# Parse and display results
echo "$HEALTH_DATA" | jq -r '.data.result[] | 
  "\(.metric.instance): \(if .value[1] == "1" then "✅ UP" else "❌ DOWN" end)"' | 
  sed 's|http://lims-||g' | 
  sed 's|/health||g' | 
  sort | 
  column -t -s ':'

echo ""
echo "Summary:"
TOTAL=$(echo "$HEALTH_DATA" | jq '.data.result | length')
UP=$(echo "$HEALTH_DATA" | jq '[.data.result[] | select(.value[1] == "1")] | length')
DOWN=$((TOTAL - UP))

echo "✅ Services UP: $UP"
echo "❌ Services DOWN: $DOWN"
echo "📊 Total: $TOTAL"
echo ""
echo "Last checked: $(date)"
echo ""

# Show which specific services are down
if [ $DOWN -gt 0 ]; then
    echo "⚠️  Services that need attention:"
    echo "$HEALTH_DATA" | jq -r '.data.result[] | 
      select(.value[1] == "0") | 
      .metric.instance' | 
      sed 's|http://||g' | 
      sed 's|/health||g'
fi 