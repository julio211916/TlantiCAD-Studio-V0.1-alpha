//! GPU / WebGPU detection using wgpu
//!
//! Enumerates GPU adapters and reports capabilities.

use crate::{GpuInfo, Capabilities, RenderQuality};

/// Detect all available GPUs and their capabilities
pub fn detect_gpus() -> (Vec<GpuInfo>, Capabilities) {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapters = instance.enumerate_adapters(wgpu::Backends::all());
    let mut gpus = Vec::new();
    let mut best_caps = Capabilities {
        webgpu_supported: false,
        vulkan_supported: false,
        metal_supported: false,
        dx12_supported: false,
        max_texture_size: 0,
        max_buffer_size: 0,
        recommended_render_quality: RenderQuality::Low,
    };

    for adapter in adapters {
        let info = adapter.get_info();
        let limits = adapter.limits();

        let backend_str = format!("{:?}", info.backend);
        let device_type_str = format!("{:?}", info.device_type);

        match info.backend {
            wgpu::Backend::Vulkan => best_caps.vulkan_supported = true,
            wgpu::Backend::Metal => best_caps.metal_supported = true,
            wgpu::Backend::Dx12 => best_caps.dx12_supported = true,
            wgpu::Backend::BrowserWebGpu => best_caps.webgpu_supported = true,
            _ => {}
        }

        if limits.max_texture_dimension_2d > best_caps.max_texture_size {
            best_caps.max_texture_size = limits.max_texture_dimension_2d;
        }
        if limits.max_buffer_size > best_caps.max_buffer_size {
            best_caps.max_buffer_size = limits.max_buffer_size;
        }

        gpus.push(GpuInfo {
            name: info.name.clone(),
            vendor: format!("0x{:x}", info.vendor),
            backend: backend_str,
            driver_info: info.driver_info.clone(),
            device_type: device_type_str,
            features: Vec::new(),
        });
    }

    // Determine recommended quality
    best_caps.recommended_render_quality = if best_caps.max_texture_size >= 16384 {
        RenderQuality::Ultra
    } else if best_caps.max_texture_size >= 8192 {
        RenderQuality::High
    } else if best_caps.max_texture_size >= 4096 {
        RenderQuality::Medium
    } else {
        RenderQuality::Low
    };

    (gpus, best_caps)
}
