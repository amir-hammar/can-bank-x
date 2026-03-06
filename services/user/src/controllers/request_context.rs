use axum::http::HeaderMap;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde_json::Value;
use uuid::Uuid;

use crate::services::ServiceError;

#[derive(Debug, Clone)]
pub struct AuthIdentity {
    pub sub: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub province: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub nas: Option<String>,
}

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

pub fn get_auth_identity(headers: &HeaderMap) -> Result<AuthIdentity, ServiceError> {
    let claims = extract_claims_from_authorization(headers);

    let sub = headers
        .get("x-user-sub")
        .or_else(|| headers.get("x-jwt-sub"))
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
        .or_else(|| get_claim_str(&claims, "sub"))
        .ok_or_else(|| ServiceError::unauthorized("Missing authenticated subject header"))?;

    let username = headers
        .get("x-user-preferred-username")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
        .or_else(|| get_claim_str(&claims, "preferred_username"));

    let email = headers
        .get("x-user-email")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
        .or_else(|| get_claim_str(&claims, "email"));

    let full_name = headers
        .get("x-user-name")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
        .or_else(|| {
            headers
                .get("x-user-full-name")
                .and_then(|value| value.to_str().ok())
                .map(|value| value.to_string())
        })
        .or_else(|| get_claim_str(&claims, "fullName"))
        .or_else(|| get_claim_str(&claims, "name"))
        .or_else(|| {
            let given = get_claim_str(&claims, "given_name")?;
            let family = get_claim_str(&claims, "family_name").unwrap_or_default();
            let combined = format!("{} {}", given, family).trim().to_string();
            if combined.is_empty() {
                None
            } else {
                Some(combined)
            }
        });

    let street = headers
        .get("x-user-street")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
        .or_else(|| get_claim_str(&claims, "street"))
        .or_else(|| get_claim_nested_str(&claims, "address", "street_address"));

    let city = headers
        .get("x-user-city")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
        .or_else(|| get_claim_str(&claims, "city"))
        .or_else(|| get_claim_nested_str(&claims, "address", "locality"));

    let province = headers
        .get("x-user-province")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
        .or_else(|| get_claim_str(&claims, "province"))
        .or_else(|| get_claim_nested_str(&claims, "address", "region"));

    let postal_code = headers
        .get("x-user-postal-code")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
        .or_else(|| {
            headers
                .get("x-user-postalcode")
                .and_then(|value| value.to_str().ok())
                .map(|value| value.to_string())
        })
        .or_else(|| get_claim_str(&claims, "postalCode"))
        .or_else(|| get_claim_nested_str(&claims, "address", "postalCode"));

    let country = headers
        .get("x-user-country")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
        .or_else(|| get_claim_str(&claims, "country"))
        .or_else(|| get_claim_nested_str(&claims, "address", "country"));

    let nas = headers
        .get("x-user-nas")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
        .or_else(|| get_claim_str(&claims, "nas"))
        .or_else(|| get_claim_str(&claims, "sin"));

    Ok(AuthIdentity {
        sub,
        username,
        email,
        full_name,
        street,
        city,
        province,
        postal_code,
        country,
        nas,
    })
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

fn extract_claims_from_authorization(headers: &HeaderMap) -> Option<Value> {
    let header = headers.get("authorization")?.to_str().ok()?;
    let token = header.strip_prefix("Bearer ")?;
    let mut parts = token.split('.');
    let _header = parts.next()?;
    let payload = parts.next()?;

    let decoded = URL_SAFE_NO_PAD.decode(payload).ok()?;
    serde_json::from_slice(&decoded).ok()
}

fn get_claim_str(claims: &Option<Value>, key: &str) -> Option<String> {
    claims.as_ref()?.get(key)?.as_str().map(str::to_string)
}

fn get_claim_nested_str(claims: &Option<Value>, object_key: &str, key: &str) -> Option<String> {
    claims
        .as_ref()?
        .get(object_key)?
        .get(key)?
        .as_str()
        .map(str::to_string)
}
