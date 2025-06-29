// TracSeq 2.0 - Event Store Implementation
// Core event sourcing infrastructure for laboratory operations

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub event_type: String,
    pub event_version: i32,
    pub event_data: serde_json::Value,
    pub metadata: EventMetadata,
    pub created_at: DateTime<Utc>,
    pub sequence_number: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub user_id: Option<Uuid>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub tenant_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug)]
pub struct EventStore {
    pool: PgPool,
    event_handlers: HashMap<String, Vec<Box<dyn EventHandler>>>,
}

#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &Event) -> Result<(), EventStoreError>;
}

#[derive(Debug, thiserror::Error)]
pub enum EventStoreError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Concurrency conflict for aggregate {0}")]
    ConcurrencyConflict(Uuid),
    #[error("Event handler error: {0}")]
    HandlerError(String),
}

impl EventStore {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            event_handlers: HashMap::new(),
        }
    }

    pub async fn append_events(
        &self,
        events: Vec<Event>,
        expected_version: Option<i32>,
    ) -> Result<(), EventStoreError> {
        let mut tx = self.pool.begin().await?;

        for event in &events {
            // Check for concurrency conflicts
            if let Some(expected) = expected_version {
                let current_version = sqlx::query_scalar::<_, i32>(
                    r#"
                    SELECT COALESCE(MAX(event_version), 0)
                    FROM events
                    WHERE aggregate_id = $1
                    "#,
                )
                .bind(&event.aggregate_id)
                .fetch_one(&mut tx)
                .await?;

                if current_version != expected {
                    return Err(EventStoreError::ConcurrencyConflict(event.aggregate_id));
                }
            }

            // Insert event
            sqlx::query(
                r#"
                INSERT INTO events (
                    id, aggregate_id, aggregate_type, event_type, 
                    event_version, event_data, metadata, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(&event.id)
            .bind(&event.aggregate_id)
            .bind(&event.aggregate_type)
            .bind(&event.event_type)
            .bind(event.event_version)
            .bind(&event.event_data)
            .bind(serde_json::to_value(&event.metadata)?)
            .bind(&event.created_at)
            .execute(&mut tx)
            .await?;
        }

        tx.commit().await?;

        // Publish events to handlers
        for event in events {
            self.publish_event(&event).await?;
        }

        Ok(())
    }

    pub async fn get_events(
        &self,
        aggregate_id: Uuid,
        from_version: Option<i32>,
    ) -> Result<Vec<Event>, EventStoreError> {
        let query = if let Some(version) = from_version {
            sqlx::query_as::<_, EventRow>(
                r#"
                SELECT * FROM events
                WHERE aggregate_id = $1 AND event_version > $2
                ORDER BY event_version ASC
                "#,
            )
            .bind(aggregate_id)
            .bind(version)
        } else {
            sqlx::query_as::<_, EventRow>(
                r#"
                SELECT * FROM events
                WHERE aggregate_id = $1
                ORDER BY event_version ASC
                "#,
            )
            .bind(aggregate_id)
        };

        let rows = query.fetch_all(&self.pool).await?;
        
        rows.into_iter()
            .map(|row| row.try_into())
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn get_events_by_type(
        &self,
        event_type: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Event>, EventStoreError> {
        let rows = sqlx::query_as::<_, EventRow>(
            r#"
            SELECT * FROM events
            WHERE event_type = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(event_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| row.try_into())
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn get_snapshot<T: Aggregate>(
        &self,
        aggregate_id: Uuid,
    ) -> Result<Option<AggregateSnapshot<T>>, EventStoreError> {
        let row = sqlx::query(
            r#"
            SELECT snapshot_data, version, created_at
            FROM snapshots
            WHERE aggregate_id = $1
            ORDER BY version DESC
            LIMIT 1
            "#,
        )
        .bind(aggregate_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let snapshot_data: serde_json::Value = row.get("snapshot_data");
            let version: i32 = row.get("version");
            let created_at: DateTime<Utc> = row.get("created_at");

            Ok(Some(AggregateSnapshot {
                aggregate_id,
                version,
                data: serde_json::from_value(snapshot_data)?,
                created_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn save_snapshot<T: Aggregate>(
        &self,
        snapshot: &AggregateSnapshot<T>,
    ) -> Result<(), EventStoreError> {
        sqlx::query(
            r#"
            INSERT INTO snapshots (
                id, aggregate_id, version, snapshot_data, created_at
            ) VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&snapshot.aggregate_id)
        .bind(snapshot.version)
        .bind(serde_json::to_value(&snapshot.data)?)
        .bind(&snapshot.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub fn register_handler(&mut self, event_type: String, handler: Box<dyn EventHandler>) {
        self.event_handlers
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(handler);
    }

    async fn publish_event(&self, event: &Event) -> Result<(), EventStoreError> {
        if let Some(handlers) = self.event_handlers.get(&event.event_type) {
            for handler in handlers {
                handler.handle(event).await.map_err(|e| {
                    EventStoreError::HandlerError(format!("Handler error: {}", e))
                })?;
            }
        }
        Ok(())
    }
}

// Aggregate trait for event sourcing
#[async_trait]
pub trait Aggregate: Serialize + for<'de> Deserialize<'de> + Send + Sync {
    fn aggregate_type() -> &'static str;
    fn apply_event(&mut self, event: &Event) -> Result<(), EventStoreError>;
    fn get_uncommitted_events(&self) -> &[Event];
    fn mark_events_as_committed(&mut self);
    fn get_version(&self) -> i32;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AggregateSnapshot<T: Aggregate> {
    pub aggregate_id: Uuid,
    pub version: i32,
    pub data: T,
    pub created_at: DateTime<Utc>,
}

// Helper struct for database queries
#[derive(sqlx::FromRow)]
struct EventRow {
    id: Uuid,
    aggregate_id: Uuid,
    aggregate_type: String,
    event_type: String,
    event_version: i32,
    event_data: serde_json::Value,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    sequence_number: i64,
}

impl TryFrom<EventRow> for Event {
    type Error = EventStoreError;

    fn try_from(row: EventRow) -> Result<Self, Self::Error> {
        Ok(Event {
            id: row.id,
            aggregate_id: row.aggregate_id,
            aggregate_type: row.aggregate_type,
            event_type: row.event_type,
            event_version: row.event_version,
            event_data: row.event_data,
            metadata: serde_json::from_value(row.metadata)?,
            created_at: row.created_at,
            sequence_number: row.sequence_number,
        })
    }
}

// Laboratory-specific event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LaboratoryEvent {
    // Sample Events
    SampleCreated {
        sample_id: Uuid,
        barcode: String,
        sample_type: String,
        patient_id: Option<Uuid>,
    },
    SampleValidated {
        sample_id: Uuid,
        validation_results: serde_json::Value,
        validated_by: Uuid,
    },
    SampleStored {
        sample_id: Uuid,
        location_id: Uuid,
        temperature: f32,
        stored_by: Uuid,
    },
    
    // Sequencing Events
    SequencingStarted {
        sequencing_id: Uuid,
        sample_id: Uuid,
        protocol: String,
        machine_id: String,
    },
    SequencingCompleted {
        sequencing_id: Uuid,
        results_url: String,
        quality_score: f32,
    },
    
    // Storage Events
    TemperatureAlert {
        location_id: Uuid,
        current_temp: f32,
        target_temp: f32,
        deviation: f32,
    },
    StorageCapacityWarning {
        location_id: Uuid,
        current_capacity: f32,
        threshold: f32,
    },
}