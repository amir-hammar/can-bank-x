use axum::{extract::{Query, State}, http::StatusCode, Json};

use crate::{
    app_state::AppState,
    models::dto::{
        AccountBalanceQuery, AccountBalanceResponse, AccountSummaryResponse, CreateAccountRequest,
        CreateAccountResponse, ListAccountsQuery,
    },
    utils::errors::AppError,
};

pub async fn create_account(
    State(state): State<AppState>,
    Json(payload): Json<CreateAccountRequest>,
) -> Result<(StatusCode, Json<CreateAccountResponse>), AppError> {
    let response = state.account_service.create_account(payload).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn list_accounts(
    State(state): State<AppState>,
    Query(query): Query<ListAccountsQuery>,
) -> Result<(StatusCode, Json<Vec<AccountSummaryResponse>>), AppError> {
    let response = state.account_service.list_accounts(query).await?;
    Ok((StatusCode::OK, Json(response)))
}

pub async fn get_balance(
    State(state): State<AppState>,
    Query(query): Query<AccountBalanceQuery>,
) -> Result<(StatusCode, Json<AccountBalanceResponse>), AppError> {
    let response = state.account_service.get_balance(query).await?;
    Ok((StatusCode::OK, Json(response)))
}
