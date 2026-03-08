use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Account {
    pub account_id: String,
    pub customer_id: String,
    pub account_type: String,
    pub status: String,
    pub currency: String,
    pub available_balance: f64,
    pub ledger_balance: f64,
    pub is_default: bool,
}
