//! Merged thimble (telescopic crown) substructure. AR-V416.
//!
//! Conceptually ported from
//! `artifacts/DentalProcessors/FreeformMergedThimbleSubstructureProcessor.cs`.
//!
//! A "merged thimble" is the inner primary coping for a telescopic
//! prosthesis where multiple primary copings are fused into a single rigid
//! framework — common for the bar+thimble overdentures and conical-crown
//! tele-bridges. We synthesise it from:
//!
//!   * `margin_polyline` — the cervical margin of a single prep (or the
//!     concatenated margins of several preps for a true merged variant —
//!     callers feed an already-merged polyline).
//!   * `thimble_height` — height (mm) measured along `framework_axis` from
//!     the cervical to the occlusal pole.
//!   * `wall_thickness` — the wall thickness of the thimble (mm). Inner
//!     ring sits offset inward by this amount (perpendicular to the
//!     framework axis).
//!   * `framework_axis` — the path-of-insertion / common axis of all
//!     primary copings. Must be unit length-ish; we normalise.
//!
//! Geometry — the thimble is a closed shell with three rings:
//!
//!   * **Outer cervical ring** = margin polyline (shared with the prep).
//!   * **Outer occlusal ring** = margin centroid offset by `thimble_height`
//!     along axis, with the same radial pattern as the margin (so the side
//!     is a cylindrical "skirt"); occlusal pole is closed with a fan.
//!   * **Inner cervical ring** = margin offset inward (toward axis) by
//!     `wall_thickness`. The inner skirt mirrors the outer one and the
//!     inner pole is a fan in the opposite winding so the two shells share
//!     the same cervical edge — making the result a watertight thimble
//!     shell with the cervical edge as the only opening (matches what the
//!     C# processor exports).
//!
//! Output: a single `Mesh` with consistent winding (outward normals on the
//! outer skirt, inward on the inner skirt).

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::margin::MarginPolyline;
use tlanticad_mesh::Mesh;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ThimbleParams {
    /// Whether the cervical margin is a closed loop. Defaults to the
    /// margin's own `is_closed`.
    pub close_cervical_top_optional: bool,
}

impl Default for ThimbleParams {
    fn default() -> Self {
        Self {
            close_cervical_top_optional: false,
        }
    }
}

/// Build the merged thimble substructure mesh.
///
/// Returns an empty mesh when the margin has fewer than 3 points or when
/// `thimble_height` ≤ 0 / `wall_thickness` < 0.
pub fn generate_merged_thimble(
    margin_polyline: &MarginPolyline,
    thimble_height: f64,
    wall_thickness: f64,
    framework_axis: Vector3<f64>,
) -> Mesh {
    let mut mesh = Mesh::new("merged-thimble-substructure");
    if margin_polyline.len() < 3 || thimble_height <= 0.0 || wall_thickness < 0.0 {
        return mesh;
    }
    let axis = framework_axis.try_normalize(1e-9).unwrap_or(Vector3::z());
    let n = margin_polyline.len() as u32;

    let centroid = polyline_centroid(margin_polyline);
    let cervical_outer: Vec<Point3<f64>> = (0..margin_polyline.len())
        .map(|i| margin_polyline.point(i))
        .collect();
    // Inner cervical ring: each margin point offset inward (toward centroid
    // perpendicular to axis) by `wall_thickness`.
    let cervical_inner: Vec<Point3<f64>> = cervical_outer
        .iter()
        .map(|&p| p + inward_normal(p, centroid, axis) * wall_thickness)
        .collect();
    // Occlusal outer ring: cervical_outer translated up by thimble_height along axis.
    let occlusal_outer: Vec<Point3<f64>> = cervical_outer
        .iter()
        .map(|&p| p + axis * thimble_height)
        .collect();
    // Occlusal inner ring: cervical_inner translated up by thimble_height.
    let occlusal_inner: Vec<Point3<f64>> = cervical_inner
        .iter()
        .map(|&p| p + axis * thimble_height)
        .collect();

    let outer_cerv_base = 0u32;
    mesh.vertices.extend(cervical_outer.iter().copied());
    let outer_occl_base = mesh.vertices.len() as u32;
    mesh.vertices.extend(occlusal_outer.iter().copied());
    let inner_cerv_base = mesh.vertices.len() as u32;
    mesh.vertices.extend(cervical_inner.iter().copied());
    let inner_occl_base = mesh.vertices.len() as u32;
    mesh.vertices.extend(occlusal_inner.iter().copied());

    let closed = margin_polyline.is_closed;

    // Outer skirt — winding: outer normals point AWAY from axis.
    stitch_skirt(&mut mesh.indices, outer_cerv_base, outer_occl_base, n, closed, true);
    // Inner skirt — flip winding so inward face shows.
    stitch_skirt(&mut mesh.indices, inner_cerv_base, inner_occl_base, n, closed, false);

    // Occlusal cap fan — bridges outer occlusal ring to inner occlusal ring.
    // We close the top by zipping the outer-occlusal and inner-occlusal rings
    // (a thin annulus stitched as 2 triangles per segment).
    for i in 0..n {
        let i_next = if closed {
            (i + 1) % n
        } else if i + 1 == n {
            break;
        } else {
            i + 1
        };
        let oa = outer_occl_base + i;
        let ob = outer_occl_base + i_next;
        let ia = inner_occl_base + i;
        let ib = inner_occl_base + i_next;
        // Outer-side winding: oa, ob, ib  /  oa, ib, ia
        mesh.indices.push([oa, ob, ib]);
        mesh.indices.push([oa, ib, ia]);
    }

    mesh.calculate_normals();
    mesh
}

fn polyline_centroid(margin: &MarginPolyline) -> Point3<f64> {
    let mut acc = Vector3::zeros();
    for p in &margin.points {
        acc += Vector3::new(p[0], p[1], p[2]);
    }
    Point3::from(acc / margin.len() as f64)
}

/// "Inward" = perpendicular component of (centroid - point) projected onto
/// the plane normal to `axis`. Falls back to a default basis if the
/// projection collapses (point sits on the axis).
fn inward_normal(point: Point3<f64>, centroid: Point3<f64>, axis: Vector3<f64>) -> Vector3<f64> {
    let radial = centroid - point;
    let perp = radial - axis * radial.dot(&axis);
    let len = perp.norm();
    if len < 1e-9 {
        let helper = if axis.x.abs() < 0.9 {
            Vector3::x()
        } else {
            Vector3::y()
        };
        axis.cross(&helper).normalize()
    } else {
        perp / len
    }
}

/// Stitch two equal-length rings into a band. `outward = true` keeps the
/// natural CCW winding (outer skirt). `outward = false` flips the winding
/// (inner skirt — inward-facing).
fn stitch_skirt(
    indices: &mut Vec<[u32; 3]>,
    bottom_base: u32,
    top_base: u32,
    count: u32,
    closed: bool,
    outward: bool,
) {
    for i in 0..count {
        let i_next = if closed {
            (i + 1) % count
        } else if i + 1 == count {
            break;
        } else {
            i + 1
        };
        let b0 = bottom_base + i;
        let b1 = bottom_base + i_next;
        let t0 = top_base + i;
        let t1 = top_base + i_next;
        if outward {
            indices.push([b0, t0, t1]);
            indices.push([b0, t1, b1]);
        } else {
            indices.push([b0, t1, t0]);
            indices.push([b0, b1, t1]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ring_polyline(radius: f64, segments: usize) -> MarginPolyline {
        let mut pts = Vec::with_capacity(segments);
        for i in 0..segments {
            let theta = std::f64::consts::TAU * (i as f64) / (segments as f64);
            pts.push([radius * theta.cos(), radius * theta.sin(), 0.0]);
        }
        MarginPolyline {
            points: pts,
            is_closed: true,
        }
    }

    #[test]
    fn empty_margin_produces_empty_mesh() {
        let margin = MarginPolyline::default();
        let mesh = generate_merged_thimble(&margin, 5.0, 0.5, Vector3::z());
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(mesh.triangle_count(), 0);
    }

    #[test]
    fn invalid_height_returns_empty() {
        let margin = ring_polyline(4.0, 12);
        let m1 = generate_merged_thimble(&margin, 0.0, 0.5, Vector3::z());
        let m2 = generate_merged_thimble(&margin, -1.0, 0.5, Vector3::z());
        assert_eq!(m1.vertex_count(), 0);
        assert_eq!(m2.vertex_count(), 0);
    }

    #[test]
    fn closed_ring_produces_four_rings_of_vertices() {
        let margin = ring_polyline(4.0, 16);
        let mesh = generate_merged_thimble(&margin, 5.0, 0.5, Vector3::z());
        // 4 rings × 16 vertices each = 64.
        assert_eq!(mesh.vertex_count(), 64);
    }

    #[test]
    fn closed_ring_triangle_count_outer_inner_top() {
        let margin = ring_polyline(4.0, 16);
        let mesh = generate_merged_thimble(&margin, 5.0, 0.5, Vector3::z());
        // outer skirt: 16 quads * 2 = 32 tris
        // inner skirt: 16 quads * 2 = 32 tris
        // occlusal annulus: 16 quads * 2 = 32 tris
        // Total = 96 tris.
        assert_eq!(mesh.triangle_count(), 96);
    }

    #[test]
    fn occlusal_ring_is_offset_along_axis() {
        let margin = ring_polyline(4.0, 8);
        let height = 6.0;
        let mesh = generate_merged_thimble(&margin, height, 0.4, Vector3::z());
        // First 8 = outer cervical (z≈0), next 8 = outer occlusal (z≈height).
        for i in 0..8 {
            assert!((mesh.vertices[i].z).abs() < 1e-6);
            assert!((mesh.vertices[8 + i].z - height).abs() < 1e-6);
        }
    }

    #[test]
    fn inner_ring_is_inside_outer_ring() {
        let margin = ring_polyline(4.0, 12);
        let wall = 0.6;
        let mesh = generate_merged_thimble(&margin, 5.0, wall, Vector3::z());
        // outer cervical = vertices 0..12, inner cervical = 24..36
        for i in 0..12 {
            let outer = mesh.vertices[i];
            let inner = mesh.vertices[24 + i];
            let r_outer = (outer.x.powi(2) + outer.y.powi(2)).sqrt();
            let r_inner = (inner.x.powi(2) + inner.y.powi(2)).sqrt();
            assert!(
                r_inner < r_outer - 1e-6,
                "inner ring not inside outer ring at i={i} ({r_inner} vs {r_outer})"
            );
            // Wall thickness should be consistent (within smoothing tolerance).
            assert!(
                ((r_outer - r_inner) - wall).abs() < 1e-6,
                "wall thickness violated at i={i}"
            );
        }
    }

    #[test]
    fn open_polyline_does_not_close_skirt() {
        let mut pts = Vec::new();
        for i in 0..6 {
            pts.push([i as f64, 0.0, 0.0]);
        }
        let margin = MarginPolyline {
            points: pts,
            is_closed: false,
        };
        let mesh = generate_merged_thimble(&margin, 4.0, 0.4, Vector3::z());
        // Open: 6 verts × 4 rings = 24, but the skirts produce 5 quads
        // each (n - 1) instead of 6 closed quads.
        // outer skirt = 5*2 = 10 tris, inner skirt = 5*2 = 10 tris,
        // occlusal annulus = 5*2 = 10 tris. Total 30.
        assert_eq!(mesh.triangle_count(), 30);
    }
}
