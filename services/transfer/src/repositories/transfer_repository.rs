use sqlx::{PgPool, Row};
use uuid::Uuid;
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
    pool: PgPool,
}

impl TransferRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_idempotency_key(
        &self,
        customer_id: &str,
        idempotency_key: &str,
    ) -> Option<Transfer> {
        let customer_uuid = Uuid::parse_str(customer_id).ok()?;

        let row = sqlx::query(
            "SELECT id, customer_id, from_account_id, to_account_id, amount::double precision, status, idempotency_key, created_at::text
             FROM transfers
             WHERE customer_id = $1 AND idempotency_key = $2"
        )
        .bind(customer_uuid)
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await
        .ok()?;

        row.map(Self::row_to_transfer)
    }

    pub async fn create_transfer(&self, input: CreateTransferInput) -> Transfer {
        let customer_uuid = Uuid::parse_str(&input.customer_id)
            .unwrap_or_else(|_| Uuid::new_v4());
        let from_uuid = Uuid::parse_str(input.from_account_id.strip_prefix("acc_").unwrap_or(&input.from_account_id))
            .unwrap_or_else(|_| Uuid::new_v4());
        let to_uuid = Uuid::parse_str(input.to_account_id.strip_prefix("acc_").unwrap_or(&input.to_account_id))
            .unwrap_or_else(|_| Uuid::new_v4());

        let row = sqlx::query(
            "INSERT INTO transfers (customer_id, from_account_id, to_account_id, amount, status, idempotency_key)
             VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, customer_id, from_account_id, to_account_id, amount::double precision, status, idempotency_key, created_at::text"
        )
        .bind(customer_uuid)
        .bind(from_uuid)
        .bind(to_uuid)
        .bind(input.amount)
        .bind(input.status)
        .bind(input.idempotency_key)
        .fetch_one(&self.pool)
        .await
        .expect("failed to create transfer in database");

        let mut transfer = Self::row_to_transfer(row);
        transfer.currency = input.currency;
        transfer
    }

    pub async fn find_by_id(&self, transfer_id: &str) -> Option<Transfer> {
        let transfer_uuid = Uuid::parse_str(transfer_id.strip_prefix("tr_").unwrap_or(transfer_id)).ok()?;

        let row = sqlx::query(
            "SELECT id, customer_id, from_account_id, to_account_id, amount::double precision, status, idempotency_key, created_at::text
             FROM transfers
             WHERE id = $1"
        )
        .bind(transfer_uuid)
        .fetch_optional(&self.pool)
        .await
        .ok()?;

        row.map(Self::row_to_transfer)
    }

    pub async fn list(
        &self,
        customer_id: Option<&str>,
        account_id: Option<&str>,
        limit: usize,
    ) -> Vec<Transfer> {
        let customer_uuid = customer_id.and_then(|id| Uuid::parse_str(id).ok());
        let account_uuid = account_id.and_then(|id| Uuid::parse_str(id.strip_prefix("acc_").unwrap_or(id)).ok());

        let rows = sqlx::query(
                        "SELECT id, customer_id, from_account_id, to_account_id, amount::double precision, status, idempotency_key, created_at::text
             FROM transfers
             WHERE ($1::uuid IS NULL OR customer_id = $1)
               AND ($2::uuid IS NULL OR from_account_id = $2 OR to_account_id = $2)
             ORDER BY created_at DESC
             LIMIT $3"
        )
        .bind(customer_uuid)
        .bind(account_uuid)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        rows.into_iter().map(Self::row_to_transfer).collect()
    }

    fn row_to_transfer(row: sqlx::postgres::PgRow) -> Transfer {
        let transfer_id = row.get::<Uuid, _>(0);
        let customer_id = row.get::<Uuid, _>(1);
        let from_account_id = row.get::<Uuid, _>(2);
        let to_account_id = row.get::<Uuid, _>(3);

        Transfer {
            transfer_id: format!("tr_{}", transfer_id),
            customer_id: customer_id.to_string(),
            from_account_id: format!("acc_{}", from_account_id),
            to_account_id: format!("acc_{}", to_account_id),
            amount: row.get(4),
            currency: "CAD".to_string(),
            status: row.get(5),
            idempotency_key: row.get(6),
            created_at: row.get(7),
        }
    }
}

impl Default for TransferRepository {
    fn default() -> Self {
        panic!("TransferRepository::default is not supported; use TransferRepository::new(pool)")
    }
}
