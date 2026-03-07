use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateTransferRequest {
    pub customer_id: String,
    pub from_account_id: String,
    pub to_account_id: String,
    pub amount: f64,
    pub idempotency_key: String,
}

#[derive(Debug, Serialize)]
pub struct CreateTransferResponse {
    pub transfer_id: String,
    pub status: String,
    pub customer_id: String,
    pub from_account_id: String,
    pub to_account_id: String,
    pub amount: f64,
    pub currency: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ListTransfersQuery {
    pub customer_id: Option<String>,
    pub account_id: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct TransferSummaryResponse {
    pub transfer_id: String,
    pub customer_id: String,
    pub from_account_id: String,
    pub to_account_id: String,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct TransferDetailResponse {
    pub transfer_id: String,
    pub customer_id: String,
    pub from_account_id: String,
    pub to_account_id: String,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub idempotency_key: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct AccountBalanceResponse {
    pub account_id: String,
    pub available_balance: f64,
    pub ledger_balance: f64,
    pub currency: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: &'static str,
    pub message: String,
}
