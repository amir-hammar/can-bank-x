use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub customer_id: String,
    pub account_type: String,
    pub initial_balance: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct CreateAccountResponse {
    pub account_id: String,
    pub status: &'static str,
    pub account_type: String,
    pub currency: String,
    pub available_balance: f64,
    pub is_default: bool,
}

#[derive(Debug, Deserialize)]
pub struct ListAccountsQuery {
    pub customer_id: String,
}

#[derive(Debug, Deserialize)]
pub struct AccountBalanceQuery {
    pub account_id: String,
}

#[derive(Debug, Deserialize)]
pub struct DefaultAccountQuery {
    pub customer_id: String,
}

#[derive(Debug, Serialize)]
pub struct AccountSummaryResponse {
    pub account_id: String,
    pub customer_id: String,
    pub account_type: String,
    pub status: String,
    pub currency: String,
    pub available_balance: f64,
    pub is_default: bool,
}

#[derive(Debug, Serialize)]
pub struct AccountBalanceResponse {
    pub account_id: String,
    pub available_balance: f64,
    pub ledger_balance: f64,
    pub currency: String,
}

#[derive(Debug, Serialize)]
pub struct DefaultAccountResponse {
    pub account_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ApplyTransferRequest {
    pub from_account_id: String,
    pub to_account_id: String,
    pub amount: f64,
}

#[derive(Debug, Serialize)]
pub struct ApplyTransferResponse {
    pub from_account_id: String,
    pub to_account_id: String,
    pub amount: f64,
    pub currency: String,
    pub from_available_balance: f64,
    pub to_available_balance: f64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: &'static str,
    pub message: &'static str,
}
