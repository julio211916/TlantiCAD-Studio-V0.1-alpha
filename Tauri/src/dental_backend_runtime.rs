use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DentalBackendLayer {
    pub id: &'static str,
    pub owner: &'static str,
    pub responsibility: &'static str,
    pub runtime: &'static str,
    pub performance_rule: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DentalBackendEndpoint {
    pub method: &'static str,
    pub route: &'static str,
    pub owner: &'static str,
    pub notes: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DentalBackendCrateBinding {
    pub crate_or_module: &'static str,
    pub domain: &'static str,
    pub bound_to: &'static str,
    pub status: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DentalBackendWorkflow {
    pub id: &'static str,
    pub label: &'static str,
    pub stages: Vec<&'static str>,
    pub clinical_export_guard: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DentalBackendArchitecture {
    pub name: &'static str,
    pub offline_only: bool,
    pub python_version_target: &'static str,
    pub database: &'static str,
    pub queue: &'static str,
    pub layers: Vec<DentalBackendLayer>,
    pub endpoints: Vec<DentalBackendEndpoint>,
    pub crate_bindings: Vec<DentalBackendCrateBinding>,
    pub workflows: Vec<DentalBackendWorkflow>,
    pub bottlenecks: Vec<&'static str>,
    pub optimizations: Vec<&'static str>,
}

#[tauri::command]
pub fn get_dental_backend_architecture() -> DentalBackendArchitecture {
    DentalBackendArchitecture {
        name: "TlantiCAD local dental backend",
        offline_only: true,
        python_version_target: "Python 3.11+",
        database: "SQLModel/Alembic local SQLite now, PostgreSQL-compatible schema later",
        queue: "Celery + Redis for Python jobs; Tauri clinical jobs remain authoritative for CAD artifacts",
        layers: vec![
            DentalBackendLayer {
                id: "react-ui",
                owner: "React",
                responsibility: "orchestrate screens, show handles, never decode full DICOM/STL buffers",
                runtime: "Vite/Tauri WebView",
                performance_rule: "lazy boundaries for Cornerstone, Three, DICOM and AI panels",
            },
            DentalBackendLayer {
                id: "tauri-orchestration",
                owner: "Tauri commands",
                responsibility: "capability discovery, file handles, artifact manifest and local process supervision",
                runtime: "Rust/Tauri",
                performance_rule: "batch IPC and move CPU work into Rust/Python jobs",
            },
            DentalBackendLayer {
                id: "rust-core",
                owner: "Rust crates",
                responsibility: "case-core, asset-vault, crown pipeline, mesh artifacts and clinical export guards",
                runtime: "native Rust",
                performance_rule: "stream files, bounded queues, zero-copy handles where possible",
            },
            DentalBackendLayer {
                id: "python-dicom-ai",
                owner: "Python backend",
                responsibility: "pydicom metadata, SimpleITK volume IO, VTK mesh extraction, ONNX Runtime inference",
                runtime: "embedded/local Python",
                performance_rule: "no browser pixel buffers; chunked DICOM and INT8 CPU inference for production",
            },
        ],
        endpoints: vec![
            DentalBackendEndpoint {
                method: "GET",
                route: "/api/v1/health/local",
                owner: "FastAPI",
                notes: "reports pydicom, VTK, SimpleITK, DB and queue capabilities",
            },
            DentalBackendEndpoint {
                method: "POST",
                route: "/api/v1/studies/inspect",
                owner: "Python/pydicom",
                notes: "metadata-only DICOM inspection; no pixel payload returned to React",
            },
            DentalBackendEndpoint {
                method: "POST",
                route: "/api/v1/inference/start",
                owner: "Python/ONNX Runtime",
                notes: "queues local inference or blocks when model is missing",
            },
            DentalBackendEndpoint {
                method: "WS",
                route: "/api/ws/events",
                owner: "FastAPI",
                notes: "local progress events for jobs and future task telemetry",
            },
        ],
        crate_bindings: vec![
            DentalBackendCrateBinding {
                crate_or_module: "case_repository",
                domain: "case-core",
                bound_to: "SQLModel Study metadata and Rust manifest artifacts",
                status: "active",
            },
            DentalBackendCrateBinding {
                crate_or_module: "mesh_vault",
                domain: "asset-vault",
                bound_to: "backend/app/services/dicom_service.py for DICOM metadata handles",
                status: "active",
            },
            DentalBackendCrateBinding {
                crate_or_module: "cad_crown_pipeline",
                domain: "restorative CAD",
                bound_to: "clinical export guard; rejects missing real artifacts",
                status: "active",
            },
            DentalBackendCrateBinding {
                crate_or_module: "backend/modules/dicom",
                domain: "CBCT/DICOM",
                bound_to: "pydicom + SimpleITK + VTK local runtime",
                status: "new",
            },
        ],
        workflows: vec![
            DentalBackendWorkflow {
                id: "implant-planning",
                label: "Dental implant planning",
                stages: vec![
                    "Import DICOM",
                    "Inspect pydicom metadata",
                    "Build VTK/SimpleITK volume handle",
                    "Register implant library selection",
                    "Plan axis and safety zones",
                    "Persist manifest and artifacts",
                ],
                clinical_export_guard: "blocked until implant geometry, transforms and collision checks are persisted",
            },
            DentalBackendWorkflow {
                id: "crown",
                label: "Crown vertical slice",
                stages: vec![
                    "Open case",
                    "Import prep scan handle",
                    "Compute margin/crown bottom",
                    "Validate thickness/contact",
                    "Save STL/JSON artifact",
                    "Export manifest",
                ],
                clinical_export_guard: "blocked on explicit placeholder-artifact detection",
            },
        ],
        bottlenecks: vec![
            "DICOM decompression and full-volume allocation",
            "marching cubes on dense CBCT without ROI clipping",
            "browser-side Cornerstone/Three chunk size",
            "CSG/offset operations on high triangle-count scans",
        ],
        optimizations: vec![
            "metadata-first pydicom inspection before pixel load",
            "ROI cropping and downsampled preview volumes before full VTK mesh extraction",
            "lazy import boundaries for DICOM, Three, AI and manufacturing panels",
            "Rust artifact handles instead of React arrayBuffer pipelines",
            "ONNX Runtime INT8 for production inference; PyTorch only for dev training",
        ],
    }
}
