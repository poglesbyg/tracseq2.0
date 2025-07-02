# LlamaIndex Workflows Integration for TracSeq 2.0

This directory contains the implementation of LlamaIndex Workflows 1.0 for the TracSeq 2.0 laboratory management system. These workflows provide event-driven, async-first orchestration for complex laboratory operations.

## Overview

LlamaIndex Workflows 1.0 provides a powerful framework for building multi-step, event-driven AI applications. Key benefits include:

- **Event-Driven Architecture**: Steps communicate through events, enabling flexible control flow
- **Async-First Design**: Built for high-performance async operations
- **State Management**: Maintain context across workflow execution
- **Built-in Observability**: Native support for tracing and debugging
- **Self-Correcting Loops**: Easy implementation of retry logic and quality control

## Implemented Workflows

### 1. Document Processing Workflow (`document_processing.py`)

Handles the complete document processing pipeline:
- Document validation
- Text extraction and chunking
- Vector embedding and storage
- Information extraction using LLM
- Database persistence

**Key Events:**
- `DocumentValidatedEvent` - Document ready for processing
- `ChunksCreatedEvent` - Document chunked successfully
- `ExtractionCompletedEvent` - Information extracted
- `DatabaseSavedEvent` - Results persisted

**Usage:**
```python
from lab_submission_rag.workflows.document_processing import process_document_with_workflow

result = await process_document_with_workflow("/path/to/document.pdf")
```

### 2. Quality Control Workflow (`quality_control.py`)

Implements self-correcting extraction with confidence scoring:
- Confidence-based validation
- Retry logic for failed extractions
- Self-correction with feedback
- Optional human-in-the-loop approval

**Key Features:**
- Configurable confidence thresholds
- Maximum retry attempts
- Validation error tracking
- Human review integration

**Usage:**
```python
from lab_submission_rag.workflows.quality_control import extract_with_quality_control

result = await extract_with_quality_control(
    text="Laboratory submission text...",
    require_human_review=True
)
```

### 3. Experiment Tracking Workflow (`experiment_tracking.py`)

MLOps workflow for model comparison and deployment:
- Parallel model evaluation
- Performance metric tracking
- Automated deployment decisions
- Model registry integration

**Key Events:**
- `ModelEvaluationEvent` - Evaluate a model
- `ComparisonEvent` - Compare model results
- `DeploymentDecisionEvent` - Deployment decision
- `ModelDeployedEvent` - Model deployed

**Usage:**
```python
from lab_submission_rag.workflows.experiment_tracking import run_mlops_experiment

result = await run_mlops_experiment(
    experiment_name="Model Comparison 2024",
    model_configs=[...],
    test_data=[...]
)
```

### 4. Multi-Agent Laboratory Workflow (`multi_agent.py`)

Coordinates multiple specialized agents:
- Document Processing Agent
- Quality Control Agent
- Storage Allocation Agent
- Notification Agent

**Agent Coordination:**
- Sequential processing with event handoffs
- Agent activity logging
- Conditional execution based on quality scores

**Usage:**
```python
from lab_submission_rag.workflows.multi_agent import process_laboratory_submission

result = await process_laboratory_submission(
    document_path="/path/to/document.pdf",
    submission_type="urgent",
    priority="high"
)
```

## Integration with Existing System

### Workflow Adapter (`workflow_integration.py`)

The `WorkflowAdapter` class provides a compatibility layer between the new workflow-based approach and the existing RAG orchestrator:

```python
from lab_submission_rag.workflows.workflow_integration import WorkflowAdapter

# Create adapter (use_workflows=True for new approach)
adapter = WorkflowAdapter(use_workflows=True)

# Process document with workflows
result = await adapter.process_document("/path/to/doc.pdf")

# Run MLOps experiment
experiment = await adapter.run_experiment(
    "Experiment Name",
    test_documents=[...]
)

# Use multi-agent processing
agent_result = await adapter.process_with_multi_agent(
    document_path="/path/to/doc.pdf",
    submission_type="urgent"
)
```

### FastAPI Integration

Add workflow endpoints to your existing API:

```python
from lab_submission_rag.workflows.workflow_integration import create_workflow_routes

# In your FastAPI app
app.include_router(create_workflow_routes(app))
```

This adds:
- `POST /api/v2/workflows/process` - Document processing with workflows
- `POST /api/v2/workflows/multi-agent` - Multi-agent processing

## Migration Strategy

### Phase 1: Parallel Testing
Run workflows alongside existing system to compare results:

```python
# Test both approaches
traditional_result = await traditional_adapter.process_document(doc)
workflow_result = await workflow_adapter.process_document(doc)

# Compare confidence scores and results
```

### Phase 2: Gradual Rollout
Use workflows for specific document types or conditions:

```python
async def smart_process(doc_path: str) -> ExtractionResult:
    doc_type = Path(doc_path).suffix.lower()
    
    if doc_type in ['.pdf', '.docx']:  # Well-tested formats
        return await workflow_adapter.process_document(doc_path)
    else:  # Fallback for other formats
        return await traditional_adapter.process_document(doc_path)
```

### Phase 3: Full Migration
Replace RAG orchestrator calls with workflow adapter:

```python
# Old approach
result = await rag_system.process_document(file_path)

# New approach
result = await workflow_adapter.process_document(file_path)
```

## Debugging and Observability

### Workflow Visualization

```python
from llama_index.workflows import draw_all_possible_flows

# Visualize workflow structure
workflow = DocumentProcessingWorkflow()
draw_all_possible_flows(workflow, "document_workflow.png")
```

### Event Streaming

Monitor workflow execution in real-time:

```python
workflow = MultiAgentLabWorkflow()
handler = workflow.run(document_path="/path/to/doc.pdf")

async for event in handler.stream_events():
    print(f"Event: {type(event).__name__}")
    if hasattr(event, 'agent'):
        print(f"Agent: {event.agent}")
```

### Step-by-Step Execution

Debug workflows by executing steps manually:

```python
workflow = QualityControlWorkflow()
ctx = Context(workflow)

# Execute steps one by one
await workflow.run_step(ctx)
# Inspect ctx.data for intermediate results
```

## Best Practices

1. **Event Design**: Keep events focused and include only necessary data
2. **Error Handling**: Use validation events for graceful error handling
3. **State Management**: Store critical data in context for access across steps
4. **Parallel Processing**: Return multiple events from a step for parallel execution
5. **Observability**: Use logging and event streaming for monitoring

## Configuration

Workflows use the existing configuration from `config.py`. Key settings:

```python
# Workflow timeouts
WORKFLOW_TIMEOUT = 600  # seconds

# Quality control thresholds
CONFIDENCE_THRESHOLD_HIGH = 0.85
CONFIDENCE_THRESHOLD_MEDIUM = 0.70
MAX_RETRY_ATTEMPTS = 3

# MLOps settings
DEPLOYMENT_THRESHOLD = 0.7
MODEL_COMPARISON_WEIGHTS = {
    "success_rate": 0.4,
    "confidence": 0.3,
    "speed": 0.3
}
```

## Extending Workflows

### Creating Custom Workflows

```python
from llama_index.workflows import Workflow, Event, step

class CustomWorkflow(Workflow):
    @step
    async def my_step(self, ev: MyEvent) -> AnotherEvent:
        # Process event
        result = await self.process(ev.data)
        return AnotherEvent(result=result)
```

### Adding New Agents

```python
class NewAgent:
    async def perform_task(self, data: Dict) -> Dict:
        # Agent logic
        return processed_data

# Add to multi-agent workflow
self.new_agent = NewAgent()
```

## Performance Considerations

1. **Async Operations**: All workflow steps are async for optimal performance
2. **Parallel Evaluation**: MLOps workflow evaluates models in parallel
3. **Streaming**: Use event streaming for large document processing
4. **Caching**: Context stores intermediate results to avoid recomputation

## Troubleshooting

### Common Issues

1. **Import Errors**: Ensure `llama-index-workflows` is installed:
   ```bash
   pip install llama-index-workflows
   ```

2. **Timeout Errors**: Increase workflow timeout for large documents:
   ```python
   workflow = DocumentProcessingWorkflow(timeout=1200)
   ```

3. **Memory Issues**: Process documents in batches for large datasets

### Logging

Enable debug logging for detailed workflow execution:

```python
import logging
logging.getLogger("llama_index.workflows").setLevel(logging.DEBUG)
```

## Future Enhancements

1. **Distributed Execution**: Run workflow steps across multiple workers
2. **Persistent State**: Save and resume workflow execution
3. **Advanced Routing**: Dynamic step selection based on conditions
4. **Workflow Composition**: Combine workflows for complex operations

## Resources

- [LlamaIndex Workflows Documentation](https://docs.llamaindex.ai/en/stable/understanding/workflows/)
- [Workflow Examples](https://github.com/run-llama/workflows-py/tree/main/examples)
- [Blog Post: Announcing Workflows 1.0](https://www.llamaindex.ai/blog/announcing-workflows-1-0-a-lightweight-framework-for-agentic-systems)

*Context improved by Giga AI*