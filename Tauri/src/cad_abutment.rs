use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tlanticad_abutment::motor::{
    generate_custom_abutment_mesh, plan_screw_channel, AbutmentProfilePreset,
    CustomAbutmentMeshRequest, ScrewChannelPlan,
};
use tlanticad_mesh::save_stl;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbutmentGenerateMeshRequest {
    pub case_folder_path: String,
    pub output_file_name: Option<String>,
    pub margin_polyline: Vec<[f64; 3]>,
    pub implant_axis: [f64; 3],
    pub implant_diameter_mm: f64,
    pub emergence_height_mm: f64,
    pub shoulder_width_mm: f64,
    pub taper_degrees: f64,
    pub axial_rings: Option<usize>,
    pub profile: Option<AbutmentProfilePreset>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbutmentGenerateMeshResponse {
    pub output_path: String,
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub watertight: bool,
    pub volume_mm3: f64,
    pub warnings: Vec<String>,
    pub backend: &'static str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbutmentScrewChannelRequest {
    pub implant_position: [f64; 3],
    pub implant_axis: [f64; 3],
    pub prosthetic_axis: [f64; 3],
    pub length_mm: f64,
    pub diameter_mm: f64,
    pub library_angle_limit_deg: f64,
}

#[tauri::command]
pub fn abutment_generate_mesh(
    _app: AppHandle,
    request: AbutmentGenerateMeshRequest,
) -> Result<AbutmentGenerateMeshResponse, String> {
    let case_folder = PathBuf::from(&request.case_folder_path);
    if !case_folder.exists() {
        return Err(format!(
            "case folder does not exist: {}",
            request.case_folder_path
        ));
    }
    if !case_folder.is_dir() {
        return Err(format!(
            "case folder is not a directory: {}",
            request.case_folder_path
        ));
    }

    let output_dir = case_folder.join("work").join("implants").join("abutments");
    std::fs::create_dir_all(&output_dir).map_err(|error| error.to_string())?;
    let output_name = safe_output_name(
        request
            .output_file_name
            .as_deref()
            .unwrap_or("custom-abutment.stl"),
    );
    let output_path = output_dir.join(output_name);

    let engine_request = CustomAbutmentMeshRequest {
        margin_polyline: request.margin_polyline,
        implant_axis: request.implant_axis,
        implant_diameter_mm: request.implant_diameter_mm,
        emergence_height_mm: request.emergence_height_mm,
        shoulder_width_mm: request.shoulder_width_mm,
        taper_degrees: request.taper_degrees,
        axial_rings: request.axial_rings.unwrap_or(14),
        profile: request.profile.unwrap_or(AbutmentProfilePreset::Default),
    };

    let (mesh, qa) = generate_custom_abutment_mesh(&engine_request)?;
    save_stl(&mesh, Path::new(&output_path)).map_err(|error| error.to_string())?;

    Ok(AbutmentGenerateMeshResponse {
        output_path: output_path.to_string_lossy().to_string(),
        vertex_count: qa.vertex_count,
        triangle_count: qa.triangle_count,
        watertight: qa.watertight,
        volume_mm3: qa.signed_volume_mm3,
        warnings: qa.warnings,
        backend: "rust:tlanticad-abutment",
    })
}

#[tauri::command]
pub fn abutment_plan_screw_channel(
    request: AbutmentScrewChannelRequest,
) -> Result<ScrewChannelPlan, String> {
    if request.length_mm <= 0.0 {
        return Err("lengthMm must be positive".to_string());
    }
    if request.diameter_mm <= 0.0 {
        return Err("diameterMm must be positive".to_string());
    }
    Ok(plan_screw_channel(
        request.implant_position,
        request.implant_axis,
        request.prosthetic_axis,
        request.length_mm,
        request.diameter_mm,
        request.library_angle_limit_deg,
    ))
}

fn safe_output_name(input: &str) -> String {
    let mut safe = input
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => ch,
            _ => '-',
        })
        .collect::<String>();
    if !safe.to_lowercase().ends_with(".stl") {
        safe.push_str(".stl");
    }
    if safe == ".stl" {
        "custom-abutment.stl".to_string()
    } else {
        safe
    }
}
