//! Nityaksha Prompt Analysis Loop & Dynamic Few-Shot Learning Engine
//! Vector search prompt matching, dynamic success few-shot injection, negative reinforcement cache, and intent classification.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessExample {
    pub prompt_summary: String,
    pub solution_code: String,
    pub verified_by_judge: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegativePattern {
    pub pattern_id: String,
    pub trigger_phrase: String,
    pub failure_mode: String,
    pub penalty_instruction: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelTier {
    FastFlash,    // Lightweight, fast, low-cost model for simple queries
    DeepReasoning // Large, heavy reasoning model for complex architectural refactors
}

pub struct PromptLoopEngine {
    success_database: HashMap<String, SuccessExample>,
    negative_cache: Vec<NegativePattern>,
}

impl PromptLoopEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            success_database: HashMap::new(),
            negative_cache: Vec::new(),
        };
        engine.seed_patterns();
        engine
    }

    fn seed_patterns(&mut self) {
        // Seed default negative patterns (What Not to Do)
        self.negative_cache.push(NegativePattern {
            pattern_id: "NEG_01".into(),
            trigger_phrase: "unwrap()".into(),
            failure_mode: "Panic in production due to bare unwrap on Option/Result".into(),
            penalty_instruction: "CRITICAL: Do NOT use .unwrap(). Always use match or ? operator.".into(),
        });

        self.negative_cache.push(NegativePattern {
            pattern_id: "NEG_02".into(),
            trigger_phrase: "rm -rf".into(),
            failure_mode: "Destructive deletion attempted in workspace".into(),
            penalty_instruction: "CRITICAL: Do NOT execute destructive rm commands. Use non-destructive rollbacks.".into(),
        });
    }

    /// Classify prompt complexity to determine model routing tier
    pub fn classify_intent(prompt: &str) -> ModelTier {
        let p_lower = prompt.to_lowercase();
        let is_complex = p_lower.contains("architect")
            || p_lower.contains("refactor")
            || p_lower.contains("distributed")
            || p_lower.contains("lock-free")
            || p_lower.contains("security audit")
            || p_lower.contains("kernel")
            || p_lower.contains("cgroup");

        if is_complex {
            ModelTier::DeepReasoning
        } else {
            ModelTier::FastFlash
        }
    }

    /// Dynamically inject relevant past success examples and negative reinforcement warnings
    pub fn augment_prompt(&self, original_prompt: &str) -> String {
        let mut augmented = String::new();

        // 1. Negative Reinforcement: check if prompt touches known failure modes
        let mut negative_warnings = Vec::new();
        for neg in &self.negative_cache {
            if original_prompt.to_lowercase().contains(&neg.trigger_phrase.to_lowercase()) {
                negative_warnings.push(&neg.penalty_instruction);
            }
        }

        if !negative_warnings.is_empty() {
            augmented.push_str("\n### [WHAT NOT TO DO - INVARIANT GUARDS]\n");
            for w in negative_warnings {
                augmented.push_str(&format!("- {}\n", w));
            }
            augmented.push('\n');
        }

        // 2. Dynamic Few-Shot Injection: match against successful past patterns
        let p_lower = original_prompt.to_lowercase();
        for (kw, ex) in &self.success_database {
            if p_lower.contains(kw) && ex.verified_by_judge {
                augmented.push_str(&format!(
                    "\n### [DYNAMIC FEW-SHOT SUCCESS EXAMPLE]\nGoal: {}\nSolution Snippet:\n```\n{}\n```\n\n",
                    ex.prompt_summary, ex.solution_code
                ));
                break;
            }
        }

        augmented.push_str("### [USER REQUEST]\n");
        augmented.push_str(original_prompt);
        augmented
    }

    /// Record a successful execution into the few-shot database
    pub fn record_success(&mut self, keyword: &str, summary: &str, solution: &str) {
        self.success_database.insert(
            keyword.to_lowercase(),
            SuccessExample {
                prompt_summary: summary.to_string(),
                solution_code: solution.to_string(),
                verified_by_judge: true,
            },
        );
    }
}
