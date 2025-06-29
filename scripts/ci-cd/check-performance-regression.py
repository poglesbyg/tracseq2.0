#!/usr/bin/env python3
"""Performance regression detection for TracSeq 2.0"""

import json
import sys
import argparse
from typing import Dict, Any

def load_test_results(file_path: str) -> Dict[str, Any]:
    try:
        with open(file_path, 'r') as f:
            return json.load(f)
    except FileNotFoundError:
        print(f"❌ File not found: {file_path}")
        sys.exit(1)
    except json.JSONDecodeError:
        print(f"❌ Invalid JSON in file: {file_path}")
        sys.exit(1)

def check_regression(current: Dict[str, Any], baseline: Dict[str, Any], threshold: float = 0.1) -> bool:
    """Check if current results show performance regression"""
    
    metrics = [
        'http_req_duration.avg',
        'http_req_duration.p95',
        'http_req_duration.p99',
        'http_req_failed.rate'
    ]
    
    regressions = []
    improvements = []
    
    for metric in metrics:
        current_metrics = current.get('metrics', {})
        baseline_metrics = baseline.get('metrics', {})
        
        if metric in current_metrics and metric in baseline_metrics:
            current_val = current_metrics[metric]
            baseline_val = baseline_metrics[metric]
            
            if baseline_val > 0:
                change = (current_val - baseline_val) / baseline_val
                
                if abs(change) > threshold:
                    if change > 0:
                        if metric == 'http_req_failed.rate':
                            regressions.append(f"{metric}: {change*100:.1f}% increase (worse)")
                        else:
                            regressions.append(f"{metric}: {change*100:.1f}% increase")
                    else:
                        improvements.append(f"{metric}: {abs(change)*100:.1f}% improvement")
    
    print(f"🔍 Performance Regression Analysis")
    print(f"📊 Baseline: {baseline.get('timestamp', 'Unknown')}")
    print(f"📊 Current:  {current.get('timestamp', 'Unknown')}")
    print()
    
    if regressions:
        print("❌ Performance Regressions Detected:")
        for regression in regressions:
            print(f"   - {regression}")
        print()
    
    if improvements:
        print("✅ Performance Improvements:")
        for improvement in improvements:
            print(f"   - {improvement}")
        print()
    
    if not regressions and not improvements:
        print("✅ No significant performance changes detected")
    
    return len(regressions) > 0

def main():
    parser = argparse.ArgumentParser(description='Check for performance regressions')
    parser.add_argument('--current', required=True, help='Current test results JSON file')
    parser.add_argument('--baseline', required=True, help='Baseline test results JSON file')
    parser.add_argument('--threshold', type=float, default=0.1, help='Regression threshold (default: 0.1 = 10%)')
    
    args = parser.parse_args()
    
    current = load_test_results(args.current)
    baseline = load_test_results(args.baseline)
    
    has_regression = check_regression(current, baseline, args.threshold)
    
    if has_regression:
        print("❌ Performance regression detected!")
        sys.exit(1)
    else:
        print("✅ No performance regression detected")
        sys.exit(0)

if __name__ == "__main__":
    main()
