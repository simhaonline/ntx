//! Nityaksha Multi-Tier Memory & Context State Management
//! Short-Term Context Window Cache, Episodic Trajectory Memory, Long-Term Persistence, and State Checkpointing.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCheckpoint {
    pub session_id: String,
    pub step_index: usize,
    pub status: String,
    pub goal: String,
    pub completed_steps: Vec<String>,
    pub pending_steps: Vec<String>,
    pub accumulated_context: Vec<ChatMessage>,
    pub updated_at: String,
}

pub struct HarnessMemory {
    checkpoint_dir: PathBuf,
    short_term_window: VecDeque<ChatMessage>,
    max_window_size: usize,
    long_term_guidelines: String,
}

impl HarnessMemory {
    pub fn new(checkpoint_dir: impl AsRef<Path>, max_window_size: usize) -> io::Result<Self> {
        let cp_dir = checkpoint_dir.as_ref().to_path_buf();
        fs::create_dir_all(&cp_dir)?;

        // Load long-term persistence guidelines (e.g. GEMINI.md, AGENTS.md, NITYAKSHA.md)
        let mut guidelines = String::new();
        for candidate in ["/root/install/GEMINI.md", "/root/install/AGENTS.md", "NITYAKSHA.md"] {
            let p = Path::new(candidate);
            if p.exists() {
                if let Ok(c) = fs::read_to_string(p) {
                    guidelines.push_str(&format!("\n# North Star Guidelines ({})\n{}\n", candidate, c));
                    break;
                }
            }
        }

        Ok(Self {
            checkpoint_dir: cp_dir,
            short_term_window: VecDeque::with_capacity(max_window_size),
            max_window_size,
            long_term_guidelines: guidelines,
        })
    }

    /// Add a message to the active short-term sliding window
    pub fn push_message(&mut self, role: &str, content: &str) {
        if self.short_term_window.len() >= self.max_window_size {
            self.short_term_window.pop_front();
        }
        self.short_term_window.push_back(ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
    }

    pub fn get_active_window(&self) -> Vec<ChatMessage> {
        self.short_term_window.iter().cloned().collect()
    }

    pub fn get_long_term_guidelines(&self) -> &str {
        &self.long_term_guidelines
    }

    /// Save state checkpoint to disk continuously for crash resilience
    pub fn save_checkpoint(&self, checkpoint: &TaskCheckpoint) -> io::Result<()> {
        let cp_file = self.checkpoint_dir.join(format!("{}.json", checkpoint.session_id));
        let temp_file = cp_file.with_extension("tmp");
        let data = serde_json::to_vec_pretty(checkpoint)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        fs::write(&temp_file, data)?;
        fs::rename(temp_file, cp_file)?;
        Ok(())
    }

    /// Resume an interrupted task exactly from its checkpoint
    pub fn load_checkpoint(&self, session_id: &str) -> Option<TaskCheckpoint> {
        let cp_file = self.checkpoint_dir.join(format!("{}.json", session_id));
        if cp_file.exists() {
            if let Ok(data) = fs::read(&cp_file) {
                if let Ok(cp) = serde_json::from_slice::<TaskCheckpoint>(&data) {
                    return Some(cp);
                }
            }
        }
        None
    }
}
