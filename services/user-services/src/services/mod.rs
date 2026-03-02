pub mod auth_service;
pub mod customer_service;
pub mod kyc_service;

#[derive(Debug)]
pub struct ServiceError {
	pub code: String,
	pub message: String,
	pub details: Vec<String>,
	pub status_code: u16,
}

impl ServiceError {
	pub fn bad_request(code: &str, message: &str, details: Vec<String>) -> Self {
		Self {
			code: code.to_string(),
			message: message.to_string(),
			details,
			status_code: 400,
		}
	}

	pub fn unauthorized(message: &str) -> Self {
		Self {
			code: "UNAUTHORIZED".to_string(),
			message: message.to_string(),
			details: vec![],
			status_code: 401,
		}
	}

	pub fn not_found(message: &str) -> Self {
		Self {
			code: "NOT_FOUND".to_string(),
			message: message.to_string(),
			details: vec![],
			status_code: 404,
		}
	}

	pub fn conflict(message: &str, details: Vec<String>) -> Self {
		Self {
			code: "CONFLICT".to_string(),
			message: message.to_string(),
			details,
			status_code: 409,
		}
	}

	pub fn internal(message: &str) -> Self {
		Self {
			code: "INTERNAL_ERROR".to_string(),
			message: message.to_string(),
			details: vec![],
			status_code: 500,
		}
	}
}
