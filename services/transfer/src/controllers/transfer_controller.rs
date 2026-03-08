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

fn extract_trace_id(headers: &HeaderMap) -> String {
    headers
        .get("x-trace-id")
        .or_else(|| headers.get("x-request-id"))
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string)
        .unwrap_or_else(|| "unknown".to_string())
}

pub async fn create_transfer(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateTransferRequest>,
) -> Result<(StatusCode, Json<CreateTransferResponse>), AppError> {
    let trace_id = extract_trace_id(&headers);

    let response = state
        .transfer_service
        .create_transfer(payload, Some(trace_id.clone()))
        .await
        .map_err(|error| error.with_trace_id(trace_id.clone()))?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_transfer_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(transfer_id): Path<String>,
) -> Result<(StatusCode, Json<TransferDetailResponse>), AppError> {
    let trace_id = extract_trace_id(&headers);

    let response = state
        .transfer_service
        .get_transfer_by_id(&transfer_id, Some(trace_id.clone()))
        .await
        .map_err(|error| error.with_trace_id(trace_id.clone()))?;
    Ok((StatusCode::OK, Json(response)))
}

pub async fn list_transfers(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListTransfersQuery>,
) -> Result<(StatusCode, Json<Vec<TransferSummaryResponse>>), AppError> {
    let trace_id = extract_trace_id(&headers);

    let response = state
        .transfer_service
        .list_transfers(query, Some(trace_id.clone()))
        .await
        .map_err(|error| error.with_trace_id(trace_id.clone()))?;
    Ok((StatusCode::OK, Json(response)))
}
