//! Nityaksha cgroup v2 Resource Governance (Inspired by ContainerOS ADR-18)
//! Memory ceiling, process/thread fork-bomb guard, CPU quotas, and freeze/thaw.

use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CgroupLimits {
    pub memory_max_bytes: u64, // e.g. 512MB = 536870912
    pub pids_max: u64,          // e.g. 128
    pub cpu_quota_pct: u32,     // e.g. 100% = 100, 50% = 50
}

impl Default for CgroupLimits {
    fn default() -> Self {
        Self {
            memory_max_bytes: 512 * 1024 * 1024, // 512 MB
            pids_max: 256,                       // Fork bomb guard
            cpu_quota_pct: 100,
        }
    }
}

pub struct CgroupManager {
    base_dir: PathBuf,
}

impl CgroupManager {
    pub const DEFAULT_ROOT: &'static str = "/sys/fs/cgroup/nityaksha";

    pub fn new() -> io::Result<Self> {
        let path = PathBuf::from(Self::DEFAULT_ROOT);
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }
        Ok(Self { base_dir: path })
    }

    /// Provision cgroup subtree for a session and apply limits
    pub fn provision_session(&self, session_id: &str, limits: &CgroupLimits) -> io::Result<PathBuf> {
        let session_cg = self.base_dir.join(session_id);
        let _ = fs::create_dir_all(&session_cg);

        if session_cg.exists() {
            // Apply memory.max
            let mem_file = session_cg.join("memory.max");
            let _ = fs::write(mem_file, limits.memory_max_bytes.to_string());

            // Apply pids.max
            let pids_file = session_cg.join("pids.max");
            let _ = fs::write(pids_file, limits.pids_max.to_string());

            // Apply cpu.max (quota, period)
            let cpu_file = session_cg.join("cpu.max");
            let period = 100_000u64;
            let quota = (period * limits.cpu_quota_pct as u64) / 100;
            let _ = fs::write(cpu_file, format!("{} {}", quota, period));
        }

        Ok(session_cg)
    }

    /// Attach a process PID to the session cgroup
    pub fn attach_pid(&self, session_id: &str, pid: u32) -> io::Result<()> {
        let session_cg = self.base_dir.join(session_id);
        if session_cg.exists() {
            let procs_file = session_cg.join("cgroup.procs");
            let _ = fs::write(procs_file, pid.to_string());
        }
        Ok(())
    }

    /// Freeze or thaw session processes
    pub fn set_frozen(&self, session_id: &str, freeze: bool) -> io::Result<()> {
        let freeze_file = self.base_dir.join(session_id).join("cgroup.freeze");
        if freeze_file.exists() {
            fs::write(freeze_file, if freeze { "1" } else { "0" })?;
        }
        Ok(())
    }

    /// Clean up session cgroup on termination
    pub fn cleanup_session(&self, session_id: &str) {
        let session_cg = self.base_dir.join(session_id);
        if session_cg.exists() {
            let _ = fs::remove_dir(session_cg);
        }
    }
}
