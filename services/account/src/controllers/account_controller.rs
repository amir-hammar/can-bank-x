use axum::{
    extract::{Query, State},
    http::HeaderMap,
    http::StatusCode,
    Json,
};

use crate::{
    app_state::AppState,
    models::dto::{
        AccountBalanceQuery, AccountBalanceResponse, AccountSummaryResponse, ApplyTransferRequest,
        ApplyTransferResponse, CreateAccountRequest, CreateAccountResponse, DefaultAccountQuery,
        DefaultAccountResponse, ListAccountsQuery,
    },
    utils::errors::AppError,
};

fn extract_trace_id(headers: &HeaderMap) -> String {
    headers
        .get("x-trace-id")
        .or_else(|| headers.get("x-request-id"))
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string)
        .unwrap_or_else(|| "unknown".to_string())
}

pub async fn create_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateAccountRequest>,
) -> Result<(StatusCode, Json<CreateAccountResponse>), AppError> {
    let trace_id = extract_trace_id(&headers);

    let response = state
        .account_service
        .create_account(payload, Some(trace_id.clone()))
        .await
        .map_err(|error| error.with_trace_id(trace_id.clone()))?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn list_accounts(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListAccountsQuery>,
) -> Result<(StatusCode, Json<Vec<AccountSummaryResponse>>), AppError> {
    let trace_id = extract_trace_id(&headers);

    let response = state
        .account_service
        .list_accounts(query, Some(trace_id.clone()))
        .await
        .map_err(|error| error.with_trace_id(trace_id.clone()))?;
    Ok((StatusCode::OK, Json(response)))
}

pub async fn get_balance(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AccountBalanceQuery>,
) -> Result<(StatusCode, Json<AccountBalanceResponse>), AppError> {
    let trace_id = extract_trace_id(&headers);

    let response = state
        .account_service
        .get_balance(query, Some(trace_id.clone()))
        .await
        .map_err(|error| error.with_trace_id(trace_id.clone()))?;
    Ok((StatusCode::OK, Json(response)))
}

pub async fn get_default_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DefaultAccountQuery>,
) -> Result<(StatusCode, Json<DefaultAccountResponse>), AppError> {
    let trace_id = extract_trace_id(&headers);

    let response = state
        .account_service
        .get_default_account(query, Some(trace_id.clone()))
        .await
        .map_err(|error| error.with_trace_id(trace_id.clone()))?;
    Ok((StatusCode::OK, Json(response)))
}

pub async fn apply_transfer(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ApplyTransferRequest>,
) -> Result<(StatusCode, Json<ApplyTransferResponse>), AppError> {
    let trace_id = extract_trace_id(&headers);

    let response = state
        .account_service
        .apply_transfer(payload, Some(trace_id.clone()))
        .await
        .map_err(|error| error.with_trace_id(trace_id.clone()))?;
    Ok((StatusCode::OK, Json(response)))
}
