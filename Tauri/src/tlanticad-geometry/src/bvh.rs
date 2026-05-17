//! Bounding Volume Hierarchy for O(log n) ray casting and spatial queries

use nalgebra::{Point3, Vector3};
use crate::bbox::Aabb;

/// BVH node: either a leaf with triangle indices, or an internal node with children
#[derive(Debug)]
enum BvhNode {
    Leaf {
        bounds: Aabb,
        triangles: Vec<u32>,
    },
    Internal {
        bounds: Aabb,
        left: Box<BvhNode>,
        right: Box<BvhNode>,
    },
}

/// Bounding Volume Hierarchy for triangle meshes
#[derive(Debug)]
pub struct Bvh {
    root: Option<BvhNode>,
}

/// Ray-triangle hit result
#[derive(Debug, Clone, Copy)]
pub struct BvhHit {
    pub triangle_index: u32,
    pub t: f64,
    pub u: f64,
    pub v: f64,
}

impl Bvh {
    /// Build BVH from triangle mesh data
    /// vertices: point positions, indices: triangle triplets
    pub fn build(vertices: &[Point3<f64>], indices: &[[u32; 3]]) -> Self {
        if indices.is_empty() {
            return Self { root: None };
        }
        let tri_ids: Vec<u32> = (0..indices.len() as u32).collect();
        let centroids: Vec<Point3<f64>> = indices.iter().map(|tri| {
            let a = &vertices[tri[0] as usize];
            let b = &vertices[tri[1] as usize];
            let c = &vertices[tri[2] as usize];
            Point3::new(
                (a.x + b.x + c.x) / 3.0,
                (a.y + b.y + c.y) / 3.0,
                (a.z + b.z + c.z) / 3.0,
            )
        }).collect();

        let root = Self::build_recursive(vertices, indices, tri_ids, &centroids, 0);
        Self { root: Some(root) }
    }

    fn build_recursive(
        vertices: &[Point3<f64>],
        indices: &[[u32; 3]],
        mut tri_ids: Vec<u32>,
        centroids: &[Point3<f64>],
        depth: u32,
    ) -> BvhNode {
        // Compute bounds for all triangles
        let bounds = Self::compute_bounds(vertices, indices, &tri_ids);

        // Leaf threshold
        if tri_ids.len() <= 4 || depth > 40 {
            return BvhNode::Leaf { bounds, triangles: tri_ids };
        }

        // Split along longest axis using median centroid
        let axis = bounds.longest_axis();
        tri_ids.sort_by(|&a, &b| {
            let ca = centroids[a as usize][axis];
            let cb = centroids[b as usize][axis];
            ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mid = tri_ids.len() / 2;
        let right_ids = tri_ids.split_off(mid);
        let left_ids = tri_ids;

        let left = Self::build_recursive(vertices, indices, left_ids, centroids, depth + 1);
        let right = Self::build_recursive(vertices, indices, right_ids, centroids, depth + 1);

        BvhNode::Internal {
            bounds,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn compute_bounds(vertices: &[Point3<f64>], indices: &[[u32; 3]], tri_ids: &[u32]) -> Aabb {
        let mut min = Point3::new(f64::MAX, f64::MAX, f64::MAX);
        let mut max = Point3::new(f64::MIN, f64::MIN, f64::MIN);
        for &ti in tri_ids {
            let tri = &indices[ti as usize];
            for &vi in tri {
                let v = &vertices[vi as usize];
                min.x = min.x.min(v.x); min.y = min.y.min(v.y); min.z = min.z.min(v.z);
                max.x = max.x.max(v.x); max.y = max.y.max(v.y); max.z = max.z.max(v.z);
            }
        }
        Aabb::new(min, max)
    }

    /// Find closest ray intersection
    pub fn ray_cast(
        &self,
        vertices: &[Point3<f64>],
        indices: &[[u32; 3]],
        origin: &Point3<f64>,
        direction: &Vector3<f64>,
    ) -> Option<BvhHit> {
        let Some(ref root) = self.root else { return None };
        let dir_norm = direction.normalize();
        Self::ray_cast_node(root, vertices, indices, origin, &dir_norm, f64::MAX)
    }

    fn ray_cast_node(
        node: &BvhNode,
        vertices: &[Point3<f64>],
        indices: &[[u32; 3]],
        origin: &Point3<f64>,
        dir: &Vector3<f64>,
        mut best_t: f64,
    ) -> Option<BvhHit> {
        match node {
            BvhNode::Leaf { bounds, triangles } => {
                if bounds.ray_intersect(origin, dir).is_none() {
                    return None;
                }
                let mut best_hit: Option<BvhHit> = None;
                for &ti in triangles {
                    let tri = &indices[ti as usize];
                    let a = &vertices[tri[0] as usize];
                    let b = &vertices[tri[1] as usize];
                    let c = &vertices[tri[2] as usize];
                    if let Some((t, u, v)) = ray_triangle_intersect(origin, dir, a, b, c) {
                        if t < best_t && t >= 0.0 {
                            best_t = t;
                            best_hit = Some(BvhHit { triangle_index: ti, t, u, v });
                        }
                    }
                }
                best_hit
            }
            BvhNode::Internal { bounds, left, right } => {
                if bounds.ray_intersect(origin, dir).is_none() {
                    return None;
                }
                let hit_left = Self::ray_cast_node(left, vertices, indices, origin, dir, best_t);
                if let Some(ref h) = hit_left { best_t = h.t; }
                let hit_right = Self::ray_cast_node(right, vertices, indices, origin, dir, best_t);
                match (hit_left, hit_right) {
                    (Some(l), Some(r)) => Some(if l.t < r.t { l } else { r }),
                    (Some(h), None) | (None, Some(h)) => Some(h),
                    (None, None) => None,
                }
            }
        }
    }

    /// Find all triangles whose AABB intersects the query box
    pub fn query_aabb(
        &self,
        vertices: &[Point3<f64>],
        indices: &[[u32; 3]],
        query: &Aabb,
    ) -> Vec<u32> {
        let mut result = Vec::new();
        if let Some(ref root) = self.root {
            Self::query_aabb_node(root, vertices, indices, query, &mut result);
        }
        result
    }

    fn query_aabb_node(
        node: &BvhNode,
        _vertices: &[Point3<f64>],
        _indices: &[[u32; 3]],
        query: &Aabb,
        result: &mut Vec<u32>,
    ) {
        match node {
            BvhNode::Leaf { bounds, triangles } => {
                if bounds.intersects(query) {
                    result.extend_from_slice(triangles);
                }
            }
            BvhNode::Internal { bounds, left, right } => {
                if !bounds.intersects(query) { return; }
                Self::query_aabb_node(left, _vertices, _indices, query, result);
                Self::query_aabb_node(right, _vertices, _indices, query, result);
            }
        }
    }
}

/// Möller–Trumbore ray-triangle intersection
/// Returns (t, u, v) where u, v are barycentric coordinates
fn ray_triangle_intersect(
    origin: &Point3<f64>,
    dir: &Vector3<f64>,
    a: &Point3<f64>,
    b: &Point3<f64>,
    c: &Point3<f64>,
) -> Option<(f64, f64, f64)> {
    let edge1 = b - a;
    let edge2 = c - a;
    let h = dir.cross(&edge2);
    let det = edge1.dot(&h);

    if det.abs() < 1e-12 { return None; }
    let inv_det = 1.0 / det;

    let s = origin - a;
    let u = inv_det * s.dot(&h);
    if !(0.0..=1.0).contains(&u) { return None; }

    let q = s.cross(&edge1);
    let v = inv_det * dir.dot(&q);
    if v < 0.0 || u + v > 1.0 { return None; }

    let t = inv_det * edge2.dot(&q);
    if t > 1e-12 { Some((t, u, v)) } else { None }
}
