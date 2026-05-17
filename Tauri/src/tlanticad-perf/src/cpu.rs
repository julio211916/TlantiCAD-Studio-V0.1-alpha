//! CPU performance monitoring

use serde::{Deserialize, Serialize};
use sysinfo::System;

/// CPU core metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCoreInfo {
    pub index: usize,
    pub usage_percent: f32,
    pub frequency_mhz: u64,
    pub brand: String,
}

/// Overall CPU stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuStats {
    pub cores: Vec<CpuCoreInfo>,
    pub global_usage_percent: f32,
    pub physical_core_count: usize,
    pub logical_core_count: usize,
    pub cpu_arch: String,
    pub brand: String,
}

/// Collect current CPU stats
pub fn collect_cpu_stats() -> CpuStats {
    let mut sys = System::new_all();
    sys.refresh_all();
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_cpu_all();

    let cpus = sys.cpus();
    let global_usage = sys.global_cpu_usage();

    let brand = cpus.first().map(|c| c.brand().to_string()).unwrap_or_default();
    let cores: Vec<CpuCoreInfo> = cpus.iter().enumerate().map(|(i, cpu)| {
        CpuCoreInfo {
            index: i,
            usage_percent: cpu.cpu_usage(),
            frequency_mhz: cpu.frequency(),
            brand: cpu.brand().to_string(),
        }
    }).collect();

    CpuStats {
        global_usage_percent: global_usage,
        logical_core_count: cores.len(),
        physical_core_count: sys.physical_core_count().unwrap_or(cores.len()),
        cpu_arch: std::env::consts::ARCH.to_string(),
        brand,
        cores,
    }
}

/// CPU load category for clinical workstation assessment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CpuLoadCategory {
    Idle,
    Low,
    Moderate,
    High,
    Critical,
}

impl CpuLoadCategory {
    pub fn from_usage(usage: f32) -> Self {
        match usage as u32 {
            0..=20  => CpuLoadCategory::Idle,
            21..=50 => CpuLoadCategory::Low,
            51..=70 => CpuLoadCategory::Moderate,
            71..=90 => CpuLoadCategory::High,
            _       => CpuLoadCategory::Critical,
        }
    }
}
