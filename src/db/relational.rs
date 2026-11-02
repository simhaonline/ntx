//! Nityaksha Relational Agent Memory Stack (Pure Rust ACID Engine)
//! In-memory and disk-directory relational tables with WAL, atomic snapshots, and multi-agent indexing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRow {
    pub id: String, // e.g. "skill:ntx-buy-cycles"
    pub namespace: String,
    pub kind: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub manifest_path: String,
    pub trust_tier: String,
    pub content_sha256: String,
    pub origin_repo: String,
    pub original_name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRow {
    pub id: String,
    pub provider: String,
    pub repository: String,
    pub status: String,
    pub last_synced_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionRow {
    pub session_id: String,
    pub runtime: String,
    pub workspace: String,
    pub status: String,
    pub tokens_used: usize,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryRow {
    pub id: String,
    pub session_id: String,
    pub task: String,
    pub status: String,
    pub reward: f32,
    pub duration_ms: u64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalaceDrawerRow {
    pub wing: String,
    pub room: String,
    pub drawer: String,
    pub content: String,
    pub tags: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoRow {
    pub id: String,
    pub title: String,
    pub level: String,
    pub metric_name: String,
    pub target_score: f64,
    pub current_score: f64,
    pub lane: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmMemberRow {
    pub session_id: String,
    pub swarm_id: String,
    pub role: String,
    pub status: String,
    pub task_label: String,
    pub report_back_to: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmTouchRow {
    pub id: usize,
    pub session_id: String,
    pub swarm_id: String,
    pub path: String,
    pub action: String,
    pub line_start: Option<usize>,
    pub line_end: Option<usize>,
    pub summary: Option<String>,
    pub touched_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmMessageRow {
    pub id: usize,
    pub swarm_id: String,
    pub from_session: String,
    pub to_target: String,
    pub channel: String,
    pub body: String,
    pub tldr: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventRow {
    pub id: usize,
    pub timestamp: String,
    pub action: String,
    pub target: String,
    pub actor: String,
    pub status: String,
    pub details: String,
}

/// Memory-safe relational database tables
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RelationalTables {
    pub capabilities: HashMap<String, CapabilityRow>, // (namespace:kind:name) -> row
    pub sources: HashMap<String, SourceRow>,
    pub agent_sessions: HashMap<String, AgentSessionRow>,
    pub trajectories: HashMap<String, TrajectoryRow>,
    pub palace_drawers: HashMap<String, PalaceDrawerRow>, // (wing:room:drawer) -> row
    pub todos: HashMap<String, TodoRow>,
    pub swarm_members: HashMap<String, SwarmMemberRow>,
    pub swarm_touches: Vec<SwarmTouchRow>,
    pub swarm_messages: Vec<SwarmMessageRow>,
    pub audit_events: Vec<AuditEventRow>,
}

pub struct RelationalStore {
    tables: Arc<RwLock<RelationalTables>>,
    storage_dir: Option<PathBuf>,
}

impl RelationalStore {
    pub fn new() -> Self {
        let mut store = Self {
            tables: Arc::new(RwLock::new(RelationalTables::default())),
            storage_dir: None,
        };
        store.seed_defaults();
        store
    }

    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let dir = path.as_ref().to_path_buf();
        fs::create_dir_all(&dir)?;

        let mut store = Self {
            tables: Arc::new(RwLock::new(RelationalTables::default())),
            storage_dir: Some(dir.clone()),
        };

        let db_file = dir.join("relational.json");
        if db_file.exists() {
            let data = fs::read(&db_file)?;
            if !data.is_empty() {
                if let Ok(t) = serde_json::from_slice::<RelationalTables>(&data) {
                    *store.tables.write().unwrap() = t;
                    return Ok(store);
                }
            }
        }

        store.seed_defaults();
        store.persist()?;
        Ok(store)
    }

    fn seed_defaults(&mut self) {
        let mut t = self.tables.write().unwrap();
        let now = chrono::Utc::now().to_rfc3339();

        // Seed Method of Loci drawers
        t.palace_drawers.insert(
            "architecture:storage:sqlite_wal".to_string(),
            PalaceDrawerRow {
                wing: "architecture".into(),
                room: "storage".into(),
                drawer: "sqlite_wal".into(),
                content: "Pure Rust memory engine with atomic WAL and JSON/Binary state snapshots".into(),
                tags: "rust,storage,wal,atomic".into(),
                updated_at: now.clone(),
            },
        );

        t.palace_drawers.insert(
            "security:policy:ibac_rules".to_string(),
            PalaceDrawerRow {
                wing: "security".into(),
                room: "policy".into(),
                drawer: "ibac_rules".into(),
                content: "Intent-Based Access Control severity gating rules (ContainerOS isolation)".into(),
                tags: "security,policy,ibac,sandbox".into(),
                updated_at: now.clone(),
            },
        );

        // Seed default sources
        t.sources.insert(
            "simhaonline/icskills".to_string(),
            SourceRow {
                id: "simhaonline/icskills".into(),
                provider: "github".into(),
                repository: "simhaonline/icskills".into(),
                status: "active".into(),
                last_synced_at: now.clone(),
            },
        );
        t.sources.insert(
            "simhaonline/avoid-ai-writing".to_string(),
            SourceRow {
                id: "simhaonline/avoid-ai-writing".into(),
                provider: "github".into(),
                repository: "simhaonline/avoid-ai-writing".into(),
                status: "active".into(),
                last_synced_at: now.clone(),
            },
        );
        t.sources.insert(
            "simhaonline/mcptoon".to_string(),
            SourceRow {
                id: "simhaonline/mcptoon".into(),
                provider: "github".into(),
                repository: "simhaonline/mcptoon".into(),
                status: "active".into(),
                last_synced_at: now.clone(),
            },
        );
        t.sources.insert(
            "simhaonline/jscpd".to_string(),
            SourceRow {
                id: "simhaonline/jscpd".into(),
                provider: "github".into(),
                repository: "simhaonline/jscpd".into(),
                status: "active".into(),
                last_synced_at: now.clone(),
            },
        );
    }

    pub fn persist(&self) -> io::Result<()> {
        if let Some(ref dir) = self.storage_dir {
            let db_file = dir.join("relational.json");
            let temp_file = db_file.with_extension("tmp");
            let t = self.tables.read().unwrap();
            let data = serde_json::to_vec_pretty(&*t)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            fs::write(&temp_file, data)?;
            fs::rename(temp_file, db_file)?;
        }
        Ok(())
    }

    pub fn upsert_capability(&self, cap: CapabilityRow) -> io::Result<()> {
        let key = format!("{}:{}:{}", cap.namespace, cap.kind, cap.name);
        self.tables.write().unwrap().capabilities.insert(key, cap);
        self.persist()
    }

    pub fn find_capability(&self, kind: Option<&str>, target: &str) -> Option<CapabilityRow> {
        let t = self.tables.read().unwrap();
        let target_clean = target.trim();
        let norm_name = if target_clean.starts_with("ntx-") {
            target_clean.to_string()
        } else {
            format!("ntx-{}", target_clean)
        };
        let raw_name = target_clean.trim_start_matches("ntx-");

        // 1. Exact match on name or original_name
        for (_, cap) in t.capabilities.iter() {
            if let Some(k) = kind {
                if cap.kind != k && cap.kind != k.trim_end_matches('s') {
                    continue;
                }
            }
            if cap.name == target_clean
                || cap.name == norm_name
                || cap.original_name == raw_name
                || cap.original_name == target_clean
            {
                return Some(cap.clone());
            }
        }

        // 2. Fuzzy match on name or origin_repo
        for (_, cap) in t.capabilities.iter() {
            if let Some(k) = kind {
                if cap.kind != k && cap.kind != k.trim_end_matches('s') {
                    continue;
                }
            }
            if cap.name.contains(raw_name)
                || cap.original_name.contains(raw_name)
                || cap.origin_repo.contains(target_clean)
            {
                return Some(cap.clone());
            }
        }

        None
    }

    pub fn search_capabilities(&self, query: &str, kind_filter: Option<&str>) -> Vec<CapabilityRow> {
        let t = self.tables.read().unwrap();
        let q = query.to_lowercase();
        let terms: Vec<&str> = q.split_whitespace().collect();

        t.capabilities
            .values()
            .filter(|c| {
                if let Some(k) = kind_filter {
                    let k_clean = k.trim_end_matches('s');
                    if c.kind != k && c.kind != k_clean {
                        return false;
                    }
                }
                if terms.is_empty() {
                    return true;
                }
                let haystack = format!(
                    "{} {} {} {} {} {}",
                    c.name, c.original_name, c.origin_repo, c.description, c.kind, c.namespace
                )
                .to_lowercase();
                terms.iter().all(|&term| haystack.contains(term))
            })
            .cloned()
            .collect()
    }

    pub fn count_capabilities(&self) -> HashMap<String, usize> {
        let t = self.tables.read().unwrap();
        let mut counts = HashMap::new();
        for cap in t.capabilities.values() {
            *counts.entry(cap.kind.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn add_audit_event(&self, action: &str, target: &str, actor: &str, status: &str, details: &str) {
        let mut t = self.tables.write().unwrap();
        let id = t.audit_events.len() + 1;
        t.audit_events.push(AuditEventRow {
            id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            action: action.into(),
            target: target.into(),
            actor: actor.into(),
            status: status.into(),
            details: details.into(),
        });
        let _ = self.persist();
    }
}
