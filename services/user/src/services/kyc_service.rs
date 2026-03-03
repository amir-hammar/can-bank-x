use sqlx::{Pool, Postgres};

use crate::{
    models::dto::kyc_dto::KycStatusResponse,
    repositories::{audit_repository, customer_repository, kyc_repository},
    services::ServiceError,
};

pub async fn submit_kyc(
    pool: &Pool<Postgres>,
    keycloak_sub: &str,
    trace_id: &str,
) -> Result<KycStatusResponse, ServiceError> {
    let customer = customer_repository::get_customer_with_profile_by_sub(pool, keycloak_sub)
        .await
        .map_err(|_| ServiceError::internal("Could not fetch customer"))?;

    let Some(customer) = customer else {
        return Err(ServiceError::not_found("Customer profile not found"));
    };

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| ServiceError::internal("Could not start transaction"))?;

    let case = kyc_repository::get_or_create_pending_kyc(&mut tx, &customer.customer.id)
        .await
        .map_err(|_| ServiceError::internal("Failed to submit KYC"))?;

    audit_repository::record_event(
        &mut tx,
        "CUSTOMER",
        keycloak_sub,
        "KYC_SUBMITTED",
        "KYC_CASE",
        &case.id,
        None,
        trace_id,
    )
    .await
    .map_err(|_| ServiceError::internal("Failed to record audit event"))?;

    tx.commit()
        .await
        .map_err(|_| ServiceError::internal("Could not commit transaction"))?;

    Ok(KycStatusResponse {
        customer_id: case.customer_id,
        kyc_case_id: case.id,
        status: case.status,
    })
}

pub async fn confirm_kyc(
    pool: &Pool<Postgres>,
    keycloak_sub: &str,
    approved: bool,
    trace_id: &str,
) -> Result<KycStatusResponse, ServiceError> {
    let customer = customer_repository::get_customer_with_profile_by_sub(pool, keycloak_sub)
        .await
        .map_err(|_| ServiceError::internal("Could not fetch customer"))?;

    let Some(customer) = customer else {
        return Err(ServiceError::not_found("Customer profile not found"));
    };

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| ServiceError::internal("Could not start transaction"))?;

    let case = kyc_repository::confirm_kyc(&mut tx, &customer.customer.id, approved)
        .await
        .map_err(|_| ServiceError::not_found("KYC case not found"))?;

    if approved {
        kyc_repository::set_customer_status_active(&mut tx, &customer.customer.id)
            .await
            .map_err(|_| ServiceError::internal("Failed to activate customer"))?;
    }

    audit_repository::record_event(
        &mut tx,
        "SYSTEM",
        "kyc-review",
        "KYC_CONFIRMED",
        "KYC_CASE",
        &case.id,
        Some(serde_json::json!({ "approved": approved })),
        trace_id,
    )
    .await
    .map_err(|_| ServiceError::internal("Failed to record audit event"))?;

    tx.commit()
        .await
        .map_err(|_| ServiceError::internal("Could not commit transaction"))?;

    Ok(KycStatusResponse {
        customer_id: case.customer_id,
        kyc_case_id: case.id,
        status: case.status,
    })
}

pub async fn kyc_status(
    pool: &Pool<Postgres>,
    keycloak_sub: &str,
) -> Result<KycStatusResponse, ServiceError> {
    let customer = customer_repository::get_customer_with_profile_by_sub(pool, keycloak_sub)
        .await
        .map_err(|_| ServiceError::internal("Could not fetch customer"))?;

    let Some(customer) = customer else {
        return Err(ServiceError::not_found("Customer profile not found"));
    };

    let case = kyc_repository::get_kyc_by_customer(pool, &customer.customer.id)
        .await
        .map_err(|_| ServiceError::internal("Could not fetch KYC status"))?;

    let Some(case) = case else {
        return Err(ServiceError::not_found("KYC case not found"));
    };

    Ok(KycStatusResponse {
        customer_id: case.customer_id,
        kyc_case_id: case.id,
        status: case.status,
    })
}
