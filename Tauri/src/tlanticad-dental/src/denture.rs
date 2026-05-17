//! S176-S180: Motor Wax-up & Dentures — digital wax-up and complete denture design.
//!
//! Tooth setup on virtual wax rim, balanced occlusion, denture base design,
//! and tooth arrangement with Bonwill triangle compliance.

use nalgebra::Point3;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Denture type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DentureType {
    CompleteUpper,
    CompleteLower,
    PartialUpper,
    PartialLower,
}

/// Tooth setup position on the wax rim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToothSetup {
    pub fdi_number: u8,
    pub position: [f64; 3],
    pub rotation_degrees: [f64; 3],
    pub tilt_labial: f64,
    pub tilt_mesial: f64,
}

/// Wax rim parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaxRimParams {
    pub rim_height: f64,
    pub rim_width: f64,
    pub arch_form: ArchForm,
    pub occlusal_plane_height: f64,
}

impl Default for WaxRimParams {
    fn default() -> Self {
        Self {
            rim_height: 22.0,
            rim_width: 8.0,
            arch_form: ArchForm::Ovoid,
            occlusal_plane_height: 70.0,
        }
    }
}

/// Arch form classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArchForm {
    Ovoid,
    Square,
    Tapered,
}

/// Denture design parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DentureParams {
    pub denture_type: DentureType,
    pub wax_rim: WaxRimParams,
    pub tooth_set_id: String,
    pub base_thickness: f64,
    pub post_dam_depth: f64,
    pub relief_depth: f64,
}

impl Default for DentureParams {
    fn default() -> Self {
        Self {
            denture_type: DentureType::CompleteUpper,
            wax_rim: WaxRimParams::default(),
            tooth_set_id: "standard-33".into(),
            base_thickness: 2.0,
            post_dam_depth: 1.0,
            relief_depth: 0.3,
        }
    }
}

/// Generated denture result.
#[derive(Debug, Clone)]
pub struct DentureResult {
    pub base_vertices: Vec<Point3<f64>>,
    pub base_indices: Vec<[u32; 3]>,
    pub tooth_setups: Vec<ToothSetup>,
    pub metrics: DentureMetrics,
}

/// Denture quality metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DentureMetrics {
    pub occlusal_plane_deviation: f64,
    pub bilateral_balance_score: f64,
    pub bonwill_triangle_deviation: f64,
    pub tooth_count: usize,
    pub warnings: Vec<String>,
}

// ---------------------------------------------------------------------------
// Wax-up tooth arrangement (S176-S177)
// ---------------------------------------------------------------------------

/// Arrange teeth on the wax rim following arch form.
pub fn arrange_teeth_on_rim(
    arch_curve: &[Point3<f64>],
    params: &DentureParams,
) -> Vec<ToothSetup> {
    // Determine teeth to place based on denture type
    let tooth_range = match params.denture_type {
        DentureType::CompleteUpper => (11u8..=18).chain(21..=28).collect::<Vec<u8>>(),
        DentureType::CompleteLower => (31u8..=38).chain(41..=48).collect::<Vec<u8>>(),
        DentureType::PartialUpper => (11u8..=18).chain(21..=28).collect::<Vec<u8>>(),
        DentureType::PartialLower => (31u8..=38).chain(41..=48).collect::<Vec<u8>>(),
    };

    let n_teeth = tooth_range.len();
    tooth_range
        .into_iter()
        .enumerate()
        .map(|(i, fdi)| {
            let t = i as f64 / n_teeth as f64;
            let pos = interpolate_arch(arch_curve, t);
            ToothSetup {
                fdi_number: fdi,
                position: [pos.x, pos.y, pos.z],
                rotation_degrees: [0.0, 0.0, 0.0],
                tilt_labial: default_labial_tilt(fdi),
                tilt_mesial: 0.0,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Denture base (S178-S179)
// ---------------------------------------------------------------------------

/// Generate a denture base from the master cast.
pub fn generate_denture_base(
    cast_vertices: &[Point3<f64>],
    cast_indices: &[[u32; 3]],
    params: &DentureParams,
) -> (Vec<Point3<f64>>, Vec<[u32; 3]>) {
    if cast_vertices.is_empty() {
        return (vec![], vec![]);
    }

    // Offset tissue surface outward by base thickness
    let base_verts: Vec<Point3<f64>> = cast_vertices
        .iter()
        .map(|v| Point3::new(v.x, v.y, v.z - params.base_thickness))
        .collect();

    (base_verts, cast_indices.to_vec())
}

// ---------------------------------------------------------------------------
// Occlusal scheme validation (S180)
// ---------------------------------------------------------------------------

/// Validate the denture design for balanced occlusion.
pub fn validate_denture(result: &DentureResult, params: &DentureParams) -> Vec<String> {
    let mut issues = Vec::new();
    let m = &result.metrics;

    if m.occlusal_plane_deviation > 2.0 {
        issues.push(format!(
            "Occlusal plane deviation {:.1} mm exceeds tolerance",
            m.occlusal_plane_deviation
        ));
    }

    if m.bilateral_balance_score < 0.7 {
        issues.push("Bilateral balance insufficient — check contact distribution".into());
    }

    if m.bonwill_triangle_deviation > 5.0 {
        issues.push(format!(
            "Bonwill triangle deviation {:.1} mm — recheck condylar distance",
            m.bonwill_triangle_deviation
        ));
    }

    if result.tooth_setups.len() < 14 && matches!(params.denture_type, DentureType::CompleteUpper | DentureType::CompleteLower) {
        issues.push(format!("Only {} teeth set — expected at least 14", result.tooth_setups.len()));
    }

    issues
}

/// Compute Bonwill triangle compliance.
pub fn bonwill_triangle_check(
    condylar_distance: f64,
    incisal_point: &Point3<f64>,
    left_condyle: &Point3<f64>,
    right_condyle: &Point3<f64>,
) -> f64 {
    let expected_side = condylar_distance;
    let a = (incisal_point - left_condyle).norm();
    let b = (incisal_point - right_condyle).norm();
    let c = (left_condyle - right_condyle).norm();
    // Deviation from equilateral triangle where all sides = condylar distance
    ((a - expected_side).abs() + (b - expected_side).abs() + (c - expected_side).abs()) / 3.0
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn interpolate_arch(curve: &[Point3<f64>], t: f64) -> Point3<f64> {
    if curve.is_empty() {
        return Point3::origin();
    }
    if curve.len() == 1 {
        return curve[0];
    }
    let n = curve.len() - 1;
    let seg = (t * n as f64).floor() as usize;
    let seg = seg.min(n - 1);
    let local_t = t * n as f64 - seg as f64;
    Point3::from(
        curve[seg].coords * (1.0 - local_t) + curve[seg + 1].coords * local_t,
    )
}

fn default_labial_tilt(fdi: u8) -> f64 {
    let tooth = fdi % 10;
    match tooth {
        1 | 2 => 5.0,   // incisors: slight labial tilt
        3 => 3.0,        // canines
        4 | 5 => 0.0,    // premolars
        _ => -2.0,       // molars: slight lingual
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_arch() -> Vec<Point3<f64>> {
        (0..20)
            .map(|i| {
                let angle = std::f64::consts::PI * i as f64 / 19.0;
                Point3::new(25.0 * angle.cos(), 25.0 * angle.sin(), 0.0)
            })
            .collect()
    }

    #[test]
    fn arrange_teeth_complete_upper() {
        let arch = make_arch();
        let params = DentureParams::default();
        let setups = arrange_teeth_on_rim(&arch, &params);
        assert_eq!(setups.len(), 16); // 8 per side
    }

    #[test]
    fn denture_base_offsets() {
        let verts = vec![Point3::new(0.0, 0.0, 0.0)];
        let indices = vec![];
        let params = DentureParams { base_thickness: 3.0, ..Default::default() };
        let (base, _) = generate_denture_base(&verts, &indices, &params);
        assert_eq!(base[0].z, -3.0);
    }

    #[test]
    fn bonwill_equilateral() {
        let cd = 100.0;
        let incisal = Point3::new(0.0, 86.6, 0.0); // equilateral height ≈ 86.6
        let left = Point3::new(-50.0, 0.0, 0.0);
        let right = Point3::new(50.0, 0.0, 0.0);
        let dev = bonwill_triangle_check(cd, &incisal, &left, &right);
        assert!(dev < 1.0, "Equilateral triangle deviation should be small: {}", dev);
    }

    #[test]
    fn validate_flags_missing_teeth() {
        let result = DentureResult {
            base_vertices: vec![],
            base_indices: vec![],
            tooth_setups: vec![ToothSetup {
                fdi_number: 11,
                position: [0.0; 3],
                rotation_degrees: [0.0; 3],
                tilt_labial: 0.0,
                tilt_mesial: 0.0,
            }],
            metrics: DentureMetrics {
                bilateral_balance_score: 0.9,
                ..Default::default()
            },
        };
        let issues = validate_denture(&result, &DentureParams::default());
        assert!(issues.iter().any(|i| i.contains("Only 1 teeth")));
    }

    #[test]
    fn empty_arch_no_crash() {
        let setups = arrange_teeth_on_rim(&[], &DentureParams::default());
        assert_eq!(setups.len(), 16); // Still generates with origin positions
    }
}
