use axum::http::{HeaderMap, HeaderValue};
use user_service::controllers::request_context::{get_keycloak_sub, get_trace_id};

#[test]
fn uses_trace_header_when_present() {
    let mut headers = HeaderMap::new();
    headers.insert("x-trace-id", HeaderValue::from_static("trace-123"));

    let trace_id = get_trace_id(&headers);
    assert_eq!(trace_id, "trace-123");
}

#[test]
fn uses_request_id_when_trace_missing() {
    let mut headers = HeaderMap::new();
    headers.insert("x-request-id", HeaderValue::from_static("req-123"));

    let trace_id = get_trace_id(&headers);
    assert_eq!(trace_id, "req-123");
}

#[test]
fn extracts_sub_from_user_header() {
    let mut headers = HeaderMap::new();
    headers.insert("x-user-sub", HeaderValue::from_static("sub-abc"));

    let sub = get_keycloak_sub(&headers).expect("sub should be extracted");
    assert_eq!(sub, "sub-abc");
}

#[test]
fn returns_unauthorized_when_sub_missing() {
    let headers = HeaderMap::new();
    let err = get_keycloak_sub(&headers).expect_err("missing sub should fail");
    assert_eq!(err.status_code, 401);
    assert_eq!(err.code, "UNAUTHORIZED");
}
