//! AI Workload & System Capability Analysis
//!
//! Analyzes system capabilities for dental AI tasks:
//! - GPU/CUDA/Metal availability for ML inference
//! - Memory sufficiency for DICOM/CBCT processing
//! - Disk space for PACS storage
//! - Network bandwidth estimation
//! - Recommended optimizations

use serde::{Deserialize, Serialize};

/// AI workload capability report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiWorkloadReport {
    pub gpu_compute: GpuComputeCapability,
    pub memory_assessment: MemoryAssessment,
    pub storage_assessment: StorageAssessment,
    pub recommended_settings: RecommendedSettings,
    pub workload_scores: WorkloadScores,
}

/// GPU compute capabilities for ML/AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuComputeCapability {
    pub has_discrete_gpu: bool,
    pub gpu_name: String,
    pub backend: String,
    pub compute_shader_support: bool,
    pub estimated_vram_mb: u64,
    pub suitable_for_inference: bool,
    pub suitable_for_training: bool,
    pub cuda_available: bool,
    pub metal_available: bool,
    pub vulkan_compute_available: bool,
}

/// Memory analysis for imaging workloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAssessment {
    pub total_ram_gb: f64,
    pub available_ram_gb: f64,
    pub can_load_cbct: bool,
    pub can_load_panoramic: bool,
    pub max_concurrent_dicom_views: u32,
    pub recommended_cache_mb: u64,
}

/// Storage analysis for PACS/imaging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageAssessment {
    pub data_disk_total_gb: f64,
    pub data_disk_available_gb: f64,
    pub estimated_pacs_capacity_studies: u64,
    pub ssd_detected: bool,
    pub write_speed_estimate: String,
}

/// Recommended application settings based on system capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedSettings {
    pub render_quality: String,
    pub max_texture_size: u32,
    pub dicom_cache_mb: u64,
    pub enable_gpu_inference: bool,
    pub enable_3d_viewer: bool,
    pub enable_cbct_viewer: bool,
    pub max_concurrent_imports: u32,
    pub thumbnail_quality: String,
}

/// Composite scores (0-100) for different workload types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadScores {
    pub overall: u32,
    pub dicom_viewing: u32,
    pub cbct_3d: u32,
    pub ai_inference: u32,
    pub pacs_storage: u32,
    pub mesh_generation: u32,
}

/// Analyze AI workload capabilities from system info
pub fn analyze_ai_workload(
    gpus: &[crate::GpuInfo],
    memory: &crate::MemoryInfo,
    disks: &[crate::DiskInfo],
    capabilities: &crate::Capabilities,
) -> AiWorkloadReport {
    let gpu_compute = analyze_gpu_compute(gpus, capabilities);
    let memory_assessment = analyze_memory(memory);
    let storage_assessment = analyze_storage(disks);
    let workload_scores = compute_scores(&gpu_compute, &memory_assessment, &storage_assessment);
    let recommended_settings =
        compute_recommendations(&gpu_compute, &memory_assessment, &storage_assessment, capabilities);

    AiWorkloadReport {
        gpu_compute,
        memory_assessment,
        storage_assessment,
        recommended_settings,
        workload_scores,
    }
}

fn analyze_gpu_compute(
    gpus: &[crate::GpuInfo],
    capabilities: &crate::Capabilities,
) -> GpuComputeCapability {
    let best_gpu = gpus.iter().find(|g| g.device_type == "DiscreteGpu")
        .or(gpus.first());

    let (name, vendor, backend, _device_type) = match best_gpu {
        Some(g) => (g.name.clone(), g.vendor.clone(), g.backend.clone(), g.device_type.clone()),
        None => ("None".into(), "Unknown".into(), "None".into(), "Unknown".into()),
    };

    let has_discrete = gpus.iter().any(|g| g.device_type == "DiscreteGpu");
    let metal = capabilities.metal_supported;
    let vulkan = capabilities.vulkan_supported;

    // Estimate VRAM based on GPU name heuristics
    let estimated_vram = estimate_vram(&name, has_discrete);

    GpuComputeCapability {
        has_discrete_gpu: has_discrete,
        gpu_name: name,
        backend,
        compute_shader_support: vulkan || metal,
        estimated_vram_mb: estimated_vram,
        suitable_for_inference: estimated_vram >= 2048 || metal,
        suitable_for_training: estimated_vram >= 8192,
        cuda_available: vendor.to_lowercase().contains("nvidia") && vulkan,
        metal_available: metal,
        vulkan_compute_available: vulkan,
    }
}

fn analyze_memory(memory: &crate::MemoryInfo) -> MemoryAssessment {
    let total_gb = memory.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    let available_gb = memory.available_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    // CBCT needs ~2-4 GB, panoramic ~200MB
    let can_cbct = available_gb > 4.0;
    let can_panoramic = available_gb > 0.5;

    // Each DICOM view uses ~50-200MB
    let max_views = (available_gb * 1024.0 / 200.0).min(20.0) as u32;

    // Cache recommendation: 10% of available RAM, max 2GB
    let cache_mb = ((available_gb * 0.10) * 1024.0).min(2048.0) as u64;

    MemoryAssessment {
        total_ram_gb: total_gb,
        available_ram_gb: available_gb,
        can_load_cbct: can_cbct,
        can_load_panoramic: can_panoramic,
        max_concurrent_dicom_views: max_views,
        recommended_cache_mb: cache_mb,
    }
}

fn analyze_storage(disks: &[crate::DiskInfo]) -> StorageAssessment {
    // Find the largest disk (likely data disk)
    let data_disk = disks.iter().max_by_key(|d| d.total_bytes);

    let (total_gb, available_gb, fs) = match data_disk {
        Some(d) => (
            d.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
            d.available_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
            d.file_system.clone(),
        ),
        None => (0.0, 0.0, "unknown".into()),
    };

    // Average dental study ~50MB (panoramic) to ~500MB (CBCT)
    let avg_study_mb = 100.0;
    let capacity = (available_gb * 1024.0 / avg_study_mb) as u64;

    let ssd = fs.to_lowercase().contains("apfs")
        || fs.to_lowercase().contains("ntfs")
        || fs.to_lowercase().contains("ext4");

    StorageAssessment {
        data_disk_total_gb: total_gb,
        data_disk_available_gb: available_gb,
        estimated_pacs_capacity_studies: capacity,
        ssd_detected: ssd,
        write_speed_estimate: if ssd { "Fast (SSD)".into() } else { "Standard (HDD)".into() },
    }
}

fn compute_scores(
    gpu: &GpuComputeCapability,
    mem: &MemoryAssessment,
    storage: &StorageAssessment,
) -> WorkloadScores {
    // DICOM viewing: mostly CPU + RAM
    let dicom = score_clamp(
        (mem.total_ram_gb * 8.0) as u32 + if gpu.compute_shader_support { 20 } else { 0 },
    );

    // CBCT 3D: needs GPU + RAM
    let cbct = score_clamp(
        (if gpu.has_discrete_gpu { 40 } else { 10 })
            + (mem.total_ram_gb * 5.0) as u32
            + if gpu.vulkan_compute_available || gpu.metal_available { 20 } else { 0 },
    );

    // AI inference: GPU critical
    let ai = score_clamp(
        (if gpu.suitable_for_inference { 50 } else { 10 })
            + (gpu.estimated_vram_mb / 100) as u32
            + (mem.total_ram_gb * 3.0) as u32,
    );

    // PACS storage: disk space
    let pacs = score_clamp(
        (storage.data_disk_available_gb * 0.5) as u32
            + if storage.ssd_detected { 20 } else { 0 },
    );

    // Mesh generation: CPU + RAM + GPU
    let mesh = score_clamp(
        (mem.total_ram_gb * 5.0) as u32
            + if gpu.compute_shader_support { 30 } else { 0 }
            + if gpu.has_discrete_gpu { 20 } else { 0 },
    );

    let overall = (dicom + cbct + ai + pacs + mesh) / 5;

    WorkloadScores {
        overall,
        dicom_viewing: dicom,
        cbct_3d: cbct,
        ai_inference: ai,
        pacs_storage: pacs,
        mesh_generation: mesh,
    }
}

fn compute_recommendations(
    gpu: &GpuComputeCapability,
    mem: &MemoryAssessment,
    _storage: &StorageAssessment,
    capabilities: &crate::Capabilities,
) -> RecommendedSettings {
    let quality = if gpu.has_discrete_gpu && mem.total_ram_gb > 16.0 {
        "ultra"
    } else if gpu.compute_shader_support && mem.total_ram_gb > 8.0 {
        "high"
    } else if mem.total_ram_gb > 4.0 {
        "medium"
    } else {
        "low"
    };

    let max_tex = if gpu.has_discrete_gpu {
        capabilities.max_texture_size.min(8192)
    } else {
        capabilities.max_texture_size.min(4096)
    };

    RecommendedSettings {
        render_quality: quality.into(),
        max_texture_size: max_tex,
        dicom_cache_mb: mem.recommended_cache_mb,
        enable_gpu_inference: gpu.suitable_for_inference,
        enable_3d_viewer: gpu.compute_shader_support || capabilities.webgpu_supported,
        enable_cbct_viewer: mem.can_load_cbct && gpu.compute_shader_support,
        max_concurrent_imports: if mem.total_ram_gb > 8.0 { 4 } else { 2 },
        thumbnail_quality: if mem.total_ram_gb > 8.0 { "high".into() } else { "medium".into() },
    }
}

fn estimate_vram(gpu_name: &str, has_discrete: bool) -> u64 {
    let name = gpu_name.to_lowercase();
    if name.contains("4090") { return 24576; }
    if name.contains("4080") { return 16384; }
    if name.contains("4070") { return 12288; }
    if name.contains("4060") { return 8192; }
    if name.contains("3090") { return 24576; }
    if name.contains("3080") { return 10240; }
    if name.contains("3070") { return 8192; }
    if name.contains("3060") { return 12288; }
    if name.contains("m1") || name.contains("m2") || name.contains("m3") || name.contains("m4") {
        return 16384; // Apple Silicon shared memory
    }
    if has_discrete { 4096 } else { 2048 }
}

fn score_clamp(val: u32) -> u32 {
    val.min(100)
}
