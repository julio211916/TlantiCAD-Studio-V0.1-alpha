//! Manufacturing constraint validation
//!
//! Checks dental restorations meet manufacturing requirements for
//! milling (CNC) and 3D printing.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};

/// Manufacturing method
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ManufacturingMethod {
    /// CNC milling (3-axis, 4-axis, 5-axis)
    Milling { axes: u8 },
    /// 3D printing (SLA, DLP, SLS)
    Printing,
    /// Casting / pressing
    Casting,
}

/// Manufacturing material
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub name: String,
    pub min_thickness: f64,       // mm
    pub min_connector_area: f64,  // mm²
    pub min_radius: f64,          // mm (inner radius)
    pub max_overhang_angle: f64,  // degrees (for printing)
    pub shrinkage_factor: f64,    // % shrinkage during processing
}

impl Material {
    pub fn zirconia() -> Self {
        Self {
            name: "Zirconia".into(),
            min_thickness: 0.5,
            min_connector_area: 6.0,
            min_radius: 0.3,
            max_overhang_angle: 45.0,
            shrinkage_factor: 20.0,
        }
    }

    pub fn emax() -> Self {
        Self {
            name: "IPS e.max".into(),
            min_thickness: 0.8,
            min_connector_area: 8.0,
            min_radius: 0.4,
            max_overhang_angle: 45.0,
            shrinkage_factor: 0.2,
        }
    }

    pub fn pmma() -> Self {
        Self {
            name: "PMMA".into(),
            min_thickness: 1.0,
            min_connector_area: 4.0,
            min_radius: 0.3,
            max_overhang_angle: 60.0,
            shrinkage_factor: 0.1,
        }
    }

    pub fn cobalt_chrome() -> Self {
        Self {
            name: "CoCr".into(),
            min_thickness: 0.3,
            min_connector_area: 4.0,
            min_radius: 0.2,
            max_overhang_angle: 45.0,
            shrinkage_factor: 1.5,
        }
    }

    pub fn titanium() -> Self {
        Self {
            name: "Titanium".into(),
            min_thickness: 0.4,
            min_connector_area: 3.0,
            min_radius: 0.2,
            max_overhang_angle: 40.0,
            shrinkage_factor: 0.0,
        }
    }
}

/// A manufacturing constraint violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub rule: String,
    pub severity: Severity,
    pub location: [f64; 3],
    pub detail: String,
    pub affected_vertices: Vec<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Severity {
    Warning,
    Error,
    Critical,
}

/// Manufacturing validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturingReport {
    pub method: String,
    pub material: String,
    pub violations: Vec<Violation>,
    pub is_manufacturable: bool,
}

/// Validate a restoration mesh against manufacturing constraints
pub fn validate_manufacturing(
    vertices: &[Point3<f64>],
    normals: &[Vector3<f64>],
    indices: &[[u32; 3]],
    thicknesses: &[f64],
    method: ManufacturingMethod,
    material: &Material,
) -> ManufacturingReport {
    let mut violations = Vec::new();

    // Rule 1: Minimum thickness
    for (vi, &t) in thicknesses.iter().enumerate() {
        if t > 0.0 && t < material.min_thickness {
            violations.push(Violation {
                rule: "Minimum thickness".into(),
                severity: if t < material.min_thickness * 0.5 { Severity::Critical } else { Severity::Error },
                location: point_to_arr(&vertices[vi]),
                detail: format!("{:.2}mm < {:.2}mm minimum", t, material.min_thickness),
                affected_vertices: vec![vi as u32],
            });
        }
    }

    // Rule 2: Sharp internal angles (stress concentrators)
    check_sharp_angles(vertices, indices, material.min_radius, &mut violations);

    // Rule 3: Method-specific checks
    match method {
        ManufacturingMethod::Milling { axes } => {
            check_milling_accessibility(vertices, normals, indices, axes, &mut violations);
        }
        ManufacturingMethod::Printing => {
            check_print_overhangs(vertices, normals, indices, material.max_overhang_angle, &mut violations);
        }
        ManufacturingMethod::Casting => {
            // Casting checks: undercuts, sprue access
        }
    }

    let has_critical = violations.iter().any(|v| matches!(v.severity, Severity::Critical));

    ManufacturingReport {
        method: format!("{:?}", method),
        material: material.name.clone(),
        is_manufacturable: !has_critical,
        violations,
    }
}

/// Apply shrinkage compensation to the restoration mesh
pub fn apply_shrinkage_compensation(
    vertices: &mut [Point3<f64>],
    _normals: &[Vector3<f64>],
    shrinkage_percent: f64,
) {
    let factor = 1.0 + shrinkage_percent / 100.0;
    // Compute centroid
    let centroid = vertices.iter().fold(Vector3::zeros(), |acc, v| acc + v.coords) / vertices.len() as f64;

    for v in vertices.iter_mut() {
        let offset = v.coords - centroid;
        v.coords = centroid + offset * factor;
    }
}

fn check_sharp_angles(
    vertices: &[Point3<f64>],
    indices: &[[u32; 3]],
    min_radius: f64,
    violations: &mut Vec<Violation>,
) {
    for tri in indices {
        for k in 0..3 {
            let a = &vertices[tri[k] as usize];
            let b = &vertices[tri[(k + 1) % 3] as usize];
            let c = &vertices[tri[(k + 2) % 3] as usize];

            let e1 = (b - a).normalize();
            let e2 = (c - a).normalize();
            let angle = e1.dot(&e2).clamp(-1.0, 1.0).acos();

            if angle < min_radius * 2.0 { // Very acute angle
                violations.push(Violation {
                    rule: "Sharp internal angle".into(),
                    severity: Severity::Warning,
                    location: point_to_arr(a),
                    detail: format!("Angle {:.1}° may cause stress concentration", angle.to_degrees()),
                    affected_vertices: vec![tri[k]],
                });
            }
        }
    }
}

fn check_milling_accessibility(
    vertices: &[Point3<f64>],
    normals: &[Vector3<f64>],
    _indices: &[[u32; 3]],
    axes: u8,
    violations: &mut Vec<Violation>,
) {
    // For 3-axis milling: check for undercuts from top direction
    let milling_dir = Vector3::new(0.0, 0.0, -1.0);

    if axes <= 3 {
        for (vi, normal) in normals.iter().enumerate() {
            let dot = normal.dot(&milling_dir);
            if dot > 0.3 { // Surface facing away from tool
                violations.push(Violation {
                    rule: "Milling accessibility".into(),
                    severity: Severity::Warning,
                    location: point_to_arr(&vertices[vi]),
                    detail: format!("{}-axis mill cannot reach this area", axes),
                    affected_vertices: vec![vi as u32],
                });
            }
        }
    }
}

fn check_print_overhangs(
    vertices: &[Point3<f64>],
    normals: &[Vector3<f64>],
    _indices: &[[u32; 3]],
    max_angle: f64,
    violations: &mut Vec<Violation>,
) {
    let up = Vector3::new(0.0, 1.0, 0.0);
    let max_cos = max_angle.to_radians().cos();

    for (vi, normal) in normals.iter().enumerate() {
        let dot = normal.dot(&up);
        if dot < -max_cos {
            violations.push(Violation {
                rule: "Print overhang".into(),
                severity: Severity::Warning,
                location: point_to_arr(&vertices[vi]),
                detail: format!("Overhang exceeds {:.0}° limit, may need support", max_angle),
                affected_vertices: vec![vi as u32],
            });
        }
    }
}

fn point_to_arr(p: &Point3<f64>) -> [f64; 3] {
    [p.x, p.y, p.z]
}

// ---------------------------------------------------------------------------
// S141-145 additions: manufacturing parameter tweaks
// ---------------------------------------------------------------------------

/// Compute available material thickness at each vertex relative to inner/outer shell.
pub fn compute_thickness_map(
    inner_verts: &[Point3<f64>],
    outer_verts: &[Point3<f64>],
) -> Vec<f64> {
    inner_verts
        .iter()
        .zip(outer_verts.iter())
        .map(|(inner, outer)| (inner - outer).norm())
        .collect()
}

/// Check that no vertex has thickness below the material's minimum.
pub fn validate_thickness(
    thickness_map: &[f64],
    material: &Material,
) -> Vec<Violation> {
    thickness_map
        .iter()
        .enumerate()
        .filter_map(|(vi, &t)| {
            if t < material.min_thickness {
                Some(Violation {
                    rule: "Minimum thickness".into(),
                    severity: Severity::Error,
                    location: [0.0, 0.0, 0.0],
                    detail: format!(
                        "Vertex {} thickness {:.3} mm below minimum {:.2} mm",
                        vi, t, material.min_thickness,
                    ),
                    affected_vertices: vec![vi as u32],
                })
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad_mesh() -> (Vec<Point3<f64>>, Vec<Vector3<f64>>, Vec<[u32; 3]>) {
        let v = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(2.0, 2.0, 0.0),
            Point3::new(0.0, 2.0, 0.0),
        ];
        let n = vec![Vector3::z(); 4];
        let i = vec![[0, 1, 2], [0, 2, 3]];
        (v, n, i)
    }

    #[test]
    fn validate_milling_no_errors() {
        let (v, n, i) = quad_mesh();
        let mat = Material::zirconia();
        let method = ManufacturingMethod::Milling { axes: 5 };
        let thicknesses = vec![1.0; v.len()];
        let result = validate_manufacturing(&v, &n, &i, &thicknesses, method, &mat);
        // Flat quad with thick material should have no thickness violations
        let errors: Vec<_> = result.violations.iter().filter(|v| matches!(v.severity, Severity::Error)).collect();
        assert!(errors.is_empty());
    }

    #[test]
    fn material_presets() {
        let z = Material::zirconia();
        assert!(z.min_thickness > 0.0);
        assert!(z.shrinkage_factor > 0.0);
        let e = Material::emax();
        assert!(e.min_thickness >= z.min_thickness * 0.5);
        let p = Material::pmma();
        assert!(p.shrinkage_factor < z.shrinkage_factor);
    }

    #[test]
    fn shrinkage_compensation_scales() {
        let mut v = vec![
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
        ];
        let n = vec![Vector3::z(); 3];
        let centroid_orig: Vector3<f64> = v.iter().map(|p| p.coords).sum::<Vector3<f64>>() / v.len() as f64;
        let dist_orig: Vec<f64> = v.iter().map(|p| (p.coords - centroid_orig).norm()).collect();
        apply_shrinkage_compensation(&mut v, &n, 20.0);
        let centroid_comp: Vector3<f64> = v.iter().map(|p| p.coords).sum::<Vector3<f64>>() / v.len() as f64;
        for (i, pt) in v.iter().enumerate() {
            let d_comp = (pt.coords - centroid_comp).norm();
            assert!(d_comp >= dist_orig[i] - 1e-9, "compensated should be scaled up");
        }
    }

    #[test]
    fn compute_thickness_basic() {
        let inner = vec![Point3::new(0.0, 0.0, 0.0)];
        let outer = vec![Point3::new(1.0, 0.0, 0.0)];
        let map = compute_thickness_map(&inner, &outer);
        assert!((map[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn validate_thickness_flags_thin() {
        let map = vec![0.1];
        let mat = Material::zirconia(); // min_thickness = 0.5
        let violations = validate_thickness(&map, &mat);
        assert_eq!(violations.len(), 1);
    }
}
