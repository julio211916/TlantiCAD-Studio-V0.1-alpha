// AR-V373 — Implant manager (Tauri command surface).
//
// Four commands:
//   * `cad_implant_change_type`         — replace SKU preserving position+axis.
//   * `cad_implant_delete`              — remove implant + dependent reconstructions.
//   * `cad_implant_define_references`   — mark FDIs as planning-references-only.
//   * `cad_implant_validate_placement`  — collision + axis divergence warnings.

use serde::{Deserialize, Serialize};
use tlanticad_implant::manager::{
    change_implant_type, define_reference_objects, delete_implant, validate_proposed_placement,
    ImplantPlacement, ImplantPlanningState, ManagerWarning,
};

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ImplantManagerError {
    #[error("implant for FDI {fdi} not found")]
    NotFound { fdi: u8 },
    #[error("invalid request: {message}")]
    Invalid { message: String },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeTypeRequest {
    pub state: ImplantPlanningState,
    pub fdi: u8,
    pub new_sku: String,
    #[serde(default = "default_invalidates")]
    pub invalidates_reconstructions: bool,
}

fn default_invalidates() -> bool {
    true
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeTypeResponse {
    pub state: ImplantPlanningState,
    pub orphaned_reconstructions: Vec<String>,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_implant_change_type(
    request: ChangeTypeRequest,
) -> Result<ChangeTypeResponse, ImplantManagerError> {
    let mut state = request.state;
    let orphaned = change_implant_type(
        &mut state,
        request.fdi,
        request.new_sku,
        request.invalidates_reconstructions,
    )
    .map_err(|_| ImplantManagerError::NotFound { fdi: request.fdi })?;
    Ok(ChangeTypeResponse {
        state,
        orphaned_reconstructions: orphaned,
        backend: "tlanticad-implant::manager",
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRequest {
    pub state: ImplantPlanningState,
    pub fdi: u8,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteResponse {
    pub state: ImplantPlanningState,
    pub orphaned_reconstructions: Vec<String>,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_implant_delete(request: DeleteRequest) -> Result<DeleteResponse, ImplantManagerError> {
    let mut state = request.state;
    let orphaned = delete_implant(&mut state, request.fdi)
        .map_err(|_| ImplantManagerError::NotFound { fdi: request.fdi })?;
    Ok(DeleteResponse {
        state,
        orphaned_reconstructions: orphaned,
        backend: "tlanticad-implant::manager",
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefineReferencesRequest {
    pub state: ImplantPlanningState,
    pub fdis: Vec<u8>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DefineReferencesResponse {
    pub state: ImplantPlanningState,
    pub warnings: Vec<ManagerWarning>,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_implant_define_references(request: DefineReferencesRequest) -> DefineReferencesResponse {
    let mut state = request.state;
    let warnings = define_reference_objects(&mut state, &request.fdis);
    DefineReferencesResponse {
        state,
        warnings,
        backend: "tlanticad-implant::manager",
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatePlacementRequest {
    pub state: ImplantPlanningState,
    pub proposal: ImplantPlacement,
    #[serde(default = "default_radius")]
    pub proposed_radius_mm: f64,
    #[serde(default = "default_radius")]
    pub existing_radius_mm: f64,
}

fn default_radius() -> f64 {
    2.0
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatePlacementResponse {
    pub warnings: Vec<ManagerWarning>,
    pub backend: &'static str,
}

#[tauri::command]
pub fn cad_implant_validate_placement(
    request: ValidatePlacementRequest,
) -> ValidatePlacementResponse {
    let warnings = validate_proposed_placement(
        &request.state,
        &request.proposal,
        request.proposed_radius_mm,
        request.existing_radius_mm,
    );
    ValidatePlacementResponse {
        warnings,
        backend: "tlanticad-implant::manager",
    }
}
