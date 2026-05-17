//! Crown fast-path AI inference — kernel selector + ONNX bridge stub. AR-V370.
//!
//! Ports the inference-side contract from `DentalServices/CloudBackendCrownGenerationProcessor`
//! (we strip the cloud bits — local-first only).
//!
//! Status: this module exposes the **selector + dispatcher**. The ONNX model bytes are
//! downloaded by the V313-V322 model registry; the actual `tract-onnx` invocation lives behind
//! the existing `backend-ml` cargo feature flag in the Tauri app and is gated to avoid
//! pulling 200 MB of ONNX weights into a clean checkout.
//!
//! When `tract-onnx` is wired (V370 follow-up), we replace `infer_with_kernel` with the real
//! `tract::SimpleState` invocation. Until then this module returns a typed
//! `KernelStatus::ModelMissing` so the caller (CrownPipeline) falls back to the deterministic
//! Bezier+offset path.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum InferenceKernel {
    /// Pure CPU implementation — slowest but always available.
    Cpu,
    /// Apple Silicon Metal Performance Shaders.
    Mps,
    /// NVIDIA CUDA. Requires the `backend-ml` feature + a system CUDA install.
    Cuda,
    /// Force the deterministic geometric path even if a model is present.
    GeometricFallback,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum KernelAvailability {
    Available,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelInfo {
    pub kernel: InferenceKernel,
    pub availability: KernelAvailability,
    pub display_name: &'static str,
    pub backend_id: &'static str,
}

/// Detect which inference kernels are runnable on this machine.
///
/// `Cpu` is always available. `Mps`/`Cuda`/`GeometricFallback` depend on cargo features.
pub fn detect_available_kernels() -> Vec<KernelInfo> {
    let mut out = vec![KernelInfo {
        kernel: InferenceKernel::Cpu,
        availability: KernelAvailability::Available,
        display_name: "CPU (tract-onnx)",
        backend_id: "tract-cpu",
    }];

    // Apple Silicon — runtime test would dlopen Metal; here we surface the static
    // build-time flag so the UI can grey-out unavailable kernels.
    let mps_available = cfg!(target_os = "macos") && cfg!(target_arch = "aarch64");
    out.push(KernelInfo {
        kernel: InferenceKernel::Mps,
        availability: if mps_available {
            KernelAvailability::Available
        } else {
            KernelAvailability::Unavailable
        },
        display_name: "Apple MPS",
        backend_id: "ane-mps",
    });

    // CUDA — gated by feature; when not built with it, mark unavailable.
    let cuda_available = cfg!(feature = "cuda");
    out.push(KernelInfo {
        kernel: InferenceKernel::Cuda,
        availability: if cuda_available {
            KernelAvailability::Available
        } else {
            KernelAvailability::Unavailable
        },
        display_name: "NVIDIA CUDA",
        backend_id: "tract-cuda",
    });

    out.push(KernelInfo {
        kernel: InferenceKernel::GeometricFallback,
        availability: KernelAvailability::Available,
        display_name: "Geometric (no model)",
        backend_id: "geometric",
    });

    out
}

/// Pick the best kernel out of `requested`. If `requested` is empty, default to CPU.
pub fn pick_kernel(requested: &[InferenceKernel]) -> KernelInfo {
    let available = detect_available_kernels();
    for k in requested {
        if let Some(info) = available
            .iter()
            .find(|i| i.kernel == *k && matches!(i.availability, KernelAvailability::Available))
        {
            return info.clone();
        }
    }
    // Fall back to CPU which is always available.
    available
        .into_iter()
        .find(|i| i.kernel == InferenceKernel::Cpu)
        .expect("CPU kernel always present")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    /// Desired kernel order; first available is used.
    pub preferred_kernels: Vec<InferenceKernel>,
    /// Path to the ONNX model bytes (resolved by the caller via the V313-V322 registry).
    pub model_sha256: String,
    /// Per-tooth FDI input — used by the real model to condition the output.
    pub fdi: u8,
    /// Material name — drives default constraint bounds the geometric fallback uses.
    pub material: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KernelStatus {
    Ok,
    ModelMissing,
    KernelUnavailable { reason: String },
    Fallback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub kernel: KernelInfo,
    pub status: KernelStatus,
    pub backend: &'static str,
}

/// Dispatch a crown inference request. Returns `KernelStatus::ModelMissing` so the caller
/// can immediately switch to the deterministic geometric path until ONNX wiring lands.
pub fn dispatch(request: &InferenceRequest) -> InferenceResult {
    let kernel = pick_kernel(&request.preferred_kernels);
    if matches!(kernel.kernel, InferenceKernel::GeometricFallback) {
        return InferenceResult {
            kernel,
            status: KernelStatus::Fallback,
            backend: "tlanticad-ai::crown_inference::geometric",
        };
    }
    if request.model_sha256.is_empty() {
        return InferenceResult {
            kernel,
            status: KernelStatus::ModelMissing,
            backend: "tlanticad-ai::crown_inference::dispatcher",
        };
    }
    // Real ONNX call would land here behind the `backend-ml` feature.
    InferenceResult {
        kernel,
        status: KernelStatus::KernelUnavailable {
            reason: format!(
                "ONNX runtime not yet linked for {}; supply `backend-ml` feature or use GeometricFallback",
                request.material
            ),
        },
        backend: "tlanticad-ai::crown_inference::dispatcher",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_kernels_always_includes_cpu() {
        let kernels = detect_available_kernels();
        assert!(kernels
            .iter()
            .any(|k| k.kernel == InferenceKernel::Cpu
                && matches!(k.availability, KernelAvailability::Available)));
    }

    #[test]
    fn pick_kernel_falls_back_to_cpu_when_unavailable_requested() {
        let pick = pick_kernel(&[InferenceKernel::Cuda]);
        // On a machine without CUDA built, this must fall back to CPU.
        if !cfg!(feature = "cuda") {
            assert_eq!(pick.kernel, InferenceKernel::Cpu);
        }
    }

    #[test]
    fn dispatch_with_no_model_returns_model_missing() {
        let request = InferenceRequest {
            preferred_kernels: vec![InferenceKernel::Cpu],
            model_sha256: String::new(),
            fdi: 16,
            material: "zirconia".into(),
        };
        let result = dispatch(&request);
        assert!(matches!(result.status, KernelStatus::ModelMissing));
    }

    #[test]
    fn dispatch_with_geometric_fallback_returns_fallback() {
        let request = InferenceRequest {
            preferred_kernels: vec![InferenceKernel::GeometricFallback],
            model_sha256: "abc".into(),
            fdi: 11,
            material: "metal".into(),
        };
        let result = dispatch(&request);
        assert!(matches!(result.status, KernelStatus::Fallback));
    }
}
