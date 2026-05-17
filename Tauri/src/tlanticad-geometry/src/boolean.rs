//! Boolean CSG (Constructive Solid Geometry) mesh operations
//! Implements Difference, Union, Intersection for triangle meshes.
//! Algorithm: face-classification approach using ray-casting inside/outside tests.

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use tlanticad_mesh::Mesh;

/// Type of Boolean operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BooleanOp {
    /// A minus B — used for cement space, hollowing
    Difference,
    /// A union B — used for framework joining
    Union,
    /// A intersect B — used for collision detection regions
    Intersection,
}

/// Result of a Boolean operation
#[derive(Debug, Clone)]
pub struct BooleanResult {
    pub mesh: Mesh,
    pub op: BooleanOp,
    pub a_triangles_kept: usize,
    pub b_triangles_kept: usize,
}

/// Axis-aligned bounding box for quick rejection
#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Point3<f64>,
    pub max: Point3<f64>,
}

impl Aabb {
    pub fn from_mesh(mesh: &Mesh) -> Self {
        let mut min = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
        for v in &mesh.vertices {
            min.x = min.x.min(v.x);
            min.y = min.y.min(v.y);
            min.z = min.z.min(v.z);
            max.x = max.x.max(v.x);
            max.y = max.y.max(v.y);
            max.z = max.z.max(v.z);
        }
        Aabb { min, max }
    }

    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    pub fn contains_point(&self, p: &Point3<f64>) -> bool {
        p.x >= self.min.x && p.x <= self.max.x &&
        p.y >= self.min.y && p.y <= self.max.y &&
        p.z >= self.min.z && p.z <= self.max.z
    }
}

/// Classify a point relative to a mesh: inside (true) or outside (false).
/// Uses ray casting along +Z axis, counts intersections with triangles.
pub fn point_inside_mesh(point: &Point3<f64>, mesh: &Mesh) -> bool {
    let aabb = Aabb::from_mesh(mesh);
    if !aabb.contains_point(point) {
        return false;
    }

    let ray_dir = Vector3::new(0.0, 0.0, 1.0);
    let mut crossings = 0u32;
    let verts = &mesh.vertices;

    for tri in &mesh.indices {
        let ia = tri[0] as usize;
        let ib = tri[1] as usize;
        let ic = tri[2] as usize;
        if ia >= verts.len() || ib >= verts.len() || ic >= verts.len() { continue; }

        if let Some(t) = ray_triangle_intersect_t(point, &ray_dir, &verts[ia], &verts[ib], &verts[ic]) {
            if t > 0.0 { crossings += 1; }
        }
    }
    crossings % 2 == 1
}

/// Möller–Trumbore ray-triangle intersection, returns t parameter
pub fn ray_triangle_intersect_t(
    origin: &Point3<f64>,
    dir: &Vector3<f64>,
    v0: &Point3<f64>,
    v1: &Point3<f64>,
    v2: &Point3<f64>,
) -> Option<f64> {
    const EPS: f64 = 1e-8;
    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let h = dir.cross(&e2);
    let a = e1.dot(&h);
    if a.abs() < EPS { return None; }
    let f = 1.0 / a;
    let s = origin - v0;
    let u = f * s.dot(&h);
    if !(0.0..=1.0).contains(&u) { return None; }
    let q = s.cross(&e1);
    let v = f * dir.dot(&q);
    if v < 0.0 || u + v > 1.0 { return None; }
    let t = f * e2.dot(&q);
    Some(t)
}

fn tri_centroid(v0: &Point3<f64>, v1: &Point3<f64>, v2: &Point3<f64>) -> Point3<f64> {
    Point3::new(
        (v0.x + v1.x + v2.x) / 3.0,
        (v0.y + v1.y + v2.y) / 3.0,
        (v0.z + v1.z + v2.z) / 3.0,
    )
}

/// Perform a Boolean operation on two meshes.
///
/// - DIFFERENCE (A - B): keep A triangles outside B, keep B triangles inside A (flipped)
/// - UNION (A + B): keep A triangles outside B, keep B triangles outside A
/// - INTERSECTION (A ∩ B): keep A triangles inside B, keep B triangles inside A
pub fn boolean_op(a: &Mesh, b: &Mesh, op: BooleanOp) -> BooleanResult {
    let aabb_a = Aabb::from_mesh(a);
    let aabb_b = Aabb::from_mesh(b);

    if !aabb_a.intersects(&aabb_b) {
        return match op {
            BooleanOp::Difference   => BooleanResult { mesh: a.clone(), op, a_triangles_kept: a.indices.len(), b_triangles_kept: 0 },
            BooleanOp::Union        => merge_meshes(a, b, op),
            BooleanOp::Intersection => BooleanResult { mesh: Mesh::new("intersection"), op, a_triangles_kept: 0, b_triangles_kept: 0 },
        };
    }

    let mut out_verts: Vec<Point3<f64>> = Vec::new();
    let mut out_indices: Vec<[u32; 3]> = Vec::new();
    let mut a_kept = 0;
    let mut b_kept = 0;

    // Process triangles from mesh A
    for tri in &a.indices {
        let ia = tri[0] as usize;
        let ib = tri[1] as usize;
        let ic = tri[2] as usize;
        if ia >= a.vertices.len() || ib >= a.vertices.len() || ic >= a.vertices.len() { continue; }

        let v0 = &a.vertices[ia];
        let v1 = &a.vertices[ib];
        let v2 = &a.vertices[ic];
        let centroid = tri_centroid(v0, v1, v2);
        let inside_b = point_inside_mesh(&centroid, b);

        let keep = match op {
            BooleanOp::Difference   => !inside_b,
            BooleanOp::Union        => !inside_b,
            BooleanOp::Intersection => inside_b,
        };

        if keep {
            let base = out_verts.len() as u32;
            out_verts.push(*v0);
            out_verts.push(*v1);
            out_verts.push(*v2);
            out_indices.push([base, base + 1, base + 2]);
            a_kept += 1;
        }
    }

    // Process triangles from mesh B
    for tri in &b.indices {
        let ia = tri[0] as usize;
        let ib = tri[1] as usize;
        let ic = tri[2] as usize;
        if ia >= b.vertices.len() || ib >= b.vertices.len() || ic >= b.vertices.len() { continue; }

        let v0 = &b.vertices[ia];
        let v1 = &b.vertices[ib];
        let v2 = &b.vertices[ic];
        let centroid = tri_centroid(v0, v1, v2);
        let inside_a = point_inside_mesh(&centroid, a);

        let keep = match op {
            BooleanOp::Difference   => inside_a,
            BooleanOp::Union        => !inside_a,
            BooleanOp::Intersection => inside_a,
        };

        if keep {
            let base = out_verts.len() as u32;
            if op == BooleanOp::Difference {
                out_verts.push(*v0);
                out_verts.push(*v2); // swapped for normal flip
                out_verts.push(*v1);
            } else {
                out_verts.push(*v0);
                out_verts.push(*v1);
                out_verts.push(*v2);
            }
            out_indices.push([base, base + 1, base + 2]);
            b_kept += 1;
        }
    }

    let mut result_mesh = Mesh::new("boolean_result");
    result_mesh.vertices = out_verts;
    result_mesh.indices = out_indices;
    compute_normals(&mut result_mesh);

    BooleanResult { mesh: result_mesh, op, a_triangles_kept: a_kept, b_triangles_kept: b_kept }
}

fn merge_meshes(a: &Mesh, b: &Mesh, op: BooleanOp) -> BooleanResult {
    let offset = a.vertices.len() as u32;
    let mut verts = a.vertices.clone();
    let mut indices = a.indices.clone();
    verts.extend_from_slice(&b.vertices);
    for tri in &b.indices {
        indices.push([tri[0] + offset, tri[1] + offset, tri[2] + offset]);
    }
    let mut mesh = Mesh::new("merged");
    mesh.vertices = verts;
    mesh.indices = indices;
    compute_normals(&mut mesh);
    BooleanResult { mesh, op, a_triangles_kept: a.indices.len(), b_triangles_kept: b.indices.len() }
}

/// Recompute per-vertex normals from face normals
pub fn compute_normals(mesh: &mut Mesh) {
    let n = mesh.vertices.len();
    let mut normals = vec![Vector3::zeros(); n];

    for tri in &mesh.indices {
        let ia = tri[0] as usize;
        let ib = tri[1] as usize;
        let ic = tri[2] as usize;
        if ia >= n || ib >= n || ic >= n { continue; }

        let v0 = &mesh.vertices[ia];
        let v1 = &mesh.vertices[ib];
        let v2 = &mesh.vertices[ic];
        let face_normal = (v1 - v0).cross(&(v2 - v0));
        normals[ia] += face_normal;
        normals[ib] += face_normal;
        normals[ic] += face_normal;
    }

    mesh.normals = normals.iter().map(|n| {
        let len = n.norm();
        if len > 1e-8 { n / len } else { Vector3::new(0.0, 0.0, 1.0) }
    }).collect();
}

/// Offset all vertices along their normals by `amount` mm
pub fn offset_mesh_by_normals(mesh: &Mesh, amount: f64) -> Mesh {
    let mut result = mesh.clone();
    if result.normals.is_empty() {
        let mut tmp = mesh.clone();
        compute_normals(&mut tmp);
        result.normals = tmp.normals;
    }
    for (i, v) in result.vertices.iter_mut().enumerate() {
        if let Some(n) = result.normals.get(i) {
            v.x += n.x * amount;
            v.y += n.y * amount;
            v.z += n.z * amount;
        }
    }
    result
}

/// Apply cement space by offsetting the preparation outward then subtracting.
/// Typical cement space: 0.08 mm
pub fn apply_cement_space(crown_inner: &Mesh, preparation: &Mesh, cement_mm: f64) -> Mesh {
    let offset_prep = offset_mesh_by_normals(preparation, cement_mm);
    boolean_op(crown_inner, &offset_prep, BooleanOp::Difference).mesh
}
