use sqlx::{Pool, Postgres, Transaction};

use crate::models::{
    domain::customer::{Customer, CustomerProfile, CustomerWithProfile},
    dto::customer_dto::RegisterRequest,
};

pub async fn insert_customer_with_profile(
    tx: &mut Transaction<'_, Postgres>,
    keycloak_sub: &str,
    payload: &RegisterRequest,
) -> Result<CustomerWithProfile, sqlx::Error> {
    let customer = sqlx::query_as::<_, Customer>(
        r#"
        INSERT INTO customers (keycloak_sub, username, email, status)
        VALUES ($1, $2, $3, 'PENDING')
        RETURNING id::text, keycloak_sub, username, email, status, created_at, updated_at
        "#,
    )
    .bind(keycloak_sub)
    .bind(&payload.username)
    .bind(&payload.email)
    .fetch_one(&mut **tx)
    .await?;

    let profile = sqlx::query_as::<_, CustomerProfile>(
        r#"
        INSERT INTO customer_profiles (customer_id, full_name, street, city, province, postal_code, country, nas)
        VALUES ($1::uuid, $2, $3, $4, $5, $6, $7, $8)
        RETURNING customer_id::text, full_name, street, city, province, postal_code, country, nas, created_at
        "#,
    )
    .bind(&customer.id)
    .bind(&payload.full_name)
    .bind(&payload.street)
    .bind(&payload.city)
    .bind(&payload.province)
    .bind(&payload.postal_code)
    .bind(&payload.country)
    .bind(&payload.nas)
    .fetch_one(&mut **tx)
    .await?;

    Ok(CustomerWithProfile { customer, profile })
}

pub async fn get_customer_with_profile_by_sub(
    pool: &Pool<Postgres>,
    keycloak_sub: &str,
) -> Result<Option<CustomerWithProfile>, sqlx::Error> {
    let row = sqlx::query_as::<_, Customer>(
        r#"
        SELECT id::text, keycloak_sub, username, email, status, created_at, updated_at
        FROM customers
        WHERE keycloak_sub = $1
        "#,
    )
    .bind(keycloak_sub)
    .fetch_optional(pool)
    .await?;

    let Some(customer) = row else {
        return Ok(None);
    };

    let profile = sqlx::query_as::<_, CustomerProfile>(
        r#"
        SELECT customer_id::text, full_name, street, city, province, postal_code, country, nas, created_at
        FROM customer_profiles
        WHERE customer_id = $1::uuid
        "#,
    )
    .bind(&customer.id)
    .fetch_one(pool)
    .await?;

    Ok(Some(CustomerWithProfile { customer, profile }))
}

pub async fn get_customer_by_username(
    pool: &Pool<Postgres>,
    username: &str,
) -> Result<Option<Customer>, sqlx::Error> {
    sqlx::query_as::<_, Customer>(
        r#"
        SELECT id::text, keycloak_sub, username, email, status, created_at, updated_at
        FROM customers
        WHERE username = $1
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await
}

pub async fn rebind_customer_sub_by_identity(
    pool: &Pool<Postgres>,
    new_keycloak_sub: &str,
    username: &str,
    email: &str,
) -> Result<bool, sqlx::Error> {
    let updated = sqlx::query(
        r#"
        UPDATE customers
        SET keycloak_sub = $1,
            updated_at = NOW()
        WHERE username = $2
          AND email = $3
          AND keycloak_sub <> $1
        "#,
    )
    .bind(new_keycloak_sub)
    .bind(username)
    .bind(email)
    .execute(pool)
    .await?;

    Ok(updated.rows_affected() > 0)
}
