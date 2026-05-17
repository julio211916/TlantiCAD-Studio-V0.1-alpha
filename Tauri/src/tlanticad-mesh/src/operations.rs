//! Mesh operations: smooth, decimate, subdivide, offset, boolean-like merge

use nalgebra::{Point3, Vector3};
use crate::Mesh;
use std::collections::HashMap;

/// Laplacian smoothing
pub fn smooth(mesh: &mut Mesh, iterations: usize, factor: f64) {
    for _ in 0..iterations {
        let mut adjacency: HashMap<u32, Vec<u32>> = HashMap::new();
        for tri in &mesh.indices {
            for i in 0..3 {
                for j in 0..3 {
                    if i != j {
                        adjacency.entry(tri[i]).or_default().push(tri[j]);
                    }
                }
            }
        }

        let mut new_verts = mesh.vertices.clone();
        for (idx, neighbors) in &adjacency {
            if neighbors.is_empty() { continue; }
            let center: Vector3<f64> = neighbors.iter()
                .map(|&n| mesh.vertices[n as usize].coords)
                .sum::<Vector3<f64>>() / neighbors.len() as f64;
            let v = mesh.vertices[*idx as usize];
            new_verts[*idx as usize] = Point3::from(v.coords.lerp(&center, factor));
        }
        mesh.vertices = new_verts;
    }
    mesh.calculate_normals();
}

/// Edge-collapse decimation (greedy, reduces to target_ratio of original triangles)
pub fn decimate(mesh: &mut Mesh, target_ratio: f64) {
    let target = (mesh.indices.len() as f64 * target_ratio.clamp(0.01, 1.0)) as usize;
    if mesh.indices.len() <= target { return; }

    // Build edge cost map (shortest edges first)
    let mut edges: Vec<(f64, u32, u32)> = Vec::new();
    for tri in &mesh.indices {
        for i in 0..3 {
            let a = tri[i];
            let b = tri[(i + 1) % 3];
            if a < b {
                let cost = (mesh.vertices[a as usize] - mesh.vertices[b as usize]).norm();
                edges.push((cost, a, b));
            }
        }
    }
    edges.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    edges.dedup_by(|a, b| a.1 == b.1 && a.2 == b.2);

    let mut remap: Vec<u32> = (0..mesh.vertices.len() as u32).collect();
    fn find(remap: &[u32], mut v: u32) -> u32 {
        while remap[v as usize] != v { v = remap[v as usize]; }
        v
    }

    for (_, a, b) in &edges {
        if mesh.indices.len() <= target { break; }
        let fa = find(&remap, *a);
        let fb = find(&remap, *b);
        if fa == fb { continue; }

        // Collapse b into a
        let mid = Point3::from((mesh.vertices[fa as usize].coords + mesh.vertices[fb as usize].coords) * 0.5);
        mesh.vertices[fa as usize] = mid;
        remap[fb as usize] = fa;

        // Remove degenerate triangles
        mesh.indices.retain(|tri| {
            let t: [u32; 3] = [find(&remap, tri[0]), find(&remap, tri[1]), find(&remap, tri[2])];
            t[0] != t[1] && t[1] != t[2] && t[0] != t[2]
        });
    }

    // Remap indices
    for tri in &mut mesh.indices {
        for v in tri.iter_mut() {
            *v = find(&remap, *v);
        }
    }

    mesh.calculate_normals();
}

/// Loop subdivision (one iteration)
pub fn subdivide(mesh: &mut Mesh) {
    let mut new_vertices = mesh.vertices.clone();
    let mut new_indices = Vec::new();
    let mut edge_midpoints: HashMap<(u32, u32), u32> = HashMap::new();

    let get_mid = |a: u32, b: u32, verts: &mut Vec<Point3<f64>>, map: &mut HashMap<(u32, u32), u32>| -> u32 {
        let key = if a < b { (a, b) } else { (b, a) };
        if let Some(&idx) = map.get(&key) {
            return idx;
        }
        let mid = Point3::from((verts[a as usize].coords + verts[b as usize].coords) * 0.5);
        let idx = verts.len() as u32;
        verts.push(mid);
        map.insert(key, idx);
        idx
    };

    for tri in &mesh.indices {
        let m01 = get_mid(tri[0], tri[1], &mut new_vertices, &mut edge_midpoints);
        let m12 = get_mid(tri[1], tri[2], &mut new_vertices, &mut edge_midpoints);
        let m20 = get_mid(tri[2], tri[0], &mut new_vertices, &mut edge_midpoints);
        new_indices.push([tri[0], m01, m20]);
        new_indices.push([tri[1], m12, m01]);
        new_indices.push([tri[2], m20, m12]);
        new_indices.push([m01, m12, m20]);
    }

    mesh.vertices = new_vertices;
    mesh.indices = new_indices;
    mesh.calculate_normals();
}

/// Offset mesh along normals by distance
pub fn offset(mesh: &mut Mesh, distance: f64) {
    mesh.calculate_normals();
    for i in 0..mesh.vertices.len() {
        mesh.vertices[i] += mesh.normals[i] * distance;
    }
    if distance < 0.0 {
        // Flip winding order for negative offset
        for tri in &mut mesh.indices {
            tri.swap(0, 2);
        }
    }
    mesh.calculate_normals();
}

/// Merge two meshes into one
pub fn merge(a: &Mesh, b: &Mesh) -> Mesh {
    let offset = a.vertices.len() as u32;
    let mut result = a.clone();
    result.name = format!("{}+{}", a.name, b.name);
    result.vertices.extend_from_slice(&b.vertices);
    result.normals.extend_from_slice(&b.normals);
    for tri in &b.indices {
        result.indices.push([tri[0] + offset, tri[1] + offset, tri[2] + offset]);
    }
    result
}

/// Compute mesh volume (assumes watertight)
pub fn volume(mesh: &Mesh) -> f64 {
    let mut vol = 0.0;
    for tri in &mesh.indices {
        let v0 = mesh.vertices[tri[0] as usize];
        let v1 = mesh.vertices[tri[1] as usize];
        let v2 = mesh.vertices[tri[2] as usize];
        vol += v0.coords.dot(&v1.coords.cross(&v2.coords));
    }
    vol.abs() / 6.0
}

/// Compute mesh surface area
pub fn surface_area(mesh: &Mesh) -> f64 {
    mesh.indices.iter().map(|tri| {
        let v0 = mesh.vertices[tri[0] as usize];
        let v1 = mesh.vertices[tri[1] as usize];
        let v2 = mesh.vertices[tri[2] as usize];
        (v1 - v0).cross(&(v2 - v0)).norm() * 0.5
    }).sum()
}
