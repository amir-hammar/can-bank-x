use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::{
    controllers::request_context::{get_auth_identity, get_trace_id},
    models::{
        domain::AppState,
        dto::error_dto::ErrorResponse,
    },
    services::{customer_service, ServiceError},
};

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
