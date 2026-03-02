use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Vec<String>,
    #[serde(rename = "traceId")]
    pub trace_id: String,
}

impl ErrorResponse {
    pub fn new(code: &str, message: &str, details: Vec<String>, trace_id: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details,
            trace_id: trace_id.to_string(),
        }
    }
}
