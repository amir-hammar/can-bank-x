use serde_json::Value;
use sqlx::{Postgres, Transaction};

pub struct AuditEvent<'a> {
    pub actor_type: &'a str,
    pub actor_id: &'a str,
    pub action: &'a str,
    pub entity_type: &'a str,
    pub entity_id: &'a str,
    pub metadata: Option<Value>,
    pub trace_id: &'a str,
}

pub async fn record_event(
    tx: &mut Transaction<'_, Postgres>,
    event: AuditEvent<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO audit_log (actor_type, actor_id, action, entity_type, entity_id, metadata, trace_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(event.actor_type)
    .bind(event.actor_id)
    .bind(event.action)
    .bind(event.entity_type)
    .bind(event.entity_id)
    .bind(event.metadata)
    .bind(event.trace_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}
