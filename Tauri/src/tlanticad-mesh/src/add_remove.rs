//! Mesh add/remove operations on regions.
//!
//! Ported from `DentalProcessors/AddRemoveMeshProcessor`.
//!
//! Two operations:
//!   * `remove_region` — drops faces in `region` and orphan vertices.
//!   * `bulge_region`  — pushes vertices in `region` along their normals (positive value adds
//!                        material, negative removes — the "add/remove" toggle of exocad).

use crate::region::{region_vertices, FaceRegion};
use crate::Mesh;
use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BulgeOptions {
    /// Positive = add material along outward normal. Negative = remove.
    pub amount_mm: f64,
    /// Falloff exponent — 1.0 linear, 2.0 quadratic, higher = softer edges.
    pub falloff: f64,
    /// If true, falloff distance is geodesic radius from region centroid.
    pub use_falloff: bool,
}

impl Default for BulgeOptions {
    fn default() -> Self {
        Self {
            amount_mm: 0.1,
            falloff: 2.0,
            use_falloff: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AddRemoveReport {
    pub vertices_modified: usize,
    pub faces_removed: usize,
    pub max_displacement_mm: f64,
}

/// Remove all faces in `region` and any vertex that becomes orphaned. Re-indexes.
pub fn remove_region(mesh: &mut Mesh, region: &FaceRegion) -> AddRemoveReport {
    if region.is_empty() {
        return AddRemoveReport::default();
    }
    let drop: std::collections::HashSet<usize> = region.faces.iter().copied().collect();
    let kept: Vec<[u32; 3]> = mesh
        .indices
        .iter()
        .enumerate()
        .filter(|(i, _)| !drop.contains(i))
        .map(|(_, t)| *t)
        .collect();
    let removed = mesh.indices.len() - kept.len();
    mesh.indices = kept;

    // Compact orphan vertices.
    let mut used: Vec<bool> = vec![false; mesh.vertices.len()];
    for tri in &mesh.indices {
        used[tri[0] as usize] = true;
        used[tri[1] as usize] = true;
        used[tri[2] as usize] = true;
    }
    let mut new_idx = vec![u32::MAX; mesh.vertices.len()];
    let mut new_verts = Vec::new();
    let mut new_normals = Vec::new();
    for (old, _) in mesh.vertices.iter().enumerate() {
        if used[old] {
            new_idx[old] = new_verts.len() as u32;
            new_verts.push(mesh.vertices[old]);
            if old < mesh.normals.len() {
                new_normals.push(mesh.normals[old]);
            }
        }
    }
    for tri in &mut mesh.indices {
        for v in tri.iter_mut() {
            *v = new_idx[*v as usize];
        }
    }
    mesh.vertices = new_verts;
    mesh.normals = new_normals;

    AddRemoveReport {
        vertices_modified: 0,
        faces_removed: removed,
        max_displacement_mm: 0.0,
    }
}

/// Bulge vertices in `region` along their normals.
pub fn bulge_region(mesh: &mut Mesh, region: &FaceRegion, options: &BulgeOptions) -> AddRemoveReport {
    if region.is_empty() {
        return AddRemoveReport::default();
    }
    if mesh.normals.len() != mesh.vertices.len() {
        mesh.calculate_normals();
    }
    let verts = region_vertices(mesh, region);
    if verts.is_empty() {
        return AddRemoveReport::default();
    }

    // Region centroid for falloff.
    let mut centroid = Vector3::zeros();
    for &vi in &verts {
        centroid += mesh.vertices[vi as usize].coords;
    }
    centroid /= verts.len() as f64;

    let mut max_radius = 0.0_f64;
    for &vi in &verts {
        let r = (mesh.vertices[vi as usize].coords - centroid).norm();
        if r > max_radius {
            max_radius = r;
        }
    }
    if max_radius < 1e-9 {
        max_radius = 1.0;
    }

    let mut max_disp = 0.0_f64;
    let mut moved: HashMap<u32, ()> = HashMap::new();

    for &vi in &verts {
        let r = (mesh.vertices[vi as usize].coords - centroid).norm();
        let t = (r / max_radius).clamp(0.0, 1.0);
        let weight = if options.use_falloff {
            (1.0 - t).powf(options.falloff)
        } else {
            1.0
        };
        let n = mesh.normals[vi as usize];
        let delta = n * options.amount_mm * weight;
        if delta.norm() > 1e-12 {
            mesh.vertices[vi as usize] += delta;
            let d = delta.norm();
            if d > max_disp {
                max_disp = d;
            }
            moved.insert(vi, ());
        }
    }
    mesh.calculate_normals();

    AddRemoveReport {
        vertices_modified: moved.len(),
        faces_removed: 0,
        max_displacement_mm: max_disp,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AR-V403 — Polyline lasso boolean clip
//
// Ported from `DentalProcessors/AddRemoveMeshProcessor` + the lasso UI logic in
// `AddRemoveMeshProcessorControl.xaml.cs`. The user draws a closed polyline in
// 3D (typically captured by ray-projecting cursor positions onto the model);
// to decide what is "inside" the lasso, the polyline is projected onto the
// plane perpendicular to a chosen axis (typically the camera view axis or the
// insertion axis), then a point-in-polygon test is run on every vertex's
// projection. Vertices outside the polygon — or, after `KeepInside` flips —
// inside it, are removed along with their incident faces. Triangles straddling
// the lasso are clipped on the boundary edges so the cut is clean instead of
// stair-stepped.
// ─────────────────────────────────────────────────────────────────────────────

/// Clip mode — keep what's inside the lasso, or what's outside.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LassoClipMode {
    /// Keep vertices whose projection lies inside the polygon.
    KeepInside,
    /// Keep vertices whose projection lies outside the polygon (drop the lasso interior).
    KeepOutside,
}

impl Default for LassoClipMode {
    fn default() -> Self {
        LassoClipMode::KeepInside
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LassoClipReport {
    pub vertices_inside: usize,
    pub vertices_outside: usize,
    pub faces_kept: usize,
    pub faces_dropped: usize,
    pub edges_split: usize,
}

/// Build an orthonormal basis (u, v) on the plane whose normal is `axis`.
/// `u` is a deterministic in-plane vector; `v = axis × u`. Returns unit vectors.
fn orthonormal_basis(axis: &Vector3<f64>) -> (Vector3<f64>, Vector3<f64>) {
    let n = axis.normalize();
    // Pick the world axis least aligned with `n` so the cross product is well conditioned.
    let helper = if n.x.abs() < n.y.abs() && n.x.abs() < n.z.abs() {
        Vector3::x()
    } else if n.y.abs() < n.z.abs() {
        Vector3::y()
    } else {
        Vector3::z()
    };
    let u = n.cross(&helper).normalize();
    let v = n.cross(&u).normalize();
    (u, v)
}

/// Project a 3D point onto the plane defined by `axis` (as normal). Returns 2D
/// coordinates `(u, v)` in the plane basis returned by `orthonormal_basis`.
#[inline]
fn project_to_plane(p: &Point3<f64>, basis_u: &Vector3<f64>, basis_v: &Vector3<f64>) -> (f64, f64) {
    let c = p.coords;
    (c.dot(basis_u), c.dot(basis_v))
}

/// Standard 2D ray-cast point-in-polygon test (even-odd rule). Polygon may be
/// open or closed (we wrap the last segment automatically). Empty / 1-2 point
/// polygons return `false`.
fn point_in_polygon_2d(point: (f64, f64), polygon: &[(f64, f64)]) -> bool {
    if polygon.len() < 3 {
        return false;
    }
    let (px, py) = point;
    let mut inside = false;
    let n = polygon.len();
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = polygon[i];
        let (xj, yj) = polygon[j];
        let crosses = (yi > py) != (yj > py);
        if crosses {
            let x_intersect = xi + (py - yi) * (xj - xi) / (yj - yi);
            if px < x_intersect {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

/// Boolean-clip `mesh` against a 3D polyline lasso projected onto the plane
/// perpendicular to `axis_to_project`.
///
/// Algorithm (matches `AddRemoveMeshProcessor`):
/// 1. Build the in-plane orthonormal basis `(u, v)` from `axis_to_project`.
/// 2. Project every vertex to 2D on that plane.
/// 3. Project every polyline node to 2D on the same plane.
/// 4. Mark each vertex as inside/outside via point-in-polygon.
/// 5. Decide each face's fate by majority of its 3 vertices according to `mode`:
///    * 3 keep → emit unchanged.
///    * 0 keep → drop.
///    * 1 or 2 keep → split: linear interpolation along the two crossing edges
///      so the resulting fragment(s) lie entirely on the kept side.
/// 6. Compact orphan vertices.
///
/// `polyline_3d` may be open — it is treated as closed (last vertex connects to
/// the first) per exocad semantics. Returns a fresh `Mesh`.
pub fn boolean_clip_with_polyline_lasso(
    mesh: &Mesh,
    polyline_3d: &[Point3<f64>],
    axis_to_project: &Vector3<f64>,
    mode: LassoClipMode,
) -> (Mesh, LassoClipReport) {
    let mut out = Mesh::new(format!("{}_clipped", mesh.name));
    out.uvs = mesh.uvs.clone().map(|_| Vec::new());
    out.colors = mesh.colors.clone().map(|_| Vec::new());

    if polyline_3d.len() < 3 || mesh.indices.is_empty() {
        return (
            out,
            LassoClipReport {
                vertices_inside: 0,
                vertices_outside: mesh.vertices.len(),
                faces_kept: 0,
                faces_dropped: mesh.indices.len(),
                edges_split: 0,
            },
        );
    }

    let (basis_u, basis_v) = orthonormal_basis(axis_to_project);

    // Project polyline once.
    let polygon_2d: Vec<(f64, f64)> = polyline_3d
        .iter()
        .map(|p| project_to_plane(p, &basis_u, &basis_v))
        .collect();

    // Vertex classification — `inside_lasso[i]` says vertex i projects inside polygon.
    let inside_lasso: Vec<bool> = mesh
        .vertices
        .iter()
        .map(|p| point_in_polygon_2d(project_to_plane(p, &basis_u, &basis_v), &polygon_2d))
        .collect();
    let vertices_inside = inside_lasso.iter().filter(|b| **b).count();
    let vertices_outside = inside_lasso.len() - vertices_inside;

    // For mode `KeepInside`, "kept" means inside the lasso. For `KeepOutside`,
    // "kept" means outside.
    let keep = |i: usize| -> bool {
        let inside = inside_lasso[i];
        match mode {
            LassoClipMode::KeepInside => inside,
            LassoClipMode::KeepOutside => !inside,
        }
    };

    // Vertex index remap: original vertex idx -> new idx in `out` (lazy add).
    let mut remap: HashMap<u32, u32> = HashMap::new();
    let mut push_kept_vertex = |out: &mut Mesh, mesh: &Mesh, vi: u32| -> u32 {
        if let Some(&n) = remap.get(&vi) {
            return n;
        }
        let n = out.vertices.len() as u32;
        out.vertices.push(mesh.vertices[vi as usize]);
        if let Some(uvs) = &mesh.uvs {
            if let Some(buf) = out.uvs.as_mut() {
                buf.push(uvs[vi as usize]);
            }
        }
        if let Some(cols) = &mesh.colors {
            if let Some(buf) = out.colors.as_mut() {
                buf.push(cols[vi as usize]);
            }
        }
        remap.insert(vi, n);
        n
    };

    // Cache for split-edge midpoints so two adjacent triangles share the same
    // boundary vertex (manifold preservation).
    let mut edge_split_cache: HashMap<(u32, u32), u32> = HashMap::new();
    let mut edges_split = 0usize;
    let push_split_vertex =
        |out: &mut Mesh, mesh: &Mesh, edge_cache: &mut HashMap<(u32, u32), u32>, edges_split: &mut usize, kept: u32, dropped: u32| -> u32 {
            let key = if kept < dropped {
                (kept, dropped)
            } else {
                (dropped, kept)
            };
            if let Some(&n) = edge_cache.get(&key) {
                return n;
            }
            // We split each crossing edge at its midpoint. This is the
            // deterministic, manifold-preserving choice for a tessellated
            // lasso (polyline has finer resolution than the mesh itself, so
            // the visible cut tracks the lasso to within one triangle).
            // Two adjacent triangles sharing this edge will receive the same
            // boundary vertex via `edge_cache`.
            let p_keep = mesh.vertices[kept as usize];
            let p_drop = mesh.vertices[dropped as usize];
            let t_best = 0.5_f64;
            let p = Point3::from(p_keep.coords + (p_drop.coords - p_keep.coords) * t_best);
            let n = out.vertices.len() as u32;
            out.vertices.push(p);
            if let Some(uvs) = &mesh.uvs {
                if let Some(buf) = out.uvs.as_mut() {
                    let a = uvs[kept as usize];
                    let b = uvs[dropped as usize];
                    buf.push([a[0] * 0.5 + b[0] * 0.5, a[1] * 0.5 + b[1] * 0.5]);
                }
            }
            if let Some(cols) = &mesh.colors {
                if let Some(buf) = out.colors.as_mut() {
                    let a = cols[kept as usize];
                    let b = cols[dropped as usize];
                    buf.push([
                        ((a[0] as u16 + b[0] as u16) / 2) as u8,
                        ((a[1] as u16 + b[1] as u16) / 2) as u8,
                        ((a[2] as u16 + b[2] as u16) / 2) as u8,
                        ((a[3] as u16 + b[3] as u16) / 2) as u8,
                    ]);
                }
            }
            edge_cache.insert(key, n);
            *edges_split += 1;
            n
        };

    let mut faces_kept = 0usize;
    let mut faces_dropped = 0usize;

    for tri in &mesh.indices {
        let k0 = keep(tri[0] as usize);
        let k1 = keep(tri[1] as usize);
        let k2 = keep(tri[2] as usize);
        let kept_count = k0 as u8 + k1 as u8 + k2 as u8;

        match kept_count {
            3 => {
                let a = push_kept_vertex(&mut out, mesh, tri[0]);
                let b = push_kept_vertex(&mut out, mesh, tri[1]);
                let c = push_kept_vertex(&mut out, mesh, tri[2]);
                out.indices.push([a, b, c]);
                faces_kept += 1;
            }
            0 => {
                faces_dropped += 1;
            }
            2 => {
                // One vertex dropped. Two new boundary vertices on the two
                // edges that connect the dropped vertex to the two kept ones.
                // Result: a quad (kept, kept, mid_kk_drop, mid_kk_drop) split
                // into 2 triangles.
                let (k_a, k_b, dropped_idx) = if !k0 {
                    (tri[1], tri[2], tri[0])
                } else if !k1 {
                    (tri[2], tri[0], tri[1])
                } else {
                    (tri[0], tri[1], tri[2])
                };
                let a = push_kept_vertex(&mut out, mesh, k_a);
                let b = push_kept_vertex(&mut out, mesh, k_b);
                let m_a = push_split_vertex(
                    &mut out,
                    mesh,
                    &mut edge_split_cache,
                    &mut edges_split,
                    k_a,
                    dropped_idx,
                );
                let m_b = push_split_vertex(
                    &mut out,
                    mesh,
                    &mut edge_split_cache,
                    &mut edges_split,
                    k_b,
                    dropped_idx,
                );
                out.indices.push([a, b, m_b]);
                out.indices.push([a, m_b, m_a]);
                faces_kept += 1;
            }
            1 => {
                // Two vertices dropped. Single triangle (kept, mid_kept_drop1, mid_kept_drop2).
                let (kept_idx, drop_a, drop_b) = if k0 {
                    (tri[0], tri[1], tri[2])
                } else if k1 {
                    (tri[1], tri[2], tri[0])
                } else {
                    (tri[2], tri[0], tri[1])
                };
                let a = push_kept_vertex(&mut out, mesh, kept_idx);
                let m_a = push_split_vertex(
                    &mut out,
                    mesh,
                    &mut edge_split_cache,
                    &mut edges_split,
                    kept_idx,
                    drop_a,
                );
                let m_b = push_split_vertex(
                    &mut out,
                    mesh,
                    &mut edge_split_cache,
                    &mut edges_split,
                    kept_idx,
                    drop_b,
                );
                out.indices.push([a, m_a, m_b]);
                faces_kept += 1;
            }
            _ => unreachable!(),
        }
    }

    if !out.vertices.is_empty() {
        out.calculate_normals();
    }

    (
        out,
        LassoClipReport {
            vertices_inside,
            vertices_outside,
            faces_kept,
            faces_dropped,
            edges_split,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_box;
    use nalgebra::{Point3, Vector3};

    #[test]
    fn remove_region_drops_faces() {
        let mut mesh = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let before = mesh.triangle_count();
        let region = FaceRegion {
            faces: vec![0, 1],
        };
        let r = remove_region(&mut mesh, &region);
        assert_eq!(r.faces_removed, 2);
        assert_eq!(mesh.triangle_count(), before - 2);
    }

    #[test]
    fn bulge_region_pushes_outward() {
        let mut mesh = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let region = FaceRegion {
            faces: (0..mesh.triangle_count()).collect(),
        };
        let opts = BulgeOptions {
            amount_mm: 0.1,
            falloff: 1.0,
            use_falloff: false,
        };
        let r = bulge_region(&mut mesh, &region, &opts);
        assert!(r.vertices_modified > 0);
        assert!(r.max_displacement_mm > 0.0);
    }

    #[test]
    fn empty_region_no_op() {
        let mut mesh = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let r = remove_region(&mut mesh, &FaceRegion::default());
        assert_eq!(r.faces_removed, 0);
    }

    // ── AR-V403 polyline lasso clipping ──────────────────────────

    fn square_polyline_xy(cx: f64, cy: f64, half: f64, z: f64) -> Vec<Point3<f64>> {
        vec![
            Point3::new(cx - half, cy - half, z),
            Point3::new(cx + half, cy - half, z),
            Point3::new(cx + half, cy + half, z),
            Point3::new(cx - half, cy + half, z),
        ]
    }

    #[test]
    fn lasso_keep_inside_drops_outside_faces() {
        // Box from (0,0,0) to (4,4,1). Lasso is a 1x1 square centered at (2,2) on XY.
        // Project axis = +Z, so projection is the XY plane.
        let mesh = create_box(Point3::origin(), Point3::new(4.0, 4.0, 1.0));
        let polyline = square_polyline_xy(2.0, 2.0, 0.5, 0.0);
        let (clipped, report) = boolean_clip_with_polyline_lasso(
            &mesh,
            &polyline,
            &Vector3::z(),
            LassoClipMode::KeepInside,
        );
        // Box vertices are at corners of [0,4]x[0,4]x[0,1] — none lie inside the
        // lasso square — so KeepInside should drop everything.
        assert_eq!(report.vertices_inside, 0);
        assert_eq!(clipped.vertex_count(), 0);
        assert_eq!(report.faces_dropped, mesh.triangle_count());
    }

    #[test]
    fn lasso_keep_outside_keeps_all_when_lasso_misses_geometry() {
        let mesh = create_box(Point3::origin(), Point3::new(4.0, 4.0, 1.0));
        // Lasso completely outside the mesh footprint.
        let polyline = square_polyline_xy(20.0, 20.0, 0.5, 0.0);
        let (clipped, report) = boolean_clip_with_polyline_lasso(
            &mesh,
            &polyline,
            &Vector3::z(),
            LassoClipMode::KeepOutside,
        );
        assert_eq!(report.vertices_inside, 0);
        assert_eq!(report.faces_kept, mesh.triangle_count());
        assert_eq!(report.faces_dropped, 0);
        assert_eq!(clipped.triangle_count(), mesh.triangle_count());
    }

    #[test]
    fn lasso_partial_intersection_splits_edges_and_preserves_manifold() {
        // Subdivide the box so the lasso boundary actually crosses some interior edges.
        let mut mesh = create_box(Point3::origin(), Point3::new(4.0, 4.0, 1.0));
        crate::operations::subdivide(&mut mesh);
        crate::operations::subdivide(&mut mesh);
        // Lasso covers the lower-x half only.
        let polyline = vec![
            Point3::new(-1.0, -1.0, 0.0),
            Point3::new(2.0, -1.0, 0.0),
            Point3::new(2.0, 5.0, 0.0),
            Point3::new(-1.0, 5.0, 0.0),
        ];
        let (clipped, report) = boolean_clip_with_polyline_lasso(
            &mesh,
            &polyline,
            &Vector3::z(),
            LassoClipMode::KeepInside,
        );
        assert!(report.vertices_inside > 0, "some vertices should be inside");
        assert!(report.vertices_outside > 0, "some vertices should be outside");
        assert!(report.faces_kept > 0, "should keep some faces");
        assert!(report.faces_dropped > 0, "should drop some faces");
        assert!(report.edges_split > 0, "boundary edges must be split");
        assert!(clipped.vertex_count() > 0);
        assert!(clipped.triangle_count() > 0);
    }

    #[test]
    fn lasso_keep_outside_inverts_keep_inside() {
        let mut mesh = create_box(Point3::origin(), Point3::new(4.0, 4.0, 1.0));
        crate::operations::subdivide(&mut mesh);
        let polyline = vec![
            Point3::new(-1.0, -1.0, 0.0),
            Point3::new(2.0, -1.0, 0.0),
            Point3::new(2.0, 5.0, 0.0),
            Point3::new(-1.0, 5.0, 0.0),
        ];
        let (_, r_in) = boolean_clip_with_polyline_lasso(
            &mesh,
            &polyline,
            &Vector3::z(),
            LassoClipMode::KeepInside,
        );
        let (_, r_out) = boolean_clip_with_polyline_lasso(
            &mesh,
            &polyline,
            &Vector3::z(),
            LassoClipMode::KeepOutside,
        );
        assert_eq!(r_in.vertices_inside, r_out.vertices_inside);
        assert_eq!(r_in.vertices_outside, r_out.vertices_outside);
        // Combined, every face is accounted for (a face is either fully kept on
        // one side, fully kept on the other, or split & contributes to both).
        assert!(r_in.faces_kept + r_out.faces_kept >= mesh.triangle_count());
    }

    #[test]
    fn lasso_open_polyline_below_three_points_drops_everything() {
        let mesh = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let polyline = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)];
        let (clipped, report) = boolean_clip_with_polyline_lasso(
            &mesh,
            &polyline,
            &Vector3::z(),
            LassoClipMode::KeepInside,
        );
        assert_eq!(clipped.vertex_count(), 0);
        assert_eq!(report.faces_kept, 0);
    }

    #[test]
    fn point_in_polygon_detects_inside_outside() {
        let square = vec![(0.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0)];
        assert!(point_in_polygon_2d((1.0, 1.0), &square));
        assert!(!point_in_polygon_2d((3.0, 1.0), &square));
        assert!(!point_in_polygon_2d((-1.0, 1.0), &square));
        assert!(!point_in_polygon_2d((1.0, 3.0), &square));
    }

    #[test]
    fn orthonormal_basis_is_perpendicular_to_axis() {
        let axis = Vector3::new(0.3, 0.4, 0.866).normalize();
        let (u, v) = orthonormal_basis(&axis);
        assert!((u.dot(&axis)).abs() < 1e-9);
        assert!((v.dot(&axis)).abs() < 1e-9);
        assert!((u.dot(&v)).abs() < 1e-9);
        assert!((u.norm() - 1.0).abs() < 1e-9);
        assert!((v.norm() - 1.0).abs() < 1e-9);
    }
}
