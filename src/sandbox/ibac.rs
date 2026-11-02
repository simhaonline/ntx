//! Nityaksha Intent-Based Access Control (IBAC) Security Gate
//! Evaluates commands and tool calls against invariant security policies before execution.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateDecision {
    Allow,
    Block,
    RequireHumanApproval,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateEvaluation {
    pub decision: GateDecision,
    pub rule_matched: Option<String>,
    pub reason: String,
}

pub struct IbacGate;

impl IbacGate {
    /// Evaluate a shell command or tool invocation intent
    pub fn evaluate(command: &str) -> GateEvaluation {
        let cmd = command.trim();
        let cmd_lower = cmd.to_lowercase();

        // 1. Host Docker Socket Escape Guard
        if cmd_lower.contains("/var/run/docker.sock") || cmd_lower.contains("docker.sock") {
            return GateEvaluation {
                decision: GateDecision::Block,
                rule_matched: Some("NO_HOST_DOCKER_SOCKET".into()),
                reason: "Direct mount or access to /var/run/docker.sock is strictly forbidden to prevent container breakouts.".into(),
            };
        }

        // 2. Destructive Root Filesystem Operations
        if cmd_lower.contains("rm -rf /")
            || cmd_lower.contains("rm -rf /*")
            || cmd_lower.contains("mkfs.")
            || cmd_lower.contains("dd if=/dev/zero of=/dev/")
            || cmd_lower.contains("dd if=/dev/urandom of=/dev/")
        {
            return GateEvaluation {
                decision: GateDecision::Block,
                rule_matched: Some("DESTRUCTIVE_FILESYSTEM_WRITE".into()),
                reason: "Command contains destructive operations targeting the host or container root filesystem.".into(),
            };
        }

        // 3. Fork Bombs
        if cmd_lower.contains(":(){ :|:& };:") || cmd_lower.contains("forkbomb") {
            return GateEvaluation {
                decision: GateDecision::Block,
                rule_matched: Some("FORK_BOMB_GUARD".into()),
                reason: "Malicious process exhaustion / fork bomb pattern detected.".into(),
            };
        }

        // 4. Unauthorized Live Trading Order Execution Gating
        if (cmd_lower.contains("place_order")
            || cmd_lower.contains("broker.execute")
            || cmd_lower.contains("order_type: market")
            || cmd_lower.contains("submit_order"))
            && !cmd_lower.contains("dry_run")
            && !cmd_lower.contains("paper_trade")
        {
            return GateEvaluation {
                decision: GateDecision::RequireHumanApproval,
                rule_matched: Some("LIVE_BROKER_MUTATION_GATE".into()),
                reason: "Live capital broker order mutation detected. Explicit human authorization required.".into(),
            };
        }

        // 5. Raw Block Device & Kernel Probing
        if cmd_lower.contains("/dev/kmem") || cmd_lower.contains("/dev/mem") || cmd_lower.contains("/proc/kcore") {
            return GateEvaluation {
                decision: GateDecision::Block,
                rule_matched: Some("KERNEL_MEMORY_ACCESS".into()),
                reason: "Direct physical memory or kernel core probing is prohibited.".into(),
            };
        }

        GateEvaluation {
            decision: GateDecision::Allow,
            rule_matched: None,
            reason: "Command complies with all Nityaksha sandbox invariants.".into(),
        }
    }
}
