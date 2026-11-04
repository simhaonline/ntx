//! NITYAKSHA - Pre-Configured Bots & App Store Registry
//! Engineered for high-velocity, deterministic AI orchestration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The Bot Schema (The Registry Blueprint)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotDefinition {
    pub id: String,
    pub name: String,
    pub category: String,
    pub version: String,
    pub avatar: String,
    pub description: String,
    pub system_prompt_template: String,
    pub model_tier: String,
    pub temperature: f32,
    pub allowed_tools: Vec<String>,
    pub sandbox_policy: String,
    pub starter_prompts: Vec<String>,
    pub tags: Vec<String>,
    pub author: String,
}

/// Dynamic System Prompt Wrapper (Hydration Engine)
pub struct DynamicPromptWrapper;

impl DynamicPromptWrapper {
    /// Hydrates prompt template variables, injects security guardrails, user identity, and tool constraints
    pub fn hydrate(
        template: &str,
        vars: &HashMap<String, String>,
        allowed_tools: &[String],
        sandbox_policy: &str,
    ) -> String {
        let mut hydrated = template.to_string();

        // 1. Substitute variables (e.g. {{user_name}}, {{workspace}}, {{domain}})
        for (key, val) in vars {
            let placeholder = format!("{{{{{}}}}}", key);
            hydrated = hydrated.replace(&placeholder, val);
        }

        // 2. Inject Sandbox Policy Invariant
        let sandbox_header = format!(
            "\n\n[EXECUTION INVARIANT - SANDBOX POLICY: {}]\n- All filesystem operations are strictly confined.\n- Host Docker sockets and unauthorized broker mutations are blocked.",
            sandbox_policy.to_uppercase()
        );
        hydrated.push_str(&sandbox_header);

        // 3. Inject Permitted Tool Signatures
        if !allowed_tools.is_empty() {
            let tools_header = format!(
                "\n[PERMITTED CAPABILITY TOOLS: {}]\n- Only invoke tools explicitly granted in this whitelist.",
                allowed_tools.join(", ")
            );
            hydrated.push_str(&tools_header);
        }

        // 4. Inject Noise-Free Persona Protocol (CORE-PERSONA-01)
        hydrated.push_str("\n[SYSTEM INVARIANT]: Maintain Objective Technical Execution Mode. Zero conversational filler, zero hedging, raw high-density execution.");

        hydrated
    }
}

/// In-Memory Pre-Configured Bot Catalog
pub struct BotRegistry;

impl BotRegistry {
    /// Returns the out-of-the-box pre-configured bots for the App Store
    pub fn default_bots() -> Vec<BotDefinition> {
        vec![
            BotDefinition {
                id: "ntx-bot-concierge-prime".into(),
                name: "Nityaksha Sovereign Concierge".into(),
                category: "Executive Concierge".into(),
                version: "1.0.0".into(),
                avatar: "🏛️".into(),
                description: "Master concierge orchestrating multi-agent delegations, state checkpoints, and sovereign platform operations.".into(),
                system_prompt_template: "You are Nityaksha Sovereign Concierge, operating inside {{workspace_root}} for user {{user_name}}. Supervise agent task routing and enforce architectural invariants.".into(),
                model_tier: "DeepReasoning".into(),
                temperature: 0.2,
                allowed_tools: vec!["Read".into(), "VectorSearch".into(), "DelegateAgent".into(), "HarnessGate".into()],
                sandbox_policy: "workspace-write".into(),
                starter_prompts: vec![
                    "Audit platform health and list active agent trajectories".into(),
                    "Review security policy compliance across workspaces".into(),
                    "Delegate code refactoring to specialized background bots".into(),
                ],
                tags: vec!["concierge".into(), "orchestrator".into(), "governance".into()],
                author: "Simha (simhaonline)".into(),
            },
            BotDefinition {
                id: "ntx-bot-code-architect".into(),
                name: "Systems Code Architect".into(),
                category: "Engineering".into(),
                version: "1.0.0".into(),
                avatar: "🦀".into(),
                description: "Deep codebase discovery, AST dependency pruning, memory safety invariants, and high-velocity systems engineering.".into(),
                system_prompt_template: "You are the Systems Code Architect. Analyze AST trees in {{workspace_root}} without wasteful file dumping. Enforce zero unwrap(), zero unsafe leaks, and sub-millisecond execution.".into(),
                model_tier: "DeepReasoning".into(),
                temperature: 0.1,
                allowed_tools: vec!["Read".into(), "Edit".into(), "Bash".into(), "ASTInspect".into(), "GitDiff".into()],
                sandbox_policy: "workspace-write".into(),
                starter_prompts: vec![
                    "Analyze repository AST and prune unused function symbols".into(),
                    "Refactor error handling to eliminate panics and unwraps".into(),
                    "Synthesize unit test vectors before implementing changes".into(),
                ],
                tags: vec!["rust".into(), "ast".into(), "refactor".into(), "architecture".into()],
                author: "Simha (simhaonline)".into(),
            },
            BotDefinition {
                id: "ntx-bot-quant-risk".into(),
                name: "Quant Risk & Market Modeler".into(),
                category: "Quant & Finance".into(),
                version: "1.0.0".into(),
                avatar: "📈".into(),
                description: "Quantitative market structure engineer. Manages cumulative volume delta (CVD), liquidation zones, and order risk gating.".into(),
                system_prompt_template: "You are the Quant Risk Modeler for {{domain_context}}. Validate market microstructure, Killzones (London/NY UTC), and tick data integrity. Never mutate production trading states without HITL approval.".into(),
                model_tier: "DeepReasoning".into(),
                temperature: 0.1,
                allowed_tools: vec!["Read".into(), "VectorSearch".into(), "QuestDbReader".into(), "RiskStressTest".into()],
                sandbox_policy: "read-only".into(),
                starter_prompts: vec![
                    "Compute volume profile and CVD absorption levels for BTC/USDT".into(),
                    "Stress-test liquidation band risk under 3-sigma volatility".into(),
                    "Verify Pine Script v6 matrix and UDT execution invariants".into(),
                ],
                tags: vec!["quant".into(), "risk".into(), "market-data".into(), "tradingview".into()],
                author: "Simha (simhaonline)".into(),
            },
            BotDefinition {
                id: "ntx-bot-security-redteam".into(),
                name: "RedTeam Adversarial Shield".into(),
                category: "Security".into(),
                version: "1.0.0".into(),
                avatar: "🛡️".into(),
                description: "Proactive prompt injection defender, credential leakage scrubber, and container namespace jail inspector.".into(),
                system_prompt_template: "You are the RedTeam Adversarial Shield. Inspect prompts, environment variables, and git trees in {{workspace_root}} for injection attacks, secret leaks, and sandbox escapes.".into(),
                model_tier: "Balanced".into(),
                temperature: 0.0,
                allowed_tools: vec!["Read".into(), "LeakageSanitizer".into(), "RedTeamInspect".into()],
                sandbox_policy: "read-only".into(),
                starter_prompts: vec![
                    "Scan workspace environment for accidental API token leakage".into(),
                    "Audit model prompts for delimiter collisions and instruction bypasses".into(),
                    "Verify Linux namespace and cgroup quota isolation boundaries".into(),
                ],
                tags: vec!["security".into(), "redteam".into(), "injection-defense".into(), "cgroup".into()],
                author: "Simha (simhaonline)".into(),
            },
            BotDefinition {
                id: "ntx-bot-devops-sre".into(),
                name: "SRE Incident Commander".into(),
                category: "SRE & DevOps".into(),
                version: "1.0.0".into(),
                avatar: "⚡".into(),
                description: "Kernel panic diagnostic specialist, out-of-memory isolation, automated service remediation, and cgroups quota supervisor.".into(),
                system_prompt_template: "You are the SRE Incident Commander. Triage system logs, monitor cgroups v2 quotas in {{workspace_root}}, and trigger automated recovery runbooks.".into(),
                model_tier: "FastFlash".into(),
                temperature: 0.1,
                allowed_tools: vec!["Read".into(), "Bash".into(), "SreTriage".into(), "CgroupMonitor".into()],
                sandbox_policy: "workspace-write".into(),
                starter_prompts: vec![
                    "Triage kernel out-of-memory crash log and suggest remediation".into(),
                    "Inspect cgroups v2 memory.max and pids.max quota utilization".into(),
                    "Verify systemd service unit health and restart policies".into(),
                ],
                tags: vec!["sre".into(), "devops".into(), "cgroups".into(), "systemd".into()],
                author: "Simha (simhaonline)".into(),
            },
            BotDefinition {
                id: "ntx-bot-anti-slop-strategist".into(),
                name: "Anti-Slop Text Sanitizer".into(),
                category: "Content & Documentation".into(),
                version: "1.0.0".into(),
                avatar: "✨".into(),
                description: "High-throughput content sanitizer eliminating conversational AI boilerplate, hedging, apologies, and moralizing preamble.".into(),
                system_prompt_template: "You are the Anti-Slop Content Strategist. Strip all conversational filler, 'I'd be happy to help', apologies, and repetitive throat-clearing from all outputs.".into(),
                model_tier: "FastFlash".into(),
                temperature: 0.2,
                allowed_tools: vec!["Read".into(), "Edit".into(), "AntiSlopFilter".into()],
                sandbox_policy: "workspace-write".into(),
                starter_prompts: vec![
                    "Sanitize technical documentation to eliminate AI conversational filler".into(),
                    "Audit whitepaper manuscript for concise, high-signal engineering tone".into(),
                    "Convert verbose explanations into dense, technical reference runbooks".into(),
                ],
                tags: vec!["anti-slop".into(), "documentation".into(), "content".into(), "sanitizer".into()],
                author: "Simha (simhaonline)".into(),
            },
            BotDefinition {
                id: "ntx-bot-legal-compliance".into(),
                name: "Legal Compliance & Audit Counsel".into(),
                category: "Legal & Governance".into(),
                version: "1.0.0".into(),
                avatar: "⚖️".into(),
                description: "Autonomous specialist for cross-document synthesis, licensing clause comparison, and immutable trajectory proof verification.".into(),
                system_prompt_template: "You are the Legal Compliance Counsel. Verify licensing invariants, proprietary asset provenance, and corporate compliance audit trails.".into(),
                model_tier: "DeepReasoning".into(),
                temperature: 0.1,
                allowed_tools: vec!["Read".into(), "VectorSearch".into(), "AuditLogVerify".into()],
                sandbox_policy: "read-only".into(),
                starter_prompts: vec![
                    "Verify proprietary capability licenses against corporate policy".into(),
                    "Compare contract clauses across multiple vendor agreements".into(),
                    "Audit tamper-evident append-only logs for compliance reporting".into(),
                ],
                tags: vec!["legal".into(), "compliance".into(), "audit".into(), "governance".into()],
                author: "Simha (simhaonline)".into(),
            },
        ]
    }
}
