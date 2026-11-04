//! Nityaksha AI Harness Architecture
//! Master controller uniting Multi-Tier Memory, Container Isolation, Invariant Gating, AST Pruning, and AI Judge.

pub mod budget;
pub mod governance;
pub mod memory;
pub mod ast_pruner;
pub mod eval_judge;
pub mod prompt_loop;
pub mod adapter;
pub mod mcp_client;

use std::path::Path;
use std::sync::{Arc, Mutex};

pub use budget::{ExitReason, LoopController, TaskBudget};
pub use governance::{GovernanceManager, HumanApprovalRequest};
pub use memory::{ChatMessage, HarnessMemory, TaskCheckpoint};
pub use ast_pruner::{AstPruner, CodeSymbol, PrunedContextSnippet};
pub use eval_judge::{DeterministicCheckResult, EvalJudgeEngine, JudgeEvaluation};
pub use prompt_loop::{ModelTier, PromptLoopEngine};
pub use adapter::{ProviderType, SharedAdapter, StandardLlmRequest, StandardLlmResponse};
pub use mcp_client::{McpHandshakeReport, McpStdioClient, McpToolSchema};

/// Central Nityaksha Autonomous AI Harness
pub struct AiHarness {
    pub memory: Arc<Mutex<HarnessMemory>>,
    pub governance: Arc<Mutex<GovernanceManager>>,
    pub prompt_loop: Arc<Mutex<PromptLoopEngine>>,
}

impl AiHarness {
    pub fn new(base_dir: impl AsRef<Path>) -> std::io::Result<Self> {
        let root = base_dir.as_ref();
        let checkpoint_dir = root.join("checkpoints");
        let audit_log = root.join("audit.log");

        let memory = Arc::new(Mutex::new(HarnessMemory::new(checkpoint_dir, 50)?));
        let governance = Arc::new(Mutex::new(GovernanceManager::new(audit_log)?));
        let prompt_loop = Arc::new(Mutex::new(PromptLoopEngine::new()));

        Ok(Self {
            memory,
            governance,
            prompt_loop,
        })
    }

    /// Run full prompt preparation loop: intent classification, dynamic few-shot injection, negative reinforcement
    pub fn prepare_prompt(&self, prompt: &str) -> (ModelTier, String) {
        let tier = PromptLoopEngine::classify_intent(prompt);
        let augmented = self.prompt_loop.lock().unwrap().augment_prompt(prompt);
        (tier, augmented)
    }

    /// Evaluate an action against HITL merge gates and record in immutable audit log
    pub fn guard_action(&self, actor: &str, action: &str, target: &str) -> Option<HumanApprovalRequest> {
        let mut gov = self.governance.lock().unwrap();
        let _ = gov.append_audit_entry(actor, action, target);
        GovernanceManager::evaluate_hitl_gate(action, target)
    }
}
