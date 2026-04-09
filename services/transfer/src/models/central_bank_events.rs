use serde::{Deserialize, Serialize};

// ─── Generic Kafka envelope ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaEnvelope {
    pub r#type: String,
    pub payload: serde_json::Value,
}

// ─── Publish (outgoing) event payloads ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AliasCreationRequestedPayload {
    pub alias: String,
    pub debtor_participant: String,
    pub account_id: String,
    pub holder_name: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AliasLookupRequestedPayload {
    pub alias: String,
    pub debtor_participant: String,
    pub occurred_at: String,
    pub correlation_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AliasDeleteRequestedPayload {
    pub alias: String,
    pub debtor_participant: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AliasTransferRequestedPayload {
    pub transfer_id: String,
    pub account_id: String,
    pub debtor_participant: String,
    pub new_debtor_participant: String,
    pub destination_email: String,
    pub alias_value: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AliasTransferApprovedPayload {
    pub transfer_id: String,
    pub debtor_participant: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AliasTransferDeniedPayload {
    pub transfer_id: String,
    pub debtor_participant: String,
    pub reason: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DestinationEmailValidatedPayload {
    pub transfer_id: String,
    pub new_debtor_participant: String,
    pub new_account_id: String,
    pub destination_email: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DestinationEmailInvalidPayload {
    pub transfer_id: String,
    pub reason: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentInitiatedPayload {
    pub payment_id: String,
    pub alias: String,
    pub amount: f64,
    pub currency: String,
    pub debtor_participant: String,
    pub occurred_at: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomingPaymentValidatedPayload {
    pub payment_id: String,
    pub decision: String,
    pub reason: Option<String>,
    pub occurred_at: String,
}

// ─── Consume (incoming) event payloads ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AliasCreatedNotificationPayload {
    pub alias: String,
    pub debtor_participant: String,
    pub account_id: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AliasLookupResultPayload {
    pub alias: String,
    pub debtor_participant: String,
    pub found: bool,
    pub creditor_participant: Option<String>,
    pub masked_name: Option<String>,
    pub reason: Option<String>,
    pub occurred_at: String,
    pub correlation_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AliasDeactivatedNotificationPayload {
    pub alias: String,
    pub debtor_participant: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AliasTransferredNotificationPayload {
    pub alias: String,
    pub debtor_participant: String,
    pub new_debtor_participant: String,
    pub new_account_id: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AliasTransferRejectedPayload {
    pub alias: String,
    pub debtor_participant: String,
    pub reason: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApproveAliasTransferRequestedPayload {
    pub transfer_id: String,
    pub alias: String,
    pub debtor_participant: String,
    pub new_debtor_participant: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateDestinationEmailRequestedPayload {
    pub transfer_id: String,
    pub email: String,
    pub new_debtor_participant: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentSettledPayload {
    pub payment_id: String,
    pub amount: f64,
    pub currency: String,
    pub debtor_participant: String,
    pub creditor_participant: String,
    pub creditor_account_id: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRejectedPayload {
    pub payment_id: String,
    pub reason: String,
    pub debtor_participant: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentExpiredPayload {
    pub payment_id: String,
    pub debtor_participant: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateIncomingPaymentRequestedPayload {
    pub payment_id: String,
    pub amount: f64,
    pub currency: String,
    pub debtor_participant: String,
    pub creditor_participant: String,
    pub account_id: String,
    pub occurred_at: String,
}
