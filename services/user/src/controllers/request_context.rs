use axum::http::HeaderMap;
use uuid::Uuid;

use crate::services::ServiceError;

pub fn get_trace_id(headers: &HeaderMap) -> String {
    headers
        .get("x-trace-id")
        .or_else(|| headers.get("x-request-id"))
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

pub fn get_keycloak_sub(headers: &HeaderMap) -> Result<String, ServiceError> {
    headers
        .get("x-user-sub")
        .or_else(|| headers.get("x-jwt-sub"))
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
        .ok_or_else(|| ServiceError::unauthorized("Missing authenticated subject header"))
}
