use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::audit_log::AuditLog;

#[derive(Clone)]
pub struct AuditRepository {
    events: Arc<Mutex<Vec<AuditLog>>>,
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
        let mut guard = self.events.lock().expect("audit repository mutex poisoned");
        let next_event_id = guard.len() + 1;
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string());

        guard.push(AuditLog {
            event_id: format!("evt_{}", next_event_id),
            actor_type: actor_type.to_string(),
            actor_id: actor_id.to_string(),
            action: action.to_string(),
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            trace_id,
            created_at,
        });
    }

    pub async fn count(&self) -> usize {
        self.events
            .lock()
            .expect("audit repository mutex poisoned")
            .len()
    }
}

impl Default for AuditRepository {
    fn default() -> Self {
        Self::new()
    }
}
