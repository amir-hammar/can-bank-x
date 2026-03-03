use serde_json::Value;
use sqlx::{Postgres, Transaction};

pub async fn record_event(
    tx: &mut Transaction<'_, Postgres>,
    actor_type: &str,
    actor_id: &str,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    metadata: Option<Value>,
    trace_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO audit_log (actor_type, actor_id, action, entity_type, entity_id, metadata, trace_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(actor_type)
    .bind(actor_id)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(metadata)
    .bind(trace_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}
