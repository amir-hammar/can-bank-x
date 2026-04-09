use log::{info, warn};
use reqwest::Client;
use serde::Serialize;
use uuid::Uuid;

use crate::repositories::central_bank_repository::{
    CentralBankRepository, PaymentLink, PendingAliasTransfer,
};
use crate::services::kafka_producer::CentralBankKafkaProducer;
use crate::utils::errors::AppError;

#[derive(Clone)]
pub struct CentralBankService {
    pub producer: CentralBankKafkaProducer,
    pub repo: CentralBankRepository,
    pub participant_id: String,
    pub payment_service_url: String,
    http_client: Client,
}

#[derive(Debug, Serialize)]
pub struct LookupResult {
    pub alias: String,
    pub found: bool,
    pub creditor_participant: Option<String>,
    pub masked_name: Option<String>,
}

impl CentralBankService {
    pub fn new(
        producer: CentralBankKafkaProducer,
        repo: CentralBankRepository,
        participant_id: String,
        payment_service_url: String,
    ) -> Self {
        Self {
            producer,
            repo,
            participant_id,
            payment_service_url,
            http_client: Client::new(),
        }
    }

    /// Register an alias locally (PENDING) and publish AliasCreationRequestedEvent.
    pub async fn register_alias(
        &self,
        alias: &str,
        account_id: &str,
        _customer_id: &str,
        holder_name: &str,
    ) -> Result<PaymentLink, AppError> {
        let account_uuid = Uuid::parse_str(account_id).map_err(|_| {
            AppError::bad_request("INVALID_ACCOUNT_ID", "account_id must be a valid UUID")
        })?;

        // Delete old alias for this account if exists
        if let Some(old) = self.repo.find_link_by_account_id(account_uuid).await {
            self.producer.publish_alias_delete_requested(&old.alias).await.ok();
            self.repo.delete_link_by_alias(&old.alias).await.ok();
        }

        let link = self
            .repo
            .create_link(alias, account_uuid, holder_name, "PENDING")
            .await
            .map_err(|e| {
                AppError::internal("DB_ERROR", format!("failed to create alias link: {}", e))
            })?;

        self.producer
            .publish_alias_creation_requested(alias, account_id, holder_name)
            .await
            .map_err(|e| {
                AppError::internal("KAFKA_ERROR", format!("failed to publish alias creation: {}", e))
            })?;

        info!("Alias registration requested: alias={}, account={}", alias, account_id);
        Ok(link)
    }

    /// Publish AliasDeleteRequestedEvent and delete locally.
    pub async fn delete_alias(&self, alias: &str) -> Result<(), AppError> {
        self.producer
            .publish_alias_delete_requested(alias)
            .await
            .map_err(|e| {
                AppError::internal("KAFKA_ERROR", format!("failed to publish alias deletion: {}", e))
            })?;

        self.repo.delete_link_by_alias(alias).await.map_err(|e| {
            AppError::internal("DB_ERROR", format!("failed to delete alias link: {}", e))
        })?;

        info!("Alias deletion requested: alias={}", alias);
        Ok(())
    }

    /// Publish AliasLookupRequestedEvent, then poll DB for result with 3s timeout.
    pub async fn lookup_alias(&self, alias: &str) -> Result<Option<LookupResult>, AppError> {
        // Delete stale lookup cache for this alias so we get a fresh result
        let _ = self.repo.delete_alias_lookup(alias).await;

        let correlation_id = Uuid::new_v4().to_string();

        self.producer
            .publish_alias_lookup_requested(alias, &correlation_id)
            .await
            .map_err(|e| {
                AppError::internal("KAFKA_ERROR", format!("failed to publish alias lookup: {}", e))
            })?;

        // Poll DB for the lookup result with 3s timeout
        let timeout = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            if tokio::time::Instant::now() >= timeout {
                warn!("Alias lookup timed out for alias={}", alias);
                return Ok(None);
            }

            if let Some(cached) = self.repo.find_alias_lookup(alias).await {
                return Ok(Some(LookupResult {
                    alias: cached.alias,
                    found: cached.found,
                    creditor_participant: cached.creditor_participant,
                    masked_name: cached.masked_name,
                }));
            }

            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    }

    /// Save payment link and publish PaymentInitiatedEvent.
    pub async fn initiate_payment(
        &self,
        _customer_id: &str,
        source_account_id: &str,
        alias: &str,
        amount: f64,
        currency: &str,
        idempotency_key: &str,
    ) -> Result<serde_json::Value, AppError> {
        let payment_id = Uuid::new_v4().to_string();

        self.producer
            .publish_payment_initiated(&payment_id, alias, amount, currency, idempotency_key)
            .await
            .map_err(|e| {
                AppError::internal("KAFKA_ERROR", format!("failed to publish payment: {}", e))
            })?;

        info!(
            "Payment initiated: payment_id={}, alias={}, amount={} {}, source={}",
            payment_id, alias, amount, currency, source_account_id
        );

        Ok(serde_json::json!({
            "payment_id": payment_id,
            "status": "INITIATED",
            "alias": alias,
            "amount": amount,
            "currency": currency,
            "source_account_id": source_account_id,
        }))
    }

    /// Publish AliasTransferRequestedEvent.
    pub async fn request_alias_transfer(
        &self,
        alias: &str,
        receiving_account_id: &str,
    ) -> Result<serde_json::Value, AppError> {
        let transfer_id = Uuid::new_v4().to_string();

        // Lookup who owns the alias to set correct debtorParticipant
        let lookup = self.lookup_alias(alias).await?;
        let current_owner = lookup
            .and_then(|r| r.creditor_participant)
            .unwrap_or_else(|| "unknown".to_string());

        self.producer
            .publish_alias_transfer_requested(
                &transfer_id,
                receiving_account_id,
                &current_owner,
                &self.participant_id,
                alias,
                alias,
            )
            .await
            .map_err(|e| {
                AppError::internal(
                    "KAFKA_ERROR",
                    format!("failed to publish alias transfer request: {}", e),
                )
            })?;

        // Save pending transfer so validation handler can find it
        let account_uuid = uuid::Uuid::parse_str(receiving_account_id).unwrap_or_default();
        self.repo.create_pending_transfer(
            &transfer_id,
            alias,
            account_uuid,
            &current_owner,
            &self.participant_id,
            Some(alias),
            "PENDING",
        ).await.ok();

        info!(
            "Alias transfer requested: transfer_id={}, alias={}",
            transfer_id, alias
        );

        Ok(serde_json::json!({
            "transfer_id": transfer_id,
            "alias": alias,
            "status": "REQUESTED",
        }))
    }

    /// Publish AliasTransferApprovedEvent.
    pub async fn approve_transfer(&self, transfer_id: &str) -> Result<(), AppError> {
        self.producer
            .publish_alias_transfer_approved(transfer_id)
            .await
            .map_err(|e| {
                AppError::internal(
                    "KAFKA_ERROR",
                    format!("failed to publish transfer approval: {}", e),
                )
            })?;

        // Update local pending status
        let _ = self
            .repo
            .update_pending_status(transfer_id, "APPROVED", None)
            .await;

        info!("Alias transfer approved: transfer_id={}", transfer_id);
        Ok(())
    }

    /// Publish AliasTransferDeniedEvent.
    pub async fn deny_transfer(
        &self,
        transfer_id: &str,
        reason: &str,
    ) -> Result<(), AppError> {
        self.producer
            .publish_alias_transfer_denied(transfer_id, reason)
            .await
            .map_err(|e| {
                AppError::internal(
                    "KAFKA_ERROR",
                    format!("failed to publish transfer denial: {}", e),
                )
            })?;

        // Update local pending status
        let _ = self
            .repo
            .update_pending_status(transfer_id, "DENIED", Some(reason))
            .await;

        info!(
            "Alias transfer denied: transfer_id={}, reason={}",
            transfer_id, reason
        );
        Ok(())
    }

    /// List aliases for an account.
    pub async fn list_aliases(&self, account_id: &str) -> Result<Vec<PaymentLink>, AppError> {
        if account_id.is_empty() {
            return Ok(self.repo.list_all_active_links().await);
        }
        let account_uuid = Uuid::parse_str(account_id).map_err(|_| {
            AppError::bad_request("INVALID_ACCOUNT_ID", "account_id must be a valid UUID")
        })?;
        Ok(self.repo.list_links_by_account(account_uuid).await)
    }

    /// List pending alias transfers for an account.
    pub async fn list_pending_transfers(
        &self,
        account_id: &str,
    ) -> Result<Vec<PendingAliasTransfer>, AppError> {
        // Clean up stale pending transfers older than 2 minutes
        let cutoff = (chrono::Utc::now() - chrono::Duration::seconds(120)).format("%Y-%m-%d %H:%M:%S").to_string();
        self.repo.delete_stale_pending(&cutoff).await.ok();

        if account_id.is_empty() {
            return Ok(self.repo.find_all_pending().await);
        }
        let account_uuid = Uuid::parse_str(account_id).map_err(|_| {
            AppError::bad_request("INVALID_ACCOUNT_ID", "account_id must be a valid UUID")
        })?;

        Ok(self
            .repo
            .find_pending_by_account(account_uuid, "PENDING")
            .await)
    }

    /// HTTP GET to central bank settlement endpoint via KrakenD.
    pub async fn get_settlement(
        &self,
        from: &str,
        to: &str,
    ) -> Result<serde_json::Value, AppError> {
        let url = format!(
            "{}/settlements",
            self.payment_service_url.trim_end_matches('/')
        );

        let response = self
            .http_client
            .get(&url)
            .query(&[("from", from), ("to", to)])
            .send()
            .await
            .map_err(|e| {
                AppError::failed_dependency(
                    "CENTRAL_BANK_UNAVAILABLE",
                    format!("failed to reach central bank settlement endpoint: {}", e),
                )
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::failed_dependency(
                "SETTLEMENT_QUERY_FAILED",
                format!("settlement query failed: {} {}", status, text),
            ));
        }

        let body: serde_json::Value = response.json().await.map_err(|e| {
            AppError::internal(
                "PARSE_ERROR",
                format!("failed to parse settlement response: {}", e),
            )
        })?;

        Ok(body)
    }
}
