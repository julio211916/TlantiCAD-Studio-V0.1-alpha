//! TlantiCAD Mesh Processing Module
//! 
//! Operations: load, save, repair, decimate, smooth, boolean, etc.

// Core
pub mod mesh;
pub mod operations;
pub mod formats;
pub mod algorithms;

// Advanced structures
pub mod halfedge;
pub mod hole_fill;
pub mod remesh;
pub mod boolean;
pub mod thickness;
pub mod topology;
pub mod subdivision;

// AR-V363 — mesh kernel ops (region select, compare, adapt, add/remove)
pub mod region;
pub mod compare;
pub mod adapt;
pub mod add_remove;

// AR-V365 — margin processors
pub mod margin;

// AR-V418 — scan-data hygiene (drop islands, fill small holes)
pub mod scan_data;

// AR-V404 — generic visualization mesh tag conversions
pub mod visualization_convert;

pub use mesh::*;
pub use operations::*;
pub use formats::*;
pub use algorithms::*;
pub use topology::{Edge, is_manifold, boundary_edges, boundary_loops, connected_components, extract_submesh, split_by_component, boundary_edge_count, degenerate_face_count};
pub use subdivision::{loop_subdivide, laplacian_smooth};

// Re-export nalgebra so consumers (Tauri app) don't need a separate dep.
pub use nalgebra;

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_core::Id;

/// Mesh structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub id: Id,
    pub name: String,
    pub vertices: Vec<Point3<f64>>,
    pub normals: Vec<Vector3<f64>>,
    pub indices: Vec<[u32; 3]>,
    pub uvs: Option<Vec<[f64; 2]>>,
    pub colors: Option<Vec<[u8; 4]>>,
}

impl Mesh {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Id::new_v4(),
            name: name.into(),
            vertices: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
            uvs: None,
            colors: None,
        }
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len()
    }

    pub fn calculate_bounds(&self) -> (Point3<f64>, Point3<f64>) {
        let mut min = Point3::new(f64::MAX, f64::MAX, f64::MAX);
        let mut max = Point3::new(f64::MIN, f64::MIN, f64::MIN);

        for v in &self.vertices {
            min.x = min.x.min(v.x);
            min.y = min.y.min(v.y);
            min.z = min.z.min(v.z);
            max.x = max.x.max(v.x);
            max.y = max.y.max(v.y);
            max.z = max.z.max(v.z);
        }

        (min, max)
    }

    pub fn calculate_normals(&mut self) {
        self.normals = vec![Vector3::zeros(); self.vertices.len()];

        for tri in &self.indices {
            let v0 = self.vertices[tri[0] as usize];
            let v1 = self.vertices[tri[1] as usize];
            let v2 = self.vertices[tri[2] as usize];

            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let normal = edge1.cross(&edge2).normalize();

            for &idx in tri.iter() {
                self.normals[idx as usize] += normal;
            }
        }

        for n in &mut self.normals {
            *n = n.normalize();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Point3;

    #[test]
    fn test_mesh_new() {
        let m = Mesh::new("test");
        assert_eq!(m.name, "test");
        assert_eq!(m.vertex_count(), 0);
        assert_eq!(m.triangle_count(), 0);
    }

    #[test]
    fn test_mesh_builder() {
        let m = MeshBuilder::new("built")
            .vertex(Point3::origin())
            .vertex(Point3::new(1.0, 0.0, 0.0))
            .vertex(Point3::new(0.0, 1.0, 0.0))
            .triangle(0, 1, 2)
            .build();
        assert_eq!(m.vertex_count(), 3);
        assert_eq!(m.triangle_count(), 1);
    }

    #[test]
    fn test_create_box() {
        let b = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        assert_eq!(b.vertex_count(), 8);
        assert_eq!(b.triangle_count(), 12);
    }

    #[test]
    fn test_create_sphere() {
        let s = create_sphere(Point3::origin(), 1.0, 16, 16);
        assert!(s.vertex_count() > 50);
        assert!(s.triangle_count() > 50);
    }

    #[test]
    fn test_create_cylinder() {
        let c = create_cylinder(Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 2.0), 0.5, 16);
        assert!(c.vertex_count() > 10);
    }

    #[test]
    fn test_box_volume() {
        let b = create_box(Point3::origin(), Point3::new(2.0, 3.0, 4.0));
        let vol = volume(&b);
        assert!((vol - 24.0).abs() < 0.5);
    }

    #[test]
    fn test_box_surface_area() {
        let b = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let sa = surface_area(&b);
        assert!((sa - 6.0).abs() < 0.1);
    }

    #[test]
    fn test_box_is_watertight() {
        let b = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        assert!(is_watertight(&b));
    }

    #[test]
    fn test_boundary_edges_of_watertight() {
        let b = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let be = boundary_edges(&b);
        assert_eq!(be.len(), 0);
    }

    #[test]
    fn test_smooth_doesnt_panic() {
        let mut b = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        smooth(&mut b, 1, 0.5);
        assert!(b.vertex_count() > 0);
    }

    #[test]
    fn test_subdivide() {
        let mut b = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let before = b.triangle_count();
        subdivide(&mut b);
        assert!(b.triangle_count() > before);
    }

    #[test]
    fn test_offset() {
        let mut b = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        offset(&mut b, 0.1);
        assert!(b.vertex_count() > 0);
    }

    #[test]
    fn test_merge() {
        let a = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let b = create_box(Point3::new(2.0, 0.0, 0.0), Point3::new(3.0, 1.0, 1.0));
        let m = merge(&a, &b);
        assert_eq!(m.vertex_count(), a.vertex_count() + b.vertex_count());
    }

    #[test]
    fn test_connected_components_two_boxes() {
        let a = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        let b = create_box(Point3::new(5.0, 5.0, 5.0), Point3::new(6.0, 6.0, 6.0));
        let m = merge(&a, &b);
        let comps = connected_components(&m);
        assert_eq!(comps.len(), 2);
    }

    #[test]
    fn test_calculate_bounds() {
        let b = create_box(Point3::new(1.0, 2.0, 3.0), Point3::new(4.0, 5.0, 6.0));
        let (lo, hi) = b.calculate_bounds();
        assert!((lo.x - 1.0).abs() < 1e-6);
        assert!((hi.z - 6.0).abs() < 1e-6);
    }

    #[test]
    fn test_flip_normals() {
        let mut b = create_box(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
        b.calculate_normals();
        let n_before = b.normals[0];
        flip_normals(&mut b);
        let n_after = b.normals[0];
        assert!((n_before + n_after).norm() < 1e-3);
    }
}
