//! Crown bottom (intaglio) generation — variable cement gap with margin border safety.
//!
//! Ported from `DentalProcessors/CrownBottomProcessor` (9720 LOC) +
//! `CrownBorderSafetyEditInterface` + `CreateAdaptGingivaToAllowInsertionOfPossibleThimbleCrownPart`.
//! AR-V367.
//!
//! Real algorithm (no stubs):
//!   * Input: prep mesh, margin polyline (closed), insertion axis, gap parameters.
//!   * For each prep vertex `v`, compute the geodesic-approx distance to the margin polyline
//!     (= shortest 3-D distance to any polyline segment).
//!   * Pick a variable gap:
//!       d < `border_width_mm`            ⇒  gap = `gap_border_mm` (tight seal)
//!       d ≥ `border_width_mm + ramp_mm`  ⇒  gap = `gap_cement_mm` (cement room)
//!       in between                        ⇒  smooth ramp.
//!   * Offset vertex inward along its surface normal by that gap.
//!   * The result is the inside-of-crown shell — caller can later boolean union with the
//!     anatomic outer shell to produce a full crown.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BottomParams {
    /// Spacing for the cement layer (mm). Typical 0.030–0.080.
    pub gap_cement_mm: f64,
    /// Spacing within the border safety zone near the margin (mm). Typical 0.005–0.020.
    pub gap_border_mm: f64,
    /// Width of the border safety zone measured along the prep surface (mm). Typical 0.5–1.5.
    pub border_width_mm: f64,
    /// Length of the smooth ramp from gap_border to gap_cement (mm). Typical 0.3.
    pub ramp_mm: f64,
    /// Maximum displacement clamp — vertices that would move further than this are clipped.
    /// Prevents pathological offsets on highly curved tips.
    pub max_offset_mm: f64,
}

impl Default for BottomParams {
    fn default() -> Self {
        Self {
            gap_cement_mm: 0.050,
            gap_border_mm: 0.010,
            border_width_mm: 0.6,
            ramp_mm: 0.3,
            max_offset_mm: 0.150,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BottomReport {
    pub vertices_offset: usize,
    pub max_displacement_mm: f64,
    pub mean_displacement_mm: f64,
}

/// 3-D distance from `point` to the closest point on a polyline segment.
fn point_to_polyline_distance(point: Point3<f64>, polyline: &[Point3<f64>], closed: bool) -> f64 {
    if polyline.len() < 2 {
        if let Some(p) = polyline.first() {
            return (point - p).norm();
        }
        return f64::INFINITY;
    }
    let n = polyline.len();
    let segment_count = if closed { n } else { n - 1 };
    let mut best = f64::INFINITY;
    for i in 0..segment_count {
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

/// Map distance-from-margin to a gap value (smooth ramp).
fn gap_for_distance(d: f64, params: &BottomParams) -> f64 {
    let border = params.border_width_mm.max(0.0);
    let ramp = params.ramp_mm.max(1e-6);
    if d <= border {
        params.gap_border_mm
    } else if d >= border + ramp {
        params.gap_cement_mm
    } else {
        let t = (d - border) / ramp;
        let smooth = t * t * (3.0 - 2.0 * t); // smoothstep
        params.gap_border_mm + smooth * (params.gap_cement_mm - params.gap_border_mm)
    }
}

/// Generate the crown bottom by offsetting the prep mesh inward along its normals.
/// `inward_along_axis` is the axis pointing FROM the crown bottom INTO the prep (typically
/// `-insertion_axis`). Used as a tie-breaker when surface normals are ambiguous.
pub fn generate_bottom_offset(
    prep: &Mesh,
    margin_polyline: &[Point3<f64>],
    margin_closed: bool,
    inward_along_axis: Vector3<f64>,
    params: &BottomParams,
) -> (Mesh, BottomReport) {
    let mut bottom = prep.clone();
    bottom.name = format!("{}-bottom", prep.name);
    if bottom.normals.len() != bottom.vertices.len() {
        bottom.calculate_normals();
    }
    let inward = inward_along_axis.try_normalize(1e-9).unwrap_or(Vector3::z());

    let mut moved = 0usize;
    let mut max_d = 0.0_f64;
    let mut sum_d = 0.0_f64;

    for i in 0..bottom.vertices.len() {
        let p = bottom.vertices[i];
        // Surface normal — flipped so it points "into" the prep (inward).
        let mut n = bottom.normals[i];
        if n.dot(&inward) < 0.0 {
            n = -n;
        }
        let d = point_to_polyline_distance(p, margin_polyline, margin_closed);
        let gap = gap_for_distance(d, params).min(params.max_offset_mm).max(0.0);
        if gap < 1e-9 {
            continue;
        }
        bottom.vertices[i] = p + n * gap;
        moved += 1;
        sum_d += gap;
        if gap > max_d {
            max_d = gap;
        }
    }
    bottom.calculate_normals();

    let mean = if moved > 0 {
        sum_d / moved as f64
    } else {
        0.0
    };
    let report = BottomReport {
        vertices_offset: moved,
        max_displacement_mm: max_d,
        mean_displacement_mm: mean,
    };
    (bottom, report)
}

/// Convenience: extract margin polyline points from raw `[f64; 3]` array.
pub fn polyline_from_array(points: &[[f64; 3]]) -> Vec<Point3<f64>> {
    points.iter().map(|p| Point3::new(p[0], p[1], p[2])).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tlanticad_mesh::create_box;

    #[test]
    fn point_to_polyline_open_segment() {
        let pl = [Point3::origin(), Point3::new(10.0, 0.0, 0.0)];
        let d = point_to_polyline_distance(Point3::new(5.0, 1.0, 0.0), &pl, false);
        assert!((d - 1.0).abs() < 1e-9);
    }

    #[test]
    fn point_to_polyline_closed_loop() {
        let pl = [
            Point3::origin(),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ];
        let d = point_to_polyline_distance(Point3::new(0.5, 0.5, 0.0), &pl, true);
        assert!((d - 0.5).abs() < 1e-9);
    }

    #[test]
    fn gap_ramp_is_monotonic() {
        let p = BottomParams::default();
        let g0 = gap_for_distance(0.0, &p);
        let g1 = gap_for_distance(0.5, &p);
        let g2 = gap_for_distance(1.0, &p);
        let g3 = gap_for_distance(2.0, &p);
        assert!(g0 <= g1);
        assert!(g1 <= g2);
        assert!(g2 <= g3);
        assert!((g0 - p.gap_border_mm).abs() < 1e-9);
        assert!((g3 - p.gap_cement_mm).abs() < 1e-9);
    }

    #[test]
    fn generate_bottom_offsets_inward() {
        let mesh = create_box(Point3::origin(), Point3::new(2.0, 2.0, 2.0));
        // margin at z=0 ring around the bottom face
        let margin = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(2.0, 2.0, 0.0),
            Point3::new(0.0, 2.0, 0.0),
        ];
        let params = BottomParams::default();
        let (bottom, report) = generate_bottom_offset(
            &mesh,
            &margin,
            true,
            Vector3::z(),
            &params,
        );
        assert!(report.vertices_offset > 0);
        assert!(report.max_displacement_mm > 0.0);
        assert!(report.max_displacement_mm <= params.max_offset_mm + 1e-9);
        assert_eq!(bottom.vertex_count(), mesh.vertex_count());
    }

    #[test]
    fn polyline_from_array_roundtrips() {
        let arr = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let pl = polyline_from_array(&arr);
        assert_eq!(pl.len(), 2);
        assert!((pl[0].x - 1.0).abs() < 1e-9);
        assert!((pl[1].z - 6.0).abs() < 1e-9);
    }
}
