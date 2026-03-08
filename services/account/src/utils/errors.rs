use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::models::dto::ErrorResponse;

#[derive(Debug)]
pub struct AppError {
    pub status_code: StatusCode,
    pub code: &'static str,
    pub message: &'static str,
    pub details: Vec<String>,
    pub trace_id: String,
}

impl AppError {
    pub fn bad_request(code: &'static str, message: &'static str) -> Self {
        Self {
            status_code: StatusCode::BAD_REQUEST,
            code,
            message,
            details: vec![],
            trace_id: "unknown".to_string(),
        }
    }

    pub fn not_found(code: &'static str, message: &'static str) -> Self {
        Self {
            status_code: StatusCode::NOT_FOUND,
            code,
            message,
            details: vec![],
            trace_id: "unknown".to_string(),
        }
    }

    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = trace_id;
        self
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status_code,
            Json(ErrorResponse {
                code: self.code.to_string(),
                message: self.message.to_string(),
                details: self.details,
                trace_id: self.trace_id,
            }),
        )
            .into_response()
    }
}
