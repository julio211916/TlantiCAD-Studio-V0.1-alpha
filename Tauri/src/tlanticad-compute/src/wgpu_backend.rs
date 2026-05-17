//! wgpu backend skeleton — MP-103.
//!
//! Cross-vendor GPU compute (NVIDIA + AMD + Intel + Apple) via WGSL. This is the
//! **skeleton** sprint: we own the device, advertise capabilities, and load the
//! `per_vertex_distance.wgsl` shader stub. The real KD-tree closest-point kernel
//! lands in **MP-103.b**; until then `per_vertex_distance` and `laplacian_smooth`
//! return `OpUnsupported` so the [`crate::ComputeRouter`] can fall back to CPU.
//!
//! The constructor [`WgpuBackend::try_new`] returns `None` when no adapter is
//! available (CI without a GPU, headless Linux, etc.) — callers should treat that
//! as "feature compiled but device absent" and skip the backend.
//!
//! Embedded WGSL source kept inline so the crate works without `include_dir`.

use crate::backend::{
    BackendCapabilities, ComputeBackend, ComputeError, ComputeOp, ComputeStats,
};
use crate::ComputeKind;
use std::time::Instant;
use tlanticad_mesh::Mesh;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct DistanceParams {
    src_count: u32,
    dst_count: u32,
    _pad0: u32,
    _pad1: u32,
}

/// Embedded WGSL source for the per-vertex distance kernel (MP-103.b stub).
const PER_VERTEX_DISTANCE_WGSL: &str = include_str!("shaders/per_vertex_distance.wgsl");

/// wgpu backend. Owns a `Device` + `Queue` and a precompiled shader module for
/// the per-vertex distance kernel.
pub struct WgpuBackend {
    #[allow(dead_code)]
    instance: wgpu::Instance,
    #[allow(dead_code)]
    adapter: wgpu::Adapter,
    #[allow(dead_code)]
    device: wgpu::Device,
    #[allow(dead_code)]
    queue: wgpu::Queue,
    #[allow(dead_code)]
    per_vertex_distance_shader: wgpu::ShaderModule,
}

impl WgpuBackend {
    /// Try to construct a wgpu backend. Returns `None` when no adapter is
    /// available (no GPU driver, headless CI, virtualised env without paravirt
    /// GPU, …). Never panics.
    pub fn try_new() -> Option<Self> {
        let instance = wgpu::Instance::default();

        // wgpu 22: enumerate_adapters returns Vec<Adapter> directly.
        let adapters: Vec<wgpu::Adapter> = instance
            .enumerate_adapters(wgpu::Backends::all())
            .into_iter()
            .collect();
        if adapters.is_empty() {
            return None;
        }

        // Pick the first adapter — MP-103.c will add a "best adapter" picker
        // (discrete > integrated > virtual > cpu) keyed off `AdapterInfo`.
        let adapter = adapters.into_iter().next()?;

        let (device, queue) = pollster::block_on(async {
            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("tlanticad-compute.wgpu.device"),
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::downlevel_defaults(),
                        memory_hints: wgpu::MemoryHints::default(),
                    },
                    None,
                )
                .await
        })
        .ok()?;

        let per_vertex_distance_shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("tlanticad-compute.wgpu.per_vertex_distance"),
                source: wgpu::ShaderSource::Wgsl(PER_VERTEX_DISTANCE_WGSL.into()),
            });

        Some(Self {
            instance,
            adapter,
            device,
            queue,
            per_vertex_distance_shader,
        })
    }
}

impl ComputeBackend for WgpuBackend {
    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            kind: ComputeKind::Wgpu,
            max_vertices: 100_000_000,
            supports_fp16: true,
            supports_int8: false,
            peak_tops: 5.0,
            low_power_friendly: false,
        }
    }

    fn per_vertex_distance(
        &self,
        src: &Mesh,
        dst: &Mesh,
    ) -> Result<(Vec<f64>, ComputeStats), ComputeError> {
        let started = Instant::now();
        let n_src = src.vertices.len();
        let n_dst = dst.vertices.len();
        if n_src == 0 {
            return Ok((Vec::new(), ComputeStats {
                backend: Some(ComputeKind::Wgpu),
                op: Some(ComputeOp::PerVertexDistance),
                elapsed_ms: 0,
                items_processed: 0,
                fell_back_to_cpu: false,
            }));
        }

        // ── Buffers ─────────────────────────────────────────────────────────
        // Pack as flat [x,y,z, x,y,z, ...] f32 array (12 bytes per vertex). The shader
        // reads with explicit `i * 3` indexing — avoids `array<vec3<f32>>` alignment
        // quirks on Metal/MoltenVK that broke a previous iteration.
        let pack = |verts: &[tlanticad_mesh::nalgebra::Point3<f64>]| -> Vec<f32> {
            let mut out = Vec::with_capacity(verts.len() * 3);
            for v in verts {
                out.push(v.x as f32);
                out.push(v.y as f32);
                out.push(v.z as f32);
            }
            out
        };
        let src_packed = pack(&src.vertices);
        let dst_packed = pack(&dst.vertices);

        let src_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("tlanticad.compute.wgpu.src"),
            contents: bytemuck::cast_slice(&src_packed),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let dst_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("tlanticad.compute.wgpu.dst"),
            contents: bytemuck::cast_slice(&dst_packed),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let distances_size_bytes = (n_src * std::mem::size_of::<f32>()) as u64;
        let dist_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tlanticad.compute.wgpu.distances"),
            size: distances_size_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let params = DistanceParams {
            src_count: n_src as u32,
            dst_count: n_dst as u32,
            _pad0: 0,
            _pad1: 0,
        };
        let params_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("tlanticad.compute.wgpu.params"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tlanticad.compute.wgpu.readback"),
            size: distances_size_bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // ── Pipeline ────────────────────────────────────────────────────────
        let bgl = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tlanticad.compute.wgpu.bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pl = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("tlanticad.compute.wgpu.pipeline-layout"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });
        let pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("tlanticad.compute.wgpu.per_vertex_distance"),
            layout: Some(&pl),
            module: &self.per_vertex_distance_shader,
            entry_point: "main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tlanticad.compute.wgpu.bg"),
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: src_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: dst_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: dist_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: params_buf.as_entire_binding() },
            ],
        });

        // ── Dispatch ────────────────────────────────────────────────────────
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("tlanticad.compute.wgpu.encoder"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("tlanticad.compute.wgpu.pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            let workgroups = ((n_src as u32) + 63) / 64;
            pass.dispatch_workgroups(workgroups, 1, 1);
        }
        encoder.copy_buffer_to_buffer(&dist_buf, 0, &readback, 0, distances_size_bytes);
        self.queue.submit(Some(encoder.finish()));

        // ── Read back ──────────────────────────────────────────────────────
        let slice = readback.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = sender.send(res);
        });
        self.device.poll(wgpu::Maintain::Wait);
        receiver
            .recv()
            .map_err(|e| ComputeError::Kernel { message: format!("readback recv: {e}") })?
            .map_err(|e| ComputeError::Kernel { message: format!("map_async: {e:?}") })?;

        let raw = slice.get_mapped_range();
        let f32_slice: &[f32] = bytemuck::cast_slice(&raw);
        let result: Vec<f64> = f32_slice.iter().map(|&v| v as f64).collect();
        drop(raw);
        readback.unmap();

        Ok((
            result,
            ComputeStats {
                backend: Some(ComputeKind::Wgpu),
                op: Some(ComputeOp::PerVertexDistance),
                elapsed_ms: started.elapsed().as_millis() as u64,
                items_processed: n_src as u64,
                fell_back_to_cpu: false,
            },
        ))
    }

    fn laplacian_smooth(
        &self,
        _mesh: &mut Mesh,
        _iterations: u32,
        _lambda: f64,
    ) -> Result<ComputeStats, ComputeError> {
        // MP-103.b — placeholder; cotangent-Laplacian compute kernel pending.
        Err(ComputeError::OpUnsupported {
            op: ComputeOp::LaplacianSmooth,
            backend: ComputeKind::Wgpu,
        })
    }

    fn is_runnable(&self) -> bool {
        // try_new() already verified at least one adapter and a working device.
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wgpu_backend_constructs_or_skips() {
        match WgpuBackend::try_new() {
            Some(backend) => {
                let caps = backend.capabilities();
                assert_eq!(caps.kind, ComputeKind::Wgpu);
                assert!(backend.is_runnable());
            }
            None => {
                // No adapter / no device — acceptable on headless CI.
            }
        }
    }

    #[test]
    fn wgpu_per_vertex_distance_matches_cpu_when_available() {
        let Some(backend) = WgpuBackend::try_new() else {
            // No GPU on this host — skip silently.
            return;
        };
        let src = tlanticad_mesh::create_box(
            tlanticad_mesh::nalgebra::Point3::origin(),
            tlanticad_mesh::nalgebra::Point3::new(1.0, 1.0, 1.0),
        );
        let dst = src.clone();
        let (gpu_dists, stats) = backend
            .per_vertex_distance(&src, &dst)
            .expect("per_vertex_distance should succeed when adapter is present");
        assert_eq!(gpu_dists.len(), src.vertices.len());
        assert!(gpu_dists.iter().all(|&d| d.abs() < 1e-3));
        assert_eq!(stats.backend, Some(ComputeKind::Wgpu));
        assert!(!stats.fell_back_to_cpu);
    }

    #[test]
    fn wgpu_per_vertex_distance_translated_pair() {
        let Some(backend) = WgpuBackend::try_new() else {
            return;
        };
        let a = tlanticad_mesh::create_box(
            tlanticad_mesh::nalgebra::Point3::origin(),
            tlanticad_mesh::nalgebra::Point3::new(1.0, 1.0, 1.0),
        );
        let b = tlanticad_mesh::create_box(
            tlanticad_mesh::nalgebra::Point3::new(2.0, 0.0, 0.0),
            tlanticad_mesh::nalgebra::Point3::new(3.0, 1.0, 1.0),
        );
        let (dists, _) = backend.per_vertex_distance(&a, &b).unwrap();
        assert!(dists.iter().all(|&d| d >= 0.0 && d.is_finite()));
        // Worst-case corner pairs land within sqrt(2² + 1² + 1²) ≈ 2.45
        let max = dists.iter().cloned().fold(0.0_f64, f64::max);
        assert!(max > 1.0 && max < 4.0, "expected ~2..3, got {max}");
    }
}
