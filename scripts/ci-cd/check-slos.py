#!/usr/bin/env python3
"""
TracSeq 2.0 SLO Compliance Checker
Verifies that Service Level Objectives are being met
"""

import argparse
import sys
import json
import time
from datetime import datetime, timedelta
from typing import Dict, List, Tuple, Any
import boto3
import requests

# Service Level Objectives
SLOS = {
    "availability": {
        "target": 99.9,  # 99.9% uptime
        "description": "Service availability"
    },
    "latency_p95": {
        "target": 500,   # 95th percentile under 500ms
        "description": "95th percentile latency"
    },
    "latency_p99": {
        "target": 1000,  # 99th percentile under 1000ms
        "description": "99th percentile latency"
    },
    "error_rate": {
        "target": 1.0,   # Less than 1% errors
        "description": "Error rate percentage"
    },
    "sample_processing_time": {
        "target": 5000,  # Sample processing under 5 seconds
        "description": "Time to process new samples"
    },
    "sequencing_queue_depth": {
        "target": 100,   # Queue depth under 100
        "description": "Sequencing queue backlog"
    }
}


class MetricsCollector:
    """Collects metrics from various sources"""
    
    def __init__(self, environment: str):
        self.environment = environment
        self.cloudwatch = boto3.client('cloudwatch')
        self.prometheus_url = self._get_prometheus_url()
    
    def _get_prometheus_url(self) -> str:
        """Get Prometheus URL based on environment"""
        # In production, this would come from service discovery
        return f"http://prometheus.{self.environment}.tracseq.io"
    
    def get_availability(self, time_range: timedelta) -> float:
        """Calculate service availability percentage"""
        end_time = datetime.utcnow()
        start_time = end_time - time_range
        
        # Query CloudWatch for ALB metrics
        response = self.cloudwatch.get_metric_statistics(
            Namespace='AWS/ApplicationELB',
            MetricName='TargetResponseTime',
            Dimensions=[
                {'Name': 'LoadBalancer', 'Value': f'tracseq-{self.environment}'}
            ],
            StartTime=start_time,
            EndTime=end_time,
            Period=300,
            Statistics=['SampleCount']
        )
        
        total_periods = len(response['Datapoints'])
        successful_periods = sum(1 for dp in response['Datapoints'] if dp['SampleCount'] > 0)
        
        return (successful_periods / total_periods * 100) if total_periods > 0 else 0
    
    def get_latency_percentiles(self, time_range: timedelta) -> Dict[str, float]:
        """Get latency percentiles from Prometheus"""
        query_p95 = 'histogram_quantile(0.95, http_request_duration_seconds_bucket)'
        query_p99 = 'histogram_quantile(0.99, http_request_duration_seconds_bucket)'
        
        try:
            # Query Prometheus
            p95_response = requests.get(
                f"{self.prometheus_url}/api/v1/query",
                params={'query': query_p95}
            )
            p99_response = requests.get(
                f"{self.prometheus_url}/api/v1/query",
                params={'query': query_p99}
            )
            
            p95_data = p95_response.json()
            p99_data = p99_response.json()
            
            # Extract values (convert to milliseconds)
            p95_value = float(p95_data['data']['result'][0]['value'][1]) * 1000
            p99_value = float(p99_data['data']['result'][0]['value'][1]) * 1000
            
            return {
                "p95": p95_value,
                "p99": p99_value
            }
        except Exception as e:
            print(f"Error querying Prometheus: {e}")
            return {"p95": 0, "p99": 0}
    
    def get_error_rate(self, time_range: timedelta) -> float:
        """Calculate error rate percentage"""
        query = 'rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m]) * 100'
        
        try:
            response = requests.get(
                f"{self.prometheus_url}/api/v1/query",
                params={'query': query}
            )
            
            data = response.json()
            if data['data']['result']:
                return float(data['data']['result'][0]['value'][1])
            return 0.0
        except Exception:
            return 0.0
    
    def get_custom_metrics(self) -> Dict[str, float]:
        """Get custom business metrics"""
        metrics = {}
        
        # Sample processing time
        sample_query = 'histogram_quantile(0.95, sample_processing_duration_seconds_bucket)'
        try:
            response = requests.get(
                f"{self.prometheus_url}/api/v1/query",
                params={'query': sample_query}
            )
            data = response.json()
            if data['data']['result']:
                metrics['sample_processing_time'] = float(data['data']['result'][0]['value'][1]) * 1000
        except Exception:
            metrics['sample_processing_time'] = 0
        
        # Sequencing queue depth
        queue_query = 'sequencing_queue_depth'
        try:
            response = requests.get(
                f"{self.prometheus_url}/api/v1/query",
                params={'query': queue_query}
            )
            data = response.json()
            if data['data']['result']:
                metrics['sequencing_queue_depth'] = float(data['data']['result'][0]['value'][1])
        except Exception:
            metrics['sequencing_queue_depth'] = 0
        
        return metrics


def check_slos(collector: MetricsCollector, time_range: timedelta) -> List[Tuple[str, float, float, bool]]:
    """Check all SLOs and return results"""
    results = []
    
    # Get metrics
    availability = collector.get_availability(time_range)
    latencies = collector.get_latency_percentiles(time_range)
    error_rate = collector.get_error_rate(time_range)
    custom_metrics = collector.get_custom_metrics()
    
    # Check availability
    results.append((
        "availability",
        availability,
        SLOS["availability"]["target"],
        availability >= SLOS["availability"]["target"]
    ))
    
    # Check latency P95
    results.append((
        "latency_p95",
        latencies["p95"],
        SLOS["latency_p95"]["target"],
        latencies["p95"] <= SLOS["latency_p95"]["target"]
    ))
    
    # Check latency P99
    results.append((
        "latency_p99",
        latencies["p99"],
        SLOS["latency_p99"]["target"],
        latencies["p99"] <= SLOS["latency_p99"]["target"]
    ))
    
    # Check error rate
    results.append((
        "error_rate",
        error_rate,
        SLOS["error_rate"]["target"],
        error_rate <= SLOS["error_rate"]["target"]
    ))
    
    # Check custom metrics
    for metric_name, metric_value in custom_metrics.items():
        if metric_name in SLOS:
            results.append((
                metric_name,
                metric_value,
                SLOS[metric_name]["target"],
                metric_value <= SLOS[metric_name]["target"]
            ))
    
    return results


def generate_slo_report(results: List[Tuple[str, float, float, bool]]) -> str:
    """Generate SLO compliance report"""
    report = ["TracSeq 2.0 SLO Compliance Report", "=" * 50, ""]
    report.append(f"Timestamp: {datetime.utcnow().isoformat()}")
    report.append("")
    
    violations = []
    
    for slo_name, current_value, target_value, compliant in results:
        status = "✓ PASS" if compliant else "✗ FAIL"
        description = SLOS[slo_name]["description"]
        
        report.append(f"{description} ({slo_name}):")
        report.append(f"  Current: {current_value:.2f}")
        report.append(f"  Target:  {target_value:.2f}")
        report.append(f"  Status:  {status}")
        report.append("")
        
        if not compliant:
            violations.append(slo_name)
    
    if violations:
        report.append("SLO VIOLATIONS:")
        for slo in violations:
            report.append(f"  - {SLOS[slo]['description']}")
        report.append("")
        report.append("⚠️  SLO compliance check FAILED!")
    else:
        report.append("✅ All SLOs are being met!")
    
    # Calculate error budget
    availability_result = next((r for r in results if r[0] == "availability"), None)
    if availability_result:
        current_availability = availability_result[1]
        target_availability = availability_result[2]
        error_budget_total = 100 - target_availability
        error_budget_used = 100 - current_availability
        error_budget_remaining = error_budget_total - error_budget_used
        
        report.append("")
        report.append("Error Budget:")
        report.append(f"  Total:     {error_budget_total:.3f}%")
        report.append(f"  Used:      {error_budget_used:.3f}%")
        report.append(f"  Remaining: {error_budget_remaining:.3f}%")
    
    return "\n".join(report)


def send_alert(report: str, webhook_url: str):
    """Send alert via webhook if SLOs are violated"""
    if "FAILED" in report:
        payload = {
            "text": "🚨 SLO Violation Alert",
            "blocks": [
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": f"```{report}```"
                    }
                }
            ]
        }
        
        try:
            response = requests.post(webhook_url, json=payload)
            response.raise_for_status()
        except Exception as e:
            print(f"Failed to send alert: {e}")


def main():
    parser = argparse.ArgumentParser(description="Check SLO compliance")
    parser.add_argument("--environment", required=True, help="Environment name")
    parser.add_argument("--time-range", default="1h", help="Time range (e.g., 1h, 24h, 7d)")
    parser.add_argument("--output", help="Output file for report")
    parser.add_argument("--webhook", help="Webhook URL for alerts")
    
    args = parser.parse_args()
    
    # Parse time range
    time_map = {
        'h': 'hours',
        'd': 'days',
        'w': 'weeks'
    }
    
    value = int(args.time_range[:-1])
    unit = args.time_range[-1]
    time_range = timedelta(**{time_map.get(unit, 'hours'): value})
    
    # Collect metrics
    collector = MetricsCollector(args.environment)
    results = check_slos(collector, time_range)
    
    # Generate report
    report = generate_slo_report(results)
    print(report)
    
    # Save report if requested
    if args.output:
        with open(args.output, 'w') as f:
            f.write(report)
    
    # Send alert if webhook provided
    if args.webhook:
        send_alert(report, args.webhook)
    
    # Exit with error if any SLO violations
    if any(not compliant for _, _, _, compliant in results):
        sys.exit(1)


if __name__ == "__main__":
    main()