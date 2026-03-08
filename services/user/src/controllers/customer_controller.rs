use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::{
    controllers::request_context::{get_auth_identity, get_trace_id},
    models::{domain::AppState, dto::error_dto::ErrorResponse},
    services::{customer_service, ServiceError},
};

#[derive(Debug, Deserialize)]
pub struct GetByUsernameQuery {
    pub username: String,
}

pub async fn register(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let trace_id = get_trace_id(&headers);

    let identity = match get_auth_identity(&headers) {
        Ok(identity) => identity,
        Err(error) => return map_error(error, &trace_id),
    };

    match customer_service::register_customer_from_identity(&state.pool, &identity, &trace_id)
        .await
    {
        Ok(_) => {
            let response = serde_json::json!({
                "status": "registered",
                "message": "Customer registered and KYC countdown started"
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(error) => map_error(error, &trace_id),
    }
}

pub async fn me(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let trace_id = get_trace_id(&headers);

    let identity = match get_auth_identity(&headers) {
        Ok(identity) => identity,
        Err(error) => return map_error(error, &trace_id),
    };

    match customer_service::customer_me(
        &state.pool,
        &identity,
        &state.kyc_mock_data_path,
        &trace_id,
    )
    .await
    {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(error) => map_error(error, &trace_id),
    }
}

pub async fn get_by_username(
    State(state): State<AppState>,
    Query(query): Query<GetByUsernameQuery>,
) -> impl IntoResponse {
    match customer_service::get_customer_by_username(&state.pool, &query.username).await {
        Ok(customer) => {
            let response = serde_json::json!({
                "id": customer.id,
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(error) => {
            let status = StatusCode::from_u16(error.status_code)
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            (
                status,
                Json(ErrorResponse::new(
                    &error.code,
                    &error.message,
                    error.details,
                    "",
                )),
            )
                .into_response()
        }
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
