use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct AuthMeResponse {
    pub sub: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
}
