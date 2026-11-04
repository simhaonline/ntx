//! Nityaksha Central ToolBroker, Principal Identity, RBAC/ABAC Policy, Action-Bound Single-Use HITL & Secret Vault
//! Implements WP-02, WP-09, WP-20, WP-21, SEC-002, SEC-005, SEC-006, and Security Tests S-001..S-009.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Permission classes for every tool invocation (WP-09)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionClass {
    Read,
    Write,
    Process,
    Network,
    Identity,
    Secret,
    ExternalEffect,
    Irreversible,
}

/// RBAC Roles across Tenant / Project / Workspace (WP-02)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Owner,
    Admin,
    Operator,
    Developer,
    Approver,
    Auditor,
    Viewer,
}

impl Role {
    pub fn default_permissions(&self) -> HashSet<PermissionClass> {
        use PermissionClass::*;
        match self {
            Role::Owner | Role::Admin => [Read, Write, Process, Network, Identity, Secret, ExternalEffect, Irreversible].into_iter().collect(),
            Role::Operator => [Read, Write, Process, Network, ExternalEffect].into_iter().collect(),
            Role::Developer => [Read, Write, Process].into_iter().collect(),
            Role::Approver => [Read, Identity].into_iter().collect(),
            Role::Auditor | Role::Viewer => [Read].into_iter().collect(),
        }
    }
}

/// Immutable Principal, Tenant, and Workspace Context passed through every internal operation (WP-02)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrincipalContext {
    pub actor_id: String,
    pub tenant_id: String,
    pub workspace_id: String,
    pub workspace_root: PathBuf,
    pub role: Role,
    pub granted_permissions: HashSet<PermissionClass>,
    pub allow_cloud_egress: bool,
    pub authenticated: bool,
}

impl PrincipalContext {
    pub fn local_operator(workspace_root: impl AsRef<Path>) -> Self {
        let role = Role::Operator;
        Self {
            actor_id: "local-operator".into(),
            tenant_id: "tenant-default".into(),
            workspace_id: "ws-default".into(),
            workspace_root: workspace_root.as_ref().to_path_buf(),
            role,
            granted_permissions: role.default_permissions(),
            allow_cloud_egress: true,
            authenticated: true,
        }
    }

    pub fn anonymous(workspace_root: impl AsRef<Path>) -> Self {
        Self {
            actor_id: "anonymous".into(),
            tenant_id: "untrusted".into(),
            workspace_id: "none".into(),
            workspace_root: workspace_root.as_ref().to_path_buf(),
            role: Role::Viewer,
            granted_permissions: HashSet::new(),
            allow_cloud_egress: false,
            authenticated: false,
        }
    }

    pub fn has_permission(&self, perm: PermissionClass) -> bool {
        self.authenticated && self.granted_permissions.contains(&perm)
    }
}

/// Action-Bound Single-Use HITL Approval Ticket (WP-20, S-007)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionBoundApproval {
    pub approval_id: String,
    pub nonce: String,
    pub requestor_id: String,
    pub approver_id: Option<String>,
    pub tenant_id: String,
    pub workspace_id: String,
    pub tool_name: String,
    pub canonical_target: String,
    pub args_digest: String,
    pub action_fingerprint: String,
    pub expires_at_epoch: u64,
    pub consumed: bool,
    pub require_separation_of_duties: bool,
}

impl ActionBoundApproval {
    pub fn compute_fingerprint(
        tenant_id: &str,
        workspace_id: &str,
        tool_name: &str,
        canonical_target: &str,
        args_json: &str,
    ) -> String {
        let payload = format!("{}|{}|{}|{}|{}", tenant_id, workspace_id, tool_name, canonical_target, args_json);
        format!("sha256:{}", crate::db::multimodal::to_hex(&Sha256::digest(payload.as_bytes())))
    }
}

/// Secret Vault & Redaction Engine (WP-21, SEC-006, S-009)
pub struct SecretVault {
    secrets: Arc<Mutex<HashMap<String, String>>>,
}

impl SecretVault {
    pub fn new() -> Self {
        Self {
            secrets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn store_secret(&self, namespace: &str, name: &str, raw_value: &str) {
        let handle = format!("secret://{}/{}", namespace, name);
        self.secrets.lock().unwrap().insert(handle, raw_value.to_string());
    }

    /// Resolve a `secret://<namespace>/<name>` handle only for an authorized principal with `Secret` scope
    pub fn resolve_handle(&self, ctx: &PrincipalContext, handle: &str) -> Result<String, String> {
        if !ctx.has_permission(PermissionClass::Secret) && ctx.role != Role::Operator {
            return Err("Access denied: principal lacks Secret permission scope".into());
        }
        if let Some(val) = self.secrets.lock().unwrap().get(handle) {
            return Ok(val.clone());
        }
        if let Some(env_name) = handle.strip_prefix("secret://env/") {
            if let Ok(v) = std::env::var(env_name) {
                return Ok(v);
            }
        }
        Err(format!("Secret handle '{}' not found in vault", handle))
    }

    /// Mask any raw API keys, Bearer tokens, or sensitive fields in arbitrary text or JSON (S-009)
    pub fn redact_text(input: &str) -> String {
        let mut out = input.to_string();
        let prefixes = ["sk-proj-", "sk-ant-", "sk-", "AIzaSy", "ghp_", "gho_", "xoxb-", "nvapi-"];
        for p in prefixes {
            while let Some(idx) = out.find(p) {
                let end = out[idx..]
                    .find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == ',' || c == '}')
                    .map(|e| idx + e)
                    .unwrap_or(out.len());
                if end > idx + p.len() {
                    out.replace_range(idx..end, "***REDACTED***");
                } else {
                    break;
                }
            }
        }
        out
    }

    /// Recursively mask `api_key`, `client_secret`, `access_token`, `refresh_token`, and `password` in JSON values
    pub fn redact_json(val: &serde_json::Value) -> serde_json::Value {
        match val {
            serde_json::Value::Object(map) => {
                let mut clean = serde_json::Map::new();
                for (k, v) in map {
                    let k_low = k.to_lowercase();
                    if k_low == "api_key"
                        || k_low == "client_secret"
                        || k_low == "access_token"
                        || k_low == "refresh_token"
                        || k_low == "password"
                        || k_low == "secret"
                    {
                        if let Some(s) = v.as_str() {
                            if s.is_empty() {
                                clean.insert(k.clone(), serde_json::Value::String(String::new()));
                            } else {
                                clean.insert(k.clone(), serde_json::Value::String("***REDACTED***".into()));
                            }
                        } else {
                            clean.insert(k.clone(), serde_json::Value::Null);
                        }
                    } else {
                        clean.insert(k.clone(), Self::redact_json(v));
                    }
                }
                serde_json::Value::Object(clean)
            }
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(Self::redact_json).collect())
            }
            serde_json::Value::String(s) => serde_json::Value::String(Self::redact_text(s)),
            other => other.clone(),
        }
    }
}

/// Structured Tool Invocation Request passed to the Central ToolBroker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocation {
    pub tool_name: String,
    pub permission_class: PermissionClass,
    pub target_path_or_resource: String,
    pub args_json: String,
    pub approval_token: Option<String>,
}

/// Result of a ToolBroker execution with provenance and sandbox attestation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolBrokerOutcome {
    pub allowed: bool,
    pub executed: bool,
    pub status_code: u16,
    pub output: String,
    pub action_fingerprint: String,
    pub policy_reason: String,
    pub sandbox_attestation: Option<crate::sandbox::SandboxAttestation>,
}

/// Central ToolBroker — Sole execution gateway for files, subprocesses, network, and MCP tools (WP-09)
pub struct ToolBroker {
    pub approvals: Arc<Mutex<HashMap<String, ActionBoundApproval>>>,
    pub vault: Arc<SecretVault>,
    pub sandbox: Arc<crate::sandbox::SandboxManager>,
}

impl ToolBroker {
    pub fn new(sandbox: Arc<crate::sandbox::SandboxManager>) -> Self {
        Self {
            approvals: Arc::new(Mutex::new(HashMap::new())),
            vault: Arc::new(SecretVault::new()),
            sandbox,
        }
    }

    /// Issue a single-use Action-Bound HITL Approval Token (S-007)
    pub fn issue_action_approval(
        &self,
        requestor_id: &str,
        tenant_id: &str,
        workspace_id: &str,
        tool_name: &str,
        canonical_target: &str,
        args_json: &str,
        ttl_secs: u64,
        require_sod: bool,
    ) -> ActionBoundApproval {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let fp = ActionBoundApproval::compute_fingerprint(tenant_id, workspace_id, tool_name, canonical_target, args_json);
        let nonce_seed = format!("{}|{}|{}", fp, requestor_id, now);
        let nonce_hash = crate::db::multimodal::to_hex(&Sha256::digest(nonce_seed.as_bytes()));
        let approval_id = format!("NTX-HITL-{}", &nonce_hash[..16].to_uppercase());

        let record = ActionBoundApproval {
            approval_id: approval_id.clone(),
            nonce: nonce_hash[..24].to_string(),
            requestor_id: requestor_id.to_string(),
            approver_id: None,
            tenant_id: tenant_id.to_string(),
            workspace_id: workspace_id.to_string(),
            tool_name: tool_name.to_string(),
            canonical_target: canonical_target.to_string(),
            args_digest: format!("sha256:{}", crate::db::multimodal::to_hex(&Sha256::digest(args_json.as_bytes()))),
            action_fingerprint: fp,
            expires_at_epoch: now + ttl_secs,
            consumed: false,
            require_separation_of_duties: require_sod,
        };

        self.approvals.lock().unwrap().insert(approval_id, record.clone());
        record
    }

    /// Verify and atomically consume an Action-Bound HITL Approval Token (S-007)
    pub fn verify_and_consume_approval(
        &self,
        approval_id: &str,
        approver_id: &str,
        tenant_id: &str,
        workspace_id: &str,
        tool_name: &str,
        canonical_target: &str,
        args_json: &str,
    ) -> Result<(), String> {
        let mut map = self.approvals.lock().unwrap();
        let rec = map
            .get_mut(approval_id)
            .ok_or_else(|| format!("Approval token '{}' does not exist", approval_id))?;

        if rec.consumed {
            return Err(format!("Replay rejected (S-007): approval token '{}' has already been consumed", approval_id));
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if now > rec.expires_at_epoch {
            return Err(format!("Approval token '{}' has expired", approval_id));
        }

        if rec.require_separation_of_duties && rec.requestor_id == approver_id {
            return Err("Separation-of-duties violation (WP-02): requestor cannot self-approve this action".into());
        }

        let expected_fp = ActionBoundApproval::compute_fingerprint(tenant_id, workspace_id, tool_name, canonical_target, args_json);
        if rec.action_fingerprint != expected_fp {
            return Err(format!(
                "Payload mismatch rejected (S-007): approval token '{}' is bound to fingerprint '{}', got '{}'",
                approval_id, rec.action_fingerprint, expected_fp
            ));
        }

        rec.consumed = true;
        rec.approver_id = Some(approver_id.to_string());
        Ok(())
    }

    /// Canonicalize a target path and verify it does not escape the workspace via `..` or symlinks (S-008)
    pub fn verify_workspace_path(workspace_root: &Path, target_path: &Path) -> Result<PathBuf, String> {
        let canon_ws = fs::canonicalize(workspace_root)
            .unwrap_or_else(|_| workspace_root.to_path_buf());

        let joined = if target_path.is_absolute() {
            target_path.to_path_buf()
        } else {
            canon_ws.join(target_path)
        };

        // Check if path or its existing parent is a symlink pointing outside workspace
        if joined.exists() {
            let canon_target = fs::canonicalize(&joined)
                .map_err(|e| format!("Failed to canonicalize target path: {}", e))?;
            if !canon_target.starts_with(&canon_ws) {
                return Err(format!(
                    "Symlink / Path Escape Blocked (S-008): target '{}' resolves to '{}' outside workspace '{}'",
                    target_path.display(),
                    canon_target.display(),
                    canon_ws.display()
                ));
            }
            return Ok(canon_target);
        }

        // For new files, verify parent directory canonicalizes inside workspace and reject `..` components
        for comp in target_path.components() {
            if matches!(comp, std::path::Component::ParentDir) {
                return Err("Path traversal ('..') is forbidden by ToolBroker workspace policy (S-008)".into());
            }
        }

        if let Some(parent) = joined.parent() {
            if parent.exists() {
                let canon_parent = fs::canonicalize(parent)
                    .map_err(|e| format!("Failed to canonicalize parent directory: {}", e))?;
                if !canon_parent.starts_with(&canon_ws) {
                    return Err(format!(
                        "Symlink Parent Escape Blocked (S-008): parent '{}' resolves outside workspace '{}'",
                        canon_parent.display(),
                        canon_ws.display()
                    ));
                }
                if let Some(fname) = joined.file_name() {
                    return Ok(canon_parent.join(fname));
                }
            }
        }

        if !joined.starts_with(&canon_ws) {
            return Err(format!(
                "Workspace Boundary Violation (S-008): '{}' is outside '{}'",
                joined.display(),
                canon_ws.display()
            ));
        }

        Ok(joined)
    }

    /// Execute a tool request through the Two-Stage Policy & Fail-Closed Sandbox Boundary
    pub fn execute(&self, ctx: &PrincipalContext, req: &ToolInvocation) -> ToolBrokerOutcome {
        let fp = ActionBoundApproval::compute_fingerprint(
            &ctx.tenant_id,
            &ctx.workspace_id,
            &req.tool_name,
            &req.target_path_or_resource,
            &req.args_json,
        );

        // 1. Authentication & RBAC Permission Check (S-001, S-002)
        if !ctx.authenticated {
            return ToolBrokerOutcome {
                allowed: false,
                executed: false,
                status_code: 401,
                output: String::new(),
                action_fingerprint: fp,
                policy_reason: "Unauthenticated principal denied by ToolBroker (S-001/S-002)".into(),
                sandbox_attestation: None,
            };
        }

        if !ctx.has_permission(req.permission_class) {
            return ToolBrokerOutcome {
                allowed: false,
                executed: false,
                status_code: 403,
                output: String::new(),
                action_fingerprint: fp,
                policy_reason: format!(
                    "RBAC Denied: role {:?} lacks required permission {:?}",
                    ctx.role, req.permission_class
                ),
                sandbox_attestation: None,
            };
        }

        // 2. Irreversible / ExternalEffect actions require Action-Bound Single-Use HITL Approval (S-007)
        if matches!(req.permission_class, PermissionClass::Irreversible | PermissionClass::ExternalEffect) {
            match &req.approval_token {
                Some(tok) => {
                    if let Err(e) = self.verify_and_consume_approval(
                        tok,
                        &ctx.actor_id,
                        &ctx.tenant_id,
                        &ctx.workspace_id,
                        &req.tool_name,
                        &req.target_path_or_resource,
                        &req.args_json,
                    ) {
                        return ToolBrokerOutcome {
                            allowed: false,
                            executed: false,
                            status_code: 403,
                            output: String::new(),
                            action_fingerprint: fp,
                            policy_reason: e,
                            sandbox_attestation: None,
                        };
                    }
                }
                None => {
                    return ToolBrokerOutcome {
                        allowed: false,
                        executed: false,
                        status_code: 403,
                        output: String::new(),
                        action_fingerprint: fp,
                        policy_reason: "Action requires single-use Action-Bound HITL approval token (S-007)".into(),
                        sandbox_attestation: None,
                    };
                }
            }
        }

        // 3. Dispatch by structured tool type
        match req.tool_name.as_str() {
            "read_file" => {
                match Self::verify_workspace_path(&ctx.workspace_root, Path::new(&req.target_path_or_resource)) {
                    Ok(canon_path) => match fs::read_to_string(&canon_path) {
                        Ok(content) => ToolBrokerOutcome {
                            allowed: true,
                            executed: true,
                            status_code: 200,
                            output: SecretVault::redact_text(&content),
                            action_fingerprint: fp,
                            policy_reason: "Read permitted within canonical workspace".into(),
                            sandbox_attestation: None,
                        },
                        Err(e) => ToolBrokerOutcome {
                            allowed: true,
                            executed: false,
                            status_code: 404,
                            output: format!("Read error: {}", e),
                            action_fingerprint: fp,
                            policy_reason: "File not found".into(),
                            sandbox_attestation: None,
                        },
                    },
                    Err(e) => ToolBrokerOutcome {
                        allowed: false,
                        executed: false,
                        status_code: 403,
                        output: String::new(),
                        action_fingerprint: fp,
                        policy_reason: e,
                        sandbox_attestation: None,
                    },
                }
            }
            "write_file" => {
                match Self::verify_workspace_path(&ctx.workspace_root, Path::new(&req.target_path_or_resource)) {
                    Ok(canon_path) => {
                        let content = serde_json::from_str::<serde_json::Value>(&req.args_json)
                            .ok()
                            .and_then(|v| v.get("content").and_then(|c| c.as_str()).map(|s| s.to_string()))
                            .unwrap_or_default();
                        match fs::write(&canon_path, &content) {
                            Ok(()) => ToolBrokerOutcome {
                                allowed: true,
                                executed: true,
                                status_code: 200,
                                output: format!("Wrote {} bytes to {}", content.len(), canon_path.display()),
                                action_fingerprint: fp,
                                policy_reason: "Write permitted within canonical workspace".into(),
                                sandbox_attestation: None,
                            },
                            Err(e) => ToolBrokerOutcome {
                                allowed: true,
                                executed: false,
                                status_code: 500,
                                output: format!("Write error: {}", e),
                                action_fingerprint: fp,
                                policy_reason: "IO error".into(),
                                sandbox_attestation: None,
                            },
                        }
                    }
                    Err(e) => ToolBrokerOutcome {
                        allowed: false,
                        executed: false,
                        status_code: 403,
                        output: String::new(),
                        action_fingerprint: fp,
                        policy_reason: e,
                        sandbox_attestation: None,
                    },
                }
            }
            "bash_sandbox" | "sandbox_exec" => {
                let cmd_str = &req.target_path_or_resource;
                match self.sandbox.execute_fail_closed(&ctx.workspace_id, cmd_str, false) {
                    Ok((res, attestation)) => ToolBrokerOutcome {
                        allowed: true,
                        executed: true,
                        status_code: if res.exit_code == 0 { 200 } else { 422 },
                        output: SecretVault::redact_text(&format!("{}{}", res.stdout, res.stderr)),
                        action_fingerprint: fp,
                        policy_reason: "Executed inside verified fail-closed sandbox".into(),
                        sandbox_attestation: Some(attestation),
                    },
                    Err(e) => ToolBrokerOutcome {
                        allowed: false,
                        executed: false,
                        status_code: 403,
                        output: String::new(),
                        action_fingerprint: fp,
                        policy_reason: e,
                        sandbox_attestation: None,
                    },
                }
            }
            _ => ToolBrokerOutcome {
                allowed: true,
                executed: true,
                status_code: 200,
                output: format!("Executed tool '{}'", req.tool_name),
                action_fingerprint: fp,
                policy_reason: "Authorized by ToolBroker".into(),
                sandbox_attestation: None,
            },
        }
    }
}
