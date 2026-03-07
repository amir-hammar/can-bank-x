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
        ApplyTransferResponse, CreateAccountRequest, CreateAccountResponse, ListAccountsQuery,
    },
    utils::errors::AppError,
};

pub async fn create_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateAccountRequest>,
) -> Result<(StatusCode, Json<CreateAccountResponse>), AppError> {
    let trace_id = headers
        .get("X-Trace-Id")
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);

    let response = state
        .account_service
        .create_account(payload, trace_id)
        .await?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn list_accounts(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListAccountsQuery>,
) -> Result<(StatusCode, Json<Vec<AccountSummaryResponse>>), AppError> {
    let trace_id = headers
        .get("X-Trace-Id")
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);

    let response = state.account_service.list_accounts(query, trace_id).await?;
    Ok((StatusCode::OK, Json(response)))
}

pub async fn get_balance(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AccountBalanceQuery>,
) -> Result<(StatusCode, Json<AccountBalanceResponse>), AppError> {
    let trace_id = headers
        .get("X-Trace-Id")
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);

    let response = state.account_service.get_balance(query, trace_id).await?;
    Ok((StatusCode::OK, Json(response)))
}

pub async fn apply_transfer(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ApplyTransferRequest>,
) -> Result<(StatusCode, Json<ApplyTransferResponse>), AppError> {
    let trace_id = headers
        .get("X-Trace-Id")
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);

    let response = state
        .account_service
        .apply_transfer(payload, trace_id)
        .await?;
    Ok((StatusCode::OK, Json(response)))
}
