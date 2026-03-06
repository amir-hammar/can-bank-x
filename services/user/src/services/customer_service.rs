use sqlx::{Pool, Postgres};

use crate::{
    controllers::request_context::AuthIdentity,
    models::dto::customer_dto::{CustomerMeResponse, RegisterRequest, RegisterResponse},
    repositories::{audit_repository, customer_repository, kyc_repository},
    services::ServiceError,
    validation::customer_validation,
};

pub async fn register_customer_from_identity(
    pool: &Pool<Postgres>,
    identity: &AuthIdentity,
    trace_id: &str,
) -> Result<(), ServiceError> {
    let full_name = identity
        .full_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    let street = identity
        .street
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let city = identity
        .city
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let province = identity
        .province
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let postal_code = identity
        .postal_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let country = identity
        .country
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let nas = identity
        .nas
        .as_deref()
        .map(compact_nas)
        .filter(|value| !value.is_empty());

    let mut missing_fields: Vec<String> = Vec::new();
    if full_name.is_none() {
        missing_fields.push("fullName".to_string());
    }
    if street.is_none() {
        missing_fields.push("street".to_string());
    }
    if city.is_none() {
        missing_fields.push("city".to_string());
    }
    if province.is_none() {
        missing_fields.push("province".to_string());
    }
    if postal_code.is_none() {
        missing_fields.push("postalCode".to_string());
    }
    if country.is_none() {
        missing_fields.push("country".to_string());
    }
    if nas.is_none() {
        missing_fields.push("nas".to_string());
    }

    if !missing_fields.is_empty() {
        return Err(ServiceError::bad_request(
            "MISSING_PROFILE_CLAIMS",
            "Missing profile fields from Keycloak token/headers",
            vec![format!("missing fields: {}", missing_fields.join(", "))],
        ));
    }

    let payload = RegisterRequest {
        username: resolve_username(identity),
        email: resolve_email(identity),
        full_name: full_name.unwrap_or_default(),
        street: street.unwrap_or_default(),
        city: city.unwrap_or_default(),
        province: province.unwrap_or_default(),
        postal_code: postal_code.unwrap_or_default(),
        country: country.unwrap_or_default(),
        nas: nas.unwrap_or_default(),
    };

    match register_customer(pool, &identity.sub, payload, "", trace_id).await {
        Ok(_) => Ok(()),
        Err(error) if error.status_code == 409 => {
            let username = resolve_username(identity);
            let email = resolve_email(identity);
            customer_repository::rebind_customer_sub_by_identity(
                pool,
                &identity.sub,
                &username,
                &email,
            )
            .await
            .map_err(|_| ServiceError::internal("Failed to reconcile customer identity"))?;
            Ok(())
        }
        Err(error) => Err(error),
    }
}

pub async fn register_customer(
    pool: &Pool<Postgres>,
    keycloak_sub: &str,
    payload: RegisterRequest,
    _kyc_mock_data_path: &str,
    trace_id: &str,
) -> Result<RegisterResponse, ServiceError> {
    if let Err(details) = customer_validation::validate_register_request(&payload) {
        return Err(ServiceError::bad_request(
            "INVALID_REGISTER_PAYLOAD",
            "Register payload validation failed",
            details,
        ));
    }

    let normalized = customer_validation::normalize_register_request(&payload);

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| ServiceError::internal("Could not start transaction"))?;

    let customer_with_profile =
        customer_repository::insert_customer_with_profile(&mut tx, keycloak_sub, &normalized)
            .await
            .map_err(map_sqlx_insert_error)?;

    let kyc_case =
        kyc_repository::get_or_create_pending_kyc(&mut tx, &customer_with_profile.customer.id)
            .await
            .map_err(|_| ServiceError::internal("Failed to initialize KYC case"))?;

    audit_repository::record_event(
        &mut tx,
        audit_repository::AuditEvent {
            actor_type: "CUSTOMER",
            actor_id: keycloak_sub,
            action: "CUSTOMER_REGISTERED",
            entity_type: "CUSTOMER",
            entity_id: &customer_with_profile.customer.id,
            metadata: None,
            trace_id,
        },
    )
    .await
    .map_err(|_| ServiceError::internal("Failed to record audit event"))?;

    audit_repository::record_event(
        &mut tx,
        audit_repository::AuditEvent {
            actor_type: "CUSTOMER",
            actor_id: keycloak_sub,
            action: "KYC_SUBMITTED",
            entity_type: "KYC_CASE",
            entity_id: &kyc_case.id,
            metadata: None,
            trace_id,
        },
    )
    .await
    .map_err(|_| ServiceError::internal("Failed to record audit event"))?;

    tx.commit()
        .await
        .map_err(|_| ServiceError::internal("Could not commit transaction"))?;

    Ok(RegisterResponse {
        status: "registered".to_string(),
        customer_id: customer_with_profile.customer.id,
        username: customer_with_profile.customer.username,
        email: customer_with_profile.customer.email,
        full_name: customer_with_profile.profile.full_name,
        postal_code: customer_with_profile.profile.postal_code,
        nas_masked: mask_nas(&customer_with_profile.profile.nas),
        kyc_status: "PENDING".to_string(),
    })
}

pub async fn customer_me(
    pool: &Pool<Postgres>,
    identity: &AuthIdentity,
    kyc_mock_data_path: &str,
    trace_id: &str,
) -> Result<CustomerMeResponse, ServiceError> {
    let customer = customer_repository::get_customer_with_profile_by_sub(pool, &identity.sub)
        .await
        .map_err(|_| ServiceError::internal("Could not fetch customer"))?;

    let customer = match customer {
        Some(c) => c,
        None => {
            register_customer_from_identity(pool, identity, trace_id).await?;
            customer_repository::get_customer_with_profile_by_sub(pool, &identity.sub)
                .await
                .map_err(|_| ServiceError::internal("Could not fetch customer"))?
                .ok_or_else(|| ServiceError::internal("Failed to create customer"))?
        }
    };

    let kyc = crate::services::kyc_service::resolve_kyc_status_for_customer(
        pool,
        &customer,
        kyc_mock_data_path,
        trace_id,
    )
    .await?;

    Ok(CustomerMeResponse {
        customer_id: customer.customer.id,
        username: customer.customer.username,
        email: customer.customer.email,
        status: customer.customer.status,
        kyc_status: kyc.status,
        kyc_approved: kyc.approved,
        kyc_decision_available_in_seconds: kyc.decision_available_in_seconds,
        full_name: customer.profile.full_name,
        street: customer.profile.street,
        city: customer.profile.city,
        province: customer.profile.province,
        postal_code: customer.profile.postal_code,
        country: customer.profile.country,
        nas_masked: mask_nas(&customer.profile.nas),
    })
}

fn map_sqlx_insert_error(error: sqlx::Error) -> ServiceError {
    if let sqlx::Error::Database(database_error) = &error {
        if let Some(code) = database_error.code() {
            if code == "23505" {
                let details = match database_error.constraint() {
                    Some("customers_keycloak_sub_key") => vec![
                        "duplicate keycloak_sub: customer already registered for this authenticated user"
                            .to_string(),
                    ],
                    Some("customers_username_key") => {
                        vec!["duplicate username: username already exists".to_string()]
                    }
                    Some("customers_email_key") => {
                        vec!["duplicate email: email already exists".to_string()]
                    }
                    Some(constraint) => vec![format!(
                        "registration request conflicts with unique constraint: {}",
                        constraint
                    )],
                    None => vec!["registration request could not be completed".to_string()],
                };

                return ServiceError::conflict("Registration failed", details);
            }
        }
    }

    ServiceError::internal("Failed to register customer")
}

fn mask_nas(nas: &str) -> String {
    if nas.len() <= 4 {
        return "****".to_string();
    }

    let visible = &nas[nas.len() - 4..];
    format!("*****{}", visible)
}

fn compact_nas(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .collect::<String>()
}

fn resolve_username(identity: &AuthIdentity) -> String {
    let fallback_username = format!(
        "user_{}",
        &identity.sub.chars().take(12).collect::<String>()
    );
    identity
        .username
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(&fallback_username)
        .to_string()
}

fn resolve_email(identity: &AuthIdentity) -> String {
    let fallback_email = format!(
        "{}@canbankx.local",
        &identity.sub.chars().take(12).collect::<String>()
    );
    identity
        .email
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(&fallback_email)
        .to_string()
}
