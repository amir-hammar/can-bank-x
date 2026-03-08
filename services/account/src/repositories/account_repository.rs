use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::{account::Account, dto::CreateAccountRequest};

#[derive(Clone)]
pub struct AccountRepository {
    pool: PgPool,
}

pub enum ApplyTransferRepoError {
    SourceAccountNotFound,
    DestinationAccountNotFound,
    CurrencyMismatch,
    InsufficientFunds,
}

impl AccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_account(&self, payload: &CreateAccountRequest) -> Account {
        let customer_id = Uuid::parse_str(&payload.customer_id)
            .expect("customer_id must be a valid UUID; validated upstream");
        let initial_balance = payload.initial_balance.unwrap_or(0.0);

        // Check if this is the first account for the customer
        let existing_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM accounts WHERE customer_id = $1")
                .bind(customer_id)
                .fetch_one(&self.pool)
                .await
                .unwrap_or((0,));

        let is_default = existing_count.0 == 0;

        // Create account and balance in transaction-like manner
        let account_id = Uuid::new_v4();

        let _ = sqlx::query(
            "INSERT INTO accounts (id, customer_id, type, status, is_default)
             VALUES ($1, $2, $3, 'OPEN', $4)",
        )
        .bind(account_id)
        .bind(customer_id)
        .bind(&payload.account_type)
        .bind(is_default)
        .execute(&self.pool)
        .await;

        let _ = sqlx::query(
            "INSERT INTO account_balances (account_id, available, ledger)
             VALUES ($1, $2, $3)",
        )
        .bind(account_id)
        .bind(initial_balance)
        .bind(initial_balance)
        .execute(&self.pool)
        .await;

        Account {
            account_id: format!("acc_{}", account_id),
            customer_id: payload.customer_id.clone(),
            account_type: payload.account_type.clone(),
            status: "OPEN".to_string(),
            currency: "CAD".to_string(),
            available_balance: initial_balance,
            ledger_balance: initial_balance,
            is_default,
        }
    }

    pub async fn list_accounts_by_customer(&self, customer_id: &str) -> Vec<Account> {
        let customer_uuid = match Uuid::parse_str(customer_id) {
            Ok(value) => value,
            Err(_) => return vec![],
        };

        let rows = sqlx::query(
                "SELECT a.id, a.customer_id, a.type, a.status, a.currency, 
                    a.is_default, COALESCE(ab.available, 0.0)::double precision, COALESCE(ab.ledger, 0.0)::double precision
             FROM accounts a
             LEFT JOIN account_balances ab ON a.id = ab.account_id
             WHERE a.customer_id = $1
             ORDER BY a.created_at"
        )
        .bind(customer_uuid)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        rows.iter()
            .map(|row| Account {
                account_id: format!("acc_{}", row.get::<Uuid, _>(0)),
                customer_id: customer_id.to_string(),
                account_type: row.get(2),
                status: row.get(3),
                currency: row
                    .get::<Option<String>, _>(4)
                    .unwrap_or_else(|| "CAD".to_string()),
                available_balance: row.get(6),
                ledger_balance: row.get(7),
                is_default: row.get(5),
            })
            .collect()
    }

    pub async fn get_account_by_id(&self, account_id: &str) -> Option<Account> {
        // Remove "acc_" prefix if present
        let uuid_str = account_id.strip_prefix("acc_").unwrap_or(account_id);
        let account_uuid = Uuid::parse_str(uuid_str).ok()?;

        let row = sqlx::query(
                "SELECT a.id, a.customer_id, a.type, a.status, a.currency, 
                    a.is_default, COALESCE(ab.available, 0.0)::double precision, COALESCE(ab.ledger, 0.0)::double precision
             FROM accounts a
             LEFT JOIN account_balances ab ON a.id = ab.account_id
             WHERE a.id = $1"
        )
        .bind(account_uuid)
        .fetch_optional(&self.pool)
        .await
        .ok()?;

        row.map(|r| Account {
            account_id: account_id.to_string(),
            customer_id: r.get::<Uuid, _>(1).to_string(),
            account_type: r.get(2),
            status: r.get(3),
            currency: r
                .get::<Option<String>, _>(4)
                .unwrap_or_else(|| "CAD".to_string()),
            available_balance: r.get(6),
            ledger_balance: r.get(7),
            is_default: r.get(5),
        })
    }

    pub async fn get_default_account_by_customer_id(&self, customer_id: &str) -> Option<Account> {
        let customer_uuid = Uuid::parse_str(customer_id).ok()?;

        let row = sqlx::query(
                "SELECT a.id, a.customer_id, a.type, a.status, a.currency, 
                    a.is_default, COALESCE(ab.available, 0.0)::double precision, COALESCE(ab.ledger, 0.0)::double precision
             FROM accounts a
             LEFT JOIN account_balances ab ON a.id = ab.account_id
             WHERE a.customer_id = $1 AND a.is_default = true
             LIMIT 1"
        )
        .bind(customer_uuid)
        .fetch_optional(&self.pool)
        .await
        .ok()?;

        row.map(|r| Account {
            account_id: format!("acc_{}", r.get::<Uuid, _>(0)),
            customer_id: customer_id.to_string(),
            account_type: r.get(2),
            status: r.get(3),
            currency: r
                .get::<Option<String>, _>(4)
                .unwrap_or_else(|| "CAD".to_string()),
            available_balance: r.get(6),
            ledger_balance: r.get(7),
            is_default: true,
        })
    }

    pub async fn apply_transfer(
        &self,
        from_account_id: &str,
        to_account_id: &str,
        amount: f64,
    ) -> Result<(Account, Account), ApplyTransferRepoError> {
        let from_uuid_str = from_account_id
            .strip_prefix("acc_")
            .unwrap_or(from_account_id);
        let to_uuid_str = to_account_id.strip_prefix("acc_").unwrap_or(to_account_id);

        let from_uuid = Uuid::parse_str(from_uuid_str)
            .map_err(|_| ApplyTransferRepoError::SourceAccountNotFound)?;
        let to_uuid = Uuid::parse_str(to_uuid_str)
            .map_err(|_| ApplyTransferRepoError::DestinationAccountNotFound)?;

        // Fetch both accounts
        let from_row = sqlx::query(
                "SELECT a.id, a.customer_id, a.type, a.status, a.currency, 
                    a.is_default, COALESCE(ab.available, 0.0)::double precision, COALESCE(ab.ledger, 0.0)::double precision
             FROM accounts a
             LEFT JOIN account_balances ab ON a.id = ab.account_id
             WHERE a.id = $1"
        )
        .bind(from_uuid)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .and_then(|opt| opt)
        .ok_or(ApplyTransferRepoError::SourceAccountNotFound)?;

        let to_row = sqlx::query(
                "SELECT a.id, a.customer_id, a.type, a.status, a.currency, 
                    a.is_default, COALESCE(ab.available, 0.0)::double precision, COALESCE(ab.ledger, 0.0)::double precision
             FROM accounts a
             LEFT JOIN account_balances ab ON a.id = ab.account_id
             WHERE a.id = $1"
        )
        .bind(to_uuid)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .and_then(|opt| opt)
        .ok_or(ApplyTransferRepoError::DestinationAccountNotFound)?;

        let from_currency: String = from_row.get(4);
        let to_currency: String = to_row.get(4);

        if from_currency != to_currency {
            return Err(ApplyTransferRepoError::CurrencyMismatch);
        }

        let from_available: f64 = from_row.get(6);

        if from_available < amount {
            return Err(ApplyTransferRepoError::InsufficientFunds);
        }

        // Update balances
        let _ = sqlx::query(
            "UPDATE account_balances 
             SET available = available - $2, ledger = ledger - $2
             WHERE account_id = $1",
        )
        .bind(from_uuid)
        .bind(amount)
        .execute(&self.pool)
        .await;

        let _ = sqlx::query(
            "UPDATE account_balances 
             SET available = available + $2, ledger = ledger + $2
             WHERE account_id = $1",
        )
        .bind(to_uuid)
        .bind(amount)
        .execute(&self.pool)
        .await;

        // Fetch updated accounts
        let updated_from = self
            .get_account_by_id(&format!("acc_{}", from_uuid))
            .await
            .ok_or(ApplyTransferRepoError::SourceAccountNotFound)?;

        let updated_to = self
            .get_account_by_id(&format!("acc_{}", to_uuid))
            .await
            .ok_or(ApplyTransferRepoError::DestinationAccountNotFound)?;

        Ok((updated_from, updated_to))
    }
}
