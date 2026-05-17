//! Aggregate performance metrics snapshot

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::{cpu::{CpuStats, CpuLoadCategory}, memory::MemoryStats, gpu::GpuInfo};

/// Full system performance snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfSnapshot {
    pub timestamp: DateTime<Utc>,
    pub cpu: CpuStats,
    pub memory: MemoryStats,
    pub gpus: Vec<GpuInfo>,
    pub process_memory_mb: u64,
}

impl PerfSnapshot {
    /// Collect a full snapshot right now
    pub fn collect() -> Self {
        let cpu    = crate::cpu::collect_cpu_stats();
        let memory = crate::memory::collect_memory_stats();
        let gpus   = crate::gpu::collect_gpu_info();
        let process_memory_mb = current_process_memory_mb();

        PerfSnapshot {
            timestamp: Utc::now(),
            cpu,
            memory,
            gpus,
            process_memory_mb,
        }
    }

    pub fn cpu_load_category(&self) -> CpuLoadCategory {
        CpuLoadCategory::from_usage(self.cpu.global_usage_percent)
    }

    pub fn is_system_healthy(&self) -> bool {
        !self.memory.is_low() &&
        self.cpu.global_usage_percent < 95.0
    }

    pub fn summary(&self) -> String {
        format!(
            "CPU: {:.0}% | RAM: {} | GPU: {} | App: {} MB",
            self.cpu.global_usage_percent,
            self.memory.summary(),
            self.gpus.first().map(|g| g.name.as_str()).unwrap_or("N/A"),
            self.process_memory_mb
        )
    }
}

/// Periodic metrics collector
pub struct MetricsCollector {
    pub history: Vec<PerfSnapshot>,
    pub max_history: usize,
}

impl MetricsCollector {
    pub fn new(max_history: usize) -> Self {
        Self { history: Vec::new(), max_history }
    }

    pub fn tick(&mut self) {
        let snapshot = PerfSnapshot::collect();
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(snapshot);
    }

    pub fn latest(&self) -> Option<&PerfSnapshot> {
        self.history.last()
    }

    pub fn avg_cpu_last_n(&self, n: usize) -> f32 {
        let window: Vec<&PerfSnapshot> = self.history.iter().rev().take(n).collect();
        if window.is_empty() { return 0.0; }
        window.iter().map(|s| s.cpu.global_usage_percent).sum::<f32>() / window.len() as f32
    }
}

/// Get current process memory in MB using sysinfo
fn current_process_memory_mb() -> u64 {
    use sysinfo::{System, Pid, ProcessesToUpdate};
    let mut sys = System::new();
    let pid = Pid::from(std::process::id() as usize);
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]));
    sys.process(pid)
        .map(|p| p.memory() / 1_048_576)
        .unwrap_or(0)
}
