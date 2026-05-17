//! Mesh topology analysis: manifold check, boundary edges, connected components

use std::collections::{HashMap, HashSet, VecDeque};
use nalgebra::Point3;
use crate::Mesh;

/// An edge defined by two vertex indices (ordered low, high)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Edge(pub u32, pub u32);

impl Edge {
    pub fn new(a: u32, b: u32) -> Self {
        if a < b { Edge(a, b) } else { Edge(b, a) }
    }
}

/// Build edge → face_count map
pub fn edge_face_count(mesh: &Mesh) -> HashMap<Edge, u32> {
    let mut map: HashMap<Edge, u32> = HashMap::new();
    for tri in &mesh.indices {
        let (a, b, c) = (tri[0], tri[1], tri[2]);
        *map.entry(Edge::new(a, b)).or_insert(0) += 1;
        *map.entry(Edge::new(b, c)).or_insert(0) += 1;
        *map.entry(Edge::new(c, a)).or_insert(0) += 1;
    }
    map
}

/// Returns true if the mesh is 2-manifold (every edge shared by exactly 2 faces)
pub fn is_manifold(mesh: &Mesh) -> bool {
    edge_face_count(mesh).values().all(|&c| c == 2)
}

/// Returns all boundary edges (edges with only 1 adjacent face)
pub fn boundary_edges(mesh: &Mesh) -> Vec<Edge> {
    edge_face_count(mesh).into_iter().filter(|&(_, c)| c == 1).map(|(e, _)| e).collect()
}

/// Returns boundary edge loops as ordered vertex index sequences
pub fn boundary_loops(mesh: &Mesh) -> Vec<Vec<u32>> {
    let b_edges = boundary_edges(mesh);
    if b_edges.is_empty() { return vec![]; }

    let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();
    for edge in &b_edges {
        adj.entry(edge.0).or_default().push(edge.1);
        adj.entry(edge.1).or_default().push(edge.0);
    }

    let mut visited: HashSet<u32> = HashSet::new();
    let mut loops: Vec<Vec<u32>> = Vec::new();

    for &start in adj.keys() {
        if visited.contains(&start) { continue; }
        let mut loop_verts = Vec::new();
        let mut current = start;
        let mut prev = u32::MAX;

        loop {
            if visited.contains(&current) { break; }
            visited.insert(current);
            loop_verts.push(current);

            let neighbors = adj.get(&current).cloned().unwrap_or_default();
            let next = neighbors.iter().copied().find(|&n| n != prev && !visited.contains(&n));
            match next {
                Some(n) => { prev = current; current = n; }
                None => break,
            }
        }

        if loop_verts.len() >= 3 {
            loops.push(loop_verts);
        }
    }

    loops
}

/// Find connected components — returns vec of face index sets
pub fn connected_components(mesh: &Mesh) -> Vec<Vec<usize>> {
    let n_faces = mesh.indices.len();
    if n_faces == 0 { return vec![]; }

    let mut edge_to_faces: HashMap<Edge, Vec<usize>> = HashMap::new();
    for (i, tri) in mesh.indices.iter().enumerate() {
        let (a, b, c) = (tri[0], tri[1], tri[2]);
        edge_to_faces.entry(Edge::new(a, b)).or_default().push(i);
        edge_to_faces.entry(Edge::new(b, c)).or_default().push(i);
        edge_to_faces.entry(Edge::new(c, a)).or_default().push(i);
    }

    let mut face_adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for (_, faces) in &edge_to_faces {
        if faces.len() == 2 {
            face_adj.entry(faces[0]).or_default().push(faces[1]);
            face_adj.entry(faces[1]).or_default().push(faces[0]);
        }
    }

    let mut visited = vec![false; n_faces];
    let mut components: Vec<Vec<usize>> = Vec::new();

    for start in 0..n_faces {
        if visited[start] { continue; }
        let mut component = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);
        visited[start] = true;

        while let Some(fi) = queue.pop_front() {
            component.push(fi);
            if let Some(neighbors) = face_adj.get(&fi) {
                for &nb in neighbors {
                    if !visited[nb] {
                        visited[nb] = true;
                        queue.push_back(nb);
                    }
                }
            }
        }
        components.push(component);
    }

    components
}

/// Extract a sub-mesh from a list of face indices
pub fn extract_submesh(mesh: &Mesh, face_indices: &[usize]) -> Mesh {
    let mut new_verts: Vec<Point3<f64>> = Vec::new();
    let mut new_indices: Vec<[u32; 3]> = Vec::new();
    let mut vert_map: HashMap<u32, u32> = HashMap::new();

    for &fi in face_indices {
        if fi >= mesh.indices.len() { continue; }
        let tri = mesh.indices[fi];
        let mut new_tri = [0u32; 3];
        for k in 0..3 {
            let old_idx = tri[k];
            let new_idx = *vert_map.entry(old_idx).or_insert_with(|| {
                let ni = new_verts.len() as u32;
                if (old_idx as usize) < mesh.vertices.len() {
                    new_verts.push(mesh.vertices[old_idx as usize]);
                }
                ni
            });
            new_tri[k] = new_idx;
        }
        new_indices.push(new_tri);
    }

    let mut sub = Mesh::new("submesh");
    sub.vertices = new_verts;
    sub.indices = new_indices;
    sub
}

/// Split mesh into separate meshes by connected component
pub fn split_by_component(mesh: &Mesh) -> Vec<Mesh> {
    connected_components(mesh)
        .iter()
        .map(|comp| extract_submesh(mesh, comp))
        .collect()
}

/// Count boundary edges (open edges) — quick manifold indicator
pub fn boundary_edge_count(mesh: &Mesh) -> usize {
    boundary_edges(mesh).len()
}

/// Check mesh has no degenerate faces (zero-area triangles)
pub fn degenerate_face_count(mesh: &Mesh) -> usize {
    let mut count = 0;
    for tri in &mesh.indices {
        let ia = tri[0] as usize;
        let ib = tri[1] as usize;
        let ic = tri[2] as usize;
        if ia >= mesh.vertices.len() || ib >= mesh.vertices.len() || ic >= mesh.vertices.len() { continue; }
        let v0 = &mesh.vertices[ia];
        let v1 = &mesh.vertices[ib];
        let v2 = &mesh.vertices[ic];
        if (v1 - v0).cross(&(v2 - v0)).norm() < 1e-10 { count += 1; }
    }
    count
}
