//! System information using the `sysinfo` crate
//!
//! Gathers CPU, RAM, disk, and OS information.

use crate::{CpuInfo, DiskInfo, MemoryInfo, OsInfo};
use sysinfo::System;

/// Gather OS information
pub fn get_os_info() -> OsInfo {
    OsInfo {
        name: System::name().unwrap_or_else(|| "Unknown".into()),
        version: System::os_version().unwrap_or_else(|| "Unknown".into()),
        arch: std::env::consts::ARCH.to_string(),
        hostname: System::host_name().unwrap_or_else(|| "Unknown".into()),
    }
}

/// Gather CPU information
pub fn get_cpu_info() -> CpuInfo {
    let mut sys = System::new_all();
    sys.refresh_cpu_all();

    let cpus = sys.cpus();
    let brand = cpus.first().map(|c| c.brand().to_string()).unwrap_or_default();
    let freq = cpus.first().map(|c| c.frequency()).unwrap_or(0);
    let usage: f32 = if !cpus.is_empty() {
        cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32
    } else {
        0.0
    };

    CpuInfo {
        brand,
        cores: cpus.len(),
        frequency_mhz: freq,
        usage_percent: usage,
    }
}

/// Gather memory information
pub fn get_memory_info() -> MemoryInfo {
    let mut sys = System::new_all();
    sys.refresh_memory();

    let total = sys.total_memory();
    let used = sys.used_memory();
    let available = sys.available_memory();
    let usage = if total > 0 {
        (used as f32 / total as f32) * 100.0
    } else {
        0.0
    };

    MemoryInfo {
        total_bytes: total,
        used_bytes: used,
        available_bytes: available,
        usage_percent: usage,
    }
}

/// Gather disk information
pub fn get_disk_info() -> Vec<DiskInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();

    sysinfo::Disks::new_with_refreshed_list()
        .iter()
        .map(|d| DiskInfo {
            name: d.name().to_string_lossy().to_string(),
            mount_point: d.mount_point().to_string_lossy().to_string(),
            total_bytes: d.total_space(),
            available_bytes: d.available_space(),
            file_system: d.file_system().to_string_lossy().to_string(),
        })
        .collect()
}

/// Gather complete system report (without GPU — use gpu module for that)
pub fn get_system_report_base() -> (OsInfo, CpuInfo, MemoryInfo, Vec<DiskInfo>) {
    (get_os_info(), get_cpu_info(), get_memory_info(), get_disk_info())
}
