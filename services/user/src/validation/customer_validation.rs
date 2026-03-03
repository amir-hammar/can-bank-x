use crate::models::dto::customer_dto::RegisterRequest;
use regex::Regex;

pub fn validate_register_request(payload: &RegisterRequest) -> Result<Vec<String>, Vec<String>> {
    let mut details: Vec<String> = Vec::new();

    if payload.email.trim().is_empty() || !payload.email.contains('@') {
        details.push("email is invalid".to_string());
    }

    if payload.full_name.trim().len() < 2 {
        details.push("full_name must contain at least 2 characters".to_string());
    }

    if payload.street.trim().is_empty() {
        details.push("street is required".to_string());
    }

    if payload.city.trim().is_empty() {
        details.push("city is required".to_string());
    }

    if payload.province.trim().is_empty() {
        details.push("province is required".to_string());
    }

    if payload.country.trim().is_empty() {
        details.push("country is required".to_string());
    }

    let postal_regex = Regex::new(r"^[A-Za-z]\d[A-Za-z][ -]?\d[A-Za-z]\d$").expect("postal regex");
    if !postal_regex.is_match(payload.postal_code.trim()) {
        details.push("postal_code must follow Canadian format A1A 1A1".to_string());
    }

    let nas_regex = Regex::new(r"^\d{9}$").expect("nas regex");
    if !nas_regex.is_match(payload.nas.trim()) {
        details.push("nas must contain exactly 9 digits".to_string());
    }

    if details.is_empty() {
        Ok(vec![])
    } else {
        Err(details)
    }
}

pub fn normalize_register_request(payload: &RegisterRequest) -> RegisterRequest {
    RegisterRequest {
        email: payload.email.trim().to_lowercase(),
        full_name: payload.full_name.trim().to_string(),
        street: payload.street.trim().to_string(),
        city: payload.city.trim().to_string(),
        province: payload.province.trim().to_uppercase(),
        postal_code: payload.postal_code.trim().to_uppercase().replace('-', " "),
        country: payload.country.trim().to_string(),
        nas: payload.nas.trim().to_string(),
    }
}
