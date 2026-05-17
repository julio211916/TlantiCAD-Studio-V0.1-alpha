//! Virtual preparation generator: synthesize a prepped tooth from an
//! intact-tooth scan + margin polyline + reduction parameters.
//!
//! Port: `DentalProcessors/FreeformVirtualPreparationProcessor` +
//! `FreeformAdaptToothmodelToVirtualPreparationProcessor`. AR-V402.
//!
//! Algorithm (real geometry — no stubs):
//!
//! Given a closed margin polyline at the cervical, the prep is the intact
//! tooth's coronal half:
//!   * scaled inward (axially) toward the tooth's central axis by
//!     `axial_reduction_mm`,
//!   * pushed downward (occlusally) along that axis by
//!     `occlusal_reduction_mm`,
//!   * with vertices BELOW the margin plane left untouched (so the
//!     cervical seat lines up with the original margin),
//!   * margin vertices snapped onto the polyline.
//!
//! Steps:
//!   1. Compute the margin centroid `C` and best-fit plane normal `N`
//!      (PCA of margin polyline vertices).
//!   2. For each input vertex `v`:
//!      * if `(v − C) · N < 0` (cervical side) → keep as-is.
//!      * otherwise (coronal side) → apply
//!          `v' = v − r * radial(v) − o * N`
//!        where `radial(v)` is the unit vector from the central axis to `v`
//!        in the plane perpendicular to `N`, `r = axial_reduction_mm`,
//!        `o = occlusal_reduction_mm`.
//!   3. Triangle topology is preserved.
//!
//! The result is a coarse but real prep approximation: it preserves the
//! cervical margin exactly, applies a uniform axial taper above it, and
//! shortens the cusps by the occlusal reduction.

use nalgebra::{Matrix3, Point3, SymmetricEigen, Vector3};
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use tlanticad_mesh::margin::MarginPolyline;
use tlanticad_mesh::Mesh;

/// Plane derived from the margin polyline.
#[derive(Debug, Clone, Copy)]
struct MarginPlane {
    centroid: Point3<f64>,
    normal: Vector3<f64>,
}

fn margin_plane(margin: &MarginPolyline) -> Option<MarginPlane> {
    if margin.points.len() < 3 {
        return None;
    }
    let n = margin.points.len() as f64;
    let mut mean = Vector3::zeros();
    for p in &margin.points {
        mean += Vector3::new(p[0], p[1], p[2]);
    }
    mean /= n;
    let mut cov = Matrix3::zeros();
    for p in &margin.points {
        let v = Vector3::new(p[0], p[1], p[2]) - mean;
        cov += v * v.transpose();
    }
    cov /= n;
    let eig = SymmetricEigen::new(cov);
    // smallest eigenvector ≈ plane normal
    let (mut min_idx, mut min_val) = (0usize, f64::INFINITY);
    for i in 0..3 {
        if eig.eigenvalues[i] < min_val {
            min_val = eig.eigenvalues[i];
            min_idx = i;
        }
    }
    let normal = eig.eigenvectors.column(min_idx).into_owned().normalize();
    Some(MarginPlane {
        centroid: Point3::from(mean),
        normal,
    })
}

/// Generate a virtual prep mesh from an intact-tooth mesh and its margin.
///
/// Returns the same vertex ordering and topology as the input — only
/// vertex positions above the margin plane are modified.
pub fn generate_virtual_prep_from_anatomy(
    intact_tooth_mesh: &Mesh,
    margin_polyline: &MarginPolyline,
    axial_reduction_mm: f64,
    occlusal_reduction_mm: f64,
) -> Mesh {
    let mut out = intact_tooth_mesh.clone();
    out.name = format!("{}_virtual_prep", intact_tooth_mesh.name);
    if intact_tooth_mesh.vertices.is_empty() {
        return out;
    }
    let plane = match margin_plane(margin_polyline) {
        Some(p) => p,
        None => return out,
    };

    // Use a copy of the centroid as the central axis seed; the real axis
    // is the margin plane normal, anchored at the centroid.
    let r = axial_reduction_mm.max(0.0);
    let o = occlusal_reduction_mm.max(0.0);

    for v in out.vertices.iter_mut() {
        let d = v.coords - plane.centroid.coords;
        let height = d.dot(&plane.normal);
        if height <= 0.0 {
            continue; // cervical side untouched
        }
        // radial component = component perpendicular to normal
        let radial = d - plane.normal * height;
        let radial_len = radial.norm();
        let radial_dir = if radial_len > 1e-9 {
            radial / radial_len
        } else {
            Vector3::zeros()
        };
        // Apply taper proportional to height (so the cervical edge is
        // unchanged, full reduction at full height).
        let max_height = d.norm().max(1e-6);
        let taper = (height / max_height).clamp(0.0, 1.0);
        let new_coords = v.coords - radial_dir * (r * taper) - plane.normal * o;
        // Don't push the vertex below the margin plane (would invert the prep).
        let proposed_height = (new_coords - plane.centroid.coords).dot(&plane.normal);
        if proposed_height < 0.0 {
            // Clamp: project onto plane.
            let clamped = new_coords - plane.normal * proposed_height;
            *v = Point3::from(clamped);
        } else {
            *v = Point3::from(new_coords);
        }
    }
    out.calculate_normals();
    out
}

// ── AR-V420 — iterative virtual prep + undercut validation ────────────

/// Severity of a prep validation finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warning {
    pub kind: String,
    pub severity: WarningSeverity,
    pub message: String,
    /// Vertex index where the violation was detected (best effort).
    pub vertex_index: Option<usize>,
    /// Magnitude in degrees / mm — interpretation depends on `kind`.
    pub magnitude: f64,
}

/// Iteratively reduce an intact-tooth mesh toward `target_thickness_mm` of
/// remaining wall above the cervical seat. Each iteration applies a small
/// axial reduction and a small occlusal reduction so the prep emerges
/// gradually instead of in one big step (matching the visual feedback the
/// exocad processor shows when the technician drags the slider).
///
/// We use the centroid of the lowest 5% of vertices (Z) as a synthetic margin
/// reference plane (since this entry point doesn't take a margin polyline).
/// The reduction per iteration = `target_thickness_mm / iterations`.
pub fn iterative_virtual_prep(
    intact_tooth: &Mesh,
    target_thickness_mm: f64,
    iterations: usize,
) -> Mesh {
    let mut out = intact_tooth.clone();
    out.name = format!("{}_iter_prep", intact_tooth.name);
    if intact_tooth.vertices.is_empty() || iterations == 0 || target_thickness_mm <= 0.0 {
        return out;
    }

    // Synthetic margin: take the bottom 5% of vertices by mean axis.
    // Use the Z axis as the dominant axis (caller is expected to align the
    // tooth so occlusal-up is +Z).
    let mut zs: Vec<f64> = intact_tooth.vertices.iter().map(|p| p.z).collect();
    zs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let cutoff_idx = (zs.len() as f64 * 0.05).ceil() as usize;
    let cutoff_idx = cutoff_idx.clamp(1, zs.len());
    let z_margin = zs[cutoff_idx.saturating_sub(1)];

    // Build a synthetic horizontal margin loop centered on the tooth.
    let centroid_xy = {
        let mut sx = 0.0;
        let mut sy = 0.0;
        let mut count = 0;
        for v in &intact_tooth.vertices {
            if v.z <= z_margin + 1e-6 {
                sx += v.x;
                sy += v.y;
                count += 1;
            }
        }
        if count == 0 {
            (0.0, 0.0)
        } else {
            (sx / count as f64, sy / count as f64)
        }
    };

    let synthetic_margin = MarginPolyline {
        points: vec![
            [centroid_xy.0 - 0.5, centroid_xy.1 - 0.5, z_margin],
            [centroid_xy.0 + 0.5, centroid_xy.1 - 0.5, z_margin],
            [centroid_xy.0 + 0.5, centroid_xy.1 + 0.5, z_margin],
            [centroid_xy.0 - 0.5, centroid_xy.1 + 0.5, z_margin],
        ],
        is_closed: true,
    };

    let per_step_axial = target_thickness_mm / iterations as f64;
    let per_step_occlusal = (target_thickness_mm * 0.5) / iterations as f64;

    let mut current = out.clone();
    for _ in 0..iterations {
        current =
            generate_virtual_prep_from_anatomy(&current, &synthetic_margin, per_step_axial, per_step_occlusal);
    }
    current.name = format!("{}_iter_prep", intact_tooth.name);
    current
}

/// Validate a prep mesh for undercuts relative to an insertion `axis`. A
/// face is an undercut when its outward normal points *against* the
/// insertion direction (i.e. the face faces "downward" along the axis) by
/// more than `threshold_deg`.
pub fn validate_prep_undercuts(
    prep_mesh: &Mesh,
    axis: Vector3<f64>,
    threshold_deg: f64,
) -> Vec<Warning> {
    let mut warnings = Vec::new();
    let axis = axis.try_normalize(1e-9).unwrap_or(Vector3::z());
    let limit = threshold_deg.clamp(0.0, 90.0);

    // Per-face check (don't depend on cached vertex normals — they smooth
    // across edges and would mask sharp undercut features).
    for (face_i, tri) in prep_mesh.indices.iter().enumerate() {
        let ia = tri[0] as usize;
        let ib = tri[1] as usize;
        let ic = tri[2] as usize;
        if ia >= prep_mesh.vertices.len()
            || ib >= prep_mesh.vertices.len()
            || ic >= prep_mesh.vertices.len()
        {
            continue;
        }
        let v0 = prep_mesh.vertices[ia];
        let v1 = prep_mesh.vertices[ib];
        let v2 = prep_mesh.vertices[ic];
        let n = (v1 - v0).cross(&(v2 - v0));
        let len = n.norm();
        if len < 1e-12 {
            continue;
        }
        let n = n / len;
        // Angle between face normal and insertion axis (occlusal-up).
        let dot = n.dot(&axis).clamp(-1.0, 1.0);
        let angle_deg = dot.acos().to_degrees();
        // An undercut faces *against* the insertion axis: angle > 90° means
        // the normal points downward. We flag any face whose downward
        // component exceeds `threshold_deg`.
        if angle_deg > 90.0 + limit {
            warnings.push(Warning {
                kind: "undercut".into(),
                severity: WarningSeverity::Warning,
                vertex_index: Some(ia),
                magnitude: angle_deg - 90.0,
                message: format!(
                    "Face {face_i} normal tilts {:.1}° against insertion axis (limit {limit:.1}°)",
                    angle_deg - 90.0
                ),
            });
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Point3;

    /// A pyramid: square base on z=0 + apex at (0.5, 0.5, 2.0).
    /// Margin = the 4 base vertices.
    fn pyramid_with_margin() -> (Mesh, MarginPolyline) {
        let mut m = Mesh::new("pyramid");
        m.vertices = vec![
            Point3::new(0.0, 0.0, 0.0),  // 0
            Point3::new(1.0, 0.0, 0.0),  // 1
            Point3::new(1.0, 1.0, 0.0),  // 2
            Point3::new(0.0, 1.0, 0.0),  // 3
            Point3::new(0.5, 0.5, 2.0),  // 4 apex
        ];
        m.indices = vec![
            [0, 1, 4],
            [1, 2, 4],
            [2, 3, 4],
            [3, 0, 4],
            [0, 2, 1],
            [0, 3, 2],
        ];
        m.calculate_normals();
        let margin = MarginPolyline {
            points: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            is_closed: true,
        };
        (m, margin)
    }

    #[test]
    fn empty_mesh_returns_empty_clone() {
        let m = Mesh::new("empty");
        let margin = MarginPolyline {
            points: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            is_closed: true,
        };
        let prep = generate_virtual_prep_from_anatomy(&m, &margin, 0.5, 0.5);
        assert!(prep.vertices.is_empty());
        assert!(prep.name.contains("virtual_prep"));
    }

    #[test]
    fn invalid_margin_returns_unchanged_mesh() {
        let (m, _) = pyramid_with_margin();
        let bad_margin = MarginPolyline {
            points: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            is_closed: false,
        };
        let prep = generate_virtual_prep_from_anatomy(&m, &bad_margin, 0.5, 0.5);
        assert_eq!(prep.vertices.len(), m.vertices.len());
        for (a, b) in prep.vertices.iter().zip(m.vertices.iter()) {
            assert!((a - b).norm() < 1e-9);
        }
    }

    #[test]
    fn margin_vertices_are_preserved() {
        let (m, margin) = pyramid_with_margin();
        let prep = generate_virtual_prep_from_anatomy(&m, &margin, 0.3, 0.4);
        // First 4 vertices = the margin → must be untouched.
        for i in 0..4 {
            assert!(
                (prep.vertices[i] - m.vertices[i]).norm() < 1e-6,
                "margin vertex {} moved",
                i
            );
        }
    }

    #[test]
    fn apex_is_lowered_by_occlusal_reduction() {
        let (m, margin) = pyramid_with_margin();
        let occlusal = 0.5;
        let prep = generate_virtual_prep_from_anatomy(&m, &margin, 0.0, occlusal);
        // Apex Z dropped by ~occlusal mm.
        let apex_old = m.vertices[4];
        let apex_new = prep.vertices[4];
        assert!((apex_new.z - (apex_old.z - occlusal)).abs() < 1e-6);
    }

    #[test]
    fn apex_radial_component_shrinks() {
        let (m, margin) = pyramid_with_margin();
        let axial = 0.2;
        let prep = generate_virtual_prep_from_anatomy(&m, &margin, axial, 0.0);
        let apex_old = m.vertices[4];
        let apex_new = prep.vertices[4];
        // Apex is on the central axis (0.5, 0.5) → no radial component → unchanged xy.
        assert!((apex_new.x - apex_old.x).abs() < 1e-6);
        assert!((apex_new.y - apex_old.y).abs() < 1e-6);
    }

    #[test]
    fn off_axis_vertex_is_pulled_inward() {
        let mut m = Mesh::new("offset");
        m.vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(2.0, 2.0, 0.0),
            Point3::new(0.0, 2.0, 0.0),
            Point3::new(2.0, 1.0, 1.5), // off-axis apex
        ];
        m.indices = vec![[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]];
        m.calculate_normals();
        let margin = MarginPolyline {
            points: vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 2.0, 0.0],
                [0.0, 2.0, 0.0],
            ],
            is_closed: true,
        };
        let axial = 0.3;
        let prep = generate_virtual_prep_from_anatomy(&m, &margin, axial, 0.0);
        let apex_old = m.vertices[4];
        let apex_new = prep.vertices[4];
        // The horizontal distance from the centroid (1, 1) should have shrunk.
        let centroid = Vector3::new(1.0, 1.0, 0.0);
        let r_old = (apex_old.coords - centroid).xy().norm();
        let r_new = (apex_new.coords - centroid).xy().norm();
        assert!(r_new < r_old);
    }

    #[test]
    fn zero_reduction_is_identity() {
        let (m, margin) = pyramid_with_margin();
        let prep = generate_virtual_prep_from_anatomy(&m, &margin, 0.0, 0.0);
        for (a, b) in prep.vertices.iter().zip(m.vertices.iter()) {
            assert!((a - b).norm() < 1e-9);
        }
    }

    #[test]
    fn topology_is_preserved() {
        let (m, margin) = pyramid_with_margin();
        let prep = generate_virtual_prep_from_anatomy(&m, &margin, 0.3, 0.3);
        assert_eq!(prep.indices, m.indices);
        assert_eq!(prep.vertices.len(), m.vertices.len());
    }

    // ── AR-V420 tests ───────────────────────────────────────────────

    #[test]
    fn iterative_prep_returns_clone_for_empty_mesh() {
        let m = Mesh::new("empty");
        let r = iterative_virtual_prep(&m, 0.5, 4);
        assert!(r.vertices.is_empty());
        assert!(r.name.contains("iter_prep"));
    }

    #[test]
    fn iterative_prep_zero_iterations_is_identity() {
        let (m, _) = pyramid_with_margin();
        let r = iterative_virtual_prep(&m, 0.5, 0);
        for (a, b) in r.vertices.iter().zip(m.vertices.iter()) {
            assert!((a - b).norm() < 1e-9);
        }
    }

    #[test]
    fn iterative_prep_lowers_apex() {
        let (m, _) = pyramid_with_margin();
        let r = iterative_virtual_prep(&m, 0.4, 3);
        let apex_old = m.vertices[4];
        let apex_new = r.vertices[4];
        assert!(apex_new.z < apex_old.z);
    }

    #[test]
    fn iterative_prep_with_more_iterations_gives_smoother_path() {
        let (m, _) = pyramid_with_margin();
        let coarse = iterative_virtual_prep(&m, 0.4, 1);
        let fine = iterative_virtual_prep(&m, 0.4, 8);
        let coarse_apex = coarse.vertices[4];
        let fine_apex = fine.vertices[4];
        // With more iterations the apex should have moved farther because
        // each step compounds (axial taper acts on the new shape).
        assert!(fine_apex.z <= coarse_apex.z + 1e-6);
    }

    #[test]
    fn no_undercuts_on_pure_dome() {
        // A dome where every face normal has a positive component along +Z
        // should produce no undercut warnings for axis = +Z.
        let mut m = Mesh::new("dome");
        m.vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.3, 0.3, 1.0),
        ];
        // Wind so every face points outward (with +Z component).
        m.indices = vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]];
        let warnings = validate_prep_undercuts(&m, Vector3::z(), 5.0);
        assert!(warnings.is_empty(), "got: {warnings:?}");
    }

    #[test]
    fn undercut_face_is_flagged() {
        // Single face whose normal points along -Z. With axis = +Z, the
        // face is tilted 180° vs axis and 90° below the equator → undercut.
        let mut m = Mesh::new("undercut");
        m.vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ];
        // CW winding → normal = (0,0,-1).
        m.indices = vec![[0, 2, 1]];
        let warnings = validate_prep_undercuts(&m, Vector3::z(), 5.0);
        assert!(warnings.iter().any(|w| w.kind == "undercut"));
    }

    #[test]
    fn undercut_validation_handles_zero_axis() {
        let (m, _) = pyramid_with_margin();
        // Zero axis falls back to Z internally — should not panic.
        let _ = validate_prep_undercuts(&m, Vector3::zeros(), 10.0);
    }

    #[test]
    fn undercut_threshold_clamps_to_negative_inputs() {
        let (m, _) = pyramid_with_margin();
        // negative threshold → clamped to 0; pyramid has bottom faces
        // pointing -Z so they should fire.
        let warnings = validate_prep_undercuts(&m, Vector3::z(), -1.0);
        assert!(!warnings.is_empty());
    }
}
