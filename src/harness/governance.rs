//! Nityaksha Granular Governance, HITL Merge Gates & Data Leakage Prevention
//! Enforces user identity inheritance, human approval gates, secret scrubbers, and tamper-evident audit logs.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanApprovalRequest {
    pub token: String,
    pub action: String,
    pub target: String,
    pub risk_level: String,
    pub payload_summary: String,
    pub created_at: String,
    pub approved: bool,
}

pub struct GovernanceManager {
    audit_log_path: PathBuf,
    last_audit_hash: String,
}

impl GovernanceManager {
    pub fn new(log_path: impl AsRef<Path>) -> io::Result<Self> {
        let p = log_path.as_ref().to_path_buf();
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(Self {
            audit_log_path: p,
            last_audit_hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        })
    }

    /// Strip sensitive credentials and answers from environment variables before model exposure
    pub fn sanitize_environment(raw_env: &HashMap<String, String>) -> HashMap<String, String> {
        let sensitive_substrings = [
            "TOKEN", "SECRET", "KEY", "PASS", "AUTH", "CREDENTIAL", "PRIVATE",
            "DATABASE_URL", "IDN_JWT", "API_KEY", "SSH_KEY"
        ];

        raw_env
            .iter()
            .map(|(k, v)| {
                let is_sensitive = sensitive_substrings.iter().any(|sub| k.to_uppercase().contains(sub));
                if is_sensitive {
                    (k.clone(), "[REDACTED_BY_NITYAKSHA_LEAKAGE_GUARD]".to_string())
                } else {
                    (k.clone(), v.clone())
                }
            })
            .collect()
    }

    /// Human-in-the-Loop (HITL) Gate check for critical operations
    pub fn evaluate_hitl_gate(action: &str, target: &str) -> Option<HumanApprovalRequest> {
        let act_lower = action.to_lowercase();
        let is_critical = act_lower.contains("git push")
            || act_lower.contains("merge")
            || act_lower.contains("delete")
            || act_lower.contains("drop table")
            || act_lower.contains("spend_capital")
            || act_lower.contains("transfer");

        if is_critical {
            let mut hasher = Sha256::new();
            hasher.update(format!("{}:{}:{}", action, target, chrono::Utc::now().to_rfc3339()).as_bytes());
            let token = format!("HITL-{}", &crate::db::multimodal::to_hex(&hasher.finalize())[..12]);

            Some(HumanApprovalRequest {
                token,
                action: action.to_string(),
                target: target.to_string(),
                risk_level: "CRITICAL".into(),
                payload_summary: format!("Attempted execution of protected operation '{}' on '{}'", action, target),
                created_at: chrono::Utc::now().to_rfc3339(),
                approved: false,
            })
        } else {
            None
        }
    }

    /// Append to tamper-evident immutable audit log with cryptographic hash chaining
    pub fn append_audit_entry(&mut self, actor: &str, action: &str, details: &str) -> io::Result<String> {
        let ts = chrono::Utc::now().to_rfc3339();
        let payload = format!("{}|{}|{}|{}|{}", ts, actor, action, details, self.last_audit_hash);

        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        let current_hash = crate::db::multimodal::to_hex(&hasher.finalize());

        let line = format!("{} | HASH:{} | PREV:{}\n", payload, current_hash, self.last_audit_hash);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.audit_log_path)?;
        file.write_all(line.as_bytes())?;

        self.last_audit_hash = current_hash.clone();
        Ok(current_hash)
    }
}
