use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{app_state::AppState, utils::errors::AppError};

// ─── Request / Response DTOs ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RegisterAliasRequest {
    pub alias: String,
    pub account_id: String,
    pub customer_id: Option<String>,
    pub holder_name: String,
}

#[derive(Debug, Serialize)]
pub struct AliasResponse {
    pub alias: String,
    pub account_id: String,
    pub holder_name: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ListAliasesQuery {
    pub account_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LookupAliasQuery {
    pub alias: String,
}

#[derive(Debug, Deserialize)]
pub struct InitiatePaymentRequest {
    pub customer_id: Option<String>,
    pub source_account_id: String,
    pub alias: Option<String>,
    pub beneficiary_alias: Option<String>,
    pub amount: f64,
    pub currency: String,
    pub idempotency_key: String,
}

#[derive(Debug, Deserialize)]
pub struct ListPaymentsQuery {
    pub account_id: String,
}

#[derive(Debug, Deserialize)]
pub struct RequestAliasTransferRequest {
    pub alias: String,
    pub receiving_account_id: String,
}

#[derive(Debug, Deserialize)]
pub struct DenyTransferRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct ListPendingQuery {
    pub account_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SettlementQuery {
    pub from: String,
    pub to: String,
}

// ─── Helper ────────────────────────────────────────────────────────────────

fn get_cb_service(
    state: &AppState,
) -> Result<&crate::services::central_bank_service::CentralBankService, AppError> {
    state
        .central_bank_service
        .as_ref()
        .map(|arc| arc.as_ref())
        .ok_or_else(|| {
            AppError::bad_request(
                "CENTRAL_BANK_DISABLED",
                "central bank integration is not enabled",
            )
        })
}

// ─── Alias endpoints ───────────────────────────────────────────────────────

pub async fn register_alias(
    State(state): State<AppState>,
    Json(payload): Json<RegisterAliasRequest>,
) -> Result<(StatusCode, Json<AliasResponse>), AppError> {
    let svc = get_cb_service(&state)?;

    let cid = payload.customer_id.as_deref().unwrap_or("unknown");
    let link = svc
        .register_alias(
            &payload.alias,
            &payload.account_id,
            cid,
            &payload.holder_name,
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(AliasResponse {
            alias: link.alias,
            account_id: link.account_id.to_string(),
            holder_name: link.holder_name,
            status: link.status,
            created_at: link.created_at,
        }),
    ))
}

pub async fn delete_alias(
    State(state): State<AppState>,
    Path(alias): Path<String>,
) -> Result<StatusCode, AppError> {
    let svc = get_cb_service(&state)?;
    svc.delete_alias(&alias).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_aliases(
    State(state): State<AppState>,
    Query(query): Query<ListAliasesQuery>,
) -> Result<(StatusCode, Json<Vec<AliasResponse>>), AppError> {
    let svc = get_cb_service(&state)?;

    let links = svc.list_aliases(query.account_id.as_deref().unwrap_or("")).await?;

    let response: Vec<AliasResponse> = links
        .into_iter()
        .map(|l| AliasResponse {
            alias: l.alias,
            account_id: l.account_id.to_string(),
            holder_name: l.holder_name,
            status: l.status,
            created_at: l.created_at,
        })
        .collect();

    Ok((StatusCode::OK, Json(response)))
}

pub async fn lookup_alias(
    State(state): State<AppState>,
    Query(query): Query<LookupAliasQuery>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let svc = get_cb_service(&state)?;

    match svc.lookup_alias(&query.alias).await? {
        Some(result) => Ok((StatusCode::OK, Json(serde_json::to_value(result).unwrap()))),
        None => Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "alias": query.alias,
                "found": false,
                "message": "lookup timed out or no result"
            })),
        )),
    }
}

// ─── Payment endpoints ─────────────────────────────────────────────────────

pub async fn initiate_payment(
    State(state): State<AppState>,
    Json(payload): Json<InitiatePaymentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let svc = get_cb_service(&state)?;

    let alias = payload.alias.as_deref()
        .or(payload.beneficiary_alias.as_deref())
        .unwrap_or("");
    let cid = payload.customer_id.as_deref().unwrap_or("unknown");
    let result = svc
        .initiate_payment(
            cid,
            &payload.source_account_id,
            alias,
            payload.amount,
            &payload.currency,
            &payload.idempotency_key,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(result)))
}

pub async fn list_payments(
    State(state): State<AppState>,
    Query(query): Query<ListPaymentsQuery>,
) -> Result<(StatusCode, Json<Vec<AliasResponse>>), AppError> {
    let svc = get_cb_service(&state)?;

    // Payment links are stored as aliases — list by account
    let links = svc.list_aliases(&query.account_id).await?;

    let response: Vec<AliasResponse> = links
        .into_iter()
        .map(|l| AliasResponse {
            alias: l.alias,
            account_id: l.account_id.to_string(),
            holder_name: l.holder_name,
            status: l.status,
            created_at: l.created_at,
        })
        .collect();

    Ok((StatusCode::OK, Json(response)))
}

// ─── Alias transfer endpoints ──────────────────────────────────────────────

pub async fn request_alias_transfer(
    State(state): State<AppState>,
    Json(payload): Json<RequestAliasTransferRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let svc = get_cb_service(&state)?;

    let result = svc
        .request_alias_transfer(&payload.alias, &payload.receiving_account_id)
        .await?;

    Ok((StatusCode::CREATED, Json(result)))
}

pub async fn approve_transfer(
    State(state): State<AppState>,
    Path(transfer_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let svc = get_cb_service(&state)?;
    svc.approve_transfer(&transfer_id).await?;
    Ok(StatusCode::OK)
}

pub async fn deny_transfer(
    State(state): State<AppState>,
    Path(transfer_id): Path<String>,
    Json(payload): Json<DenyTransferRequest>,
) -> Result<StatusCode, AppError> {
    let svc = get_cb_service(&state)?;
    svc.deny_transfer(&transfer_id, &payload.reason).await?;
    Ok(StatusCode::OK)
}

pub async fn list_pending_transfers(
    State(state): State<AppState>,
    Query(query): Query<ListPendingQuery>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let svc = get_cb_service(&state)?;

    let pending = svc.list_pending_transfers(query.account_id.as_deref().unwrap_or("")).await?;

    let response: Vec<serde_json::Value> = pending
        .into_iter()
        .map(|p| {
            serde_json::json!({
                "transfer_id": p.transfer_id,
                "alias_value": p.alias_value,
                "account_id": p.account_id.to_string(),
                "debtor_participant": p.debtor_participant,
                "new_debtor_participant": p.new_debtor_participant,
                "destination_email": p.destination_email,
                "status": p.status,
                "reason": p.reason,
                "created_at": p.created_at,
            })
        })
        .collect();

    Ok((StatusCode::OK, Json(serde_json::json!(response))))
}

// ─── Settlement endpoint ───────────────────────────────────────────────────

pub async fn get_settlement(
    State(state): State<AppState>,
    Query(query): Query<SettlementQuery>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let svc = get_cb_service(&state)?;
    let result = svc.get_settlement(&query.from, &query.to).await?;
    Ok((StatusCode::OK, Json(result)))
}

// ─── Status endpoint ───────────────────────────────────────────────────────

pub async fn status(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let info = if let Some(svc) = &state.central_bank_service {
        serde_json::json!({
            "enabled": true,
            "participant_id": svc.participant_id,
            "payment_service_url": svc.payment_service_url,
            "status": "connected",
        })
    } else {
        serde_json::json!({
            "enabled": false,
            "status": "disabled",
        })
    };

    Ok((StatusCode::OK, Json(info)))
}
