use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::info;
use sha2::{Sha256, Digest};

use crate::{
    error::StorageResult,
    models::*,
    AppState,
};

/// Verify blockchain integrity
/// GET /blockchain/integrity
pub async fn verify_integrity(
    State(state): State<AppState>,
) -> StorageResult<Json<ApiResponse<BlockchainIntegrityReport>>> {
    info!("Verifying blockchain integrity");

    let integrity_report = perform_integrity_verification(&state).await?;

    Ok(Json(ApiResponse::success(integrity_report)))
}

/// List blockchain transactions
/// GET /blockchain/transactions
pub async fn list_transactions(
    State(state): State<AppState>,
    Query(query): Query<TransactionListQuery>,
) -> StorageResult<Json<ApiResponse<PaginatedResponse<BlockchainTransaction>>>> {
    info!("Listing blockchain transactions");

    let transactions = get_blockchain_transactions(&state, &query).await?;

    Ok(Json(ApiResponse::success(transactions)))
}

/// Get specific transaction details
/// GET /blockchain/transactions/:transaction_id
pub async fn get_transaction(
    State(state): State<AppState>,
    Path(transaction_id): Path<String>,
) -> StorageResult<Json<ApiResponse<BlockchainTransaction>>> {
    info!("Getting transaction details for: {}", transaction_id);

    let transaction = get_transaction_details(&state, &transaction_id).await?;

    Ok(Json(ApiResponse::success(transaction)))
}

/// List blockchain blocks
/// GET /blockchain/blocks
pub async fn list_blocks(
    State(state): State<AppState>,
    Query(query): Query<BlockListQuery>,
) -> StorageResult<Json<ApiResponse<PaginatedResponse<BlockchainBlock>>>> {
    info!("Listing blockchain blocks");

    let blocks = get_blockchain_blocks(&state, &query).await?;

    Ok(Json(ApiResponse::success(blocks)))
}

/// Get specific block details
/// GET /blockchain/blocks/:block_hash
pub async fn get_block(
    State(state): State<AppState>,
    Path(block_hash): Path<String>,
) -> StorageResult<Json<ApiResponse<BlockchainBlock>>> {
    info!("Getting block details for: {}", block_hash);

    let block = get_block_details(&state, &block_hash).await?;

    Ok(Json(ApiResponse::success(block)))
}

/// Record sample chain of custody event
/// POST /blockchain/custody/:sample_id/events
pub async fn record_custody_event(
    State(state): State<AppState>,
    Path(sample_id): Path<Uuid>,
    Json(request): Json<CustodyEventRequest>,
) -> StorageResult<Json<ApiResponse<CustodyEvent>>> {
    info!("Recording custody event for sample: {}", sample_id);

    let custody_event = create_custody_event(&state, sample_id, &request).await?;

    // Add to blockchain
    let transaction = add_to_blockchain(&state, &custody_event).await?;

    Ok(Json(ApiResponse::success(custody_event)))
}

/// Get sample chain of custody history
/// GET /blockchain/custody/:sample_id/history
pub async fn get_custody_history(
    State(state): State<AppState>,
    Path(sample_id): Path<Uuid>,
    Query(query): Query<CustodyHistoryQuery>,
) -> StorageResult<Json<ApiResponse<CustodyHistory>>> {
    info!("Getting custody history for sample: {}", sample_id);

    let history = get_sample_custody_history(&state, sample_id, &query).await?;

    Ok(Json(ApiResponse::success(history)))
}

/// Validate chain of custody for sample
/// GET /blockchain/custody/:sample_id/validate
pub async fn validate_custody_chain(
    State(state): State<AppState>,
    Path(sample_id): Path<Uuid>,
) -> StorageResult<Json<ApiResponse<CustodyValidationResult>>> {
    info!("Validating custody chain for sample: {}", sample_id);

    let validation_result = validate_sample_custody_chain(&state, sample_id).await?;

    Ok(Json(ApiResponse::success(validation_result)))
}

/// Create blockchain audit entry
/// POST /blockchain/audit
pub async fn create_audit_entry(
    State(state): State<AppState>,
    Json(request): Json<AuditEntryRequest>,
) -> StorageResult<Json<ApiResponse<AuditEntry>>> {
    info!("Creating blockchain audit entry: {}", request.event_type);

    let audit_entry = create_audit_record(&state, &request).await?;

    // Add to blockchain
    add_audit_to_blockchain(&state, &audit_entry).await?;

    Ok(Json(ApiResponse::success(audit_entry)))
}

/// Search audit trail
/// GET /blockchain/audit/search
pub async fn search_audit_trail(
    State(state): State<AppState>,
    Query(query): Query<AuditSearchQuery>,
) -> StorageResult<Json<ApiResponse<PaginatedResponse<AuditEntry>>>> {
    info!("Searching audit trail");

    let audit_entries = search_audit_records(&state, &query).await?;

    Ok(Json(ApiResponse::success(audit_entries)))
}

/// Get blockchain statistics
/// GET /blockchain/stats
pub async fn get_blockchain_stats(
    State(state): State<AppState>,
) -> StorageResult<Json<ApiResponse<BlockchainStats>>> {
    info!("Getting blockchain statistics");

    let stats = calculate_blockchain_statistics(&state).await?;

    Ok(Json(ApiResponse::success(stats)))
}

/// Create new block
/// POST /blockchain/blocks
pub async fn create_block(
    State(state): State<AppState>,
    Json(request): Json<CreateBlockRequest>,
) -> StorageResult<Json<ApiResponse<BlockchainBlock>>> {
    info!("Creating new blockchain block");

    let block = create_new_block(&state, &request).await?;

    Ok(Json(ApiResponse::success(block)))
}

/// Export blockchain data
/// GET /blockchain/export
pub async fn export_blockchain(
    State(state): State<AppState>,
    Query(query): Query<ExportQuery>,
) -> StorageResult<Json<ApiResponse<ExportResult>>> {
    info!("Exporting blockchain data");

    let export_result = export_blockchain_data(&state, &query).await?;

    Ok(Json(ApiResponse::success(export_result)))
}

/// Verify digital signature
/// POST /blockchain/verify-signature
pub async fn verify_signature(
    State(state): State<AppState>,
    Json(request): Json<SignatureVerificationRequest>,
) -> StorageResult<Json<ApiResponse<SignatureVerificationResult>>> {
    info!("Verifying digital signature");

    let verification_result = verify_digital_signature(&state, &request).await?;

    Ok(Json(ApiResponse::success(verification_result)))
}

// Helper functions
async fn perform_integrity_verification(state: &AppState) -> StorageResult<BlockchainIntegrityReport> {
    // Mock implementation - in production would verify entire chain
    Ok(BlockchainIntegrityReport {
        is_valid: true,
        total_blocks: 1205,
        verified_blocks: 1205,
        corrupted_blocks: 0,
        last_block_hash: "0x1a2b3c4d5e6f7890abcdef1234567890abcdef12".to_string(),
        verification_timestamp: Utc::now(),
        chain_length: 1205,
        genesis_hash: "0x0000000000000000000000000000000000000000".to_string(),
        verification_time_ms: 2500,
        issues: vec![],
        recommendations: vec![
            "Blockchain integrity verified successfully".to_string(),
            "No issues detected in chain of custody records".to_string(),
        ],
    })
}

async fn get_blockchain_transactions(state: &AppState, query: &TransactionListQuery) -> StorageResult<PaginatedResponse<BlockchainTransaction>> {
    // Mock implementation
    let transactions = vec![
        BlockchainTransaction {
            transaction_id: "tx_001".to_string(),
            block_hash: "0xabc123def456".to_string(),
            transaction_type: "custody_transfer".to_string(),
            sample_id: Some(Uuid::new_v4()),
            from_user: "user_001".to_string(),
            to_user: Some("user_002".to_string()),
            timestamp: Utc::now(),
            data_hash: "0x789xyz012abc".to_string(),
            signature: "0xsignature123".to_string(),
            gas_used: 21000,
            status: "confirmed".to_string(),
            confirmations: 6,
            metadata: Some(json!({
                "event_type": "sample_transfer",
                "location": "lab_storage_1"
            })),
        }
    ];

    Ok(PaginatedResponse {
        data: transactions,
        pagination: PaginationInfo {
            page: query.page.unwrap_or(1),
            per_page: query.per_page.unwrap_or(50),
            total_pages: 1,
            total_items: 1,
            has_next: false,
            has_prev: false,
        },
    })
}

async fn get_transaction_details(state: &AppState, transaction_id: &str) -> StorageResult<BlockchainTransaction> {
    // Mock implementation
    Ok(BlockchainTransaction {
        transaction_id: transaction_id.to_string(),
        block_hash: "0xabc123def456".to_string(),
        transaction_type: "custody_transfer".to_string(),
        sample_id: Some(Uuid::new_v4()),
        from_user: "user_001".to_string(),
        to_user: Some("user_002".to_string()),
        timestamp: Utc::now(),
        data_hash: "0x789xyz012abc".to_string(),
        signature: "0xsignature123".to_string(),
        gas_used: 21000,
        status: "confirmed".to_string(),
        confirmations: 6,
        metadata: Some(json!({
            "event_type": "sample_transfer",
            "location": "lab_storage_1"
        })),
    })
}

async fn get_blockchain_blocks(state: &AppState, query: &BlockListQuery) -> StorageResult<PaginatedResponse<BlockchainBlock>> {
    // Mock implementation
    let blocks = vec![
        BlockchainBlock {
            block_number: 1205,
            block_hash: "0xabc123def456789012345678901234567890abcd".to_string(),
            previous_hash: "0x987654321098765432109876543210987654321".to_string(),
            merkle_root: "0x456789012345678901234567890123456789012".to_string(),
            timestamp: Utc::now(),
            miner: "system".to_string(),
            transaction_count: 15,
            block_size: 2048,
            gas_used: 315000,
            gas_limit: 500000,
            difficulty: 1000000,
            nonce: 123456789,
            transactions: vec![
                "tx_001".to_string(),
                "tx_002".to_string(),
                "tx_003".to_string(),
            ],
        }
    ];

    Ok(PaginatedResponse {
        data: blocks,
        pagination: PaginationInfo {
            page: query.page.unwrap_or(1),
            per_page: query.per_page.unwrap_or(50),
            total_pages: 1,
            total_items: 1,
            has_next: false,
            has_prev: false,
        },
    })
}

async fn get_block_details(state: &AppState, block_hash: &str) -> StorageResult<BlockchainBlock> {
    // Mock implementation
    Ok(BlockchainBlock {
        block_number: 1205,
        block_hash: block_hash.to_string(),
        previous_hash: "0x987654321098765432109876543210987654321".to_string(),
        merkle_root: "0x456789012345678901234567890123456789012".to_string(),
        timestamp: Utc::now(),
        miner: "system".to_string(),
        transaction_count: 15,
        block_size: 2048,
        gas_used: 315000,
        gas_limit: 500000,
        difficulty: 1000000,
        nonce: 123456789,
        transactions: vec![
            "tx_001".to_string(),
            "tx_002".to_string(),
        ],
    })
}

async fn create_custody_event(state: &AppState, sample_id: Uuid, request: &CustodyEventRequest) -> StorageResult<CustodyEvent> {
    let event_id = Uuid::new_v4();
    let data_to_hash = format!("{}{}{}{}", sample_id, request.event_type, request.user_id, Utc::now().timestamp());
    let hash = calculate_sha256(&data_to_hash);

    Ok(CustodyEvent {
        id: event_id,
        sample_id,
        event_type: request.event_type.clone(),
        user_id: request.user_id.clone(),
        timestamp: Utc::now(),
        location: request.location.clone(),
        description: request.description.clone(),
        metadata: request.metadata.clone(),
        data_hash: hash,
        previous_event_hash: request.previous_event_hash.clone(),
        signature: generate_signature(&event_id.to_string()),
        blockchain_transaction_id: None,
    })
}

async fn add_to_blockchain(state: &AppState, custody_event: &CustodyEvent) -> StorageResult<String> {
    // Mock implementation - in production would add to actual blockchain
    let transaction_id = format!("tx_{}", Uuid::new_v4());
    info!("Added custody event to blockchain: {}", transaction_id);
    Ok(transaction_id)
}

async fn get_sample_custody_history(state: &AppState, sample_id: Uuid, query: &CustodyHistoryQuery) -> StorageResult<CustodyHistory> {
    // Mock implementation
    let events = vec![
        CustodyEvent {
            id: Uuid::new_v4(),
            sample_id,
            event_type: "sample_created".to_string(),
            user_id: "lab_tech_001".to_string(),
            timestamp: Utc::now() - chrono::Duration::hours(48),
            location: "reception_desk".to_string(),
            description: "Sample received and registered".to_string(),
            metadata: Some(json!({"temperature": -20.0, "condition": "frozen"})),
            data_hash: "0xhash001".to_string(),
            previous_event_hash: None,
            signature: "0xsig001".to_string(),
            blockchain_transaction_id: Some("tx_001".to_string()),
        },
        CustodyEvent {
            id: Uuid::new_v4(),
            sample_id,
            event_type: "sample_stored".to_string(),
            user_id: "storage_robot_001".to_string(),
            timestamp: Utc::now() - chrono::Duration::hours(47),
            location: "freezer_unit_1".to_string(),
            description: "Sample placed in long-term storage".to_string(),
            metadata: Some(json!({"position": "A1-B2-C3", "temperature": -80.0})),
            data_hash: "0xhash002".to_string(),
            previous_event_hash: Some("0xhash001".to_string()),
            signature: "0xsig002".to_string(),
            blockchain_transaction_id: Some("tx_002".to_string()),
        },
    ];

    Ok(CustodyHistory {
        sample_id,
        total_events: events.len() as i32,
        events,
        chain_valid: true,
        last_verified: Utc::now(),
        compliance_status: "compliant".to_string(),
        integrity_score: 1.0,
    })
}

async fn validate_sample_custody_chain(state: &AppState, sample_id: Uuid) -> StorageResult<CustodyValidationResult> {
    // Mock validation logic
    Ok(CustodyValidationResult {
        sample_id,
        is_valid: true,
        validation_timestamp: Utc::now(),
        total_events: 15,
        validated_events: 15,
        failed_validations: 0,
        chain_breaks: vec![],
        signature_failures: vec![],
        hash_mismatches: vec![],
        compliance_violations: vec![],
        overall_integrity_score: 1.0,
        recommendations: vec![
            "Chain of custody is complete and valid".to_string(),
            "No compliance violations detected".to_string(),
        ],
    })
}

async fn create_audit_record(state: &AppState, request: &AuditEntryRequest) -> StorageResult<AuditEntry> {
    Ok(AuditEntry {
        id: Uuid::new_v4(),
        event_type: request.event_type.clone(),
        user_id: request.user_id.clone(),
        resource_type: request.resource_type.clone(),
        resource_id: request.resource_id.clone(),
        action: request.action.clone(),
        timestamp: Utc::now(),
        ip_address: request.ip_address.clone(),
        user_agent: request.user_agent.clone(),
        session_id: request.session_id.clone(),
        details: request.details.clone(),
        severity: request.severity.clone().unwrap_or("info".to_string()),
        status: "recorded".to_string(),
        blockchain_hash: calculate_sha256(&format!("{}{}{}", request.event_type, request.user_id, Utc::now().timestamp())),
    })
}

async fn add_audit_to_blockchain(state: &AppState, audit_entry: &AuditEntry) -> StorageResult<()> {
    // Mock implementation
    info!("Added audit entry to blockchain: {}", audit_entry.id);
    Ok(())
}

async fn search_audit_records(state: &AppState, query: &AuditSearchQuery) -> StorageResult<PaginatedResponse<AuditEntry>> {
    // Mock implementation
    let entries = vec![
        AuditEntry {
            id: Uuid::new_v4(),
            event_type: "sample_access".to_string(),
            user_id: "user_001".to_string(),
            resource_type: "sample".to_string(),
            resource_id: "SAM001".to_string(),
            action: "view".to_string(),
            timestamp: Utc::now(),
            ip_address: Some("192.168.1.100".to_string()),
            user_agent: Some("TracSeq-Client/1.0".to_string()),
            session_id: Some("session_123".to_string()),
            details: Some(json!({"reason": "quality_check"})),
            severity: "info".to_string(),
            status: "recorded".to_string(),
            blockchain_hash: "0xaudit123".to_string(),
        }
    ];

    Ok(PaginatedResponse {
        data: entries,
        pagination: PaginationInfo {
            page: query.page.unwrap_or(1),
            per_page: query.per_page.unwrap_or(50),
            total_pages: 1,
            total_items: 1,
            has_next: false,
            has_prev: false,
        },
    })
}

async fn calculate_blockchain_statistics(state: &AppState) -> StorageResult<BlockchainStats> {
    Ok(BlockchainStats {
        total_blocks: 1205,
        total_transactions: 18060,
        chain_size_mb: 45.2,
        average_block_time_seconds: 15.0,
        transactions_per_second: 12.5,
        pending_transactions: 3,
        confirmed_transactions: 18057,
        custody_events: 4520,
        audit_entries: 13540,
        integrity_violations: 0,
        last_block_timestamp: Utc::now(),
        network_hash_rate: 1000000.0,
        difficulty: 1000000,
    })
}

async fn create_new_block(state: &AppState, request: &CreateBlockRequest) -> StorageResult<BlockchainBlock> {
    let previous_block = get_latest_block(state).await?;
    let block_number = previous_block.block_number + 1;
    
    let block_data = format!("{}{}{}", block_number, previous_block.block_hash, Utc::now().timestamp());
    let block_hash = calculate_sha256(&block_data);

    Ok(BlockchainBlock {
        block_number,
        block_hash,
        previous_hash: previous_block.block_hash,
        merkle_root: calculate_merkle_root(&request.transactions),
        timestamp: Utc::now(),
        miner: request.miner.clone().unwrap_or("system".to_string()),
        transaction_count: request.transactions.len() as i32,
        block_size: estimate_block_size(&request.transactions),
        gas_used: 315000,
        gas_limit: 500000,
        difficulty: 1000000,
        nonce: 123456789,
        transactions: request.transactions.clone(),
    })
}

async fn get_latest_block(state: &AppState) -> StorageResult<BlockchainBlock> {
    // Mock implementation - would get from blockchain
    Ok(BlockchainBlock {
        block_number: 1204,
        block_hash: "0x987654321098765432109876543210987654321".to_string(),
        previous_hash: "0x123456789012345678901234567890123456789".to_string(),
        merkle_root: "0x456789012345678901234567890123456789012".to_string(),
        timestamp: Utc::now() - chrono::Duration::minutes(15),
        miner: "system".to_string(),
        transaction_count: 12,
        block_size: 1800,
        gas_used: 252000,
        gas_limit: 500000,
        difficulty: 1000000,
        nonce: 987654321,
        transactions: vec!["tx_100".to_string(), "tx_101".to_string()],
    })
}

async fn export_blockchain_data(state: &AppState, query: &ExportQuery) -> StorageResult<ExportResult> {
    Ok(ExportResult {
        export_id: Uuid::new_v4(),
        export_type: query.export_type.clone().unwrap_or("full".to_string()),
        format: query.format.clone().unwrap_or("json".to_string()),
        total_records: 18060,
        file_size_mb: 125.8,
        download_url: "/downloads/blockchain_export_123.json".to_string(),
        created_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::days(7),
        status: "ready".to_string(),
    })
}

async fn verify_digital_signature(state: &AppState, request: &SignatureVerificationRequest) -> StorageResult<SignatureVerificationResult> {
    // Mock signature verification
    Ok(SignatureVerificationResult {
        is_valid: true,
        signer: request.signer.clone(),
        signature: request.signature.clone(),
        message_hash: request.message_hash.clone(),
        verification_timestamp: Utc::now(),
        algorithm: "ECDSA".to_string(),
        public_key: "0xpubkey123".to_string(),
        certificate_valid: true,
        trust_level: "high".to_string(),
    })
}

// Utility functions
fn calculate_sha256(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("0x{:x}", hasher.finalize())
}

fn generate_signature(data: &str) -> String {
    // Mock signature generation
    format!("0xsig_{}", calculate_sha256(data)[2..18].to_string())
}

fn calculate_merkle_root(transactions: &[String]) -> String {
    // Mock merkle root calculation
    let combined = transactions.join("");
    calculate_sha256(&combined)
}

fn estimate_block_size(transactions: &[String]) -> i32 {
    // Mock size estimation
    transactions.len() as i32 * 150 // Rough estimate
}

// Request/Response structures
#[derive(Debug, Deserialize)]
pub struct TransactionListQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub transaction_type: Option<String>,
    pub sample_id: Option<Uuid>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct BlockListQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub from_block: Option<i32>,
    pub to_block: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CustodyEventRequest {
    pub event_type: String,
    pub user_id: String,
    pub location: String,
    pub description: String,
    pub metadata: Option<serde_json::Value>,
    pub previous_event_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CustodyHistoryQuery {
    pub include_metadata: Option<bool>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct AuditEntryRequest {
    pub event_type: String,
    pub user_id: String,
    pub resource_type: String,
    pub resource_id: String,
    pub action: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
    pub details: Option<serde_json::Value>,
    pub severity: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AuditSearchQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub event_type: Option<String>,
    pub user_id: Option<String>,
    pub resource_type: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBlockRequest {
    pub transactions: Vec<String>,
    pub miner: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub export_type: Option<String>,
    pub format: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct SignatureVerificationRequest {
    pub signature: String,
    pub message_hash: String,
    pub signer: String,
}

// Data structures
#[derive(Debug, Serialize)]
pub struct BlockchainIntegrityReport {
    pub is_valid: bool,
    pub total_blocks: i32,
    pub verified_blocks: i32,
    pub corrupted_blocks: i32,
    pub last_block_hash: String,
    pub verification_timestamp: DateTime<Utc>,
    pub chain_length: i32,
    pub genesis_hash: String,
    pub verification_time_ms: i32,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BlockchainTransaction {
    pub transaction_id: String,
    pub block_hash: String,
    pub transaction_type: String,
    pub sample_id: Option<Uuid>,
    pub from_user: String,
    pub to_user: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub data_hash: String,
    pub signature: String,
    pub gas_used: i32,
    pub status: String,
    pub confirmations: i32,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct BlockchainBlock {
    pub block_number: i32,
    pub block_hash: String,
    pub previous_hash: String,
    pub merkle_root: String,
    pub timestamp: DateTime<Utc>,
    pub miner: String,
    pub transaction_count: i32,
    pub block_size: i32,
    pub gas_used: i32,
    pub gas_limit: i32,
    pub difficulty: i32,
    pub nonce: i32,
    pub transactions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CustodyEvent {
    pub id: Uuid,
    pub sample_id: Uuid,
    pub event_type: String,
    pub user_id: String,
    pub timestamp: DateTime<Utc>,
    pub location: String,
    pub description: String,
    pub metadata: Option<serde_json::Value>,
    pub data_hash: String,
    pub previous_event_hash: Option<String>,
    pub signature: String,
    pub blockchain_transaction_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CustodyHistory {
    pub sample_id: Uuid,
    pub total_events: i32,
    pub events: Vec<CustodyEvent>,
    pub chain_valid: bool,
    pub last_verified: DateTime<Utc>,
    pub compliance_status: String,
    pub integrity_score: f64,
}

#[derive(Debug, Serialize)]
pub struct CustodyValidationResult {
    pub sample_id: Uuid,
    pub is_valid: bool,
    pub validation_timestamp: DateTime<Utc>,
    pub total_events: i32,
    pub validated_events: i32,
    pub failed_validations: i32,
    pub chain_breaks: Vec<String>,
    pub signature_failures: Vec<String>,
    pub hash_mismatches: Vec<String>,
    pub compliance_violations: Vec<String>,
    pub overall_integrity_score: f64,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AuditEntry {
    pub id: Uuid,
    pub event_type: String,
    pub user_id: String,
    pub resource_type: String,
    pub resource_id: String,
    pub action: String,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
    pub details: Option<serde_json::Value>,
    pub severity: String,
    pub status: String,
    pub blockchain_hash: String,
}

#[derive(Debug, Serialize)]
pub struct BlockchainStats {
    pub total_blocks: i32,
    pub total_transactions: i32,
    pub chain_size_mb: f64,
    pub average_block_time_seconds: f64,
    pub transactions_per_second: f64,
    pub pending_transactions: i32,
    pub confirmed_transactions: i32,
    pub custody_events: i32,
    pub audit_entries: i32,
    pub integrity_violations: i32,
    pub last_block_timestamp: DateTime<Utc>,
    pub network_hash_rate: f64,
    pub difficulty: i32,
}

#[derive(Debug, Serialize)]
pub struct ExportResult {
    pub export_id: Uuid,
    pub export_type: String,
    pub format: String,
    pub total_records: i32,
    pub file_size_mb: f64,
    pub download_url: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct SignatureVerificationResult {
    pub is_valid: bool,
    pub signer: String,
    pub signature: String,
    pub message_hash: String,
    pub verification_timestamp: DateTime<Utc>,
    pub algorithm: String,
    pub public_key: String,
    pub certificate_valid: bool,
    pub trust_level: String,
}
