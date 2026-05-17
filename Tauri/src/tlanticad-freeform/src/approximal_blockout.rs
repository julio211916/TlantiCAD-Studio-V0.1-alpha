//! Approximal blockout visualizer — AR-V394.
//!
//! Ported from `DentalProcessors/FreeformApproximalBlockoutVisualizer`. The original wraps
//! tooth parts of type `Marker_ApproximalBlockout_Mesial` / `Marker_ApproximalBlockout_Distal`
//! and produces a per-vertex distance map highlighting zones near a neighbour tooth that fall
//! into an undercut along the insertion axis. We reduce the algorithm to the pure-Rust core:
//!
//! For each vertex `v` of the crown mesh:
//!   1. Compute distance to nearest point on the neighbour mesh.
//!   2. Compute the angle between the vertex normal and the insertion axis.
//!   3. A vertex needs blockout when:
//!        a) it is closer than `proximity_mm` to the neighbour, AND
//!        b) its surface normal angles further than `(90° - undercut_threshold_deg)` away
//!           from the insertion axis (i.e. the surface tilts away from the path of insertion,
//!           creating a mesial/distal undercut against the neighbour).
//!
//! The result is a per-vertex `Vec<bool>` mask the renderer can use to drive the
//! red-on-white texture (`CreateTwoColorTexture1D` in the C# original).

use serde::{Deserialize, Serialize};
use tlanticad_mesh::compare::closest_points_batch;
use tlanticad_mesh::nalgebra::Vector3;
use tlanticad_mesh::Mesh;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ApproximalBlockoutOptions {
    /// Maximum distance to neighbour mesh that still counts as "approximal".
    pub proximity_mm: f64,
    /// Minimum surface tilt away from insertion axis (in degrees from the axis-perpendicular
    /// plane) that flags a vertex as undercut.
    pub undercut_threshold_deg: f64,
}

impl Default for ApproximalBlockoutOptions {
    fn default() -> Self {
        Self {
            proximity_mm: 0.5,
            undercut_threshold_deg: 5.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlockoutReport {
    pub flagged_count: usize,
    pub max_proximity_mm: f64,
    pub min_proximity_mm: f64,
}

/// Per-vertex blockout mask: `true` = vertex falls in a region that needs an approximal
/// blockout fill (close to neighbour AND undercut along insertion axis).
pub fn compute_blockout_mask(
    crown_mesh: &Mesh,
    neighbour_mesh: &Mesh,
    insertion_axis: &Vector3<f64>,
    options: &ApproximalBlockoutOptions,
) -> (Vec<bool>, BlockoutReport) {
    let n = crown_mesh.vertices.len();
    if n == 0 || neighbour_mesh.vertices.is_empty() {
        return (vec![false; n], BlockoutReport::default());
    }
    let axis = if insertion_axis.norm() > 1e-12 {
        insertion_axis.normalize()
    } else {
        Vector3::z()
    };

    // Normals — we may need to compute on a clone to avoid mutating the input.
    let normals: Vec<Vector3<f64>> = if crown_mesh.normals.len() == n {
        crown_mesh.normals.clone()
    } else {
        let mut tmp = crown_mesh.clone();
        tmp.calculate_normals();
        tmp.normals
    };

    let closest = closest_points_batch(crown_mesh, neighbour_mesh);

    let undercut_cos = (90.0 - options.undercut_threshold_deg).to_radians().cos();
    // A vertex is "facing away" from the insertion axis when its normal is roughly perpendicular
    // to the axis (cos angle near 0). When |dot(n, axis)| < undercut_cos AND the normal points
    // toward the neighbour direction, we flag it.

    let mut mask = vec![false; n];
    let mut flagged = 0usize;
    let mut max_p = 0.0_f64;
    let mut min_p = f64::INFINITY;

    for i in 0..n {
        let Some(c) = closest[i] else { continue };
        let v = crown_mesh.vertices[i];
        let dist = (v - c).norm();
        if dist > max_p {
            max_p = dist;
        }
        if dist < min_p {
            min_p = dist;
        }
        if dist > options.proximity_mm {
            continue;
        }
        let n_v = normals[i];
        let n_norm = if n_v.norm() > 1e-12 {
            n_v.normalize()
        } else {
            continue;
        };
        let cos_with_axis = n_norm.dot(&axis).abs();
        // Surface tilts away from axis ⇒ small |dot|. When |dot| < undercut_cos we flag.
        if cos_with_axis < undercut_cos {
            // Also require that the toward-neighbour direction roughly aligns with the normal —
            // skip back faces.
            let toward_neighbour = (c - v).normalize();
            if n_norm.dot(&toward_neighbour) < 0.5 {
                // Normal points away from neighbour ⇒ ignore (interior face).
                continue;
            }
            mask[i] = true;
            flagged += 1;
        }
    }

    if !min_p.is_finite() {
        min_p = 0.0;
    }

    (
        mask,
        BlockoutReport {
            flagged_count: flagged,
            max_proximity_mm: max_p,
            min_proximity_mm: min_p,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tlanticad_mesh::create_box;
    use tlanticad_mesh::nalgebra::{Point3, Vector3};

    #[test]
    fn empty_meshes_yield_empty_mask() {
        let a = Mesh::new("a");
        let b = Mesh::new("b");
        let opts = ApproximalBlockoutOptions::default();
        let (mask, report) = compute_blockout_mask(&a, &b, &Vector3::z(), &opts);
        assert!(mask.is_empty());
        assert_eq!(report.flagged_count, 0);
    }

    #[test]
    fn far_neighbour_yields_no_flags() {
        let crown = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let neighbour = create_box(Point3::new(50.0, 0.0, 0.0), Point3::new(51.0, 1.0, 1.0));
        let opts = ApproximalBlockoutOptions::default();
        let (mask, report) = compute_blockout_mask(&crown, &neighbour, &Vector3::z(), &opts);
        assert!(mask.iter().all(|&m| !m));
        assert_eq!(report.flagged_count, 0);
        assert!(report.min_proximity_mm > opts.proximity_mm);
    }

    #[test]
    fn close_neighbour_with_undercut_flags_vertices() {
        // Construye un mesh "tira" cuyo +x face es plano (normales puramente +x), sin
        // promediar con caras vecinas. Evita el degenerado de un cubo donde las 3 caras
        // que comparten un vértice promedian la normal a casi-diagonal.
        let mut crown = Mesh::new("strip");
        crown.vertices = vec![
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(1.0, 1.0, 1.0),
            Point3::new(1.0, 0.0, 1.0),
        ];
        crown.indices = vec![[0, 1, 2], [0, 2, 3]];
        crown.calculate_normals();
        // Vecino a 0.2 mm en +x, cobertura completa.
        let mut neighbour = Mesh::new("neighbour");
        neighbour.vertices = vec![
            Point3::new(1.2, 0.0, 0.0),
            Point3::new(1.2, 1.0, 0.0),
            Point3::new(1.2, 1.0, 1.0),
            Point3::new(1.2, 0.0, 1.0),
        ];
        neighbour.indices = vec![[0, 1, 2], [0, 2, 3]];
        let opts = ApproximalBlockoutOptions {
            proximity_mm: 0.5,
            undercut_threshold_deg: 5.0,
        };
        // Insertion axis along +z ⇒ +x face is perpendicular to axis ⇒ undercut.
        let (mask, report) = compute_blockout_mask(&crown, &neighbour, &Vector3::z(), &opts);
        assert!(report.flagged_count > 0, "expected at least one flagged vertex; got report = {:?}", report);
        assert!(mask.iter().any(|&m| m));
    }

    #[test]
    fn no_undercut_when_axis_aligned_with_normal() {
        // Insertion axis along +x parallel with the +x face normal ⇒ no undercut even close.
        let crown = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let neighbour = create_box(Point3::new(1.2, 0.0, 0.0), Point3::new(2.2, 1.0, 1.0));
        let opts = ApproximalBlockoutOptions {
            proximity_mm: 0.5,
            undercut_threshold_deg: 5.0,
        };
        let (mask, _report) = compute_blockout_mask(&crown, &neighbour, &Vector3::x(), &opts);
        // Vertices on the +x face whose normal is aligned with axis should NOT flag.
        assert!(mask.iter().all(|&m| !m));
    }
}
