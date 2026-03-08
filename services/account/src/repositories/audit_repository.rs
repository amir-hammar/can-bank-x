use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct AuditEvent {
    pub actor_type: String,
    pub actor_id: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub trace_id: Option<String>,
}

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
        let _ = sqlx::query(
            "INSERT INTO audit_log (id, actor_type, actor_id, action, entity_type, entity_id, trace_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(Uuid::new_v4())
        .bind(actor_type)
        .bind(actor_id)
        .bind(action)
        .bind(entity_type)
        .bind(entity_id)
        .bind(trace_id)
        .execute(&self.pool)
        .await;
    }
}
