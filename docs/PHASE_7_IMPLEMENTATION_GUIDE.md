# TracSeq 2.0 - Phase 7 Implementation Guide
## Advanced Microservices Patterns

### 📋 Overview

Phase 7 transforms TracSeq 2.0 into a truly event-driven, distributed system using cutting-edge microservices patterns. This phase implements Event Sourcing, CQRS, Apache Kafka for event streaming, and enhanced saga patterns for distributed transactions.

### 🚀 Phase 7 Components

#### 1. **Event Sourcing**
- **Event Store**: Immutable log of all domain events
- **Event Aggregation**: Rebuild state from events
- **Snapshots**: Performance optimization for event replay
- **Projections**: Real-time event processing

#### 2. **CQRS (Command Query Responsibility Segregation)**
- **Command Handlers**: Process write operations
- **Query Handlers**: Optimized read models
- **Read Model Projections**: Denormalized views
- **Eventual Consistency**: Between write and read models

#### 3. **Apache Kafka Integration**
- **Event Streaming**: Real-time event distribution
- **Topic Management**: Organized event channels
- **Schema Registry**: Event schema evolution
- **ksqlDB**: Stream processing and analytics

#### 4. **Enhanced Saga Pattern**
- **Orchestrated Sagas**: Complex workflow coordination
- **Compensation Logic**: Automatic rollback on failure
- **Saga State Management**: Persistent workflow state
- **Retry and Timeout**: Resilient execution

### 📁 Phase 7 File Structure

```
/workspace/
├── event-sourcing/
│   ├── event-store/
│   │   ├── event_store.rs              # Core event store implementation
│   │   └── migrations/
│   │       └── 001_event_store_schema.sql
│   └── projections/
│       └── projection_handlers.rs
├── cqrs/
│   ├── commands/
│   │   └── command_handler.rs          # Command processing
│   └── queries/
│       └── query_handler.rs            # Read model queries
├── kafka/
│   ├── kafka_integration.rs            # Kafka producer/consumer
│   ├── config/
│   │   └── kafka.yml
│   └── schemas/
│       └── event_schemas.avsc
├── saga-enhanced/
│   ├── orchestrator/
│   │   └── saga_orchestrator.rs        # Saga coordination
│   ├── compensations/
│   │   └── compensation_handlers.rs
│   └── laboratory_saga_example.rs      # Complete example
├── docker-compose.phase7-advanced.yml  # Infrastructure
└── deploy-phase7.sh                    # Deployment script
```

### 🛠️ Implementation Steps

#### Step 1: Deploy Infrastructure

```bash
# Make deployment script executable
chmod +x deploy-phase7.sh

# Deploy Phase 7 infrastructure
./deploy-phase7.sh deploy
```

This deploys:
- PostgreSQL databases for Event Store and Read Models
- Apache Kafka cluster with Zookeeper
- Schema Registry for event schemas
- Kafka UI for monitoring
- ksqlDB for stream processing
- Kafka Connect for CDC

#### Step 2: Implement Event Sourcing in Services

**Add to each microservice's Cargo.toml:**
```toml
[dependencies]
# Event Sourcing
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres", "json", "uuid", "chrono"] }
async-trait = "0.1"

# Kafka Integration
rdkafka = { version = "0.35", features = ["tokio"] }
```

**Example: Modify Sample Service for Event Sourcing:**
```rust
use crate::event_store::{Event, EventStore, LaboratoryEvent};

// In your command handler
async fn handle_create_sample(&self, command: CreateSampleCommand) -> Result<(), Error> {
    // Create domain event
    let event = Event {
        id: Uuid::new_v4(),
        aggregate_id: command.sample_id,
        aggregate_type: "Sample".to_string(),
        event_type: "SampleCreated".to_string(),
        event_version: 1,
        event_data: serde_json::to_value(LaboratoryEvent::SampleCreated {
            sample_id: command.sample_id,
            barcode: command.barcode,
            sample_type: command.sample_type,
            patient_id: command.patient_id,
        })?,
        metadata: create_metadata(&command),
        created_at: Utc::now(),
        sequence_number: 0,
    };
    
    // Store event
    self.event_store.append_events(vec![event], None).await?;
    
    // Publish to Kafka
    self.kafka_producer.publish_event(
        Topics::SAMPLE_EVENTS,
        event.into()
    ).await?;
    
    Ok(())
}
```

#### Step 3: Implement CQRS Read Models

**Create Read Model Projections:**
```rust
// In projection service
pub async fn project_sample_events(&self) -> Result<(), Error> {
    let consumer = self.create_kafka_consumer(vec![Topics::SAMPLE_EVENTS])?;
    
    consumer.register_handler(
        "SampleCreated".to_string(),
        Box::new(SampleCreatedProjection::new(self.read_db.clone()))
    ).await;
    
    consumer.start().await
}

// Projection handler
impl EventHandler for SampleCreatedProjection {
    async fn handle(&self, event: EventEnvelope) -> Result<(), KafkaError> {
        // Insert into read model
        sqlx::query(
            r#"
            INSERT INTO samples_read_model (
                sample_id, barcode, sample_type, patient_id, 
                status, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $6)
            "#
        )
        .bind(&event.aggregate_id)
        .bind(&event.payload["barcode"])
        .bind(&event.payload["sample_type"])
        .bind(&event.payload["patient_id"])
        .bind("created")
        .bind(&event.timestamp)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
```

#### Step 4: Configure Kafka Topics

**Create ksqlDB Streams:**
```sql
-- Connect to ksqlDB
docker exec -it tracseq-ksqldb-server ksql

-- Create sample events stream
CREATE STREAM sample_events (
    event_id VARCHAR KEY,
    event_type VARCHAR,
    aggregate_id VARCHAR,
    aggregate_type VARCHAR,
    event_version INT,
    payload STRUCT<
        sample_id VARCHAR,
        barcode VARCHAR,
        sample_type VARCHAR,
        patient_id VARCHAR
    >,
    metadata STRUCT<
        correlation_id VARCHAR,
        user_id VARCHAR,
        source_service VARCHAR
    >,
    timestamp BIGINT
) WITH (
    KAFKA_TOPIC='laboratory.sample.events',
    VALUE_FORMAT='JSON'
);

-- Create materialized view for sample counts
CREATE TABLE sample_counts AS
    SELECT 
        sample_type,
        COUNT(*) as total_count,
        COUNT(DISTINCT patient_id) as unique_patients
    FROM sample_events
    WHERE event_type = 'SampleCreated'
    GROUP BY sample_type
    EMIT CHANGES;

-- Query real-time statistics
SELECT * FROM sample_counts WHERE sample_type = 'blood';
```

#### Step 5: Implement Laboratory Saga

**Register Saga Definition:**
```rust
// In your application startup
let saga_orchestrator = SagaOrchestrator::new(event_store, kafka_producer);

// Register laboratory processing saga
saga_orchestrator.register_saga_definition(
    create_laboratory_processing_saga()
).await;

// Register step handlers
saga_orchestrator.register_step_handler(
    "CreateSample".to_string(),
    Box::new(CreateSampleStepHandler::new(sample_service, event_producer))
).await;

// Start a saga
let saga_id = saga_orchestrator.start_saga(
    "LaboratoryProcessing".to_string(),
    json!({
        "barcode": "TEST-001",
        "sample_type": "blood",
        "patient_id": patient_id,
        "user_id": user_id,
        "volume_ml": 5.0,
        "temperature_requirement": -80.0
    }),
    correlation_id
).await?;
```

### 📊 Monitoring & Analytics

#### Kafka Monitoring
Access Kafka UI at `http://localhost:8080` to:
- View topics and partitions
- Monitor consumer lag
- Inspect message content
- Track topic throughput

#### Event Store Analytics
```sql
-- Connect to event store database
psql postgres://event_store_user:event_store_pass@localhost:5434/event_store

-- Query event statistics
SELECT 
    aggregate_type,
    event_type,
    COUNT(*) as event_count,
    MIN(created_at) as first_event,
    MAX(created_at) as last_event
FROM events
GROUP BY aggregate_type, event_type
ORDER BY event_count DESC;

-- Query saga execution metrics
SELECT 
    saga_type,
    status,
    COUNT(*) as count,
    AVG(EXTRACT(EPOCH FROM (completed_at - started_at))) as avg_duration_seconds
FROM saga_states
WHERE completed_at IS NOT NULL
GROUP BY saga_type, status;
```

### 🔧 Troubleshooting

#### Common Issues

1. **Kafka Connection Issues**
   ```bash
   # Check Kafka broker logs
   docker logs tracseq-kafka
   
   # Verify topic creation
   docker exec tracseq-kafka kafka-topics --bootstrap-server localhost:9093 --list
   ```

2. **Event Projection Lag**
   ```bash
   # Check consumer group lag
   docker exec tracseq-kafka kafka-consumer-groups \
       --bootstrap-server localhost:9093 \
       --group projection-service \
       --describe
   ```

3. **Saga Failures**
   ```sql
   -- Query failed sagas
   SELECT * FROM saga_states 
   WHERE status IN ('Failed', 'TimedOut')
   ORDER BY started_at DESC;
   ```

### 📈 Performance Optimization

#### Event Store Optimization
```sql
-- Partition events table by month
CREATE TABLE events_2024_01 PARTITION OF events
    FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');

-- Create indexes for common queries
CREATE INDEX idx_events_aggregate_created 
    ON events(aggregate_id, created_at DESC);
```

#### Kafka Performance Tuning
```yaml
# In docker-compose.phase7-advanced.yml
environment:
  # Increase partition count for high-throughput topics
  KAFKA_NUM_PARTITIONS: 6
  
  # Optimize for throughput
  KAFKA_COMPRESSION_TYPE: lz4
  KAFKA_BATCH_SIZE: 32768
  KAFKA_LINGER_MS: 20
```

### 🎯 Best Practices

1. **Event Design**
   - Keep events immutable
   - Include all necessary data for projections
   - Version events for schema evolution
   - Use meaningful event names

2. **CQRS Implementation**
   - Keep read models denormalized
   - Handle eventual consistency gracefully
   - Use appropriate caching strategies
   - Monitor projection lag

3. **Saga Design**
   - Keep saga steps idempotent
   - Design clear compensation logic
   - Set appropriate timeouts
   - Log all state transitions

4. **Kafka Usage**
   - Use appropriate partition keys
   - Configure retention policies
   - Monitor consumer lag
   - Use schema registry for evolution

### 🚀 Next Steps

1. **Implement Event Sourcing Aggregates**
   ```rust
   impl Aggregate for Sample {
       fn apply_event(&mut self, event: &Event) -> Result<(), Error> {
           match event.event_type.as_str() {
               "SampleCreated" => self.apply_sample_created(event),
               "SampleValidated" => self.apply_sample_validated(event),
               _ => Ok(())
           }
       }
   }
   ```

2. **Create Business Analytics Streams**
   ```sql
   -- Real-time sample processing metrics
   CREATE STREAM processing_metrics AS
       SELECT 
           WINDOWSTART() as window_start,
           WINDOWEND() as window_end,
           COUNT(*) as samples_processed,
           COUNT(DISTINCT aggregate_id) as unique_samples
       FROM sample_events
       WINDOW TUMBLING (SIZE 1 HOUR)
       GROUP BY sample_type
       EMIT CHANGES;
   ```

3. **Implement Distributed Tracing**
   ```rust
   // Add tracing context to events
   event.metadata.trace_id = span.context().trace_id();
   event.metadata.span_id = span.context().span_id();
   ```

### 🏆 Success Criteria

Phase 7 is complete when:
- ✅ All events are stored in the event store
- ✅ Read models are updated via projections
- ✅ Kafka handles all inter-service events
- ✅ Sagas coordinate complex workflows
- ✅ Real-time analytics are available via ksqlDB
- ✅ System maintains eventual consistency

### 📚 Additional Resources

- [Event Sourcing Pattern](https://martinfowler.com/eaaDev/EventSourcing.html)
- [CQRS Documentation](https://martinfowler.com/bliki/CQRS.html)
- [Apache Kafka Documentation](https://kafka.apache.org/documentation/)
- [Saga Pattern](https://microservices.io/patterns/data/saga.html)

---

**Phase 7 establishes TracSeq 2.0 as a state-of-the-art event-driven microservices platform with advanced distributed system patterns!**