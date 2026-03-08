use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    http::StatusCode,
    Json,
};

use crate::{
    app_state::AppState,
    models::dto::{
        CreateTransferRequest, CreateTransferResponse, ListTransfersQuery, TransferDetailResponse,
        TransferSummaryResponse,
    },
    utils::errors::AppError,
};

pub async fn create_transfer(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateTransferRequest>,
) -> Result<(StatusCode, Json<CreateTransferResponse>), AppError> {
    let trace_id = headers
        .get("X-Trace-Id")
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);

    let response = state
        .transfer_service
        .create_transfer(payload, trace_id)
        .await?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_transfer_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(transfer_id): Path<String>,
) -> Result<(StatusCode, Json<TransferDetailResponse>), AppError> {
    let trace_id = headers
        .get("X-Trace-Id")
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);

    let response = state
        .transfer_service
        .get_transfer_by_id(&transfer_id, trace_id)
        .await?;
    Ok((StatusCode::OK, Json(response)))
}

pub async fn list_transfers(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListTransfersQuery>,
) -> Result<(StatusCode, Json<Vec<TransferSummaryResponse>>), AppError> {
    let trace_id = headers
        .get("X-Trace-Id")
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);

    let response = state
        .transfer_service
        .list_transfers(query, trace_id)
        .await?;
    Ok((StatusCode::OK, Json(response)))
}
