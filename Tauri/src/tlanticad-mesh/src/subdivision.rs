//! Loop subdivision for smooth mesh refinement

use std::collections::HashMap;
use nalgebra::Point3;
use crate::Mesh;
use super::topology::Edge;

/// Apply N iterations of Loop subdivision to a mesh
pub fn loop_subdivide(mesh: &Mesh, iterations: u32) -> Mesh {
    let mut current = mesh.clone();
    for _ in 0..iterations {
        current = loop_subdivide_once(&current);
    }
    current
}

fn loop_subdivide_once(mesh: &Mesh) -> Mesh {
    let n_verts = mesh.vertices.len();

    // Build edge → opposite vertex map (for Loop weights)
    let mut edge_to_opp: HashMap<Edge, Vec<u32>> = HashMap::new();
    for tri in &mesh.indices {
        let (a, b, c) = (tri[0], tri[1], tri[2]);
        let edges = [(Edge::new(a, b), c), (Edge::new(b, c), a), (Edge::new(c, a), b)];
        for (e, opp) in edges {
            edge_to_opp.entry(e).or_default().push(opp);
        }
    }

    // Step 1: compute edge midpoints
    let mut edge_vert: HashMap<Edge, u32> = HashMap::new();
    let mut new_verts: Vec<Point3<f64>> = mesh.vertices.clone();

    for (edge, opp_verts) in &edge_to_opp {
        let va = &mesh.vertices[edge.0 as usize];
        let vb = &mesh.vertices[edge.1 as usize];

        let mid = if opp_verts.len() == 2 {
            let vc = &mesh.vertices[opp_verts[0] as usize];
            let vd = &mesh.vertices[opp_verts[1] as usize];
            Point3::new(
                3.0/8.0 * (va.x + vb.x) + 1.0/8.0 * (vc.x + vd.x),
                3.0/8.0 * (va.y + vb.y) + 1.0/8.0 * (vc.y + vd.y),
                3.0/8.0 * (va.z + vb.z) + 1.0/8.0 * (vc.z + vd.z),
            )
        } else {
            Point3::new((va.x + vb.x) / 2.0, (va.y + vb.y) / 2.0, (va.z + vb.z) / 2.0)
        };

        let idx = new_verts.len() as u32;
        edge_vert.insert(*edge, idx);
        new_verts.push(mid);
    }

    // Step 2: update original vertex positions
    let mut valence = vec![0u32; n_verts];
    let mut neighbor_sum = vec![Point3::origin(); n_verts];
    let mut is_boundary = vec![false; n_verts];

    for (edge, opps) in &edge_to_opp {
        if opps.len() == 1 {
            is_boundary[edge.0 as usize] = true;
            is_boundary[edge.1 as usize] = true;
        }
    }

    for edge in edge_to_opp.keys() {
        let va = &mesh.vertices[edge.0 as usize];
        let vb = &mesh.vertices[edge.1 as usize];

        if !is_boundary[edge.0 as usize] {
            valence[edge.0 as usize] += 1;
            neighbor_sum[edge.0 as usize].coords += vb.coords;
        }
        if !is_boundary[edge.1 as usize] {
            valence[edge.1 as usize] += 1;
            neighbor_sum[edge.1 as usize].coords += va.coords;
        }
    }

    for i in 0..n_verts {
        if is_boundary[i] { continue; }
        let k = valence[i] as f64;
        if k < 1.0 { continue; }
        let beta = if k == 3.0 { 3.0/16.0 } else { 3.0 / (8.0 * k) };
        let v = &mesh.vertices[i];
        new_verts[i] = Point3::new(
            (1.0 - k * beta) * v.x + beta * neighbor_sum[i].x,
            (1.0 - k * beta) * v.y + beta * neighbor_sum[i].y,
            (1.0 - k * beta) * v.z + beta * neighbor_sum[i].z,
        );
    }

    // Step 3: generate 4 new triangles per old triangle
    let mut new_indices: Vec<[u32; 3]> = Vec::with_capacity(mesh.indices.len() * 4);

    for tri in &mesh.indices {
        let (a, b, c) = (tri[0], tri[1], tri[2]);
        let ab = *edge_vert.get(&Edge::new(a, b)).unwrap();
        let bc = *edge_vert.get(&Edge::new(b, c)).unwrap();
        let ca = *edge_vert.get(&Edge::new(c, a)).unwrap();

        new_indices.push([a, ab, ca]);
        new_indices.push([ab, b, bc]);
        new_indices.push([ca, bc, c]);
        new_indices.push([ab, bc, ca]);
    }

    let mut out = Mesh::new("subdivided");
    out.vertices = new_verts;
    out.indices = new_indices;
    out
}

/// Simple Laplacian smoothing (for post-boolean cleanup)
pub fn laplacian_smooth(mesh: &Mesh, iterations: u32, factor: f64) -> Mesh {
    let mut current = mesh.clone();
    let n = current.vertices.len();

    for _ in 0..iterations {
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        for tri in &current.indices {
            let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
            if !adj[a].contains(&b) { adj[a].push(b); }
            if !adj[a].contains(&c) { adj[a].push(c); }
            if !adj[b].contains(&a) { adj[b].push(a); }
            if !adj[b].contains(&c) { adj[b].push(c); }
            if !adj[c].contains(&a) { adj[c].push(a); }
            if !adj[c].contains(&b) { adj[c].push(b); }
        }

        let old = current.vertices.clone();
        for i in 0..n {
            if adj[i].is_empty() { continue; }
            let k = adj[i].len() as f64;
            let (mut cx, mut cy, mut cz) = (0.0f64, 0.0f64, 0.0f64);
            for &nb in &adj[i] {
                cx += old[nb].x;
                cy += old[nb].y;
                cz += old[nb].z;
            }
            cx /= k; cy /= k; cz /= k;
            current.vertices[i].x += factor * (cx - current.vertices[i].x);
            current.vertices[i].y += factor * (cy - current.vertices[i].y);
            current.vertices[i].z += factor * (cz - current.vertices[i].z);
        }
    }
    current
}
