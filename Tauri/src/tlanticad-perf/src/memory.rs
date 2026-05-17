//! Memory usage monitoring

use serde::{Deserialize, Serialize};
use sysinfo::System;

/// System RAM stats in bytes and human-readable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f32,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
}

impl MemoryStats {
    pub fn total_gb(&self) -> f32 {
        self.total_bytes as f32 / 1_073_741_824.0
    }

    pub fn used_gb(&self) -> f32 {
        self.used_bytes as f32 / 1_073_741_824.0
    }

    pub fn available_gb(&self) -> f32 {
        self.available_bytes as f32 / 1_073_741_824.0
    }

    pub fn is_low(&self) -> bool {
        self.usage_percent > 85.0
    }

    pub fn summary(&self) -> String {
        format!(
            "{:.1} GB / {:.1} GB ({:.0}%)",
            self.used_gb(), self.total_gb(), self.usage_percent
        )
    }
}

/// Collect current memory stats
pub fn collect_memory_stats() -> MemoryStats {
    let mut sys = System::new_all();
    sys.refresh_memory();

    let total = sys.total_memory();
    let used  = sys.used_memory();
    let avail = sys.available_memory();
    let usage = if total > 0 { used as f32 / total as f32 * 100.0 } else { 0.0 };

    MemoryStats {
        total_bytes: total,
        used_bytes: used,
        available_bytes: avail,
        usage_percent: usage,
        swap_total_bytes: sys.total_swap(),
        swap_used_bytes: sys.used_swap(),
    }
}

/// Recommended minimum RAM for different TlantiCAD features
pub fn required_ram_gb(feature: &str) -> f32 {
    match feature {
        "cbct_viewer"      => 8.0,
        "mesh_editor"      => 4.0,
        "ml_inference"     => 8.0,
        "full_workflow"    => 16.0,
        _                  => 2.0,
    }
}
