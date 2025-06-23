"""
Multi-Agent Orchestrator for Laboratory Management System

This module provides orchestration capabilities for managing multiple AI agents
in a laboratory environment, with proper async task management to prevent
task garbage collection and resource leaks.
"""

import asyncio
import logging
import uuid
from typing import Dict, List, Optional, Set, Any, Callable
from dataclasses import dataclass, field
from enum import Enum
from datetime import datetime, timedelta
import weakref
import json


class TaskStatus(Enum):
    """Task execution status"""
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


class AgentType(Enum):
    """Types of available agents"""
    DOCUMENT_PROCESSOR = "document_processor"
    SAMPLE_TRACKER = "sample_tracker"
    QUALITY_CONTROLLER = "quality_controller"
    WORKFLOW_MANAGER = "workflow_manager"
    NOTIFICATION_HANDLER = "notification_handler"


@dataclass
class Task:
    """Task representation"""
    task_id: str
    agent_type: AgentType
    payload: Dict[str, Any]
    priority: int = 0
    created_at: datetime = field(default_factory=datetime.now)
    status: TaskStatus = TaskStatus.PENDING
    result: Optional[Any] = None
    error: Optional[str] = None
    retry_count: int = 0
    max_retries: int = 3


@dataclass
class Agent:
    """Agent representation"""
    agent_id: str
    agent_type: AgentType
    handler: Callable
    max_concurrent_tasks: int = 5
    current_tasks: int = 0
    is_available: bool = True
    last_activity: datetime = field(default_factory=datetime.now)


class MultiAgentOrchestrator:
    """
    Multi-Agent Orchestrator with proper async task management.
    
    This class manages multiple AI agents and ensures proper async task lifecycle
    management to prevent garbage collection of running tasks.
    """
    
    def __init__(self, max_concurrent_tasks: int = 100):
        self.max_concurrent_tasks = max_concurrent_tasks
        self.agents: Dict[str, Agent] = {}
        self.task_queue: asyncio.Queue = asyncio.Queue()
        self.active_tasks: Set[asyncio.Task] = set()  # Store task references
        self.completed_tasks: Dict[str, Task] = {}
        self.task_registry: Dict[str, Task] = {}
        self.logger = logging.getLogger(__name__)
        self._shutdown_event = asyncio.Event()
        self._cleanup_interval = 300  # 5 minutes
        
        # Background tasks
        self._background_tasks: Set[asyncio.Task] = set()
        
    async def start(self):
        """Start the orchestrator and background tasks"""
        self.logger.info("Starting Multi-Agent Orchestrator")
        
        # Start background task management
        cleanup_task = asyncio.create_task(self._cleanup_completed_tasks())
        self._background_tasks.add(cleanup_task)
        cleanup_task.add_done_callback(self._background_tasks.discard)
        
        # Start task processing loop
        process_task = asyncio.create_task(self._process_task_queue())
        self._background_tasks.add(process_task)
        process_task.add_done_callback(self._background_tasks.discard)
        
        self.logger.info("Multi-Agent Orchestrator started successfully")
    
    async def stop(self):
        """Stop the orchestrator and cleanup resources"""
        self.logger.info("Stopping Multi-Agent Orchestrator")
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
        self.logger.info("Multi-Agent Orchestrator stopped")
    
    def register_agent(self, agent_type: AgentType, handler: Callable, 
                      max_concurrent_tasks: int = 5) -> str:
        """Register a new agent"""
        agent_id = str(uuid.uuid4())
        agent = Agent(
            agent_id=agent_id,
            agent_type=agent_type,
            handler=handler,
            max_concurrent_tasks=max_concurrent_tasks
        )
        self.agents[agent_id] = agent
        self.logger.info(f"Registered agent {agent_id} of type {agent_type}")
        return agent_id
    
    def unregister_agent(self, agent_id: str):
        """Unregister an agent"""
        if agent_id in self.agents:
            del self.agents[agent_id]
            self.logger.info(f"Unregistered agent {agent_id}")
    
    async def submit_task(self, agent_type: AgentType, payload: Dict[str, Any], 
                         priority: int = 0) -> str:
        """Submit a task for processing"""
        task_id = str(uuid.uuid4())
        task = Task(
            task_id=task_id,
            agent_type=agent_type,
            payload=payload,
            priority=priority
        )
        
        self.task_registry[task_id] = task
        await self.task_queue.put(task)
        self.logger.info(f"Submitted task {task_id} for agent type {agent_type}")
        return task_id
    
    async def get_task_status(self, task_id: str) -> Optional[Task]:
        """Get the status of a task"""
        return self.task_registry.get(task_id) or self.completed_tasks.get(task_id)
    
    async def _process_task_queue(self):
        """Process tasks from the queue"""
        while not self._shutdown_event.is_set():
            try:
                # Wait for a task or shutdown signal
                task = await asyncio.wait_for(
                    self.task_queue.get(), 
                    timeout=1.0
                )
                
                if len(self.active_tasks) >= self.max_concurrent_tasks:
                    # Re-queue the task if we're at capacity
                    await self.task_queue.put(task)
                    await asyncio.sleep(0.1)
                    continue
                
                # Find available agent
                available_agent = self._find_available_agent(task.agent_type)
                if available_agent:
                    await self._assign_task_to_agent(task, available_agent)
                else:
                    # Re-queue the task if no agent is available
                    await self.task_queue.put(task)
                    await asyncio.sleep(0.1)
                    
            except asyncio.TimeoutError:
                continue
            except Exception as e:
                self.logger.error(f"Error processing task queue: {e}")
                await asyncio.sleep(1.0)
    
    def _find_available_agent(self, agent_type: AgentType) -> Optional[Agent]:
        """Find an available agent of the specified type"""
        for agent in self.agents.values():
            if (agent.agent_type == agent_type and 
                agent.is_available and 
                agent.current_tasks < agent.max_concurrent_tasks):
                return agent
        return None
    
    async def _assign_task_to_agent(self, task: Task, agent: Agent):
        """
        Assign a task to an agent with proper async task management.
        
        This method properly stores task references to prevent garbage collection
        and ensures proper cleanup when tasks complete.
        """
        self.logger.info(f"Assigning task {task.task_id} to agent {agent.agent_id}")
        
        # Update task and agent status
        task.status = TaskStatus.RUNNING
        agent.current_tasks += 1
        agent.last_activity = datetime.now()
        
        # Create the task and store reference to prevent garbage collection
        async_task = asyncio.create_task(
            self._execute_task(task, agent)
        )
        
        # CRITICAL: Store task reference to prevent garbage collection
        self.active_tasks.add(async_task)
        
        # Set up cleanup callback when task completes
        async_task.add_done_callback(
            lambda t: self._task_completion_callback(t, task, agent)
        )
    
    def _task_completion_callback(self, async_task: asyncio.Task, 
                                 task: Task, agent: Agent):
        """
        Callback executed when a task completes.
        
        This ensures proper cleanup of task references and agent state.
        """
        # Remove from active tasks to prevent memory leaks
        self.active_tasks.discard(async_task)
        
        # Update agent state
        agent.current_tasks = max(0, agent.current_tasks - 1)
        agent.last_activity = datetime.now()
        
        # Handle task completion or failure
        if async_task.cancelled():
            task.status = TaskStatus.CANCELLED
            self.logger.info(f"Task {task.task_id} was cancelled")
        elif async_task.exception():
            task.status = TaskStatus.FAILED
            task.error = str(async_task.exception())
            self.logger.error(f"Task {task.task_id} failed: {task.error}")
        else:
            task.status = TaskStatus.COMPLETED
            task.result = async_task.result()
            self.logger.info(f"Task {task.task_id} completed successfully")
        
        # Move to completed tasks
        self.completed_tasks[task.task_id] = task
        if task.task_id in self.task_registry:
            del self.task_registry[task.task_id]
    
    async def _execute_task(self, task: Task, agent: Agent) -> Any:
        """Execute a task using the specified agent"""
        try:
            self.logger.debug(f"Executing task {task.task_id} with agent {agent.agent_id}")
            
            # Call the agent's handler
            result = await agent.handler(task.payload)
            
            self.logger.debug(f"Task {task.task_id} executed successfully")
            return result
            
        except Exception as e:
            self.logger.error(f"Task {task.task_id} execution failed: {e}")
            
            # Retry logic
            if task.retry_count < task.max_retries:
                task.retry_count += 1
                self.logger.info(f"Retrying task {task.task_id} (attempt {task.retry_count})")
                
                # Re-queue the task for retry
                await self.task_queue.put(task)
                return None
            else:
                raise e
    
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
                    self.logger.info(f"Cleaned up {len(tasks_to_remove)} old completed tasks")
                    
            except Exception as e:
                self.logger.error(f"Error during cleanup: {e}")
    
    async def get_system_status(self) -> Dict[str, Any]:
        """Get current system status"""
        return {
            "active_tasks": len(self.active_tasks),
            "queued_tasks": self.task_queue.qsize(),
            "completed_tasks": len(self.completed_tasks),
            "registered_agents": len(self.agents),
            "agents": [
                {
                    "agent_id": agent.agent_id,
                    "agent_type": agent.agent_type.value,
                    "current_tasks": agent.current_tasks,
                    "max_concurrent_tasks": agent.max_concurrent_tasks,
                    "is_available": agent.is_available,
                    "last_activity": agent.last_activity.isoformat()
                }
                for agent in self.agents.values()
            ]
        }
    
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
            self.logger.warning("Timeout waiting for task completion")


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
    
    # Create orchestrator
    orchestrator = MultiAgentOrchestrator(max_concurrent_tasks=50)
    
    # Create sample agents
    doc_agent = SampleAgent(AgentType.DOCUMENT_PROCESSOR)
    sample_agent = SampleAgent(AgentType.SAMPLE_TRACKER)
    qc_agent = SampleAgent(AgentType.QUALITY_CONTROLLER)
    
    # Register agents
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
        await orchestrator.start()
        
        # Submit some tasks
        tasks = []
        for i in range(10):
            task_id = await orchestrator.submit_task(
                AgentType.DOCUMENT_PROCESSOR,
                {"document_id": f"doc_{i}", "action": "process"},
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
        await orchestrator.stop()


if __name__ == "__main__":
    asyncio.run(main())