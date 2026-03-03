use axum::http::HeaderMap;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde_json::Value;
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
        .or_else(|| extract_sub_from_authorization(headers))
        .ok_or_else(|| ServiceError::unauthorized("Missing authenticated subject header"))
}

fn extract_sub_from_authorization(headers: &HeaderMap) -> Option<String> {
    let header = headers.get("authorization")?.to_str().ok()?;
    let token = header.strip_prefix("Bearer ")?;
    let mut parts = token.split('.');
    let _header = parts.next()?;
    let payload = parts.next()?;

    let decoded = URL_SAFE_NO_PAD.decode(payload).ok()?;
    let value: Value = serde_json::from_slice(&decoded).ok()?;
    value.get("sub")?.as_str().map(str::to_string)
}
