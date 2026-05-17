use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CadShellBootstrapRequest {
    pub case_id: Option<String>,
    pub module_id: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CadShellCapabilities {
    pub rust_mesh_ops: bool,
    pub python_ai: bool,
    pub dicom_pipeline: bool,
    pub export_pipeline: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CadExtensionPoint {
    pub id: &'static str,
    pub layer: &'static str,
    pub label: &'static str,
    pub status: &'static str,
    pub notes: &'static str,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CadShellBootstrap {
    pub route: String,
    pub offline_required: bool,
    pub capabilities: CadShellCapabilities,
    pub extension_points: Vec<CadExtensionPoint>,
}

fn build_cad_shell_bootstrap(request: &CadShellBootstrapRequest) -> CadShellBootstrap {
    let route = format!(
        "cad-shell/bootstrap/{}",
        request
            .module_id
            .as_deref()
            .filter(|value| !value.is_empty())
            .unwrap_or("cad")
    );

    CadShellBootstrap {
        route,
        offline_required: true,
        capabilities: CadShellCapabilities {
            rust_mesh_ops: true,
            python_ai: true,
            dicom_pipeline: true,
            export_pipeline: true,
        },
        extension_points: vec![
            CadExtensionPoint {
                id: "react-cad-shell",
                layer: "react",
                label: "React CAD UI",
                status: "ready",
                notes: "Owns UI and tool controls only; no mesh compute.",
            },
            CadExtensionPoint {
                id: "three-viewport",
                layer: "three",
                label: "Three.js viewport",
                status: "ready",
                notes: "Owns rendering, camera controls, GPU buffers and disposal lifecycle.",
            },
            CadExtensionPoint {
                id: "tauri-orchestrator",
                layer: "tauri",
                label: "Tauri orchestration",
                status: "ready",
                notes: "Routes UI requests to local Rust/Python services without owning compute.",
            },
            CadExtensionPoint {
                id: "rust-mesh-core",
                layer: "rust",
                label: "Rust mesh core",
                status: "ready",
                notes: "Extension point for CSG, STL/OBJ IO, validation, export and future OCCT.",
            },
            CadExtensionPoint {
                id: "python-ai-dicom",
                layer: "python",
                label: "Python AI/DICOM",
                status: "ready",
                notes: "Extension point for offline ONNX/TorchScript inference and DICOM post-processing.",
            },
        ],
    }
}

#[tauri::command]
pub fn cad_shell_bootstrap(request: CadShellBootstrapRequest) -> Result<CadShellBootstrap, String> {
    let _case_id = request.case_id.as_deref().unwrap_or("no-case");
    Ok(build_cad_shell_bootstrap(&request))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cad_shell_bootstrap_is_offline_and_exposes_local_extension_points() {
        let bootstrap = build_cad_shell_bootstrap(&CadShellBootstrapRequest {
            case_id: Some("case-1".into()),
            module_id: Some("crown".into()),
        });

        assert_eq!(bootstrap.route, "cad-shell/bootstrap/crown");
        assert!(bootstrap.offline_required);
        assert!(bootstrap.capabilities.rust_mesh_ops);
        assert!(bootstrap.capabilities.python_ai);
        assert!(bootstrap.capabilities.dicom_pipeline);
        assert!(bootstrap.capabilities.export_pipeline);
        assert!(bootstrap
            .extension_points
            .iter()
            .any(|point| point.layer == "rust"));
        assert!(bootstrap
            .extension_points
            .iter()
            .any(|point| point.layer == "python"));
        assert!(bootstrap
            .extension_points
            .iter()
            .any(|point| point.layer == "tauri"));
    }
}
