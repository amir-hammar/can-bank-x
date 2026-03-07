use axum::{http::StatusCode, response::IntoResponse};

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

pub async fn metrics() -> impl IntoResponse {
    (
        StatusCode::OK,
        "# HELP canbankx_service_info Service info\n# TYPE canbankx_service_info gauge\ncanbankx_service_info{service=\"transfer-service\"} 1\n",
    )
}
