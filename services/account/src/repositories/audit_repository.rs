use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct AuditEvent {
    pub actor_type: String,
    pub actor_id: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub trace_id: Option<String>,
    pub created_at: String,
}

#[derive(Clone)]
pub struct AuditRepository {
    events: Arc<Mutex<Vec<AuditEvent>>>,
}

impl AuditRepository {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
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
        let mut guard = self.events.lock().expect("account audit mutex poisoned");
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string());

        guard.push(AuditEvent {
            actor_type: actor_type.to_string(),
            actor_id: actor_id.to_string(),
            action: action.to_string(),
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            trace_id,
            created_at,
        });
    }
}

impl Default for AuditRepository {
    fn default() -> Self {
        Self::new()
    }
}
