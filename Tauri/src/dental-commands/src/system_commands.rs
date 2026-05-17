//! Tauri commands for system info / GPU detection / AI workload analysis

use crate::DentalCommandError;
use system_info::{ai_workload, gpu, system, Capabilities, GpuInfo, SystemReport};

#[tauri::command]
pub async fn system_get_full_report() -> Result<SystemReport, DentalCommandError> {
    let (os, cpu, memory, disks) = system::get_system_report_base();
    let (gpus, capabilities) = gpu::detect_gpus();

    Ok(SystemReport {
        os,
        cpu,
        memory,
        gpus,
        disks,
        capabilities,
    })
}

#[tauri::command]
pub async fn system_detect_gpu() -> Result<(Vec<GpuInfo>, Capabilities), DentalCommandError> {
    Ok(gpu::detect_gpus())
}

#[tauri::command]
pub async fn system_get_memory() -> Result<system_info::MemoryInfo, DentalCommandError> {
    Ok(system::get_memory_info())
}

#[tauri::command]
pub async fn system_get_os() -> Result<system_info::OsInfo, DentalCommandError> {
    Ok(system::get_os_info())
}

/// Analyze system AI/imaging workload capabilities
#[tauri::command]
pub async fn system_ai_workload_report(
) -> Result<ai_workload::AiWorkloadReport, DentalCommandError> {
    let (_os, _cpu, memory, disks) = system::get_system_report_base();
    let (gpus, capabilities) = gpu::detect_gpus();

    Ok(ai_workload::analyze_ai_workload(
        &gpus,
        &memory,
        &disks,
        &capabilities,
    ))
}

/// Get just the workload scores (lightweight)
#[tauri::command]
pub async fn system_ai_workload_scores(
) -> Result<ai_workload::WorkloadScores, DentalCommandError> {
    let report = system_ai_workload_report().await?;
    Ok(report.workload_scores)
}

/// Get recommended application settings
#[tauri::command]
pub async fn system_recommended_settings(
) -> Result<ai_workload::RecommendedSettings, DentalCommandError> {
    let report = system_ai_workload_report().await?;
    Ok(report.recommended_settings)
}
