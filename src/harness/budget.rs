//! Nityaksha Task Resource Constraints & Deterministic Hard Exits
//! Enforces outer runtime budgets, physical loop counter, and oscillation thrashing detection.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBudget {
    pub max_iterations: usize,
    pub max_time_seconds: u64,
    pub max_memory_mb: usize,
    pub max_cpu_percent: u32,
    pub network_restricted: bool,
}

impl Default for TaskBudget {
    fn default() -> Self {
        Self {
            max_iterations: 25,
            max_time_seconds: 300,
            max_memory_mb: 1024,
            max_cpu_percent: 100,
            network_restricted: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExitReason {
    Completed,
    IterationLimitExceeded(usize),
    TimeBudgetExceeded(u64),
    OscillationDetected(String), // File path being repeatedly edited
    SafetyGateTriggered(String),
}

pub struct LoopController {
    budget: TaskBudget,
    start_time: Instant,
    current_iteration: usize,
    file_mutation_history: HashMap<String, usize>,
    last_file_hashes: HashMap<String, String>,
}

impl LoopController {
    pub fn new(budget: TaskBudget) -> Self {
        Self {
            budget,
            start_time: Instant::now(),
            current_iteration: 0,
            file_mutation_history: HashMap::new(),
            last_file_hashes: HashMap::new(),
        }
    }

    /// Advance the physical loop counter and verify invariants
    pub fn step(&mut self) -> Result<usize, ExitReason> {
        self.current_iteration += 1;

        if self.current_iteration > self.budget.max_iterations {
            return Err(ExitReason::IterationLimitExceeded(self.budget.max_iterations));
        }

        if self.start_time.elapsed() > Duration::from_secs(self.budget.max_time_seconds) {
            return Err(ExitReason::TimeBudgetExceeded(self.budget.max_time_seconds));
        }

        Ok(self.current_iteration)
    }

    /// Record a file mutation to detect oscillation thrashing
    pub fn record_mutation(&mut self, path: &str, content_hash: &str) -> Result<(), ExitReason> {
        let count = self.file_mutation_history.entry(path.to_string()).or_insert(0);
        *count += 1;

        // If the same file is rewritten 4 or more times in a single session, trigger hard oscillation exit
        if *count >= 4 {
            if let Some(prev_hash) = self.last_file_hashes.get(path) {
                if prev_hash != content_hash {
                    return Err(ExitReason::OscillationDetected(path.to_string()));
                }
            }
        }

        self.last_file_hashes.insert(path.to_string(), content_hash.to_string());
        Ok(())
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn iterations(&self) -> usize {
        self.current_iteration
    }
}
