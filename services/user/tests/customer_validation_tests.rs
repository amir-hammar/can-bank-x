use user_service::{
    models::dto::customer_dto::RegisterRequest,
    validation::customer_validation::{normalize_register_request, validate_register_request},
};

fn sample_request() -> RegisterRequest {
    RegisterRequest {
        email: "User@Example.com".to_string(),
        full_name: "Jane Doe".to_string(),
        street: "123   Main".to_string(),
        city: "Montreal".to_string(),
        province: "qc".to_string(),
        postal_code: "h2x-1z5".to_string(),
        country: "Canada".to_string(),
        nas: "123 456 789".to_string(),
    }
}

#[test]
fn reject_invalid_email() {
    let mut payload = sample_request();
    payload.email = "bad-email".to_string();

    assert!(validate_register_request(&payload).is_err());
}

#[test]
fn accept_valid_payload() {
    let payload = sample_request();

    assert!(validate_register_request(&payload).is_ok());
}

#[test]
fn normalize_payload() {
    let payload = sample_request();
    let normalized = normalize_register_request(&payload);

    assert_eq!(normalized.email, "user@example.com");
    assert_eq!(normalized.province, "QC");
    assert_eq!(normalized.street, "123 Main");
    assert_eq!(normalized.postal_code, "H2X 1Z5");
    assert_eq!(normalized.nas, "123456789");
}

#[test]
fn reject_invalid_full_name() {
    let mut payload = sample_request();
    payload.full_name = "123456".to_string();

    assert!(validate_register_request(&payload).is_err());
}
