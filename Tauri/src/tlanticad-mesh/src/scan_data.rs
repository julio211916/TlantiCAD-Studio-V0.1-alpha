//! Scan-data cleanup for freeform processors. AR-V418.
//!
//! Port of `DentalProcessors/FreeformScanDataProcessor.cs`. The original
//! processor wraps a raw scan mesh in a hygiene step before sending it to the
//! freeforming stage:
//!
//!   * tiny floating "islands" (loose triangle clusters from the scanner —
//!     hair, saliva strings, cotton fibers) are dropped,
//!   * small holes (probe gaps under contacts, scanner shadows on the
//!     gingiva) are closed by ear-clipping their boundary loop.
//!
//! We reimplement both steps in safe Rust:
//!   1. `connected_components` from `topology` segments the mesh into face
//!      clusters; clusters with fewer faces than `min_island_faces` get
//!      dropped (a face is removed by skipping it from the new index list,
//!      leaving its vertices orphaned, which the topology still treats as
//!      manifold for the surviving sub-mesh).
//!   2. `detect_holes` from `hole_fill` returns boundary loops; loops whose
//!      bounding-box diagonal stays under `hole_max_diameter_mm` are closed
//!      via `fill_hole_ear_clip`. Larger loops (real prep margins, scan
//!      borders) are left open.
//!
//! The function returns a `CleanedScan` carrying the new mesh plus a small
//! report so the UI can surface a "X islands removed, Y holes closed" toast.

use nalgebra::Point3;

use crate::hole_fill::{detect_holes, fill_hole_ear_clip};
use crate::topology::connected_components;
use crate::Mesh;

/// Result of `clean_scan_artifacts`.
#[derive(Debug, Clone)]
pub struct CleanedScan {
    pub mesh: Mesh,
    /// Number of connected components dropped because they were too small.
    pub islands_removed: usize,
    /// Number of boundary loops closed.
    pub holes_filled: usize,
    /// Number of triangles added by hole filling.
    pub triangles_added: usize,
}

/// Clean a raw scan mesh by removing small floating islands and closing
/// small holes. `min_island_faces == 0` disables the island filter;
/// `hole_max_diameter_mm <= 0` disables hole filling.
pub fn clean_scan_artifacts(
    scan_mesh: &Mesh,
    min_island_faces: usize,
    hole_max_diameter_mm: f64,
) -> CleanedScan {
    let mut report = CleanedScan {
        mesh: scan_mesh.clone(),
        islands_removed: 0,
        holes_filled: 0,
        triangles_added: 0,
    };
    if scan_mesh.indices.is_empty() {
        report.mesh.name = format!("{}_cleaned", scan_mesh.name);
        return report;
    }

    // ── 1. Drop small islands ───────────────────────────────────────────
    let mut working = scan_mesh.clone();
    if min_island_faces > 0 {
        let comps = connected_components(&working);
        if comps.len() > 1 {
            // Keep faces whose component is large enough.
            let mut keep = vec![false; working.indices.len()];
            for comp in &comps {
                if comp.len() >= min_island_faces {
                    for &fi in comp {
                        if fi < keep.len() {
                            keep[fi] = true;
                        }
                    }
                } else {
                    report.islands_removed += 1;
                }
            }
            working.indices = working
                .indices
                .iter()
                .enumerate()
                .filter_map(|(i, t)| if keep[i] { Some(*t) } else { None })
                .collect();
        }
    }

    // ── 2. Close small holes via ear-clipping their boundary loops ─────
    if hole_max_diameter_mm > 0.0 {
        let holes = detect_holes(&working.indices);
        for hole in &holes {
            if hole.vertices.len() < 3 {
                continue;
            }
            let diameter = boundary_loop_diameter(&working.vertices, &hole.vertices);
            if diameter > hole_max_diameter_mm {
                continue;
            }
            let new_tris = fill_hole_ear_clip(&working.vertices, hole);
            if !new_tris.is_empty() {
                report.holes_filled += 1;
                report.triangles_added += new_tris.len();
                working.indices.extend(new_tris);
            }
        }
    }

    working.calculate_normals();
    working.name = format!("{}_cleaned", scan_mesh.name);
    report.mesh = working;
    report
}

/// Bounding-box diagonal of a closed boundary loop (treated as the loop's
/// effective diameter).
fn boundary_loop_diameter(vertices: &[Point3<f64>], loop_verts: &[u32]) -> f64 {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for &i in loop_verts {
        let v = match vertices.get(i as usize) {
            Some(v) => v,
            None => continue,
        };
        for k in 0..3 {
            if v[k] < min[k] {
                min[k] = v[k];
            }
            if v[k] > max[k] {
                max[k] = v[k];
            }
        }
    }
    if min[0].is_infinite() {
        return 0.0;
    }
    let dx = max[0] - min[0];
    let dy = max[1] - min[1];
    let dz = max[2] - min[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_box, merge};

    #[test]
    fn empty_mesh_is_passthrough() {
        let m = Mesh::new("empty");
        let r = clean_scan_artifacts(&m, 4, 1.0);
        assert_eq!(r.islands_removed, 0);
        assert_eq!(r.holes_filled, 0);
        assert!(r.mesh.indices.is_empty());
        assert!(r.mesh.name.contains("cleaned"));
    }

    #[test]
    fn drops_small_island_keeps_large_box() {
        // big box (12 tris) merged with a tiny detached pyramid (4 tris).
        let big = create_box(Point3::origin(), Point3::new(10.0, 10.0, 10.0));
        let mut tiny = Mesh::new("tiny");
        tiny.vertices = vec![
            Point3::new(50.0, 50.0, 50.0),
            Point3::new(50.5, 50.0, 50.0),
            Point3::new(50.0, 50.5, 50.0),
            Point3::new(50.25, 50.25, 50.5),
        ];
        tiny.indices = vec![[0, 1, 2], [0, 1, 3], [1, 2, 3], [0, 2, 3]];
        let scan = merge(&big, &tiny);
        let before_tris = scan.indices.len();
        let r = clean_scan_artifacts(&scan, 5, 0.0);
        assert_eq!(r.islands_removed, 1);
        assert!(r.mesh.indices.len() < before_tris);
        assert!(r.mesh.indices.len() >= big.indices.len());
    }

    #[test]
    fn keeps_everything_when_threshold_zero() {
        let big = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let small = create_box(Point3::new(5.0, 5.0, 5.0), Point3::new(6.0, 6.0, 6.0));
        let scan = merge(&big, &small);
        let before = scan.indices.len();
        let r = clean_scan_artifacts(&scan, 0, 0.0);
        assert_eq!(r.islands_removed, 0);
        assert_eq!(r.mesh.indices.len(), before);
    }

    #[test]
    fn fills_small_triangle_hole() {
        // Two adjacent triangles sharing edge (0,1) → quad with one missing
        // triangle leaves a 3-vertex boundary loop. Removing one triangle
        // creates a hole with the other triangle's edges as the boundary.
        let mut m = Mesh::new("plate");
        m.vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.5, 0.5, 0.0),
        ];
        // Star fan: only 3 of 4 fan triangles → leaves one open wedge.
        m.indices = vec![[0, 1, 4], [1, 2, 4], [2, 3, 4]];
        let r = clean_scan_artifacts(&m, 0, 5.0);
        assert!(r.holes_filled >= 1);
        assert!(r.triangles_added >= 1);
    }

    #[test]
    fn skips_holes_that_are_too_large() {
        let mut m = Mesh::new("hole");
        m.vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(10.0, 0.0, 0.0),
            Point3::new(10.0, 10.0, 0.0),
            Point3::new(0.0, 10.0, 0.0),
            Point3::new(5.0, 5.0, 0.0),
        ];
        m.indices = vec![[0, 1, 4], [1, 2, 4], [2, 3, 4]]; // big open wedge
        let r = clean_scan_artifacts(&m, 0, 1.0); // tight 1 mm threshold
        assert_eq!(r.holes_filled, 0);
        assert_eq!(r.triangles_added, 0);
    }

    #[test]
    fn diameter_helper_computes_bbox_diagonal() {
        let verts = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(3.0, 0.0, 0.0),
            Point3::new(3.0, 4.0, 0.0),
        ];
        let d = boundary_loop_diameter(&verts, &[0, 1, 2]);
        assert!((d - 5.0).abs() < 1e-9); // 3-4-5 triangle bbox diagonal
    }
}
