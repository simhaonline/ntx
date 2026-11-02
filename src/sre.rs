//! Autonomous Site Reliability Engineering (SRE) Incident Triage & Runbook Engine
//! Inspired by Tracer-Cloud/opensre

#[derive(Debug)]
pub struct IncidentReport {
    pub anomaly: String,
    pub severity: &'static str,
    pub root_cause: String,
    pub automated_runbook: &'static str,
}

pub struct SreEngine;

impl SreEngine {
    pub fn triage(log_snippet: &str) -> IncidentReport {
        let (severity, cause, runbook) = if log_snippet.contains("OOM") || log_snippet.contains("out of memory") {
            ("P0", "Kernel Out-Of-Memory Killer triggered by unbounded cache", "ntx storage gc --force")
        } else if log_snippet.contains("connection refused") || log_snippet.contains("502") {
            ("P1", "Target service daemon unreachable or crashing", "systemctl restart nityaksha-webui.service")
        } else if log_snippet.contains("disk full") || log_snippet.contains("no space") {
            ("P0", "Storage capacity exhausted", "ntx storage gc && clean_journal")
        } else {
            ("P3", "Transient anomaly or unclassified warning", "ntx doctor --all")
        };

        IncidentReport {
            anomaly: log_snippet.to_string(),
            severity,
            root_cause: cause.to_string(),
            automated_runbook: runbook,
        }
    }
}
