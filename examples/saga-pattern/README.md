# Saga Pattern Example for TracSeq 2.0

This directory contains example code demonstrating how to implement the Saga pattern for distributed transactions in the TracSeq 2.0 laboratory management system.

## Files

- `laboratory_saga_example.rs` - Example implementation of a laboratory processing saga
- `orchestrator/saga_orchestrator.rs` - Core saga orchestrator implementation
- `Dockerfile` - Example Dockerfile for a saga service (for reference)

## Overview

The Saga pattern is used to manage distributed transactions across multiple microservices. In the laboratory context, a single workflow might involve:

1. Creating a sample record
2. Validating sample metadata
3. Allocating storage space
4. Storing the physical sample
5. Scheduling sequencing
6. Sending notifications

Each step is handled by a different service, and the saga orchestrator ensures that if any step fails, appropriate compensation actions are taken.

## Usage

To implement a saga in your service:

1. Define your saga steps using `SagaDefinition`
2. Implement `StepHandler` for each step
3. Configure the saga orchestrator with your handlers
4. Start sagas through the orchestrator API

## Example

```rust
let saga_def = create_laboratory_processing_saga();
let saga_id = orchestrator.start_saga(
    "LaboratoryProcessing",
    initial_context,
    correlation_id,
).await?;
```

## Future Development

When implementing this as a production service:

1. Create a proper Cargo.toml with dependencies
2. Implement persistent saga state storage
3. Add proper error handling and monitoring
4. Integrate with the event bus (Kafka/RabbitMQ)
5. Add REST/gRPC API for saga management
