use sqlx::{Pool, Postgres, Transaction};

use crate::models::domain::kyc_case::KycCase;

pub async fn get_or_create_pending_kyc(
    tx: &mut Transaction<'_, Postgres>,
    customer_id: &str,
) -> Result<KycCase, sqlx::Error> {
    if let Some(case) = sqlx::query_as::<_, KycCase>(
        r#"
        SELECT id::text, customer_id::text, status, created_at, updated_at
        FROM kyc_cases
        WHERE customer_id = $1::uuid
        "#,
    )
    .bind(customer_id)
    .fetch_optional(&mut **tx)
    .await?
    {
        return Ok(case);
    }

    sqlx::query_as::<_, KycCase>(
        r#"
        INSERT INTO kyc_cases (customer_id, status)
        VALUES ($1::uuid, 'PENDING')
        RETURNING id::text, customer_id::text, status, created_at, updated_at
        "#,
    )
    .bind(customer_id)
    .fetch_one(&mut **tx)
    .await
}

pub async fn get_kyc_by_customer(
    pool: &Pool<Postgres>,
    customer_id: &str,
) -> Result<Option<KycCase>, sqlx::Error> {
    sqlx::query_as::<_, KycCase>(
        r#"
        SELECT id::text, customer_id::text, status, created_at, updated_at
        FROM kyc_cases
        WHERE customer_id = $1::uuid
        "#,
    )
    .bind(customer_id)
    .fetch_optional(pool)
    .await
}

pub async fn confirm_kyc(
    tx: &mut Transaction<'_, Postgres>,
    customer_id: &str,
    approved: bool,
) -> Result<KycCase, sqlx::Error> {
    let status = if approved { "ACTIVE" } else { "PENDING" };

    sqlx::query_as::<_, KycCase>(
        r#"
        UPDATE kyc_cases
        SET status = $2
        WHERE customer_id = $1::uuid
        RETURNING id::text, customer_id::text, status, created_at, updated_at
        "#,
    )
    .bind(customer_id)
    .bind(status)
    .fetch_one(&mut **tx)
    .await
}

pub async fn set_customer_status_active(
    tx: &mut Transaction<'_, Postgres>,
    customer_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE customers
        SET status = 'ACTIVE'
        WHERE id = $1::uuid
        "#,
    )
    .bind(customer_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}
