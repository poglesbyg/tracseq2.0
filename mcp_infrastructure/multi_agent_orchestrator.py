#!/usr/bin/env python3
"""
Multi-Agent Orchestrator for TracSeq 2.0

This orchestrator coordinates multiple specialized AI agents using 
the Multi-Agent Communication Protocol (MACP) extension to MCP.
Enhanced with proper async task management to prevent task garbage 
collection and resource leaks.
"""

import asyncio
import logging
from typing import Dict, List, Any, Optional, Union, Set, Callable
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from enum import Enum
import uuid
import json
import weakref

from anthropic import AsyncAnthropic
from mcp_client import McpClient, McpError

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class AgentType(Enum):
    """Types of specialized agents"""
    LABORATORY_ASSISTANT = "laboratory_assistant"
    PREDICTIVE_ANALYTICS = "predictive_analytics"
    QUALITY_INTELLIGENCE = "quality_intelligence"
    OPTIMIZATION = "optimization"
    COMPLIANCE = "compliance"
    RESEARCH_ASSISTANT = "research_assistant"
    DOCUMENT_PROCESSOR = "document_processor"
    SAMPLE_TRACKER = "sample_tracker"
    QUALITY_CONTROLLER = "quality_controller"
    WORKFLOW_MANAGER = "workflow_manager"
    NOTIFICATION_HANDLER = "notification_handler"

class TaskPriority(Enum):
    """Task priority levels"""
    CRITICAL = "critical"
    HIGH = "high"
    MEDIUM = "medium"
    LOW = "low"

class TaskStatus(Enum):
    """Task execution status"""
    PENDING = "pending"
    ASSIGNED = "assigned"
    RUNNING = "running"
    IN_PROGRESS = "in_progress"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"

@dataclass
class Task:
    """Represents a task that can be executed by an agent"""
    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    type: str = ""
    description: str = ""
    priority: TaskPriority = TaskPriority.MEDIUM
    status: TaskStatus = TaskStatus.PENDING
    assigned_agent: Optional[str] = None
    context: Dict[str, Any] = field(default_factory=dict)
    requirements: Dict[str, Any] = field(default_factory=dict)
    created_at: datetime = field(default_factory=datetime.now)
    started_at: Optional[datetime] = None
    completed_at: Optional[datetime] = None
    result: Optional[Dict[str, Any]] = None
    error: Optional[str] = None
    dependencies: List[str] = field(default_factory=list)
    estimated_duration: Optional[int] = None  # seconds
    retry_count: int = 0
    max_retries: int = 3

@dataclass
class Agent:
    """Represents a specialized AI agent"""
    id: str
    type: AgentType
    name: str
    description: str
    mcp_endpoint: str
    capabilities: List[str]
    max_concurrent_tasks: int = 5
    current_tasks: List[str] = field(default_factory=list)
    performance_metrics: Dict[str, float] = field(default_factory=dict)
    last_heartbeat: Optional[datetime] = None
    status: str = "idle"  # idle, busy, error, offline
    handler: Optional[Callable] = None  # For direct handler registration

@dataclass
class CollaborationRequest:
    """Request for multi-agent collaboration"""
    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    initiator_agent: str = ""
    participating_agents: List[str] = field(default_factory=list)
    task_id: str = ""
    collaboration_type: str = ""  # sequential, parallel, hierarchical
    shared_context: Dict[str, Any] = field(default_factory=dict)
    coordination_strategy: str = "consensus"  # consensus, voting, hierarchy

class MultiAgentOrchestrator:
    """
    Advanced multi-agent orchestrator that coordinates AI agents
    for complex laboratory management tasks with proper async task management.
    """
    
    def __init__(self, config: Dict[str, Any], max_concurrent_tasks: int = 100):
        self.config = config
        self.max_concurrent_tasks = max_concurrent_tasks
        self.anthropic = AsyncAnthropic(api_key=config.get("anthropic_api_key", ""))
        
        # Agent management
        self.agents: Dict[str, Agent] = {}
        self.mcp_clients: Dict[str, McpClient] = {}
        
        # Task management with proper async handling
        self.tasks: Dict[str, Task] = {}
        self.task_queue: asyncio.PriorityQueue = asyncio.PriorityQueue()
        self.active_tasks: Set[asyncio.Task] = set()  # Store task references
        self.completed_tasks: Dict[str, Task] = {}
        self.task_registry: Dict[str, Task] = {}
        self.active_collaborations: Dict[str, CollaborationRequest] = {}
        
        # Performance tracking
        self.execution_history: List[Dict[str, Any]] = []
        self.performance_metrics: Dict[str, Any] = {}
        
        # Orchestrator state
        self.is_running = False
        self.orchestrator_id = str(uuid.uuid4())
        self._shutdown_event = asyncio.Event()
        self._cleanup_interval = 300  # 5 minutes
        
        # Background tasks with proper reference management
        self._background_tasks: Set[asyncio.Task] = set()
        
        self._initialize_agents()
    
    def _initialize_agents(self):
        """Initialize all specialized agents"""
        agent_configs = self.config.get("agents", {})
        
        for agent_id, agent_config in agent_configs.items():
            agent = Agent(
                id=agent_id,
                type=AgentType(agent_config["type"]),
                name=agent_config["name"],
                description=agent_config["description"],
                mcp_endpoint=agent_config["mcp_endpoint"],
                capabilities=agent_config["capabilities"],
                max_concurrent_tasks=agent_config.get("max_concurrent_tasks", 5)
            )
            
            self.agents[agent_id] = agent
            
            # Initialize MCP client for each agent
            try:
                self.mcp_clients[agent_id] = McpClient(
                    endpoint=agent_config["mcp_endpoint"],
                    timeout=self.config.get("operation_timeout", 300)
                )
                logger.info(f"Initialized agent: {agent.name} ({agent.type.value})")
            except Exception as e:
                logger.error(f"Failed to initialize agent {agent_id}: {e}")
    
    async def start_orchestration(self):
        """Start the orchestration engine with proper async task management"""
        logger.info("Starting Multi-Agent Orchestrator")
        self.is_running = True
        
        # Start background tasks with proper reference management
        orchestration_tasks = [
            self._create_background_task(self._task_scheduler()),
            self._create_background_task(self._agent_health_monitor()),
            self._create_background_task(self._performance_monitor()),
            self._create_background_task(self._collaboration_coordinator()),
            self._create_background_task(self._cleanup_completed_tasks())
        ]
        
        try:
            await asyncio.gather(*orchestration_tasks, return_exceptions=True)
        except Exception as e:
            logger.error(f"Orchestration error: {e}")
        finally:
            self.is_running = False
    
    def _create_background_task(self, coro) -> asyncio.Task:
        """Create a background task with proper reference management"""
        task = asyncio.create_task(coro)
        self._background_tasks.add(task)
        task.add_done_callback(self._background_tasks.discard)
        return task
    
    async def stop_orchestration(self):
        """Stop the orchestration engine with proper cleanup"""
        logger.info("Stopping Multi-Agent Orchestrator")
        self.is_running = False
        self._shutdown_event.set()
        
        # Cancel all active tasks
        for task in self.active_tasks.copy():
            if not task.done():
                task.cancel()
                try:
                    await task
                except asyncio.CancelledError:
                    pass
        
        # Cancel background tasks
        for task in self._background_tasks.copy():
            if not task.done():
                task.cancel()
                try:
                    await task
                except asyncio.CancelledError:
                    pass
        
        self.active_tasks.clear()
        self._background_tasks.clear()
        logger.info("Multi-Agent Orchestrator stopped")
    
    def register_agent(self, agent_type: AgentType, handler: Callable, 
                      max_concurrent_tasks: int = 5, name: str = None) -> str:
        """Register a new agent with direct handler"""
        agent_id = str(uuid.uuid4())
        agent = Agent(
            id=agent_id,
            type=agent_type,
            name=name or f"{agent_type.value}_agent",
            description=f"Agent for {agent_type.value}",
            mcp_endpoint="",  # No MCP endpoint for direct handlers
            capabilities=[agent_type.value],
            max_concurrent_tasks=max_concurrent_tasks,
            handler=handler
        )
        self.agents[agent_id] = agent
        logger.info(f"Registered agent {agent_id} of type {agent_type}")
        return agent_id
    
    def unregister_agent(self, agent_id: str):
        """Unregister an agent"""
        if agent_id in self.agents:
            del self.agents[agent_id]
            if agent_id in self.mcp_clients:
                del self.mcp_clients[agent_id]
            logger.info(f"Unregistered agent {agent_id}")
    
    async def submit_task(self, task: Task = None, agent_type: AgentType = None, 
                         payload: Dict[str, Any] = None, priority: int = None) -> str:
        """Submit a task for execution (supports both styles)"""
        
        # Support both new Task object and legacy parameters
        if task is None and agent_type and payload is not None:
            task_priority = TaskPriority.MEDIUM
            if priority is not None:
                priority_map = {1: TaskPriority.CRITICAL, 2: TaskPriority.HIGH, 
                              3: TaskPriority.MEDIUM, 4: TaskPriority.LOW}
                task_priority = priority_map.get(priority, TaskPriority.MEDIUM)
            
            task = Task(
                type=agent_type.value,
                description=f"Task for {agent_type.value}",
                priority=task_priority,
                context=payload
            )
        elif task is None:
            raise ValueError("Either task object or agent_type+payload must be provided")
        
        logger.info(f"Submitting task: {task.description} (Priority: {task.priority.value})")
        
        # Store task
        self.tasks[task.id] = task
        self.task_registry[task.id] = task
        
        # Add to priority queue (lower number = higher priority)
        priority_map = {
            TaskPriority.CRITICAL: 1,
            TaskPriority.HIGH: 2,
            TaskPriority.MEDIUM: 3,
            TaskPriority.LOW: 4
        }
        
        await self.task_queue.put((priority_map[task.priority], task.id))
        
        return task.id
    
    async def _task_scheduler(self):
        """Main task scheduling loop"""
        while self.is_running:
            try:
                # Wait for task with timeout
                try:
                    priority, task_id = await asyncio.wait_for(
                        self.task_queue.get(), timeout=1.0
                    )
                except asyncio.TimeoutError:
                    continue
                
                task = self.tasks.get(task_id)
                if not task:
                    continue
                
                # Find best agent for task
                best_agent = await self._select_optimal_agent(task)
                
                if best_agent:
                    await self._assign_task_to_agent(task, best_agent)
                else:
                    # No available agent, put task back in queue
                    await self.task_queue.put((priority, task_id))
                    await asyncio.sleep(5)  # Wait before retrying
                
            except Exception as e:
                logger.error(f"Task scheduler error: {e}")
                await asyncio.sleep(1)
    
    async def _select_optimal_agent(self, task: Task) -> Optional[Agent]:
        """Select the best agent for a task using AI-powered decision making"""
        
        # Get available agents that can handle the task
        available_agents = []
        for agent in self.agents.values():
            if (len(agent.current_tasks) < agent.max_concurrent_tasks and
                agent.status in ["idle", "busy"] and
                self._can_agent_handle_task(agent, task)):
                available_agents.append(agent)
        
        if not available_agents:
            return None
        
        if len(available_agents) == 1:
            return available_agents[0]
        
        # Use AI to select the best agent
        return await self._ai_agent_selection(task, available_agents)
    
    def _can_agent_handle_task(self, agent: Agent, task: Task) -> bool:
        """Check if an agent can handle a specific task"""
        required_capabilities = task.requirements.get("capabilities", [])
        return all(cap in agent.capabilities for cap in required_capabilities)
    
    async def _ai_agent_selection(self, task: Task, agents: List[Agent]) -> Agent:
        """Use AI to select the optimal agent for a task"""
        
        # Prepare context for AI decision
        agent_info = []
        for agent in agents:
            agent_info.append({
                "id": agent.id,
                "name": agent.name,
                "type": agent.type.value,
                "capabilities": agent.capabilities,
                "current_load": len(agent.current_tasks),
                "max_capacity": agent.max_concurrent_tasks,
                "performance_score": agent.performance_metrics.get("success_rate", 1.0),
                "avg_completion_time": agent.performance_metrics.get("avg_completion_time", 60)
            })
        
        prompt = f"""
        You are an intelligent task scheduler for a laboratory management system.
        Select the best agent to handle the following task:

        Task Details:
        - Type: {task.type}
        - Description: {task.description}
        - Priority: {task.priority.value}
        - Requirements: {json.dumps(task.requirements, indent=2)}
        - Context: {json.dumps(task.context, indent=2)}

        Available Agents:
        {json.dumps(agent_info, indent=2)}

        Consider:
        1. Agent capabilities matching task requirements
        2. Current workload and capacity
        3. Historical performance
        4. Task priority and urgency
        5. Agent specialization

        Respond with only the agent ID of the best choice.
        """
        
        try:
            response = await self.anthropic.messages.create(
                model="claude-3-sonnet-20240229",
                max_tokens=100,
                messages=[{"role": "user", "content": prompt}]
            )
            
            selected_agent_id = response.content[0].text.strip()
            
            # Find and return the selected agent
            for agent in agents:
                if agent.id == selected_agent_id:
                    return agent
            
            # Fallback to first available agent if AI selection fails
            return agents[0]
            
        except Exception as e:
            logger.warning(f"AI agent selection failed: {e}, using first available")
            return agents[0]
    
    async def _assign_task_to_agent(self, task: Task, agent: Agent):
        """Assign a task to a specific agent"""
        logger.info(f"Assigning task {task.id} to agent {agent.name}")
        
        task.assigned_agent = agent.id
        task.status = TaskStatus.ASSIGNED
        task.started_at = datetime.now()
        
        agent.current_tasks.append(task.id)
        
        # Execute task asynchronously
        asyncio.create_task(self._execute_task(task, agent))
    
    async def _execute_task(self, task: Task, agent: Agent):
        """Execute a task using the assigned agent"""
        try:
            task.status = TaskStatus.IN_PROGRESS
            
            # Call the agent's MCP endpoint to execute the task
            mcp_client = self.mcp_clients[agent.id]
            
            # Determine the appropriate MCP tool based on task type
            tool_name = self._get_tool_for_task_type(task.type)
            
            result = await mcp_client.call_tool(tool_name, {
                "task_id": task.id,
                "task_type": task.type,
                "description": task.description,
                "context": task.context,
                "requirements": task.requirements
            })
            
            # Update task with result
            task.result = result
            task.status = TaskStatus.COMPLETED
            task.completed_at = datetime.now()
            
            # Update agent performance metrics
            completion_time = (task.completed_at - task.started_at).total_seconds()
            self._update_agent_performance(agent, task, completion_time, success=True)
            
            logger.info(f"Task {task.id} completed successfully by {agent.name}")
            
        except Exception as e:
            task.error = str(e)
            task.status = TaskStatus.FAILED
            task.completed_at = datetime.now()
            
            # Update agent performance metrics
            completion_time = (task.completed_at - task.started_at).total_seconds()
            self._update_agent_performance(agent, task, completion_time, success=False)
            
            logger.error(f"Task {task.id} failed on agent {agent.name}: {e}")
        
        finally:
            # Remove task from agent's current tasks
            if task.id in agent.current_tasks:
                agent.current_tasks.remove(task.id)
    
    def _get_tool_for_task_type(self, task_type: str) -> str:
        """Map task types to MCP tools"""
        tool_mapping = {
            "document_processing": "process_laboratory_submission",
            "quality_control": "automated_quality_control",
            "sample_search": "intelligent_sample_search",
            "prediction": "predict_outcome",
            "optimization": "optimize_workflow",
            "compliance_check": "check_compliance",
            "research_analysis": "analyze_research_data"
        }
        
        return tool_mapping.get(task_type, "execute_general_task")
    
    def _update_agent_performance(self, agent: Agent, task: Task, completion_time: float, success: bool):
        """Update agent performance metrics"""
        metrics = agent.performance_metrics
        
        # Update success rate
        total_tasks = metrics.get("total_tasks", 0) + 1
        successful_tasks = metrics.get("successful_tasks", 0) + (1 if success else 0)
        metrics["success_rate"] = successful_tasks / total_tasks
        metrics["total_tasks"] = total_tasks
        metrics["successful_tasks"] = successful_tasks
        
        # Update average completion time
        total_time = metrics.get("total_completion_time", 0) + completion_time
        metrics["avg_completion_time"] = total_time / total_tasks
        metrics["total_completion_time"] = total_time
        
        # Update last activity
        metrics["last_task_completed"] = datetime.now().isoformat()
    
    async def _agent_health_monitor(self):
        """Monitor agent health and availability"""
        while self.is_running:
            try:
                for agent in self.agents.values():
                    try:
                        # Ping agent health
                        mcp_client = self.mcp_clients[agent.id]
                        health_response = await mcp_client.call_tool("health_check", {})
                        
                        if health_response.get("status") == "healthy":
                            agent.status = "idle" if not agent.current_tasks else "busy"
                            agent.last_heartbeat = datetime.now()
                        else:
                            agent.status = "error"
                            
                    except Exception as e:
                        logger.warning(f"Health check failed for agent {agent.name}: {e}")
                        agent.status = "offline"
                
                await asyncio.sleep(30)  # Check every 30 seconds
                
            except Exception as e:
                logger.error(f"Health monitor error: {e}")
                await asyncio.sleep(10)
    
    async def _performance_monitor(self):
        """Monitor overall orchestrator performance"""
        while self.is_running:
            try:
                # Calculate performance metrics
                total_tasks = len(self.tasks)
                completed_tasks = len([t for t in self.tasks.values() if t.status == TaskStatus.COMPLETED])
                failed_tasks = len([t for t in self.tasks.values() if t.status == TaskStatus.FAILED])
                
                self.performance_metrics = {
                    "total_tasks": total_tasks,
                    "completed_tasks": completed_tasks,
                    "failed_tasks": failed_tasks,
                    "success_rate": completed_tasks / total_tasks if total_tasks > 0 else 0,
                    "active_agents": len([a for a in self.agents.values() if a.status != "offline"]),
                    "total_agents": len(self.agents),
                    "timestamp": datetime.now().isoformat()
                }
                
                logger.info(f"Orchestrator Performance: {self.performance_metrics}")
                
                await asyncio.sleep(60)  # Update every minute
                
            except Exception as e:
                logger.error(f"Performance monitor error: {e}")
                await asyncio.sleep(30)
    
    async def _collaboration_coordinator(self):
        """Coordinate multi-agent collaborations"""
        while self.is_running:
            try:
                # Process active collaborations
                for collab_id, collab in list(self.active_collaborations.items()):
                    await self._manage_collaboration(collab)
                
                await asyncio.sleep(10)  # Check every 10 seconds
                
            except Exception as e:
                logger.error(f"Collaboration coordinator error: {e}")
                await asyncio.sleep(15)
    
    async def _manage_collaboration(self, collaboration: CollaborationRequest):
        """Manage a specific collaboration between agents"""
        try:
            # Check if all participating agents are available
            available_agents = []
            for agent_id in collaboration.participating_agents:
                agent = self.agents.get(agent_id)
                if agent and agent.status in ["idle", "busy"]:
                    available_agents.append(agent)
            
            if len(available_agents) == len(collaboration.participating_agents):
                # All agents available, coordinate the collaboration
                logger.info(f"Coordinating collaboration {collaboration.id}")
                
                # Implementation depends on collaboration type
                if collaboration.collaboration_type == "sequential":
                    await self._coordinate_sequential_collaboration(collaboration)
                elif collaboration.collaboration_type == "parallel":
                    await self._coordinate_parallel_collaboration(collaboration)
                elif collaboration.collaboration_type == "hierarchical":
                    await self._coordinate_hierarchical_collaboration(collaboration)
                
                # Remove completed collaboration
                del self.active_collaborations[collaboration.id]
            
        except Exception as e:
            logger.error(f"Collaboration management error: {e}")
    
    async def _coordinate_sequential_collaboration(self, collaboration: CollaborationRequest):
        """Coordinate sequential collaboration between agents"""
        # Implementation for sequential task execution
        pass
    
    async def _coordinate_parallel_collaboration(self, collaboration: CollaborationRequest):
        """Coordinate parallel collaboration between agents"""
        # Implementation for parallel task execution
        pass
    
    async def _coordinate_hierarchical_collaboration(self, collaboration: CollaborationRequest):
        """Coordinate hierarchical collaboration between agents"""
        # Implementation for hierarchical task coordination
        pass
    
    async def request_collaboration(self, collaboration: CollaborationRequest) -> str:
        """Request a collaboration between multiple agents"""
        logger.info(f"Collaboration requested: {collaboration.collaboration_type}")
        
        self.active_collaborations[collaboration.id] = collaboration
        return collaboration.id
    
    async def get_task_status(self, task_id: str) -> Optional[Dict[str, Any]]:
        """Get the current status of a task"""
        task = self.tasks.get(task_id) or self.completed_tasks.get(task_id)
        if not task:
            return None
        
        return {
            "id": task.id,
            "type": task.type,
            "description": task.description,
            "status": task.status.value,
            "assigned_agent": task.assigned_agent,
            "created_at": task.created_at.isoformat(),
            "started_at": task.started_at.isoformat() if task.started_at else None,
            "completed_at": task.completed_at.isoformat() if task.completed_at else None,
            "result": task.result,
            "error": task.error,
            "retry_count": task.retry_count
        }
    
    async def get_orchestrator_status(self) -> Dict[str, Any]:
        """Get the current status of the orchestrator"""
        return {
            "orchestrator_id": self.orchestrator_id,
            "is_running": self.is_running,
            "total_agents": len(self.agents),
            "active_agents": len([a for a in self.agents.values() if a.status != "offline"]),
            "total_tasks": len(self.tasks),
            "pending_tasks": len([t for t in self.tasks.values() if t.status == TaskStatus.PENDING]),
            "active_tasks": len([t for t in self.tasks.values() if t.status == TaskStatus.IN_PROGRESS]),
            "performance_metrics": self.performance_metrics,
            "active_collaborations": len(self.active_collaborations)
        }
    
    async def _cleanup_completed_tasks(self):
        """Periodically clean up old completed tasks"""
        while not self._shutdown_event.is_set():
            try:
                await asyncio.sleep(self._cleanup_interval)
                
                # Remove completed tasks older than 1 hour
                cutoff_time = datetime.now() - timedelta(hours=1)
                tasks_to_remove = []
                
                for task_id, task in self.completed_tasks.items():
                    if task.created_at < cutoff_time:
                        tasks_to_remove.append(task_id)
                
                for task_id in tasks_to_remove:
                    del self.completed_tasks[task_id]
                
                if tasks_to_remove:
                    logger.info(f"Cleaned up {len(tasks_to_remove)} old completed tasks")
                    
            except Exception as e:
                logger.error(f"Error during cleanup: {e}")
    
    async def cancel_task(self, task_id: str) -> bool:
        """Cancel a running task"""
        task = self.task_registry.get(task_id)
        if not task:
            return False
        
        # Find and cancel the corresponding async task
        for async_task in self.active_tasks:
            if hasattr(async_task, '_task_id') and async_task._task_id == task_id:
                async_task.cancel()
                return True
        
        return False
    
    async def wait_for_completion(self, timeout: Optional[float] = None):
        """Wait for all active tasks to complete"""
        if not self.active_tasks:
            return
        
        try:
            await asyncio.wait_for(
                asyncio.gather(*self.active_tasks, return_exceptions=True),
                timeout=timeout
            )
        except asyncio.TimeoutError:
            logger.warning("Timeout waiting for task completion")
    
    async def get_system_status(self) -> Dict[str, Any]:
        """Get current system status"""
        return {
            "active_tasks": len(self.active_tasks),
            "queued_tasks": self.task_queue.qsize(),
            "completed_tasks": len(self.completed_tasks),
            "registered_agents": len(self.agents),
            "orchestrator_id": self.orchestrator_id,
            "is_running": self.is_running,
            "agents": [
                {
                    "agent_id": agent.id,
                    "agent_type": agent.type.value,
                    "current_tasks": len(agent.current_tasks),
                    "max_concurrent_tasks": agent.max_concurrent_tasks,
                    "status": agent.status,
                    "last_activity": agent.last_heartbeat.isoformat() if agent.last_heartbeat else None
                }
                for agent in self.agents.values()
            ]
        }


# Example usage and agent implementations
class SampleAgent:
    """Example agent implementation"""
    
    def __init__(self, agent_type: AgentType):
        self.agent_type = agent_type
        self.logger = logging.getLogger(f"{__name__}.{agent_type.value}")
    
    async def handle_task(self, payload: Dict[str, Any]) -> Dict[str, Any]:
        """Handle a task payload"""
        self.logger.info(f"Processing {self.agent_type.value} task: {payload}")
        
        # Simulate work
        await asyncio.sleep(1.0)
        
        return {
            "status": "success",
            "agent_type": self.agent_type.value,
            "processed_payload": payload,
            "timestamp": datetime.now().isoformat()
        }


async def main():
    """Example usage of the Multi-Agent Orchestrator"""
    logging.basicConfig(level=logging.INFO)
    
    # Configuration for MCP-based agents
    config = {
        "anthropic_api_key": "your-anthropic-api-key",
        "operation_timeout": 300,
        "agents": {
            "lab_assistant": {
                "type": "laboratory_assistant",
                "name": "Laboratory Assistant",
                "description": "General laboratory operations and document processing",
                "mcp_endpoint": "http://localhost:8090/mcp",
                "capabilities": ["document_processing", "sample_management", "workflow_coordination"],
                "max_concurrent_tasks": 5
            },
            "predictive_analytics": {
                "type": "predictive_analytics",
                "name": "Predictive Analytics Agent",
                "description": "ML-powered predictions and forecasting",
                "mcp_endpoint": "http://localhost:8091/mcp",
                "capabilities": ["prediction", "analysis", "optimization"],
                "max_concurrent_tasks": 3
            }
        }
    }
    
    # Create orchestrator
    orchestrator = MultiAgentOrchestrator(config, max_concurrent_tasks=50)
    
    # Create sample agents for direct registration
    doc_agent = SampleAgent(AgentType.DOCUMENT_PROCESSOR)
    sample_agent = SampleAgent(AgentType.SAMPLE_TRACKER)
    qc_agent = SampleAgent(AgentType.QUALITY_CONTROLLER)
    
    # Register agents with direct handlers
    orchestrator.register_agent(
        AgentType.DOCUMENT_PROCESSOR, 
        doc_agent.handle_task, 
        max_concurrent_tasks=3
    )
    orchestrator.register_agent(
        AgentType.SAMPLE_TRACKER, 
        sample_agent.handle_task, 
        max_concurrent_tasks=5
    )
    orchestrator.register_agent(
        AgentType.QUALITY_CONTROLLER, 
        qc_agent.handle_task, 
        max_concurrent_tasks=2
    )
    
    try:
        # Start orchestrator
        await orchestrator.start_orchestration()
        
        # Submit some tasks
        tasks = []
        for i in range(10):
            task_id = await orchestrator.submit_task(
                agent_type=AgentType.DOCUMENT_PROCESSOR,
                payload={"document_id": f"doc_{i}", "action": "process"},
                priority=i
            )
            tasks.append(task_id)
        
        # Wait a bit for processing
        await asyncio.sleep(5)
        
        # Check system status
        status = await orchestrator.get_system_status()
        print(f"System Status: {json.dumps(status, indent=2)}")
        
        # Wait for all tasks to complete
        await orchestrator.wait_for_completion(timeout=30.0)
        
    finally:
        # Clean shutdown
        await orchestrator.stop_orchestration()

if __name__ == "__main__":
    asyncio.run(main())
