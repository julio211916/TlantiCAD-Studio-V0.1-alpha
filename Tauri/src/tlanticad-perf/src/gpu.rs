//! GPU information (cross-platform via sysinfo + platform-specific queries)

use serde::{Deserialize, Serialize};

/// GPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: String,
    pub vram_total_mb: Option<u64>,
    pub vram_used_mb: Option<u64>,
    pub driver_version: Option<String>,
    pub backend: GpuBackend,
}

/// Graphics API backend
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuBackend {
    Metal,
    DirectX12,
    Vulkan,
    OpenGL,
    Unknown,
}

impl GpuBackend {
    pub fn current_platform() -> Self {
        #[cfg(target_os = "macos")]
        return GpuBackend::Metal;
        #[cfg(target_os = "windows")]
        return GpuBackend::DirectX12;
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        return GpuBackend::Vulkan;
    }
}

/// Collect GPU info (best-effort, no external GPU libraries required)
pub fn collect_gpu_info() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("system_profiler")
            .arg("SPDisplaysDataType")
            .arg("-json")
            .output()
        {
            if let Ok(json_str) = std::str::from_utf8(&output.stdout) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let Some(displays) = json["SPDisplaysDataType"].as_array() {
                        for display in displays {
                            let name = display["sppci_model"].as_str().unwrap_or("Apple GPU").to_string();
                            let vram = display["spdisplays_vram"].as_str()
                                .and_then(|s| s.split_whitespace().next())
                                .and_then(|s| s.parse::<u64>().ok());
                            gpus.push(GpuInfo {
                                name,
                                vendor: "Apple".to_string(),
                                vram_total_mb: vram,
                                vram_used_mb: None,
                                driver_version: None,
                                backend: GpuBackend::Metal,
                            });
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("wmic")
            .args(&["path", "win32_videocontroller", "get", "Name,AdapterRAM", "/format:csv"])
            .output()
        {
            if let Ok(text) = std::str::from_utf8(&output.stdout) {
                for line in text.lines().skip(2) {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 3 {
                        let vram = parts[1].trim().parse::<u64>().ok().map(|b| b / 1_048_576);
                        let name = parts[2].trim().to_string();
                        if !name.is_empty() {
                            gpus.push(GpuInfo {
                                name,
                                vendor: String::new(),
                                vram_total_mb: vram,
                                vram_used_mb: None,
                                driver_version: None,
                                backend: GpuBackend::DirectX12,
                            });
                        }
                    }
                }
            }
        }
    }

    if gpus.is_empty() {
        gpus.push(GpuInfo {
            name: "Unknown GPU".to_string(),
            vendor: String::new(),
            vram_total_mb: None,
            vram_used_mb: None,
            driver_version: None,
            backend: GpuBackend::current_platform(),
        });
    }

    gpus
}
