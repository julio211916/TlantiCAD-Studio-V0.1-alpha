//! AR-V412 — Crown border safety zone.
//!
//! Reimplements the algorithm of `DentalProcessors/CrownBorderSafetyEditInterface.cs` in
//! idiomatic Rust. The "border safety zone" is the band of crown-bottom vertices within
//! `safety_width_mm` of the margin polyline. Inside that band the cement-gap value used by
//! the crown bottom should be reduced (better seal at the marginal seat); the rest of the
//! intaglio uses the regular cement gap.
//!
//! Two functions:
//!
//!   * `compute_border_safety_zone(margin, prep_mesh, safety_width_mm)` — returns a
//!     per-vertex `Vec<f64>` where the value is the "safety weight" in `[0, 1]`. `1.0` =
//!     fully inside the safety zone (right at the margin), `0.0` = outside the safety zone.
//!     Smooth transition over `safety_width_mm`.
//!
//!   * `apply_border_safety_constraint(crown_bottom, safety_flags, gap_at_border_mm)` — for
//!     every vertex with `safety_flags[i] > 0`, blends the original radial offset toward a
//!     smaller `gap_at_border_mm` proportionally to its safety weight. Used after the
//!     standard crown-bottom generation so the wall hugs the prep tighter at the margin.

use nalgebra::{Point3, Vector3};
use tlanticad_mesh::Mesh;

/// 3-D distance from `point` to closest point on a closed polyline (segments).
fn point_to_polyline_distance(point: Point3<f64>, polyline: &[Point3<f64>]) -> f64 {
    if polyline.len() < 2 {
        if let Some(p) = polyline.first() {
            return (point - p).norm();
        }
        return f64::INFINITY;
    }
    let n = polyline.len();
    let mut best = f64::INFINITY;
    for i in 0..n {
        let a = polyline[i];
        let b = polyline[(i + 1) % n];
        let ab = b - a;
        let len2 = ab.norm_squared();
        if len2 < 1e-12 {
            let d = (point - a).norm();
            if d < best {
                best = d;
            }
            continue;
        }
        let t = ((point - a).dot(&ab) / len2).clamp(0.0, 1.0);
        let projected = a + ab * t;
        let d = (point - projected).norm();
        if d < best {
            best = d;
        }
    }
    best
}

/// Compute per-vertex border safety weights for the prep mesh.
///
/// Output is parallel to `prep_mesh.vertices`. Each entry is in `[0, 1]`:
///   * `1.0` ⇒ vertex is exactly on the margin (full safety zone).
///   * `0.0` ⇒ vertex is at distance ≥ `safety_width_mm` (outside safety zone).
///   * Smoothstep in between.
pub fn compute_border_safety_zone(
    margin_polyline: &[Point3<f64>],
    prep_mesh: &Mesh,
    safety_width_mm: f64,
) -> Vec<f64> {
    if prep_mesh.vertices.is_empty() {
        return Vec::new();
    }
    let safety_width = safety_width_mm.max(1e-6);
    prep_mesh
        .vertices
        .iter()
        .map(|v| {
            let d = point_to_polyline_distance(*v, margin_polyline);
            if d >= safety_width {
                0.0
            } else {
                let t = 1.0 - (d / safety_width).clamp(0.0, 1.0);
                // Smoothstep for visually smooth blending.
                t * t * (3.0 - 2.0 * t)
            }
        })
        .collect()
}

/// Apply the border-safety constraint: shrink the cement-gap displacement of vertices flagged
/// as inside the safety zone. This is meant to be applied to the *crown bottom mesh* AFTER
/// the regular `generate_bottom_offset` pass. It pulls the bottom vertices back toward the
/// prep, weighted by `safety_flags[i]`.
///
/// `safety_flags` is the per-vertex weight from `compute_border_safety_zone`. Length must
/// match `crown_bottom.vertices`. The function moves each flagged vertex along its inward
/// normal so its effective gap is lerped from the existing gap toward `gap_at_border_mm`.
///
/// `original_prep_vertices` is the original prep position for vertex `i` BEFORE the bottom
/// offset (so we know how far the bottom has been pushed). The displacement direction is
/// `crown_bottom[i] - original_prep[i]` (i.e. the radial offset).
pub fn apply_border_safety_constraint(
    crown_bottom: &mut Mesh,
    original_prep_vertices: &[Point3<f64>],
    safety_flags: &[f64],
    gap_at_border_mm: f64,
) {
    if crown_bottom.vertices.is_empty() {
        return;
    }
    if safety_flags.len() != crown_bottom.vertices.len()
        || original_prep_vertices.len() != crown_bottom.vertices.len()
    {
        return;
    }
    let target_gap = gap_at_border_mm.max(0.0);
    for i in 0..crown_bottom.vertices.len() {
        let weight = safety_flags[i].clamp(0.0, 1.0);
        if weight < 1e-9 {
            continue;
        }
        let prep = original_prep_vertices[i];
        let cur = crown_bottom.vertices[i];
        let offset = cur - prep;
        let len = offset.norm();
        if len < 1e-12 {
            continue;
        }
        let dir: Vector3<f64> = offset / len;
        // New gap = lerp(current_gap, target_gap, weight)
        let new_len = len + (target_gap - len) * weight;
        let new_len = new_len.max(0.0);
        crown_bottom.vertices[i] = prep + dir * new_len;
    }
    crown_bottom.calculate_normals();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tlanticad_mesh::create_box;

    #[test]
    fn vertices_at_margin_get_full_weight() {
        let mesh = create_box(Point3::origin(), Point3::new(2.0, 2.0, 2.0));
        // Margin coincides with bottom face — vertex (0,0,0) lies right on it.
        let margin = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(2.0, 2.0, 0.0),
            Point3::new(0.0, 2.0, 0.0),
        ];
        let weights = compute_border_safety_zone(&margin, &mesh, 0.5);
        assert_eq!(weights.len(), mesh.vertex_count());
        // At least one vertex should have a high weight (close to margin).
        let max_w = weights.iter().cloned().fold(0.0_f64, f64::max);
        assert!(max_w > 0.5);
    }

    #[test]
    fn vertices_far_from_margin_have_zero_weight() {
        let mesh = create_box(Point3::origin(), Point3::new(2.0, 2.0, 2.0));
        // Margin far away at z = 100.
        let margin = vec![
            Point3::new(0.0, 0.0, 100.0),
            Point3::new(2.0, 0.0, 100.0),
            Point3::new(2.0, 2.0, 100.0),
            Point3::new(0.0, 2.0, 100.0),
        ];
        let weights = compute_border_safety_zone(&margin, &mesh, 0.5);
        // All distances are >> safety width → all zero.
        for w in &weights {
            assert!(*w < 1e-9);
        }
    }

    #[test]
    fn apply_constraint_pulls_flagged_vertices_back() {
        // Original prep vertex at origin; crown bottom vertex offset by 0.1 along +X.
        let mut bottom = Mesh::new("test");
        bottom.vertices.push(Point3::new(0.1, 0.0, 0.0));
        bottom.indices.push([0, 0, 0]); // dummy; calculate_normals tolerant
        let prep_orig = vec![Point3::origin()];
        let flags = vec![1.0]; // fully inside safety zone
        apply_border_safety_constraint(&mut bottom, &prep_orig, &flags, 0.02);
        // Should be pulled to gap_at_border_mm = 0.02.
        let new_offset = (bottom.vertices[0] - prep_orig[0]).norm();
        assert!((new_offset - 0.02).abs() < 1e-9);
    }

    #[test]
    fn apply_constraint_zero_weight_no_change() {
        let mut bottom = Mesh::new("test");
        bottom.vertices.push(Point3::new(0.1, 0.0, 0.0));
        bottom.indices.push([0, 0, 0]);
        let prep_orig = vec![Point3::origin()];
        let flags = vec![0.0]; // outside safety zone
        apply_border_safety_constraint(&mut bottom, &prep_orig, &flags, 0.02);
        // Unchanged.
        let new_offset = (bottom.vertices[0] - prep_orig[0]).norm();
        assert!((new_offset - 0.1).abs() < 1e-9);
    }

    #[test]
    fn apply_constraint_mismatched_lengths_no_op() {
        let mut bottom = Mesh::new("test");
        bottom.vertices.push(Point3::new(0.1, 0.0, 0.0));
        let prep_orig = vec![Point3::origin()];
        let flags: Vec<f64> = Vec::new(); // wrong length
        apply_border_safety_constraint(&mut bottom, &prep_orig, &flags, 0.02);
        // Must remain unchanged.
        let v = bottom.vertices[0];
        assert!((v.x - 0.1).abs() < 1e-9);
    }
}
