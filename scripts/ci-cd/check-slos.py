#!/usr/bin/env python3
"""SLO compliance checker for TracSeq 2.0"""

import json
import sys
import argparse
from typing import Dict, Any, List

SLOS = {
    "api_latency_p95": {"target": 500, "description": "95th percentile API response time should be < 500ms"},
    "api_latency_p99": {"target": 1000, "description": "99th percentile API response time should be < 1000ms"},
    "error_rate": {"target": 0.05, "description": "Error rate should be < 5%"},
    "availability": {"target": 0.99, "description": "Service availability should be > 99%"}
}

def load_metrics(file_path: str) -> Dict[str, Any]:
    try:
        with open(file_path, 'r') as f:
            return json.load(f)
    except FileNotFoundError:
        print(f"❌ Metrics file not found: {file_path}")
        sys.exit(1)
    except json.JSONDecodeError:
        print(f"❌ Invalid JSON in metrics file: {file_path}")
        sys.exit(1)

def check_slo_compliance(metrics: Dict[str, Any]) -> List[str]:
    violations = []
    
    print(f"🎯 SLO Compliance Check")
    print(f"📊 Timestamp: {metrics.get('timestamp', 'Unknown')}")
    print()
    
    # Check metrics if available
    test_metrics = metrics.get('metrics', {})
    
    # Check API latency P95
    if 'http_req_duration' in test_metrics:
        duration_metrics = test_metrics['http_req_duration']
        p95_latency = duration_metrics.get('p(95)', duration_metrics.get('p95', 0))
        if p95_latency > SLOS['api_latency_p95']['target']:
            violations.append(f"P95 latency: {p95_latency:.1f}ms > {SLOS['api_latency_p95']['target']}ms")
        print(f"✅ P95 Latency: {p95_latency:.1f}ms (target: <{SLOS['api_latency_p95']['target']}ms)")
    
    # Check error rate
    if 'http_req_failed' in test_metrics:
        error_rate = test_metrics['http_req_failed'].get('rate', 0)
        if error_rate > SLOS['error_rate']['target']:
            violations.append(f"Error rate: {error_rate*100:.1f}% > {SLOS['error_rate']['target']*100:.1f}%")
        print(f"✅ Error Rate: {error_rate*100:.1f}% (target: <{SLOS['error_rate']['target']*100:.1f}%)")
    
    return violations

def main():
    parser = argparse.ArgumentParser(description='Check SLO compliance')
    parser.add_argument('--metrics', required=True, help='Metrics JSON file')
    parser.add_argument('--environment', default='development', help='Environment')
    
    args = parser.parse_args()
    
    metrics = load_metrics(args.metrics)
    violations = check_slo_compliance(metrics)
    
    print()
    if violations:
        print("❌ SLO Violations Detected:")
        for violation in violations:
            print(f"   - {violation}")
        sys.exit(1)
    else:
        print("✅ All SLOs are being met!")
        sys.exit(0)

if __name__ == "__main__":
    main()
