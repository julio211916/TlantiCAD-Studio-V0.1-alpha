use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::case_repository::{self, DentalCaseDto};
use crate::clinical_jobs::{self, ClinicalJobDto, ClinicalJobRecordRequest};

const CASE_DIRECTORIES: [&str; 22] = [
    "input/scans",
    "input/dicom",
    "input/photos",
    "input/jaw-motion",
    "working/scene",
    "working/meshes",
    "working/masks",
    "working/registrations",
    "working/articulator",
    "working/aligners",
    "jobs/rust",
    "jobs/python",
    "jobs/logs",
    "libraries/implants",
    "libraries/materials",
    "libraries/teeth",
    "output/stl",
    "output/obj",
    "output/3mf",
    "output/reports",
    "output/surgical-guide",
    "output/cam",
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CadBootstrapRequest {
    pub case_id: Option<String>,
    pub module_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CadStorageBootstrap {
    pub database: String,
    pub case_root: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CadToolDefinitionDto {
    pub id: &'static str,
    pub label: &'static str,
    pub category: &'static str,
    pub owner: &'static str,
    pub permissions: Vec<&'static str>,
    pub required_assets: Vec<&'static str>,
    pub job_kinds: Vec<&'static str>,
    pub performance_rule: &'static str,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WizardStepDto {
    pub id: &'static str,
    pub label: &'static str,
    pub owner: &'static str,
    pub tools: Vec<&'static str>,
    pub jobs: Vec<&'static str>,
    pub required_assets: Vec<&'static str>,
    pub output_assets: Vec<&'static str>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WizardDefinitionDto {
    pub id: &'static str,
    pub label: &'static str,
    pub module_id: &'static str,
    pub steps: Vec<WizardStepDto>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModuleManifestDto {
    pub id: &'static str,
    pub label: &'static str,
    pub owner: &'static str,
    pub purpose: &'static str,
    pub workflows: Vec<WizardDefinitionDto>,
    pub tools: Vec<&'static str>,
    pub permissions: Vec<&'static str>,
    pub dependencies: Vec<&'static str>,
    pub output_assets: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolRegistryDto {
    pub tools: Vec<CadToolDefinitionDto>,
    pub modules: Vec<ModuleManifestDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CadDependencyDto {
    pub id: &'static str,
    pub label: &'static str,
    pub owner: &'static str,
    pub required_for: Vec<&'static str>,
    pub offline: bool,
    pub notes: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CadServiceContractDto {
    pub id: &'static str,
    pub source_pattern: &'static str,
    pub owner: &'static str,
    pub operations: Vec<&'static str>,
    pub input_refs: Vec<&'static str>,
    pub output_refs: Vec<&'static str>,
    pub streaming: &'static str,
    pub performance_rule: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CadBootstrapDto {
    pub offline_required: bool,
    pub storage: CadStorageBootstrap,
    pub tool_registry: ToolRegistryDto,
    pub dependencies: Vec<CadDependencyDto>,
    pub service_contracts: Vec<CadServiceContractDto>,
    pub commands: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseFolderLayoutDto {
    pub case_id: String,
    pub root: String,
    pub manifest_path: String,
    pub work_definition_path: String,
    pub directories: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseOpenResponseDto {
    pub dental_case: Option<DentalCaseDto>,
    pub layout: CaseFolderLayoutDto,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetImportPrepareRequest {
    pub case_id: String,
    pub asset_id: String,
    pub file_name: String,
    pub kind: String,
    pub module_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetImportPreparationDto {
    pub case_id: String,
    pub asset_id: String,
    pub target_directory: String,
    pub target_path: String,
    pub manifest_path: String,
    pub allowed_extensions: Vec<&'static str>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CadJobRequestDto {
    pub case_id: String,
    pub module_id: String,
    pub kind: String,
    pub runtime: String,
    pub input_asset_ids: Vec<String>,
    pub params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStartRequestDto {
    pub case_id: String,
    pub module_id: String,
    pub workflow_step_id: String,
    pub input_asset_ids: Vec<String>,
    pub params: serde_json::Value,
}

const WORKFLOW_STEP_IDS: [&str; 6] = ["import", "clean", "segment", "design", "validate", "export"];

fn workflow_runtime(step_id: &str) -> Option<&'static str> {
    match step_id {
        "import" | "export" => Some("tauri-rust"),
        "clean" | "design" | "validate" => Some("rust-core"),
        "segment" => Some("fastapi-python"),
        _ => None,
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CadJobStatusDto {
    pub id: String,
    pub case_id: String,
    pub kind: String,
    pub status: String,
    pub progress: f64,
    pub runtime: String,
    pub artifacts: Vec<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModulePermissionsRequestDto {
    pub module_id: String,
    pub features: Vec<String>,
    pub role: Option<String>,
    pub installed_dependencies: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModulePermissionDecisionDto {
    pub permission: &'static str,
    pub expression: &'static str,
    pub allowed: bool,
    pub reason: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModulePermissionsResponseDto {
    pub module: ModuleManifestDto,
    pub decisions: Vec<ModulePermissionDecisionDto>,
}

fn app_data_root(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve app data dir: {error}"))
}

fn safe_segment(value: &str, fallback: &str) -> String {
    let segment: String = value
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
        .collect();

    if segment.is_empty() {
        fallback.to_string()
    } else {
        segment
    }
}

fn safe_filename(value: &str, fallback: &str) -> String {
    PathBuf::from(value)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|value| safe_segment(value, fallback))
        .unwrap_or_else(|| fallback.to_string())
}

fn case_root(app: &AppHandle, case_id: &str) -> Result<PathBuf, String> {
    Ok(app_data_root(app)?
        .join("cases")
        .join(safe_segment(case_id, "case")))
}

fn build_case_layout(app: &AppHandle, case_id: &str) -> Result<CaseFolderLayoutDto, String> {
    let root = case_root(app, case_id)?;
    let root_string = root.to_string_lossy().to_string();
    Ok(CaseFolderLayoutDto {
        case_id: safe_segment(case_id, "case"),
        manifest_path: root.join("manifest.json").to_string_lossy().to_string(),
        work_definition_path: root
            .join("work-definition.json")
            .to_string_lossy()
            .to_string(),
        directories: CASE_DIRECTORIES
            .iter()
            .map(|directory| root.join(directory).to_string_lossy().to_string())
            .collect(),
        root: root_string,
    })
}

fn ensure_case_layout(app: &AppHandle, case_id: &str) -> Result<CaseFolderLayoutDto, String> {
    let layout = build_case_layout(app, case_id)?;
    fs::create_dir_all(&layout.root)
        .map_err(|error| format!("Could not create case root {}: {error}", layout.root))?;
    for directory in &layout.directories {
        fs::create_dir_all(directory)
            .map_err(|error| format!("Could not create case directory {directory}: {error}"))?;
    }
    Ok(layout)
}

fn tool_registry() -> ToolRegistryDto {
    ToolRegistryDto {
        tools: vec![
            tool(
                "select",
                "Select",
                "scene",
                "react-ui",
                vec![],
                vec![],
                vec![],
                "Selector writes ids only; no mesh copies.",
            ),
            tool(
                "mesh-vault",
                "Mesh push / pull / find",
                "mesh",
                "rust-core",
                vec!["cad:mesh_edit"],
                vec!["stl-mesh"],
                vec!["mesh-import", "mesh-export"],
                "Large meshes are hash-keyed and chunk streamed.",
            ),
            tool(
                "repair",
                "Repair mesh",
                "mesh",
                "rust-core",
                vec!["cad:mesh_edit"],
                vec!["stl-mesh"],
                vec!["mesh-repair"],
                "Repair runs async and emits artifacts.",
            ),
            tool(
                "margin",
                "Margin",
                "dental-design",
                "rust-core",
                vec!["cad:mesh_edit"],
                vec!["stl-mesh"],
                vec!["margin-detection"],
                "Margin detection is a cancellable job.",
            ),
            tool(
                "dicom-mpr",
                "DICOM MPR",
                "dicom",
                "python-sidecar",
                vec!["asset:import_dicom"],
                vec!["dicom-series"],
                vec!["dicom-metadata"],
                "DICOM is tiled/streamed outside React.",
            ),
            tool(
                "implant-library",
                "Implant library",
                "implant",
                "tauri-command",
                vec!["module:implant"],
                vec![],
                vec!["implant-plan"],
                "Libraries are local manifests.",
            ),
            tool(
                "sleeve-guide",
                "Sleeve / surgical guide",
                "implant",
                "rust-core",
                vec!["module:surgical_guide"],
                vec!["stl-mesh"],
                vec!["guide-export"],
                "Guide generation is async.",
            ),
            tool(
                "articulator",
                "Virtual articulator",
                "articulator",
                "rust-core",
                vec!["module:splint"],
                vec!["stl-mesh"],
                vec!["jaw-motion"],
                "Jaw transforms are cached.",
            ),
            tool(
                "ceph-landmarks",
                "Ceph landmarks",
                "ceph",
                "python-sidecar",
                vec!["module:ceph"],
                vec!["dicom-series"],
                vec!["ceph-landmarks"],
                "Landmark confidence is persisted.",
            ),
            tool(
                "ortho-setup",
                "Ortho setup",
                "ortho",
                "python-sidecar",
                vec!["module:aligner"],
                vec!["stl-mesh"],
                vec!["ortho-fipos"],
                "Stage transforms are artifacts.",
            ),
            tool(
                "export-manufacturing",
                "Manufacturing export",
                "manufacturing",
                "rust-core",
                vec!["export:manufacturing"],
                vec!["stl-mesh"],
                vec!["stl-export", "3mf-export"],
                "Exports include provenance.",
            ),
        ],
        modules: module_manifests(),
    }
}

#[tauri::command]
pub fn tool_registry_get() -> Result<ToolRegistryDto, String> {
    Ok(tool_registry())
}

fn tool(
    id: &'static str,
    label: &'static str,
    category: &'static str,
    owner: &'static str,
    permissions: Vec<&'static str>,
    required_assets: Vec<&'static str>,
    job_kinds: Vec<&'static str>,
    performance_rule: &'static str,
) -> CadToolDefinitionDto {
    CadToolDefinitionDto {
        id,
        label,
        category,
        owner,
        permissions,
        required_assets,
        job_kinds,
        performance_rule,
    }
}

fn wizard(
    id: &'static str,
    label: &'static str,
    module_id: &'static str,
    steps: Vec<WizardStepDto>,
) -> WizardDefinitionDto {
    WizardDefinitionDto {
        id,
        label,
        module_id,
        steps,
    }
}

fn step(
    id: &'static str,
    label: &'static str,
    owner: &'static str,
    tools: Vec<&'static str>,
    jobs: Vec<&'static str>,
    required_assets: Vec<&'static str>,
    output_assets: Vec<&'static str>,
) -> WizardStepDto {
    WizardStepDto {
        id,
        label,
        owner,
        tools,
        jobs,
        required_assets,
        output_assets,
    }
}

fn module_manifests() -> Vec<ModuleManifestDto> {
    vec![
        module_manifest(
            "cad",
            "CAD Core",
            "rust-core",
            "Mesh-first restorative CAD core.",
            vec![cad_wizard()],
            vec![
                "select",
                "mesh-vault",
                "repair",
                "margin",
                "export-manufacturing",
            ],
            vec!["module:cad", "cad:mesh_edit"],
            vec!["sqlite-local-db", "case-folder-vault", "rust-mesh-core"],
            vec!["stl-mesh", "manufacturing-export"],
        ),
        module_manifest(
            "dicom",
            "DICOM Core",
            "python-sidecar",
            "CBCT/DICOM processing and CAD handoff.",
            vec![dicom_wizard()],
            vec!["dicom-mpr", "mesh-vault"],
            vec!["module:dicom", "asset:import_dicom"],
            vec!["python-ai-dicom", "case-folder-vault"],
            vec!["stl-mesh", "report"],
        ),
        module_manifest(
            "implant",
            "Implant Core",
            "rust-core",
            "Implant planning, abutment and guide handoff.",
            vec![implant_wizard()],
            vec!["implant-library", "sleeve-guide", "dicom-mpr"],
            vec!["module:implant"],
            vec!["implant-library", "rust-mesh-core", "python-ai-dicom"],
            vec!["report", "manufacturing-export"],
        ),
        module_manifest(
            "guide",
            "Surgical Guide Core",
            "rust-core",
            "Sleeves, guide body and drill protocol.",
            vec![implant_wizard()],
            vec!["sleeve-guide", "mesh-vault", "export-manufacturing"],
            vec!["module:surgical_guide"],
            vec!["implant-library", "rust-mesh-core"],
            vec!["manufacturing-export"],
        ),
        module_manifest(
            "splint",
            "Splint and Articulator Core",
            "rust-core",
            "Bite splints, articulator and occlusion maps.",
            vec![splint_wizard()],
            vec!["articulator", "mesh-vault", "export-manufacturing"],
            vec!["module:splint"],
            vec!["rust-mesh-core", "python-ai-dicom"],
            vec!["manufacturing-export", "report"],
        ),
        module_manifest(
            "ceph",
            "Cephalometrics Core",
            "python-sidecar",
            "Landmarks, tracing and reports.",
            vec![ceph_wizard()],
            vec!["ceph-landmarks", "dicom-mpr"],
            vec!["module:ceph"],
            vec!["python-ai-dicom"],
            vec!["report"],
        ),
        module_manifest(
            "aligners",
            "Aligner Core",
            "python-sidecar",
            "Tooth setup, staging, IPR and trays.",
            vec![aligner_wizard()],
            vec!["ortho-setup", "export-manufacturing"],
            vec!["module:aligner"],
            vec!["python-ai-dicom", "rust-mesh-core"],
            vec!["manufacturing-export"],
        ),
        module_manifest(
            "orthocad",
            "Smile and Ortho Core",
            "python-sidecar",
            "Smile, ortho setup and waxup handoff.",
            vec![aligner_wizard()],
            vec!["ortho-setup", "ceph-landmarks"],
            vec!["module:orthocad"],
            vec!["python-ai-dicom"],
            vec!["scene-snapshot", "report"],
        ),
        module_manifest(
            "model-creator",
            "Model Creator Core",
            "rust-core",
            "Printable models, bases, labels and dies.",
            vec![cad_wizard()],
            vec!["mesh-vault", "repair", "export-manufacturing"],
            vec!["module:model_creator"],
            vec!["rust-mesh-core"],
            vec!["manufacturing-export"],
        ),
        module_manifest(
            "partials",
            "Partial and Bar Core",
            "rust-core",
            "Survey, framework, clasps and bars.",
            vec![cad_wizard()],
            vec!["mesh-vault", "repair", "export-manufacturing"],
            vec!["module:partials"],
            vec!["rust-mesh-core"],
            vec!["manufacturing-export", "report"],
        ),
        module_manifest(
            "fab",
            "Manufacturing Core",
            "rust-core",
            "Validation, material profiles and CAM handoff.",
            vec![cad_wizard()],
            vec!["repair", "export-manufacturing"],
            vec!["module:fab", "export:manufacturing"],
            vec!["rust-mesh-core"],
            vec!["manufacturing-export", "report"],
        ),
    ]
}

fn module_manifest(
    id: &'static str,
    label: &'static str,
    owner: &'static str,
    purpose: &'static str,
    workflows: Vec<WizardDefinitionDto>,
    tools: Vec<&'static str>,
    permissions: Vec<&'static str>,
    dependencies: Vec<&'static str>,
    output_assets: Vec<&'static str>,
) -> ModuleManifestDto {
    ModuleManifestDto {
        id,
        label,
        owner,
        purpose,
        workflows,
        tools,
        permissions,
        dependencies,
        output_assets,
    }
}

fn cad_wizard() -> WizardDefinitionDto {
    wizard(
        "cad-core-import-design-export",
        "Core CAD: Import -> Clean -> Segment -> Design -> Validate -> Export",
        "cad",
        vec![
            step(
                "cad-import",
                "Import scans/assets",
                "tauri-command",
                vec!["mesh-vault"],
                vec!["mesh-import"],
                vec![],
                vec!["stl-mesh"],
            ),
            step(
                "cad-clean",
                "Clean and repair",
                "rust-core",
                vec!["repair"],
                vec!["mesh-repair"],
                vec!["stl-mesh"],
                vec!["stl-mesh"],
            ),
            step(
                "cad-design",
                "Design restoration",
                "rust-core",
                vec!["margin"],
                vec!["margin-detection"],
                vec!["stl-mesh"],
                vec!["stl-mesh"],
            ),
            step(
                "cad-export",
                "Validate and export",
                "rust-core",
                vec!["export-manufacturing"],
                vec!["stl-export"],
                vec!["stl-mesh"],
                vec!["manufacturing-export"],
            ),
        ],
    )
}

fn dicom_wizard() -> WizardDefinitionDto {
    wizard(
        "dicom-ai-pipeline",
        "DICOM: Load -> Preprocess -> Infer -> Postprocess -> Edit",
        "dicom",
        vec![
            step(
                "dicom-load",
                "Load DICOM series",
                "tauri-command",
                vec!["dicom-mpr"],
                vec!["dicom-metadata"],
                vec!["dicom-series"],
                vec!["scene-snapshot"],
            ),
            step(
                "dicom-infer",
                "Offline segmentation",
                "python-sidecar",
                vec!["dicom-mpr"],
                vec!["dicom-segmentation"],
                vec!["dicom-series"],
                vec!["mask"],
            ),
            step(
                "dicom-mesh",
                "Mesh extraction and CAD handoff",
                "rust-core",
                vec!["mesh-vault"],
                vec!["mask-to-mesh"],
                vec!["mask"],
                vec!["stl-mesh"],
            ),
        ],
    )
}

fn implant_wizard() -> WizardDefinitionDto {
    wizard(
        "implant-surgical-guide",
        "Implant: DICOM/STL -> Plan -> Guide -> Report",
        "implant",
        vec![
            step(
                "implant-ingest",
                "CBCT and STL ingest",
                "python-sidecar",
                vec!["dicom-mpr", "mesh-vault"],
                vec!["dicom-metadata", "mesh-import"],
                vec!["dicom-series", "stl-mesh"],
                vec!["scene-snapshot"],
            ),
            step(
                "implant-plan",
                "Implant and abutment plan",
                "rust-core",
                vec!["implant-library"],
                vec!["implant-plan"],
                vec!["stl-mesh"],
                vec!["report"],
            ),
            step(
                "implant-guide",
                "Surgical guide and drill protocol",
                "rust-core",
                vec!["sleeve-guide", "export-manufacturing"],
                vec!["guide-export"],
                vec!["stl-mesh"],
                vec!["manufacturing-export", "report"],
            ),
        ],
    )
}

fn splint_wizard() -> WizardDefinitionDto {
    wizard(
        "splint-articulator",
        "Splint: Bite -> Articulator -> Bottom -> Top -> Export",
        "splint",
        vec![
            step(
                "splint-bite",
                "Scan pair and bite relation",
                "tauri-command",
                vec!["articulator"],
                vec!["jaw-registration"],
                vec!["stl-mesh"],
                vec!["scene-snapshot"],
            ),
            step(
                "splint-bottom",
                "Create bite splint bottom",
                "rust-core",
                vec!["mesh-vault"],
                vec!["bitesplint-create-bottom"],
                vec!["stl-mesh"],
                vec!["stl-mesh"],
            ),
            step(
                "splint-export",
                "Validate and export splint",
                "rust-core",
                vec!["export-manufacturing"],
                vec!["stl-export"],
                vec!["stl-mesh"],
                vec!["manufacturing-export"],
            ),
        ],
    )
}

fn ceph_wizard() -> WizardDefinitionDto {
    wizard(
        "ceph-analysis",
        "Ceph: Image/CBCT -> Landmarks -> Trace -> Report",
        "ceph",
        vec![
            step(
                "ceph-load",
                "Load image or CBCT",
                "tauri-command",
                vec!["dicom-mpr"],
                vec!["dicom-metadata"],
                vec!["dicom-series"],
                vec!["scene-snapshot"],
            ),
            step(
                "ceph-landmarks",
                "Landmarks and planes",
                "python-sidecar",
                vec!["ceph-landmarks"],
                vec!["ceph-landmarks"],
                vec!["dicom-series"],
                vec!["report"],
            ),
        ],
    )
}

fn aligner_wizard() -> WizardDefinitionDto {
    wizard(
        "ortho-aligner-fipos",
        "Ortho/Aligners: Segment -> Setup -> Stage -> Attachments -> Export",
        "aligners",
        vec![
            step(
                "aligner-segment",
                "Segment teeth and roots",
                "python-sidecar",
                vec!["ortho-setup"],
                vec!["tooth-segmentation"],
                vec!["stl-mesh"],
                vec!["mask"],
            ),
            step(
                "aligner-setup",
                "FIPOS setup and constraints",
                "python-sidecar",
                vec!["ortho-setup"],
                vec!["ortho-fipos"],
                vec!["stl-mesh"],
                vec!["scene-snapshot", "report"],
            ),
            step(
                "aligner-export",
                "Tray export",
                "rust-core",
                vec!["export-manufacturing"],
                vec!["3mf-export", "stl-export"],
                vec!["stl-mesh"],
                vec!["manufacturing-export"],
            ),
        ],
    )
}

fn dependencies() -> Vec<CadDependencyDto> {
    vec![
        dependency(
            "sqlite-local-db",
            "SQLite local DB",
            "asset-vault",
            vec![
                "cad", "dicom", "implant", "guide", "splint", "ceph", "aligners", "orthocad",
            ],
            true,
            "Metadata, jobs, permissions and audit index.",
        ),
        dependency(
            "case-folder-vault",
            "Case folder vault",
            "asset-vault",
            vec![
                "cad", "dicom", "implant", "guide", "splint", "ceph", "aligners", "orthocad",
            ],
            true,
            "Large files, meshes, DICOM, masks, reports and exports.",
        ),
        dependency(
            "rust-mesh-core",
            "Rust mesh core",
            "rust-core",
            vec!["cad", "guide", "splint", "model-creator", "partials", "fab"],
            true,
            "Mesh repair, boolean, offset, export and registration.",
        ),
        dependency(
            "python-ai-dicom",
            "Embedded Python AI/DICOM",
            "python-sidecar",
            vec!["dicom", "implant", "ceph", "aligners", "orthocad"],
            true,
            "PyDICOM, ONNX/TorchScript and MONAI style jobs without external APIs.",
        ),
        dependency(
            "implant-library",
            "Implant and sleeve libraries",
            "asset-vault",
            vec!["implant", "guide"],
            true,
            "Local manifests for implants, scan bodies, sleeves and drill protocols.",
        ),
        dependency(
            "material-library",
            "Material library",
            "asset-vault",
            vec!["cad", "splint", "fab"],
            true,
            "Material rules, connector thresholds, minimum thickness and export profiles.",
        ),
    ]
}

fn dependency(
    id: &'static str,
    label: &'static str,
    owner: &'static str,
    required_for: Vec<&'static str>,
    offline: bool,
    notes: &'static str,
) -> CadDependencyDto {
    CadDependencyDto {
        id,
        label,
        owner,
        required_for,
        offline,
        notes,
    }
}

fn service_contracts() -> Vec<CadServiceContractDto> {
    vec![
        service_contract("scanner-wizard-service", "scanner-wizard", "tauri-command", vec!["connect", "preview", "scan", "pause", "resume", "cancel", "triangulate", "finish"], vec!["Treatment", "ScanDefinition", "ScannerSettings"], vec!["ScanWizardResult", "scan mesh asset", "scanner events"], "server", "Scanner state is event-driven; mesh payloads become asset refs before React sees them."),
        service_contract("mesh-vault-service", "mesh-vault", "rust-core", vec!["importFromPath", "jobStatus", "cancel", "findMesh", "pushMesh", "pullMesh", "pushTexture", "pullTexture"], vec!["local file path", "mesh hash key", "mesh kind", "format", "ttl", "metadata chunks"], vec!["mesh key", "job progress", "blob chunks", "texture count", "GPU upload hints"], "bidirectional", "Large meshes are chunked and keyed by hash; UI passes keys and never clones byte buffers."),
        service_contract("ortho-fipos-service", "ortho-fipos", "python-sidecar", vec!["pushInitialBite", "pushBite", "pullSegmentedBite", "pullFipos", "pushFeedback", "listAssets", "resetPatientData"], vec!["bite id", "creation control", "IPR settings", "restorative objects"], vec!["tooth transforms", "stage count", "interdental distances", "boundary constraints"], "server", "Aligner setup is a job artifact; Three renders staged transforms without recalculating treatment planning."),
        service_contract("bitesplint-bottom-service", "bitesplint-bottom", "rust-core", vec!["createBottom"], vec!["jaw scan mesh key", "insertion axis", "max undercut", "offset", "milling head radius", "closure radius"], vec!["bottom mesh key", "vanilla mesh key", "operation progress"], "server", "Splint bottom generation streams progress and emits mesh keys; no blocking WebGL or React state writes."),
        service_contract("dicom-ai-service", "dicom-ai", "python-sidecar", vec!["sanitize", "metadata", "mprPreview", "segment", "extractMesh", "landmarks"], vec!["dicom series path", "model id", "mask params"], vec!["sanitized series", "preview tiles", "mask", "mesh key", "landmarks"], "server", "DICOM volumes are streamed/tiled; segmentation is offline and cacheable by series hash."),
    ]
}

fn service_contract(
    id: &'static str,
    source_pattern: &'static str,
    owner: &'static str,
    operations: Vec<&'static str>,
    input_refs: Vec<&'static str>,
    output_refs: Vec<&'static str>,
    streaming: &'static str,
    performance_rule: &'static str,
) -> CadServiceContractDto {
    CadServiceContractDto {
        id,
        source_pattern,
        owner,
        operations,
        input_refs,
        output_refs,
        streaming,
        performance_rule,
    }
}

fn allowed_extensions(kind: &str) -> Vec<&'static str> {
    match kind {
        "dicom-series" => vec!["dcm", "dicom", "ima"],
        "obj-mesh" => vec!["obj"],
        "ply-mesh" => vec!["ply"],
        "texture" => vec!["png", "jpg", "jpeg"],
        "photo" => vec!["png", "jpg", "jpeg", "heic"],
        "report" => vec!["pdf", "txt", "json"],
        "manufacturing-export" => vec!["stl", "obj", "ply", "3mf"],
        _ => vec!["stl", "obj", "ply"],
    }
}

fn directory_for_asset(kind: &str) -> &'static str {
    match kind {
        "dicom-series" => "input/dicom",
        "photo" => "input/photos",
        "mask" => "working/masks",
        "scene-snapshot" => "working/scene",
        "report" => "output/reports",
        "manufacturing-export" => "output/cam",
        _ => "input/scans",
    }
}

fn map_job(record: ClinicalJobDto) -> CadJobStatusDto {
    let runtime = serde_json::from_str::<serde_json::Value>(&record.params_json)
        .ok()
        .and_then(|value| {
            value
                .get("runtime")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "tauri-command".to_string());

    CadJobStatusDto {
        id: record.id,
        case_id: record.case_id.unwrap_or_default(),
        kind: record.kind,
        status: record.status,
        progress: record.progress,
        runtime,
        artifacts: Vec::new(),
        error: record.error,
    }
}

fn permission_rules(module_id: &str) -> Vec<(&'static str, &'static str)> {
    match module_id {
        "dicom" => vec![
            ("asset:import_dicom", "asset:import_dicom"),
            ("module:dicom", "module:dicom"),
        ],
        "implant" => vec![
            ("module:implant", "module:implant"),
            ("asset:import_dicom", "asset:import_dicom"),
            ("module:surgical_guide", "module:surgical_guide;role:admin"),
            ("export:manufacturing", "export:manufacturing"),
        ],
        "guide" => vec![
            ("module:surgical_guide", "module:surgical_guide"),
            ("dependency:implant-library", "dependency:implant-library"),
            ("export:manufacturing", "export:manufacturing"),
        ],
        "splint" => vec![
            ("module:splint", "module:splint"),
            ("cad:mesh_edit", "cad:mesh_edit"),
            ("export:manufacturing", "export:manufacturing"),
        ],
        "ceph" => vec![
            ("module:ceph", "module:ceph"),
            ("dependency:python-ai-dicom", "dependency:python-ai-dicom"),
        ],
        "aligners" | "orthocad" => vec![
            ("module:aligner", "module:aligner;module:orthocad"),
            ("dependency:python-ai-dicom", "dependency:python-ai-dicom"),
            ("export:manufacturing", "export:manufacturing"),
        ],
        _ => vec![
            ("module:cad", "module:cad"),
            ("cad:mesh_edit", "cad:mesh_edit"),
            ("export:manufacturing", "export:manufacturing"),
        ],
    }
}

fn evaluate_expression(expression: &str, features: &HashSet<String>) -> bool {
    let trimmed = expression.trim();
    if trimmed.is_empty() {
        return true;
    }

    trimmed
        .split(';')
        .map(str::trim)
        .filter(|clause| !clause.is_empty())
        .any(|clause| {
            clause
                .split(',')
                .map(str::trim)
                .filter(|term| !term.is_empty())
                .all(|term| {
                    let (negated, key) = term
                        .strip_prefix('!')
                        .map(|value| (true, value.trim()))
                        .unwrap_or((false, term));
                    let allowed = features.contains(key);
                    if negated {
                        !allowed
                    } else {
                        allowed
                    }
                })
        })
}

#[tauri::command]
pub fn cad_bootstrap(
    app: AppHandle,
    request: CadBootstrapRequest,
) -> Result<CadBootstrapDto, String> {
    let root = app_data_root(&app)?;
    let _route_context = (
        request.case_id.as_deref().unwrap_or("no-case"),
        request.module_id.as_deref().unwrap_or("cad"),
    );

    Ok(CadBootstrapDto {
        offline_required: true,
        storage: CadStorageBootstrap {
            database: root.join("tlanticad.db").to_string_lossy().to_string(),
            case_root: root.join("cases").to_string_lossy().to_string(),
        },
        tool_registry: tool_registry(),
        dependencies: dependencies(),
        service_contracts: service_contracts(),
        commands: vec![
            "cad_bootstrap",
            "tool_registry_get",
            "case_create",
            "case_open",
            "asset_import_prepare",
            "cad_job_start",
            "cad_job_status",
            "cad_job_cancel",
            "workflow_start",
            "workflow_status",
            "workflow_cancel",
            "module_permissions_get",
            "mesh_vault_import_start",
            "mesh_vault_job_status",
            "mesh_vault_cancel",
            "mesh_vault_find",
        ],
    })
}

#[tauri::command]
pub fn case_open(app: AppHandle, case_id: String) -> Result<CaseOpenResponseDto, String> {
    let layout = ensure_case_layout(&app, &case_id)?;
    let dental_case = case_repository::case_get_graph(app, case_id)?;
    Ok(CaseOpenResponseDto {
        dental_case,
        layout,
    })
}

#[tauri::command]
pub fn asset_import_prepare(
    app: AppHandle,
    request: AssetImportPrepareRequest,
) -> Result<AssetImportPreparationDto, String> {
    let layout = ensure_case_layout(&app, &request.case_id)?;
    let _module_id = request.module_id.as_deref().unwrap_or("cad");
    let target_directory = PathBuf::from(&layout.root).join(directory_for_asset(&request.kind));
    fs::create_dir_all(&target_directory).map_err(|error| {
        format!(
            "Could not create import directory {}: {error}",
            target_directory.display()
        )
    })?;

    let file_name = safe_filename(&request.file_name, "asset.bin");
    let asset_id = safe_segment(&request.asset_id, "asset");
    let target_path = target_directory.join(format!("{asset_id}-{file_name}"));

    Ok(AssetImportPreparationDto {
        case_id: safe_segment(&request.case_id, "case"),
        asset_id,
        target_directory: target_directory.to_string_lossy().to_string(),
        target_path: target_path.to_string_lossy().to_string(),
        manifest_path: layout.manifest_path,
        allowed_extensions: allowed_extensions(&request.kind),
    })
}

#[tauri::command]
pub fn cad_job_start(app: AppHandle, request: CadJobRequestDto) -> Result<CadJobStatusDto, String> {
    let params_json = json!({
        "moduleId": request.module_id,
        "runtime": request.runtime,
        "inputAssetIds": request.input_asset_ids,
        "params": request.params,
    })
    .to_string();

    let record = clinical_jobs::clinical_job_record(
        app,
        ClinicalJobRecordRequest {
            id: Some(Uuid::new_v4().to_string()),
            case_id: Some(request.case_id),
            kind: request.kind,
            status: Some("queued".to_string()),
            progress: Some(0.0),
            vendor: None,
            model_id: None,
            checkpoint_sha256: None,
            params_json: Some(params_json),
            result_json: None,
            error: None,
        },
    )?;

    Ok(map_job(record))
}

#[tauri::command]
pub fn cad_job_status(app: AppHandle, job_id: String) -> Result<CadJobStatusDto, String> {
    clinical_jobs::clinical_job_get(app, job_id).map(map_job)
}

#[tauri::command]
pub fn cad_job_cancel(app: AppHandle, job_id: String) -> Result<CadJobStatusDto, String> {
    clinical_jobs::clinical_job_cancel(app, job_id).map(map_job)
}

#[tauri::command]
pub fn workflow_start(
    app: AppHandle,
    request: WorkflowStartRequestDto,
) -> Result<CadJobStatusDto, String> {
    let runtime = workflow_runtime(&request.workflow_step_id).ok_or_else(|| {
        format!(
            "Unsupported workflow step: {}. Expected one of {}",
            request.workflow_step_id,
            WORKFLOW_STEP_IDS.join(", ")
        )
    })?;

    cad_job_start(
        app,
        CadJobRequestDto {
            case_id: request.case_id,
            module_id: request.module_id,
            kind: format!("workflow.{}", request.workflow_step_id),
            runtime: runtime.to_string(),
            input_asset_ids: request.input_asset_ids,
            params: json!({
                "workflowStepId": request.workflow_step_id,
                "params": request.params,
            }),
        },
    )
}

#[tauri::command]
pub fn workflow_status(app: AppHandle, job_id: String) -> Result<CadJobStatusDto, String> {
    cad_job_status(app, job_id)
}

#[tauri::command]
pub fn workflow_cancel(app: AppHandle, job_id: String) -> Result<CadJobStatusDto, String> {
    cad_job_cancel(app, job_id)
}

#[tauri::command]
pub fn module_permissions_get(
    request: ModulePermissionsRequestDto,
) -> Result<ModulePermissionsResponseDto, String> {
    let mut features: HashSet<String> = request.features.into_iter().collect();
    if let Some(role) = request.role {
        features.insert(format!("role:{role}"));
    }
    for dependency in request.installed_dependencies.unwrap_or_default() {
        features.insert(format!("dependency:{dependency}"));
    }

    let module = module_manifests()
        .into_iter()
        .find(|candidate| candidate.id == request.module_id)
        .unwrap_or_else(|| module_manifests().remove(0));
    features.insert(format!("module:{}", module.id));

    let decisions = permission_rules(module.id)
        .into_iter()
        .map(|(permission, expression)| {
            let allowed = evaluate_expression(expression, &features);
            ModulePermissionDecisionDto {
                permission,
                expression,
                allowed,
                reason: if allowed {
                    "permission expression satisfied locally".to_string()
                } else {
                    "missing local feature, role or dependency".to_string()
                },
            }
        })
        .collect();

    Ok(ModulePermissionsResponseDto { module, decisions })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_expression_supports_or_and_not() {
        let features = HashSet::from([
            "module:implant".to_string(),
            "asset:import_dicom".to_string(),
            "role:designer".to_string(),
        ]);

        assert!(evaluate_expression(
            "module:implant,asset:import_dicom",
            &features
        ));
        assert!(evaluate_expression(
            "module:surgical_guide;module:implant",
            &features
        ));
        assert!(evaluate_expression("module:implant,!role:admin", &features));
        assert!(!evaluate_expression(
            "module:surgical_guide,asset:import_dicom",
            &features
        ));
    }

    #[test]
    fn bootstrap_exposes_complete_core_contracts() {
        let registry = tool_registry();
        assert!(registry.tools.iter().any(|tool| tool.id == "mesh-vault"));
        assert!(registry.modules.iter().any(|module| module.id == "ceph"));
        assert!(registry
            .modules
            .iter()
            .any(|module| module.id == "aligners"));
        assert!(service_contracts()
            .iter()
            .any(|contract| contract.source_pattern == "bitesplint-bottom"));
        assert!(service_contracts()
            .iter()
            .any(|contract| contract.streaming == "bidirectional"));
    }
}
