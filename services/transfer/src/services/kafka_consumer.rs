use log::{debug, error, info, warn};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::{ClientConfig, Message as KafkaMessage};
use reqwest::Client;
use serde::Deserialize;
use sqlx::PgPool;
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::config::env::AppConfig;
use crate::models::central_bank_events::*;
use crate::repositories::central_bank_repository::CentralBankRepository;
use crate::services::kafka_producer::CentralBankKafkaProducer;

/// Runs the central bank Kafka consumer loop. Should be spawned as a background task.
pub async fn run_central_bank_consumer(
    config: AppConfig,
    pool: PgPool,
    producer: CentralBankKafkaProducer,
) {
    let participant_id = config.central_bank_participant_id.clone();
    let group_id = format!("{}-canbankx-consumer", participant_id.to_lowercase());

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &config.central_bank_bootstrap_servers)
        .set("group.id", &group_id)
        .set("auto.offset.reset", "earliest")
        .set("enable.auto.commit", "true")
        .create()
        .expect("failed to create Kafka consumer");

    consumer
        .subscribe(&[&config.central_bank_topic])
        .expect("failed to subscribe to central bank topic");

    info!(
        "Central bank Kafka consumer started. Topic='{}', Broker='{}', Participant='{}', Group='{}'",
        config.central_bank_topic, config.central_bank_bootstrap_servers, participant_id, group_id
    );

    let repo = CentralBankRepository::new(pool);
    let http_client = Client::new();
    let account_service_url = config.account_service_base_url.clone();

    let mut stream = consumer.stream();

    while let Some(result) = stream.next().await {
        match result {
            Ok(msg) => {
                let payload_bytes = match msg.payload() {
                    Some(p) => p,
                    None => continue,
                };

                let envelope: KafkaEnvelope = match serde_json::from_slice(payload_bytes) {
                    Ok(e) => e,
                    Err(err) => {
                        warn!("Failed to parse Kafka envelope: {}", err);
                        continue;
                    }
                };

                let event_type = envelope.r#type.as_str();
                debug!("Received event type={}", event_type);

                if let Err(err) = dispatch(
                    event_type,
                    &envelope.payload,
                    &participant_id,
                    &repo,
                    &producer,
                    &http_client,
                    &account_service_url,
                )
                .await
                {
                    error!("Error handling event {}: {}", event_type, err);
                }
            }
            Err(err) => {
                error!("Kafka consumer error: {}", err);
            }
        }
    }
}

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

async fn dispatch(
    event_type: &str,
    payload: &serde_json::Value,
    participant_id: &str,
    repo: &CentralBankRepository,
    producer: &CentralBankKafkaProducer,
    http_client: &Client,
    account_service_url: &str,
) -> HandlerResult {
    match event_type {
        "AliasCreatedNotificationEvent" => {
            handle_alias_created_notification(payload, participant_id, repo).await
        }
        "AliasLookupResultEvent" => {
            handle_alias_lookup_result(payload, participant_id, repo).await
        }
        "AliasDeactivatedNotificationEvent" | "AliasDeletedNotificationEvent" => {
            handle_alias_deactivated_notification(payload, participant_id, repo).await
        }
        "AliasReactivatedNotificationEvent" => {
            handle_alias_reactivated_notification(payload, participant_id, repo).await
        }
        "AliasTransferredNotificationEvent" => {
            handle_alias_transferred_notification(payload, participant_id, repo).await
        }
        "AliasTransferRejectedEvent" => {
            handle_alias_transfer_rejected(payload, participant_id, repo).await
        }
        "ApproveAliasTransferRequestedEvent" => {
            handle_approve_alias_transfer_requested(payload, participant_id, repo).await
        }
        "ValidateDestinationEmailRequestedEvent" => {
            handle_validate_destination_email_requested(
                payload,
                participant_id,
                repo,
                producer,
            )
            .await
        }
        "PaymentSettledEvent" => {
            handle_payment_settled(
                payload,
                participant_id,
                repo,
                http_client,
                account_service_url,
            )
            .await
        }
        "PaymentRejectedEvent" => {
            handle_payment_rejected(
                payload,
                participant_id,
                repo,
                http_client,
                account_service_url,
            )
            .await
        }
        "PaymentExpiredEvent" => {
            handle_payment_expired(
                payload,
                participant_id,
                repo,
                http_client,
                account_service_url,
            )
            .await
        }
        "ValidateIncomingPaymentRequestedEvent" => {
            handle_validate_incoming_payment(
                payload,
                participant_id,
                producer,
                http_client,
                account_service_url,
            )
            .await
        }
        _ => {
            debug!("Ignored unknown event type={}", event_type);
            Ok(())
        }
    }
}

// ─── Alias lifecycle handlers ───────────────────────────────────────────────

async fn handle_alias_created_notification(
    payload: &serde_json::Value,
    participant_id: &str,
    repo: &CentralBankRepository,
) -> HandlerResult {
    let evt: AliasCreatedNotificationPayload = serde_json::from_value(payload.clone())?;
    if !evt.debtor_participant.eq_ignore_ascii_case(participant_id) {
        return Ok(());
    }

    repo.update_link_status(&evt.alias, "ACTIVE").await?;
    info!("Alias {} confirmed active by central bank", evt.alias);
    Ok(())
}

async fn handle_alias_lookup_result(
    payload: &serde_json::Value,
    participant_id: &str,
    repo: &CentralBankRepository,
) -> HandlerResult {
    let evt: AliasLookupResultPayload = serde_json::from_value(payload.clone())?;
    if !evt.debtor_participant.eq_ignore_ascii_case(participant_id) {
        return Ok(());
    }

    repo.upsert_alias_lookup(
        &evt.alias,
        evt.found,
        evt.creditor_participant.as_deref(),
        evt.masked_name.as_deref(),
    )
    .await?;

    info!(
        "Alias lookup result stored. Alias={}, Found={}, CorrelationId={}",
        evt.alias, evt.found, evt.correlation_id
    );
    Ok(())
}

async fn handle_alias_deactivated_notification(
    payload: &serde_json::Value,
    participant_id: &str,
    repo: &CentralBankRepository,
) -> HandlerResult {
    let evt: AliasDeactivatedNotificationPayload = serde_json::from_value(payload.clone())?;
    if !evt.debtor_participant.eq_ignore_ascii_case(participant_id) {
        return Ok(());
    }

    repo.update_link_status(&evt.alias, "DEACTIVATED").await?;
    info!("Alias {} deactivated", evt.alias);
    Ok(())
}

async fn handle_alias_reactivated_notification(
    payload: &serde_json::Value,
    participant_id: &str,
    repo: &CentralBankRepository,
) -> HandlerResult {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Evt {
        alias: String,
        debtor_participant: String,
    }

    let evt: Evt = serde_json::from_value(payload.clone())?;
    if !evt.debtor_participant.eq_ignore_ascii_case(participant_id) {
        return Ok(());
    }

    repo.update_link_status(&evt.alias, "ACTIVE").await?;
    info!("Alias {} reactivated", evt.alias);
    Ok(())
}

async fn handle_alias_transferred_notification(
    payload: &serde_json::Value,
    participant_id: &str,
    repo: &CentralBankRepository,
) -> HandlerResult {
    let evt: AliasTransferredNotificationPayload = serde_json::from_value(payload.clone())?;

    // We are the OLD owner - delete the alias locally
    if evt.debtor_participant.eq_ignore_ascii_case(participant_id) {
        repo.delete_link_by_alias(&evt.alias).await?;
        // Clean up any pending transfers for this alias
        repo.delete_pending_by_alias_value(&evt.alias).await?;
        info!(
            "Alias {} transferred away to {} — deleted locally",
            evt.alias, evt.new_debtor_participant
        );
    }

    // We are the NEW owner - create/activate alias locally
    if evt.new_debtor_participant.eq_ignore_ascii_case(participant_id) {
        // Get account from pending transfer or event
        let pending = repo.find_pending_by_alias_value(&evt.alias).await;
        let account_id = if let Some(ref p) = pending {
            p.account_id
        } else {
            Uuid::parse_str(&evt.new_account_id).unwrap_or_default()
        };

        // Delete old alias for this account first
        if let Some(old) = repo.find_link_by_account_id(account_id).await {
            repo.delete_link_by_alias(&old.alias).await.ok();
        }

        repo.create_link(&evt.alias, account_id, "", "ACTIVE")
            .await?;

        // Mark pending transfers as completed
        if let Some(p) = pending {
            repo.update_pending_status(&p.transfer_id, "COMPLETED", None)
                .await?;
        }

        info!(
            "Alias {} transferred in and activated for {}",
            evt.alias, participant_id
        );
    }

    Ok(())
}

async fn handle_alias_transfer_rejected(
    payload: &serde_json::Value,
    participant_id: &str,
    repo: &CentralBankRepository,
) -> HandlerResult {
    let evt: AliasTransferRejectedPayload = serde_json::from_value(payload.clone())?;
    if !evt.debtor_participant.eq_ignore_ascii_case(participant_id) {
        return Ok(());
    }

    if evt.reason == "ALIAS_ALREADY_EXISTS" {

        repo.delete_link_by_alias(&evt.alias).await?;
        info!(
            "Alias {} already exists at another bank — removed local record",
            evt.alias
        );
    } else {
        // Mark as deactivated / failed
        repo.update_link_status(&evt.alias, "DEACTIVATED").await?;
        warn!("Alias transfer rejected for {}: {}", evt.alias, evt.reason);
    }

    Ok(())
}

async fn handle_approve_alias_transfer_requested(
    payload: &serde_json::Value,
    participant_id: &str,
    repo: &CentralBankRepository,
) -> HandlerResult {
    let evt: ApproveAliasTransferRequestedPayload = serde_json::from_value(payload.clone())?;
    if !evt.debtor_participant.eq_ignore_ascii_case(participant_id) {
        return Ok(());
    }

    // Find the local alias to get the account_id
    let account_id = match repo.find_link_by_alias(&evt.alias).await {
        Some(link) => link.account_id,
        None => Uuid::nil(),
    };

    // Remove existing pending for same alias to avoid duplicates
    repo.delete_pending_by_alias_value(&evt.alias).await?;

    repo.create_pending_transfer(
        &evt.transfer_id,
        &evt.alias,
        account_id,
        &evt.debtor_participant,
        &evt.new_debtor_participant,
        None,
        "PENDING",
    )
    .await?;

    info!(
        "Owner approval requested for alias={} transfer={} to {}",
        evt.alias, evt.transfer_id, evt.new_debtor_participant
    );
    Ok(())
}

// ─── Destination email validation ───────────────────────────────────────────

async fn handle_validate_destination_email_requested(
    payload: &serde_json::Value,
    participant_id: &str,
    repo: &CentralBankRepository,
    producer: &CentralBankKafkaProducer,
) -> HandlerResult {
    let evt: ValidateDestinationEmailRequestedPayload = serde_json::from_value(payload.clone())?;
    if !evt.new_debtor_participant.eq_ignore_ascii_case(participant_id) {
        return Ok(());
    }

    // Check if we have a pending transfer request 
    let pending = repo.find_pending_by_transfer_id(&evt.transfer_id).await
        .or(repo.find_pending_by_alias_value(&evt.email).await);
    if let Some(p) = pending {
        producer
            .publish_destination_email_validated(
                &evt.transfer_id,
                participant_id,
                &p.account_id.to_string(),
                &evt.email,
            )
            .await?;
        info!("ValidateDestinationEmail: alias={} validated via pending transfer", evt.email);
        return Ok(());
    }

    // Fallback: check if we have this alias locally
    let local_alias = repo.find_link_by_alias(&evt.email).await;
    match local_alias {
        Some(link) if link.status == "ACTIVE" => {
            producer
                .publish_destination_email_validated(
                    &evt.transfer_id,
                    participant_id,
                    &link.account_id.to_string(),
                    &evt.email,
                )
                .await?;
            info!("ValidateDestinationEmail: alias={} validated via local alias", evt.email);
        }
        _ => {
            producer
                .publish_destination_email_invalid(&evt.transfer_id, "ALIAS_NOT_FOUND_OR_INACTIVE")
                .await?;
            info!("ValidateDestinationEmail: alias={} not found — invalid", evt.email);
        }
    }

    Ok(())
}

// ─── Payment handlers ───────────────────────────────────────────────────────

async fn handle_payment_settled(
    payload: &serde_json::Value,
    participant_id: &str,
    _repo: &CentralBankRepository,
    http_client: &Client,
    account_service_url: &str,
) -> HandlerResult {
    let evt: PaymentSettledPayload = serde_json::from_value(payload.clone())?;

    let is_debtor = evt.debtor_participant.eq_ignore_ascii_case(participant_id);
    let is_creditor = evt.creditor_participant.eq_ignore_ascii_case(participant_id);

    if !is_debtor && !is_creditor {
        return Ok(());
    }

    // We are the CREDITOR — credit the destination account
    if is_creditor {
        let creditor_account_id = &evt.creditor_account_id;
        credit_account(
            http_client,
            account_service_url,
            creditor_account_id,
            evt.amount,
            &format!("cb-settled-{}-credit", evt.payment_id),
        )
        .await?;
        info!(
            "PaymentSettled: credited account {} with {} for PaymentId={}",
            creditor_account_id, evt.amount, evt.payment_id
        );
    }

    if is_debtor {
        info!(
            "PaymentSettled: payment {} settled by central bank (debtor side)",
            evt.payment_id
        );
    }

    Ok(())
}

async fn handle_payment_rejected(
    payload: &serde_json::Value,
    participant_id: &str,
    _repo: &CentralBankRepository,
    _http_client: &Client,
    _account_service_url: &str,
) -> HandlerResult {
    let evt: PaymentRejectedPayload = serde_json::from_value(payload.clone())?;
    if !evt.debtor_participant.eq_ignore_ascii_case(participant_id) {
        return Ok(());
    }

    warn!(
        "PaymentRejected: payment {} rejected: {}",
        evt.payment_id, evt.reason
    );
    // Note: refund logic would require tracking from_account_id per payment_id.
    // In the simple model, the REST caller gets the rejection async.
    Ok(())
}

async fn handle_payment_expired(
    payload: &serde_json::Value,
    participant_id: &str,
    _repo: &CentralBankRepository,
    _http_client: &Client,
    _account_service_url: &str,
) -> HandlerResult {
    let evt: PaymentExpiredPayload = serde_json::from_value(payload.clone())?;
    if !evt.debtor_participant.eq_ignore_ascii_case(participant_id) {
        return Ok(());
    }

    warn!("PaymentExpired: payment {} expired", evt.payment_id);
    // Note: refund logic would require tracking from_account_id per payment_id.
    Ok(())
}

async fn handle_validate_incoming_payment(
    payload: &serde_json::Value,
    participant_id: &str,
    producer: &CentralBankKafkaProducer,
    http_client: &Client,
    account_service_url: &str,
) -> HandlerResult {
    let evt: ValidateIncomingPaymentRequestedPayload = serde_json::from_value(payload.clone())?;
    if !evt.creditor_participant.eq_ignore_ascii_case(participant_id) {
        return Ok(());
    }

    // Check if the account exists and is active via account-service
    let account_exists = check_account_exists(
        http_client,
        account_service_url,
        &evt.account_id,
    )
    .await;

    match account_exists {
        Ok(true) => {
            producer
                .publish_incoming_payment_validated(&evt.payment_id, "ACCEPTED", None)
                .await?;
            info!(
                "IncomingPayment {}: account {} validated — ACCEPTED",
                evt.payment_id, evt.account_id
            );
        }
        Ok(false) => {
            producer
                .publish_incoming_payment_validated(
                    &evt.payment_id,
                    "REJECTED",
                    Some("ACCOUNT_CLOSED"),
                )
                .await?;
            warn!(
                "IncomingPayment {}: account {} closed — REJECTED",
                evt.payment_id, evt.account_id
            );
        }
        Err(_) => {
            producer
                .publish_incoming_payment_validated(
                    &evt.payment_id,
                    "REJECTED",
                    Some("ACCOUNT_NOT_FOUND"),
                )
                .await?;
            warn!(
                "IncomingPayment {}: account {} not found — REJECTED",
                evt.payment_id, evt.account_id
            );
        }
    }

    Ok(())
}

// ─── HTTP helpers ───────────────────────────────────────────────────────────

async fn credit_account(
    http_client: &Client,
    account_service_url: &str,
    account_id: &str,
    amount: f64,
    _idempotency_key: &str,
) -> HandlerResult {
    let url = format!(
        "{}/api/v1/accounts/credit",
        account_service_url.trim_end_matches('/')
    );

    let prefixed = if account_id.starts_with("acc_") {
        account_id.to_string()
    } else {
        format!("acc_{}", account_id)
    };

    let body = serde_json::json!({
        "account_id": prefixed,
        "amount": amount
    });

    let response = http_client.post(&url).json(&body).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        error!(
            "Failed to credit account {}: {} - {}",
            account_id, status, text
        );
        return Err(format!("credit failed: {} {}", status, text).into());
    }

    Ok(())
}

async fn check_account_exists(
    http_client: &Client,
    account_service_url: &str,
    account_id: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!(
        "{}/api/v1/accounts/balance",
        account_service_url.trim_end_matches('/')
    );

    let prefixed = if account_id.starts_with("acc_") {
        account_id.to_string()
    } else {
        format!("acc_{}", account_id)
    };

    let response = http_client
        .get(&url)
        .query(&[("account_id", &prefixed)])
        .send()
        .await?;

    if response.status().is_success() {
        Ok(true)
    } else if response.status() == reqwest::StatusCode::NOT_FOUND {
        Err("account not found".into())
    } else {
        Ok(false)
    }
}
