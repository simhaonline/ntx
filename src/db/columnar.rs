//! Nityaksha Columnar Store for High-Throughput Analytics & Telemetry
//! Column-oriented arrays for agent tool calls, latencies, token consumption, and sub-millisecond aggregations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryRow {
    pub timestamp_ms: i64,
    pub session_id: String,
    pub agent_role: String,
    pub tool_name: String,
    pub duration_ms: u64,
    pub tokens_in: u32,
    pub tokens_out: u32,
    pub cost_microcents: u64,
    pub status: String,
}

/// Columnar layout for high-throughput vectorized aggregations
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ColumnarTable {
    pub timestamps: Vec<i64>,
    pub session_ids: Vec<String>,
    pub agent_roles: Vec<String>,
    pub tool_names: Vec<String>,
    pub durations_ms: Vec<u64>,
    pub tokens_in: Vec<u32>,
    pub tokens_out: Vec<u32>,
    pub costs_microcents: Vec<u64>,
    pub statuses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedStats {
    pub total_records: usize,
    pub total_duration_ms: u64,
    pub avg_duration_ms: f64,
    pub p95_duration_ms: u64,
    pub total_tokens_in: u64,
    pub total_tokens_out: u64,
    pub total_cost_dollars: f64,
    pub success_rate: f64,
    pub tool_frequencies: HashMap<String, usize>,
}

impl ColumnarTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_row(&mut self, row: TelemetryRow) {
        self.timestamps.push(row.timestamp_ms);
        self.session_ids.push(row.session_id);
        self.agent_roles.push(row.agent_role);
        self.tool_names.push(row.tool_name);
        self.durations_ms.push(row.duration_ms);
        self.tokens_in.push(row.tokens_in);
        self.tokens_out.push(row.tokens_out);
        self.costs_microcents.push(row.cost_microcents);
        self.statuses.push(row.status);
    }

    pub fn len(&self) -> usize {
        self.timestamps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.timestamps.is_empty()
    }

    /// Fast analytical summary over columns
    pub fn aggregate(&self) -> AggregatedStats {
        let count = self.len();
        if count == 0 {
            return AggregatedStats {
                total_records: 0,
                total_duration_ms: 0,
                avg_duration_ms: 0.0,
                p95_duration_ms: 0,
                total_tokens_in: 0,
                total_tokens_out: 0,
                total_cost_dollars: 0.0,
                success_rate: 1.0,
                tool_frequencies: HashMap::new(),
            };
        }

        let total_dur: u64 = self.durations_ms.iter().sum();
        let avg_dur = total_dur as f64 / count as f64;

        let mut sorted_dur = self.durations_ms.clone();
        sorted_dur.sort_unstable();
        let p95_idx = ((count as f64) * 0.95).floor() as usize;
        let p95_dur = sorted_dur[p95_idx.min(count - 1)];

        let total_in: u64 = self.tokens_in.iter().map(|&x| x as u64).sum();
        let total_out: u64 = self.tokens_out.iter().map(|&x| x as u64).sum();
        let total_cost_mc: u64 = self.costs_microcents.iter().sum();
        let total_cost_usd = total_cost_mc as f64 / 1_000_000.0;

        let success_count = self.statuses.iter().filter(|&s| s == "success" || s == "executed").count();
        let success_rate = success_count as f64 / count as f64;

        let mut tool_freqs: HashMap<String, usize> = HashMap::new();
        for t in &self.tool_names {
            *tool_freqs.entry(t.clone()).or_insert(0) += 1;
        }

        AggregatedStats {
            total_records: count,
            total_duration_ms: total_dur,
            avg_duration_ms: avg_dur,
            p95_duration_ms: p95_dur,
            total_tokens_in: total_in,
            total_tokens_out: total_out,
            total_cost_dollars: total_cost_usd,
            success_rate,
            tool_frequencies: tool_freqs,
        }
    }
}
