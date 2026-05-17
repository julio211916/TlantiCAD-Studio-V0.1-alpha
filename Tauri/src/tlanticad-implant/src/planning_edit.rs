//! Mesh editing for implant planning. AR-V389.
//!
//! Ported from `DentalProcessors/EditMeshForImplantPlanningProcessor.cs` +
//! `DeleteReconstructionsForImplantPlanningProcessor.cs`. The original processors
//! manipulated `ToothPart` meshes inside a DentalData jaw — they tracked checksums
//! per part (`meshhashes`), wiped attached reconstructions, and let the user
//! sculpt the mesh region used by implant planning. We expose the equivalent as
//! three pure mesh ops the wizard can chain:
//!
//!   * `trim_below_plane` — drops every triangle whose centroid lies below
//!     `plane_origin · plane_normal`. Mirrors the "scrape soft tissue / clean
//!     mucosa" gesture in the planning preview.
//!   * `weld_islands_above_threshold` — merges vertices that fall within
//!     `weld_radius_mm` of one another after trim. The C# version called the
//!     internal CSGRS `merge_close_points` after edits to keep the topology
//!     watertight.
//!   * `drop_floating_components` — `DeleteReconstructionsForImplantPlanningProcessor`
//!     dropped any reconstruction (crown / abutment) that no longer touched the
//!     planned implant cylinder; the mesh-level analogue is dropping connected
//!     components below `min_component_volume_mm3` so isolated debris doesn't
//!     leak into the AI planner input.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlanningEditReport {
    pub triangles_before: usize,
    pub triangles_after: usize,
    pub vertices_before: usize,
    pub vertices_after: usize,
    pub components_dropped: usize,
}

impl PlanningEditReport {
    fn from_diff(before: &Mesh, after: &Mesh, components_dropped: usize) -> Self {
        Self {
            triangles_before: before.triangle_count(),
            triangles_after: after.triangle_count(),
            vertices_before: before.vertex_count(),
            vertices_after: after.vertex_count(),
            components_dropped,
        }
    }
}

/// Drop every triangle whose centroid is on the negative side of the half-space
/// `(p - plane_origin) · plane_normal >= 0`. Vertices not referenced anymore are
/// stripped to keep the mesh tight.
pub fn trim_below_plane(
    mesh: &Mesh,
    plane_origin: Point3<f64>,
    plane_normal: Vector3<f64>,
) -> (Mesh, PlanningEditReport) {
    let n = plane_normal.try_normalize(1e-9).unwrap_or(Vector3::z());
    let mut kept_indices: Vec<[u32; 3]> = Vec::with_capacity(mesh.indices.len());
    for tri in &mesh.indices {
        let a = mesh.vertices[tri[0] as usize];
        let b = mesh.vertices[tri[1] as usize];
        let c = mesh.vertices[tri[2] as usize];
        let centroid = Point3::from((a.coords + b.coords + c.coords) / 3.0);
        if (centroid - plane_origin).dot(&n) >= 0.0 {
            kept_indices.push(*tri);
        }
    }
    let out = compact_mesh(mesh, &kept_indices);
    let report = PlanningEditReport::from_diff(mesh, &out, 0);
    (out, report)
}

/// Snap vertices that fall within `weld_radius_mm` to a shared representative,
/// then rebuild indices. O(N²) — acceptable for the planning-region meshes
/// (typically ≤ 8k vertices) the processor handled.
pub fn weld_islands_above_threshold(mesh: &Mesh, weld_radius_mm: f64) -> (Mesh, PlanningEditReport) {
    if weld_radius_mm <= 0.0 || mesh.vertices.is_empty() {
        return (mesh.clone(), PlanningEditReport::from_diff(mesh, mesh, 0));
    }
    let r2 = weld_radius_mm * weld_radius_mm;
    let n = mesh.vertices.len();
    let mut remap: Vec<usize> = (0..n).collect();
    for i in 0..n {
        if remap[i] != i {
            continue;
        }
        for j in (i + 1)..n {
            if remap[j] != j {
                continue;
            }
            let d2 = (mesh.vertices[i] - mesh.vertices[j]).norm_squared();
            if d2 <= r2 {
                remap[j] = i;
            }
        }
    }

    // Build new vertex list compacting parents.
    let mut new_index_for_parent: Vec<i32> = vec![-1; n];
    let mut new_vertices: Vec<Point3<f64>> = Vec::new();
    for i in 0..n {
        let parent = remap[i];
        if new_index_for_parent[parent] < 0 {
            new_index_for_parent[parent] = new_vertices.len() as i32;
            new_vertices.push(mesh.vertices[parent]);
        }
    }

    let mut new_indices: Vec<[u32; 3]> = Vec::with_capacity(mesh.indices.len());
    for tri in &mesh.indices {
        let a = new_index_for_parent[remap[tri[0] as usize]] as u32;
        let b = new_index_for_parent[remap[tri[1] as usize]] as u32;
        let c = new_index_for_parent[remap[tri[2] as usize]] as u32;
        if a == b || b == c || a == c {
            continue;
        }
        new_indices.push([a, b, c]);
    }

    let mut out = Mesh::new(&mesh.name);
    out.vertices = new_vertices;
    out.indices = new_indices;
    out.calculate_normals();
    let report = PlanningEditReport::from_diff(mesh, &out, 0);
    (out, report)
}

/// Drop every connected component whose absolute signed volume is under
/// `min_component_volume_mm3`. Component ids come from a union-find on the
/// triangle adjacency graph (shared edges).
pub fn drop_floating_components(mesh: &Mesh, min_component_volume_mm3: f64) -> (Mesh, PlanningEditReport) {
    let tri_count = mesh.indices.len();
    if tri_count == 0 {
        return (mesh.clone(), PlanningEditReport::from_diff(mesh, mesh, 0));
    }
    let mut parent: Vec<usize> = (0..tri_count).collect();
    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }
    fn union(parent: &mut [usize], a: usize, b: usize) {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra != rb {
            parent[ra] = rb;
        }
    }

    // Map directed half-edge → (triangle, opposite half-edge) to detect shared edges.
    use std::collections::HashMap;
    let mut edge_to_tri: HashMap<(u32, u32), usize> = HashMap::new();
    for (idx, tri) in mesh.indices.iter().enumerate() {
        for &(a, b) in &[(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            let key = if a < b { (a, b) } else { (b, a) };
            if let Some(&other) = edge_to_tri.get(&key) {
                union(&mut parent, idx, other);
            } else {
                edge_to_tri.insert(key, idx);
            }
        }
    }

    let mut comp_volume: HashMap<usize, f64> = HashMap::new();
    for (idx, tri) in mesh.indices.iter().enumerate() {
        let v0 = mesh.vertices[tri[0] as usize];
        let v1 = mesh.vertices[tri[1] as usize];
        let v2 = mesh.vertices[tri[2] as usize];
        let signed = v0.coords.dot(&v1.coords.cross(&v2.coords)) / 6.0;
        let root = find(&mut parent, idx);
        *comp_volume.entry(root).or_insert(0.0) += signed;
    }

    let mut kept_indices: Vec<[u32; 3]> = Vec::with_capacity(tri_count);
    let mut dropped_components = 0usize;
    let mut dropped_seen: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for (idx, tri) in mesh.indices.iter().enumerate() {
        let root = find(&mut parent, idx);
        let vol = comp_volume.get(&root).copied().unwrap_or(0.0).abs();
        if vol >= min_component_volume_mm3 {
            kept_indices.push(*tri);
        } else if dropped_seen.insert(root) {
            dropped_components += 1;
        }
    }
    let out = compact_mesh(mesh, &kept_indices);
    let report = PlanningEditReport::from_diff(mesh, &out, dropped_components);
    (out, report)
}

/// Re-index the kept triangle list against `mesh.vertices`, dropping orphan
/// vertices and recomputing normals.
fn compact_mesh(mesh: &Mesh, kept_indices: &[[u32; 3]]) -> Mesh {
    let mut new_index_for_old: Vec<i32> = vec![-1; mesh.vertices.len()];
    let mut new_vertices: Vec<Point3<f64>> = Vec::new();
    let mut new_indices: Vec<[u32; 3]> = Vec::with_capacity(kept_indices.len());
    for tri in kept_indices {
        let mut remapped = [0u32; 3];
        for (k, &old) in tri.iter().enumerate() {
            let old_usize = old as usize;
            if new_index_for_old[old_usize] < 0 {
                new_index_for_old[old_usize] = new_vertices.len() as i32;
                new_vertices.push(mesh.vertices[old_usize]);
            }
            remapped[k] = new_index_for_old[old_usize] as u32;
        }
        new_indices.push(remapped);
    }
    let mut out = Mesh::new(&mesh.name);
    out.vertices = new_vertices;
    out.indices = new_indices;
    out.calculate_normals();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_cube_two_layers() -> Mesh {
        // Two stacked tetrahedra: one below z=0, one above. Each tetra contributes
        // 4 triangles. Shared apex vertex between them so they're one component
        // for `drop_floating_components`.
        let mut mesh = Mesh::new("two-layers");
        // Lower tetra (z negative): apex below, base at z=-1.
        mesh.vertices = vec![
            Point3::new(0.0, 0.0, -2.0),  // 0 lower apex
            Point3::new(1.0, 0.0, -1.0),  // 1
            Point3::new(-1.0, 0.0, -1.0), // 2
            Point3::new(0.0, 1.0, -1.0),  // 3
            Point3::new(0.0, 0.0, 2.0),   // 4 upper apex
            Point3::new(1.0, 0.0, 1.0),   // 5
            Point3::new(-1.0, 0.0, 1.0),  // 6
            Point3::new(0.0, 1.0, 1.0),   // 7
        ];
        mesh.indices = vec![
            [0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2], // lower
            [4, 5, 6], [4, 6, 7], [4, 7, 5], [5, 7, 6], // upper
        ];
        mesh.calculate_normals();
        mesh
    }

    fn pyramid(centroid: Point3<f64>, scale: f64, vertex_offset: u32) -> (Vec<Point3<f64>>, Vec<[u32; 3]>) {
        let v = vec![
            centroid + Vector3::new(0.0, 0.0, scale),
            centroid + Vector3::new(scale, 0.0, -scale * 0.5),
            centroid + Vector3::new(-scale * 0.5, scale * 0.86, -scale * 0.5),
            centroid + Vector3::new(-scale * 0.5, -scale * 0.86, -scale * 0.5),
        ];
        let i = vec![
            [vertex_offset, vertex_offset + 1, vertex_offset + 2],
            [vertex_offset, vertex_offset + 2, vertex_offset + 3],
            [vertex_offset, vertex_offset + 3, vertex_offset + 1],
            [vertex_offset + 1, vertex_offset + 3, vertex_offset + 2],
        ];
        (v, i)
    }

    #[test]
    fn trim_drops_lower_tetra() {
        let mesh = unit_cube_two_layers();
        let (out, report) = trim_below_plane(
            &mesh,
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        );
        assert_eq!(report.triangles_before, 8);
        assert_eq!(report.triangles_after, 4);
        // All kept centroids should have z >= 0.
        for tri in &out.indices {
            let c = (out.vertices[tri[0] as usize].coords
                + out.vertices[tri[1] as usize].coords
                + out.vertices[tri[2] as usize].coords)
                / 3.0;
            assert!(c.z >= -1e-9);
        }
    }

    #[test]
    fn weld_collapses_close_duplicates() {
        let mut mesh = Mesh::new("dup");
        mesh.vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            // duplicate of vertex 0 within 0.001 mm.
            Point3::new(0.0005, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(2.0, 1.0, 0.0),
        ];
        mesh.indices = vec![[0, 1, 2], [3, 4, 5]];
        mesh.calculate_normals();
        let (out, _) = weld_islands_above_threshold(&mesh, 0.01);
        // Vertex 0 and 3 should be merged.
        assert!(out.vertices.len() < 6);
        // The first triangle still survives.
        assert_eq!(out.indices.len(), 2);
    }

    #[test]
    fn drop_floating_keeps_main_body() {
        let mut mesh = Mesh::new("multi");
        let (big_v, big_i) = pyramid(Point3::origin(), 5.0, 0);
        let (small_v, small_i) = pyramid(Point3::new(50.0, 0.0, 0.0), 0.4, big_v.len() as u32);
        mesh.vertices = [big_v.as_slice(), small_v.as_slice()].concat();
        mesh.indices = [big_i.as_slice(), small_i.as_slice()].concat();
        mesh.calculate_normals();
        let (out, report) = drop_floating_components(&mesh, 1.0);
        assert_eq!(report.components_dropped, 1);
        // The small pyramid was 4 triangles.
        assert_eq!(out.triangle_count(), 4);
    }

    #[test]
    fn empty_mesh_round_trips() {
        let mesh = Mesh::new("empty");
        let (out, _) = trim_below_plane(&mesh, Point3::origin(), Vector3::z());
        assert_eq!(out.triangle_count(), 0);
        let (out, _) = weld_islands_above_threshold(&mesh, 0.1);
        assert_eq!(out.vertex_count(), 0);
        let (out, report) = drop_floating_components(&mesh, 1.0);
        assert_eq!(out.triangle_count(), 0);
        assert_eq!(report.components_dropped, 0);
    }
}
