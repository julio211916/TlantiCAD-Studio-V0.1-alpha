//! Raycasting-based mesh picker for 3D viewport selection

use nalgebra::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tlanticad_mesh::Mesh;

/// A ray in 3D space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ray {
    pub origin: Point3<f64>,
    pub direction: Vector3<f64>,
}

impl Ray {
    /// Construct a world-space ray from screen coordinates and a combined view-projection matrix.
    ///
    /// `x`, `y` are in pixels; `width` and `height` are the viewport dimensions;
    /// `view_proj` is the 4×4 view-projection matrix (row-major).
    pub fn from_screen(
        x: f32,
        y: f32,
        width: u32,
        height: u32,
        view_proj: &[[f32; 4]; 4],
    ) -> Ray {
        // Normalize device coordinates
        let ndc_x = (2.0 * x / width as f32 - 1.0) as f64;
        let ndc_y = (1.0 - 2.0 * y / height as f32) as f64;

        // Invert the view-projection matrix (approximate for picking)
        let vp = nalgebra::Matrix4::from_fn(|r, c| view_proj[r][c] as f64);
        let inv = match vp.try_inverse() {
            Some(m) => m,
            None => nalgebra::Matrix4::identity(),
        };

        // Near and far clip points in NDC
        let near_h = inv * nalgebra::Vector4::new(ndc_x, ndc_y, -1.0, 1.0);
        let far_h = inv * nalgebra::Vector4::new(ndc_x, ndc_y, 1.0, 1.0);

        let near = Point3::new(
            near_h.x / near_h.w,
            near_h.y / near_h.w,
            near_h.z / near_h.w,
        );
        let far = Point3::new(
            far_h.x / far_h.w,
            far_h.y / far_h.w,
            far_h.z / far_h.w,
        );

        Ray {
            origin: near,
            direction: (far - near).normalize(),
        }
    }
}

/// Result of a pick operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickResult {
    pub hit: bool,
    pub distance: f64,
    pub point: Point3<f64>,
    pub triangle_idx: usize,
    pub mesh_id: Uuid,
}

/// Möller–Trumbore ray-triangle intersection.
///
/// Returns the parametric distance `t` along the ray, or `None` if no hit.
pub fn ray_triangle_intersect(
    ray: &Ray,
    v0: &Point3<f64>,
    v1: &Point3<f64>,
    v2: &Point3<f64>,
) -> Option<f64> {
    const EPSILON: f64 = 1e-10;

    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    let h = ray.direction.cross(&edge2);
    let a = edge1.dot(&h);

    if a.abs() < EPSILON {
        return None; // Ray is parallel to triangle
    }

    let f = 1.0 / a;
    let s = ray.origin - v0;
    let u = f * s.dot(&h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = s.cross(&edge1);
    let v = f * ray.direction.dot(&q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = f * edge2.dot(&q);
    if t > EPSILON {
        Some(t)
    } else {
        None
    }
}

/// Cast a ray against a mesh and return the closest hit.
pub fn pick_mesh(ray: &Ray, mesh: &Mesh, mesh_id: Uuid) -> Option<PickResult> {
    let mut closest_t = f64::MAX;
    let mut closest_tri = 0usize;

    for (tri_idx, tri) in mesh.indices.iter().enumerate() {
        let v0 = &mesh.vertices[tri[0] as usize];
        let v1 = &mesh.vertices[tri[1] as usize];
        let v2 = &mesh.vertices[tri[2] as usize];

        if let Some(t) = ray_triangle_intersect(ray, v0, v1, v2) {
            if t < closest_t {
                closest_t = t;
                closest_tri = tri_idx;
            }
        }
    }

    if closest_t < f64::MAX {
        let hit_point = ray.origin + ray.direction * closest_t;
        Some(PickResult {
            hit: true,
            distance: closest_t,
            point: hit_point,
            triangle_idx: closest_tri,
            mesh_id,
        })
    } else {
        None
    }
}
