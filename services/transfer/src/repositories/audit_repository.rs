use serde_json::json;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AuditRepository {
    pool: PgPool,
}

impl AuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn append(
        &self,
        actor_type: &str,
        actor_id: &str,
        action: &str,
        entity_type: &str,
        entity_id: &str,
        trace_id: Option<String>,
    ) {
        let metadata = json!({ "service": "transfer-service" });

        let _ = sqlx::query(
            "INSERT INTO audit_log (actor_type, actor_id, action, entity_type, entity_id, metadata, trace_id)
             VALUES ($1, $2, $3, $4, $5, $6::jsonb, $7)"
        )
        .bind(actor_type)
        .bind(actor_id)
        .bind(action)
        .bind(entity_type)
        .bind(entity_id)
        .bind(metadata)
        .bind(trace_id)
        .execute(&self.pool)
        .await;
    }

    pub async fn count(&self) -> usize {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM audit_log")
            .fetch_one(&self.pool)
            .await
            .unwrap_or((0,));
        count.0 as usize
    }
}

impl Default for AuditRepository {
    fn default() -> Self {
        panic!("AuditRepository::default is not supported; use AuditRepository::new(pool)")
    }
}
