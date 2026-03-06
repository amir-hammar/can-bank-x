use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::models::dto::ErrorResponse;

#[derive(Debug)]
pub struct AppError {
    pub status_code: StatusCode,
    pub code: &'static str,
    pub message: &'static str,
}

impl AppError {
    pub fn bad_request(code: &'static str, message: &'static str) -> Self {
        Self {
            status_code: StatusCode::BAD_REQUEST,
            code,
            message,
        }
    }

    pub fn not_found(code: &'static str, message: &'static str) -> Self {
        Self {
            status_code: StatusCode::NOT_FOUND,
            code,
            message,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status_code,
            Json(ErrorResponse {
                code: self.code,
                message: self.message,
            }),
        )
            .into_response()
    }
}
