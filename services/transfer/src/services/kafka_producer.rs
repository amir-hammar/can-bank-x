use log::{error, info};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use serde::Serialize;
use std::time::Duration;

use crate::models::central_bank_events::*;

#[derive(Clone)]
pub struct CentralBankKafkaProducer {
    producer: FutureProducer,
    topic: String,
    participant_id: String,
}

impl CentralBankKafkaProducer {
    pub fn new(bootstrap_servers: &str, topic: &str, participant_id: &str) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("failed to create Kafka producer");

        Self {
            producer,
            topic: topic.to_string(),
            participant_id: participant_id.to_string(),
        }
    }

    async fn publish(
        &self,
        event_type: &str,
        payload: impl Serialize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let envelope = serde_json::json!({
            "type": event_type,
            "payload": serde_json::to_value(&payload)?
        });

        let envelope_str = serde_json::to_string(&envelope)?;
        let key = uuid::Uuid::new_v4().to_string();

        let record = FutureRecord::to(&self.topic)
            .key(&key)
            .payload(&envelope_str);

        match self.producer.send(record, Duration::from_secs(5)).await {
            Ok(_) => {
                info!("Published {} to {}", event_type, self.topic);
                Ok(())
            }
            Err((err, _)) => {
                error!("Failed to publish {} to {}: {}", event_type, self.topic, err);
                Err(Box::new(err))
            }
        }
    }

    fn now_utc() -> String {
        chrono::Utc::now().to_rfc3339()
    }

    // ── Alias events ────────────────────────────────────────────────────────

    pub async fn publish_alias_creation_requested(
        &self,
        alias: &str,
        account_id: &str,
        holder_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.publish(
            "AliasCreationRequestedEvent",
            AliasCreationRequestedPayload {
                alias: alias.to_string(),
                debtor_participant: self.participant_id.clone(),
                account_id: account_id.to_string(),
                holder_name: holder_name.to_string(),
                occurred_at: Self::now_utc(),
            },
        )
        .await
    }

    pub async fn publish_alias_lookup_requested(
        &self,
        alias: &str,
        correlation_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.publish(
            "AliasLookupRequestedEvent",
            AliasLookupRequestedPayload {
                alias: alias.to_string(),
                debtor_participant: self.participant_id.clone(),
                occurred_at: Self::now_utc(),
                correlation_id: correlation_id.to_string(),
            },
        )
        .await
    }

    pub async fn publish_alias_delete_requested(
        &self,
        alias: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.publish(
            "AliasDeleteRequestedEvent",
            AliasDeleteRequestedPayload {
                alias: alias.to_string(),
                debtor_participant: self.participant_id.clone(),
                occurred_at: Self::now_utc(),
            },
        )
        .await
    }

    pub async fn publish_alias_transfer_requested(
        &self,
        transfer_id: &str,
        account_id: &str,
        debtor_participant: &str,
        new_debtor_participant: &str,
        destination_email: &str,
        alias_value: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.publish(
            "AliasTransferRequestedEvent",
            AliasTransferRequestedPayload {
                transfer_id: transfer_id.to_string(),
                account_id: account_id.to_string(),
                debtor_participant: debtor_participant.to_string(),
                new_debtor_participant: new_debtor_participant.to_string(),
                destination_email: destination_email.to_string(),
                alias_value: alias_value.to_string(),
                occurred_at: Self::now_utc(),
            },
        )
        .await
    }

    pub async fn publish_alias_transfer_approved(
        &self,
        transfer_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.publish(
            "AliasTransferApprovedEvent",
            AliasTransferApprovedPayload {
                transfer_id: transfer_id.to_string(),
                debtor_participant: self.participant_id.clone(),
                occurred_at: Self::now_utc(),
            },
        )
        .await
    }

    pub async fn publish_alias_transfer_denied(
        &self,
        transfer_id: &str,
        reason: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.publish(
            "AliasTransferDeniedEvent",
            AliasTransferDeniedPayload {
                transfer_id: transfer_id.to_string(),
                debtor_participant: self.participant_id.clone(),
                reason: reason.to_string(),
                occurred_at: Self::now_utc(),
            },
        )
        .await
    }

    // ── Destination email validation ────────────────────────────────────────

    pub async fn publish_destination_email_validated(
        &self,
        transfer_id: &str,
        new_debtor_participant: &str,
        new_account_id: &str,
        destination_email: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.publish(
            "DestinationEmailValidatedEvent",
            DestinationEmailValidatedPayload {
                transfer_id: transfer_id.to_string(),
                new_debtor_participant: new_debtor_participant.to_string(),
                new_account_id: new_account_id.to_string(),
                destination_email: destination_email.to_string(),
                occurred_at: Self::now_utc(),
            },
        )
        .await
    }

    pub async fn publish_destination_email_invalid(
        &self,
        transfer_id: &str,
        reason: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.publish(
            "DestinationEmailInvalidEvent",
            DestinationEmailInvalidPayload {
                transfer_id: transfer_id.to_string(),
                reason: reason.to_string(),
                occurred_at: Self::now_utc(),
            },
        )
        .await
    }

    // ── Payment events ──────────────────────────────────────────────────────

    pub async fn publish_payment_initiated(
        &self,
        payment_id: &str,
        alias: &str,
        amount: f64,
        currency: &str,
        idempotency_key: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.publish(
            "PaymentInitiatedEvent",
            PaymentInitiatedPayload {
                payment_id: payment_id.to_string(),
                alias: alias.to_string(),
                amount,
                currency: currency.to_string(),
                debtor_participant: self.participant_id.clone(),
                occurred_at: Self::now_utc(),
                idempotency_key: idempotency_key.to_string(),
            },
        )
        .await
    }

    pub async fn publish_incoming_payment_validated(
        &self,
        payment_id: &str,
        decision: &str,
        reason: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.publish(
            "IncomingPaymentValidatedEvent",
            IncomingPaymentValidatedPayload {
                payment_id: payment_id.to_string(),
                decision: decision.to_string(),
                reason: reason.map(|s| s.to_string()),
                occurred_at: Self::now_utc(),
            },
        )
        .await
    }

    pub fn participant_id(&self) -> &str {
        &self.participant_id
    }
}
