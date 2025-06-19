"""
Load Balancer Service for TracSeq API Gateway

Implements load balancing algorithms and instance management.
"""

import random
from typing import Dict, List, Optional
from collections import defaultdict
import threading

import structlog

from api_gateway.core.config import TracSeqAPIGatewayConfig, ServiceEndpoint

logger = structlog.get_logger(__name__)


class LoadBalancerService:
    """Service for load balancing requests across service instances."""
    
    def __init__(self, config: TracSeqAPIGatewayConfig):
        self.config = config
        self.round_robin_counters: Dict[str, int] = defaultdict(int)
        self.service_instances: Dict[str, List[ServiceEndpoint]] = {}
        self.lock = threading.Lock()
        
        # Initialize with single instances (expandable for multiple instances)
        for service_name, service_config in config.services.items():
            self.service_instances[service_name] = [service_config]
    
    def add_service_instance(self, service_name: str, instance: ServiceEndpoint):
        """Add a new instance for a service."""
        with self.lock:
            if service_name not in self.service_instances:
                self.service_instances[service_name] = []
            self.service_instances[service_name].append(instance)
        
        logger.info("Service instance added",
                   service=service_name,
                   instance=instance.base_url)
    
    def remove_service_instance(self, service_name: str, instance_url: str):
        """Remove a service instance."""
        with self.lock:
            if service_name in self.service_instances:
                self.service_instances[service_name] = [
                    inst for inst in self.service_instances[service_name]
                    if inst.base_url != instance_url
                ]
        
        logger.info("Service instance removed",
                   service=service_name,
                   instance=instance_url)
    
    def get_next_instance(self, service_name: str) -> Optional[ServiceEndpoint]:
        """Get the next instance using the configured load balancing algorithm."""
        
        if service_name not in self.service_instances:
            logger.warning("Service not found", service=service_name)
            return None
        
        instances = self.service_instances[service_name]
        if not instances:
            logger.warning("No instances available", service=service_name)
            return None
        
        # Use configured algorithm
        algorithm = self.config.load_balancer.algorithm
        
        if algorithm == "round_robin":
            return self._round_robin_select(service_name, instances)
        elif algorithm == "weighted_round_robin":
            return self._weighted_round_robin_select(service_name, instances)
        elif algorithm == "least_connections":
            return self._least_connections_select(service_name, instances)
        else:
            # Default to round robin
            return self._round_robin_select(service_name, instances)
    
    def _round_robin_select(self, service_name: str, instances: List[ServiceEndpoint]) -> ServiceEndpoint:
        """Select instance using round-robin algorithm."""
        with self.lock:
            counter = self.round_robin_counters[service_name]
            selected_instance = instances[counter % len(instances)]
            self.round_robin_counters[service_name] = (counter + 1) % len(instances)
            return selected_instance
    
    def _weighted_round_robin_select(self, service_name: str, instances: List[ServiceEndpoint]) -> ServiceEndpoint:
        """Select instance using weighted round-robin algorithm."""
        
        # Create weighted list based on load_balancer_weight
        weighted_instances = []
        for instance in instances:
            weighted_instances.extend([instance] * instance.load_balancer_weight)
        
        if not weighted_instances:
            return instances[0]  # Fallback
        
        with self.lock:
            counter = self.round_robin_counters[service_name]
            selected_instance = weighted_instances[counter % len(weighted_instances)]
            self.round_robin_counters[service_name] = (counter + 1) % len(weighted_instances)
            return selected_instance
    
    def _least_connections_select(self, service_name: str, instances: List[ServiceEndpoint]) -> ServiceEndpoint:
        """Select instance with least connections (simplified implementation)."""
        
        # For now, just return a random instance
        # In a real implementation, this would track active connections per instance
        return random.choice(instances)
    
    def get_service_instances(self, service_name: str) -> List[ServiceEndpoint]:
        """Get all instances for a service."""
        return self.service_instances.get(service_name, [])
    
    def get_all_instances(self) -> Dict[str, List[ServiceEndpoint]]:
        """Get all service instances."""
        return dict(self.service_instances)
    
    def mark_instance_unhealthy(self, service_name: str, instance_url: str):
        """Mark an instance as unhealthy (temporarily remove from rotation)."""
        
        # For now, just log the event
        # In a production implementation, this would remove the instance from rotation
        logger.warning("Instance marked unhealthy",
                      service=service_name,
                      instance=instance_url)
    
    def mark_instance_healthy(self, service_name: str, instance_url: str):
        """Mark an instance as healthy (add back to rotation)."""
        
        # For now, just log the event
        # In a production implementation, this would add the instance back to rotation
        logger.info("Instance marked healthy",
                   service=service_name,
                   instance=instance_url)
    
    def get_load_balancer_stats(self) -> Dict[str, any]:
        """Get load balancer statistics."""
        
        stats = {
            "algorithm": self.config.load_balancer.algorithm,
            "total_services": len(self.service_instances),
            "services": {}
        }
        
        for service_name, instances in self.service_instances.items():
            stats["services"][service_name] = {
                "instance_count": len(instances),
                "round_robin_counter": self.round_robin_counters.get(service_name, 0),
                "instances": [
                    {
                        "url": inst.base_url,
                        "weight": inst.load_balancer_weight
                    }
                    for inst in instances
                ]
            }
        
        return stats 
