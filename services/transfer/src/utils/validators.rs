use crate::models::dto::{CreateTransferRequest, ListTransfersQuery};

use super::errors::AppError;

pub fn validate_create_transfer(payload: &CreateTransferRequest) -> Result<(), AppError> {
    if payload.customer_id.trim().is_empty() {
        return Err(AppError::bad_request(
            "INVALID_TRANSFER",
            "customer_id is required",
        ));
    }
    if payload.from_account_id.trim().is_empty() {
        return Err(AppError::bad_request(
            "INVALID_TRANSFER",
            "from_account_id is required",
        ));
    }
    if payload.to_account_id.trim().is_empty() {
        return Err(AppError::bad_request(
            "INVALID_TRANSFER",
            "to_account_id is required",
        ));
    }
    if payload.from_account_id == payload.to_account_id {
        return Err(AppError::bad_request(
            "INVALID_TRANSFER",
            "from_account_id and to_account_id must be different",
        ));
    }
    if payload.amount <= 0.0 {
        return Err(AppError::bad_request(
            "INVALID_TRANSFER",
            "amount must be greater than 0",
        ));
    }
    if payload.idempotency_key.trim().is_empty() {
        return Err(AppError::bad_request(
            "INVALID_TRANSFER",
            "idempotency_key is required",
        ));
    }

    Ok(())
}

pub fn validate_list_transfers(query: &ListTransfersQuery) -> Result<usize, AppError> {
    if query
        .customer_id
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
        && query
            .account_id
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty()
    {
        return Err(AppError::bad_request(
            "INVALID_FILTER",
            "either customer_id or account_id is required",
        ));
    }

    let limit = query.limit.unwrap_or(50);
    if limit == 0 || limit > 200 {
        return Err(AppError::bad_request(
            "INVALID_FILTER",
            "limit must be between 1 and 200",
        ));
    }

    Ok(limit)
}
