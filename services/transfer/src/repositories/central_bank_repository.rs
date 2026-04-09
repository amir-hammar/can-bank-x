use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct CentralBankRepository {
    pool: PgPool,
}

// ─── Payment Links ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PaymentLink {
    pub id: Uuid,
    pub alias: String,
    pub account_id: Uuid,
    pub holder_name: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

// ─── Aliases (lookup cache) ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CachedAliasLookup {
    pub id: Uuid,
    pub alias: String,
    pub creditor_participant: Option<String>,
    pub masked_name: Option<String>,
    pub found: bool,
    pub looked_up_at: String,
}

// ─── Pending Alias Transfers ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PendingAliasTransfer {
    pub id: Uuid,
    pub transfer_id: String,
    pub alias_value: String,
    pub account_id: Uuid,
    pub debtor_participant: String,
    pub new_debtor_participant: String,
    pub destination_email: Option<String>,
    pub status: String,
    pub reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl CentralBankRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ── Payment Links ───────────────────────────────────────────────────────

    pub async fn create_link(
        &self,
        alias: &str,
        account_id: Uuid,
        holder_name: &str,
        status: &str,
    ) -> Result<PaymentLink, sqlx::Error> {
        let row = sqlx::query(
            "INSERT INTO central_bank_payment_links (alias, account_id, holder_name, status)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (alias) DO UPDATE SET
                account_id = EXCLUDED.account_id,
                holder_name = EXCLUDED.holder_name,
                status = EXCLUDED.status
             RETURNING id, alias, account_id, holder_name, status, created_at::text, updated_at::text",
        )
        .bind(alias)
        .bind(account_id)
        .bind(holder_name)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::row_to_payment_link(row))
    }

    pub async fn find_link_by_alias(&self, alias: &str) -> Option<PaymentLink> {
        let row = sqlx::query(
            "SELECT id, alias, account_id, holder_name, status, created_at::text, updated_at::text
             FROM central_bank_payment_links
             WHERE alias = $1",
        )
        .bind(alias)
        .fetch_optional(&self.pool)
        .await
        .ok()?;

        row.map(Self::row_to_payment_link)
    }

    pub async fn find_link_by_account_id(&self, account_id: Uuid) -> Option<PaymentLink> {
        let row = sqlx::query(
            "SELECT id, alias, account_id, holder_name, status, created_at::text, updated_at::text
             FROM central_bank_payment_links
             WHERE account_id = $1",
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .ok()?;

        row.map(Self::row_to_payment_link)
    }

    pub async fn update_link_status(
        &self,
        alias: &str,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE central_bank_payment_links SET status = $1 WHERE alias = $2",
        )
        .bind(status)
        .bind(alias)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_link_by_alias(&self, alias: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM central_bank_payment_links WHERE alias = $1")
            .bind(alias)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn list_links_by_account(&self, account_id: Uuid) -> Vec<PaymentLink> {
        let rows = sqlx::query(
            "SELECT id, alias, account_id, holder_name, status, created_at::text, updated_at::text
             FROM central_bank_payment_links
             WHERE account_id = $1
             ORDER BY created_at DESC",
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        rows.into_iter().map(Self::row_to_payment_link).collect()
    }

    pub async fn list_all_active_links(&self) -> Vec<PaymentLink> {
        let rows = sqlx::query(
            "SELECT id, alias, account_id, holder_name, status, created_at::text, updated_at::text
             FROM central_bank_payment_links
             WHERE status = 'ACTIVE'
             ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        rows.into_iter().map(Self::row_to_payment_link).collect()
    }

    // ── Alias Lookups (cache) ───────────────────────────────────────────────

    pub async fn upsert_alias_lookup(
        &self,
        alias: &str,
        found: bool,
        creditor_participant: Option<&str>,
        masked_name: Option<&str>,
    ) -> Result<CachedAliasLookup, sqlx::Error> {
        let row = sqlx::query(
            "INSERT INTO central_bank_aliases (alias, found, creditor_participant, masked_name)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (alias) DO UPDATE SET
                found = EXCLUDED.found,
                creditor_participant = EXCLUDED.creditor_participant,
                masked_name = EXCLUDED.masked_name,
                looked_up_at = NOW()
             RETURNING id, alias, creditor_participant, masked_name, found, looked_up_at::text",
        )
        .bind(alias)
        .bind(found)
        .bind(creditor_participant)
        .bind(masked_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::row_to_alias_lookup(row))
    }

    pub async fn find_alias_lookup(&self, alias: &str) -> Option<CachedAliasLookup> {
        let row = sqlx::query(
            "SELECT id, alias, creditor_participant, masked_name, found, looked_up_at::text
             FROM central_bank_aliases
             WHERE alias = $1",
        )
        .bind(alias)
        .fetch_optional(&self.pool)
        .await
        .ok()?;

        row.map(Self::row_to_alias_lookup)
    }

    pub async fn delete_alias_lookup(&self, alias: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM central_bank_aliases WHERE alias = $1")
            .bind(alias)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // ── Pending Alias Transfers ─────────────────────────────────────────────

    pub async fn create_pending_transfer(
        &self,
        transfer_id: &str,
        alias_value: &str,
        account_id: Uuid,
        debtor_participant: &str,
        new_debtor_participant: &str,
        destination_email: Option<&str>,
        status: &str,
    ) -> Result<PendingAliasTransfer, sqlx::Error> {
        let row = sqlx::query(
            "INSERT INTO central_bank_pending_alias_transfers
                (transfer_id, alias_value, account_id, debtor_participant, new_debtor_participant, destination_email, status)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (transfer_id) DO UPDATE SET
                status = EXCLUDED.status,
                destination_email = EXCLUDED.destination_email
             RETURNING id, transfer_id, alias_value, account_id, debtor_participant, new_debtor_participant,
                       destination_email, status, reason, created_at::text, updated_at::text",
        )
        .bind(transfer_id)
        .bind(alias_value)
        .bind(account_id)
        .bind(debtor_participant)
        .bind(new_debtor_participant)
        .bind(destination_email)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::row_to_pending_transfer(row))
    }

    pub async fn find_pending_by_transfer_id(
        &self,
        transfer_id: &str,
    ) -> Option<PendingAliasTransfer> {
        let row = sqlx::query(
            "SELECT id, transfer_id, alias_value, account_id, debtor_participant, new_debtor_participant,
                    destination_email, status, reason, created_at::text, updated_at::text
             FROM central_bank_pending_alias_transfers
             WHERE transfer_id = $1",
        )
        .bind(transfer_id)
        .fetch_optional(&self.pool)
        .await
        .ok()?;

        row.map(Self::row_to_pending_transfer)
    }

    pub async fn find_pending_by_account(
        &self,
        account_id: Uuid,
        status: &str,
    ) -> Vec<PendingAliasTransfer> {
        let rows = sqlx::query(
            "SELECT id, transfer_id, alias_value, account_id, debtor_participant, new_debtor_participant,
                    destination_email, status, reason, created_at::text, updated_at::text
             FROM central_bank_pending_alias_transfers
             WHERE account_id = $1 AND status = $2
             ORDER BY created_at DESC",
        )
        .bind(account_id)
        .bind(status)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        rows.into_iter()
            .map(Self::row_to_pending_transfer)
            .collect()
    }

    pub async fn find_all_pending(&self) -> Vec<PendingAliasTransfer> {
        let rows = sqlx::query(
            "SELECT id, transfer_id, alias_value, account_id, debtor_participant, new_debtor_participant,
                    destination_email, status, reason, created_at::text, updated_at::text
             FROM central_bank_pending_alias_transfers
             WHERE status = 'PENDING'
             ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        rows.into_iter()
            .map(Self::row_to_pending_transfer)
            .collect()
    }

    pub async fn find_pending_by_alias_value(
        &self,
        alias_value: &str,
    ) -> Option<PendingAliasTransfer> {
        let row = sqlx::query(
            "SELECT id, transfer_id, alias_value, account_id, debtor_participant, new_debtor_participant,
                    destination_email, status, reason, created_at::text, updated_at::text
             FROM central_bank_pending_alias_transfers
             WHERE alias_value = $1 AND status = 'PENDING'
             ORDER BY created_at DESC
             LIMIT 1",
        )
        .bind(alias_value)
        .fetch_optional(&self.pool)
        .await
        .ok()?;

        row.map(Self::row_to_pending_transfer)
    }

    pub async fn update_pending_status(
        &self,
        transfer_id: &str,
        status: &str,
        reason: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE central_bank_pending_alias_transfers
             SET status = $1, reason = $2
             WHERE transfer_id = $3",
        )
        .bind(status)
        .bind(reason)
        .bind(transfer_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_stale_pending(
        &self,
        cutoff: &str,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM central_bank_pending_alias_transfers
             WHERE status = 'PENDING' AND created_at < $1::timestamp",
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn delete_pending_by_alias_value(
        &self,
        alias_value: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM central_bank_pending_alias_transfers WHERE alias_value = $1",
        )
        .bind(alias_value)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ── Row mappers ─────────────────────────────────────────────────────────

    fn row_to_payment_link(row: sqlx::postgres::PgRow) -> PaymentLink {
        PaymentLink {
            id: row.get::<Uuid, _>(0),
            alias: row.get(1),
            account_id: row.get::<Uuid, _>(2),
            holder_name: row.get(3),
            status: row.get(4),
            created_at: row.get(5),
            updated_at: row.get(6),
        }
    }

    fn row_to_alias_lookup(row: sqlx::postgres::PgRow) -> CachedAliasLookup {
        CachedAliasLookup {
            id: row.get::<Uuid, _>(0),
            alias: row.get(1),
            creditor_participant: row.get(2),
            masked_name: row.get(3),
            found: row.get(4),
            looked_up_at: row.get(5),
        }
    }

    fn row_to_pending_transfer(row: sqlx::postgres::PgRow) -> PendingAliasTransfer {
        PendingAliasTransfer {
            id: row.get::<Uuid, _>(0),
            transfer_id: row.get(1),
            alias_value: row.get(2),
            account_id: row.get::<Uuid, _>(3),
            debtor_participant: row.get(4),
            new_debtor_participant: row.get(5),
            destination_email: row.get(6),
            status: row.get(7),
            reason: row.get(8),
            created_at: row.get(9),
            updated_at: row.get(10),
        }
    }
}
