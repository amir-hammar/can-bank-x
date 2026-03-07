use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};

use crate::models::{account::Account, dto::CreateAccountRequest};

#[derive(Clone)]
pub struct AccountRepository {
    sequence: Arc<AtomicU64>,
    accounts: Arc<Mutex<Vec<Account>>>,
}

impl AccountRepository {
    pub fn new() -> Self {
        Self {
            sequence: Arc::new(AtomicU64::new(1)),
            accounts: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn create_account(&self, payload: &CreateAccountRequest) -> Account {
        let id = self.sequence.fetch_add(1, Ordering::Relaxed);
        let initial_balance = payload.initial_balance.unwrap_or(0.0);

        let account = Account {
            account_id: format!("acc_{}_{}", payload.customer_id, id),
            customer_id: payload.customer_id.clone(),
            account_type: payload.account_type.clone(),
            status: "OPEN".to_string(),
            currency: "CAD".to_string(),
            available_balance: initial_balance,
            ledger_balance: initial_balance,
        };

        let mut guard = self
            .accounts
            .lock()
            .expect("account repository mutex poisoned");
        guard.push(account.clone());

        account
    }

    pub async fn list_accounts_by_customer(&self, customer_id: &str) -> Vec<Account> {
        let guard = self
            .accounts
            .lock()
            .expect("account repository mutex poisoned");
        guard
            .iter()
            .filter(|account| account.customer_id == customer_id)
            .cloned()
            .collect()
    }

    pub async fn get_account_by_id(&self, account_id: &str) -> Option<Account> {
        let guard = self
            .accounts
            .lock()
            .expect("account repository mutex poisoned");
        guard
            .iter()
            .find(|account| account.account_id == account_id)
            .cloned()
    }
}

impl Default for AccountRepository {
    fn default() -> Self {
        Self::new()
    }
}
