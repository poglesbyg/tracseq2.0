/// ERP (Enterprise Resource Planning) Integration
/// 
/// This module provides comprehensive integration with enterprise resource
/// planning systems including inventory management, financial tracking,
/// procurement, and resource planning capabilities.

use super::{Integration, IntegrationError, IntegrationStatus, ConnectionTest, HealthStatus, ConnectionStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use async_trait::async_trait;
use reqwest::Client;
use tokio::time::{timeout, Duration};

/// ERP Integration implementation
#[derive(Debug)]
pub struct ERPIntegration {
    config: ERPConfig,
    client: Client,
    auth_token: Option<String>,
    last_health_check: Option<DateTime<Utc>>,
}

impl ERPIntegration {
    pub fn new(config: &ERPConfig) -> Self {
        Self {
            config: config.clone(),
            client: Client::new(),
            auth_token: None,
            last_health_check: None,
        }
    }

    /// Authenticate with ERP system
    async fn authenticate(&mut self) -> Result<(), IntegrationError> {
        let auth_request = ERPAuthRequest {
            client_id: self.config.client_id.clone(),
            client_secret: self.config.client_secret.clone(),
            grant_type: "client_credentials".to_string(),
            scope: self.config.scope.clone(),
        };

        let response = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.client
                .post(&format!("{}/oauth/token", self.config.base_url))
                .form(&auth_request)
                .send()
        ).await
        .map_err(|_| IntegrationError::TimeoutError("Authentication timeout".to_string()))?
        .map_err(|e| IntegrationError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(IntegrationError::AuthenticationFailed(
                format!("HTTP {}: {}", response.status(), response.status().canonical_reason().unwrap_or("Unknown"))
            ));
        }

        let auth_response: ERPAuthResponse = response
            .json()
            .await
            .map_err(|e| IntegrationError::InternalError(format!("Failed to parse auth response: {}", e)))?;

        self.auth_token = Some(auth_response.access_token);
        Ok(())
    }

    /// Sync inventory item to ERP
    pub async fn sync_inventory_item(&self, item: &ERPInventoryItem) -> Result<ERPSyncResult, IntegrationError> {
        let token = self.auth_token.as_ref()
            .ok_or_else(|| IntegrationError::AuthenticationFailed("Not authenticated".to_string()))?;

        let url = if item.erp_id.is_some() {
            format!("{}/api/v1/inventory/{}", self.config.base_url, item.erp_id.as_ref().unwrap())
        } else {
            format!("{}/api/v1/inventory", self.config.base_url)
        };

        let method = if item.erp_id.is_some() { "PUT" } else { "POST" };

        let response = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            match method {
                "PUT" => self.client.put(&url),
                _ => self.client.post(&url),
            }
            .bearer_auth(token)
            .json(item)
            .send()
        ).await
        .map_err(|_| IntegrationError::TimeoutError("Inventory sync timeout".to_string()))?
        .map_err(|e| IntegrationError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(IntegrationError::InternalError(
                format!("Failed to sync inventory: HTTP {}", response.status())
            ));
        }

        let sync_response: ERPInventoryItem = response
            .json()
            .await
            .map_err(|e| IntegrationError::InternalError(format!("Failed to parse sync response: {}", e)))?;

        Ok(ERPSyncResult {
            success: true,
            erp_id: sync_response.erp_id.unwrap_or_else(|| sync_response.sku.clone()),
            updated_at: Utc::now(),
        })
    }

    /// Get purchase order status
    pub async fn get_purchase_order(&self, po_number: &str) -> Result<ERPPurchaseOrder, IntegrationError> {
        let token = self.auth_token.as_ref()
            .ok_or_else(|| IntegrationError::AuthenticationFailed("Not authenticated".to_string()))?;

        let url = format!("{}/api/v1/purchase-orders/{}", self.config.base_url, po_number);

        let response = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.client
                .get(&url)
                .bearer_auth(token)
                .send()
        ).await
        .map_err(|_| IntegrationError::TimeoutError("PO query timeout".to_string()))?
        .map_err(|e| IntegrationError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(IntegrationError::InternalError(
                format!("Failed to fetch purchase order: HTTP {}", response.status())
            ));
        }

        let po: ERPPurchaseOrder = response
            .json()
            .await
            .map_err(|e| IntegrationError::InternalError(format!("Failed to parse PO response: {}", e)))?;

        Ok(po)
    }

    /// Create purchase requisition
    pub async fn create_purchase_requisition(&self, requisition: &ERPPurchaseRequisition) -> Result<ERPRequisitionResult, IntegrationError> {
        let token = self.auth_token.as_ref()
            .ok_or_else(|| IntegrationError::AuthenticationFailed("Not authenticated".to_string()))?;

        let url = format!("{}/api/v1/purchase-requisitions", self.config.base_url);

        let response = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.client
                .post(&url)
                .bearer_auth(token)
                .json(requisition)
                .send()
        ).await
        .map_err(|_| IntegrationError::TimeoutError("Requisition creation timeout".to_string()))?
        .map_err(|e| IntegrationError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(IntegrationError::InternalError(
                format!("Failed to create requisition: HTTP {}", response.status())
            ));
        }

        let result: ERPRequisitionResult = response
            .json()
            .await
            .map_err(|e| IntegrationError::InternalError(format!("Failed to parse requisition result: {}", e)))?;

        Ok(result)
    }

    /// Get financial data
    pub async fn get_cost_center_data(&self, cost_center: &str, date_range: &ERPDateRange) -> Result<ERPCostCenterData, IntegrationError> {
        let token = self.auth_token.as_ref()
            .ok_or_else(|| IntegrationError::AuthenticationFailed("Not authenticated".to_string()))?;

        let url = format!(
            "{}/api/v1/cost-centers/{}/data?start_date={}&end_date={}",
            self.config.base_url,
            cost_center,
            date_range.start_date.format("%Y-%m-%d"),
            date_range.end_date.format("%Y-%m-%d")
        );

        let response = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.client
                .get(&url)
                .bearer_auth(token)
                .send()
        ).await
        .map_err(|_| IntegrationError::TimeoutError("Cost center query timeout".to_string()))?
        .map_err(|e| IntegrationError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(IntegrationError::InternalError(
                format!("Failed to fetch cost center data: HTTP {}", response.status())
            ));
        }

        let data: ERPCostCenterData = response
            .json()
            .await
            .map_err(|e| IntegrationError::InternalError(format!("Failed to parse cost center data: {}", e)))?;

        Ok(data)
    }

    /// Sync asset information
    pub async fn sync_asset(&self, asset: &ERPAsset) -> Result<ERPSyncResult, IntegrationError> {
        let token = self.auth_token.as_ref()
            .ok_or_else(|| IntegrationError::AuthenticationFailed("Not authenticated".to_string()))?;

        let url = if asset.erp_id.is_some() {
            format!("{}/api/v1/assets/{}", self.config.base_url, asset.erp_id.as_ref().unwrap())
        } else {
            format!("{}/api/v1/assets", self.config.base_url)
        };

        let method = if asset.erp_id.is_some() { "PUT" } else { "POST" };

        let response = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            match method {
                "PUT" => self.client.put(&url),
                _ => self.client.post(&url),
            }
            .bearer_auth(token)
            .json(asset)
            .send()
        ).await
        .map_err(|_| IntegrationError::TimeoutError("Asset sync timeout".to_string()))?
        .map_err(|e| IntegrationError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(IntegrationError::InternalError(
                format!("Failed to sync asset: HTTP {}", response.status())
            ));
        }

        let sync_response: ERPAsset = response
            .json()
            .await
            .map_err(|e| IntegrationError::InternalError(format!("Failed to parse asset sync response: {}", e)))?;

        Ok(ERPSyncResult {
            success: true,
            erp_id: sync_response.erp_id.unwrap_or_else(|| sync_response.asset_tag.clone()),
            updated_at: Utc::now(),
        })
    }

    /// Get budget information
    pub async fn get_budget_status(&self, department: &str, fiscal_year: u32) -> Result<ERPBudgetStatus, IntegrationError> {
        let token = self.auth_token.as_ref()
            .ok_or_else(|| IntegrationError::AuthenticationFailed("Not authenticated".to_string()))?;

        let url = format!(
            "{}/api/v1/budgets/{}?fiscal_year={}",
            self.config.base_url,
            department,
            fiscal_year
        );

        let response = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.client
                .get(&url)
                .bearer_auth(token)
                .send()
        ).await
        .map_err(|_| IntegrationError::TimeoutError("Budget query timeout".to_string()))?
        .map_err(|e| IntegrationError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(IntegrationError::InternalError(
                format!("Failed to fetch budget status: HTTP {}", response.status())
            ));
        }

        let budget: ERPBudgetStatus = response
            .json()
            .await
            .map_err(|e| IntegrationError::InternalError(format!("Failed to parse budget response: {}", e)))?;

        Ok(budget)
    }

    /// Get system health
    pub async fn get_system_health(&self) -> Result<ERPSystemHealth, IntegrationError> {
        let url = format!("{}/api/v1/health", self.config.base_url);

        let response = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.client
                .get(&url)
                .send()
        ).await
        .map_err(|_| IntegrationError::TimeoutError("Health check timeout".to_string()))?
        .map_err(|e| IntegrationError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(IntegrationError::InternalError(
                format!("Failed to get system health: HTTP {}", response.status())
            ));
        }

        let health: ERPSystemHealth = response
            .json()
            .await
            .map_err(|e| IntegrationError::InternalError(format!("Failed to parse health response: {}", e)))?;

        Ok(health)
    }
}

#[async_trait]
impl Integration for ERPIntegration {
    async fn initialize(&self) -> Result<(), IntegrationError> {
        // Test basic connectivity
        let _health = self.get_system_health().await?;
        Ok(())
    }

    async fn get_status(&self) -> Result<IntegrationStatus, IntegrationError> {
        let connection_status = if self.auth_token.is_some() {
            ConnectionStatus::Connected
        } else {
            ConnectionStatus::Disconnected
        };

        let health = match self.last_health_check {
            Some(last_check) if Utc::now().signed_duration_since(last_check).num_minutes() < 5 => {
                HealthStatus::Healthy
            }
            Some(_) => HealthStatus::Warning,
            None => HealthStatus::Critical,
        };

        Ok(IntegrationStatus {
            name: "ERP Integration".to_string(),
            health,
            last_sync: self.last_health_check,
            connection_status,
        })
    }

    async fn test_connection(&self) -> Result<ConnectionTest, IntegrationError> {
        let start_time = std::time::Instant::now();
        
        match self.get_system_health().await {
            Ok(_) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                Ok(ConnectionTest {
                    success: true,
                    response_time_ms: response_time,
                    error_message: None,
                })
            }
            Err(e) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                Ok(ConnectionTest {
                    success: false,
                    response_time_ms: response_time,
                    error_message: Some(e.to_string()),
                })
            }
        }
    }
}

// Configuration and data structures

/// ERP integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERPConfig {
    pub base_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub scope: String,
    pub timeout_seconds: u64,
    pub company_code: String,
    pub cost_center: String,
    pub sync_interval_minutes: u32,
    pub enable_financial_sync: bool,
    pub enable_asset_sync: bool,
    pub enable_procurement_sync: bool,
}

impl Default for ERPConfig {
    fn default() -> Self {
        Self {
            base_url: "https://erp.example.com".to_string(),
            client_id: "tracseq_integration".to_string(),
            client_secret: "secret_key".to_string(),
            scope: "inventory procurement finance".to_string(),
            timeout_seconds: 30,
            company_code: "COMP001".to_string(),
            cost_center: "LAB001".to_string(),
            sync_interval_minutes: 60,
            enable_financial_sync: true,
            enable_asset_sync: true,
            enable_procurement_sync: true,
        }
    }
}

/// ERP authentication request
#[derive(Debug, Serialize)]
struct ERPAuthRequest {
    client_id: String,
    client_secret: String,
    grant_type: String,
    scope: String,
}

/// ERP authentication response
#[derive(Debug, Deserialize)]
struct ERPAuthResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    scope: String,
}

/// Inventory item for ERP sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERPInventoryItem {
    pub erp_id: Option<String>,
    pub sku: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub unit_of_measure: String,
    pub unit_cost: f64,
    pub quantity_on_hand: i32,
    pub minimum_stock_level: i32,
    pub maximum_stock_level: i32,
    pub location: String,
    pub vendor_id: Option<String>,
    pub last_updated: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Purchase order information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERPPurchaseOrder {
    pub po_number: String,
    pub vendor_id: String,
    pub vendor_name: String,
    pub status: String,
    pub order_date: DateTime<Utc>,
    pub expected_delivery: Option<DateTime<Utc>>,
    pub total_amount: f64,
    pub currency: String,
    pub line_items: Vec<ERPPOLineItem>,
    pub approval_status: String,
    pub cost_center: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERPPOLineItem {
    pub line_number: u32,
    pub sku: String,
    pub description: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub total_price: f64,
    pub delivery_status: String,
}

/// Purchase requisition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERPPurchaseRequisition {
    pub requestor_id: String,
    pub department: String,
    pub cost_center: String,
    pub business_justification: String,
    pub urgency: String,
    pub total_estimated_cost: f64,
    pub currency: String,
    pub line_items: Vec<ERPRequisitionLineItem>,
    pub required_by_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERPRequisitionLineItem {
    pub sku: Option<String>,
    pub description: String,
    pub quantity: i32,
    pub estimated_unit_price: f64,
    pub preferred_vendor: Option<String>,
    pub specifications: Option<String>,
}

/// Purchase requisition result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERPRequisitionResult {
    pub requisition_id: String,
    pub status: String,
    pub approval_workflow_id: Option<String>,
    pub estimated_approval_date: Option<DateTime<Utc>>,
    pub next_approval_step: Option<String>,
}

/// Cost center financial data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERPCostCenterData {
    pub cost_center: String,
    pub department: String,
    pub budget_allocated: f64,
    pub budget_spent: f64,
    pub budget_remaining: f64,
    pub budget_utilization_percentage: f64,
    pub expenses: Vec<ERPExpense>,
    pub commitments: Vec<ERPCommitment>,
    pub forecast: Option<ERPBudgetForecast>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERPExpense {
    pub transaction_id: String,
    pub date: DateTime<Utc>,
    pub description: String,
    pub amount: f64,
    pub category: String,
    pub vendor: Option<String>,
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERPCommitment {
    pub commitment_id: String,
    pub po_number: Option<String>,
    pub description: String,
    pub committed_amount: f64,
    pub expected_expense_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERPBudgetForecast {
    pub projected_spend: f64,
    pub confidence_level: f64,
    pub risk_factors: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Asset information for ERP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERPAsset {
    pub erp_id: Option<String>,
    pub asset_tag: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub manufacturer: String,
    pub model: String,
    pub serial_number: String,
    pub acquisition_date: DateTime<Utc>,
    pub acquisition_cost: f64,
    pub depreciation_method: String,
    pub useful_life_years: u32,
    pub current_value: f64,
    pub location: String,
    pub responsible_person: String,
    pub status: String,
    pub maintenance_contract: Option<String>,
    pub warranty_expiry: Option<DateTime<Utc>>,
}

/// Budget status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERPBudgetStatus {
    pub department: String,
    pub fiscal_year: u32,
    pub total_budget: f64,
    pub spent_to_date: f64,
    pub committed_amount: f64,
    pub available_budget: f64,
    pub budget_categories: Vec<ERPBudgetCategory>,
    pub variance_analysis: ERPVarianceAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERPBudgetCategory {
    pub category: String,
    pub budgeted_amount: f64,
    pub actual_spend: f64,
    pub variance: f64,
    pub variance_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERPVarianceAnalysis {
    pub total_variance: f64,
    pub favorable_variances: f64,
    pub unfavorable_variances: f64,
    pub significant_variances: Vec<ERPVarianceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERPVarianceItem {
    pub category: String,
    pub variance_amount: f64,
    pub variance_percentage: f64,
    pub explanation: Option<String>,
}

/// Date range for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERPDateRange {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

/// ERP system health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERPSystemHealth {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub database_status: String,
    pub api_response_time_ms: f64,
    pub active_connections: u32,
    pub last_backup: Option<DateTime<Utc>>,
    pub modules_status: HashMap<String, String>,
}

/// Synchronization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERPSyncResult {
    pub success: bool,
    pub erp_id: String,
    pub updated_at: DateTime<Utc>,
}

/// ERP integration metrics
#[derive(Debug, Clone, Serialize)]
pub struct ERPMetrics {
    pub total_transactions: u64,
    pub successful_syncs: u64,
    pub failed_syncs: u64,
    pub average_response_time_ms: f64,
    pub inventory_items_synced: u64,
    pub purchase_orders_processed: u64,
    pub assets_tracked: u64,
    pub budget_queries: u64,
    pub last_sync_time: Option<DateTime<Utc>>,
    pub error_rate: f64,
}

/// Create ERP integration instance
pub fn create_erp_integration(config: ERPConfig) -> ERPIntegration {
    ERPIntegration::new(&config)
} 
