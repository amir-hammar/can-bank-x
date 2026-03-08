use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::models::dto::ErrorResponse;

#[derive(Debug)]
pub struct AppError {
    status: StatusCode,
    code: &'static str,
    message: String,
    details: Vec<String>,
    trace_id: String,
}

impl AppError {
    pub fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
            message: message.into(),
            details: vec![],
            trace_id: "unknown".to_string(),
        }
    }

    pub fn not_found(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code,
            message: message.into(),
            details: vec![],
            trace_id: "unknown".to_string(),
        }
    }

    pub fn failed_dependency(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FAILED_DEPENDENCY,
            code,
            message: message.into(),
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
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse {
                code: self.code.to_string(),
                message: self.message,
                details: self.details,
                trace_id: self.trace_id,
            }),
        )
            .into_response()
    }
}
