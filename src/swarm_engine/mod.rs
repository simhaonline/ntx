//! Nityaksha Multi-Agent Swarm Coordination, Shared Blackboard, Resource Leases, Quorum Consensus & Job Scheduler
//! Implements WP-07, WP-08, and WP-23.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Typed Message on the Shared Swarm Blackboard (WP-07)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackboardMessage {
    pub message_id: String,
    pub causal_seq: u64,
    pub sender_agent: String,
    pub topic: String,
    pub payload: String,
    pub ttl_secs: u64,
    pub created_at_epoch: u64,
}

/// Lease-Based File / Resource Lock with Monotonic Fencing Token (WP-07)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLease {
    pub resource_id: String,
    pub owner_agent: String,
    pub fence_token: u64,
    pub expires_at_epoch: u64,
}

/// Scheduled Background Job with Retry Backoff & Dead-Letter Queue (WP-23)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    pub job_id: String,
    pub owner_id: String,
    pub cron_or_interval: String,
    pub timezone: String,
    pub payload: String,
    pub attempts: u32,
    pub max_attempts: u32,
    pub status: String, // "PENDING", "RUNNING", "COMPLETED", "DEAD_LETTER"
    pub last_result: Option<String>,
}

/// Multi-Agent Swarm Coordination Engine (Blackboard + Leases + Quorum + Scheduler)
pub struct SwarmCoordinator {
    causal_counter: AtomicU64,
    fence_counter: AtomicU64,
    pub blackboard: Arc<Mutex<Vec<BlackboardMessage>>>,
    seen_message_ids: Arc<Mutex<HashSet<String>>>,
    pub leases: Arc<Mutex<HashMap<String, ResourceLease>>>,
    pub jobs: Arc<Mutex<HashMap<String, ScheduledJob>>>,
    pub dead_letter_queue: Arc<Mutex<Vec<ScheduledJob>>>,
}

impl SwarmCoordinator {
    pub fn new() -> Self {
        Self {
            causal_counter: AtomicU64::new(1),
            fence_counter: AtomicU64::new(1),
            blackboard: Arc::new(Mutex::new(Vec::new())),
            seen_message_ids: Arc::new(Mutex::new(HashSet::new())),
            leases: Arc::new(Mutex::new(HashMap::new())),
            jobs: Arc::new(Mutex::new(HashMap::new())),
            dead_letter_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn now_epoch() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Post a deduplicated, causally-ordered message to the shared blackboard (WP-07)
    pub fn publish_message(
        &self,
        message_id: &str,
        sender_agent: &str,
        topic: &str,
        payload: &str,
        ttl_secs: u64,
    ) -> Result<BlackboardMessage, String> {
        let mut seen = self.seen_message_ids.lock().unwrap();
        if !seen.insert(message_id.to_string()) {
            return Err(format!("Duplicate blackboard message ID '{}' rejected", message_id));
        }

        let seq = self.causal_counter.fetch_add(1, Ordering::SeqCst);
        let msg = BlackboardMessage {
            message_id: message_id.to_string(),
            causal_seq: seq,
            sender_agent: sender_agent.to_string(),
            topic: topic.to_string(),
            payload: payload.to_string(),
            ttl_secs,
            created_at_epoch: Self::now_epoch(),
        };

        let mut bb = self.blackboard.lock().unwrap();
        if bb.len() >= 1024 {
            bb.remove(0);
        }
        bb.push(msg.clone());
        Ok(msg)
    }

    /// Acquire an atomic exclusive lease on a file or resource with fencing token (WP-07)
    pub fn acquire_lease(&self, resource_id: &str, agent_id: &str, ttl_secs: u64) -> Result<ResourceLease, String> {
        let mut map = self.leases.lock().unwrap();
        let now = Self::now_epoch();

        if let Some(existing) = map.get(resource_id) {
            if existing.expires_at_epoch > now && existing.owner_agent != agent_id {
                return Err(format!(
                    "Resource '{}' is locked by '{}' (fence_token={}) until {}",
                    resource_id, existing.owner_agent, existing.fence_token, existing.expires_at_epoch
                ));
            }
        }

        let token = self.fence_counter.fetch_add(1, Ordering::SeqCst);
        let lease = ResourceLease {
            resource_id: resource_id.to_string(),
            owner_agent: agent_id.to_string(),
            fence_token: token,
            expires_at_epoch: now + ttl_secs,
        };
        map.insert(resource_id.to_string(), lease.clone());
        Ok(lease)
    }

    /// Release a held resource lease
    pub fn release_lease(&self, resource_id: &str, agent_id: &str, fence_token: u64) -> Result<(), String> {
        let mut map = self.leases.lock().unwrap();
        if let Some(existing) = map.get(resource_id) {
            if existing.owner_agent == agent_id && existing.fence_token == fence_token {
                map.remove(resource_id);
                return Ok(());
            }
        }
        Err(format!("Lease release rejected for '{}': invalid owner or stale fence token", resource_id))
    }

    /// Evaluate weighted multi-agent quorum consensus before committing high-risk output (WP-07)
    pub fn evaluate_quorum(votes: &[(String, bool, f32)], required_ratio: f32) -> (bool, f32) {
        if votes.is_empty() {
            return (false, 0.0);
        }
        let mut approve_weight = 0.0f32;
        let mut total_weight = 0.0f32;
        for (_agent, approve, weight) in votes {
            let w = weight.max(0.0);
            total_weight += w;
            if *approve {
                approve_weight += w;
            }
        }
        if total_weight <= 0.0 {
            return (false, 0.0);
        }
        let ratio = approve_weight / total_weight;
        (ratio >= required_ratio, ratio)
    }

    /// Enqueue a scheduled job with deduplication and Dead-Letter Queue (DLQ) handling (WP-23)
    pub fn enqueue_job(&self, job: ScheduledJob) {
        self.jobs.lock().unwrap().insert(job.job_id.clone(), job);
    }

    /// Record a job attempt outcome; moves to Dead-Letter Queue if `attempts >= max_attempts`
    pub fn record_job_attempt(&self, job_id: &str, success: bool, detail: &str) -> Result<ScheduledJob, String> {
        let mut map = self.jobs.lock().unwrap();
        let job = map.get_mut(job_id).ok_or_else(|| format!("Job '{}' not found", job_id))?;
        job.attempts += 1;
        job.last_result = Some(detail.to_string());

        if success {
            job.status = "COMPLETED".into();
        } else if job.attempts >= job.max_attempts {
            job.status = "DEAD_LETTER".into();
            self.dead_letter_queue.lock().unwrap().push(job.clone());
        } else {
            job.status = "PENDING_RETRY".into();
        }
        Ok(job.clone())
    }
}
