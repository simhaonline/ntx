//! Nityaksha Universal Capability Registry & Metadata-First Zero-Clone Acquisition Engine
//! Implements WP-11, WP-12, Section 5 Schema, SSRF Guard (S-011), and Untrusted Instruction Quarantine (S-010).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Capability Lifecycle State Machine (WP-11 Section 4)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityState {
    Discovered,
    Indexed,
    Quarantined,
    Reviewed,
    Approved,
    Installed,
    Deprecated,
    Revoked,
}

/// Requested Permission Grants in a Capability Manifest (Section 5.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestedPermissions {
    pub filesystem_read: Vec<String>,
    pub filesystem_write: Vec<String>,
    pub process_execution: bool,
    pub network_enabled: bool,
    pub secret_handles: Vec<String>,
}

impl Default for RequestedPermissions {
    fn default() -> Self {
        Self {
            filesystem_read: vec!["workspace:**".into()],
            filesystem_write: Vec::new(),
            process_execution: false,
            network_enabled: false,
            secret_handles: Vec::new(),
        }
    }
}

/// Signed Capability Package Manifest (`nityaksha.io/v1alpha1`, Section 5.1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedCapabilityManifest {
    pub api_version: String,
    pub kind: String, // Skill | Agent | MCPServer | Plugin | Prompt | Tool | Workflow
    pub namespace: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub source_repo: String,
    pub source_revision: String,
    pub digest: String,
    pub signature: Option<String>,
    pub spdx_license: String,
    pub state: CapabilityState,
    pub requested_permissions: RequestedPermissions,
    pub quarantine_reasons: Vec<String>,
}

impl SignedCapabilityManifest {
    pub fn compute_digest(kind: &str, namespace: &str, name: &str, version: &str, content: &str) -> String {
        let raw = format!("{}|{}|{}|{}|{}", kind, namespace, name, version, content);
        format!("sha256:{}", crate::db::multimodal::to_hex(&Sha256::digest(raw.as_bytes())))
    }

    pub fn is_executable(&self) -> bool {
        matches!(self.state, CapabilityState::Approved | CapabilityState::Installed)
    }
}

/// SSRF Prevention Gate for Remote Content Acquisition (WP-12, S-011)
pub struct SsrfGuard;

impl SsrfGuard {
    /// Validate a remote URL before fetching; block metadata endpoints, loopback, private IPs, and non-HTTP schemes
    pub fn validate_url(url: &str) -> Result<(), String> {
        let trimmed = url.trim();
        let lower = trimmed.to_lowercase();

        if !lower.starts_with("https://") && !lower.starts_with("http://") {
            return Err(format!("SSRF Blocked (S-011): only http/https schemes allowed, got '{}'", trimmed));
        }

        let without_scheme = lower
            .trim_start_matches("https://")
            .trim_start_matches("http://");
        let host_port = without_scheme.split('/').next().unwrap_or("");
        let host = host_port.split(':').next().unwrap_or("");

        if host.is_empty() {
            return Err("SSRF Blocked (S-011): empty hostname".into());
        }

        // Block loopback, cloud metadata services, and RFC1918 private networks
        let forbidden_hosts = [
            "localhost",
            "127.0.0.1",
            "0.0.0.0",
            "::1",
            "[::1]",
            "169.254.169.254",
            "metadata.google.internal",
            "100.100.100.200",
        ];
        if forbidden_hosts.contains(&host) || host.starts_with("127.") || host.starts_with("10.") || host.starts_with("192.168.") || host.starts_with("169.254.") {
            return Err(format!(
                "SSRF Blocked (S-011): access to private/loopback/metadata host '{}' is forbidden",
                host
            ));
        }

        if host.starts_with("172.") {
            if let Some(second_octet) = host.split('.').nth(1).and_then(|s| s.parse::<u8>().ok()) {
                if (16..=31).contains(&second_octet) {
                    return Err(format!(
                        "SSRF Blocked (S-011): access to RFC1918 private network '{}' is forbidden",
                        host
                    ));
                }
            }
        }

        Ok(())
    }
}

/// Metadata-First Zero-Clone Content Acquisition & Registry Manager (WP-11 & WP-12)
pub struct CapabilityRegistryManager {
    pub packages: Arc<Mutex<HashMap<String, SignedCapabilityManifest>>>,
    pub max_file_bytes: usize,
    pub max_total_cache_bytes: usize,
}

impl CapabilityRegistryManager {
    pub fn new() -> Self {
        Self {
            packages: Arc::new(Mutex::new(HashMap::new())),
            max_file_bytes: 262_144, // 256 KiB per file
            max_total_cache_bytes: 134_217_728, // 128 MiB bounded fetch quota
        }
    }

    /// Inspect untrusted document/manifest content for prompt-injection or permission-escalation patterns (S-010)
    pub fn scan_untrusted_content_for_escalation(content: &str) -> Vec<String> {
        let lower = content.to_lowercase();
        let mut findings = Vec::new();
        let patterns = [
            ("ignore previous instructions", "PROMPT_INJECTION_OVERRIDE"),
            ("grant all permissions", "UNAUTHORIZED_PERMISSION_ESCALATION"),
            ("dangerously-skip-permissions", "BYPASS_SANDBOX_FLAG_INJECTION"),
            ("curl -s", "UNREVIEWED_REMOTE_FETCH"),
            ("/var/run/docker.sock", "DOCKER_SOCKET_ESCAPE"),
            ("rm -rf /", "DESTRUCTIVE_HOST_MUTATION"),
            ("export openai_api_key", "SECRET_EXFILTRATION_DIRECTIVE"),
        ];
        for (pat, code) in patterns {
            if lower.contains(pat) {
                findings.push(format!("{}: matched '{}'", code, pat));
            }
        }
        findings
    }

    /// Register and evaluate a discovered capability manifest (never auto-approves untrusted/escalating content)
    pub fn ingest_capability(
        &self,
        kind: &str,
        namespace: &str,
        name: &str,
        version: &str,
        publisher: &str,
        source_repo: &str,
        source_revision: &str,
        spdx_license: Option<&str>,
        raw_content: &str,
    ) -> Result<SignedCapabilityManifest, String> {
        if raw_content.len() > self.max_file_bytes {
            return Err(format!(
                "Zero-Clone Quota Exceeded (WP-12): content size {} bytes exceeds max_file_bytes ({})",
                raw_content.len(),
                self.max_file_bytes
            ));
        }

        let digest = SignedCapabilityManifest::compute_digest(kind, namespace, name, version, raw_content);
        let findings = Self::scan_untrusted_content_for_escalation(raw_content);
        let license = spdx_license
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("REVIEW_REQUIRED")
            .to_string();

        let initial_state = if !findings.is_empty() || license == "REVIEW_REQUIRED" {
            CapabilityState::Quarantined
        } else {
            CapabilityState::Indexed
        };

        let manifest = SignedCapabilityManifest {
            api_version: "nityaksha.io/v1alpha1".into(),
            kind: kind.to_string(),
            namespace: namespace.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            publisher: publisher.to_string(),
            source_repo: source_repo.to_string(),
            source_revision: source_revision.to_string(),
            digest,
            signature: None,
            spdx_license: license,
            state: initial_state,
            requested_permissions: RequestedPermissions::default(),
            quarantine_reasons: findings,
        };

        let key = format!("{}/{}@{}", namespace, name, version);
        self.packages.lock().unwrap().insert(key, manifest.clone());
        Ok(manifest)
    }

    /// Approve and cryptographically sign a capability package after human/security review
    pub fn approve_and_sign(&self, key: &str, reviewer_id: &str) -> Result<SignedCapabilityManifest, String> {
        let mut map = self.packages.lock().unwrap();
        let pkg = map.get_mut(key).ok_or_else(|| format!("Capability '{}' not found", key))?;

        if pkg.state == CapabilityState::Revoked {
            return Err("Cannot approve a revoked capability package".into());
        }
        if !pkg.quarantine_reasons.is_empty() {
            return Err(format!(
                "Cannot approve quarantined capability '{}' with unresolved security findings: {:?}",
                key, pkg.quarantine_reasons
            ));
        }

        let sig_payload = format!("{}|{}|APPROVED", pkg.digest, reviewer_id);
        pkg.signature = Some(format!("sig-sha256:{}", crate::db::multimodal::to_hex(&Sha256::digest(sig_payload.as_bytes()))));
        pkg.state = CapabilityState::Approved;
        Ok(pkg.clone())
    }

    /// Revoke a capability package by key; immediately prevents execution
    pub fn revoke(&self, key: &str, reason: &str) -> Result<(), String> {
        let mut map = self.packages.lock().unwrap();
        let pkg = map.get_mut(key).ok_or_else(|| format!("Capability '{}' not found", key))?;
        pkg.state = CapabilityState::Revoked;
        pkg.quarantine_reasons.push(format!("REVOKED: {}", reason));
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<SignedCapabilityManifest> {
        self.packages.lock().unwrap().get(key).cloned()
    }

    /// Build a zero-clone GitHub Git Trees API URL (`WP-12`) after validating repository format (`owner/repo`)
    pub fn build_github_tree_api_url(owner_repo: &str, git_ref: &str) -> Result<String, String> {
        let trimmed = owner_repo.trim().trim_matches('/');
        let parts: Vec<&str> = trimmed.split('/').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(format!("INVALID_REPO_SLUG: expected 'owner/repo', got '{}'", owner_repo));
        }
        let url = format!(
            "https://api.github.com/repos/{}/{}/git/trees/{}?recursive=1",
            parts[0], parts[1], git_ref
        );
        SsrfGuard::validate_url(&url)?;
        Ok(url)
    }
}
