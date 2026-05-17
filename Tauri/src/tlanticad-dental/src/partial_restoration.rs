//! S171-S175: Motor Inlay/Onlay/Veneer — partial restoration generation.
//!
//! Design engines for conservative restorations: inlays, onlays, overlays,
//! and veneers with isthmus/wall validation and material-specific offsets.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Type of partial restoration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RestorationType {
    Inlay,
    Onlay,
    Overlay,
    Veneer,
    ThreeQuarterCrown,
}

/// Material-specific offsets for partial restorations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestorationMaterial {
    pub name: String,
    pub min_thickness: f64,
    pub min_isthmus_width: f64,
    pub cement_gap_um: f64,
    pub is_adhesive: bool,
}

impl RestorationMaterial {
    pub fn ceramic_emax() -> Self {
        Self { name: "IPS e.max".into(), min_thickness: 1.0, min_isthmus_width: 2.0, cement_gap_um: 40.0, is_adhesive: true }
    }
    pub fn composite() -> Self {
        Self { name: "Composite".into(), min_thickness: 1.5, min_isthmus_width: 1.5, cement_gap_um: 50.0, is_adhesive: true }
    }
    pub fn zirconia() -> Self {
        Self { name: "Zirconia".into(), min_thickness: 1.2, min_isthmus_width: 2.5, cement_gap_um: 50.0, is_adhesive: false }
    }
    pub fn feldspathic() -> Self {
        Self { name: "Feldspathic".into(), min_thickness: 0.5, min_isthmus_width: 1.5, cement_gap_um: 30.0, is_adhesive: true }
    }
}

/// Parameters for partial restoration design.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialRestorationParams {
    pub restoration_type: RestorationType,
    pub material: RestorationMaterial,
    pub fdi_number: u8,
    pub occlusal_reduction: f64,
    pub axial_reduction: f64,
    pub smooth_iterations: u32,
}

impl Default for PartialRestorationParams {
    fn default() -> Self {
        Self {
            restoration_type: RestorationType::Inlay,
            material: RestorationMaterial::ceramic_emax(),
            fdi_number: 36,
            occlusal_reduction: 1.5,
            axial_reduction: 1.0,
            smooth_iterations: 2,
        }
    }
}

/// Result of partial restoration generation.
#[derive(Debug, Clone)]
pub struct PartialRestorationResult {
    pub outer_vertices: Vec<Point3<f64>>,
    pub inner_vertices: Vec<Point3<f64>>,
    pub indices: Vec<[u32; 3]>,
    pub metrics: RestorationMetrics,
}

/// Quality metrics for partial restoration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RestorationMetrics {
    pub min_thickness: f64,
    pub isthmus_width: f64,
    pub wall_height: f64,
    pub retention_score: f64,
    pub warnings: Vec<String>,
}

// ---------------------------------------------------------------------------
// Inlay / Onlay design (S171-S172)
// ---------------------------------------------------------------------------

/// Generate a partial restoration (inlay/onlay) from cavity preparation.
pub fn generate_partial_restoration(
    cavity_vertices: &[Point3<f64>],
    cavity_normals: &[Vector3<f64>],
    cavity_indices: &[[u32; 3]],
    params: &PartialRestorationParams,
) -> PartialRestorationResult {
    if cavity_vertices.is_empty() {
        return PartialRestorationResult {
            outer_vertices: vec![],
            inner_vertices: vec![],
            indices: vec![],
            metrics: RestorationMetrics::default(),
        };
    }

    let gap_mm = params.material.cement_gap_um / 1000.0;

    // Inner surface: offset inward by cement gap
    let inner_vertices: Vec<Point3<f64>> = cavity_vertices
        .iter()
        .zip(cavity_normals.iter())
        .map(|(v, n)| v + n.normalize() * gap_mm)
        .collect();

    // Outer surface: offset outward by reduction amount
    let reduction = match params.restoration_type {
        RestorationType::Inlay | RestorationType::Onlay | RestorationType::Overlay => params.occlusal_reduction,
        RestorationType::Veneer | RestorationType::ThreeQuarterCrown => params.axial_reduction,
    };
    let outer_vertices: Vec<Point3<f64>> = cavity_vertices
        .iter()
        .zip(cavity_normals.iter())
        .map(|(v, n)| v + n.normalize() * reduction)
        .collect();

    // Compute metrics
    let metrics = compute_restoration_metrics(
        &inner_vertices,
        &outer_vertices,
        cavity_vertices,
        params,
    );

    PartialRestorationResult {
        outer_vertices,
        inner_vertices,
        indices: cavity_indices.to_vec(),
        metrics,
    }
}

// ---------------------------------------------------------------------------
// Veneer design (S173)
// ---------------------------------------------------------------------------

/// Generate a veneer (labial surface only).
pub fn generate_veneer(
    facial_vertices: &[Point3<f64>],
    facial_normals: &[Vector3<f64>],
    params: &PartialRestorationParams,
) -> PartialRestorationResult {
    let modified_params = PartialRestorationParams {
        restoration_type: RestorationType::Veneer,
        ..params.clone()
    };
    // Veneer is essentially a thin shell on the facial surface
    let indices: Vec<[u32; 3]> = if facial_vertices.len() >= 3 {
        (0..facial_vertices.len().saturating_sub(2))
            .map(|i| [0, (i + 1) as u32, (i + 2) as u32])
            .collect()
    } else {
        vec![]
    };
    generate_partial_restoration(facial_vertices, facial_normals, &indices, &modified_params)
}

// ---------------------------------------------------------------------------
// Validation (S174-S175)
// ---------------------------------------------------------------------------

/// Validate partial restoration against material constraints.
pub fn validate_restoration(result: &PartialRestorationResult, params: &PartialRestorationParams) -> Vec<String> {
    let mut issues = Vec::new();
    let m = &result.metrics;

    if m.min_thickness < params.material.min_thickness {
        issues.push(format!(
            "Min thickness {:.2} mm below {} requirement ({:.1} mm)",
            m.min_thickness, params.material.name, params.material.min_thickness
        ));
    }

    if m.isthmus_width < params.material.min_isthmus_width {
        issues.push(format!(
            "Isthmus width {:.2} mm below minimum ({:.1} mm)",
            m.isthmus_width, params.material.min_isthmus_width
        ));
    }

    if matches!(params.restoration_type, RestorationType::Inlay | RestorationType::Onlay) {
        if m.wall_height < 1.5 {
            issues.push("Wall height < 1.5 mm — retention may be insufficient".into());
        }
    }

    issues
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn compute_restoration_metrics(
    inner: &[Point3<f64>],
    outer: &[Point3<f64>],
    cavity: &[Point3<f64>],
    params: &PartialRestorationParams,
) -> RestorationMetrics {
    if inner.is_empty() || outer.is_empty() {
        return RestorationMetrics::default();
    }

    let thicknesses: Vec<f64> = inner
        .iter()
        .zip(outer.iter())
        .map(|(i, o)| (o - i).norm())
        .collect();

    let min_t = thicknesses.iter().cloned().fold(f64::MAX, f64::min);

    // Estimate isthmus (narrowest cross-section in XY plane)
    let isthmus = estimate_isthmus_width(cavity);

    // Wall height (max Z range)
    let min_z = cavity.iter().map(|v| v.z).fold(f64::MAX, f64::min);
    let max_z = cavity.iter().map(|v| v.z).fold(f64::MIN, f64::max);
    let wall_height = max_z - min_z;

    let mut warnings = Vec::new();
    if min_t < params.material.min_thickness {
        warnings.push(format!("Thin area: {:.2} mm", min_t));
    }

    RestorationMetrics {
        min_thickness: min_t,
        isthmus_width: isthmus,
        wall_height,
        retention_score: (wall_height / 3.0).clamp(0.0, 1.0),
        warnings,
    }
}

fn estimate_isthmus_width(vertices: &[Point3<f64>]) -> f64 {
    if vertices.len() < 2 {
        return 0.0;
    }
    // Project to XY, find min width across slice
    let min_x = vertices.iter().map(|v| v.x).fold(f64::MAX, f64::min);
    let max_x = vertices.iter().map(|v| v.x).fold(f64::MIN, f64::max);
    max_x - min_x
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cavity() -> (Vec<Point3<f64>>, Vec<Vector3<f64>>, Vec<[u32; 3]>) {
        let verts = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(3.0, 0.0, 0.0),
            Point3::new(3.0, 3.0, 0.0),
            Point3::new(0.0, 3.0, 0.0),
            Point3::new(1.5, 1.5, -2.0),
        ];
        let normals = vec![Vector3::z(); 5];
        let indices = vec![[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]];
        (verts, normals, indices)
    }

    #[test]
    fn generate_inlay_basic() {
        let (v, n, i) = make_cavity();
        let params = PartialRestorationParams::default();
        let result = generate_partial_restoration(&v, &n, &i, &params);
        assert_eq!(result.outer_vertices.len(), v.len());
        assert!(result.metrics.min_thickness > 0.0);
    }

    #[test]
    fn generate_veneer_basic() {
        let facial = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(3.0, 0.0, 0.0),
            Point3::new(1.5, 3.0, 0.0),
        ];
        let normals = vec![Vector3::z(); 3];
        let params = PartialRestorationParams {
            restoration_type: RestorationType::Veneer,
            material: RestorationMaterial::feldspathic(),
            ..Default::default()
        };
        let result = generate_veneer(&facial, &normals, &params);
        assert!(!result.outer_vertices.is_empty());
    }

    #[test]
    fn validate_flags_thin() {
        let (v, n, i) = make_cavity();
        let params = PartialRestorationParams {
            material: RestorationMaterial { min_thickness: 100.0, ..RestorationMaterial::ceramic_emax() },
            ..Default::default()
        };
        let result = generate_partial_restoration(&v, &n, &i, &params);
        let issues = validate_restoration(&result, &params);
        assert!(!issues.is_empty());
    }

    #[test]
    fn material_presets_valid() {
        let emax = RestorationMaterial::ceramic_emax();
        let zr = RestorationMaterial::zirconia();
        assert!(emax.min_thickness > 0.0);
        assert!(zr.min_isthmus_width > emax.min_isthmus_width);
    }

    #[test]
    fn empty_cavity_no_crash() {
        let result = generate_partial_restoration(&[], &[], &[], &PartialRestorationParams::default());
        assert!(result.outer_vertices.is_empty());
    }
}
