use sqlx::{Pool, Postgres};

use crate::{
    models::dto::customer_dto::{CustomerMeResponse, RegisterRequest, RegisterResponse},
    repositories::{audit_repository, customer_repository, kyc_repository},
    services::ServiceError,
    validation::customer_validation,
};

pub async fn register_customer(
    pool: &Pool<Postgres>,
    keycloak_sub: &str,
    payload: RegisterRequest,
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
        "CUSTOMER",
        keycloak_sub,
        "CUSTOMER_REGISTERED",
        "CUSTOMER",
        &customer_with_profile.customer.id,
        None,
        trace_id,
    )
    .await
    .map_err(|_| ServiceError::internal("Failed to record audit event"))?;

    audit_repository::record_event(
        &mut tx,
        "CUSTOMER",
        keycloak_sub,
        "KYC_SUBMITTED",
        "KYC_CASE",
        &kyc_case.id,
        None,
        trace_id,
    )
    .await
    .map_err(|_| ServiceError::internal("Failed to record audit event"))?;

    tx.commit()
        .await
        .map_err(|_| ServiceError::internal("Could not commit transaction"))?;

    Ok(RegisterResponse {
        status: "registered".to_string(),
        customer_id: customer_with_profile.customer.id,
        email: customer_with_profile.customer.email,
        full_name: customer_with_profile.profile.full_name,
        postal_code: customer_with_profile.profile.postal_code,
        nas_masked: mask_nas(&customer_with_profile.profile.nas),
    })
}

pub async fn customer_me(
    pool: &Pool<Postgres>,
    keycloak_sub: &str,
) -> Result<CustomerMeResponse, ServiceError> {
    let customer = customer_repository::get_customer_with_profile_by_sub(pool, keycloak_sub)
        .await
        .map_err(|_| ServiceError::internal("Could not fetch customer"))?;

    let Some(customer) = customer else {
        return Err(ServiceError::not_found("Customer profile not found"));
    };

    Ok(CustomerMeResponse {
        customer_id: customer.customer.id,
        email: customer.customer.email,
        status: customer.customer.status,
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
                return ServiceError::conflict(
                    "Customer already exists",
                    vec!["email or keycloak subject already registered".to_string()],
                );
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
