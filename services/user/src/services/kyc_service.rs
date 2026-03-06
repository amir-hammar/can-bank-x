use sqlx::{Pool, Postgres};

use crate::{
    models::domain::customer::CustomerWithProfile,
    models::dto::kyc_dto::KycStatusResponse,
    repositories::{audit_repository, customer_repository, kyc_repository},
    services::kyc_mock_service::{evaluate_decision, load_config, KycDecision},
    services::ServiceError,
};

pub async fn submit_kyc(
    pool: &Pool<Postgres>,
    keycloak_sub: &str,
    kyc_mock_data_path: &str,
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
        audit_repository::AuditEvent {
            actor_type: "CUSTOMER",
            actor_id: keycloak_sub,
            action: "KYC_SUBMITTED",
            entity_type: "KYC_CASE",
            entity_id: &case.id,
            metadata: None,
            trace_id,
        },
    )
    .await
    .map_err(|_| ServiceError::internal("Failed to record audit event"))?;

    tx.commit()
        .await
        .map_err(|_| ServiceError::internal("Could not commit transaction"))?;

    if case.status == "ACTIVE" {
        return Ok(KycStatusResponse {
            customer_id: case.customer_id,
            kyc_case_id: case.id,
            status: "APPROVED".to_string(),
            approved: Some(true),
            decision_available_in_seconds: 0,
        });
    }

    let config = load_config(kyc_mock_data_path).map_err(|message| ServiceError::internal(&message))?;

    Ok(KycStatusResponse {
        customer_id: case.customer_id,
        kyc_case_id: case.id,
        status: "PENDING".to_string(),
        approved: None,
        decision_available_in_seconds: config.decision_delay_seconds.max(0),
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

    let case = if approved {
        let case = kyc_repository::confirm_kyc(&mut tx, &customer.customer.id, true)
            .await
            .map_err(|_| ServiceError::not_found("KYC case not found"))?;

        kyc_repository::set_customer_status_active(&mut tx, &customer.customer.id)
            .await
            .map_err(|_| ServiceError::internal("Failed to activate customer"))?;

        case
    } else {
        kyc_repository::confirm_kyc(&mut tx, &customer.customer.id, false)
            .await
            .map_err(|_| ServiceError::not_found("KYC case not found"))?
    };

    audit_repository::record_event(
        &mut tx,
        audit_repository::AuditEvent {
            actor_type: "SYSTEM",
            actor_id: "kyc-review",
            action: "KYC_CONFIRMED",
            entity_type: "KYC_CASE",
            entity_id: &case.id,
            metadata: Some(serde_json::json!({ "approved": approved })),
            trace_id,
        },
    )
    .await
    .map_err(|_| ServiceError::internal("Failed to record audit event"))?;

    tx.commit()
        .await
        .map_err(|_| ServiceError::internal("Could not commit transaction"))?;

    Ok(KycStatusResponse {
        customer_id: case.customer_id,
        kyc_case_id: case.id,
        status: if approved {
            "APPROVED".to_string()
        } else {
            "REJECTED".to_string()
        },
        approved: Some(approved),
        decision_available_in_seconds: 0,
    })
}

pub async fn kyc_status(
    pool: &Pool<Postgres>,
    keycloak_sub: &str,
    kyc_mock_data_path: &str,
    trace_id: &str,
) -> Result<KycStatusResponse, ServiceError> {
    let customer = customer_repository::get_customer_with_profile_by_sub(pool, keycloak_sub)
        .await
        .map_err(|_| ServiceError::internal("Could not fetch customer"))?;

    let Some(customer) = customer else {
        return Err(ServiceError::not_found("Customer profile not found"));
    };

    resolve_kyc_status_for_customer(pool, &customer, kyc_mock_data_path, trace_id).await
}

pub async fn resolve_kyc_status_for_customer(
    pool: &Pool<Postgres>,
    customer: &CustomerWithProfile,
    kyc_mock_data_path: &str,
    trace_id: &str,
) -> Result<KycStatusResponse, ServiceError> {
    let case = kyc_repository::get_kyc_by_customer(pool, &customer.customer.id)
        .await
        .map_err(|_| ServiceError::internal("Could not fetch KYC status"))?;

    let Some(case) = case else {
        return Err(ServiceError::not_found("KYC case not found"));
    };

    if case.status == "ACTIVE" {
        return Ok(KycStatusResponse {
            customer_id: case.customer_id,
            kyc_case_id: case.id,
            status: "APPROVED".to_string(),
            approved: Some(true),
            decision_available_in_seconds: 0,
        });
    }

    let config = load_config(kyc_mock_data_path).map_err(|message| ServiceError::internal(&message))?;

    match evaluate_decision(&config, case.created_at, &customer.profile.full_name, &customer.profile.nas) {
        KycDecision::Pending { remaining_seconds } => Ok(KycStatusResponse {
            customer_id: case.customer_id,
            kyc_case_id: case.id,
            status: "PENDING".to_string(),
            approved: None,
            decision_available_in_seconds: remaining_seconds.max(0),
        }),
        KycDecision::Approved => {
            let mut tx = pool
                .begin()
                .await
                .map_err(|_| ServiceError::internal("Could not start transaction"))?;

            let confirmed = kyc_repository::confirm_kyc(&mut tx, &customer.customer.id, true)
                .await
                .map_err(|_| ServiceError::internal("Failed to update KYC status"))?;

            kyc_repository::set_customer_status_active(&mut tx, &customer.customer.id)
                .await
                .map_err(|_| ServiceError::internal("Failed to activate customer"))?;

            audit_repository::record_event(
                &mut tx,
                audit_repository::AuditEvent {
                    actor_type: "SYSTEM",
                    actor_id: "kyc-mock-engine",
                    action: "KYC_CONFIRMED",
                    entity_type: "KYC_CASE",
                    entity_id: &confirmed.id,
                    metadata: Some(serde_json::json!({ "approved": true, "mode": "mock-data" })),
                    trace_id,
                },
            )
            .await
            .map_err(|_| ServiceError::internal("Failed to record audit event"))?;

            tx.commit()
                .await
                .map_err(|_| ServiceError::internal("Could not commit transaction"))?;

            Ok(KycStatusResponse {
                customer_id: confirmed.customer_id,
                kyc_case_id: confirmed.id,
                status: "APPROVED".to_string(),
                approved: Some(true),
                decision_available_in_seconds: 0,
            })
        }
        KycDecision::Rejected => Ok(KycStatusResponse {
            customer_id: case.customer_id,
            kyc_case_id: case.id,
            status: "REJECTED".to_string(),
            approved: Some(false),
            decision_available_in_seconds: 0,
        }),
    }
}
