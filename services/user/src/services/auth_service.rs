use sqlx::{Pool, Postgres};

use crate::{
    models::dto::auth_dto::AuthMeResponse,
    repositories::customer_repository,
    services::ServiceError,
};

pub async fn auth_me(
    pool: &Pool<Postgres>,
    keycloak_sub: &str,
) -> Result<AuthMeResponse, ServiceError> {
    let customer = customer_repository::get_customer_with_profile_by_sub(pool, keycloak_sub)
        .await
        .map_err(|_| ServiceError::internal("Failed to fetch customer"))?;

    Ok(AuthMeResponse {
        sub: keycloak_sub.to_string(),
        email: customer.map(|c| c.customer.email),
        roles: vec!["customer".to_string()],
    })
}
