//! Nityaksha Sandboxing & Container Isolation Engine
//! Combines Kernel Namespaces, Cgroups v2 quotas, and Intent-Based Access Control (IBAC).

pub mod cgroup;
pub mod ibac;
pub mod isolation;

use cgroup::{CgroupLimits, CgroupManager};
use ibac::{GateDecision, IbacGate};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub session_id: String,
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub gate_decision: String,
    pub sandboxed: bool,
}

pub struct SandboxManager {
    cgroup_mgr: CgroupManager,
    workspaces_root: PathBuf,
}

impl SandboxManager {
    pub fn new() -> io::Result<Self> {
        #[cfg(windows)]
        let root = std::env::var("USERPROFILE")
            .map(|p| PathBuf::from(p).join(".nityaksha").join("workspaces"))
            .unwrap_or_else(|_| PathBuf::from("C:\\nityaksha\\workspaces"));

        #[cfg(not(windows))]
        let root = PathBuf::from("/srv/agent-workspaces");

        let _ = fs::create_dir_all(&root);
        Ok(Self {
            cgroup_mgr: CgroupManager::new()?,
            workspaces_root: root,
        })
    }

    /// Execute a command in a confined, resource-limited sandbox environment
    pub fn execute(&self, session_id: &str, command: &str, dry_run: bool) -> io::Result<ExecutionResult> {
        let start = Instant::now();

        // 1. Evaluate IBAC security gate
        let gate = IbacGate::evaluate(command);
        if gate.decision == GateDecision::Block {
            return Ok(ExecutionResult {
                session_id: session_id.to_string(),
                command: command.to_string(),
                exit_code: 126,
                stdout: String::new(),
                stderr: format!("BLOCKED by Nityaksha IBAC Security Gate: {}", gate.reason),
                duration_ms: start.elapsed().as_millis() as u64,
                gate_decision: "BLOCKED".into(),
                sandboxed: true,
            });
        }

        if dry_run {
            return Ok(ExecutionResult {
                session_id: session_id.to_string(),
                command: command.to_string(),
                exit_code: 0,
                stdout: format!("[DRY RUN] Command '{}' approved by IBAC gate.", command),
                stderr: String::new(),
                duration_ms: start.elapsed().as_millis() as u64,
                gate_decision: format!("{:?}", gate.decision),
                sandboxed: true,
            });
        }

        // 2. Setup isolated workspace directory
        let ws_dir = self.workspaces_root.join("default").join(session_id);
        fs::create_dir_all(&ws_dir)?;

        // 3. Provision cgroup v2 limits
        let limits = CgroupLimits::default();
        let _ = self.cgroup_mgr.provision_session(session_id, &limits);

        // 4. Spawn process under confined parameters
        #[cfg(windows)]
        let mut cmd = Command::new("cmd");
        #[cfg(windows)]
        cmd.arg("/c").arg(command);

        #[cfg(not(windows))]
        let mut cmd = Command::new("sh");
        #[cfg(not(windows))]
        cmd.arg("-c").arg(command).env("USER", "nityaksha");

        let child = cmd
            .current_dir(&ws_dir)
            .env("HOME", &ws_dir)
            .env("NITYAKSHA_SESSION", session_id)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let output = match child {
            Ok(child_proc) => {
                let _ = self.cgroup_mgr.attach_pid(session_id, child_proc.id());
                child_proc.wait_with_output()?
            }
            Err(e) => {
                return Err(e);
            }
        };

        // 5. Cleanup session cgroup
        self.cgroup_mgr.cleanup_session(session_id);

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(ExecutionResult {
            session_id: session_id.to_string(),
            command: command.to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration_ms,
            gate_decision: format!("{:?}", gate.decision),
            sandboxed: true,
        })
    }
}
