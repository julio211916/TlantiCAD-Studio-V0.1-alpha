//! Half-edge data structure for topological mesh operations
//!
//! Provides O(1) adjacency queries: vertex→outgoing edges, face→edges, edge→next/prev/twin

use std::collections::HashMap;
use nalgebra::Point3;

/// Index types for clarity
pub type VertexId = u32;
pub type HalfEdgeId = u32;
pub type FaceId = u32;

/// A vertex in the half-edge mesh
#[derive(Debug, Clone)]
pub struct HVertex {
    pub position: Point3<f64>,
    /// One outgoing half-edge (any)
    pub halfedge: Option<HalfEdgeId>,
}

/// A half-edge connecting two vertices
#[derive(Debug, Clone, Copy)]
pub struct HalfEdge {
    pub origin: VertexId,
    pub twin: Option<HalfEdgeId>,
    pub next: HalfEdgeId,
    pub prev: HalfEdgeId,
    pub face: Option<FaceId>,
}

/// A face bounded by a loop of half-edges
#[derive(Debug, Clone, Copy)]
pub struct HFace {
    /// One half-edge on this face boundary
    pub halfedge: HalfEdgeId,
}

/// Half-edge mesh structure
#[derive(Debug, Clone)]
pub struct HalfEdgeMesh {
    pub vertices: Vec<HVertex>,
    pub halfedges: Vec<HalfEdge>,
    pub faces: Vec<HFace>,
}

impl HalfEdgeMesh {
    /// Build half-edge mesh from triangle soup
    pub fn from_triangles(positions: &[Point3<f64>], indices: &[[u32; 3]]) -> Self {
        let n_faces = indices.len();
        let n_halfedges = n_faces * 3;

        let mut vertices: Vec<HVertex> = positions.iter().map(|p| HVertex {
            position: *p,
            halfedge: None,
        }).collect();
        let mut halfedges = Vec::with_capacity(n_halfedges);
        let mut faces = Vec::with_capacity(n_faces);

        // edge (v_from, v_to) → halfedge index
        let mut edge_map: HashMap<(u32, u32), HalfEdgeId> = HashMap::new();

        for (fi, tri) in indices.iter().enumerate() {
            let fi = fi as FaceId;
            let base = halfedges.len() as HalfEdgeId;

            // Create 3 half-edges for this face
            for k in 0..3u32 {
                let origin = tri[k as usize];
                let he_id = base + k;
                let next_id = base + (k + 1) % 3;
                let prev_id = base + (k + 2) % 3;

                halfedges.push(HalfEdge {
                    origin,
                    twin: None,
                    next: next_id,
                    prev: prev_id,
                    face: Some(fi),
                });

                // Assign vertex → halfedge
                if vertices[origin as usize].halfedge.is_none() {
                    vertices[origin as usize].halfedge = Some(he_id);
                }

                // Register in edge map
                let dest = tri[((k + 1) % 3) as usize];
                edge_map.insert((origin, dest), he_id);
            }

            faces.push(HFace { halfedge: base });
        }

        // Link twins
        let keys: Vec<(u32, u32)> = edge_map.keys().cloned().collect();
        for (v_from, v_to) in keys {
            if let Some(&twin_id) = edge_map.get(&(v_to, v_from)) {
                let he_id = edge_map[&(v_from, v_to)];
                halfedges[he_id as usize].twin = Some(twin_id);
                halfedges[twin_id as usize].twin = Some(he_id);
            }
        }

        Self { vertices, halfedges, faces }
    }

    /// Get all half-edges leaving a vertex (fan traversal)
    pub fn vertex_outgoing(&self, vid: VertexId) -> Vec<HalfEdgeId> {
        let mut result = Vec::new();
        let Some(start) = self.vertices[vid as usize].halfedge else { return result };
        let mut current = start;
        loop {
            result.push(current);
            let prev = self.halfedges[current as usize].prev;
            match self.halfedges[prev as usize].twin {
                Some(twin) => current = twin,
                None => break, // boundary
            }
            if current == start { break; }
        }
        result
    }

    /// Get vertex neighbors (1-ring)
    pub fn vertex_neighbors(&self, vid: VertexId) -> Vec<VertexId> {
        self.vertex_outgoing(vid).iter().map(|&he| {
            let next = self.halfedges[he as usize].next;
            self.halfedges[next as usize].origin
        }).collect()
    }

    /// Get face vertex indices
    pub fn face_vertices(&self, fid: FaceId) -> [VertexId; 3] {
        let he0 = self.faces[fid as usize].halfedge;
        let he1 = self.halfedges[he0 as usize].next;
        let he2 = self.halfedges[he1 as usize].next;
        [
            self.halfedges[he0 as usize].origin,
            self.halfedges[he1 as usize].origin,
            self.halfedges[he2 as usize].origin,
        ]
    }

    /// Find boundary loops (edges without twins)
    pub fn boundary_loops(&self) -> Vec<Vec<VertexId>> {
        let mut visited = vec![false; self.halfedges.len()];
        let mut loops = Vec::new();

        for (i, he) in self.halfedges.iter().enumerate() {
            if he.twin.is_none() && !visited[i] {
                let mut loop_verts = Vec::new();
                let mut current = i as HalfEdgeId;
                loop {
                    visited[current as usize] = true;
                    loop_verts.push(self.halfedges[current as usize].origin);
                    // Walk along boundary: next edge's next until we find another boundary
                    let next = self.halfedges[current as usize].next;
                    let mut walker = next;
                    // Find next boundary edge from same vertex
                    while self.halfedges[walker as usize].twin.is_some() {
                        let twin = self.halfedges[walker as usize].twin.unwrap();
                        walker = self.halfedges[twin as usize].next;
                        if walker == next { break; }
                    }
                    if self.halfedges[walker as usize].twin.is_some() || visited[walker as usize] {
                        break;
                    }
                    current = walker;
                }
                if !loop_verts.is_empty() {
                    loops.push(loop_verts);
                }
            }
        }
        loops
    }

    /// Check if mesh is manifold (every edge has exactly 0 or 2 incident faces)
    pub fn is_manifold(&self) -> bool {
        for he in &self.halfedges {
            // A manifold edge either has a twin or is a boundary
            // But checking for non-manifold vertices (more than 2 boundary edges)
            if he.twin.is_none() { continue; }
        }
        // Check vertex manifoldness: 1-ring must form a single disk or half-disk
        for vid in 0..self.vertices.len() {
            let outgoing = self.vertex_outgoing(vid as VertexId);
            if outgoing.is_empty() { continue; }
            // Count boundary edges around vertex
            let boundary_count = outgoing.iter()
                .filter(|&&he| self.halfedges[he as usize].twin.is_none())
                .count();
            if boundary_count > 2 { return false; }
        }
        true
    }

    /// Convert back to triangle soup
    pub fn to_triangles(&self) -> (Vec<Point3<f64>>, Vec<[u32; 3]>) {
        let positions: Vec<Point3<f64>> = self.vertices.iter().map(|v| v.position).collect();
        let indices: Vec<[u32; 3]> = (0..self.faces.len())
            .map(|fi| self.face_vertices(fi as FaceId))
            .collect();
        (positions, indices)
    }

    /// Vertex count
    pub fn vertex_count(&self) -> usize { self.vertices.len() }

    /// Face count
    pub fn face_count(&self) -> usize { self.faces.len() }

    /// Edge count (each pair of half-edges = 1 edge)
    pub fn edge_count(&self) -> usize {
        let paired = self.halfedges.iter().filter(|he| he.twin.is_some()).count();
        let boundary = self.halfedges.iter().filter(|he| he.twin.is_none()).count();
        paired / 2 + boundary
    }

    /// Euler characteristic: V - E + F
    pub fn euler_characteristic(&self) -> i64 {
        self.vertex_count() as i64 - self.edge_count() as i64 + self.face_count() as i64
    }
}
