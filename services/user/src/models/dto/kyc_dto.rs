use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct KycConfirmRequest {
    pub approved: bool,
}

#[derive(Debug, Serialize)]
pub struct KycStatusResponse {
    pub customer_id: String,
    pub kyc_case_id: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct KycSubmitResponse {
    pub status: String,
    pub kyc_case_id: String,
}
