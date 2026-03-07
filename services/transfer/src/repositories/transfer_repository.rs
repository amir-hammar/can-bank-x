use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::transfer::Transfer;

#[derive(Clone)]
pub struct CreateTransferInput {
    pub customer_id: String,
    pub from_account_id: String,
    pub to_account_id: String,
    pub amount: f64,
    pub currency: String,
    pub idempotency_key: String,
    pub status: String,
}

#[derive(Clone)]
pub struct TransferRepository {
    sequence: Arc<AtomicU64>,
    transfers: Arc<Mutex<Vec<Transfer>>>,
}

impl TransferRepository {
    pub fn new() -> Self {
        Self {
            sequence: Arc::new(AtomicU64::new(1)),
            transfers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn find_by_idempotency_key(
        &self,
        customer_id: &str,
        idempotency_key: &str,
    ) -> Option<Transfer> {
        let guard = self
            .transfers
            .lock()
            .expect("transfer repository mutex poisoned");

        guard
            .iter()
            .find(|transfer| {
                transfer.customer_id == customer_id && transfer.idempotency_key == idempotency_key
            })
            .cloned()
    }

    pub async fn create_transfer(&self, input: CreateTransferInput) -> Transfer {
        let id = self.sequence.fetch_add(1, Ordering::Relaxed);
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string());

        let transfer = Transfer {
            transfer_id: format!("tr_{}", id),
            customer_id: input.customer_id,
            from_account_id: input.from_account_id,
            to_account_id: input.to_account_id,
            amount: input.amount,
            currency: input.currency,
            status: input.status,
            idempotency_key: input.idempotency_key,
            created_at,
        };

        let mut guard = self
            .transfers
            .lock()
            .expect("transfer repository mutex poisoned");
        guard.push(transfer.clone());

        transfer
    }

    pub async fn find_by_id(&self, transfer_id: &str) -> Option<Transfer> {
        let guard = self
            .transfers
            .lock()
            .expect("transfer repository mutex poisoned");

        guard
            .iter()
            .find(|transfer| transfer.transfer_id == transfer_id)
            .cloned()
    }

    pub async fn list(
        &self,
        customer_id: Option<&str>,
        account_id: Option<&str>,
        limit: usize,
    ) -> Vec<Transfer> {
        let guard = self
            .transfers
            .lock()
            .expect("transfer repository mutex poisoned");

        guard
            .iter()
            .filter(|transfer| {
                let customer_ok = customer_id
                    .map(|expected| transfer.customer_id == expected)
                    .unwrap_or(true);
                let account_ok = account_id
                    .map(|expected| {
                        transfer.from_account_id == expected || transfer.to_account_id == expected
                    })
                    .unwrap_or(true);
                customer_ok && account_ok
            })
            .take(limit)
            .cloned()
            .collect()
    }
}

impl Default for TransferRepository {
    fn default() -> Self {
        Self::new()
    }
}
