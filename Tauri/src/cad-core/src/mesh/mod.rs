use crate::math::{Point3f, Vec3f};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type MeshId = Uuid;

/// GPU-ready vertex layout: position + normal packed into [f32; 6]
/// `bytemuck::Pod` allows safe `&[GpuVertex]` → `&[u8]` cast for wgpu buffers.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuVertex {
    pub position: [f32; 3],
    pub normal:   [f32; 3],
}

/// A vertex in the half-edge mesh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex {
    pub position: Point3f,
    pub normal: Vec3f,
    /// Index of one outgoing half-edge from this vertex
    pub half_edge: usize,
}

/// A triangular face
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Face {
    /// Index of one half-edge bounding this face
    pub half_edge: usize,
}

/// A directed half-edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalfEdge {
    pub vertex: usize,    // origin vertex
    pub face: usize,      // face this belongs to (usize::MAX for boundary)
    pub next: usize,      // next half-edge in the same face
    pub prev: usize,      // prev half-edge in the same face
    pub twin: usize,      // opposite half-edge (usize::MAX if boundary)
}

/// Half-Edge triangle mesh
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeMesh {
    pub vertices: Vec<Vertex>,
    pub faces: Vec<Face>,
    pub half_edges: Vec<HalfEdge>,
}

impl HeMesh {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from flat verts + triangle index list
    pub fn from_triangles(positions: &[Point3f], indices: &[[u32; 3]]) -> Self {
        let _n_verts = positions.len();
        let n_faces = indices.len();

        let mut vertices: Vec<Vertex> = positions
            .iter()
            .map(|&p| Vertex {
                position: p,
                normal: Vec3f::zeros(),
                half_edge: 0,
            })
            .collect();

        let mut faces: Vec<Face> = Vec::with_capacity(n_faces);
        let mut half_edges: Vec<HalfEdge> = Vec::with_capacity(n_faces * 3);

        // edge_map: (v_from, v_to) → half_edge index (for twin lookup)
        let mut edge_map: std::collections::HashMap<(usize, usize), usize> =
            std::collections::HashMap::with_capacity(n_faces * 6);

        for tri in indices {
            let he_base = half_edges.len();
            let face_idx = faces.len();

            for i in 0..3usize {
                let v_from = tri[i] as usize;
                let v_to = tri[(i + 1) % 3] as usize;
                let he_idx = he_base + i;

                half_edges.push(HalfEdge {
                    vertex: v_from,
                    face: face_idx,
                    next: he_base + (i + 1) % 3,
                    prev: he_base + (i + 2) % 3,
                    twin: usize::MAX,
                });

                vertices[v_from].half_edge = he_idx;
                edge_map.insert((v_from, v_to), he_idx);
            }

            faces.push(Face { half_edge: he_base });
        }

        // Set twins
        let n_he = half_edges.len();
        for i in 0..n_he {
            if half_edges[i].twin == usize::MAX {
                let v_from = half_edges[i].vertex;
                let v_to = half_edges[half_edges[i].next].vertex;
                if let Some(&twin_idx) = edge_map.get(&(v_to, v_from)) {
                    half_edges[i].twin = twin_idx;
                    half_edges[twin_idx].twin = i;
                }
            }
        }

        // Compute face normals and accumulate to vertices
        for face in &faces {
            let he0 = face.half_edge;
            let he1 = half_edges[he0].next;
            let he2 = half_edges[he1].next;
            let v0 = half_edges[he0].vertex;
            let v1 = half_edges[he1].vertex;
            let v2 = half_edges[he2].vertex;
            let p0 = vertices[v0].position;
            let p1 = vertices[v1].position;
            let p2 = vertices[v2].position;
            let n = (p1 - p0).cross(&(p2 - p0));
            vertices[v0].normal += n;
            vertices[v1].normal += n;
            vertices[v2].normal += n;
        }
        for v in &mut vertices {
            let len = v.normal.norm();
            if len > 1e-6 {
                v.normal /= len;
            }
        }

        Self { vertices, faces, half_edges }
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    /// Axis-aligned bounding box of this mesh
    pub fn aabb(&self) -> crate::math::Aabb {
        let mut bb = crate::math::Aabb::empty();
        for v in &self.vertices {
            bb.expand(v.position);
        }
        bb
    }

    /// Returns indices [v0, v1, v2] for a given face
    pub fn face_vertex_indices(&self, face_idx: usize) -> [usize; 3] {
        let he0 = self.faces[face_idx].half_edge;
        let he1 = self.half_edges[he0].next;
        let he2 = self.half_edges[he1].next;
        [
            self.half_edges[he0].vertex,
            self.half_edges[he1].vertex,
            self.half_edges[he2].vertex,
        ]
    }

    /// Flat vertex buffer for GPU upload: [x,y,z, nx,ny,nz] per vertex
    pub fn to_vertex_buffer(&self) -> Vec<f32> {
        let mut buf = Vec::with_capacity(self.vertices.len() * 6);
        for v in &self.vertices {
            buf.extend_from_slice(&[
                v.position.x, v.position.y, v.position.z,
                v.normal.x, v.normal.y, v.normal.z,
            ]);
        }
        buf
    }

    /// Typed GPU vertex buffer — can be cast directly to `&[u8]` with bytemuck
    pub fn to_gpu_vertices(&self) -> Vec<GpuVertex> {
        self.vertices
            .iter()
            .map(|v| GpuVertex {
                position: [v.position.x, v.position.y, v.position.z],
                normal:   [v.normal.x,   v.normal.y,   v.normal.z],
            })
            .collect()
    }

    /// Flat index buffer (u32) for GPU upload
    pub fn to_index_buffer(&self) -> Vec<u32> {
        let mut buf = Vec::with_capacity(self.faces.len() * 3);
        for f_idx in 0..self.faces.len() {
            let [v0, v1, v2] = self.face_vertex_indices(f_idx);
            buf.push(v0 as u32);
            buf.push(v1 as u32);
            buf.push(v2 as u32);
        }
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Point3;

    fn simple_triangle() -> HeMesh {
        let positions = vec![
            Point3::new(0.0f32, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ];
        HeMesh::from_triangles(&positions, &[[0, 1, 2]])
    }

    #[test]
    fn test_mesh_creation() {
        let mesh = simple_triangle();
        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.face_count(), 1);
        assert_eq!(mesh.half_edges.len(), 3);
    }

    #[test]
    fn test_vertex_buffers() {
        let mesh = simple_triangle();
        assert_eq!(mesh.to_vertex_buffer().len(), 3 * 6);
        assert_eq!(mesh.to_index_buffer().len(), 3);
    }
}
