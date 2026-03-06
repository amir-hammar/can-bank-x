use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};

use crate::{
    controllers::request_context::{get_keycloak_sub, get_trace_id},
    models::{
        domain::AppState,
        dto::{error_dto::ErrorResponse, kyc_dto::KycConfirmRequest},
    },
    services::{kyc_service, ServiceError},
};

pub async fn submit_kyc(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let trace_id = get_trace_id(&headers);

    let keycloak_sub = match get_keycloak_sub(&headers) {
        Ok(sub) => sub,
        Err(error) => return map_error(error, &trace_id),
    };

    match kyc_service::submit_kyc(&state.pool, &keycloak_sub, &state.kyc_mock_data_path, &trace_id)
        .await
    {
        Ok(response) => (StatusCode::ACCEPTED, Json(response)).into_response(),
        Err(error) => map_error(error, &trace_id),
    }
}

pub async fn confirm_kyc(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<KycConfirmRequest>,
) -> impl IntoResponse {
    let trace_id = get_trace_id(&headers);

    let keycloak_sub = match get_keycloak_sub(&headers) {
        Ok(sub) => sub,
        Err(error) => return map_error(error, &trace_id),
    };

    match kyc_service::confirm_kyc(&state.pool, &keycloak_sub, payload.approved, &trace_id).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(error) => map_error(error, &trace_id),
    }
}

pub async fn kyc_status(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let trace_id = get_trace_id(&headers);

    let keycloak_sub = match get_keycloak_sub(&headers) {
        Ok(sub) => sub,
        Err(error) => return map_error(error, &trace_id),
    };

    match kyc_service::kyc_status(
        &state.pool,
        &keycloak_sub,
        &state.kyc_mock_data_path,
        &trace_id,
    )
    .await
    {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(error) => map_error(error, &trace_id),
    }
}

fn map_error(error: ServiceError, trace_id: &str) -> axum::response::Response {
    let status =
        StatusCode::from_u16(error.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (
        status,
        Json(ErrorResponse::new(
            &error.code,
            &error.message,
            error.details,
            trace_id,
        )),
    )
        .into_response()
}
