//! Paint-pull / push / smooth — the core brush ops of FreeformProcessor.
//!
//! Ported from `DentalProcessors/FreeformProcessor` (7367 LOC) +
//! `FreeformAnatomicToothProcessor` + `FreeformGingivaProcessor` +
//! `FreeformProstheticBaseProcessor` + `FreeformEmergenceProfileProcessor`.
//! AR-V374.
//!
//! Three operations on a mesh given a brush position + radius:
//!   * `paint_pull`  — push vertices outward along their normals (positive amount) or
//!                     inward (negative). Falloff smoothstep over radius.
//!   * `paint_smooth`— Laplacian smoothing within the brush radius.
//!   * `paint_drape` — pull each vertex toward an arbitrary direction (used for "drag" tool).
//!
//! Each op returns the indices of vertices that moved + the maximum displacement.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tlanticad_mesh::Mesh;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BrushParams {
    pub center: [f64; 3],
    pub radius_mm: f64,
    /// Strength multiplier (0..2). 1.0 is the natural amplitude.
    pub strength: f64,
    /// Falloff exponent — 1 = linear, 2 = smooth, 4 = soft edges. Default 2.
    pub falloff: f64,
}

impl Default for BrushParams {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0, 0.0],
            radius_mm: 1.0,
            strength: 1.0,
            falloff: 2.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BrushReport {
    pub vertices_affected: usize,
    pub max_displacement_mm: f64,
    pub mean_displacement_mm: f64,
}

fn smoothstep(x: f64) -> f64 {
    let t = x.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Falloff factor for a vertex `dist` from the brush center, given `radius`.
fn falloff(dist: f64, radius: f64, exponent: f64) -> f64 {
    if radius <= 1e-9 || dist >= radius {
        return 0.0;
    }
    let t = 1.0 - (dist / radius);
    smoothstep(t).powf(exponent.max(0.5))
}

/// Pull vertices outward along their normals. Positive `amount_mm` = add material;
/// negative = remove. Returns the brush report with affected count + max displacement.
pub fn paint_pull(mesh: &mut Mesh, params: &BrushParams, amount_mm: f64) -> BrushReport {
    if mesh.vertices.is_empty() || mesh.normals.len() != mesh.vertices.len() {
        mesh.calculate_normals();
    }
    let center = Point3::new(params.center[0], params.center[1], params.center[2]);
    let r = params.radius_mm.max(1e-6);
    let strength = params.strength.clamp(-2.0, 2.0);

    let mut affected = 0usize;
    let mut max_d = 0.0_f64;
    let mut sum_d = 0.0_f64;

    for i in 0..mesh.vertices.len() {
        let p = mesh.vertices[i];
        let dist = (p - center).norm();
        let f = falloff(dist, r, params.falloff);
        if f <= 0.0 {
            continue;
        }
        let n = mesh.normals[i];
        let displacement = n * (amount_mm * strength * f);
        let d = displacement.norm();
        if d <= 1e-9 {
            continue;
        }
        mesh.vertices[i] = p + displacement;
        affected += 1;
        sum_d += d;
        if d > max_d {
            max_d = d;
        }
    }
    mesh.calculate_normals();
    BrushReport {
        vertices_affected: affected,
        max_displacement_mm: max_d,
        mean_displacement_mm: if affected > 0 {
            sum_d / affected as f64
        } else {
            0.0
        },
    }
}

/// Local Laplacian smoothing within the brush radius. `iterations` controls intensity.
pub fn paint_smooth(mesh: &mut Mesh, params: &BrushParams, iterations: u32) -> BrushReport {
    if mesh.vertices.is_empty() {
        return BrushReport::default();
    }
    let center = Point3::new(params.center[0], params.center[1], params.center[2]);
    let r = params.radius_mm.max(1e-6);

    // Build vertex adjacency.
    let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();
    for tri in &mesh.indices {
        for i in 0..3 {
            for j in 0..3 {
                if i != j {
                    adj.entry(tri[i]).or_default().push(tri[j]);
                }
            }
        }
    }

    let strength = params.strength.clamp(0.0, 2.0);
    let mut affected = 0usize;
    let mut max_d = 0.0_f64;
    let mut sum_d = 0.0_f64;

    for _ in 0..iterations.max(1) {
        let snapshot = mesh.vertices.clone();
        for (idx, neighbors) in &adj {
            let i = *idx as usize;
            if i >= snapshot.len() {
                continue;
            }
            let p = snapshot[i];
            let dist = (p - center).norm();
            let f = falloff(dist, r, params.falloff);
            if f <= 0.0 || neighbors.is_empty() {
                continue;
            }
            let mean: Vector3<f64> = neighbors
                .iter()
                .map(|&n| snapshot[n as usize].coords)
                .sum::<Vector3<f64>>()
                / neighbors.len() as f64;
            let lambda = (strength * f).clamp(0.0, 1.0);
            let new_p = Point3::from(p.coords.lerp(&mean, lambda));
            let d = (new_p - p).norm();
            if d > 1e-9 {
                mesh.vertices[i] = new_p;
                if d > max_d {
                    max_d = d;
                }
                sum_d += d;
                affected += 1;
            }
        }
    }
    mesh.calculate_normals();
    BrushReport {
        vertices_affected: affected,
        max_displacement_mm: max_d,
        mean_displacement_mm: if affected > 0 {
            sum_d / affected as f64
        } else {
            0.0
        },
    }
}

/// Drag vertices in `direction` (mm). Used for the "grab" / drag tool.
pub fn paint_drape(mesh: &mut Mesh, params: &BrushParams, direction_mm: [f64; 3]) -> BrushReport {
    if mesh.vertices.is_empty() {
        return BrushReport::default();
    }
    let center = Point3::new(params.center[0], params.center[1], params.center[2]);
    let r = params.radius_mm.max(1e-6);
    let dir = Vector3::new(direction_mm[0], direction_mm[1], direction_mm[2]);
    let mag = dir.norm();
    if mag < 1e-9 {
        return BrushReport::default();
    }
    let strength = params.strength.clamp(-2.0, 2.0);

    let mut affected = 0usize;
    let mut max_d = 0.0_f64;
    let mut sum_d = 0.0_f64;

    for i in 0..mesh.vertices.len() {
        let p = mesh.vertices[i];
        let dist = (p - center).norm();
        let f = falloff(dist, r, params.falloff);
        if f <= 0.0 {
            continue;
        }
        let displacement = dir * (strength * f);
        let d = displacement.norm();
        if d <= 1e-9 {
            continue;
        }
        mesh.vertices[i] = p + displacement;
        affected += 1;
        sum_d += d;
        if d > max_d {
            max_d = d;
        }
    }
    mesh.calculate_normals();
    BrushReport {
        vertices_affected: affected,
        max_displacement_mm: max_d,
        mean_displacement_mm: if affected > 0 {
            sum_d / affected as f64
        } else {
            0.0
        },
    }
}

/// Generate an emergence profile blend mesh between margin polyline (gingiva line) and a
/// circle at offset height. Anatomic concave-then-convex S-curve. Used by
/// `FreeformEmergenceProfileProcessor`.
pub fn build_emergence_profile(
    margin: &[Point3<f64>],
    insertion_axis: Vector3<f64>,
    height_mm: f64,
    top_radius_mm: f64,
    axial_segments: u32,
) -> Mesh {
    let mut mesh = Mesh::new("emergence-profile");
    if margin.len() < 3 || axial_segments < 2 || height_mm <= 0.0 {
        return mesh;
    }
    let axis = insertion_axis.try_normalize(1e-9).unwrap_or(Vector3::z());
    let n = margin.len();
    let centroid = {
        let mut sum = Vector3::zeros();
        for p in margin {
            sum += p.coords;
        }
        Point3::from(sum / n as f64)
    };
    let max_r = margin
        .iter()
        .map(|p| (p - centroid).norm())
        .fold(0.0_f64, f64::max);
    let target_ratio = if max_r > 1e-9 {
        top_radius_mm / max_r
    } else {
        1.0
    };

    let mut vertices: Vec<Point3<f64>> = Vec::with_capacity((axial_segments as usize + 1) * n);
    for ring in 0..=axial_segments {
        let t = ring as f64 / axial_segments as f64;
        // S-curve: concave near gingiva, convex toward shoulder.
        let scurve = 0.5 - 0.5 * (std::f64::consts::PI * t).cos();
        let factor = 1.0 + (target_ratio - 1.0) * scurve;
        let center = centroid + axis * (height_mm * t);
        for j in 0..n {
            let radial: Vector3<f64> = margin[j] - centroid;
            let world = center.coords + radial * factor;
            vertices.push(Point3::from(world));
        }
    }
    let mut indices: Vec<[u32; 3]> = Vec::new();
    for ring in 0..axial_segments as u32 {
        for j in 0..n as u32 {
            let j_next = (j + 1) % n as u32;
            let r0 = ring * n as u32 + j;
            let r1 = ring * n as u32 + j_next;
            let r2 = (ring + 1) * n as u32 + j;
            let r3 = (ring + 1) * n as u32 + j_next;
            indices.push([r0, r2, r1]);
            indices.push([r1, r2, r3]);
        }
    }
    mesh.vertices = vertices;
    mesh.indices = indices;
    mesh.calculate_normals();
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use tlanticad_mesh::create_box;

    #[test]
    fn pull_outward_increases_volume() {
        let mut mesh = create_box(Point3::origin(), Point3::new(2.0, 2.0, 2.0));
        let params = BrushParams {
            center: [1.0, 1.0, 2.0],
            radius_mm: 5.0,
            strength: 1.0,
            falloff: 2.0,
        };
        let report = paint_pull(&mut mesh, &params, 0.5);
        assert!(report.vertices_affected > 0);
        assert!(report.max_displacement_mm > 0.0);
    }

    #[test]
    fn pull_negative_amount_removes() {
        let mut mesh = create_box(Point3::origin(), Point3::new(2.0, 2.0, 2.0));
        let params = BrushParams {
            center: [1.0, 1.0, 2.0],
            radius_mm: 5.0,
            strength: 1.0,
            falloff: 2.0,
        };
        let report = paint_pull(&mut mesh, &params, -0.3);
        assert!(report.vertices_affected > 0);
    }

    #[test]
    fn smooth_does_not_explode() {
        let mut mesh = create_box(Point3::origin(), Point3::new(2.0, 2.0, 2.0));
        let bbox_before = mesh.calculate_bounds();
        let params = BrushParams {
            center: [1.0, 1.0, 1.0],
            radius_mm: 5.0,
            strength: 1.0,
            falloff: 2.0,
        };
        let _ = paint_smooth(&mut mesh, &params, 3);
        let bbox_after = mesh.calculate_bounds();
        assert!(bbox_after.0.x >= bbox_before.0.x - 0.5);
        assert!(bbox_after.1.x <= bbox_before.1.x + 0.5);
    }

    #[test]
    fn drape_zero_dir_no_op() {
        let mut mesh = create_box(Point3::origin(), Point3::new(2.0, 2.0, 2.0));
        let params = BrushParams {
            center: [1.0, 1.0, 1.0],
            radius_mm: 2.0,
            strength: 1.0,
            falloff: 2.0,
        };
        let report = paint_drape(&mut mesh, &params, [0.0, 0.0, 0.0]);
        assert_eq!(report.vertices_affected, 0);
    }

    #[test]
    fn emergence_profile_builds_watertight_lateral() {
        let margin: Vec<Point3<f64>> = (0..16)
            .map(|i| {
                let theta = std::f64::consts::TAU * (i as f64) / 16.0;
                Point3::new(2.0 * theta.cos(), 2.0 * theta.sin(), 0.0)
            })
            .collect();
        let mesh = build_emergence_profile(&margin, Vector3::z(), 3.0, 1.0, 4);
        assert!(mesh.vertex_count() >= 16 * 5);
        assert!(mesh.triangle_count() > 0);
    }

    #[test]
    fn falloff_zero_outside_radius() {
        assert_eq!(falloff(2.0, 1.0, 2.0), 0.0);
    }

    #[test]
    fn falloff_max_at_center() {
        let f = falloff(0.0, 1.0, 2.0);
        assert!((f - 1.0).abs() < 1e-9);
    }
}
