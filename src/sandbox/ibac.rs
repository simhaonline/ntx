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

        // 4. Unauthorized Live Trading Order Execution & Production Git Push Gating (SEC-005)
        if (cmd_lower.contains("place_order")
            || cmd_lower.contains("broker.execute")
            || cmd_lower.contains("order_type: market")
            || cmd_lower.contains("submit_order")
            || cmd_lower.contains("git push --force")
            || cmd_lower.contains("git push origin main")
            || cmd_lower.contains("git push origin master"))
            && !cmd_lower.contains("dry_run")
            && !cmd_lower.contains("paper_trade")
        {
            return GateEvaluation {
                decision: GateDecision::RequireHumanApproval,
                rule_matched: Some("HITL_IRREVERSIBLE_MUTATION_GATE".into()),
                reason: "Irreversible external mutation or live broker order detected. Explicit human authorization required.".into(),
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

        // 6. Obfuscated Subshell, Pipe-to-Shell & Credential Access Guard (SEC-005, S-006)
        if cmd_lower.contains("| sh")
            || cmd_lower.contains("| bash")
            || cmd_lower.contains("|sh")
            || cmd_lower.contains("|bash")
            || cmd_lower.contains("base64 -d")
            || cmd_lower.contains("base64 --decode")
            || cmd_lower.contains("/etc/shadow")
            || cmd_lower.contains("/etc/sudoers")
            || cmd_lower.contains(".ssh/id_rsa")
            || cmd_lower.contains(".ssh/id_ed25519")
            || cmd_lower.contains("os.system(")
            || cmd_lower.contains("subprocess.popen(")
            || (cmd_lower.contains("$(") && (cmd_lower.contains("rm ") || cmd_lower.contains("curl ") || cmd_lower.contains("wget ") || cmd_lower.contains("base64")))
        {
            return GateEvaluation {
                decision: GateDecision::Block,
                rule_matched: Some("OBFUSCATED_SHELL_OR_CREDENTIAL_ESCAPE".into()),
                reason: "Obfuscated subshell execution, pipe-to-shell, or host credential access is blocked (S-006).".into(),
            };
        }

        GateEvaluation {
            decision: GateDecision::Allow,
            rule_matched: None,
            reason: "Command complies with all Nityaksha sandbox invariants.".into(),
        }
    }
}
