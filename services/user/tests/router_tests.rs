use axum::body::Body;
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use tower::util::ServiceExt;
use user_service::{models::domain::AppState, router::create_router};

#[tokio::test]
async fn health_endpoint_returns_ok() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://user:pass@localhost:5432/db")
        .expect("lazy pool");

    let app = create_router(AppState {
        pool,
        kyc_mock_data_path: "mock-data/kyc/identity-mock.json".to_string(),
    });

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/health")
                .method("GET")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn customers_me_without_sub_returns_standard_unauthorized_error() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://user:pass@localhost:5432/db")
        .expect("lazy pool");

    let app = create_router(AppState {
        pool,
        kyc_mock_data_path: "mock-data/kyc/identity-mock.json".to_string(),
    });

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/v1/customers/me")
                .method("GET")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);

    let body = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    let json: Value = serde_json::from_slice(&body).expect("json body");

    assert_eq!(
        json.get("code").and_then(Value::as_str),
        Some("UNAUTHORIZED")
    );
    assert!(json.get("traceId").and_then(Value::as_str).is_some());
    assert!(json.get("details").and_then(Value::as_array).is_some());
}
