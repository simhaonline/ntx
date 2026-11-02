//! Nityaksha Automatic Version Tracking & MVCC Engine
//! Immutable commit history, cryptographic state hashing, delta tracking, and timeline time travel.

use crate::db::multimodal::to_hex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionCommit {
    pub version_num: usize,
    pub commit_hash: String,
    pub parent_hash: Option<String>,
    pub author: String,
    pub timestamp: String,
    pub summary: String,
    pub state_snapshot: serde_json::Value,
    pub delta: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VersionTimeline {
    pub entity_id: String,
    pub commits: Vec<VersionCommit>,
}

pub struct VersionManager {
    timelines: Arc<RwLock<HashMap<String, VersionTimeline>>>,
}

impl VersionManager {
    pub fn new() -> Self {
        Self {
            timelines: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a state mutation as an immutable commit
    pub fn record_mutation(
        &self,
        entity_id: &str,
        summary: &str,
        payload: serde_json::Value,
        author: &str,
    ) -> VersionCommit {
        let mut timelines = self.timelines.write().unwrap();
        let timeline = timelines
            .entry(entity_id.to_string())
            .or_insert_with(|| VersionTimeline {
                entity_id: entity_id.to_string(),
                commits: Vec::new(),
            });

        let version_num = timeline.commits.len() + 1;
        let parent_hash = timeline.commits.last().map(|c| c.commit_hash.clone());

        let payload_bytes = serde_json::to_vec(&payload).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(&payload_bytes);
        if let Some(ref p) = parent_hash {
            hasher.update(p.as_bytes());
        }
        let commit_hash = to_hex(&hasher.finalize());

        let commit = VersionCommit {
            version_num,
            commit_hash,
            parent_hash,
            author: author.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            summary: summary.to_string(),
            state_snapshot: payload,
            delta: None,
        };

        timeline.commits.push(commit.clone());
        commit
    }

    /// Retrieve state at specific revision number (1-based)
    pub fn get_version(&self, entity_id: &str, version_num: usize) -> Option<VersionCommit> {
        let timelines = self.timelines.read().unwrap();
        timelines.get(entity_id).and_then(|t| {
            if version_num > 0 && version_num <= t.commits.len() {
                Some(t.commits[version_num - 1].clone())
            } else {
                None
            }
        })
    }

    /// Retrieve latest version
    pub fn get_latest(&self, entity_id: &str) -> Option<VersionCommit> {
        let timelines = self.timelines.read().unwrap();
        timelines.get(entity_id).and_then(|t| t.commits.last().cloned())
    }

    /// Retrieve full timeline history
    pub fn get_history(&self, entity_id: &str) -> Vec<VersionCommit> {
        let timelines = self.timelines.read().unwrap();
        timelines
            .get(entity_id)
            .map(|t| t.commits.clone())
            .unwrap_or_default()
    }
}
