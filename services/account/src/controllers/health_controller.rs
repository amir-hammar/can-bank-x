use axum::http::StatusCode;

pub async fn health() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok")
}

pub async fn metrics() -> (StatusCode, &'static str) {
    (
        StatusCode::OK,
        "# HELP canbankx_service_info Service info\n# TYPE canbankx_service_info gauge\ncanbankx_service_info{service=\"account-service\"} 1\n",
    )
}
