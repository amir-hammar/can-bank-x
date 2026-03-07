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

pub enum ApplyTransferRepoError {
    SourceAccountNotFound,
    DestinationAccountNotFound,
    CurrencyMismatch,
    InsufficientFunds,
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

    pub async fn apply_transfer(
        &self,
        from_account_id: &str,
        to_account_id: &str,
        amount: f64,
    ) -> Result<(Account, Account), ApplyTransferRepoError> {
        let mut guard = self
            .accounts
            .lock()
            .expect("account repository mutex poisoned");

        let from_index = guard
            .iter()
            .position(|account| account.account_id == from_account_id)
            .ok_or(ApplyTransferRepoError::SourceAccountNotFound)?;

        let to_index = guard
            .iter()
            .position(|account| account.account_id == to_account_id)
            .ok_or(ApplyTransferRepoError::DestinationAccountNotFound)?;

        if guard[from_index].currency != guard[to_index].currency {
            return Err(ApplyTransferRepoError::CurrencyMismatch);
        }

        if guard[from_index].available_balance < amount {
            return Err(ApplyTransferRepoError::InsufficientFunds);
        }

        let (from_account, to_account) = if from_index < to_index {
            let (left, right) = guard.split_at_mut(to_index);
            (&mut left[from_index], &mut right[0])
        } else {
            let (left, right) = guard.split_at_mut(from_index);
            (&mut right[0], &mut left[to_index])
        };

        from_account.available_balance -= amount;
        from_account.ledger_balance -= amount;
        to_account.available_balance += amount;
        to_account.ledger_balance += amount;

        Ok((from_account.clone(), to_account.clone()))
    }
}

impl Default for AccountRepository {
    fn default() -> Self {
        Self::new()
    }
}
