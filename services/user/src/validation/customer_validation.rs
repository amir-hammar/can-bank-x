use crate::models::dto::customer_dto::RegisterRequest;
use regex::Regex;

pub fn validate_register_request(payload: &RegisterRequest) -> Result<Vec<String>, Vec<String>> {
    let mut details: Vec<String> = Vec::new();
    let email = payload.email.trim();
    let full_name = normalize_spaces(&payload.full_name);
    let street = normalize_spaces(&payload.street);
    let city = normalize_spaces(&payload.city);
    let province = normalize_spaces(&payload.province);
    let country = normalize_spaces(&payload.country);
    let postal_compact = compact_postal_code(&payload.postal_code);
    let nas_digits = compact_nas(&payload.nas);

    let email_regex = Regex::new(r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)+$").expect("email regex");
    if email.is_empty() || email.len() > 254 || !email_regex.is_match(email) {
        details.push("email is invalid".to_string());
    }

    let full_name_regex = Regex::new(r"^[\p{L}\s'-]+$").expect("full name regex");
    let full_name_is_digits_only = full_name.chars().all(|ch| ch.is_ascii_digit());
    if full_name.len() < 2
        || full_name.len() > 100
        || !full_name_regex.is_match(&full_name)
        || full_name_is_digits_only
    {
        details.push("full_name is invalid".to_string());
    }

    if street.is_empty() {
        details.push("street is required".to_string());
    }

    if city.is_empty() {
        details.push("city is required".to_string());
    }

    if province.is_empty() {
        details.push("province is required".to_string());
    }

    if country.is_empty() {
        details.push("country is required".to_string());
    }

    let postal_regex = Regex::new(r"^[A-Za-z]\d[A-Za-z]\d[A-Za-z]\d$").expect("postal regex");
    if !postal_regex.is_match(&postal_compact) {
        details.push("postal_code must follow Canadian format A1A 1A1".to_string());
    }

    let nas_regex = Regex::new(r"^\d{9}$").expect("nas regex");
    if !nas_regex.is_match(&nas_digits) {
        details.push("nas must contain exactly 9 digits".to_string());
    }

    if details.is_empty() {
        Ok(vec![])
    } else {
        Err(details)
    }
}

pub fn normalize_register_request(payload: &RegisterRequest) -> RegisterRequest {
    let postal_compact = compact_postal_code(&payload.postal_code).to_uppercase();
    let postal_code = if postal_compact.len() == 6 {
        format!("{} {}", &postal_compact[..3], &postal_compact[3..])
    } else {
        postal_compact
    };

    RegisterRequest {
        email: payload.email.trim().to_lowercase(),
        full_name: normalize_spaces(&payload.full_name),
        street: normalize_spaces(&payload.street),
        city: normalize_spaces(&payload.city),
        province: normalize_spaces(&payload.province).to_uppercase(),
        postal_code,
        country: normalize_spaces(&payload.country),
        nas: compact_nas(&payload.nas),
    }
}

fn normalize_spaces(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn compact_postal_code(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_whitespace() && *ch != '-')
        .collect::<String>()
}

fn compact_nas(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .collect::<String>()
}
