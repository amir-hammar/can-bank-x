use account_service::{app_state::AppState, create_app};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn create_account_success_returns_201() {
    let app = create_app(AppState::new());

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/accounts/create")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"customer_id":"cust_1","account_type":"CHEQUING","initial_balance":250.5}"#,
        ))
        .expect("failed to build request");

    let response = app.oneshot(request).await.expect("request failed");

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response
        .into_body()
        .collect()
        .await
        .expect("failed to read response body")
        .to_bytes();

    let body_text = String::from_utf8(body.to_vec()).expect("invalid utf8 body");
    assert!(body_text.contains("acc_cust_1_"));
    assert!(body_text.contains("250.5"));
}

#[tokio::test]
async fn list_and_balance_flow_returns_200() {
    let app = create_app(AppState::new());

    let create_request = Request::builder()
        .method("POST")
        .uri("/api/v1/accounts/create")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"customer_id":"cust_2","account_type":"SAVINGS","initial_balance":1000}"#,
        ))
        .expect("failed to build create request");

    let create_response = app
        .clone()
        .oneshot(create_request)
        .await
        .expect("request failed");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let list_request = Request::builder()
        .method("GET")
        .uri("/api/v1/accounts?customer_id=cust_2")
        .body(Body::empty())
        .expect("failed to build list request");

    let list_response = app
        .clone()
        .oneshot(list_request)
        .await
        .expect("request failed");
    assert_eq!(list_response.status(), StatusCode::OK);

    let list_body = list_response
        .into_body()
        .collect()
        .await
        .expect("failed to read list body")
        .to_bytes();

    let list_text = String::from_utf8(list_body.to_vec()).expect("invalid utf8 body");
    assert!(list_text.contains("cust_2"));
    assert!(list_text.contains("SAVINGS"));

    let account_id = list_text
        .split("\"account_id\":\"")
        .nth(1)
        .and_then(|tail| tail.split('\"').next())
        .expect("account_id not found")
        .to_string();

    let balance_request = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/accounts/balance?account_id={account_id}"))
        .body(Body::empty())
        .expect("failed to build balance request");

    let balance_response = app.oneshot(balance_request).await.expect("request failed");
    assert_eq!(balance_response.status(), StatusCode::OK);

    let balance_body = balance_response
        .into_body()
        .collect()
        .await
        .expect("failed to read balance body")
        .to_bytes();

    let balance_text = String::from_utf8(balance_body.to_vec()).expect("invalid utf8 body");
    assert!(balance_text.contains("1000.0"));
}
