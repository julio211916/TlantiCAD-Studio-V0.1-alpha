//! Extended mesh operations and builders

use nalgebra::{Point3, Vector3};
use crate::Mesh;

/// Builder pattern for constructing meshes
pub struct MeshBuilder {
    name: String,
    vertices: Vec<Point3<f64>>,
    normals: Vec<Vector3<f64>>,
    indices: Vec<[u32; 3]>,
}

impl MeshBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            vertices: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn vertex(mut self, p: Point3<f64>) -> Self {
        self.vertices.push(p);
        self
    }

    pub fn vertices(mut self, pts: impl IntoIterator<Item = Point3<f64>>) -> Self {
        self.vertices.extend(pts);
        self
    }

    pub fn triangle(mut self, a: u32, b: u32, c: u32) -> Self {
        self.indices.push([a, b, c]);
        self
    }

    pub fn build(self) -> Mesh {
        let mut m = Mesh::new(self.name);
        m.vertices = self.vertices;
        m.normals = self.normals;
        m.indices = self.indices;
        m.calculate_normals();
        m
    }
}

/// Generate a box mesh
pub fn create_box(min: Point3<f64>, max: Point3<f64>) -> Mesh {
    let v = [
        Point3::new(min.x, min.y, min.z), Point3::new(max.x, min.y, min.z),
        Point3::new(max.x, max.y, min.z), Point3::new(min.x, max.y, min.z),
        Point3::new(min.x, min.y, max.z), Point3::new(max.x, min.y, max.z),
        Point3::new(max.x, max.y, max.z), Point3::new(min.x, max.y, max.z),
    ];
    let indices = vec![
        [0,1,2],[0,2,3], // front
        [4,6,5],[4,7,6], // back
        [0,4,5],[0,5,1], // bottom
        [2,6,7],[2,7,3], // top
        [0,3,7],[0,7,4], // left
        [1,5,6],[1,6,2], // right
    ];
    let mut m = Mesh::new("box");
    m.vertices = v.to_vec();
    m.indices = indices;
    m.calculate_normals();
    m
}

/// Generate a sphere mesh via UV-sphere
pub fn create_sphere(center: Point3<f64>, radius: f64, segments: u32, rings: u32) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for j in 0..=rings {
        let phi = std::f64::consts::PI * j as f64 / rings as f64;
        for i in 0..=segments {
            let theta = std::f64::consts::TAU * i as f64 / segments as f64;
            let x = center.x + radius * phi.sin() * theta.cos();
            let y = center.y + radius * phi.cos();
            let z = center.z + radius * phi.sin() * theta.sin();
            vertices.push(Point3::new(x, y, z));
        }
    }

    for j in 0..rings {
        for i in 0..segments {
            let a = j * (segments + 1) + i;
            let b = a + segments + 1;
            indices.push([a, b, a + 1]);
            indices.push([a + 1, b, b + 1]);
        }
    }

    let mut m = Mesh::new("sphere");
    m.vertices = vertices;
    m.indices = indices;
    m.calculate_normals();
    m
}

/// Generate a cylinder mesh
pub fn create_cylinder(p1: Point3<f64>, p2: Point3<f64>, radius: f64, segments: u32) -> Mesh {
    let axis = (p2 - p1).normalize();
    let up = if axis.x.abs() < 0.9 { Vector3::x() } else { Vector3::y() };
    let u = axis.cross(&up).normalize();
    let v = axis.cross(&u).normalize();

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Bottom cap center
    vertices.push(p1);
    // Top cap center
    vertices.push(p2);

    for i in 0..=segments {
        let theta = std::f64::consts::TAU * i as f64 / segments as f64;
        let offset = u * radius * theta.cos() + v * radius * theta.sin();
        vertices.push(p1 + offset); // bottom ring: index 2+i
        vertices.push(p2 + offset); // top ring: index 3+i
    }

    // Side faces
    for i in 0..segments {
        let b0 = 2 + i * 2;
        let b1 = 2 + (i + 1) * 2;
        let t0 = b0 + 1;
        let t1 = b1 + 1;
        indices.push([b0, b1, t0]);
        indices.push([t0, b1, t1]);
    }

    // Bottom cap
    for i in 0..segments {
        let b0 = 2 + i * 2;
        let b1 = 2 + (i + 1) * 2;
        indices.push([0, b1, b0]);
    }

    // Top cap
    for i in 0..segments {
        let t0 = 3 + i * 2;
        let t1 = 3 + (i + 1) * 2;
        indices.push([1, t0, t1]);
    }

    let mut m = Mesh::new("cylinder");
    m.vertices = vertices;
    m.indices = indices;
    m.calculate_normals();
    m
}
