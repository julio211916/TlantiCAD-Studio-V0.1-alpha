//! Mesh algorithms: repair, analysis, collision detection

use nalgebra::{Point3, Vector3};
use crate::Mesh;
use std::collections::HashMap;

/// Check if mesh is watertight (manifold)
pub fn is_watertight(mesh: &Mesh) -> bool {
    let mut edge_count: HashMap<(u32, u32), i32> = HashMap::new();
    for tri in &mesh.indices {
        for i in 0..3 {
            let a = tri[i];
            let b = tri[(i + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    edge_count.values().all(|&c| c == 2)
}

/// Find boundary edges (non-manifold)
pub fn boundary_edges(mesh: &Mesh) -> Vec<(u32, u32)> {
    let mut edge_count: HashMap<(u32, u32), i32> = HashMap::new();
    for tri in &mesh.indices {
        for i in 0..3 {
            let a = tri[i];
            let b = tri[(i + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    edge_count.into_iter()
        .filter(|(_, c)| *c == 1)
        .map(|(e, _)| e)
        .collect()
}

/// Find duplicate/degenerate triangles and remove them
pub fn remove_degenerate(mesh: &mut Mesh, min_area: f64) {
    mesh.indices.retain(|tri| {
        let v0 = mesh.vertices[tri[0] as usize];
        let v1 = mesh.vertices[tri[1] as usize];
        let v2 = mesh.vertices[tri[2] as usize];
        let area = (v1 - v0).cross(&(v2 - v0)).norm() * 0.5;
        area > min_area && tri[0] != tri[1] && tri[1] != tri[2] && tri[0] != tri[2]
    });
}

/// Weld vertices within a tolerance distance
pub fn weld_vertices(mesh: &mut Mesh, tolerance: f64) {
    let n = mesh.vertices.len();
    let mut remap: Vec<u32> = (0..n as u32).collect();
    let tol2 = tolerance * tolerance;

    for i in 0..n {
        if remap[i] != i as u32 { continue; }
        for j in (i + 1)..n {
            if remap[j] != j as u32 { continue; }
            let d2 = (mesh.vertices[i] - mesh.vertices[j]).norm_squared();
            if d2 < tol2 {
                remap[j] = i as u32;
            }
        }
    }

    for tri in &mut mesh.indices {
        for v in tri.iter_mut() {
            *v = remap[*v as usize];
        }
    }

    // Remove degenerate triangles after welding
    mesh.indices.retain(|tri| tri[0] != tri[1] && tri[1] != tri[2] && tri[0] != tri[2]);
    mesh.calculate_normals();
}

/// Flip all triangle normals (reverse winding)
pub fn flip_normals(mesh: &mut Mesh) {
    for tri in &mut mesh.indices {
        tri.swap(0, 2);
    }
    for n in &mut mesh.normals {
        *n = -*n;
    }
}

/// Compute connected components
pub fn connected_components(mesh: &Mesh) -> Vec<Vec<usize>> {
    let n = mesh.vertices.len();
    let mut uf: Vec<usize> = (0..n).collect();
    fn find(uf: &mut Vec<usize>, mut x: usize) -> usize {
        while uf[x] != x { uf[x] = uf[uf[x]]; x = uf[x]; }
        x
    }
    fn union(uf: &mut Vec<usize>, a: usize, b: usize) {
        let ra = find(uf, a);
        let rb = find(uf, b);
        if ra != rb { uf[ra] = rb; }
    }

    for tri in &mesh.indices {
        union(&mut uf, tri[0] as usize, tri[1] as usize);
        union(&mut uf, tri[1] as usize, tri[2] as usize);
    }

    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..n {
        groups.entry(find(&mut uf, i)).or_default().push(i);
    }
    groups.into_values().collect()
}

/// Ray-mesh intersection (returns distance, triangle index if hit)
pub fn ray_intersect(mesh: &Mesh, origin: &Point3<f64>, direction: &Vector3<f64>) -> Option<(f64, usize)> {
    let inv_dir = direction.normalize();
    let mut closest: Option<(f64, usize)> = None;

    for (ti, tri) in mesh.indices.iter().enumerate() {
        let v0 = mesh.vertices[tri[0] as usize];
        let v1 = mesh.vertices[tri[1] as usize];
        let v2 = mesh.vertices[tri[2] as usize];

        // Möller–Trumbore intersection
        let e1 = v1 - v0;
        let e2 = v2 - v0;
        let h = inv_dir.cross(&e2);
        let a = e1.dot(&h);
        if a.abs() < 1e-10 { continue; }

        let f = 1.0 / a;
        let s = origin - v0;
        let u = f * s.dot(&h);
        if !(0.0..=1.0).contains(&u) { continue; }

        let q = s.cross(&e1);
        let v = f * inv_dir.dot(&q);
        if v < 0.0 || u + v > 1.0 { continue; }

        let t = f * e2.dot(&q);
        if t > 1e-10 {
            if closest.is_none() || t < closest.unwrap().0 {
                closest = Some((t, ti));
            }
        }
    }

    closest
}
