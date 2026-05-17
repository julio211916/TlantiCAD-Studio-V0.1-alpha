use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendTopology {
    pub active_app_manifest: &'static str,
    pub active_library_workspace: &'static str,
    pub canonical_database_crate: &'static str,
    pub canonical_compute_crate: &'static str,
    pub active_runtime_modules: &'static [&'static str],
    pub dormant_crates: &'static [&'static str],
    pub architecture_rules: &'static [&'static str],
}

const ACTIVE_RUNTIME_MODULES: &[&str] = &[
    "case_repository",
    "case_storage",
    "case_watcher",
    "clinical_jobs",
    "mesh_vault",
    "cad_parameters_store",
    "cad_compute_router",
    "python_runtime",
    "dicom_jobs",
    "dicom_segmentation_jobs",
    "local_share",
    "public_asset_manifest",
];

const DORMANT_CRATES: &[&str] = &[
    "Tauri/src/database",
    "Tauri/src/db",
    "Tauri/src/dental-database",
    "Tauri/src/cad-db",
    "Tauri/src/commands",
    "Tauri/src/dental-commands",
    "Tauri/src/python-bridge",
    "Tauri/src/sidecar-manager",
    "Tauri/src/cad-core",
    "Tauri/src/dental-core",
    "Tauri/src/patients",
    "Tauri/src/clinical",
];

const ARCHITECTURE_RULES: &[&str] = &[
    "Tauri/Cargo.toml is the desktop app manifest; Tauri/src/Cargo.toml is the active library workspace.",
    "tlanticad-workspace publishes package tlanticad-db and is the target for new persistent repositories.",
    "tlanticad-compute owns backend discovery and routing; Tauri/src/cad_compute_router.rs is only the IPC adapter.",
    "Dormant crates are not allowed as new imports until they are migrated into an active tlanticad-* crate.",
    "Clinical identity belongs to dental/domain crates; CAD persistence stores asset and workspace references, not duplicate Patient records.",
    "Long-running Python processes must be supervised by one runtime path; do not add new ad-hoc Python spawners.",
];

#[tauri::command]
pub fn inspect_backend_topology() -> BackendTopology {
    BackendTopology {
        active_app_manifest: "Tauri/Cargo.toml",
        active_library_workspace: "Tauri/src/Cargo.toml",
        canonical_database_crate: "Tauri/src/tlanticad-workspace (package tlanticad-db)",
        canonical_compute_crate: "Tauri/src/tlanticad-compute",
        active_runtime_modules: ACTIVE_RUNTIME_MODULES,
        dormant_crates: DORMANT_CRATES,
        architecture_rules: ARCHITECTURE_RULES,
    }
}
