#[derive(Debug, Clone)]
pub struct Transfer {
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
