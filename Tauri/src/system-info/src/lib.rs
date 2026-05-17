//! TlantiStudio System Info
//!
//! Detects system capabilities: CPU, RAM, GPU, WebGPU/Vulkan/Metal support.
//! Used to optimize rendering settings and determine available features.

pub mod gpu;
pub mod system;
pub mod ai_workload;

use serde::{Deserialize, Serialize};

/// Complete system information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemReport {
    pub os: OsInfo,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub gpus: Vec<GpuInfo>,
    pub disks: Vec<DiskInfo>,
    pub capabilities: Capabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub arch: String,
    pub hostname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub brand: String,
    pub cores: usize,
    pub frequency_mhz: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: String,
    pub backend: String,      // Vulkan, Metal, DX12, WebGPU
    pub driver_info: String,
    pub device_type: String,  // DiscreteGpu, IntegratedGpu, etc.
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub file_system: String,
}

/// What the system supports for 3D/imaging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    pub webgpu_supported: bool,
    pub vulkan_supported: bool,
    pub metal_supported: bool,
    pub dx12_supported: bool,
    pub max_texture_size: u32,
    pub max_buffer_size: u64,
    pub recommended_render_quality: RenderQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderQuality {
    Low,
    Medium,
    High,
    Ultra,
}
