use account_service::{app_state::AppState, create_app};
use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;

#[tokio::test]
async fn create_account_invalid_payload_returns_400() {
    let app = create_app(AppState::new());

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/accounts/create")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"customer_id":"","account_type":"INVALID","initial_balance":-10}"#,
        ))
        .expect("failed to build request");

    let response = app.oneshot(request).await.expect("request failed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
