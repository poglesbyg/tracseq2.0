use pact_consumer::prelude::*;
use pact_consumer::{mock_server::MockServer, Consumer, Provider};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Sample {
    id: Uuid,
    name: String,
    sample_type: String,
    volume_ml: f64,
    collection_date: DateTime<Utc>,
    patient_id: String,
    storage_condition: String,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateSampleRequest {
    name: String,
    sample_type: String,
    volume_ml: f64,
    collection_date: DateTime<Utc>,
    patient_id: String,
    storage_condition: String,
    priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StorageLocation {
    id: Uuid,
    name: String,
    temperature: i32,
    capacity: i32,
    current_usage: i32,
    location_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SequencingRun {
    id: Uuid,
    name: String,
    sequencer_id: String,
    run_type: String,
    status: String,
    samples: Vec<String>,
    created_at: DateTime<Utc>,
}

// Consumer: API Gateway
// Provider: Sample Service
#[tokio::test]
#[cfg(feature = "pact_consumer")]
async fn test_api_gateway_creates_sample() {
    let consumer = Consumer::new("api_gateway");
    let provider = Provider::new("sample_service");
    
    let sample_request = CreateSampleRequest {
        name: "TEST-SAMPLE-001".to_string(),
        sample_type: "blood".to_string(),
        volume_ml: 5.0,
        collection_date: Utc::now(),
        patient_id: "PAT-12345".to_string(),
        storage_condition: "frozen".to_string(),
        priority: "normal".to_string(),
    };
    
    let expected_response = Sample {
        id: Uuid::new_v4(),
        name: "TEST-SAMPLE-001".to_string(),
        sample_type: "blood".to_string(),
        volume_ml: 5.0,
        collection_date: sample_request.collection_date,
        patient_id: "PAT-12345".to_string(),
        storage_condition: "frozen".to_string(),
        status: "pending".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    let pact = consumer
        .interaction("create a new sample", |mut i| {
            i.given("the sample service is available");
            i.request
                .post()
                .path("/api/v1/samples")
                .header("Content-Type", "application/json")
                .json_body(json_pattern!({
                    "name": like!("TEST-SAMPLE-001"),
                    "sample_type": term!("blood", "blood|tissue|dna|rna|plasma"),
                    "volume_ml": like!(5.0),
                    "collection_date": like!("2024-01-01T00:00:00Z"),
                    "patient_id": like!("PAT-12345"),
                    "storage_condition": term!("frozen", "frozen|refrigerated|room_temperature"),
                    "priority": term!("normal", "normal|urgent|stat")
                }));
            i.response
                .created()
                .header("Content-Type", "application/json")
                .json_body(json_pattern!({
                    "id": like!(Uuid::new_v4().to_string()),
                    "name": like!("TEST-SAMPLE-001"),
                    "sample_type": like!("blood"),
                    "volume_ml": like!(5.0),
                    "collection_date": like!("2024-01-01T00:00:00Z"),
                    "patient_id": like!("PAT-12345"),
                    "storage_condition": like!("frozen"),
                    "status": term!("pending", "pending|processing|completed|failed"),
                    "created_at": like!("2024-01-01T00:00:00Z"),
                    "updated_at": like!("2024-01-01T00:00:00Z")
                }));
            i
        })
        .build();
    
    let mock_server = MockServer::start(pact).await;
    let url = mock_server.url();
    
    // Make the actual request
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/v1/samples", url))
        .header("Content-Type", "application/json")
        .json(&sample_request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), 201);
    
    let sample: Sample = response.json().await.expect("Failed to parse response");
    assert_eq!(sample.name, "TEST-SAMPLE-001");
    assert_eq!(sample.status, "pending");
}

// Consumer: Sample Service
// Provider: Storage Service
#[tokio::test]
#[cfg(feature = "pact_consumer")]
async fn test_sample_service_allocates_storage() {
    let consumer = Consumer::new("sample_service");
    let provider = Provider::new("storage_service");
    
    let allocation_request = json!({
        "sample_id": "550e8400-e29b-41d4-a716-446655440000",
        "temperature": -80,
        "duration_days": 365,
        "sample_type": "blood",
        "volume_ml": 5.0
    });
    
    let pact = consumer
        .interaction("allocate storage for sample", |mut i| {
            i.given("storage locations are available");
            i.request
                .post()
                .path("/api/v1/storage/allocate")
                .header("Content-Type", "application/json")
                .json_body(json_pattern!({
                    "sample_id": like!("550e8400-e29b-41d4-a716-446655440000"),
                    "temperature": like!(-80),
                    "duration_days": like!(365),
                    "sample_type": like!("blood"),
                    "volume_ml": like!(5.0)
                }));
            i.response
                .ok()
                .header("Content-Type", "application/json")
                .json_body(json_pattern!({
                    "allocation_id": like!(Uuid::new_v4().to_string()),
                    "location": {
                        "id": like!(Uuid::new_v4().to_string()),
                        "name": like!("Freezer-01-A"),
                        "temperature": like!(-80),
                        "rack": like!("R01"),
                        "box": like!("B12"),
                        "position": like!("A1")
                    },
                    "allocated_at": like!("2024-01-01T00:00:00Z"),
                    "expires_at": like!("2025-01-01T00:00:00Z")
                }));
            i
        })
        .build();
    
    let mock_server = MockServer::start(pact).await;
    let url = mock_server.url();
    
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/v1/storage/allocate", url))
        .header("Content-Type", "application/json")
        .json(&allocation_request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), 200);
}

// Consumer: Sample Service  
// Provider: Sequencing Service
#[tokio::test]
#[cfg(feature = "pact_consumer")]
async fn test_sample_service_creates_sequencing_run() {
    let consumer = Consumer::new("sample_service");
    let provider = Provider::new("sequencing_service");
    
    let sequencing_request = json!({
        "name": "SEQ-RUN-001",
        "sequencer_id": "SEQ-01",
        "run_type": "WGS",
        "samples": ["SAMPLE-001", "SAMPLE-002"],
        "parameters": {
            "read_length": 150,
            "paired_end": true,
            "coverage": 30
        }
    });
    
    let pact = consumer
        .interaction("create sequencing run", |mut i| {
            i.given("sequencer is available");
            i.request
                .post()
                .path("/api/v1/sequencing/runs")
                .header("Content-Type", "application/json")
                .json_body(json_pattern!({
                    "name": like!("SEQ-RUN-001"),
                    "sequencer_id": like!("SEQ-01"),
                    "run_type": term!("WGS", "WGS|WES|RNA-Seq|ChIP-Seq"),
                    "samples": each_like!("SAMPLE-001"),
                    "parameters": like!({
                        "read_length": like!(150),
                        "paired_end": like!(true),
                        "coverage": like!(30)
                    })
                }));
            i.response
                .created()
                .header("Content-Type", "application/json")
                .json_body(json_pattern!({
                    "id": like!(Uuid::new_v4().to_string()),
                    "name": like!("SEQ-RUN-001"),
                    "sequencer_id": like!("SEQ-01"),
                    "run_type": like!("WGS"),
                    "status": term!("scheduled", "scheduled|preparing|running|completed|failed"),
                    "samples": each_like!("SAMPLE-001"),
                    "created_at": like!("2024-01-01T00:00:00Z"),
                    "estimated_completion": like!("2024-01-02T00:00:00Z")
                }));
            i
        })
        .build();
    
    let mock_server = MockServer::start(pact).await;
    let url = mock_server.url();
    
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/v1/sequencing/runs", url))
        .header("Content-Type", "application/json")
        .json(&sequencing_request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), 201);
}

// Consumer: Sequencing Service
// Provider: Notification Service
#[tokio::test]
#[cfg(feature = "pact_consumer")]
async fn test_sequencing_service_sends_notification() {
    let consumer = Consumer::new("sequencing_service");
    let provider = Provider::new("notification_service");
    
    let notification_request = json!({
        "type": "sequencing_complete",
        "priority": "high",
        "recipients": ["lab_manager@tracseq.io", "pi@tracseq.io"],
        "subject": "Sequencing Run SEQ-RUN-001 Completed",
        "body": "The sequencing run SEQ-RUN-001 has completed successfully.",
        "metadata": {
            "run_id": "550e8400-e29b-41d4-a716-446655440000",
            "run_name": "SEQ-RUN-001",
            "completion_time": "2024-01-02T12:00:00Z",
            "sample_count": 2
        }
    });
    
    let pact = consumer
        .interaction("send sequencing completion notification", |mut i| {
            i.given("notification service is available");
            i.request
                .post()
                .path("/api/v1/notifications")
                .header("Content-Type", "application/json")
                .json_body(json_pattern!({
                    "type": term!("sequencing_complete", "sequencing_complete|sequencing_failed|sample_alert"),
                    "priority": term!("high", "low|medium|high|critical"),
                    "recipients": each_like!("user@example.com"),
                    "subject": like!("Sequencing Run SEQ-RUN-001 Completed"),
                    "body": like!("The sequencing run SEQ-RUN-001 has completed successfully."),
                    "metadata": like!({
                        "run_id": like!("550e8400-e29b-41d4-a716-446655440000"),
                        "run_name": like!("SEQ-RUN-001"),
                        "completion_time": like!("2024-01-02T12:00:00Z"),
                        "sample_count": like!(2)
                    })
                }));
            i.response
                .created()
                .header("Content-Type", "application/json")
                .json_body(json_pattern!({
                    "id": like!(Uuid::new_v4().to_string()),
                    "status": term!("queued", "queued|sending|sent|failed"),
                    "created_at": like!("2024-01-01T00:00:00Z"),
                    "scheduled_for": like!("2024-01-01T00:00:00Z")
                }));
            i
        })
        .build();
    
    let mock_server = MockServer::start(pact).await;
    let url = mock_server.url();
    
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/v1/notifications", url))
        .header("Content-Type", "application/json")
        .json(&notification_request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), 201);
}

// Consumer: Storage Service
// Provider: Event Service
#[tokio::test]
#[cfg(feature = "pact_consumer")]
async fn test_storage_service_publishes_event() {
    let consumer = Consumer::new("storage_service");
    let provider = Provider::new("event_service");
    
    let event_request = json!({
        "event_type": "storage.allocated",
        "aggregate_id": "550e8400-e29b-41d4-a716-446655440000",
        "aggregate_type": "storage_location",
        "data": {
            "sample_id": "660e8400-e29b-41d4-a716-446655440000",
            "location_id": "770e8400-e29b-41d4-a716-446655440000",
            "temperature": -80,
            "allocated_at": "2024-01-01T00:00:00Z"
        },
        "metadata": {
            "user_id": "admin",
            "correlation_id": "880e8400-e29b-41d4-a716-446655440000"
        }
    });
    
    let pact = consumer
        .interaction("publish storage allocation event", |mut i| {
            i.given("event service is available");
            i.request
                .post()
                .path("/api/v1/events")
                .header("Content-Type", "application/json")
                .json_body(json_pattern!({
                    "event_type": like!("storage.allocated"),
                    "aggregate_id": like!("550e8400-e29b-41d4-a716-446655440000"),
                    "aggregate_type": like!("storage_location"),
                    "data": like!({
                        "sample_id": like!("660e8400-e29b-41d4-a716-446655440000"),
                        "location_id": like!("770e8400-e29b-41d4-a716-446655440000"),
                        "temperature": like!(-80),
                        "allocated_at": like!("2024-01-01T00:00:00Z")
                    }),
                    "metadata": like!({
                        "user_id": like!("admin"),
                        "correlation_id": like!("880e8400-e29b-41d4-a716-446655440000")
                    })
                }));
            i.response
                .accepted()
                .header("Content-Type", "application/json")
                .json_body(json_pattern!({
                    "event_id": like!(Uuid::new_v4().to_string()),
                    "timestamp": like!("2024-01-01T00:00:00Z"),
                    "status": "accepted"
                }));
            i
        })
        .build();
    
    let mock_server = MockServer::start(pact).await;
    let url = mock_server.url();
    
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/v1/events", url))
        .header("Content-Type", "application/json")
        .json(&event_request)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), 202);
}