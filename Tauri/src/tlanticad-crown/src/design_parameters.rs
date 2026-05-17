//! Crown design parameters with validation

use serde::{Deserialize, Serialize};
use crate::adaptation::CementGapConfig;

/// Complete set of design parameters for a single crown unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrownDesignParams {
    pub tooth_number: u8,
    pub work_type: String,
    pub material: String,
    pub shade: String,
    pub adaptation: CementGapConfig,
    /// Minimum occlusal wall thickness in mm
    pub occlusal_thickness: f64,
    /// Minimum axial wall thickness in mm
    pub wall_thickness: f64,
    /// Emergence profile angle in degrees
    pub emergency_profile: f64,
    /// Finish line fillet radius in mm
    pub finish_line_radius: f64,
}

impl Default for CrownDesignParams {
    fn default() -> Self {
        Self {
            tooth_number: 0,
            work_type: "CrownAnatomic".into(),
            material: "Zirconia".into(),
            shade: "A2".into(),
            adaptation: CementGapConfig::default(),
            occlusal_thickness: 1.5,
            wall_thickness: 0.8,
            emergency_profile: 15.0,
            finish_line_radius: 0.3,
        }
    }
}

/// Validate design parameters and return a list of human-readable error messages.
///
/// An empty return value means all parameters pass validation.
pub fn validate_parameters(params: &CrownDesignParams) -> Vec<String> {
    let mut errors = Vec::new();

    if params.tooth_number == 0 || params.tooth_number > 48 {
        errors.push(format!(
            "Invalid tooth number {}; must be 1–48 (FDI notation)",
            params.tooth_number
        ));
    }

    if params.work_type.is_empty() {
        errors.push("Work type must not be empty".into());
    }

    if params.material.is_empty() {
        errors.push("Material must not be empty".into());
    }

    let min_wall = crate::material_space::minimum_thickness_for_material(&params.material);
    if params.wall_thickness < min_wall {
        errors.push(format!(
            "Wall thickness {:.2} mm is below the minimum {:.2} mm for {}",
            params.wall_thickness, min_wall, params.material
        ));
    }

    if params.occlusal_thickness < min_wall {
        errors.push(format!(
            "Occlusal thickness {:.2} mm is below the minimum {:.2} mm for {}",
            params.occlusal_thickness, min_wall, params.material
        ));
    }

    if params.adaptation.margin_gap < 10.0 || params.adaptation.margin_gap > 200.0 {
        errors.push(format!(
            "Margin gap {:.0} µm is outside the clinical range (10–200 µm)",
            params.adaptation.margin_gap
        ));
    }

    if params.emergency_profile < 0.0 || params.emergency_profile > 60.0 {
        errors.push(format!(
            "Emergency profile {:.1}° is outside the valid range (0–60°)",
            params.emergency_profile
        ));
    }

    if params.finish_line_radius < 0.0 || params.finish_line_radius > 1.0 {
        errors.push(format!(
            "Finish line radius {:.2} mm is outside the valid range (0–1 mm)",
            params.finish_line_radius
        ));
    }

    errors
}
