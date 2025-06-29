#!/usr/bin/env python3
"""
TracSeq 2.0 Performance Regression Check
Compares current performance test results against baseline
"""

import json
import argparse
import sys
import requests
from typing import Dict, Any, List, Tuple

# Performance thresholds (percentage allowed degradation)
THRESHOLDS = {
    "http_req_duration": {
        "p(95)": 0.10,  # 10% degradation allowed
        "p(99)": 0.15,  # 15% degradation allowed
        "avg": 0.20,    # 20% degradation allowed
    },
    "http_req_failed": {
        "rate": 0.02,   # 2% increase in error rate allowed
    },
    "sample_creation_duration": {
        "p(95)": 0.15,  # 15% degradation allowed
    },
    "auth_duration": {
        "p(95)": 0.10,  # 10% degradation allowed
    },
    "search_duration": {
        "p(95)": 0.10,  # 10% degradation allowed
    }
}


def load_test_results(file_path: str) -> Dict[str, Any]:
    """Load test results from JSON file"""
    try:
        with open(file_path, 'r') as f:
            return json.load(f)
    except Exception as e:
        print(f"Error loading test results from {file_path}: {e}")
        sys.exit(1)


def fetch_baseline(url: str) -> Dict[str, Any]:
    """Fetch baseline results from URL"""
    try:
        response = requests.get(url, timeout=30)
        response.raise_for_status()
        return response.json()
    except Exception as e:
        print(f"Error fetching baseline from {url}: {e}")
        sys.exit(1)


def extract_metric_value(metrics: Dict[str, Any], metric_path: str) -> float:
    """Extract metric value from nested dictionary"""
    parts = metric_path.split('.')
    value = metrics
    
    for part in parts:
        if part in value:
            value = value[part]
        else:
            return None
    
    # Handle different value formats
    if isinstance(value, dict) and 'values' in value:
        return value['values']
    
    return value


def compare_metrics(current: Dict[str, Any], baseline: Dict[str, Any]) -> List[Tuple[str, float, float, float, bool]]:
    """Compare current metrics against baseline"""
    results = []
    
    for metric_name, sub_metrics in THRESHOLDS.items():
        for sub_metric, threshold in sub_metrics.items():
            # Build metric path
            metric_path = f"metrics.{metric_name}.{sub_metric}"
            
            # Get values
            current_value = extract_metric_value(current, metric_path)
            baseline_value = extract_metric_value(baseline, metric_path)
            
            if current_value is None or baseline_value is None:
                print(f"Warning: Could not find metric {metric_path}")
                continue
            
            # Calculate degradation
            if baseline_value > 0:
                degradation = (current_value - baseline_value) / baseline_value
            else:
                degradation = 0
            
            # Check if within threshold
            passed = degradation <= threshold
            
            results.append((
                f"{metric_name}.{sub_metric}",
                current_value,
                baseline_value,
                degradation,
                passed
            ))
    
    return results


def generate_report(results: List[Tuple[str, float, float, float, bool]]) -> str:
    """Generate performance report"""
    report = ["Performance Regression Analysis", "=" * 50, ""]
    
    failures = []
    
    for metric, current, baseline, degradation, passed in results:
        status = "PASS" if passed else "FAIL"
        
        report.append(f"{metric}:")
        report.append(f"  Current:  {current:.2f}")
        report.append(f"  Baseline: {baseline:.2f}")
        report.append(f"  Change:   {degradation*100:+.1f}%")
        report.append(f"  Status:   {status}")
        report.append("")
        
        if not passed:
            failures.append(metric)
    
    if failures:
        report.append("FAILED METRICS:")
        for metric in failures:
            report.append(f"  - {metric}")
        report.append("")
        report.append("Performance regression detected!")
    else:
        report.append("All metrics within acceptable thresholds.")
    
    return "\n".join(report)


def main():
    parser = argparse.ArgumentParser(description="Check for performance regressions")
    parser.add_argument("--current", required=True, help="Path to current test results")
    parser.add_argument("--baseline", required=True, help="URL or path to baseline results")
    parser.add_argument("--output", help="Output file for report")
    
    args = parser.parse_args()
    
    # Load results
    current_results = load_test_results(args.current)
    
    if args.baseline.startswith("http"):
        baseline_results = fetch_baseline(args.baseline)
    else:
        baseline_results = load_test_results(args.baseline)
    
    # Compare metrics
    results = compare_metrics(current_results, baseline_results)
    
    # Generate report
    report = generate_report(results)
    print(report)
    
    # Save report if requested
    if args.output:
        with open(args.output, 'w') as f:
            f.write(report)
    
    # Exit with error if any failures
    if any(not passed for _, _, _, _, passed in results):
        sys.exit(1)


if __name__ == "__main__":
    main()