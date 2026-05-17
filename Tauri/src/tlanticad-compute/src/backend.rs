//! Backend trait + capability + op enums. The single sealed contract every accelerator
//! implements. MP-101.

use serde::{Deserialize, Serialize};

/// Tag identifying which compute backend is running.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum ComputeKind {
    /// Multi-core CPU via rayon. Always available.
    Cpu,
    /// SIMD-only fallback (no rayon). Used on minimal targets.
    CpuSimd,
    /// `wgpu` cross-vendor compute (NVIDIA + AMD + Intel + Apple). Cargo `gpu-wgpu`.
    Wgpu,
    /// NVIDIA CUDA + cuDNN. Cargo `gpu-cuda`.
    NvidiaCuda,
    /// Apple Metal MPS Graph (M1-M4). Cargo `gpu-metal`.
    AppleMetal,
    /// Apple Neural Engine via CoreML. Cargo `ane-coreml`.
    AppleAne,
    /// NVIDIA TensorRT (FP16 + INT8). Cargo `trt-tensorrt`.
    NvidiaTensorrt,
    /// AMD ROCm (Linux). Cargo `rocm-amd`.
    AmdRocm,
    /// AMD Ryzen XDNA NPU + Intel AI Boost via DirectML (Windows). Cargo `directml-npu`.
    DirectmlNpu,
}

impl ComputeKind {
    /// Human-readable label for the status bar (e.g. "Apple ANE · 38 TOPS").
    pub fn display_name(&self) -> &'static str {
        match self {
            ComputeKind::Cpu => "CPU rayon",
            ComputeKind::CpuSimd => "CPU SIMD",
            ComputeKind::Wgpu => "GPU wgpu",
            ComputeKind::NvidiaCuda => "NVIDIA CUDA",
            ComputeKind::AppleMetal => "Apple Metal",
            ComputeKind::AppleAne => "Apple ANE",
            ComputeKind::NvidiaTensorrt => "NVIDIA TensorRT",
            ComputeKind::AmdRocm => "AMD ROCm",
            ComputeKind::DirectmlNpu => "DirectML NPU",
        }
    }

    /// Stable id for telemetry / persistence.
    pub fn id(&self) -> &'static str {
        match self {
            ComputeKind::Cpu => "cpu-rayon",
            ComputeKind::CpuSimd => "cpu-simd",
            ComputeKind::Wgpu => "gpu-wgpu",
            ComputeKind::NvidiaCuda => "gpu-cuda",
            ComputeKind::AppleMetal => "gpu-metal",
            ComputeKind::AppleAne => "ane-coreml",
            ComputeKind::NvidiaTensorrt => "trt-tensorrt",
            ComputeKind::AmdRocm => "rocm-amd",
            ComputeKind::DirectmlNpu => "directml-npu",
        }
    }
}

/// Operations a backend can advertise + dispatch.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum ComputeOp {
    /// Per-vertex distance from a source mesh to a target mesh (Hausdorff field).
    PerVertexDistance,
    /// Region grow on a mesh face graph from a seed face.
    RegionGrow,
    /// Cotangent-Laplacian smoothing pass (CSurf).
    LaplacianSmooth,
    /// Voxel marching-cubes lite (mask → triangles).
    MarchingCubes,
    /// Voxel region-grow 3D.
    VoxelRegionGrow,
    /// Mesh decimation (edge collapse).
    MeshDecimate,
    /// AI inference — ONNX model run.
    OnnxInference,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BackendCapabilities {
    pub kind: ComputeKind,
    pub max_vertices: u64,
    pub supports_fp16: bool,
    pub supports_int8: bool,
    /// Theoretical peak TOPS (tera ops/sec) for AI ops; 0 when irrelevant.
    pub peak_tops: f32,
    /// Whether this backend should be considered when on battery.
    pub low_power_friendly: bool,
}

impl BackendCapabilities {
    pub fn cpu() -> Self {
        Self {
            kind: ComputeKind::Cpu,
            max_vertices: u64::MAX,
            supports_fp16: false,
            supports_int8: true,
            peak_tops: 0.0,
            low_power_friendly: true,
        }
    }
}

/// Per-op statistics returned alongside a backend invocation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComputeStats {
    pub backend: Option<ComputeKind>,
    pub op: Option<ComputeOp>,
    pub elapsed_ms: u64,
    pub items_processed: u64,
    pub fell_back_to_cpu: bool,
}

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ComputeError {
    #[error("backend {backend:?} is not supported in this build (cargo feature missing)")]
    Unsupported { backend: ComputeKind },
    #[error("backend {backend:?} reported out-of-memory; fall back to CPU")]
    OutOfMemory { backend: ComputeKind },
    #[error("op {op:?} not supported by backend {backend:?}")]
    OpUnsupported {
        op: ComputeOp,
        backend: ComputeKind,
    },
    #[error("input shape mismatch: {message}")]
    Shape { message: String },
    #[error("backend kernel error: {message}")]
    Kernel { message: String },
}

/// Sealed trait — every backend exposes the same async-friendly contract. Implementors live
/// in `cpu.rs`, `wgpu_backend.rs`, etc.
pub trait ComputeBackend: Send + Sync {
    fn capabilities(&self) -> BackendCapabilities;

    /// Per-vertex distance from `src` to `dst`. Returns one f64 per vertex of `src`.
    fn per_vertex_distance(
        &self,
        src: &tlanticad_mesh::Mesh,
        dst: &tlanticad_mesh::Mesh,
    ) -> Result<(Vec<f64>, ComputeStats), ComputeError> {
        let _ = (src, dst);
        Err(ComputeError::OpUnsupported {
            op: ComputeOp::PerVertexDistance,
            backend: self.capabilities().kind,
        })
    }

    /// Apply Laplacian smoothing in-place. Returns moved-vertex count + max displacement.
    fn laplacian_smooth(
        &self,
        mesh: &mut tlanticad_mesh::Mesh,
        iterations: u32,
        lambda: f64,
    ) -> Result<ComputeStats, ComputeError> {
        let _ = (mesh, iterations, lambda);
        Err(ComputeError::OpUnsupported {
            op: ComputeOp::LaplacianSmooth,
            backend: self.capabilities().kind,
        })
    }

    /// Optional: report whether this backend is currently runnable (e.g. driver present,
    /// device powered on). Default is true — backends that may be absent override this.
    fn is_runnable(&self) -> bool {
        true
    }
}
