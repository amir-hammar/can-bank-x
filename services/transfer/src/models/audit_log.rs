#[derive(Debug, Clone)]
pub struct AuditLog {
    pub event_id: String,
    pub actor_type: String,
    pub actor_id: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub trace_id: Option<String>,
    pub created_at: String,
}
