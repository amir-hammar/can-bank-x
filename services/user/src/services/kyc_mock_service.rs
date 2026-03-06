use chrono::{Duration, NaiveDateTime, Utc};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct KycMockConfig {
    pub decision_delay_seconds: i64,
    pub identities: Vec<KycIdentityRule>,
}

#[derive(Debug, Deserialize)]
pub struct KycIdentityRule {
    pub full_name: String,
    pub nas: String,
}

#[derive(Debug)]
pub enum KycDecision {
    Pending { remaining_seconds: i64 },
    Approved,
    Rejected,
}

pub fn load_config(path: &str) -> Result<KycMockConfig, String> {
    let raw = fs::read_to_string(path)
        .map_err(|_| format!("Could not read KYC mock data file at path: {path}"))?;

    serde_json::from_str::<KycMockConfig>(&raw)
        .map_err(|_| format!("Invalid KYC mock data JSON at path: {path}"))
}

pub fn evaluate_decision(
    config: &KycMockConfig,
    created_at: NaiveDateTime,
    full_name: &str,
    nas: &str,
) -> KycDecision {
    let decision_at = created_at + Duration::seconds(config.decision_delay_seconds.max(0));
    let now = Utc::now().naive_utc();

    if now < decision_at {
        return KycDecision::Pending {
            remaining_seconds: (decision_at - now).num_seconds(),
        };
    }

    let normalized_full_name = normalize_full_name(full_name);
    let normalized_nas = normalize_nas(nas);

    let match_found = config.identities.iter().any(|identity| {
        normalize_full_name(&identity.full_name) == normalized_full_name
            && normalize_nas(&identity.nas) == normalized_nas
    });

    if match_found {
        KycDecision::Approved
    } else {
        KycDecision::Rejected
    }
}

fn normalize_full_name(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn normalize_nas(value: &str) -> String {
    value.chars().filter(|c| c.is_ascii_digit()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> KycMockConfig {
        KycMockConfig {
            decision_delay_seconds: 30,
            identities: vec![KycIdentityRule {
                full_name: "Jane Doe".to_string(),
                nas: "123456789".to_string(),
            }],
        }
    }

    #[test]
    fn returns_approved_when_identity_matches_after_delay() {
        let config = sample_config();
        let created_at = (Utc::now() - Duration::seconds(31)).naive_utc();

        let decision = evaluate_decision(&config, created_at, "Jane Doe", "123-456-789");
        assert!(matches!(decision, KycDecision::Approved));
    }

    #[test]
    fn returns_rejected_when_identity_does_not_match_after_delay() {
        let config = sample_config();
        let created_at = (Utc::now() - Duration::seconds(31)).naive_utc();

        let decision = evaluate_decision(&config, created_at, "Jane Doe", "000000000");
        assert!(matches!(decision, KycDecision::Rejected));
    }

    #[test]
    fn returns_pending_before_delay() {
        let config = sample_config();
        let created_at = Utc::now().naive_utc();

        let decision = evaluate_decision(&config, created_at, "Jane Doe", "123456789");
        assert!(matches!(decision, KycDecision::Pending { .. }));
    }
}
