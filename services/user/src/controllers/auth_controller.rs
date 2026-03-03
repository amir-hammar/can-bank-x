use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::{
    controllers::request_context::{get_keycloak_sub, get_trace_id},
    models::{domain::AppState, dto::error_dto::ErrorResponse},
    services::{auth_service, ServiceError},
};

pub async fn me(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let trace_id = get_trace_id(&headers);

    let keycloak_sub = match get_keycloak_sub(&headers) {
        Ok(sub) => sub,
        Err(error) => return map_error(error, &trace_id),
    };

    match auth_service::auth_me(&state.pool, &keycloak_sub).await {
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
