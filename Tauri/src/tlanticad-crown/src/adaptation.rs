//! Margin adaptation: adapt crown internal surface to preparation

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};

/// Cement gap configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CementGapConfig {
    /// Gap at margin line (μm)
    pub margin_gap: f64,
    /// Gap at axial wall (μm)
    pub axial_gap: f64,
    /// Gap at occlusal surface (μm)
    pub occlusal_gap: f64,
    /// Transition smoothness (0..1)
    pub transition_smoothness: f64,
}

impl Default for CementGapConfig {
    fn default() -> Self {
        Self {
            margin_gap: 30.0,
            axial_gap: 50.0,
            occlusal_gap: 80.0,
            transition_smoothness: 0.5,
        }
    }
}

/// Apply variable cement gap to internal surface of crown
///
/// Each vertex is offset along its normal by an amount that varies
/// based on its position relative to the margin and occlusal plane.
pub fn apply_cement_gap(
    vertices: &mut [Point3<f64>],
    normals: &[Vector3<f64>],
    margin_height: f64, // z-coordinate of margin plane
    occlusal_height: f64, // z-coordinate of occlusal plane
    config: &CementGapConfig,
) {
    let height_range = (occlusal_height - margin_height).max(0.001);

    for (v, n) in vertices.iter_mut().zip(normals.iter()) {
        let t = ((v.z - margin_height) / height_range).clamp(0.0, 1.0);

        // Interpolate gap: margin → axial → occlusal
        let gap_um = if t < 0.3 {
            // Margin to axial transition
            let s = t / 0.3;
            let s = smooth_step(s, config.transition_smoothness);
            config.margin_gap + (config.axial_gap - config.margin_gap) * s
        } else {
            // Axial to occlusal transition
            let s = (t - 0.3) / 0.7;
            let s = smooth_step(s, config.transition_smoothness);
            config.axial_gap + (config.occlusal_gap - config.axial_gap) * s
        };

        let gap_mm = gap_um / 1000.0;
        v.coords += n * gap_mm;
    }
}

/// Adapt crown bottom to match margin line exactly
pub fn snap_to_margin(
    crown_vertices: &mut [Point3<f64>],
    margin_points: &[Point3<f64>],
    influence_radius: f64,
) {
    for v in crown_vertices.iter_mut() {
        // Find closest margin point
        let mut min_dist = f64::MAX;
        let mut closest = Point3::origin();
        for m in margin_points {
            let d = (v.coords - m.coords).norm();
            if d < min_dist {
                min_dist = d;
                closest = *m;
            }
        }

        if min_dist < influence_radius {
            let weight = 1.0 - (min_dist / influence_radius).powi(2);
            let blend = v.coords * (1.0 - weight) + closest.coords * weight;
            v.coords = blend;
        }
    }
}

fn smooth_step(x: f64, smoothness: f64) -> f64 {
    let t = x.clamp(0.0, 1.0);
    let cubic = t * t * (3.0 - 2.0 * t);
    t * (1.0 - smoothness) + cubic * smoothness
}

/// Create a hollow shell by offsetting vertices inward by wall_thickness_mm.
/// Used for: dental splints, custom trays, night guards, partial denture bases.
pub fn hollow_shell(mesh: &tlanticad_mesh::Mesh, wall_thickness_mm: f64) -> tlanticad_mesh::Mesh {
    use nalgebra::Vector3;

    let normals = if !mesh.normals.is_empty() {
        mesh.normals.clone()
    } else {
        let n = mesh.vertices.len();
        let mut norms = vec![Vector3::zeros(); n];
        for tri in &mesh.indices {
            let ia = tri[0] as usize;
            let ib = tri[1] as usize;
            let ic = tri[2] as usize;
            if ia >= n || ib >= n || ic >= n { continue; }
            let e1 = mesh.vertices[ib] - mesh.vertices[ia];
            let e2 = mesh.vertices[ic] - mesh.vertices[ia];
            let fn_ = e1.cross(&e2);
            norms[ia] += fn_;
            norms[ib] += fn_;
            norms[ic] += fn_;
        }
        norms.iter().map(|n| { let l = n.norm(); if l > 1e-8 { n/l } else { Vector3::z() } }).collect()
    };

    // Create inner shell vertices offset inward
    let inner_verts: Vec<nalgebra::Point3<f64>> = mesh.vertices.iter().enumerate().map(|(i, v)| {
        let n = normals.get(i).copied().unwrap_or(Vector3::z());
        nalgebra::Point3::new(
            v.x - n.x * wall_thickness_mm,
            v.y - n.y * wall_thickness_mm,
            v.z - n.z * wall_thickness_mm,
        )
    }).collect();

    let n_orig = mesh.vertices.len() as u32;
    let mut verts = mesh.vertices.clone();
    let mut indices = mesh.indices.clone();
    verts.extend_from_slice(&inner_verts);

    // Add inner faces with flipped normals
    for tri in &mesh.indices {
        let a = tri[0] + n_orig;
        let b = tri[1] + n_orig;
        let c = tri[2] + n_orig;
        indices.push([a, c, b]); // flipped winding
    }

    let mut result = tlanticad_mesh::Mesh::new("hollow_shell");
    result.vertices = verts;
    result.indices = indices;
    result
}

/// Measure approximate wall thickness at each outer vertex as the distance to the nearest inner vertex
pub fn measure_wall_thickness(outer: &tlanticad_mesh::Mesh, inner: &tlanticad_mesh::Mesh) -> Vec<f64> {
    outer.vertices.iter().map(|v| {
        inner.vertices.iter().map(|iv| {
            ((v.x - iv.x).powi(2) + (v.y - iv.y).powi(2) + (v.z - iv.z).powi(2)).sqrt()
        }).fold(f64::INFINITY, f64::min)
    }).collect()
}
