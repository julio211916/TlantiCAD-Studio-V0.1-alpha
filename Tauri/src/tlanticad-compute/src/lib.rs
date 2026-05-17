//! TlantiCAD Compute — unified backend trait for CPU + GPU + NPU.
//!
//! MP-101 → MP-110. Closes Bloque J of the master plan.
//!
//! Architecture:
//!
//!   * [`ComputeBackend`] — sealed trait every backend implements (CPU, wgpu, CUDA, Metal,
//!     CoreML, DirectML, ROCm). Each op is async-friendly but defaults to sync.
//!   * [`ComputeKind`]    — enum tag identifying the running backend (UI status bar).
//!   * [`BackendCapabilities`] — what the backend can run (ops + max mesh size + precision).
//!   * [`ComputeRouter`]  — picks the best available backend per op via [`BenchProfile`].
//!
//! Backends gated behind cargo features so a clean checkout builds with only the CPU path:
//!
//!   * `cpu-rayon`  (default) — multicore via rayon.
//!   * `gpu-wgpu`   — wgpu cross-vendor (NVIDIA + AMD + Intel + Apple) via WGSL.
//!   * `gpu-cuda`, `gpu-metal`, `ane-coreml`, `trt-tensorrt`, `rocm-amd`, `directml-npu` —
//!     real backends that require external runtimes; the placeholders return a typed
//!     `Unsupported` error until the runtime is wired.

pub mod backend;
pub mod boundary;
pub mod cpu;
pub mod profile;
pub mod router;

#[cfg(feature = "gpu-wgpu")]
pub mod wgpu_backend;

pub use backend::{
    BackendCapabilities, ComputeBackend, ComputeError, ComputeKind, ComputeOp, ComputeStats,
};
pub use boundary::{
    BoundaryBenchProfile, BoundaryBenchRequest, BoundaryBenchResult, BoundaryOwner,
    BoundaryRecommendation,
};
pub use profile::{BenchProfile, BenchResult, EnergyMode};
pub use router::{ComputeRouter, RouterDecision, RouterError};
