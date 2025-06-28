//! Test fixtures and data generators

use chrono::{DateTime, Utc};
use fake::{Fake, Faker};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User test fixture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFixture {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub department: Option<String>,
    pub lab_affiliation: Option<String>,
}

impl UserFixture {
    /// Create a new random user fixture
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let id = Uuid::new_v4();
        
        Self {
            id,
            email: format!("user_{}@test.tracseq.com", id.to_string().split('-').next().unwrap()),
            password: "SecurePassword123!".to_string(),
            first_name: Faker.fake(),
            last_name: Faker.fake(),
            role: ["guest", "technician", "researcher", "lab_administrator"]
                [rng.gen_range(0..4)]
                .to_string(),
            department: if rng.gen_bool(0.7) {
                Some(["Research", "Quality Control", "Operations", "Administration"]
                    [rng.gen_range(0..4)]
                    .to_string())
            } else {
                None
            },
            lab_affiliation: if rng.gen_bool(0.8) {
                Some(format!("Lab {}", rng.gen_range(1..10)))
            } else {
                None
            },
        }
    }
    
    /// Create a user with specific role
    pub fn with_role(role: &str) -> Self {
        let mut user = Self::new();
        user.role = role.to_string();
        user
    }
    
    /// Create an admin user
    pub fn admin() -> Self {
        Self::with_role("lab_administrator")
    }
}

/// Sample test fixture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleFixture {
    pub id: Uuid,
    pub sample_id: String,
    pub name: String,
    pub sample_type: String,
    pub status: String,
    pub barcode: Option<String>,
    pub volume: Option<f64>,
    pub concentration: Option<f64>,
    pub quality_score: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

impl SampleFixture {
    /// Create a new random sample fixture
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let id = Uuid::new_v4();
        
        Self {
            id,
            sample_id: format!("SMP-{}", id.to_string().split('-').next().unwrap().to_uppercase()),
            name: format!("Test Sample {}", rng.gen_range(1000..9999)),
            sample_type: ["DNA", "RNA", "Protein", "Cell", "Tissue"]
                [rng.gen_range(0..5)]
                .to_string(),
            status: ["pending", "processing", "completed", "failed"]
                [rng.gen_range(0..4)]
                .to_string(),
            barcode: if rng.gen_bool(0.8) {
                Some(format!("BC{:08}", rng.gen_range(10000000..99999999)))
            } else {
                None
            },
            volume: if rng.gen_bool(0.9) {
                Some(rng.gen_range(10.0..1000.0))
            } else {
                None
            },
            concentration: if rng.gen_bool(0.9) {
                Some(rng.gen_range(0.1..100.0))
            } else {
                None
            },
            quality_score: if rng.gen_bool(0.7) {
                Some(rng.gen_range(0.0..1.0))
            } else {
                None
            },
            created_at: Utc::now(),
            metadata: serde_json::json!({
                "source": "test_fixture",
                "test_id": Uuid::new_v4().to_string()
            }),
        }
    }
    
    /// Create a sample with specific type
    pub fn with_type(sample_type: &str) -> Self {
        let mut sample = Self::new();
        sample.sample_type = sample_type.to_string();
        sample
    }
    
    /// Create a DNA sample
    pub fn dna() -> Self {
        Self::with_type("DNA")
    }
    
    /// Create an RNA sample
    pub fn rna() -> Self {
        Self::with_type("RNA")
    }
}

/// Storage location fixture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageLocationFixture {
    pub id: Uuid,
    pub name: String,
    pub location_type: String,
    pub temperature: Option<f64>,
    pub capacity: i32,
    pub current_occupancy: i32,
    pub zone: String,
    pub metadata: serde_json::Value,
}

impl StorageLocationFixture {
    /// Create a new storage location fixture
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let capacity = rng.gen_range(50..500);
        
        Self {
            id: Uuid::new_v4(),
            name: format!("Storage Unit {}", rng.gen_range(100..999)),
            location_type: ["freezer", "refrigerator", "room_temperature", "incubator"]
                [rng.gen_range(0..4)]
                .to_string(),
            temperature: match rng.gen_range(0..4) {
                0 => Some(-80.0),
                1 => Some(-20.0),
                2 => Some(4.0),
                3 => Some(37.0),
                _ => None,
            },
            capacity,
            current_occupancy: rng.gen_range(0..capacity),
            zone: format!("Zone {}", ["A", "B", "C", "D"][rng.gen_range(0..4)]),
            metadata: serde_json::json!({
                "rack_count": rng.gen_range(1..10),
                "shelf_count": rng.gen_range(1..5)
            }),
        }
    }
    
    /// Create a freezer storage location
    pub fn freezer() -> Self {
        let mut location = Self::new();
        location.location_type = "freezer".to_string();
        location.temperature = Some(-80.0);
        location
    }
}

/// Sequencing run fixture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequencingRunFixture {
    pub id: Uuid,
    pub run_id: String,
    pub sequencer_id: String,
    pub run_type: String,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub read_count: Option<i64>,
    pub quality_metrics: serde_json::Value,
}

impl SequencingRunFixture {
    /// Create a new sequencing run fixture
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let id = Uuid::new_v4();
        let started_at = Some(Utc::now() - chrono::Duration::hours(rng.gen_range(1..24)));
        
        Self {
            id,
            run_id: format!("RUN-{}", id.to_string().split('-').next().unwrap().to_uppercase()),
            sequencer_id: format!("SEQ{:03}", rng.gen_range(1..10)),
            run_type: ["Illumina", "PacBio", "ONT", "454"]
                [rng.gen_range(0..4)]
                .to_string(),
            status: ["queued", "running", "completed", "failed"]
                [rng.gen_range(0..4)]
                .to_string(),
            started_at,
            completed_at: if rng.gen_bool(0.6) {
                started_at.map(|t| t + chrono::Duration::hours(rng.gen_range(2..12)))
            } else {
                None
            },
            read_count: if rng.gen_bool(0.7) {
                Some(rng.gen_range(1000000..10000000))
            } else {
                None
            },
            quality_metrics: serde_json::json!({
                "q30_percent": rng.gen_range(85.0..99.0),
                "cluster_density": rng.gen_range(800.0..1200.0),
                "error_rate": rng.gen_range(0.1..2.0)
            }),
        }
    }
}

/// QC result fixture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QcResultFixture {
    pub id: Uuid,
    pub sample_id: Uuid,
    pub qc_type: String,
    pub status: String,
    pub passed: bool,
    pub metrics: serde_json::Value,
    pub performed_at: DateTime<Utc>,
    pub performed_by: Option<Uuid>,
}

impl QcResultFixture {
    /// Create a new QC result fixture
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        
        Self {
            id: Uuid::new_v4(),
            sample_id: Uuid::new_v4(),
            qc_type: ["concentration", "purity", "integrity", "contamination"]
                [rng.gen_range(0..4)]
                .to_string(),
            status: "completed".to_string(),
            passed: rng.gen_bool(0.85),
            metrics: serde_json::json!({
                "value": rng.gen_range(0.5..2.0),
                "unit": "ng/µL",
                "threshold": 1.0
            }),
            performed_at: Utc::now(),
            performed_by: if rng.gen_bool(0.9) {
                Some(Uuid::new_v4())
            } else {
                None
            },
        }
    }
    
    /// Create a passing QC result
    pub fn passing() -> Self {
        let mut result = Self::new();
        result.passed = true;
        result
    }
    
    /// Create a failing QC result
    pub fn failing() -> Self {
        let mut result = Self::new();
        result.passed = false;
        result
    }
}

/// Test data builder for creating related fixtures
pub struct TestDataBuilder {
    users: Vec<UserFixture>,
    samples: Vec<SampleFixture>,
    storage_locations: Vec<StorageLocationFixture>,
    sequencing_runs: Vec<SequencingRunFixture>,
    qc_results: Vec<QcResultFixture>,
}

impl TestDataBuilder {
    /// Create a new test data builder
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
            samples: Vec::new(),
            storage_locations: Vec::new(),
            sequencing_runs: Vec::new(),
            qc_results: Vec::new(),
        }
    }
    
    /// Add users
    pub fn with_users(mut self, count: usize) -> Self {
        for _ in 0..count {
            self.users.push(UserFixture::new());
        }
        self
    }
    
    /// Add samples
    pub fn with_samples(mut self, count: usize) -> Self {
        for _ in 0..count {
            self.samples.push(SampleFixture::new());
        }
        self
    }
    
    /// Add storage locations
    pub fn with_storage_locations(mut self, count: usize) -> Self {
        for _ in 0..count {
            self.storage_locations.push(StorageLocationFixture::new());
        }
        self
    }
    
    /// Add sequencing runs
    pub fn with_sequencing_runs(mut self, count: usize) -> Self {
        for _ in 0..count {
            self.sequencing_runs.push(SequencingRunFixture::new());
        }
        self
    }
    
    /// Add QC results for existing samples
    pub fn with_qc_results_for_samples(mut self) -> Self {
        for sample in &self.samples {
            let mut qc_result = QcResultFixture::new();
            qc_result.sample_id = sample.id;
            self.qc_results.push(qc_result);
        }
        self
    }
    
    /// Build and return all test data
    pub fn build(self) -> TestData {
        TestData {
            users: self.users,
            samples: self.samples,
            storage_locations: self.storage_locations,
            sequencing_runs: self.sequencing_runs,
            qc_results: self.qc_results,
        }
    }
}

/// Container for all test data
pub struct TestData {
    pub users: Vec<UserFixture>,
    pub samples: Vec<SampleFixture>,
    pub storage_locations: Vec<StorageLocationFixture>,
    pub sequencing_runs: Vec<SequencingRunFixture>,
    pub qc_results: Vec<QcResultFixture>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_fixture() {
        let user = UserFixture::new();
        assert!(user.email.contains("@test.tracseq.com"));
        assert!(!user.first_name.is_empty());
        assert!(!user.last_name.is_empty());
    }
    
    #[test]
    fn test_sample_fixture() {
        let sample = SampleFixture::dna();
        assert_eq!(sample.sample_type, "DNA");
        assert!(sample.sample_id.starts_with("SMP-"));
    }
    
    #[test]
    fn test_data_builder() {
        let data = TestDataBuilder::new()
            .with_users(5)
            .with_samples(10)
            .with_qc_results_for_samples()
            .build();
        
        assert_eq!(data.users.len(), 5);
        assert_eq!(data.samples.len(), 10);
        assert_eq!(data.qc_results.len(), 10);
    }
}