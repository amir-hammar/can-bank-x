use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub full_name: String,
    pub street: String,
    pub city: String,
    pub province: String,
    pub postal_code: String,
    pub country: String,
    pub nas: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub status: String,
    pub customer_id: String,
    pub email: String,
    pub full_name: String,
    pub postal_code: String,
    pub nas_masked: String,
}

#[derive(Debug, Serialize)]
pub struct CustomerMeResponse {
    pub customer_id: String,
    pub email: String,
    pub status: String,
    pub full_name: String,
    pub street: String,
    pub city: String,
    pub province: String,
    pub postal_code: String,
    pub country: String,
    pub nas_masked: String,
}
